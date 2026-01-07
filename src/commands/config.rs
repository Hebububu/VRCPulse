//! /config command - Guild and user registration for VRCPulse alerts

use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serenity::all::{
    ButtonStyle, ChannelId, ChannelType, Colour, CommandInteraction, CommandOptionType, Context,
    CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, GuildChannel,
    GuildId, Permissions, ResolvedValue, Timestamp, UserId,
};
use tracing::{error, info};

use crate::entity::{guild_configs, user_configs};
use crate::state::AppStateKey;

// =============================================================================
// Constants
// =============================================================================

/// Custom ID prefix for unregister confirmation button
const BUTTON_CONFIRM_UNREGISTER_PREFIX: &str = "config_unregister_confirm";
/// Custom ID prefix for unregister cancel button
const BUTTON_CANCEL_UNREGISTER_PREFIX: &str = "config_unregister_cancel";

/// Brand color for embeds
const COLOR_BRAND: u32 = 0x00b0f4;
/// Success color for embeds
const COLOR_SUCCESS: u32 = 0x57f287;
/// Error color for embeds
const COLOR_ERROR: u32 = 0xed4245;
/// Warning color for embeds
const COLOR_WARNING: u32 = 0xfee75c;

// =============================================================================
// Command Registration
// =============================================================================

/// /config command definition
pub fn register() -> CreateCommand {
    CreateCommand::new("config")
        .description("Configure VRCPulse alerts for this server or your account")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "setup",
                "Register for VRCPulse alerts",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "Channel to receive alerts (required for servers)",
                )
                .channel_types(vec![ChannelType::Text, ChannelType::News])
                .required(false),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "show",
            "View current configuration",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "unregister",
            "Disable VRCPulse alerts",
        ))
}

/// /config command handler
pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    let options = &interaction.data.options();

    let Some(subcommand) = options.first() else {
        return respond_error(ctx, interaction, "Missing subcommand").await;
    };

    // Determine context: guild or user install
    let config_context = determine_context(interaction);

    match subcommand.name {
        "setup" => {
            let channel_id = if let ResolvedValue::SubCommand(opts) = &subcommand.value {
                opts.iter().find_map(|opt| {
                    if opt.name == "channel" {
                        if let ResolvedValue::Channel(ch) = opt.value {
                            return Some(ch.id);
                        }
                    }
                    None
                })
            } else {
                None
            };
            handle_setup(ctx, interaction, config_context, channel_id).await
        }
        "show" => handle_show(ctx, interaction, config_context).await,
        "unregister" => handle_unregister(ctx, interaction, config_context).await,
        _ => respond_error(ctx, interaction, "Unknown subcommand").await,
    }
}

/// Handle button interactions for unregister confirmation
pub async fn handle_button(
    ctx: &Context,
    interaction: &serenity::all::ComponentInteraction,
) -> Result<(), serenity::Error> {
    let custom_id = &interaction.data.custom_id;

    if custom_id.starts_with(BUTTON_CONFIRM_UNREGISTER_PREFIX) {
        handle_unregister_confirm(ctx, interaction).await
    } else if custom_id.starts_with(BUTTON_CANCEL_UNREGISTER_PREFIX) {
        handle_unregister_cancel(ctx, interaction).await
    } else {
        Ok(())
    }
}

// =============================================================================
// Context Detection
// =============================================================================

/// Configuration context (guild or user)
#[derive(Debug, Clone)]
pub enum ConfigContext {
    Guild(GuildId),
    User(UserId),
}

/// Determine if this is a guild or user install context
fn determine_context(interaction: &CommandInteraction) -> ConfigContext {
    // If guild_id is present, it's a guild context
    if let Some(guild_id) = interaction.guild_id {
        ConfigContext::Guild(guild_id)
    } else {
        // User install (DM context)
        ConfigContext::User(interaction.user.id)
    }
}

