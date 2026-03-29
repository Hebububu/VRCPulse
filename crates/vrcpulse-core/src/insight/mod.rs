pub mod feature_extractor;
pub mod gemini_client;

use std::time::Duration;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
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
const MODEL_ID: &str = "gemini-3.1-flash-lite-preview";

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
    let mut trigger_buf: Option<InsightTrigger> = None;

    loop {
        if initial {
            // On startup, wait 10s for collector to start fetching
            tokio::time::sleep(Duration::from_secs(10)).await;
        } else {
            tokio::select! {
                _ = ticker.tick() => {
                    trigger_buf = Some(InsightTrigger::Scheduled);
                },
                Some(event) = event_rx.recv() => {
                    trigger_buf = Some(event);
                    // Drain queued events to coalesce rapid-fire triggers
                    while event_rx.try_recv().is_ok() {}
                },
            };
        }

        let trigger = trigger_buf.take().unwrap_or(InsightTrigger::Scheduled);

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

    // Step 2: Check dedup for scheduled triggers (English hash only)
    let source_hash = compute_source_hash(trigger.trigger_type(), trigger.trigger_id(), &snapshot);

    if matches!(trigger, InsightTrigger::Scheduled) {
        if let Ok(true) = hash_exists(db, &source_hash).await {
            // Check if the latest cycle has both languages before skipping
            if let Ok(true) = latest_cycle_complete(db).await {
                debug!("Skipping analysis: source_hash unchanged and both languages present");
                return Ok(false);
            }
            // English exists but Korean missing — retry translation only
            if let Some((en_insight, cycle_id)) = get_latest_english_only_cycle(db).await? {
                debug!("Source hash unchanged, retrying Korean translation only");
                translate_and_store(
                    db,
                    client,
                    trigger,
                    &snapshot,
                    &en_insight,
                    &source_hash,
                    &cycle_id,
                )
                .await;
                return Ok(true);
            }
            debug!("Source hash unchanged but translation missing, regenerating");
        }
    }

    // Step 3: Generate cycle_id
    let cycle_id = generate_cycle_id();

    // Step 4: Call Gemini for English analysis
    let en_response = client.generate_insight(&snapshot).await?;

    // Step 5: Store English insight
    store_insight(
        db,
        trigger,
        &snapshot,
        &en_response,
        &source_hash,
        "en",
        &cycle_id,
    )
    .await
    .map_err(|e| InsightError::ParseFailed(format!("DB write error: {e}")))?;

    // Step 6: Translate to Korean with retry
    translate_and_store(
        db,
        client,
        trigger,
        &snapshot,
        &en_response,
        &source_hash,
        &cycle_id,
    )
    .await;

    Ok(true)
}

/// Translate English insight to Korean and store, with 1 retry on failure.
async fn translate_and_store(
    db: &DatabaseConnection,
    client: &GeminiClient,
    trigger: &InsightTrigger,
    snapshot: &FeatureSnapshot,
    en_response: &InsightResponse,
    source_hash: &str,
    cycle_id: &str,
) {
    // Brief delay before translation to avoid rate limiting
    tokio::time::sleep(Duration::from_secs(1)).await;

    let ko_hash = format!("{source_hash}:ko");

    match client.translate_insight(en_response).await {
        Ok(ko_response) => {
            if let Err(e) = store_insight(
                db,
                trigger,
                snapshot,
                &ko_response,
                &ko_hash,
                "ko",
                cycle_id,
            )
            .await
            {
                warn!(error = %e, "Failed to store Korean translation");
            }
        }
        Err(e) => {
            warn!(error = %e, "Korean translation failed, retrying in 3s...");
            tokio::time::sleep(Duration::from_secs(3)).await;

            match client.translate_insight(en_response).await {
                Ok(ko_response) => {
                    if let Err(e) = store_insight(
                        db,
                        trigger,
                        snapshot,
                        &ko_response,
                        &ko_hash,
                        "ko",
                        cycle_id,
                    )
                    .await
                    {
                        warn!(error = %e, "Failed to store Korean translation on retry");
                    } else {
                        info!("Korean translation succeeded on retry");
                    }
                }
                Err(e2) => {
                    warn!(error = %e2, "Korean translation failed after retry, English-only insight stored");
                }
            }
        }
    }
}

