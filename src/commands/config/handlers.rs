//! Handler functions for /config subcommands

use serenity::all::{
    ButtonStyle, ChannelId, CommandInteraction, ComponentInteraction, Context, CreateActionRow,
    CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tracing::{error, info};

use crate::commands::shared::{respond_button_error, respond_error, respond_info, respond_success};
use crate::database;
use crate::repository::{GuildConfigRepository, UserConfigRepository};

use super::context::{ConfigContext, parse_button_context};
use super::embeds;
use super::validation::{AdminCheckResult, validate_channel_permissions, validate_guild_admin};

/// Custom ID prefix for unregister confirmation button
const BUTTON_CONFIRM_UNREGISTER_PREFIX: &str = "config_unregister_confirm";
/// Custom ID prefix for unregister cancel button
const BUTTON_CANCEL_UNREGISTER_PREFIX: &str = "config_unregister_cancel";

// =============================================================================
// Setup Handler
// =============================================================================

/// Handle /config setup
pub async fn handle_setup(
    ctx: &Context,
    interaction: &CommandInteraction,
    config_context: ConfigContext,
    channel_id: Option<ChannelId>,
) -> Result<(), serenity::Error> {
    let db = database::get_db(ctx).await;

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

            let repo = GuildConfigRepository::new(db);

            // Check if already registered and enabled
            let existing = repo.get(guild_id).await;
            if let Some(ref config) = existing
                && config.enabled
            {
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
                    if let Err(e) = repo.update_channel(guild_id, channel_id).await {
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

            // Create or re-enable registration
            let result = if existing.is_some() {
                repo.reenable(guild_id, channel_id).await
            } else {
                repo.create(guild_id, channel_id).await
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
            let repo = UserConfigRepository::new(db);

            // Check if already registered
            let existing = repo.get(user_id).await;
            if let Some(ref config) = existing
                && config.enabled
            {
                return respond_info(
                    ctx,
                    interaction,
                    "Already Registered",
                    "You're already registered for DM alerts.\n\nUse `/config show` to view settings or `/config unregister` to disable.",
                )
                .await;
            }

            // Create or re-enable registration
            let result = if existing.is_some() {
                repo.reenable(user_id).await
            } else {
                repo.create(user_id).await
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

// =============================================================================
// Show Handler
// =============================================================================

/// Handle /config show
pub async fn handle_show(
    ctx: &Context,
    interaction: &CommandInteraction,
    config_context: ConfigContext,
) -> Result<(), serenity::Error> {
    let db = database::get_db(ctx).await;

    let embed = match config_context {
        ConfigContext::Guild(guild_id) => {
            let repo = GuildConfigRepository::new(db);
            match repo.get(guild_id).await {
                Some(c) if c.enabled => embeds::show_guild_active(&c),
                Some(c) => embeds::show_guild_disabled(&c),
                None => embeds::show_guild_intro(),
            }
        }
        ConfigContext::User(user_id) => {
            let repo = UserConfigRepository::new(db);
            match repo.get(user_id).await {
                Some(c) if c.enabled => embeds::show_user_active(&c),
                Some(c) => embeds::show_user_disabled(&c),
                None => embeds::show_user_intro(),
            }
        }
    };

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

// =============================================================================
// Unregister Handler
// =============================================================================

/// Handle /config unregister - show confirmation buttons
pub async fn handle_unregister(
    ctx: &Context,
    interaction: &CommandInteraction,
    config_context: ConfigContext,
) -> Result<(), serenity::Error> {
    let db = database::get_db(ctx).await;

    // Check if registered
    let is_registered = match &config_context {
        ConfigContext::Guild(guild_id) => {
            let repo = GuildConfigRepository::new(db.clone());
            repo.get(*guild_id).await.is_some_and(|c| c.enabled)
        }
        ConfigContext::User(user_id) => {
            let repo = UserConfigRepository::new(db);
            repo.get(*user_id).await.is_some_and(|c| c.enabled)
        }
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

    let is_guild = matches!(config_context, ConfigContext::Guild(_));
    let embed = embeds::unregister_confirm(&name, is_guild);

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

// =============================================================================
// Button Handlers
// =============================================================================

/// Check if button ID matches unregister confirmation
pub fn is_confirm_button(custom_id: &str) -> bool {
    custom_id.starts_with(BUTTON_CONFIRM_UNREGISTER_PREFIX)
}

/// Check if button ID matches unregister cancel
pub fn is_cancel_button(custom_id: &str) -> bool {
    custom_id.starts_with(BUTTON_CANCEL_UNREGISTER_PREFIX)
}

/// Handle unregister confirmation button
pub async fn handle_unregister_confirm(
    ctx: &Context,
    interaction: &ComponentInteraction,
) -> Result<(), serenity::Error> {
    let db = database::get_db(ctx).await;

    // Parse context from button custom_id
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
            let repo = GuildConfigRepository::new(db);
            repo.disable(guild_id).await.map(|_| ())
        }
        Some(ConfigContext::User(user_id)) => {
            let repo = UserConfigRepository::new(db);
            repo.disable(user_id).await.map(|_| ())
        }
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

    let embed = match result {
        Ok(()) => embeds::unregister_success(),
        Err(e) => {
            error!(error = %e, "Failed to disable config");
            embeds::unregister_error()
        }
    };

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

/// Handle unregister cancel button
pub async fn handle_unregister_cancel(
    ctx: &Context,
    interaction: &ComponentInteraction,
) -> Result<(), serenity::Error> {
    let response = CreateInteractionResponseMessage::new()
        .embed(embeds::unregister_cancelled())
        .components(vec![]);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(response),
        )
        .await
}