/// Parse context from button custom_id
/// Format: "prefix:guild:123456" or "prefix:user:123456"
fn parse_button_context(custom_id: &str) -> Option<ConfigContext> {
    let parts: Vec<&str> = custom_id.split(':').collect();
    if parts.len() >= 3 {
        let context_type = parts[parts.len() - 2];
        let id_str = parts[parts.len() - 1];

        match context_type {
            "guild" => id_str
                .parse::<u64>()
                .ok()
                .map(|id| ConfigContext::Guild(GuildId::new(id))),
            "user" => id_str
                .parse::<u64>()
                .ok()
                .map(|id| ConfigContext::User(UserId::new(id))),
            _ => None,
        }
    } else {
        None
    }
}

// =============================================================================
// Handlers
// =============================================================================

/// Handle /config setup
async fn handle_setup(
    ctx: &Context,
    interaction: &CommandInteraction,
    config_context: ConfigContext,
    channel_id: Option<ChannelId>,
) -> Result<(), serenity::Error> {
    let data = ctx.data.read().await;
    let state = data
        .get::<AppStateKey>()
        .expect("AppState not found in TypeMap");
    let state = state.read().await;
    let db = state.database.as_ref();

    match config_context {
        ConfigContext::Guild(guild_id) => {
            // Channel is required for guild setup
            let Some(channel_id) = channel_id else {
                return respond_error(
                    ctx,
                    interaction,
                    "Please specify a channel for alerts.\nUsage: `/config setup #channel`",
                )
                .await;
            };

            // Validate channel permissions
            if let Err(msg) = validate_channel_permissions(ctx, channel_id).await {
                return respond_error(ctx, interaction, &msg).await;
            }

            // Check if already registered and enabled
            let existing = get_guild_config(db, guild_id).await;
            if let Some(ref config) = existing {
                if config.enabled {
                    // Already registered - update channel if different
                    if config.channel_id.as_ref() == Some(&channel_id.to_string()) {
                        return respond_info(
                            ctx,
                            interaction,
                            "Already Registered",
                            &format!(
                                "This server is already registered with <#{}>.\n\nUse `/config show` to view settings or `/config unregister` to disable.",
                                channel_id
                            ),
                        )
                        .await;
                    } else {
                        // Update channel
                        if let Err(e) = update_guild_channel(db, guild_id, channel_id).await {
                            error!(error = %e, "Failed to update guild channel");
                            return respond_error(
                                ctx,
                                interaction,
                                "Failed to update configuration. Please try again.",
                            )
                            .await;
                        }
                        return respond_success(
                            ctx,
                            interaction,
                            "Channel Updated",
                            &format!("Alert channel has been changed to <#{}>.", channel_id),
                        )
                        .await;
                    }
                }
            }

            // Create or re-enable registration
            let result = if existing.is_some() {
                reenable_guild_config(db, guild_id, channel_id).await
            } else {
                create_guild_config(db, guild_id, channel_id).await
            };

            match result {
                Ok(_) => {
                    info!(guild_id = %guild_id, channel_id = %channel_id, "Guild registered for alerts");
                    respond_success(
                        ctx,
                        interaction,
                        "Registration Complete!",
                        &format!(
                            "VRCPulse alerts will be sent to <#{}>.\n\n**Commands**\n- `/config show` - View settings\n- `/config unregister` - Disable alerts\n- `/status` - Check VRChat status",
                            channel_id
                        ),
                    )
                    .await
                }
                Err(e) => {
                    error!(error = %e, "Failed to create guild config");
                    respond_error(
                        ctx,
                        interaction,
                        "Failed to complete registration. Please try again.",
                    )
                    .await
                }
            }
        }
        ConfigContext::User(user_id) => {
            // Check if already registered
            let existing = get_user_config(db, user_id).await;
            if let Some(ref config) = existing {
                if config.enabled {
                    return respond_info(
                        ctx,
                        interaction,
                        "Already Registered",
                        "You're already registered for DM alerts.\n\nUse `/config show` to view settings or `/config unregister` to disable.",
                    )
                    .await;
                }
            }

            // Create or re-enable registration
            let result = if existing.is_some() {
                reenable_user_config(db, user_id).await
            } else {
                create_user_config(db, user_id).await
            };

            match result {
                Ok(_) => {
                    info!(user_id = %user_id, "User registered for DM alerts");
                    respond_success(
                        ctx,
                        interaction,
                        "Registration Complete!",
                        "VRCPulse alerts will be sent to your DMs.\n\n**Commands**\n- `/config show` - View settings\n- `/config unregister` - Disable alerts\n- `/status` - Check VRChat status",
                    )
                    .await
                }
                Err(e) => {
                    error!(error = %e, "Failed to create user config");
                    respond_error(
                        ctx,
                        interaction,
                        "Failed to complete registration. Please try again.",
                    )
                    .await
                }
            }
        }
    }
}

