use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::collector::CollectorConfigTx;

/// TypeMap key for AppState access
pub struct AppStateKey;

impl serenity::prelude::TypeMapKey for AppStateKey {
    type Value = Arc<RwLock<AppState>>;
}

/// Application global state
/// - Accessible via `TypeMap` in Serenity event handlers
#[derive(Clone)]
pub struct AppState {
    /// Database connection
    pub database: Arc<DatabaseConnection>,
    /// Collector config sender for dynamic interval updates
    pub collector_config: CollectorConfigTx,
    /// Bot startup timestamp
    pub started_at: DateTime<Utc>,
}

impl AppState {
    /// Create a new AppState instance
    pub fn new(database: DatabaseConnection, collector_config: CollectorConfigTx) -> Self {
        Self {
            database: Arc::new(database),
            collector_config,
            started_at: Utc::now(),
        }
    }
}
