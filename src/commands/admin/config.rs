//! /admin command - Bot owner only administration

use chrono::Utc;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, Permissions, ResolvedValue,
};
use tracing::error;

use crate::collector::config::{DEFAULT_INTERVAL, PollerType, get_interval, validate_interval};
use crate::commands::shared::respond_error;
use crate::database;
use crate::repository::{GuildConfigRepository, UserConfigRepository};
use crate::state::AppStateKey;

use super::embeds;

// =============================================================================
// Command Registration
// =============================================================================

/// /admin command definition
pub fn register() -> CreateCommand {
    CreateCommand::new("admin")
        .description("Bot owner commands")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "show",
            "Display bot information and available commands",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommandGroup,
                "config",
                "Manage bot configuration",
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "show",
                "Display current polling interval settings",
            ))
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "set",
                    "Update a poller's interval",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "poller",
                        "The poller to configure",
                    )
                    .required(true)
                    .add_string_choice("status", "status")
                    .add_string_choice("incident", "incident")
                    .add_string_choice("maintenance", "maintenance")
                    .add_string_choice("metrics", "metrics"),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "seconds",
                        "Interval in seconds (60-3600)",
                    )
                    .required(true)
                    .min_int_value(60)
                    .max_int_value(3600),
                ),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "reset",
                "Reset all polling intervals to default (60s)",
            )),
        )
}

// =============================================================================
// Command Handler
// =============================================================================

/// /admin command handler (owner-only)
pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    // Check if user is bot owner (silent ignore if not)
    if !is_owner(ctx, interaction).await {
        return Ok(());
    }

    let db = database::get_db(ctx).await;

    // Parse subcommand
    let options = &interaction.data.options();
    let Some(first_opt) = options.first() else {
        return Ok(());
    };

    match first_opt.name {
        "show" => handle_admin_show(ctx, interaction).await,
        "config" => {
            let ResolvedValue::SubCommandGroup(subcommands) = &first_opt.value else {
                return respond_error(ctx, interaction, "Invalid command structure").await;
            };

            let Some(subcommand) = subcommands.first() else {
                return respond_error(ctx, interaction, "Missing subcommand").await;
            };

            match subcommand.name {
                "show" => handle_config_show(ctx, interaction, &db).await,
                "set" => {
                    let ResolvedValue::SubCommand(options) = &subcommand.value else {
                        return respond_error(ctx, interaction, "Invalid command structure").await;
                    };
                    handle_config_set(ctx, interaction, &db, options).await
                }
                "reset" => handle_config_reset(ctx, interaction, &db).await,
                _ => Ok(()),
            }
        }
        _ => Ok(()),
    }
}

// =============================================================================
// Owner Check
// =============================================================================

/// Check if the user is the bot owner
async fn is_owner(ctx: &Context, interaction: &CommandInteraction) -> bool {
    match ctx.http.get_current_application_info().await {
        Ok(app_info) => app_info
            .owner
            .as_ref()
            .is_some_and(|owner| owner.id == interaction.user.id),
        Err(e) => {
            error!(error = %e, "Failed to get application info for owner check");
            false
        }
    }
}

// =============================================================================
// Admin Show Handler
// =============================================================================

/// Handle /admin show - display bot info and command summary
async fn handle_admin_show(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<(), serenity::Error> {
    let db = database::get_db(ctx).await;

    // Get uptime from AppState
    let uptime = {
        let data = ctx.data.read().await;
        let state = data.get::<AppStateKey>().expect("AppState not found");
        let started_at = state.read().await.started_at;
        format_uptime(started_at)
    };

    // Get counts
    let guild_count = ctx.cache.guild_count() as u64;
    let registered_guilds = GuildConfigRepository::new(db.clone())
        .count_enabled()
        .await
        .unwrap_or(0);
    let registered_users = UserConfigRepository::new(db.clone())
        .count_enabled()
        .await
        .unwrap_or(0);

    // Get polling intervals
    let format_interval = |result: Result<u64, _>| match result {
        Ok(secs) => format!("{}s", secs),
        Err(_) => "Error".to_string(),
    };

    let status_interval = format_interval(get_interval(&db, PollerType::Status).await);
    let incident_interval = format_interval(get_interval(&db, PollerType::Incident).await);
    let maintenance_interval = format_interval(get_interval(&db, PollerType::Maintenance).await);
    let metrics_interval = format_interval(get_interval(&db, PollerType::Metrics).await);

    let embed = embeds::admin_show(
        env!("CARGO_PKG_VERSION"),
        &uptime,
        guild_count,
        registered_guilds,
        registered_users,
        &status_interval,
        &incident_interval,
        &maintenance_interval,
        &metrics_interval,
    );

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Format uptime duration as human-readable string
fn format_uptime(started_at: chrono::DateTime<Utc>) -> String {
    let duration = Utc::now() - started_at;
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

// =============================================================================
// Config Handlers
// =============================================================================

/// Handle /admin config show
async fn handle_config_show(
    ctx: &Context,
    interaction: &CommandInteraction,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), serenity::Error> {
    // Load current intervals from database
    let status = get_interval(db, PollerType::Status).await;
    let incident = get_interval(db, PollerType::Incident).await;
    let maintenance = get_interval(db, PollerType::Maintenance).await;
    let metrics = get_interval(db, PollerType::Metrics).await;

    let format_interval = |result: Result<u64, _>| match result {
        Ok(secs) => format!("{}s", secs),
        Err(_) => "Error".to_string(),
    };

    let embed = embeds::show_intervals(
        &format_interval(status),
        &format_interval(incident),
        &format_interval(maintenance),
        &format_interval(metrics),
    );

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Handle /admin config set <poller> <seconds>
async fn handle_config_set<'a>(
    ctx: &Context,
    interaction: &CommandInteraction,
    db: &sea_orm::DatabaseConnection,
    options: &[serenity::all::ResolvedOption<'a>],
) -> Result<(), serenity::Error> {
    // Parse options
    let poller_str = options.iter().find_map(|opt| {
        if opt.name == "poller"
            && let ResolvedValue::String(s) = opt.value
        {
            return Some(s);
        }
        None
    });

    let seconds = options.iter().find_map(|opt| {
        if opt.name == "seconds"
            && let ResolvedValue::Integer(i) = opt.value
        {
            return Some(i as u64);
        }
        None
    });

    let (Some(poller_str), Some(seconds)) = (poller_str, seconds) else {
        return respond_error(ctx, interaction, "Missing required options").await;
    };

    let Some(poller) = PollerType::from_str(poller_str) else {
        return respond_error(ctx, interaction, "Invalid poller type").await;
    };

    // Validate interval
    if let Err(msg) = validate_interval(seconds) {
        return respond_error(ctx, interaction, &msg).await;
    }

    // Update interval in database
    if let Err(e) = crate::collector::config::set_interval(db, poller, seconds).await {
        error!(error = %e, "Failed to update polling interval");
        return respond_error(ctx, interaction, "Failed to save configuration").await;
    }

    let embed = embeds::config_updated(poller.as_str(), seconds);

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Handle /admin config reset
async fn handle_config_reset(
    ctx: &Context,
    interaction: &CommandInteraction,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), serenity::Error> {
    // Reset all pollers to default
    for poller in PollerType::all() {
        if let Err(e) = crate::collector::config::set_interval(db, *poller, DEFAULT_INTERVAL).await
        {
            error!(error = %e, poller = ?poller, "Failed to reset polling interval");
            return respond_error(ctx, interaction, "Failed to reset configuration").await;
        }
    }

    let embed = embeds::config_reset(DEFAULT_INTERVAL);

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}