/// Handle /config show
async fn handle_show(
    ctx: &Context,
    interaction: &CommandInteraction,
    config_context: ConfigContext,
) -> Result<(), serenity::Error> {
    let data = ctx.data.read().await;
    let state = data
        .get::<AppStateKey>()
        .expect("AppState not found in TypeMap");
    let state = state.read().await;
    let db = state.database.as_ref();

    match config_context {
        ConfigContext::Guild(guild_id) => {
            let config = get_guild_config(db, guild_id).await;

            match config {
                Some(c) if c.enabled => {
                    // Case C: Currently registered
                    let channel_display = c
                        .channel_id
                        .as_ref()
                        .map(|id| format!("<#{}>", id))
                        .unwrap_or_else(|| "Not set".to_string());

                    let embed = CreateEmbed::default()
                        .title("VRCPulse Configuration")
                        .color(Colour::new(COLOR_BRAND))
                        .field("Status", "Active", true)
                        .field("Channel", channel_display, true)
                        .field(
                            "Registered",
                            format!("<t:{}:R>", c.created_at.timestamp()),
                            true,
                        )
                        .footer(CreateEmbedFooter::new(
                            "Use /config unregister to disable alerts",
                        ));

                    let response = CreateInteractionResponseMessage::new().embed(embed);
                    interaction
                        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                        .await
                }
                Some(c) => {
                    // Case B: Previously registered (disabled)
                    let channel_display = c
                        .channel_id
                        .as_ref()
                        .map(|id| format!("<#{}>", id))
                        .unwrap_or_else(|| "Not set".to_string());

                    let embed = CreateEmbed::default()
                        .title("VRCPulse - Unregistered")
                        .description(format!(
                            "This server was unregistered <t:{}:R>.\nRun `/config setup #channel` to re-enable alerts.",
                            c.updated_at.timestamp()
                        ))
                        .color(Colour::new(COLOR_WARNING))
                        .field("Previous Channel", channel_display, true)
                        .field(
                            "Originally Registered",
                            format!("<t:{}:R>", c.created_at.timestamp()),
                            true,
                        );

                    let response = CreateInteractionResponseMessage::new().embed(embed);
                    interaction
                        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                        .await
                }
                None => {
                    // Case A: Never registered - show intro
                    let embed = CreateEmbed::default()
                        .title("Welcome to VRCPulse!")
                        .description(
                            "VRCPulse monitors VRChat server status and alerts you when issues occur."
                        )
                        .color(Colour::new(COLOR_BRAND))
                        .field(
                            "Getting Started",
                            "1. Run `/config setup #channel` to register this server\n2. Check current VRChat status with `/status`",
                            false,
                        )
                        .field(
                            "Commands",
                            "- `/config setup <channel>` - Register and set alert channel\n- `/config show` - View current settings\n- `/config unregister` - Disable alerts",
                            false,
                        )
                        .footer(CreateEmbedFooter::new(
                            "This server isn't registered yet. Run /config setup #channel to get started!",
                        ));

                    let response = CreateInteractionResponseMessage::new().embed(embed);
                    interaction
                        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                        .await
                }
            }
        }
        ConfigContext::User(user_id) => {
            let config = get_user_config(db, user_id).await;

            match config {
                Some(c) if c.enabled => {
                    // Currently registered
                    let embed = CreateEmbed::default()
                        .title("VRCPulse Configuration")
                        .color(Colour::new(COLOR_BRAND))
                        .field("Status", "Active", true)
                        .field("Delivery", "Direct Messages", true)
                        .field(
                            "Registered",
                            format!("<t:{}:R>", c.created_at.timestamp()),
                            true,
                        )
                        .footer(CreateEmbedFooter::new(
                            "Use /config unregister to disable alerts",
                        ));

                    let response = CreateInteractionResponseMessage::new().embed(embed);
                    interaction
                        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                        .await
                }
                Some(c) => {
                    // Previously registered (disabled)
                    let embed = CreateEmbed::default()
                        .title("VRCPulse - Unregistered")
                        .description(format!(
                            "You unregistered <t:{}:R>.\nRun `/config setup` to re-enable DM alerts.",
                            c.updated_at.timestamp()
                        ))
                        .color(Colour::new(COLOR_WARNING))
                        .field(
                            "Originally Registered",
                            format!("<t:{}:R>", c.created_at.timestamp()),
                            true,
                        );

                    let response = CreateInteractionResponseMessage::new().embed(embed);
                    interaction
                        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                        .await
                }
                None => {
                    // Never registered - show intro
                    let embed = CreateEmbed::default()
                        .title("Welcome to VRCPulse!")
                        .description(
                            "VRCPulse monitors VRChat server status and alerts you when issues occur."
                        )
                        .color(Colour::new(COLOR_BRAND))
                        .field(
                            "Getting Started",
                            "1. Run `/config setup` to register for DM alerts\n2. Check current VRChat status with `/status`",
                            false,
                        )
                        .field(
                            "Commands",
                            "- `/config setup` - Register for DM alerts\n- `/config show` - View current settings\n- `/config unregister` - Disable alerts",
                            false,
                        )
                        .footer(CreateEmbedFooter::new(
                            "You aren't registered yet. Run /config setup to get started!",
                        ));

                    let response = CreateInteractionResponseMessage::new().embed(embed);
                    interaction
                        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                        .await
                }
            }
        }
    }
}

