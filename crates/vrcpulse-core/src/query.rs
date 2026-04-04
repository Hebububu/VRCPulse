//! Data query helpers for visualization
//!
//! Loads metric data from SQLite and performs downsampling.

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::entity::{component_logs, metric_logs};

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

/// Default number of buckets for component status history bars
pub const DEFAULT_BUCKET_COUNT: usize = 90;

/// Bucketed component status history for rendering status bars
#[derive(Debug, Clone)]
pub struct ComponentBuckets {
    pub component_id: String,
    pub name: String,
    pub current_status: String,
    pub buckets: Vec<String>,
}

/// Load component history from database within a time range
pub async fn load_component_history(
    db: &DatabaseConnection,
    hours: i64,
) -> Result<Vec<component_logs::Model>, sea_orm::DbErr> {
    let cutoff = Utc::now() - Duration::hours(hours);

    component_logs::Entity::find()
        .filter(component_logs::Column::SourceTimestamp.gte(cutoff))
        .order_by_asc(component_logs::Column::ComponentId)
        .order_by_asc(component_logs::Column::SourceTimestamp)
        .all(db)
        .await
}

/// Load the latest status per component (not range-limited)
pub async fn load_latest_component_statuses(
    db: &DatabaseConnection,
) -> Result<Vec<component_logs::Model>, sea_orm::DbErr> {
    // Get all component logs ordered by timestamp desc, then deduplicate by component_id
    let all = component_logs::Entity::find()
        .order_by_desc(component_logs::Column::SourceTimestamp)
        .all(db)
        .await?;

    let mut seen = std::collections::HashSet::new();
    let latest: Vec<component_logs::Model> = all
        .into_iter()
        .filter(|m| seen.insert(m.component_id.clone()))
        .collect();

    Ok(latest)
}

/// Status severity ranking (higher = worse)
fn status_severity(status: &str) -> u8 {
    match status {
        "operational" => 0,
        "degraded_performance" => 1,
        "partial_outage" => 2,
        "major_outage" => 3,
        _ => 0, // unknown statuses treated as operational for "worst" comparison
    }
}

