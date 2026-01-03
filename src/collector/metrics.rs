use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
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

    if response.is_empty() {
        return Ok(());
    }

    // Query the latest timestamp for this metric (single query)
    let latest_timestamp = get_latest_timestamp(db, metric.name).await?;

    let now = Utc::now();
    let mut inserted_count = 0;

    for (timestamp, value) in response {
        let Some(dt) = Utc.timestamp_opt(timestamp, 0).single() else {
            warn!(timestamp = timestamp, "Invalid timestamp, skipping");
            continue;
        };

        // Skip if we already have this data point
        if let Some(latest) = latest_timestamp {
            if dt <= latest {
                continue;
            }
        }

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

    if inserted_count > 0 {
        debug!(
            metric = %metric.name,
            count = inserted_count,
            "Inserted metric data points"
        );
    }

    Ok(())
}

/// Get the latest timestamp for a specific metric
async fn get_latest_timestamp(
    db: &DatabaseConnection,
    metric_name: &str,
) -> Result<Option<DateTime<Utc>>> {
    let result = metric_logs::Entity::find()
        .filter(metric_logs::Column::MetricName.eq(metric_name))
        .order_by_desc(metric_logs::Column::Timestamp)
        .select_only()
        .column(metric_logs::Column::Timestamp)
        .into_tuple::<DateTime<Utc>>()
        .one(db)
        .await?;

    Ok(result)
}
