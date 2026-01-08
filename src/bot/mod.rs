//! Discord bot setup and event handling
//!
//! This module encapsulates all Discord-specific initialization and event handling.

mod handler;
pub mod intro;

pub use handler::Handler;

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use serenity::all::{Client, GatewayIntents};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::collector;
use crate::config::Config;
use crate::error::Result;
use crate::state::{AppState, AppStateKey};

/// Set up and configure the Discord bot client
///
/// This function handles all initialization:
/// - Database connection
/// - Collector config initialization
/// - HTTP client creation
/// - Background collector task spawning
/// - Discord client configuration
///
/// Returns a configured `Client` ready to be started.
pub async fn setup(config: &Config) -> Result<Client> {
    // 1. Connect to database with optimized settings for SQLite
    let database = connect_database(&config.database_url).await?;
    info!("Database connected (WAL mode enabled)");

    // 2. Initialize collector config
    let (config_tx, config_rx) = collector::config::init(&database)
        .await
        .expect("Failed to load collector config from database");
    info!("Collector config loaded");

    // 3. Create AppState
    let app_state = Arc::new(RwLock::new(AppState::new(database.clone(), config_tx)));

    // 4. Start data collector in background
    let http_client = create_http_client();
    tokio::spawn(collector::start(http_client, database, config_rx));

    // 5. Configure Discord client
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_MEMBERS;

    let handler = Handler {
        test_guild_id: config.test_guild_id,
    };

    let client = Client::builder(&config.discord_token, intents)
        .event_handler(handler)
        .await?;

    // 6. Store AppState in TypeMap
    {
        let mut data = client.data.write().await;
        data.insert::<AppStateKey>(app_state);
    }

    Ok(client)
}

/// Connect to database with optimized settings for SQLite
async fn connect_database(database_url: &str) -> Result<DatabaseConnection> {
    let mut db_opts = ConnectOptions::new(database_url);
    db_opts
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .sqlx_logging(false); // Reduce noise, enable if debugging

    let database = Database::connect(db_opts).await?;

    // Enable WAL mode for better concurrency
    database
        .execute_unprepared("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
        .await
        .expect("Failed to set SQLite pragmas");

    Ok(database)
}

/// Create HTTP client for API requests
fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .expect("Failed to create HTTP client")
}
