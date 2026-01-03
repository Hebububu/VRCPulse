use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// Application global state
/// - Accessible via `TypeMap` in Serenity event handlers
#[derive(Clone)]
pub struct AppState {
    /// Database connection
    pub database: Arc<DatabaseConnection>,
    /// HTTP client for VRChat API calls
    pub http_client: reqwest::Client,
}

impl AppState {
    /// Create a new AppState instance
    pub fn new(database: DatabaseConnection) -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            database: Arc::new(database),
            http_client,
        }
    }
}
