use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;

use crate::entity::{incident_updates, incidents, maintenances, status_logs};
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

fn hours_from_range(range: &str) -> i64 {
    match range {
        "1h" => 1,
        "6h" => 6,
        "12h" => 12,
        "24h" => 24,
        _ => 12,
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

        use crate::entity::metric_logs;
        let data: Vec<metric_logs::Model> = metric_logs::Entity::find()
            .filter(metric_logs::Column::MetricName.eq(name))
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

        // Auto-convert 0-1 values to percentage for rate metrics
        let final_data = if name.contains("rate") || name.contains("auth") {
            query::to_percent(downsampled)
        } else {
            downsampled
        };

        Ok(metric_data_to_response(name, final_data))
    }

    pub async fn get_dashboard(
        &self,
        range: &str,
    ) -> Result<DashboardResponse, sea_orm::DbErr> {
        let metric_names = [
            "online_users",
            "api_latency",
            "api_requests",
            "api_error_rate",
            "steam_auth",
            "meta_auth",
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
        let mut query = incidents::Entity::find()
            .order_by_desc(incidents::Column::StartedAt);

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
        let mut query = maintenances::Entity::find()
            .order_by_desc(maintenances::Column::ScheduledFor);

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
}