/// Bucket component history into fixed-size time slots.
///
/// Each bucket represents a time window. The bucket value is the worst status
/// seen in that window. Buckets with no data are set to "unknown".
pub fn bucket_component_history(
    history: &[component_logs::Model],
    latest_statuses: &[component_logs::Model],
    range_start: DateTime<Utc>,
    range_end: DateTime<Utc>,
    bucket_count: usize,
) -> Vec<ComponentBuckets> {
    let total_duration = (range_end - range_start).num_seconds() as f64;
    let bucket_duration = total_duration / bucket_count as f64;

    // Group history by component_id
    let mut grouped: HashMap<String, Vec<&component_logs::Model>> = HashMap::new();
    for model in history {
        grouped
            .entry(model.component_id.clone())
            .or_default()
            .push(model);
    }

    // Build a map of latest statuses by component_id
    let mut latest_map: HashMap<String, &component_logs::Model> = HashMap::new();
    for model in latest_statuses {
        latest_map
            .entry(model.component_id.clone())
            .or_insert(model);
    }

    // Ensure all known components appear (even if no history in range)
    for model in latest_statuses {
        grouped.entry(model.component_id.clone()).or_default();
    }

    let mut results: Vec<ComponentBuckets> = grouped
        .into_iter()
        .map(|(component_id, entries)| {
            let name = entries
                .first()
                .map(|e| e.name.clone())
                .or_else(|| latest_map.get(&component_id).map(|m| m.name.clone()))
                .unwrap_or_default();

            let current_status = latest_map
                .get(&component_id)
                .map(|m| m.status.clone())
                .unwrap_or_else(|| "unknown".to_string());

            let mut buckets = vec!["unknown".to_string(); bucket_count];

            for entry in &entries {
                let offset = (entry.source_timestamp - range_start).num_seconds() as f64;
                let idx = (offset / bucket_duration) as usize;
                if idx < bucket_count {
                    let current = &buckets[idx];
                    if current == "unknown"
                        || status_severity(&entry.status) > status_severity(current)
                    {
                        buckets[idx] = entry.status.clone();
                    }
                }
            }

            ComponentBuckets {
                component_id,
                name,
                current_status,
                buckets,
            }
        })
        .collect();

    results.sort_by(|a, b| a.name.cmp(&b.name));
    results
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

    // --- Component bucketing tests ---

    fn make_component_log(
        component_id: &str,
        name: &str,
        status: &str,
        ts: DateTime<Utc>,
    ) -> component_logs::Model {
        component_logs::Model {
            id: 0,
            component_id: component_id.to_string(),
            name: name.to_string(),
            status: status.to_string(),
            source_timestamp: ts,
            created_at: ts,
        }
    }

    #[test]
    fn test_bucket_empty_input() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = start + Duration::hours(24);
        let result = bucket_component_history(&[], &[], start, end, 90);
        assert!(result.is_empty());
    }

    #[test]
    fn test_bucket_single_component_all_operational() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = start + Duration::hours(24);

        // Create entries every 16 minutes (90 buckets over 24h = ~16 min each)
        let history: Vec<component_logs::Model> = (0..90)
            .map(|i| {
                make_component_log(
                    "c1",
                    "API",
                    "operational",
                    start + Duration::minutes(i * 16),
                )
            })
            .collect();

        let latest = vec![make_component_log("c1", "API", "operational", end)];

        let result = bucket_component_history(&history, &latest, start, end, 90);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "API");
        assert_eq!(result[0].current_status, "operational");
        assert!(result[0].buckets.iter().all(|b| b == "operational"));
    }

    #[test]
    fn test_bucket_worst_status_wins() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = start + Duration::hours(24);
        let bucket_duration_secs = (24 * 3600) / 90; // ~960 seconds per bucket

        // Two entries in the same bucket: operational and major_outage
        let ts1 = start + Duration::seconds(10);
        let ts2 = start + Duration::seconds(bucket_duration_secs as i64 / 2);

        let history = vec![
            make_component_log("c1", "API", "operational", ts1),
            make_component_log("c1", "API", "major_outage", ts2),
        ];
        let latest = vec![make_component_log("c1", "API", "operational", end)];

        let result = bucket_component_history(&history, &latest, start, end, 90);
        assert_eq!(result[0].buckets[0], "major_outage");
    }

    #[test]
    fn test_bucket_gaps_are_unknown() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = start + Duration::hours(24);

        // Only one entry at the very start
        let history = vec![make_component_log(
            "c1",
            "API",
            "operational",
            start + Duration::seconds(10),
        )];
        let latest = vec![make_component_log("c1", "API", "operational", end)];

        let result = bucket_component_history(&history, &latest, start, end, 90);
        assert_eq!(result[0].buckets[0], "operational");
        // All other buckets should be "unknown"
        assert!(result[0].buckets[1..].iter().all(|b| b == "unknown"));
    }

    #[test]
    fn test_bucket_multiple_components_sorted() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = start + Duration::hours(24);
        let ts = start + Duration::seconds(10);

        let history = vec![
            make_component_log("c2", "Website", "operational", ts),
            make_component_log("c1", "API", "operational", ts),
        ];
        let latest = vec![
            make_component_log("c2", "Website", "operational", end),
            make_component_log("c1", "API", "operational", end),
        ];

        let result = bucket_component_history(&history, &latest, start, end, 90);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "API");
        assert_eq!(result[1].name, "Website");
    }

    #[test]
    fn test_bucket_component_no_history_in_range() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = start + Duration::hours(24);

        // No history entries, but latest status exists
        let latest = vec![make_component_log("c1", "API", "degraded_performance", end)];

        let result = bucket_component_history(&[], &latest, start, end, 90);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].current_status, "degraded_performance");
        assert!(result[0].buckets.iter().all(|b| b == "unknown"));
    }
}
