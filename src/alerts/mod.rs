//! Alert system for VRCPulse
//!
//! Handles threshold-based alerts when multiple users report the same issue.
//!
//! ## Status Field Lifecycle
//!
//! The `user_reports.status` field uses the following values:
//! - `active`: Report is within the time window, can contribute to threshold alerts
//! - `counted`: Report was included in a threshold alert (future use)
//! - `expired`: Time window passed without triggering alert (future use)
//!
//! Currently, only `active` is used. Status transitions (`counted`, `expired`)
//! are reserved for future implementation of report lifecycle management.

pub mod threshold;

pub use threshold::check_and_send_alerts;
