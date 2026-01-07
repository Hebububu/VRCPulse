mod alerts;
mod collector;
mod commands;
mod config;
mod database;
mod entity;
mod error;
mod logging;
mod repository;
mod state;
mod visualization;

use chrono::Utc;
use config::Config;
use error::Result;
use sea_orm::{ActiveModelTrait, ConnectOptions, ConnectionTrait, Database, Set};
use serenity::all::{
    ActivityData, Client, Colour, CommandInteraction, CreateEmbed, CreateEmbedFooter,
    CreateMessage, EventHandler, GatewayIntents, Guild, Interaction, Ready,
};
use state::{AppState, AppStateKey};

use crate::commands::shared::colors;

use crate::entity::command_logs;
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

    /// Handle interactions (slash commands and buttons)
    async fn interaction_create(&self, ctx: serenity::all::Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                // Log command request
                log_command(&ctx, &command).await;

                let result = match command.data.name.as_str() {
                    "hello" => commands::hello::run(&ctx, &command).await,
                    // "admin" => commands::admin::config::run(&ctx, &command).await,
                    "config" => commands::config::run(&ctx, &command).await,
                    "report" => commands::report::run(&ctx, &command).await,
                    "status" => commands::status::run(&ctx, &command).await,
                    _ => Ok(()),
                };

                if let Err(e) = result {
                    error!("Command error: {:?}", e);
                }
            }
            Interaction::Component(component) => {
                // Handle button interactions for /config unregister
                if component.data.custom_id.starts_with("config_")
                    && let Err(e) = commands::config::handle_button(&ctx, &component).await
                {
                    error!("Button interaction error: {:?}", e);
                }
            }
            _ => {}
        }
    }

    /// Called when bot joins a new guild
    async fn guild_create(&self, ctx: serenity::all::Context, guild: Guild, is_new: Option<bool>) {
        // Only send intro for newly joined guilds (not on reconnect)
        let is_new = is_new.unwrap_or(false);
        if !is_new {
            return;
        }

        info!(guild_id = %guild.id, guild_name = %guild.name, "Joined new guild");

        // Try to send intro message to system channel
        if let Some(system_channel_id) = guild.system_channel_id {
            let embed = create_intro_embed();
            let message = CreateMessage::new().embed(embed);

            if let Err(e) = system_channel_id.send_message(&ctx.http, message).await {
                error!(
                    guild_id = %guild.id,
                    error = %e,
                    "Failed to send intro message to system channel"
                );
            } else {
                info!(guild_id = %guild.id, "Sent intro message to system channel");
            }
        } else {
            // No system channel - users will discover the bot via /config show
            info!(
                guild_id = %guild.id,
                "No system channel configured, skipping intro message"
            );
        }
    }
}

/// Log command execution to console and database
async fn log_command(ctx: &serenity::all::Context, command: &CommandInteraction) {
    let command_name = &command.data.name;
    let user_id = command.user.id;
    let guild_id = command.guild_id;
    let channel_id = command.channel_id;

    // Extract subcommand if present
    let subcommand = command.data.options.first().and_then(|opt| {
        use serenity::all::CommandDataOptionValue;
        match &opt.value {
            CommandDataOptionValue::SubCommand(_) | CommandDataOptionValue::SubCommandGroup(_) => {
                Some(opt.name.as_str())
            }
            _ => None,
        }
    });

    // Console log
    info!(
        command = command_name,
        subcommand = subcommand,
        user_id = %user_id,
        guild_id = ?guild_id.map(|g| g.to_string()),
        channel_id = %channel_id,
        "Command received"
    );

    // Database audit log
    if let Some(db) = database::try_get_db(ctx).await {
        let log = command_logs::ActiveModel {
            command_name: Set(command_name.clone()),
            subcommand: Set(subcommand.map(|s| s.to_string())),
            user_id: Set(user_id.to_string()),
            guild_id: Set(guild_id.map(|g| g.to_string())),
            channel_id: Set(Some(channel_id.to_string())),
            executed_at: Set(Utc::now()),
            ..Default::default()
        };

        if let Err(e) = log.insert(&*db).await {
            error!(error = %e, "Failed to insert command log");
        }
    }
}

/// Create the introduction embed for new guilds
fn create_intro_embed() -> CreateEmbed {
    CreateEmbed::default()
        .title("Welcome to VRCPulse!")
        .description(
            "VRCPulse monitors VRChat server status and alerts you when issues occur.",
        )
        .color(Colour::new(colors::BRAND))
        .field(
            "Getting Started",
            "1. Run `/config setup #channel` to register this server\n2. Check current VRChat status with `/status`",
            false,
        )
        .field(
            "Commands",
            "- `/config setup <channel>` - Register and set alert channel\n- `/config show` - View current settings\n- `/status` - View VRChat status dashboard",
            false,
        )
        .footer(CreateEmbedFooter::new(
            "Thank you for adding VRCPulse to your server!",
        ))
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize logging
    logging::init();

    // 2. Load environment variables
    let config = Config::from_env()?;
    config.validate();

    info!("Starting VRCPulse...");

    // 3. Connect to database with optimized settings for SQLite
    let mut db_opts = ConnectOptions::new(&config.database_url);
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

    info!("Database connected (WAL mode enabled)");

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
