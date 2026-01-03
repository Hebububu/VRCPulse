use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, Context, CreateCommand,
    CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
    CreateInteractionResponseMessage, Permissions, ResolvedValue, Timestamp,
};
use tracing::error;

use crate::collector::config::{
    get_interval, validate_interval, PollerType, DEFAULT_INTERVAL,
};
use crate::state::{AppState, AppStateKey};

/// /admin config command definition
pub fn register() -> CreateCommand {
    CreateCommand::new("admin")
        .description("Admin commands")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommandGroup,
                "config",
                "Manage bot configuration",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "show",
                    "Display current polling interval settings",
                ),
            )
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
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "reset",
                    "Reset all polling intervals to default (60s)",
                ),
            ),
        )
}

/// /admin config command handler
pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    // Get AppState from TypeMap
    let data = ctx.data.read().await;
    let state = data
        .get::<AppStateKey>()
        .expect("AppState not found in TypeMap");
    let state = state.read().await;

    // Parse subcommand group and subcommand
    let options = &interaction.data.options();

    // Find the "config" subcommand group
    let config_group = options.iter().find(|opt| opt.name == "config");

    let Some(config_group) = config_group else {
        return respond_error(ctx, interaction, "Unknown subcommand").await;
    };

    let ResolvedValue::SubCommandGroup(subcommands) = &config_group.value else {
        return respond_error(ctx, interaction, "Invalid command structure").await;
    };

    let Some(subcommand) = subcommands.first() else {
        return respond_error(ctx, interaction, "Missing subcommand").await;
    };

    match subcommand.name {
        "show" => handle_show(ctx, interaction, &state).await,
        "set" => {
            let ResolvedValue::SubCommand(options) = &subcommand.value else {
                return respond_error(ctx, interaction, "Invalid command structure").await;
            };
            handle_set(ctx, interaction, &state, options).await
        }
        "reset" => handle_reset(ctx, interaction, &state).await,
        _ => respond_error(ctx, interaction, "Unknown subcommand").await,
    }
}

/// Handle /admin config show
async fn handle_show(
    ctx: &Context,
    interaction: &CommandInteraction,
    state: &AppState,
) -> Result<(), serenity::Error> {
    let db = state.database.as_ref();

    // Load current intervals from database
    let status = get_interval(db, PollerType::Status).await;
    let incident = get_interval(db, PollerType::Incident).await;
    let maintenance = get_interval(db, PollerType::Maintenance).await;
    let metrics = get_interval(db, PollerType::Metrics).await;

    let format_interval = |result: Result<u64, _>| match result {
        Ok(secs) => format!("{}s", secs),
        Err(_) => "Error".to_string(),
    };

    let embed = CreateEmbed::default()
        .title("Polling Intervals")
        .color(Colour::new(0x00b0f4))
        .field("Status", format_interval(status), true)
        .field("Incident", format_interval(incident), true)
        .field("Maintenance", format_interval(maintenance), true)
        .field("Metrics", format_interval(metrics), true)
        .footer(CreateEmbedFooter::new(
            "Use /admin config set to change",
        ));

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Handle /admin config set <poller> <seconds>
async fn handle_set<'a>(
    ctx: &Context,
    interaction: &CommandInteraction,
    state: &AppState,
    options: &[serenity::all::ResolvedOption<'a>],
) -> Result<(), serenity::Error> {
    // Parse options
    let poller_str = options.iter().find_map(|opt| {
        if opt.name == "poller" {
            if let ResolvedValue::String(s) = opt.value {
                return Some(s);
            }
        }
        None
    });

    let seconds = options.iter().find_map(|opt| {
        if opt.name == "seconds" {
            if let ResolvedValue::Integer(i) = opt.value {
                return Some(i as u64);
            }
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

    // Update interval
    let db = state.database.as_ref();
    if let Err(e) = state.collector_config.update(db, poller, seconds).await {
        error!(error = %e, "Failed to update polling interval");
        return respond_error(ctx, interaction, "Failed to save configuration").await;
    }

    let embed = CreateEmbed::default()
        .title("Configuration Updated")
        .description("Polling interval has been changed.")
        .color(Colour::new(0x57f287))
        .field("Poller", poller.as_str(), true)
        .field("New Interval", format!("{}s", seconds), true)
        .timestamp(Timestamp::now());

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Handle /admin config reset
async fn handle_reset(
    ctx: &Context,
    interaction: &CommandInteraction,
    state: &AppState,
) -> Result<(), serenity::Error> {
    let db = state.database.as_ref();

    if let Err(e) = state.collector_config.reset_all(db).await {
        error!(error = %e, "Failed to reset polling intervals");
        return respond_error(ctx, interaction, "Failed to reset configuration").await;
    }

    let default_str = format!("{}s", DEFAULT_INTERVAL);

    let embed = CreateEmbed::default()
        .title("Configuration Reset")
        .description("All polling intervals have been reset to default values.")
        .color(Colour::new(0x57f287))
        .field("Status", &default_str, true)
        .field("Incident", &default_str, true)
        .field("Maintenance", &default_str, true)
        .field("Metrics", &default_str, true)
        .timestamp(Timestamp::now());

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Send an error response
async fn respond_error(
    ctx: &Context,
    interaction: &CommandInteraction,
    message: &str,
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title("Error")
        .description(message)
        .color(Colour::new(0xed4245));

    let response = CreateInteractionResponseMessage::new()
        .embed(embed)
        .ephemeral(true);

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}
