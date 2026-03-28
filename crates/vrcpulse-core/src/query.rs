//! Data query helpers for visualization
//!
//! Loads metric data from SQLite and performs downsampling.

use chrono::{DateTime, Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::entity::metric_logs;

/// Default time range in hours for metric queries
pub const DEFAULT_HOURS_RANGE: i64 = 12;
/// Default downsampling interval in minutes
pub const DEFAULT_DOWNSAMPLE_MINUTES: i64 = 5;

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
    let cutoff = Utc::now() - Duration::hours(DEFAULT_HOURS_RANGE);

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

    let interval = Duration::minutes(DEFAULT_DOWNSAMPLE_MINUTES);
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_data(values: Vec<f64>, minutes_apart: i64) -> MetricData {
        let base = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let timestamps = values
            .iter()
            .enumerate()
            .map(|(i, _)| base + Duration::minutes(i as i64 * minutes_apart))
            .collect();
        MetricData {
            timestamps,
            values,
            unit: "count".to_string(),
        }
    }

    #[test]
    fn test_downsample_empty() {
        let data = MetricData {
            timestamps: vec![],
            values: vec![],
            unit: "count".to_string(),
        };
        let result = downsample(data);
        assert!(result.values.is_empty());
        assert!(result.timestamps.is_empty());
    }

    #[test]
    fn test_downsample_single_point() {
        let data = make_data(vec![42.0], 1);
        let result = downsample(data);
        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values[0], 42.0);
    }

    #[test]
    fn test_downsample_multiple_points() {
        // 10 points, 1 minute apart. DEFAULT_DOWNSAMPLE_MINUTES=5, so should produce 2 buckets.
        let data = make_data(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0], 1);
        let result = downsample(data);
        assert_eq!(result.values.len(), 2);
        // First bucket: avg(1,2,3,4,5) = 3.0
        assert!((result.values[0] - 3.0).abs() < 0.01);
        // Second bucket: avg(6,7,8,9,10) = 8.0
        assert!((result.values[1] - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_to_percent() {
        let data = make_data(vec![0.0, 0.5, 1.0], 1);
        let result = to_percent(data);
        assert_eq!(result.values, vec![0.0, 50.0, 100.0]);
    }

    #[test]
    fn test_metric_data_is_empty() {
        let empty = MetricData {
            timestamps: vec![],
            values: vec![],
            unit: String::new(),
        };
        assert!(empty.is_empty());

        let non_empty = make_data(vec![1.0], 1);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_metric_data_avg() {
        let empty = MetricData {
            timestamps: vec![],
            values: vec![],
            unit: String::new(),
        };
        assert_eq!(empty.avg(), 0.0);

        let data = make_data(vec![10.0, 20.0, 30.0], 1);
        assert!((data.avg() - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_metric_data_max() {
        let data = make_data(vec![5.0, 15.0, 10.0], 1);
        assert_eq!(data.max(), 15.0);
    }
}
