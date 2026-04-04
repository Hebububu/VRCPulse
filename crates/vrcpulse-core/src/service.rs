use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::entity::{
    ai_insights, incident_snapshots, incident_updates, incidents, maintenance_snapshots,
    maintenances, status_logs, translations,
};
use crate::insight::gemini_client::{GeminiClient, InsightResponse};
use crate::query::{self, MetricData};

/// Response from the translation endpoint.
#[derive(Debug, Serialize, Clone)]
pub struct TranslationResponse {
    pub translated_name: String,
    pub translated_body: String,
    pub translated_updates: Vec<TranslatedUpdateResponse>,
    pub cached: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranslatedUpdateResponse {
    pub update_id: String,
    pub translated_body: String,
}

/// Shared service layer used by both Tauri commands and Axum handlers.
pub struct VrcPulseService {
    db: DatabaseConnection,
    gemini_api_key: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StatusResponse {
    pub indicator: String,
    pub description: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MetricResponse {
    pub name: String,
    pub unit: String,
    pub timestamps: Vec<String>,
    pub values: Vec<f64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DashboardResponse {
    pub metrics: std::collections::HashMap<String, MetricResponse>,
    pub status: StatusResponse,
}

#[derive(Debug, Serialize, Clone)]
pub struct IncidentResponse {
    pub id: String,
    pub name: String,
    pub status: String,
    pub impact: String,
    pub created_at: String,
    pub updates: Vec<IncidentUpdateResponse>,
}

#[derive(Debug, Serialize, Clone)]
pub struct IncidentUpdateResponse {
    pub id: String,
    pub status: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct IncidentsListResponse {
    pub incidents: Vec<IncidentResponse>,
}

#[derive(Debug, Serialize, Clone)]
pub struct MaintenanceResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub scheduled_for: String,
    pub scheduled_until: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MaintenancesListResponse {
    pub maintenances: Vec<MaintenanceResponse>,
}

#[derive(Debug, Serialize, Clone)]
pub struct MaintenanceSnapshotResponse {
    pub maintenance_id: String,
    pub title: String,
    pub status: String,
    pub scheduled_for: String,
    pub scheduled_until: String,
    pub fetched_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct IncidentSnapshotResponse {
    pub incident_id: String,
    pub title: String,
    pub impact: String,
    pub status: String,
    pub update_count: i32,
    pub fetched_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct AiInsightResponse {
    pub id: i64,
    pub scope: String,
    pub trigger_type: String,
    pub headline: String,
    pub summary: InsightResponse,
    pub confidence: f64,
    pub model_id: String,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct InsightBundle {
    pub en: Option<AiInsightResponse>,
    pub ko: Option<AiInsightResponse>,
    pub jp: Option<AiInsightResponse>,
}

fn hours_from_range(range: &str) -> i64 {
    match range {
        "1h" => 1,
        "6h" => 6,
        "12h" => 12,
        "24h" => 24,
        _ => 12,
    }
}

/// Map dashboard-friendly names to actual database metric names
fn resolve_db_metric(name: &str) -> &str {
    match name {
        "online_users" => "visits",
        "api_error_rate" => "api_errors",
        "steam_auth" => "extauth_steam",
        "meta_auth" => "extauth_oculus",
        "steam_share" => "extauth_steam_count",
        "meta_share" => "extauth_oculus_count",
        // These match directly
        "api_latency" => "api_latency",
        "api_requests" => "api_requests",
        other => other,
    }
}

fn metric_data_to_response(name: &str, data: MetricData) -> MetricResponse {
    MetricResponse {
        name: name.to_string(),
        unit: data.unit,
        timestamps: data.timestamps.iter().map(|t| t.to_rfc3339()).collect(),
        values: data.values,
    }
}

impl VrcPulseService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            gemini_api_key: None,
        }
    }

    pub fn with_gemini_key(mut self, key: Option<String>) -> Self {
        self.gemini_api_key = key;
        self
    }

    pub fn db_ref(&self) -> &DatabaseConnection {
        &self.db
    }

    pub async fn get_status(&self) -> Result<StatusResponse, sea_orm::DbErr> {
        let latest = status_logs::Entity::find()
            .order_by_desc(status_logs::Column::SourceTimestamp)
            .one(&self.db)
            .await?;

        match latest {
            Some(log) => Ok(StatusResponse {
                indicator: log.indicator,
                description: log.description,
                updated_at: log.source_timestamp.to_rfc3339(),
            }),
            None => Ok(StatusResponse {
                indicator: "none".to_string(),
                description: "No status data yet".to_string(),
                updated_at: Utc::now().to_rfc3339(),
            }),
        }
    }

    /// Get raw metric data for chart rendering (returns MetricData with DateTime timestamps)
    pub async fn get_metrics_raw(
        &self,
        name: &str,
        range: &str,
    ) -> Result<MetricData, sea_orm::DbErr> {
        let hours = hours_from_range(range);
        let cutoff = Utc::now() - Duration::hours(hours);
        let db_name = resolve_db_metric(name);

        use crate::entity::metric_logs;
        let data: Vec<metric_logs::Model> = metric_logs::Entity::find()
            .filter(metric_logs::Column::MetricName.eq(db_name))
            .filter(metric_logs::Column::Timestamp.gte(cutoff))
            .order_by_asc(metric_logs::Column::Timestamp)
            .all(&self.db)
            .await?;

        let metric_data = MetricData {
            timestamps: data.iter().map(|d| d.timestamp).collect(),
            values: data.iter().map(|d| d.value).collect(),
            unit: data.first().map(|d| d.unit.clone()).unwrap_or_default(),
        };

        Ok(query::downsample(metric_data))
    }

    /// Get raw metric data as percentage (0-1 values converted to 0-100)
    ///
    /// Use for metrics like api_errors, extauth_steam, extauth_oculus that are
    /// stored as 0-1 fractions but displayed as percentages.
    pub async fn get_metrics_raw_percent(
        &self,
        name: &str,
        range: &str,
    ) -> Result<MetricData, sea_orm::DbErr> {
        let data = self.get_metrics_raw(name, range).await?;
        Ok(query::to_percent(data))
    }

    /// Get metric data as a JSON-serializable response (timestamps as RFC3339 strings)
    pub async fn get_metrics(
        &self,
        name: &str,
        range: &str,
    ) -> Result<MetricResponse, sea_orm::DbErr> {
        let data = self.get_metrics_raw(name, range).await?;
        Ok(metric_data_to_response(name, data))
    }

    pub async fn get_dashboard(&self, range: &str) -> Result<DashboardResponse, sea_orm::DbErr> {
        let metric_names = [
            "online_users",
            "api_latency",
            "api_requests",
            "api_error_rate",
            "steam_auth",
            "meta_auth",
            "steam_share",
            "meta_share",
        ];

        let mut metrics = std::collections::HashMap::new();
        for name in &metric_names {
            let data = self.get_metrics(name, range).await?;
            metrics.insert(name.to_string(), data);
        }

        let status = self.get_status().await?;

        Ok(DashboardResponse { metrics, status })
    }

    pub async fn get_incidents(
        &self,
        status_filter: &str,
    ) -> Result<IncidentsListResponse, sea_orm::DbErr> {
        let mut query = incidents::Entity::find().order_by_desc(incidents::Column::StartedAt);

        if status_filter != "all" {
            query = query.filter(incidents::Column::Status.eq(status_filter));
        }

        let incident_list = query.all(&self.db).await?;

        let mut result = Vec::new();
        for incident in incident_list {
            let updates = incident_updates::Entity::find()
                .filter(incident_updates::Column::IncidentId.eq(&incident.id))
                .order_by_desc(incident_updates::Column::PublishedAt)
                .all(&self.db)
                .await?;

            result.push(IncidentResponse {
                id: incident.id,
                name: incident.title,
                status: incident.status,
                impact: incident.impact,
                created_at: incident.started_at.to_rfc3339(),
                updates: updates
                    .into_iter()
                    .map(|u| IncidentUpdateResponse {
                        id: u.id.clone(),
                        status: u.status,
                        body: u.body,
                        created_at: u.published_at.to_rfc3339(),
                    })
                    .collect(),
            });
        }

        Ok(IncidentsListResponse { incidents: result })
    }

    pub async fn get_maintenances(
        &self,
        status_filter: &str,
    ) -> Result<MaintenancesListResponse, sea_orm::DbErr> {
        let mut query =
            maintenances::Entity::find().order_by_desc(maintenances::Column::ScheduledFor);

        if status_filter != "all" {
            query = query.filter(maintenances::Column::Status.eq(status_filter));
        }

        let list = query.all(&self.db).await?;

        Ok(MaintenancesListResponse {
            maintenances: list
                .into_iter()
                .map(|m| MaintenanceResponse {
                    id: m.id,
                    name: m.title,
                    description: m.description,
                    status: m.status,
                    scheduled_for: m.scheduled_for.to_rfc3339(),
                    scheduled_until: m.scheduled_until.to_rfc3339(),
                })
                .collect(),
        })
    }

    /// Get a single maintenance by ID.
    pub async fn get_maintenance_by_id(
        &self,
        id: &str,
    ) -> Result<Option<MaintenanceResponse>, sea_orm::DbErr> {
        let m = maintenances::Entity::find_by_id(id).one(&self.db).await?;

        Ok(m.map(|m| MaintenanceResponse {
            id: m.id,
            name: m.title,
            description: m.description,
            status: m.status,
            scheduled_for: m.scheduled_for.to_rfc3339(),
            scheduled_until: m.scheduled_until.to_rfc3339(),
        }))
    }

    /// Get snapshot history for a specific maintenance.
    pub async fn get_maintenance_history(
        &self,
        maintenance_id: &str,
    ) -> Result<Vec<MaintenanceSnapshotResponse>, sea_orm::DbErr> {
        let snapshots = maintenance_snapshots::Entity::find()
            .filter(maintenance_snapshots::Column::MaintenanceId.eq(maintenance_id))
            .order_by_asc(maintenance_snapshots::Column::FetchedAt)
            .all(&self.db)
            .await?;

        Ok(snapshots
            .into_iter()
            .map(|s| MaintenanceSnapshotResponse {
                maintenance_id: s.maintenance_id,
                title: s.title,
                status: s.status,
                scheduled_for: s.scheduled_for.to_rfc3339(),
                scheduled_until: s.scheduled_until.to_rfc3339(),
                fetched_at: s.fetched_at.to_rfc3339(),
            })
            .collect())
    }

    /// Get all incidents from snapshots (latest snapshot per incident)
    pub async fn get_incidents_from_snapshots(
        &self,
        status_filter: &str,
    ) -> Result<IncidentsListResponse, sea_orm::DbErr> {
        // Get latest snapshot per incident_id
        let snapshots = incident_snapshots::Entity::find()
            .order_by_desc(incident_snapshots::Column::FetchedAt)
            .all(&self.db)
            .await?;

        // Deduplicate: keep only the latest snapshot per incident_id
        let mut seen = std::collections::HashSet::new();
        let mut unique: Vec<incident_snapshots::Model> = Vec::new();
        for s in snapshots {
            if seen.insert(s.incident_id.clone()) {
                unique.push(s);
            }
        }

        // Filter by status
        let filtered: Vec<_> = if status_filter == "all" {
            unique
        } else {
            unique
                .into_iter()
                .filter(|s| s.status == status_filter)
                .collect()
        };

        let mut result = Vec::new();
        for s in filtered {
            // Parse updates from raw_json
            let updates =
                if let Ok(incident) = serde_json::from_str::<serde_json::Value>(&s.raw_json) {
                    incident["incident_updates"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|u| {
                                    Some(IncidentUpdateResponse {
                                        id: u["id"].as_str().unwrap_or("").to_string(),
                                        status: u["status"].as_str()?.to_string(),
                                        body: u["body"].as_str()?.to_string(),
                                        created_at: u["created_at"].as_str()?.to_string(),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

            result.push(IncidentResponse {
                id: s.incident_id,
                name: s.title,
                status: s.status,
                impact: s.impact,
                created_at: s.started_at.to_rfc3339(),
                updates,
            });
        }

        Ok(IncidentsListResponse { incidents: result })
    }

    /// Get the latest AI insight, if any.
    pub async fn get_latest_insight(&self) -> Result<Option<InsightBundle>, sea_orm::DbErr> {
        // Find the most recent non-expired insight to get its cycle_id
        let now = Utc::now();
        let latest = ai_insights::Entity::find()
            .filter(ai_insights::Column::ExpiresAt.gte(now))
            .order_by_desc(ai_insights::Column::CreatedAt)
            .one(&self.db)
            .await?;

        let latest = match latest {
            Some(l) => l,
            None => return Ok(None),
        };

        // If cycle_id is empty (legacy row), return it as-is based on language
        let insights = if latest.cycle_id.is_empty() {
            vec![latest]
        } else {
            // Fetch all insights in the same cycle
            ai_insights::Entity::find()
                .filter(ai_insights::Column::CycleId.eq(&latest.cycle_id))
                .all(&self.db)
                .await?
        };

        let mut bundle = InsightBundle {
            en: None,
            ko: None,
            jp: None,
        };

        for insight in insights {
            let response = Self::insight_to_response(insight);
            match response.1.as_str() {
                "en" => bundle.en = Some(response.0),
                "ko" => bundle.ko = Some(response.0),
                "jp" => bundle.jp = Some(response.0),
                _ => {}
            }
        }

        // Legacy fallback: if no language was set, treat as Korean
        if bundle.en.is_none() && bundle.ko.is_none() {
            return Ok(None);
        }

        Ok(Some(bundle))
    }

    fn insight_to_response(insight: ai_insights::Model) -> (AiInsightResponse, String) {
        let language = insight.language.clone();
        let summary: InsightResponse =
            serde_json::from_str(&insight.summary_json).unwrap_or(InsightResponse {
                headline: insight.headline.clone(),
                bullets: vec![],
                affected_surfaces: vec![],
                reasoning_basis: vec![],
                confidence: insight.confidence,
                severity: "stable".to_string(),
            });

        let response = AiInsightResponse {
            id: insight.id,
            scope: insight.scope,
            trigger_type: insight.trigger_type,
            headline: insight.headline,
            summary,
            confidence: insight.confidence,
            model_id: insight.model_id,
            created_at: insight.created_at.to_rfc3339(),
            expires_at: insight.expires_at.to_rfc3339(),
        };

        (response, language)
    }

    /// Get snapshot history for a specific incident
    pub async fn get_incident_history(
        &self,
        incident_id: &str,
    ) -> Result<Vec<IncidentSnapshotResponse>, sea_orm::DbErr> {
        let snapshots = incident_snapshots::Entity::find()
            .filter(incident_snapshots::Column::IncidentId.eq(incident_id))
            .order_by_asc(incident_snapshots::Column::FetchedAt)
            .all(&self.db)
            .await?;

        Ok(snapshots
            .into_iter()
            .map(|s| IncidentSnapshotResponse {
                incident_id: s.incident_id,
                title: s.title,
                impact: s.impact,
                status: s.status,
                update_count: s.update_count,
                fetched_at: s.fetched_at.to_rfc3339(),
            })
            .collect())
    }

    /// Translate incident or maintenance content.
    /// Checks DB cache first; on miss, calls Gemini and stores the result.
    pub async fn translate_content(
        &self,
        item_type: &str,
        item_id: &str,
        locale: &str,
    ) -> Result<TranslationResponse, String> {
        // 1. Fetch source content
        let (name, body, updates) = match item_type {
            "incident" => self.get_incident_content(item_id).await?,
            "maintenance" => self.get_maintenance_content(item_id).await?,
            _ => return Err("Invalid item_type. Use 'incident' or 'maintenance'.".to_string()),
        };

        // 2. Compute content hash
        let content_hash = compute_content_hash(&name, &body, &updates);

        // 3. Check DB cache
        let cached = translations::Entity::find()
            .filter(translations::Column::ItemType.eq(item_type))
            .filter(translations::Column::ItemId.eq(item_id))
            .filter(translations::Column::Locale.eq(locale))
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(ref row) = cached
            && row.content_hash == content_hash
        {
            let updates: Vec<TranslatedUpdateResponse> =
                serde_json::from_str(&row.translated_updates).unwrap_or_default();
            return Ok(TranslationResponse {
                translated_name: row.translated_name.clone(),
                translated_body: row.translated_body.clone(),
                translated_updates: updates,
                cached: true,
            });
        }

        // 4. Call Gemini
        let api_key = self
            .gemini_api_key
            .as_ref()
            .ok_or("GOOGLE_AI_STUDIO_API_KEY not configured")?;

        let client = GeminiClient::new(
            reqwest::Client::new(),
            api_key.clone(),
            crate::insight::MODEL_ID,
        );

        let update_pairs: Vec<(String, String)> = updates
            .iter()
            .map(|(id, body)| (id.clone(), body.clone()))
            .collect();

        let result = client
            .translate_content(&name, &body, &update_pairs, locale)
            .await
            .map_err(|e| format!("Translation failed: {e}"))?;

        // 5. Upsert into DB
        let now = Utc::now();
        let translated_updates_json = serde_json::to_string(
            &result
                .updates
                .iter()
                .map(|u| TranslatedUpdateResponse {
                    update_id: u.update_id.clone(),
                    translated_body: u.translated_body.clone(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or_else(|_| "[]".to_string());

        if let Some(existing) = cached {
            // Update existing row
            let mut active: translations::ActiveModel = existing.into();
            active.content_hash = Set(content_hash);
            active.translated_name = Set(result.name.clone());
            active.translated_body = Set(result.body.clone());
            active.translated_updates = Set(translated_updates_json.clone());
            active.updated_at = Set(now);
            active.update(&self.db).await.map_err(|e| e.to_string())?;
        } else {
            // Insert new row
            let active = translations::ActiveModel {
                item_type: Set(item_type.to_string()),
                item_id: Set(item_id.to_string()),
                locale: Set(locale.to_string()),
                content_hash: Set(content_hash),
                translated_name: Set(result.name.clone()),
                translated_body: Set(result.body.clone()),
                translated_updates: Set(translated_updates_json.clone()),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };
            active.insert(&self.db).await.map_err(|e| e.to_string())?;
        }

        let response_updates: Vec<TranslatedUpdateResponse> = result
            .updates
            .into_iter()
            .map(|u| TranslatedUpdateResponse {
                update_id: u.update_id,
                translated_body: u.translated_body,
            })
            .collect();

        Ok(TranslationResponse {
            translated_name: result.name,
            translated_body: result.body,
            translated_updates: response_updates,
            cached: false,
        })
    }

    async fn get_incident_content(
        &self,
        incident_id: &str,
    ) -> Result<(String, String, Vec<(String, String)>), String> {
        // Get latest snapshot
        let snapshot = incident_snapshots::Entity::find()
            .filter(incident_snapshots::Column::IncidentId.eq(incident_id))
            .order_by_desc(incident_snapshots::Column::FetchedAt)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Incident {incident_id} not found"))?;

        let name = snapshot.title;
        let body = String::new(); // Incidents don't have a top-level body

        // Parse updates from raw_json
        let updates =
            if let Ok(incident) = serde_json::from_str::<serde_json::Value>(&snapshot.raw_json) {
                incident["incident_updates"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|u| {
                                let id = u["id"]
                                    .as_str()
                                    .or_else(|| u["created_at"].as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let body = u["body"].as_str().unwrap_or("").to_string();
                                if body.is_empty() {
                                    None
                                } else {
                                    Some((id, body))
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

        Ok((name, body, updates))
    }

    async fn get_maintenance_content(
        &self,
        maintenance_id: &str,
    ) -> Result<(String, String, Vec<(String, String)>), String> {
        let m = maintenances::Entity::find_by_id(maintenance_id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Maintenance {maintenance_id} not found"))?;

        Ok((m.title, m.description, Vec::new()))
    }

    /// Pre-translate all incidents and maintenances to all supported locales.
    /// Translates most recent items first, with delays between requests to avoid rate limiting.
    pub async fn pre_translate_all(&self) {
        use tokio::time::{Duration, sleep};
        use tracing::{debug, info, warn};

        const DELAY_BETWEEN_REQUESTS: Duration = Duration::from_secs(8);
        const RATE_LIMIT_PAUSE: Duration = Duration::from_secs(120);
        const LOCALES: &[&str] = &["ko", "jp"];

        if self.gemini_api_key.is_none() {
            warn!("Skipping pre-translation: no Gemini API key");
            return;
        }

        info!("Starting pre-translation of incidents and maintenances");

        // Collect unique incident IDs (most recent first)
        let snapshots = incident_snapshots::Entity::find()
            .order_by_desc(incident_snapshots::Column::FetchedAt)
            .all(&self.db)
            .await
            .unwrap_or_default();

        let mut seen = std::collections::HashSet::new();
        let incident_ids: Vec<String> = snapshots
            .iter()
            .filter(|s| seen.insert(s.incident_id.clone()))
            .take(20)
            .map(|s| s.incident_id.clone())
            .collect();

        // Collect maintenance IDs (most recent scheduled first)
        let maint_ids: Vec<String> = maintenances::Entity::find()
            .order_by_desc(maintenances::Column::ScheduledFor)
            .all(&self.db)
            .await
            .unwrap_or_default()
            .into_iter()
            .take(10)
            .map(|m| m.id)
            .collect();

        // Interleave: translate newest incident, then newest maintenance, alternating
        let max_len = incident_ids.len().max(maint_ids.len());
        for i in 0..max_len {
            // Incident
            if let Some(id) = incident_ids.get(i) {
                for locale in LOCALES {
                    let result: Result<TranslationResponse, String> =
                        self.translate_content("incident", id, locale).await;
                    match result {
                        Ok(r) if r.cached => {
                            debug!(incident_id = %id, locale = locale, "Incident translation already cached");
                        }
                        Ok(r) => {
                            info!(incident_id = %id, locale = locale, name = %r.translated_name, "Pre-translated incident");
                            sleep(DELAY_BETWEEN_REQUESTS).await;
                        }
                        Err(e) => {
                            let rate_limited = e.contains("Rate limited");
                            warn!(incident_id = %id, locale = locale, "Failed to pre-translate incident: {e}");
                            if rate_limited {
                                info!("Rate limited, pausing pre-translation for 120s");
                                sleep(RATE_LIMIT_PAUSE).await;
                            }
                        }
                    }
                }
            }

            // Maintenance
            if let Some(id) = maint_ids.get(i) {
                for locale in LOCALES {
                    let result: Result<TranslationResponse, String> =
                        self.translate_content("maintenance", id, locale).await;
                    match result {
                        Ok(r) if r.cached => {
                            debug!(maintenance_id = %id, locale = locale, "Maintenance translation already cached");
                        }
                        Ok(r) => {
                            info!(maintenance_id = %id, locale = locale, name = %r.translated_name, "Pre-translated maintenance");
                            sleep(DELAY_BETWEEN_REQUESTS).await;
                        }
                        Err(e) => {
                            let rate_limited = e.contains("Rate limited");
                            warn!(maintenance_id = %id, locale = locale, "Failed to pre-translate maintenance: {e}");
                            if rate_limited {
                                info!("Rate limited, pausing pre-translation for 120s");
                                sleep(RATE_LIMIT_PAUSE).await;
                            }
                        }
                    }
                }
            }
        }

        info!("Pre-translation complete");
    }
}

fn compute_content_hash(name: &str, body: &str, updates: &[(String, String)]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    hasher.update(b"\n");
    hasher.update(body.as_bytes());
    for (_, update_body) in updates {
        hasher.update(b"\n");
        hasher.update(update_body.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database};
    use sea_orm_migration::MigratorTrait;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory SQLite");
        migration::Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");
        db
    }

    #[tokio::test]
    async fn test_get_status_empty_db() {
        let db = setup_test_db().await;
        let service = VrcPulseService::new(db);
        let status = service.get_status().await.unwrap();
        assert_eq!(status.indicator, "none");
        assert_eq!(status.description, "No status data yet");
    }

    #[tokio::test]
    async fn test_get_status_with_data() {
        let db = setup_test_db().await;
        db.execute_unprepared(
            "INSERT INTO status_logs (indicator, description, source_timestamp, created_at) \
             VALUES ('minor', 'Partial outage', datetime('now'), datetime('now'))",
        )
        .await
        .unwrap();

        let service = VrcPulseService::new(db);
        let status = service.get_status().await.unwrap();
        assert_eq!(status.indicator, "minor");
        assert_eq!(status.description, "Partial outage");
    }

    #[tokio::test]
    async fn test_get_metrics_empty() {
        let db = setup_test_db().await;
        let service = VrcPulseService::new(db);
        let result = service.get_metrics("online_users", "1h").await.unwrap();
        assert!(result.timestamps.is_empty());
        assert!(result.values.is_empty());
    }

    #[tokio::test]
    async fn test_get_dashboard_returns_all_keys() {
        let db = setup_test_db().await;
        let service = VrcPulseService::new(db);
        let dashboard = service.get_dashboard("1h").await.unwrap();
        let keys: Vec<&String> = dashboard.metrics.keys().collect();
        assert!(keys.len() >= 8);
        assert!(dashboard.metrics.contains_key("online_users"));
        assert!(dashboard.metrics.contains_key("api_latency"));
        assert!(dashboard.metrics.contains_key("steam_auth"));
        assert!(dashboard.metrics.contains_key("steam_share"));
    }

    #[tokio::test]
    async fn test_get_incidents_from_snapshots_empty() {
        let db = setup_test_db().await;
        let service = VrcPulseService::new(db);
        let result = service.get_incidents_from_snapshots("all").await.unwrap();
        assert!(result.incidents.is_empty());
    }

    #[test]
    fn test_resolve_db_metric() {
        assert_eq!(resolve_db_metric("online_users"), "visits");
        assert_eq!(resolve_db_metric("api_error_rate"), "api_errors");
        assert_eq!(resolve_db_metric("steam_auth"), "extauth_steam");
        assert_eq!(resolve_db_metric("meta_auth"), "extauth_oculus");
        assert_eq!(resolve_db_metric("api_latency"), "api_latency");
        assert_eq!(resolve_db_metric("unknown"), "unknown");
    }

    #[tokio::test]
    async fn test_get_latest_insight_empty_db() {
        let db = setup_test_db().await;
        let service = VrcPulseService::new(db);
        let result = service.get_latest_insight().await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_latest_insight_legacy_row() {
        let db = setup_test_db().await;
        // Legacy row: language='ko', cycle_id=''
        db.execute_unprepared(
            "INSERT INTO ai_insights (scope, trigger_type, window_start, window_end, headline, \
             summary_json, confidence, signals_json, model_id, source_hash, language, cycle_id, \
             created_at, expires_at) VALUES \
             ('hourly', 'scheduled', datetime('now'), datetime('now'), 'VRChat 서버 안정', \
             '{\"headline\":\"VRChat 서버 안정\",\"bullets\":[\"정상\"],\"confidence\":0.9,\"severity\":\"stable\"}', \
             0.9, '{}', 'gemini-2.5-flash', 'hash1', 'ko', '', datetime('now'), datetime('now', '+24 hours'))",
        )
        .await
        .unwrap();

        let service = VrcPulseService::new(db);
        let result = service.get_latest_insight().await.unwrap();
        assert!(result.is_some());
        let bundle = result.unwrap();
        assert!(bundle.en.is_none());
        assert!(bundle.ko.is_some());
        assert_eq!(bundle.ko.unwrap().headline, "VRChat 서버 안정");
    }

    #[tokio::test]
    async fn test_get_latest_insight_dual_language() {
        let db = setup_test_db().await;
        // English insight
        db.execute_unprepared(
            "INSERT INTO ai_insights (scope, trigger_type, window_start, window_end, headline, \
             summary_json, confidence, signals_json, model_id, source_hash, language, cycle_id, \
             created_at, expires_at) VALUES \
             ('hourly', 'scheduled', datetime('now'), datetime('now'), 'VRChat servers stable', \
             '{\"headline\":\"VRChat servers stable\",\"bullets\":[\"Normal\"],\"confidence\":0.9,\"severity\":\"stable\"}', \
             0.9, '{}', 'gemini-2.5-flash', 'hash-en', 'en', 'cycle-001', datetime('now'), datetime('now', '+24 hours'))",
        )
        .await
        .unwrap();
        // Korean insight
        db.execute_unprepared(
            "INSERT INTO ai_insights (scope, trigger_type, window_start, window_end, headline, \
             summary_json, confidence, signals_json, model_id, source_hash, language, cycle_id, \
             created_at, expires_at) VALUES \
             ('hourly', 'scheduled', datetime('now'), datetime('now'), 'VRChat 서버 안정', \
             '{\"headline\":\"VRChat 서버 안정\",\"bullets\":[\"정상\"],\"confidence\":0.9,\"severity\":\"stable\"}', \
             0.9, '{}', 'gemini-2.5-flash', 'hash-ko', 'ko', 'cycle-001', datetime('now'), datetime('now', '+24 hours'))",
        )
        .await
        .unwrap();

        let service = VrcPulseService::new(db);
        let result = service.get_latest_insight().await.unwrap();
        assert!(result.is_some());
        let bundle = result.unwrap();
        assert!(bundle.en.is_some());
        assert!(bundle.ko.is_some());
        assert_eq!(bundle.en.unwrap().headline, "VRChat servers stable");
        assert_eq!(bundle.ko.unwrap().headline, "VRChat 서버 안정");
    }

    #[tokio::test]
    async fn test_get_latest_insight_partial_cycle() {
        let db = setup_test_db().await;
        // Only English (Korean translation failed)
        db.execute_unprepared(
            "INSERT INTO ai_insights (scope, trigger_type, window_start, window_end, headline, \
             summary_json, confidence, signals_json, model_id, source_hash, language, cycle_id, \
             created_at, expires_at) VALUES \
             ('hourly', 'scheduled', datetime('now'), datetime('now'), 'VRChat servers stable', \
             '{\"headline\":\"VRChat servers stable\",\"bullets\":[\"Normal\"],\"confidence\":0.9,\"severity\":\"stable\"}', \
             0.9, '{}', 'gemini-2.5-flash', 'hash-en', 'en', 'cycle-partial', datetime('now'), datetime('now', '+24 hours'))",
        )
        .await
        .unwrap();

        let service = VrcPulseService::new(db);
        let result = service.get_latest_insight().await.unwrap();
        assert!(result.is_some());
        let bundle = result.unwrap();
        assert!(bundle.en.is_some());
        assert!(bundle.ko.is_none());
    }

    #[test]
    fn test_insight_to_response() {
        use chrono::Utc;
        let now = Utc::now();
        let model = ai_insights::Model {
            id: 42,
            scope: "hourly".to_string(),
            trigger_type: "scheduled".to_string(),
            trigger_id: None,
            window_start: now,
            window_end: now,
            headline: "Test headline".to_string(),
            summary_json: r#"{"headline":"Test headline","bullets":["point1"],"confidence":0.85,"severity":"stable"}"#.to_string(),
            confidence: 0.85,
            signals_json: "{}".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            source_hash: "abc123".to_string(),
            language: "en".to_string(),
            cycle_id: "cycle-test".to_string(),
            created_at: now,
            expires_at: now,
        };

        let (response, language) = VrcPulseService::insight_to_response(model);
        assert_eq!(response.id, 42);
        assert_eq!(response.headline, "Test headline");
        assert_eq!(response.summary.bullets, vec!["point1"]);
        assert!((response.confidence - 0.85).abs() < f64::EPSILON);
        assert_eq!(language, "en");
    }

    #[tokio::test]
    async fn test_get_latest_insight_expired_returns_none() {
        let db = setup_test_db().await;
        // Expired insight (1 hour ago)
        db.execute_unprepared(
            "INSERT INTO ai_insights (scope, trigger_type, window_start, window_end, headline, \
             summary_json, confidence, signals_json, model_id, source_hash, language, cycle_id, \
             created_at, expires_at) VALUES \
             ('hourly', 'scheduled', datetime('now', '-2 hours'), datetime('now', '-1 hour'), 'Expired', \
             '{\"headline\":\"Expired\",\"bullets\":[],\"confidence\":0.5,\"severity\":\"stable\"}', \
             0.5, '{}', 'gemini-2.5-flash', 'hash-expired', 'en', 'cycle-expired', \
             datetime('now', '-1 hour'), datetime('now', '-1 minute'))",
        )
        .await
        .unwrap();

        let service = VrcPulseService::new(db);
        let result = service.get_latest_insight().await.unwrap();
        assert!(result.is_none(), "Expired insights should not be returned");
    }

    #[tokio::test]
    async fn test_get_maintenance_by_id_found() {
        let db = setup_test_db().await;
        db.execute_unprepared(
            "INSERT INTO maintenances (id, title, description, status, scheduled_for, scheduled_until, created_at, updated_at) \
             VALUES ('mnt-1', 'Server Migration', 'Migrating to new infrastructure', 'scheduled', datetime('now', '+1 hour'), datetime('now', '+3 hours'), datetime('now'), datetime('now'))",
        )
        .await
        .unwrap();

        let service = VrcPulseService::new(db);
        let result = service.get_maintenance_by_id("mnt-1").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Server Migration");
    }

    #[tokio::test]
    async fn test_get_maintenance_by_id_not_found() {
        let db = setup_test_db().await;
        let service = VrcPulseService::new(db);
        let result = service.get_maintenance_by_id("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_maintenance_history_with_snapshots() {
        let db = setup_test_db().await;
        // Insert snapshots
        db.execute_unprepared(
            "INSERT INTO maintenance_snapshots (maintenance_id, title, status, scheduled_for, scheduled_until, raw_json, fetched_at) \
             VALUES ('mnt-1', 'Migration', 'scheduled', datetime('now', '+1 hour'), datetime('now', '+3 hours'), '{}', datetime('now', '-30 minutes'))",
        )
        .await
        .unwrap();
        db.execute_unprepared(
            "INSERT INTO maintenance_snapshots (maintenance_id, title, status, scheduled_for, scheduled_until, raw_json, fetched_at) \
             VALUES ('mnt-1', 'Migration', 'in_progress', datetime('now', '+1 hour'), datetime('now', '+3 hours'), '{}', datetime('now'))",
        )
        .await
        .unwrap();

        let service = VrcPulseService::new(db);
        let history = service.get_maintenance_history("mnt-1").await.unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].status, "scheduled");
        assert_eq!(history[1].status, "in_progress");
    }

    #[tokio::test]
    async fn test_get_maintenance_history_empty() {
        let db = setup_test_db().await;
        let service = VrcPulseService::new(db);
        let history = service
            .get_maintenance_history("nonexistent")
            .await
            .unwrap();
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_get_maintenances_filters() {
        let db = setup_test_db().await;
        db.execute_unprepared(
            "INSERT INTO maintenances (id, title, description, status, scheduled_for, scheduled_until, created_at, updated_at) VALUES \
             ('m1', 'Scheduled One', '', 'scheduled', datetime('now', '+1 hour'), datetime('now', '+2 hours'), datetime('now'), datetime('now')), \
             ('m2', 'In Progress', '', 'in_progress', datetime('now'), datetime('now', '+1 hour'), datetime('now'), datetime('now')), \
             ('m3', 'Done', '', 'completed', datetime('now', '-2 hours'), datetime('now', '-1 hour'), datetime('now'), datetime('now'))",
        )
        .await
        .unwrap();

        let service = VrcPulseService::new(db);

        // in_progress filter
        let result = service.get_maintenances("in_progress").await.unwrap();
        assert_eq!(result.maintenances.len(), 1);
        assert_eq!(result.maintenances[0].name, "In Progress");

        // all filter
        let result = service.get_maintenances("all").await.unwrap();
        assert_eq!(result.maintenances.len(), 3);
    }
}
