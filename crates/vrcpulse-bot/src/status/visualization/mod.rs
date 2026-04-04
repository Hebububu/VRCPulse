//! Visualization module for generating charts and dashboards
//!
//! This module provides functionality to generate PNG charts from metric data,
//! for embedding in Discord messages. Uses VrcPulseService for data access.

pub mod dashboard;
pub mod theme;

pub use dashboard::generate_dashboard;