/// Handle /config unregister - show confirmation buttons
async fn handle_unregister(
    ctx: &Context,
    interaction: &CommandInteraction,
    config_context: ConfigContext,
) -> Result<(), serenity::Error> {
    let data = ctx.data.read().await;
    let state = data
        .get::<AppStateKey>()
        .expect("AppState not found in TypeMap");
    let state = state.read().await;
    let db = state.database.as_ref();

    // Check if registered
    let is_registered = match &config_context {
        ConfigContext::Guild(guild_id) => get_guild_config(db, *guild_id)
            .await
            .is_some_and(|c| c.enabled),
        ConfigContext::User(user_id) => get_user_config(db, *user_id)
            .await
            .is_some_and(|c| c.enabled),
    };

    if !is_registered {
        return respond_error(
            ctx,
            interaction,
            "This server/account isn't registered. Use `/config setup` to register first.",
        )
        .await;
    }

    // Get name for confirmation message
    let name = match &config_context {
        ConfigContext::Guild(guild_id) => interaction
            .guild_id
            .and_then(|_| ctx.cache.guild(*guild_id).map(|g| g.name.clone()))
            .unwrap_or_else(|| "this server".to_string()),
        ConfigContext::User(_) => interaction.user.name.clone(),
    };

    let description = match config_context {
        ConfigContext::Guild(_) => format!(
            "Are you sure you want to unregister **{}**?\n\nThis will stop all VRCPulse alerts for this server.",
            name
        ),
        ConfigContext::User(_) => {
            "Are you sure you want to unregister?\n\nThis will stop all VRCPulse DM alerts."
                .to_string()
        }
    };

    let embed = CreateEmbed::default()
        .title("Confirm Unregister")
        .description(description)
        .color(Colour::new(COLOR_WARNING))
        .footer(CreateEmbedFooter::new(
            "This confirmation expires in 15 minutes",
        ));

    // Encode context in button custom_id to preserve it across interaction
    let context_suffix = match &config_context {
        ConfigContext::Guild(guild_id) => format!(":guild:{}", guild_id),
        ConfigContext::User(user_id) => format!(":user:{}", user_id),
    };

    let buttons = CreateActionRow::Buttons(vec![
        CreateButton::new(format!(
            "{}{}",
            BUTTON_CANCEL_UNREGISTER_PREFIX, context_suffix
        ))
        .label("Cancel")
        .style(ButtonStyle::Secondary),
        CreateButton::new(format!(
            "{}{}",
            BUTTON_CONFIRM_UNREGISTER_PREFIX, context_suffix
        ))
        .label("Yes, Unregister")
        .style(ButtonStyle::Danger),
    ]);

    let response = CreateInteractionResponseMessage::new()
        .embed(embed)
        .components(vec![buttons])
        .ephemeral(true);

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Handle unregister confirmation button
async fn handle_unregister_confirm(
    ctx: &Context,
    interaction: &serenity::all::ComponentInteraction,
) -> Result<(), serenity::Error> {
    let data = ctx.data.read().await;
    let state = data
        .get::<AppStateKey>()
        .expect("AppState not found in TypeMap");
    let state = state.read().await;
    let db = state.database.as_ref();

    // Parse context from button custom_id (format: "config_unregister_confirm:guild:123" or ":user:456")
    let config_context = parse_button_context(&interaction.data.custom_id);

    // SECURITY: Validate the user has permission to perform this action
    let validated_context = match config_context {
        Some(ConfigContext::Guild(guild_id)) => {
            // User must have ADMINISTRATOR permission in this guild
            match validate_guild_admin(ctx, guild_id, interaction.user.id).await {
                AdminCheckResult::IsAdmin => Some(ConfigContext::Guild(guild_id)),
                AdminCheckResult::NotAdmin => {
                    return respond_button_error(
                        ctx,
                        interaction,
                        "You don't have permission to unregister this server.",
                    )
                    .await;
                }
                AdminCheckResult::CouldNotVerify(reason) => {
                    error!(
                        guild_id = %guild_id,
                        user_id = %interaction.user.id,
                        reason = %reason,
                        "Could not verify admin permissions"
                    );
                    return respond_button_error(
                        ctx,
                        interaction,
                        "Could not verify your permissions. Please try again.",
                    )
                    .await;
                }
            }
        }
        Some(ConfigContext::User(user_id)) => {
            // User can only unregister their own account
            if user_id != interaction.user.id {
                return respond_button_error(
                    ctx,
                    interaction,
                    "You can only unregister your own account.",
                )
                .await;
            }
            Some(ConfigContext::User(user_id))
        }
        None => None,
    };

    let result: Result<(), sea_orm::DbErr> = match validated_context {
        Some(ConfigContext::Guild(guild_id)) => {
            disable_guild_config(db, guild_id).await.map(|_| ())
        }
        Some(ConfigContext::User(user_id)) => disable_user_config(db, user_id).await.map(|_| ()),
        None => {
            // Context parsing failed - don't fall back to insecure behavior
            error!(
                "Failed to parse button context: {}",
                interaction.data.custom_id
            );
            return respond_button_error(
                ctx,
                interaction,
                "Invalid button state. Please run `/config unregister` again.",
            )
            .await;
        }
    };

    match result {
        Ok(()) => {
            let embed = CreateEmbed::default()
                .title("Unregistered")
                .description(
                    "VRCPulse alerts have been disabled.\n\nYou can re-register anytime with `/config setup`.",
                )
                .color(Colour::new(COLOR_SUCCESS));

            let response = CreateInteractionResponseMessage::new()
                .embed(embed)
                .components(vec![]);

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(response),
                )
                .await
        }
        Err(e) => {
            error!(error = %e, "Failed to disable config");

            let embed = CreateEmbed::default()
                .title("Error")
                .description("Failed to unregister. Please try again.")
                .color(Colour::new(COLOR_ERROR));

            let response = CreateInteractionResponseMessage::new()
                .embed(embed)
                .components(vec![]);

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(response),
                )
                .await
        }
    }
}

