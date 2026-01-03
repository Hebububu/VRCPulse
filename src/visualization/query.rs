//! Data query helpers for visualization
//!
//! Loads metric data from SQLite and performs downsampling.

use chrono::{DateTime, Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::entity::metric_logs;
use crate::visualization::theme::{DOWNSAMPLE_MINUTES, HOURS_RANGE};

/// Metric data for chart rendering
#[derive(Debug, Clone)]
pub struct MetricData {
    pub timestamps: Vec<DateTime<Utc>>,
    pub values: Vec<f64>,
    pub unit: String,
}

impl MetricData {
    /// Check if data is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get average value
    pub fn avg(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.values.iter().sum::<f64>() / self.values.len() as f64
        }
    }

    /// Get maximum value
    pub fn max(&self) -> f64 {
        self.values.iter().cloned().fold(0.0_f64, f64::max)
    }
}

/// Load metric data from database
pub async fn load_metric(
    db: &DatabaseConnection,
    metric_name: &str,
) -> Result<MetricData, sea_orm::DbErr> {
    let cutoff = Utc::now() - Duration::hours(HOURS_RANGE);

    let data: Vec<metric_logs::Model> = metric_logs::Entity::find()
        .filter(metric_logs::Column::MetricName.eq(metric_name))
        .filter(metric_logs::Column::Timestamp.gte(cutoff))
        .order_by_asc(metric_logs::Column::Timestamp)
        .all(db)
        .await?;

    let timestamps: Vec<DateTime<Utc>> = data.iter().map(|d| d.timestamp).collect();
    let values: Vec<f64> = data.iter().map(|d| d.value).collect();
    let unit = data.first().map(|d| d.unit.clone()).unwrap_or_default();

    Ok(MetricData {
        timestamps,
        values,
        unit,
    })
}

/// Downsample data by averaging over intervals
pub fn downsample(data: MetricData) -> MetricData {
    if data.values.is_empty() {
        return data;
    }

    let interval = Duration::minutes(DOWNSAMPLE_MINUTES);
    let mut downsampled_timestamps = Vec::new();
    let mut downsampled_values = Vec::new();

    let mut bucket_start = data.timestamps[0];
    let mut bucket_values: Vec<f64> = Vec::new();

    for (ts, val) in data.timestamps.iter().zip(data.values.iter()) {
        if *ts >= bucket_start + interval {
            // Save current bucket average
            if !bucket_values.is_empty() {
                let avg = bucket_values.iter().sum::<f64>() / bucket_values.len() as f64;
                downsampled_timestamps.push(bucket_start + interval / 2);
                downsampled_values.push(avg);
            }
            // Start new bucket
            bucket_start = *ts;
            bucket_values.clear();
        }
        bucket_values.push(*val);
    }

    // Don't forget last bucket (use center timestamp for consistency)
    if !bucket_values.is_empty() {
        let avg = bucket_values.iter().sum::<f64>() / bucket_values.len() as f64;
        downsampled_timestamps.push(bucket_start + interval / 2);
        downsampled_values.push(avg);
    }

    MetricData {
        timestamps: downsampled_timestamps,
        values: downsampled_values,
        unit: data.unit,
    }
}

/// Convert 0-1 values to 0-100 percentage
pub fn to_percent(mut data: MetricData) -> MetricData {
    data.values = data.values.iter().map(|v| v * 100.0).collect();
    data
}

/// Load and process metric data (load + downsample)
pub async fn load_metric_downsampled(
    db: &DatabaseConnection,
    metric_name: &str,
) -> Result<MetricData, sea_orm::DbErr> {
    let data = load_metric(db, metric_name).await?;
    Ok(downsample(data))
}

/// Load and process metric as percentage (load + downsample + to_percent)
pub async fn load_metric_as_percent(
    db: &DatabaseConnection,
    metric_name: &str,
) -> Result<MetricData, sea_orm::DbErr> {
    let data = load_metric(db, metric_name).await?;
    Ok(to_percent(downsample(data)))
}
