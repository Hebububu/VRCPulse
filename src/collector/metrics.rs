use chrono::{TimeZone, Utc};
use reqwest::Client;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::{debug, warn};

use crate::entity::metric_logs;

use super::client::{Result, fetch_json, metrics_api_url};
use super::models::{CLOUDFRONT_METRICS, MetricDefinition, MetricsResponse};

/// Default interval for CloudFront metrics (60 seconds)
const METRIC_INTERVAL_SEC: i64 = 60;

/// Poll all CloudFront metrics endpoints
pub async fn poll(client: &Client, db: &DatabaseConnection) -> Result<()> {
    for metric in CLOUDFRONT_METRICS {
        if let Err(e) = poll_metric(client, db, metric).await {
            warn!(
                metric = %metric.name,
                error = %e,
                "Failed to poll metric, skipping"
            );
        }
    }

    Ok(())
}

async fn poll_metric(
    client: &Client,
    db: &DatabaseConnection,
    metric: &MetricDefinition,
) -> Result<()> {
    let url = metrics_api_url(metric.endpoint);
    let response: MetricsResponse = fetch_json(client, &url).await?;

    let now = Utc::now();
    let mut inserted_count = 0;

    for (timestamp, value) in response {
        let dt = Utc.timestamp_opt(timestamp, 0).single();
        let Some(dt) = dt else {
            warn!(timestamp = timestamp, "Invalid timestamp, skipping");
            continue;
        };

        // Check if already exists (dedup by metric_name + timestamp)
        let existing = metric_logs::Entity::find()
            .filter(metric_logs::Column::MetricName.eq(metric.name))
            .filter(metric_logs::Column::Timestamp.eq(dt))
            .one(db)
            .await?;

        if existing.is_none() {
            let active = metric_logs::ActiveModel {
                metric_name: Set(metric.name.to_string()),
                value: Set(value),
                unit: Set(metric.unit.to_string()),
                interval_sec: Set(METRIC_INTERVAL_SEC),
                timestamp: Set(dt),
                created_at: Set(now),
                ..Default::default()
            };
            active.insert(db).await?;
            inserted_count += 1;
        }
    }

    if inserted_count > 0 {
        debug!(
            metric = %metric.name,
            count = inserted_count,
            "Inserted metric data points"
        );
    }

    Ok(())
}
