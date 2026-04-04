//! Discord bot setup and event handling
//!
//! This module encapsulates all Discord-specific initialization and event handling.

mod handler;
pub mod intro;

pub use handler::Handler;

use serenity::all::{Client, GatewayIntents};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::config::Config;
use crate::error::Result;
use crate::state::{AppState, AppStateKey};
use vrcpulse_core::{DatabaseConfig, VrcPulseService, collector, connect_database};

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
    // 1. Connect to database with shared factory
    let db_config = DatabaseConfig::new(&config.database_url);
    let database = connect_database(db_config).await?;
    info!("Database connected (WAL mode enabled)");

    // 2. Initialize collector config
    let (config_tx, config_rx) = collector::config::init(&database)
        .await
        .expect("Failed to load collector config from database");
    info!("Collector config loaded");

    // 3. Create VrcPulseService and AppState
    let service = VrcPulseService::new(database.clone());
    let app_state = Arc::new(RwLock::new(AppState::new(service, config_tx)));

    // 4. Start data collector in background
    let http_client = create_http_client();
    tokio::spawn(collector::start(http_client, database, config_rx, None));

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
