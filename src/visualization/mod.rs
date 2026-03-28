//! Visualization module for generating charts and dashboards
//!
//! This module provides functionality to generate PNG charts from metric data
//! stored in SQLite, for embedding in Discord messages.

pub mod dashboard;
pub mod query;
pub mod theme;

pub use dashboard::generate_dashboard;
