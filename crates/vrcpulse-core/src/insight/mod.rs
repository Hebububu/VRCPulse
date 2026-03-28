pub mod feature_extractor;
pub mod gemini_client;

use std::time::Duration;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::entity::ai_insights;
use feature_extractor::{FeatureSnapshot, compute_source_hash};
#[allow(unused_imports)]
use gemini_client::InsightResponse;
use gemini_client::{GeminiClient, InsightError};

const ANALYSIS_INTERVAL_SECS: u64 = 3600; // 1 hour
const MODEL_ID: &str = "gemini-2.5-flash";

/// Trigger types for the analysis task.
#[derive(Debug, Clone)]
pub enum InsightTrigger {
    Scheduled,
    IncidentDetected { incident_id: String, title: String },
    MaintenanceDetected { maintenance_id: String },
}

impl InsightTrigger {
    fn trigger_type(&self) -> &str {
        match self {
            InsightTrigger::Scheduled => "scheduled",
            InsightTrigger::IncidentDetected { .. } => "incident_detected",
            InsightTrigger::MaintenanceDetected { .. } => "maintenance_detected",
        }
    }

    fn trigger_id(&self) -> Option<&str> {
        match self {
            InsightTrigger::Scheduled => None,
            InsightTrigger::IncidentDetected { incident_id, .. } => Some(incident_id),
            InsightTrigger::MaintenanceDetected { maintenance_id, .. } => Some(maintenance_id),
        }
    }

    fn scope(&self) -> &str {
        match self {
            InsightTrigger::Scheduled => "hourly",
            InsightTrigger::IncidentDetected { .. } => "incident",
            InsightTrigger::MaintenanceDetected { .. } => "maintenance",
        }
    }
}

/// Run the analysis task as a long-lived background worker.
/// Receives events from the collector via mpsc channel and runs analysis
/// on a 1-hour timer or when triggered by events.
pub async fn run_analysis_task(
    db: DatabaseConnection,
    api_key: String,
    mut event_rx: mpsc::Receiver<InsightTrigger>,
) {
    let http_client = reqwest::Client::builder()
        .user_agent("vrcpulse-insight/1.0.0")
        .build()
        .expect("Failed to create HTTP client for insight");

    let client = GeminiClient::new(http_client, api_key, MODEL_ID);
    let mut ticker = interval(Duration::from_secs(ANALYSIS_INTERVAL_SECS));

    info!(
        "AI insight analysis task started (interval: {}s)",
        ANALYSIS_INTERVAL_SECS
    );

    // Initial analysis: retry every 30s until data is available, then switch to hourly
    let mut initial = true;

    loop {
        if initial {
            // On startup, wait 10s for collector to start fetching
            tokio::time::sleep(Duration::from_secs(10)).await;
        } else {
            tokio::select! {
                _ = ticker.tick() => {},
                Some(_event) = event_rx.recv() => {
                    // Drain queued events to coalesce rapid-fire triggers
                    while event_rx.try_recv().is_ok() {}
                },
            };
        }

        let trigger = InsightTrigger::Scheduled;

        match run_single_analysis(&db, &client, &trigger).await {
            Ok(true) => {
                info!(trigger = ?trigger.trigger_type(), "Analysis completed and stored");
                initial = false;
            }
            Ok(false) => {
                if initial {
                    info!("Initial analysis skipped (cold start), retrying in 30s...");
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    // Don't exit initial mode — keep retrying
                } else {
                    debug!(trigger = ?trigger.trigger_type(), "Analysis skipped (dedup)");
                }
            }
            Err(e) => {
                let wait_secs = if matches!(e, InsightError::RateLimited) {
                    warn!("Rate limited by Gemini API, waiting 120s before retry");
                    120
                } else {
                    error!(trigger = ?trigger.trigger_type(), error = %e, "Analysis failed");
                    60
                };
                initial = false;
                if let Err(db_err) = extend_latest_expiry(&db).await {
                    error!(error = %db_err, "Failed to extend insight expiry");
                }
                tokio::time::sleep(Duration::from_secs(wait_secs)).await;
            }
        }
    }
}

/// Run a single analysis cycle. Returns Ok(true) if a new insight was stored.
async fn run_single_analysis(
    db: &DatabaseConnection,
    client: &GeminiClient,
    trigger: &InsightTrigger,
) -> Result<bool, InsightError> {
    // Step 1: Extract features
    let snapshot = feature_extractor::extract_features(db)
        .await
        .map_err(|e| InsightError::ParseFailed(format!("DB error: {e}")))?;

    let snapshot = match snapshot {
        Some(s) => s,
        None => {
            debug!("Insufficient data for analysis (cold start)");
            return Ok(false);
        }
    };

    // Step 2: Check dedup for scheduled triggers
    let source_hash = compute_source_hash(trigger.trigger_type(), trigger.trigger_id(), &snapshot);

    if matches!(trigger, InsightTrigger::Scheduled) {
        if let Ok(true) = hash_exists(db, &source_hash).await {
            debug!("Skipping analysis: source_hash unchanged");
            return Ok(false);
        }
    }

    // Step 3: Call Gemini
    let response = client.generate_insight(&snapshot).await?;

    // Step 4: Store insight
    store_insight(db, trigger, &snapshot, &response, &source_hash)
        .await
        .map_err(|e| InsightError::ParseFailed(format!("DB write error: {e}")))?;

    Ok(true)
}

async fn hash_exists(db: &DatabaseConnection, hash: &str) -> Result<bool, sea_orm::DbErr> {
    let existing = ai_insights::Entity::find()
        .filter(ai_insights::Column::SourceHash.eq(hash))
        .one(db)
        .await?;
    Ok(existing.is_some())
}

async fn store_insight(
    db: &DatabaseConnection,
    trigger: &InsightTrigger,
    snapshot: &FeatureSnapshot,
    response: &InsightResponse,
    source_hash: &str,
) -> Result<(), sea_orm::DbErr> {
    let now = Utc::now();
    let window_end = now;
    let window_start = now - chrono::Duration::hours(1);
    let expires_at = now + chrono::Duration::hours(1);

    let summary_json = serde_json::to_string(response).unwrap_or_default();
    let signals_json = serde_json::to_string(snapshot).unwrap_or_default();

    let model = ai_insights::ActiveModel {
        scope: Set(trigger.scope().to_string()),
        trigger_type: Set(trigger.trigger_type().to_string()),
        trigger_id: Set(trigger.trigger_id().map(|s| s.to_string())),
        window_start: Set(window_start),
        window_end: Set(window_end),
        headline: Set(response.headline.clone()),
        summary_json: Set(summary_json),
        confidence: Set(response.confidence),
        signals_json: Set(signals_json),
        model_id: Set(MODEL_ID.to_string()),
        source_hash: Set(source_hash.to_string()),
        created_at: Set(now),
        expires_at: Set(expires_at),
        ..Default::default()
    };

    model.insert(db).await?;
    info!(
        scope = trigger.scope(),
        headline = %response.headline,
        confidence = response.confidence,
        severity = %response.severity,
        "New AI insight stored"
    );

    Ok(())
}

async fn extend_latest_expiry(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let latest = ai_insights::Entity::find()
        .order_by_desc(ai_insights::Column::CreatedAt)
        .one(db)
        .await?;

    if let Some(insight) = latest {
        let new_expiry = Utc::now() + chrono::Duration::hours(1);
        let mut active: ai_insights::ActiveModel = insight.into();
        active.expires_at = Set(new_expiry);
        active.update(db).await?;
        warn!("Extended insight expiry due to analysis failure");
    }

    Ok(())
}