/// Find the latest cycle that has English but no Korean translation.
/// Returns the English InsightResponse and cycle_id if found.
async fn get_latest_english_only_cycle(
    db: &DatabaseConnection,
) -> Result<Option<(InsightResponse, String)>, InsightError> {
    let latest = ai_insights::Entity::find()
        .filter(ai_insights::Column::Language.eq("en"))
        .order_by_desc(ai_insights::Column::CreatedAt)
        .one(db)
        .await
        .map_err(|e| InsightError::ParseFailed(format!("DB error: {e}")))?;

    let latest = match latest {
        Some(l) if !l.cycle_id.is_empty() => l,
        _ => return Ok(None),
    };

    // Check if Korean exists in this cycle
    let ko_count = ai_insights::Entity::find()
        .filter(ai_insights::Column::CycleId.eq(&latest.cycle_id))
        .filter(ai_insights::Column::Language.eq("ko"))
        .count(db)
        .await
        .map_err(|e| InsightError::ParseFailed(format!("DB error: {e}")))?;

    if ko_count > 0 {
        return Ok(None); // Already has Korean
    }

    let en_response: InsightResponse = serde_json::from_str(&latest.summary_json)
        .map_err(|e| InsightError::ParseFailed(format!("JSON parse error: {e}")))?;

    Ok(Some((en_response, latest.cycle_id)))
}

fn generate_cycle_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let rand: u32 = (ts as u32).wrapping_mul(2654435761); // simple hash for uniqueness
    format!("{ts:x}-{rand:08x}")
}

/// Check if the latest cycle has both en and ko translations.
async fn latest_cycle_complete(db: &DatabaseConnection) -> Result<bool, sea_orm::DbErr> {
    let latest = ai_insights::Entity::find()
        .order_by_desc(ai_insights::Column::CreatedAt)
        .one(db)
        .await?;

    let latest = match latest {
        Some(l) if !l.cycle_id.is_empty() => l,
        _ => return Ok(false), // No cycle or legacy row
    };

    let count = ai_insights::Entity::find()
        .filter(ai_insights::Column::CycleId.eq(&latest.cycle_id))
        .count(db)
        .await?;

    Ok(count >= 2)
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
    language: &str,
    cycle_id: &str,
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
        language: Set(language.to_string()),
        cycle_id: Set(cycle_id.to_string()),
        created_at: Set(now),
        expires_at: Set(expires_at),
        ..Default::default()
    };

    model.insert(db).await?;
    info!(
        scope = trigger.scope(),
        language = language,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_cycle_id_not_empty() {
        let id = generate_cycle_id();
        assert!(!id.is_empty());
        assert!(id.contains('-'));
    }

    #[test]
    fn test_generate_cycle_id_format() {
        let id = generate_cycle_id();
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts.len(), 2);
        // Both parts should be valid hex
        assert!(u128::from_str_radix(parts[0], 16).is_ok());
        assert!(u32::from_str_radix(parts[1], 16).is_ok());
    }

    #[test]
    fn test_insight_trigger_types() {
        let scheduled = InsightTrigger::Scheduled;
        assert_eq!(scheduled.trigger_type(), "scheduled");
        assert_eq!(scheduled.scope(), "hourly");
        assert!(scheduled.trigger_id().is_none());

        let incident = InsightTrigger::IncidentDetected {
            incident_id: "inc-123".to_string(),
            title: "API Down".to_string(),
        };
        assert_eq!(incident.trigger_type(), "incident_detected");
        assert_eq!(incident.scope(), "incident");
        assert_eq!(incident.trigger_id(), Some("inc-123"));

        let maintenance = InsightTrigger::MaintenanceDetected {
            maintenance_id: "mnt-456".to_string(),
        };
        assert_eq!(maintenance.trigger_type(), "maintenance_detected");
        assert_eq!(maintenance.scope(), "maintenance");
        assert_eq!(maintenance.trigger_id(), Some("mnt-456"));
    }
}
