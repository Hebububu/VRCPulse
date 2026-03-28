use chrono::{DateTime, Utc};
use serde::Deserialize;

// =============================================================================
// VRChat Status API (Atlassian Statuspage) Response Types
// Base URL: https://status.vrchat.com/api/v2
// =============================================================================

/// Response from /summary.json
#[derive(Debug, Deserialize)]
pub struct SummaryResponse {
    pub page: PageInfo,
    pub status: StatusInfo,
    pub components: Vec<Component>,
}

#[derive(Debug, Deserialize)]
pub struct PageInfo {
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct StatusInfo {
    /// none | minor | major | critical
    pub indicator: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    /// operational | degraded_performance | partial_outage | major_outage
    pub status: String,
}

/// Response from /incidents/unresolved.json and /incidents.json
#[derive(Debug, Deserialize)]
pub struct UnresolvedIncidentsResponse {
    pub incidents: Vec<Incident>,
}

/// Also used for /incidents.json (same shape)
pub type IncidentsResponse = UnresolvedIncidentsResponse;

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct Incident {
    pub id: String,
    pub name: String,
    /// investigating | identified | monitoring | resolved
    pub status: String,
    /// none | minor | major | critical
    pub impact: String,
    pub started_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub incident_updates: Vec<IncidentUpdate>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct IncidentUpdate {
    pub id: String,
    pub status: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

/// Response from /scheduled-maintenances/upcoming.json and /scheduled-maintenances/active.json
#[derive(Debug, Deserialize)]
pub struct MaintenancesResponse {
    pub scheduled_maintenances: Vec<Maintenance>,
}

#[derive(Debug, Deserialize)]
pub struct Maintenance {
    pub id: String,
    pub name: String,
    /// scheduled | in_progress | completed
    pub status: String,
    pub scheduled_for: DateTime<Utc>,
    pub scheduled_until: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// CloudFront Metrics API Response Types
// Base URL: https://d31qqo63tn8lj0.cloudfront.net
// =============================================================================

/// Single metric data point: [unix_timestamp, value]
pub type MetricDataPoint = (i64, f64);

/// Response from CloudFront metrics endpoints (array of [timestamp, value])
pub type MetricsResponse = Vec<MetricDataPoint>;

/// Metric type definition
#[derive(Debug, Clone, Copy)]
pub struct MetricDefinition {
    pub endpoint: &'static str,
    pub name: &'static str,
    pub unit: &'static str,
    /// Multiply raw value by this factor before storing (e.g. 1000.0 for seconds → ms)
    pub scale: f64,
}

/// All available CloudFront metrics
/// Note: CloudFront API returns latency in seconds and rates as 0-1 fractions.
/// Scale factors convert to human-friendly units on ingest.
pub const CLOUDFRONT_METRICS: &[MetricDefinition] = &[
    MetricDefinition {
        endpoint: "/apilatency.json",
        name: "api_latency",
        unit: "ms",
        scale: 1000.0, // seconds → milliseconds
    },
    MetricDefinition {
        endpoint: "/visits.json",
        name: "visits",
        unit: "count",
        scale: 1.0,
    },
    MetricDefinition {
        endpoint: "/apirequests.json",
        name: "api_requests",
        unit: "req/s",
        scale: 1.0, // already requests per second
    },
    MetricDefinition {
        endpoint: "/apierrors.json",
        name: "api_errors",
        unit: "%",
        scale: 100.0, // fraction → percentage
    },
    MetricDefinition {
        endpoint: "/extauth_steam.json",
        name: "extauth_steam",
        unit: "%",
        scale: 100.0, // fraction → percentage (success rate)
    },
    MetricDefinition {
        endpoint: "/extauth_oculus.json",
        name: "extauth_oculus",
        unit: "%",
        scale: 100.0, // fraction → percentage (success rate)
    },
    MetricDefinition {
        endpoint: "/extauth_steam_count.json",
        name: "extauth_steam_count",
        unit: "%",
        scale: 100.0, // fraction → percentage
    },
    MetricDefinition {
        endpoint: "/extauth_oculus_count.json",
        name: "extauth_oculus_count",
        unit: "%",
        scale: 100.0, // fraction → percentage
    },
];
