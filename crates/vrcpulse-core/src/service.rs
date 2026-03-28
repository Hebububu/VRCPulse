use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;

use crate::entity::{incident_snapshots, incident_updates, incidents, maintenances, status_logs};
use crate::query::{self, MetricData};

/// Shared service layer used by both Tauri commands and Axum handlers.
pub struct VrcPulseService {
    db: DatabaseConnection,
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
    pub status: String,
    pub scheduled_for: String,
    pub scheduled_until: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MaintenancesListResponse {
    pub maintenances: Vec<MaintenanceResponse>,
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
        Self { db }
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

    pub async fn get_metrics(
        &self,
        name: &str,
        range: &str,
    ) -> Result<MetricResponse, sea_orm::DbErr> {
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

        let downsampled = query::downsample(metric_data);
        Ok(metric_data_to_response(name, downsampled))
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
                    status: m.status,
                    scheduled_for: m.scheduled_for.to_rfc3339(),
                    scheduled_until: m.scheduled_until.to_rfc3339(),
                })
                .collect(),
        })
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
}
