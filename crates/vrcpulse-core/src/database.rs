//! Shared database connection factory
//!
//! Provides a single `connect_database()` function used by both the bot and server
//! to establish a SQLite connection with consistent settings (WAL mode, pool config).

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};

use crate::error::Result;

/// Database connection configuration
pub struct DatabaseConfig {
    /// SQLite connection URL (e.g., "sqlite://data/vrcpulse.db")
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection acquire timeout in seconds
    pub acquire_timeout_secs: u64,
}

impl DatabaseConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 5,
            min_connections: 1,
            acquire_timeout_secs: 10,
        }
    }
}

/// Connect to the database with optimized settings for SQLite
///
/// Configures WAL mode, busy timeout, and connection pooling.
/// Returns `Result` instead of panicking on failure.
pub async fn connect_database(config: DatabaseConfig) -> Result<DatabaseConnection> {
    let mut db_opts = ConnectOptions::new(&config.url);
    db_opts
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_secs))
        .sqlx_logging(false);

    let database = Database::connect(db_opts).await?;

    // Enable WAL mode for better concurrency and set busy timeout
    database
        .execute_unprepared("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
        .await?;

    Ok(database)
}
