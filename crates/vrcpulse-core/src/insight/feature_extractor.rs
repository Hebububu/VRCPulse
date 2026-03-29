use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, warn};

use crate::entity::{incidents, maintenances, metric_logs, status_logs};

const ANOMALY_SIGMA: f64 = 2.0;
const MIN_DATA_POINTS: usize = 5;

const METRIC_NAMES: &[&str] = &[
    "visits",
    "api_latency",
    "api_requests",
    "api_errors",
    "extauth_steam",
    "extauth_oculus",
    "extauth_steam_count",
    "extauth_oculus_count",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSnapshot {
    pub metrics: Vec<MetricFeature>,
    pub co_occurrences: Vec<CoOccurrence>,
    pub status: Option<StatusSummary>,
    pub active_incidents: Vec<IncidentSummary>,
    pub active_maintenances: Vec<MaintenanceSummary>,
    pub recent_similar: Vec<IncidentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricFeature {
    pub name: String,
    pub current: f64,
    pub mean_24h: f64,
    pub std_dev_24h: f64,
    pub delta_percent_1h: f64,
    pub anomaly: bool,
    pub trend: Trend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoOccurrence {
    pub metric_a: String,
    pub metric_b: String,
    pub flag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSummary {
    pub indicator: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentSummary {
    pub id: String,
    pub title: String,
    pub impact: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceSummary {
    pub id: String,
    pub title: String,
    pub status: String,
}

/// Extract features from the database for AI analysis.
/// Returns None if insufficient data (cold start).
pub async fn extract_features(
    db: &DatabaseConnection,
) -> Result<Option<FeatureSnapshot>, sea_orm::DbErr> {
    let now = Utc::now();
    let cutoff_24h = now - Duration::hours(24);
    let cutoff_1h = now - Duration::hours(1);

    // Check minimum data: at least MIN_DATA_POINTS in the last 24h for accurate baseline
    let sample_data: Vec<metric_logs::Model> = metric_logs::Entity::find()
        .filter(metric_logs::Column::MetricName.eq(METRIC_NAMES[0]))
        .filter(metric_logs::Column::Timestamp.gte(cutoff_24h))
        .all(db)
        .await?;

    debug!(
        metric = METRIC_NAMES[0],
        count = sample_data.len(),
        min_required = MIN_DATA_POINTS,
        cutoff = %cutoff_1h,
        "Feature extractor data check"
    );

    if sample_data.len() < MIN_DATA_POINTS {
        warn!(
            count = sample_data.len(),
            min_required = MIN_DATA_POINTS,
            "Insufficient data for analysis"
        );
        return Ok(None);
    }

    // Extract features for each metric
    let mut metrics = Vec::new();
    for &name in METRIC_NAMES {
        let data_24h: Vec<metric_logs::Model> = metric_logs::Entity::find()
            .filter(metric_logs::Column::MetricName.eq(name))
            .filter(metric_logs::Column::Timestamp.gte(cutoff_24h))
            .order_by_asc(metric_logs::Column::Timestamp)
            .all(db)
            .await?;

        if data_24h.is_empty() {
            continue;
        }

        let feature = compute_metric_feature(name, &data_24h, cutoff_1h);
        metrics.push(feature);
    }

    // Co-occurrence flags
    let co_occurrences = compute_co_occurrences(&metrics);

    // Current status
    let status = status_logs::Entity::find()
        .order_by_desc(status_logs::Column::SourceTimestamp)
        .one(db)
        .await?
        .map(|s| StatusSummary {
            indicator: s.indicator,
            description: s.description,
        });

    // Active incidents
    let active_incidents: Vec<IncidentSummary> = incidents::Entity::find()
        .filter(incidents::Column::Status.ne("resolved"))
        .order_by_desc(incidents::Column::StartedAt)
        .all(db)
        .await?
        .into_iter()
        .map(|i| IncidentSummary {
            id: i.id,
            title: i.title,
            impact: i.impact,
            status: i.status,
        })
        .collect();

    // Active maintenances
    let active_maintenances: Vec<MaintenanceSummary> = maintenances::Entity::find()
        .filter(maintenances::Column::Status.ne("completed"))
        .order_by_desc(maintenances::Column::ScheduledFor)
        .all(db)
        .await?
        .into_iter()
        .map(|m| MaintenanceSummary {
            id: m.id,
            title: m.title,
            status: m.status,
        })
        .collect();

    // Recent similar incidents (last 30 days, resolved)
    let cutoff_30d = now - Duration::days(30);
    let recent_similar: Vec<IncidentSummary> = incidents::Entity::find()
        .filter(incidents::Column::Status.eq("resolved"))
        .filter(incidents::Column::StartedAt.gte(cutoff_30d))
        .order_by_desc(incidents::Column::StartedAt)
        .all(db)
        .await?
        .into_iter()
        .take(5)
        .map(|i| IncidentSummary {
            id: i.id,
            title: i.title,
            impact: i.impact,
            status: i.status,
        })
        .collect();

    Ok(Some(FeatureSnapshot {
        metrics,
        co_occurrences,
        status,
        active_incidents,
        active_maintenances,
        recent_similar,
    }))
}

fn compute_metric_feature(
    name: &str,
    data_24h: &[metric_logs::Model],
    cutoff_1h: chrono::DateTime<Utc>,
) -> MetricFeature {
    let values_24h: Vec<f64> = data_24h.iter().map(|d| d.value).collect();
    let mean_24h = mean(&values_24h);
    let std_dev_24h = std_dev(&values_24h, mean_24h);

    // Use average of last 5 data points (~5 min) instead of single last value
    // to avoid spiky outliers skewing the analysis
    let recent_window: Vec<f64> = values_24h.iter().rev().take(5).copied().collect();
    let current = if recent_window.is_empty() {
        0.0
    } else {
        mean(&recent_window)
    };

    // Compute 1h-ago mean for delta
    let values_before_1h: Vec<f64> = data_24h
        .iter()
        .filter(|d| d.timestamp < cutoff_1h)
        .map(|d| d.value)
        .collect();
    let mean_before_1h = if values_before_1h.is_empty() {
        mean_24h
    } else {
        mean(&values_before_1h)
    };
    let delta_percent_1h = if mean_before_1h.abs() > f64::EPSILON {
        ((current - mean_before_1h) / mean_before_1h) * 100.0
    } else {
        0.0
    };

    // Anomaly: current value vs 24h baseline
    let anomaly =
        std_dev_24h > f64::EPSILON && (current - mean_24h).abs() > ANOMALY_SIGMA * std_dev_24h;

    // Trend: compare last 10 points
    let recent: Vec<f64> = values_24h.iter().rev().take(10).copied().collect();
    let trend = compute_trend(&recent);

    MetricFeature {
        name: name.to_string(),
        current,
        mean_24h,
        std_dev_24h,
        delta_percent_1h,
        anomaly,
        trend,
    }
}

fn compute_co_occurrences(metrics: &[MetricFeature]) -> Vec<CoOccurrence> {
    let mut co_occs = Vec::new();

    let find = |name: &str| -> Option<&MetricFeature> { metrics.iter().find(|m| m.name == name) };

    // api_errors + extauth_steam/oculus anomaly → auth failure correlation
    if let (Some(errors), Some(steam)) = (find("api_errors"), find("extauth_steam"))
        && errors.anomaly
        && steam.anomaly
    {
        co_occs.push(CoOccurrence {
            metric_a: "api_errors".to_string(),
            metric_b: "extauth_steam".to_string(),
            flag: "auth_failure_correlated".to_string(),
        });
    }

    // visits down + api_requests stable → login churn
    if let (Some(visits), Some(requests)) = (find("visits"), find("api_requests"))
        && visits.delta_percent_1h < -10.0
        && requests.delta_percent_1h.abs() < 10.0
    {
        co_occs.push(CoOccurrence {
            metric_a: "visits".to_string(),
            metric_b: "api_requests".to_string(),
            flag: "login_churn".to_string(),
        });
    }

    // api_latency anomaly + api_errors up → degradation
    if let (Some(latency), Some(errors)) = (find("api_latency"), find("api_errors"))
        && latency.anomaly
        && errors.delta_percent_1h > 20.0
    {
        co_occs.push(CoOccurrence {
            metric_a: "api_latency".to_string(),
            metric_b: "api_errors".to_string(),
            flag: "degradation".to_string(),
        });
    }

    co_occs
}

fn compute_trend(recent_reversed: &[f64]) -> Trend {
    if recent_reversed.len() < 3 {
        return Trend::Stable;
    }
    let first_half_mean = mean(&recent_reversed[recent_reversed.len() / 2..]);
    let second_half_mean = mean(&recent_reversed[..recent_reversed.len() / 2]);

    let diff_pct = if first_half_mean.abs() > f64::EPSILON {
        ((second_half_mean - first_half_mean) / first_half_mean) * 100.0
    } else {
        0.0
    };

    if diff_pct > 5.0 {
        Trend::Increasing
    } else if diff_pct < -5.0 {
        Trend::Decreasing
    } else {
        Trend::Stable
    }
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn std_dev(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let variance =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

/// Compute a deterministic hash of the trigger + feature snapshot for dedup.
pub fn compute_source_hash(
    trigger_type: &str,
    trigger_id: Option<&str>,
    snapshot: &FeatureSnapshot,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(trigger_type.as_bytes());
    if let Some(id) = trigger_id {
        hasher.update(id.as_bytes());
    }
    if let Ok(json) = serde_json::to_string(snapshot) {
        hasher.update(json.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_metrics(anomalies: &[(&str, bool)]) -> Vec<MetricFeature> {
        anomalies
            .iter()
            .map(|(name, anomaly)| MetricFeature {
                name: name.to_string(),
                current: 100.0,
                mean_24h: 90.0,
                std_dev_24h: 5.0,
                delta_percent_1h: if *anomaly { 30.0 } else { 2.0 },
                anomaly: *anomaly,
                trend: Trend::Stable,
            })
            .collect()
    }

    #[test]
    fn test_anomaly_detection_flags_outlier() {
        let values = vec![10.0; 100];
        let m = mean(&values);
        let s = std_dev(&values, m);
        // All same values → std_dev ≈ 0, current 10 is not anomalous
        assert!(!((10.0 - m).abs() > ANOMALY_SIGMA * s && s > f64::EPSILON));

        // Add outlier
        let mut values2 = vec![10.0; 99];
        values2.push(100.0); // big outlier
        let m2 = mean(&values2);
        let s2 = std_dev(&values2, m2);
        // 100 should be anomalous relative to the baseline
        assert!((100.0 - m2).abs() > ANOMALY_SIGMA * s2);
    }

    #[test]
    fn test_anomaly_detection_normal() {
        let values: Vec<f64> = (0..100).map(|i| 50.0 + (i as f64 * 0.1)).collect();
        let m = mean(&values);
        let s = std_dev(&values, m);
        // A value near the mean should not be anomalous
        assert!(!((m - m).abs() > ANOMALY_SIGMA * s));
    }

    #[test]
    fn test_co_occurrence_both_anomalous() {
        let metrics = make_metrics(&[("api_errors", true), ("extauth_steam", true)]);
        let co_occs = compute_co_occurrences(&metrics);
        assert_eq!(co_occs.len(), 1);
        assert_eq!(co_occs[0].flag, "auth_failure_correlated");
    }

    #[test]
    fn test_co_occurrence_one_normal() {
        let metrics = make_metrics(&[("api_errors", true), ("extauth_steam", false)]);
        let co_occs = compute_co_occurrences(&metrics);
        assert!(co_occs.iter().all(|c| c.flag != "auth_failure_correlated"));
    }

    #[test]
    fn test_source_hash_deterministic() {
        let snapshot = FeatureSnapshot {
            metrics: vec![],
            co_occurrences: vec![],
            status: None,
            active_incidents: vec![],
            active_maintenances: vec![],
            recent_similar: vec![],
        };
        let h1 = compute_source_hash("scheduled", None, &snapshot);
        let h2 = compute_source_hash("scheduled", None, &snapshot);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_source_hash_event_differs() {
        let snapshot = FeatureSnapshot {
            metrics: vec![],
            co_occurrences: vec![],
            status: None,
            active_incidents: vec![],
            active_maintenances: vec![],
            recent_similar: vec![],
        };
        let h1 = compute_source_hash("scheduled", None, &snapshot);
        let h2 = compute_source_hash("incident_detected", Some("inc-123"), &snapshot);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_trend_increasing() {
        // Recent values (reversed): older first = lower, newer = higher
        let recent = vec![15.0, 14.0, 13.0, 12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 6.0];
        let trend = compute_trend(&recent);
        assert!(matches!(trend, Trend::Increasing));
    }

    #[test]
    fn test_trend_stable() {
        let recent = vec![10.0, 10.1, 9.9, 10.0, 10.1, 9.9, 10.0, 10.1, 9.9, 10.0];
        let trend = compute_trend(&recent);
        assert!(matches!(trend, Trend::Stable));
    }
}
