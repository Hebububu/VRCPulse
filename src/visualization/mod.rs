//! Visualization module for generating charts and dashboards
//!
//! This module provides functionality to generate PNG charts from metric data
//! stored in SQLite, for embedding in Discord messages.

pub mod dashboard;
pub mod query;
pub mod theme;

pub use dashboard::{DashboardStats, YAxisFormat, generate_dashboard};
pub use query::{MetricData, load_metric, load_metric_as_percent, load_metric_downsampled};
