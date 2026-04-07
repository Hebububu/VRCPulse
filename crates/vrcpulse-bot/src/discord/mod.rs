//! Discord bot setup and event handling
//!
//! This module encapsulates all Discord-specific initialization and event handling.

mod audit;
mod handler;
pub mod shared;

pub use handler::Handler;

use serenity::all::{Client, GatewayIntents};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::infrastructure::config::Config;
use crate::infrastructure::error::Result;
use crate::infrastructure::state::{AppState, AppStateKey};
use vrcpulse_core::{DatabaseConfig, VrcPulseService, connect_database};

/// Set up and configure the Discord bot client
///
/// This function handles all initialization:
/// - Database connection and migrations
/// - Discord client configuration
///
/// The bot is a pure data consumer. The web server handles data collection.
/// Returns a configured `Client` ready to be started.
pub async fn setup(config: &Config) -> Result<Client> {
    // 1. Connect to database with shared factory
    let db_config = DatabaseConfig::new(&config.database_url);
    let database = connect_database(db_config).await?;
    info!("Database connected (WAL mode enabled)");

    // 2. Run migrations
    use sea_orm_migration::MigratorTrait;
    migration::Migrator::up(&database, None)
        .await
        .expect("Failed to run migrations");
    info!("Migrations applied");

    // 3. Create VrcPulseService and AppState
    let service = VrcPulseService::new(database.clone());
    let app_state = Arc::new(RwLock::new(AppState::new(service)));

    // 4. Configure Discord client
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

    // 5. Store AppState in TypeMap
    {
        let mut data = client.data.write().await;
        data.insert::<AppStateKey>(app_state);
    }

    Ok(client)
}