/// Handle unregister cancel button
async fn handle_unregister_cancel(
    ctx: &Context,
    interaction: &serenity::all::ComponentInteraction,
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title("Cancelled")
        .description("Unregister cancelled. Your configuration remains active.")
        .color(Colour::new(COLOR_BRAND));

    let response = CreateInteractionResponseMessage::new()
        .embed(embed)
        .components(vec![]);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(response),
        )
        .await
}

// =============================================================================
// Database Operations
// =============================================================================

/// Get guild config by ID
async fn get_guild_config(
    db: &DatabaseConnection,
    guild_id: GuildId,
) -> Option<guild_configs::Model> {
    guild_configs::Entity::find_by_id(guild_id.to_string())
        .one(db)
        .await
        .ok()
        .flatten()
}

/// Get user config by ID
async fn get_user_config(db: &DatabaseConnection, user_id: UserId) -> Option<user_configs::Model> {
    user_configs::Entity::find_by_id(user_id.to_string())
        .one(db)
        .await
        .ok()
        .flatten()
}

/// Create new guild config
async fn create_guild_config(
    db: &DatabaseConnection,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<guild_configs::Model, sea_orm::DbErr> {
    let now = Utc::now();
    let model = guild_configs::ActiveModel {
        guild_id: Set(guild_id.to_string()),
        channel_id: Set(Some(channel_id.to_string())),
        enabled: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    model.insert(db).await
}

/// Create new user config
async fn create_user_config(
    db: &DatabaseConnection,
    user_id: UserId,
) -> Result<user_configs::Model, sea_orm::DbErr> {
    let now = Utc::now();
    let model = user_configs::ActiveModel {
        user_id: Set(user_id.to_string()),
        enabled: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    model.insert(db).await
}

/// Re-enable existing guild config
async fn reenable_guild_config(
    db: &DatabaseConnection,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<guild_configs::Model, sea_orm::DbErr> {
    let now = Utc::now();
    let model = guild_configs::ActiveModel {
        guild_id: Set(guild_id.to_string()),
        channel_id: Set(Some(channel_id.to_string())),
        enabled: Set(true),
        updated_at: Set(now),
        ..Default::default()
    };
    model.update(db).await
}

/// Re-enable existing user config
async fn reenable_user_config(
    db: &DatabaseConnection,
    user_id: UserId,
) -> Result<user_configs::Model, sea_orm::DbErr> {
    let now = Utc::now();
    let model = user_configs::ActiveModel {
        user_id: Set(user_id.to_string()),
        enabled: Set(true),
        updated_at: Set(now),
        ..Default::default()
    };
    model.update(db).await
}

/// Update guild channel
async fn update_guild_channel(
    db: &DatabaseConnection,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<guild_configs::Model, sea_orm::DbErr> {
    let now = Utc::now();
    let model = guild_configs::ActiveModel {
        guild_id: Set(guild_id.to_string()),
        channel_id: Set(Some(channel_id.to_string())),
        updated_at: Set(now),
        ..Default::default()
    };
    model.update(db).await
}

/// Disable guild config (soft delete)
async fn disable_guild_config(
    db: &DatabaseConnection,
    guild_id: GuildId,
) -> Result<guild_configs::Model, sea_orm::DbErr> {
    let now = Utc::now();
    let model = guild_configs::ActiveModel {
        guild_id: Set(guild_id.to_string()),
        enabled: Set(false),
        updated_at: Set(now),
        ..Default::default()
    };
    model.update(db).await
}

/// Disable user config (soft delete)
async fn disable_user_config(
    db: &DatabaseConnection,
    user_id: UserId,
) -> Result<user_configs::Model, sea_orm::DbErr> {
    let now = Utc::now();
    let model = user_configs::ActiveModel {
        user_id: Set(user_id.to_string()),
        enabled: Set(false),
        updated_at: Set(now),
        ..Default::default()
    };
    model.update(db).await
}

// =============================================================================
// Channel Validation
// =============================================================================

/// Validate bot has required permissions in the target channel
async fn validate_channel_permissions(ctx: &Context, channel_id: ChannelId) -> Result<(), String> {
    // Get channel
    let channel = channel_id
        .to_channel(&ctx.http)
        .await
        .map_err(|_| "Could not access that channel. Please check it exists and I can see it.")?;

    let guild_channel = channel
        .guild()
        .ok_or("That doesn't appear to be a server channel.")?;

    // Get bot's permissions in the channel
    let bot_id = ctx.cache.current_user().id;
    let permissions = get_channel_permissions(ctx, &guild_channel, bot_id).await?;

    // Check required permissions
    if !permissions.send_messages() {
        return Err(
            "I don't have permission to send messages in that channel. Please give me the **Send Messages** permission."
                .to_string(),
        );
    }

    if !permissions.embed_links() {
        return Err(
            "I don't have permission to send embeds in that channel. Please give me the **Embed Links** permission."
                .to_string(),
        );
    }

    Ok(())
}

/// Get bot's permissions in a channel
async fn get_channel_permissions(
    ctx: &Context,
    channel: &GuildChannel,
    user_id: UserId,
) -> Result<Permissions, String> {
    let guild_id = channel.guild_id;

    // Try to get from cache first
    if let Some(guild) = ctx.cache.guild(guild_id) {
        if let Some(member) = guild.members.get(&user_id) {
            return Ok(guild.user_permissions_in(channel, member));
        }
    }

    // Fallback: fetch member
    let member = guild_id
        .member(&ctx.http, user_id)
        .await
        .map_err(|_| "Could not verify my permissions in that channel.")?;

    let guild = ctx
        .cache
        .guild(guild_id)
        .ok_or("Could not access guild information.")?;

    Ok(guild.user_permissions_in(channel, &member))
}

// =============================================================================
// Security Validation
// =============================================================================

/// Result of admin permission check
enum AdminCheckResult {
    /// User is an administrator
    IsAdmin,
    /// User is not an administrator
    NotAdmin,
    /// Could not verify permissions (API error, cache miss, etc.)
    CouldNotVerify(String),
}

/// Validate that a user has ADMINISTRATOR permission in a guild
async fn validate_guild_admin(
    ctx: &Context,
    guild_id: GuildId,
    user_id: UserId,
) -> AdminCheckResult {
    // Try cache first
    if let Some(guild) = ctx.cache.guild(guild_id) {
        if let Some(member) = guild.members.get(&user_id) {
            let perms = guild.member_permissions(member);
            return if perms.administrator() {
                AdminCheckResult::IsAdmin
            } else {
                AdminCheckResult::NotAdmin
            };
        }
    }

    // Fallback: fetch member and check permissions
    match guild_id.member(&ctx.http, user_id).await {
        Ok(member) => {
            if let Some(guild) = ctx.cache.guild(guild_id) {
                let perms = guild.member_permissions(&member);
                return if perms.administrator() {
                    AdminCheckResult::IsAdmin
                } else {
                    AdminCheckResult::NotAdmin
                };
            }
            AdminCheckResult::CouldNotVerify("Guild not in cache after member fetch".to_string())
        }
        Err(e) => {
            error!(guild_id = %guild_id, user_id = %user_id, error = %e, "Failed to fetch member for admin check");
            AdminCheckResult::CouldNotVerify(format!("API error: {}", e))
        }
    }
}

// =============================================================================
// Response Helpers
// =============================================================================

/// Send error response for button interactions
async fn respond_button_error(
    ctx: &Context,
    interaction: &serenity::all::ComponentInteraction,
    message: &str,
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title("Error")
        .description(message)
        .color(Colour::new(COLOR_ERROR));

    let response = CreateInteractionResponseMessage::new()
        .embed(embed)
        .components(vec![]);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(response),
        )
        .await
}

/// Send success response
async fn respond_success(
    ctx: &Context,
    interaction: &CommandInteraction,
    title: &str,
    description: &str,
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::new(COLOR_SUCCESS))
        .timestamp(Timestamp::now());

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Send info response
async fn respond_info(
    ctx: &Context,
    interaction: &CommandInteraction,
    title: &str,
    description: &str,
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::new(COLOR_BRAND));

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Send error response
async fn respond_error(
    ctx: &Context,
    interaction: &CommandInteraction,
    message: &str,
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title("Error")
        .description(message)
        .color(Colour::new(COLOR_ERROR));

    let response = CreateInteractionResponseMessage::new()
        .embed(embed)
        .ephemeral(true);

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}
