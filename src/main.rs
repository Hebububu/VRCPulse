mod collector;
mod commands;
mod config;
mod entity;
mod error;
mod logging;
mod state;
mod visualization;

use config::Config;
use error::Result;
use sea_orm::Database;
use serenity::all::{ActivityData, Client, EventHandler, GatewayIntents, Interaction, Ready};
use state::{AppState, AppStateKey};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Serenity event handler
struct Handler {
    /// Test guild ID (for development)
    test_guild_id: Option<u64>,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    /// Called when the bot connects to Discord
    async fn ready(&self, ctx: serenity::all::Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // Set bot activity status
        ctx.set_activity(Some(ActivityData::watching("VRChat Status")));

        // Register slash commands
        let result = match self.test_guild_id {
            Some(guild_id) => commands::register_guild(&ctx, guild_id).await,
            None => commands::register_global(&ctx).await,
        };

        if let Err(e) = result {
            error!("Failed to register commands: {:?}", e);
        }
    }

    /// Handle interactions (slash commands)
    async fn interaction_create(&self, ctx: serenity::all::Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let result = match command.data.name.as_str() {
                "hello" => commands::hello::run(&ctx, &command).await,
                // "admin" => commands::admin::config::run(&ctx, &command).await,
                "status" => commands::status::run(&ctx, &command).await,
                _ => Ok(()),
            };

            if let Err(e) = result {
                error!("Command error: {:?}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize logging
    logging::init();

    // 2. Load environment variables
    let config = Config::from_env()?;
    config.validate();

    info!("Starting VRCPulse...");

    // 3. Connect to database
    let database = Database::connect(&config.database_url).await?;
    info!("Database connected");

    // 4. Initialize collector config
    let (config_tx, config_rx) = collector::config::init(&database)
        .await
        .expect("Failed to load collector config from database");
    info!("Collector config loaded");

    // 5. Create AppState
    let app_state = Arc::new(RwLock::new(AppState::new(database.clone(), config_tx)));

    // 6. Start data collector in background
    let http_client = reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .expect("Failed to create HTTP client");

    tokio::spawn(collector::start(http_client, database, config_rx));

    // 7. Configure Discord client
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_MEMBERS;

    let handler = Handler {
        test_guild_id: config.test_guild_id,
    };

    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(handler)
        .await?;

    // 8. Store AppState in TypeMap
    {
        let mut data = client.data.write().await;
        data.insert::<AppStateKey>(app_state);
    }

    // 9. Start bot
    info!("Connecting to Discord...");
    if let Err(e) = client.start().await {
        error!("Client error: {:?}", e);
    }

    Ok(())
}
