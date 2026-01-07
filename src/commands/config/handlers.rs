//! Handler functions for /config subcommands

use rust_i18n::t;
use serenity::all::{
    ButtonStyle, ChannelId, CommandInteraction, ComponentInteraction, Context, CreateActionRow,
    CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tracing::{error, info};

use crate::commands::shared::{respond_button_error, respond_error, respond_info, respond_success};
use crate::database;
use crate::i18n::{resolve_locale_async, resolve_locale_component};
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
    let locale = resolve_locale_async(ctx, interaction).await;

    match config_context {
        ConfigContext::Guild(guild_id) => {
            // Channel is required for guild setup
            let Some(channel_id) = channel_id else {
                return respond_error(
                    ctx,
                    interaction,
                    &t!(
                        "embeds.config.setup.error_channel_required",
                        locale = &locale
                    ),
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
                    let channel = format!("<#{}>", channel_id);
                    return respond_info(
                        ctx,
                        interaction,
                        &t!(
                            "embeds.config.setup.already_registered.title",
                            locale = &locale
                        ),
                        &t!(
                            "embeds.config.setup.already_registered.description_guild",
                            locale = &locale,
                            channel = channel
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
                            &t!("embeds.config.setup.error_update_failed", locale = &locale),
                        )
                        .await;
                    }
                    let channel = format!("<#{}>", channel_id);
                    return respond_success(
                        ctx,
                        interaction,
                        &t!(
                            "embeds.config.setup.channel_updated.title",
                            locale = &locale
                        ),
                        &t!(
                            "embeds.config.setup.channel_updated.description",
                            locale = &locale,
                            channel = channel
                        ),
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
                    let channel = format!("<#{}>", channel_id);
                    respond_success(
                        ctx,
                        interaction,
                        &t!("embeds.config.setup.success.title", locale = &locale),
                        &t!(
                            "embeds.config.setup.success.description_guild",
                            locale = &locale,
                            channel = channel
                        ),
                    )
                    .await
                }
                Err(e) => {
                    error!(error = %e, "Failed to create guild config");
                    respond_error(
                        ctx,
                        interaction,
                        &t!(
                            "embeds.config.setup.error_registration_failed",
                            locale = &locale
                        ),
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
                    &t!(
                        "embeds.config.setup.already_registered.title",
                        locale = &locale
                    ),
                    &t!(
                        "embeds.config.setup.already_registered.description_user",
                        locale = &locale
                    ),
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
                        &t!("embeds.config.setup.success.title", locale = &locale),
                        &t!(
                            "embeds.config.setup.success.description_user",
                            locale = &locale
                        ),
                    )
                    .await
                }
                Err(e) => {
                    error!(error = %e, "Failed to create user config");
                    respond_error(
                        ctx,
                        interaction,
                        &t!(
                            "embeds.config.setup.error_registration_failed",
                            locale = &locale
                        ),
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
    let locale = resolve_locale_async(ctx, interaction).await;

    let embed = match config_context {
        ConfigContext::Guild(guild_id) => {
            let repo = GuildConfigRepository::new(db);
            match repo.get(guild_id).await {
                Some(c) if c.enabled => embeds::show_guild_active(&c, &locale),
                Some(c) => embeds::show_guild_disabled(&c, &locale),
                None => embeds::show_guild_intro(&locale),
            }
        }
        ConfigContext::User(user_id) => {
            let repo = UserConfigRepository::new(db);
            match repo.get(user_id).await {
                Some(c) if c.enabled => embeds::show_user_active(&c, &locale),
                Some(c) => embeds::show_user_disabled(&c, &locale),
                None => embeds::show_user_intro(&locale),
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
    let locale = resolve_locale_async(ctx, interaction).await;

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
            &t!("embeds.config.errors.not_registered", locale = &locale),
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
    let embed = embeds::unregister_confirm(&name, is_guild, &locale);

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
        .label(t!("buttons.cancel", locale = &locale))
        .style(ButtonStyle::Secondary),
        CreateButton::new(format!(
            "{}{}",
            BUTTON_CONFIRM_UNREGISTER_PREFIX, context_suffix
        ))
        .label(t!("buttons.yes_unregister", locale = &locale))
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
    let locale = resolve_locale_component(ctx, interaction).await;

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
                        &t!("embeds.config.errors.no_permission", locale = &locale),
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
                        &t!("embeds.config.errors.could_not_verify", locale = &locale),
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
                    &t!("embeds.config.errors.only_own_account", locale = &locale),
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
                &t!(
                    "embeds.config.errors.invalid_button_state",
                    locale = &locale
                ),
            )
            .await;
        }
    };

    let embed = match result {
        Ok(()) => embeds::unregister_success(&locale),
        Err(e) => {
            error!(error = %e, "Failed to disable config");
            embeds::unregister_error(&locale)
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
    let locale = resolve_locale_component(ctx, interaction).await;

    let response = CreateInteractionResponseMessage::new()
        .embed(embeds::unregister_cancelled(&locale))
        .components(vec![]);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(response),
        )
        .await
}

// =============================================================================
// Language Handler
// =============================================================================

/// Handle /config language
pub async fn handle_language(
    ctx: &Context,
    interaction: &CommandInteraction,
    config_context: ConfigContext,
    language_code: Option<String>,
) -> Result<(), serenity::Error> {
    let db = database::get_db(ctx).await;
    let locale = resolve_locale_async(ctx, interaction).await;

    // Check if user provided any argument
    let has_argument = language_code.is_some();

    // Convert "auto" to None (NULL in database means auto-detect)
    let language = language_code.and_then(|code| if code == "auto" { None } else { Some(code) });

    match config_context {
        ConfigContext::Guild(guild_id) => {
            let repo = GuildConfigRepository::new(db.clone());

            // Check if registered
            let existing = repo.get(guild_id).await;
            if existing.is_none() {
                return respond_error(
                    ctx,
                    interaction,
                    &t!(
                        "embeds.config.setup.error_language_not_registered_guild",
                        locale = &locale
                    ),
                )
                .await;
            }

            // If no language specified, show current setting
            if !has_argument {
                let current = existing.and_then(|c| c.language);
                let embed = embeds::language_current(current.as_deref(), true, &locale);
                let response = CreateInteractionResponseMessage::new().embed(embed);
                return interaction
                    .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                    .await;
            }

            // Update language - use the NEW language for the response
            let response_locale = language.as_deref().unwrap_or(&locale);
            match repo.update_language(guild_id, language.clone()).await {
                Ok(_) => {
                    info!(guild_id = %guild_id, language = ?language, "Updated guild language");
                    let embed = embeds::language_updated(language.as_deref(), response_locale);
                    let response = CreateInteractionResponseMessage::new().embed(embed);
                    interaction
                        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                        .await
                }
                Err(e) => {
                    error!(error = %e, "Failed to update guild language");
                    respond_error(
                        ctx,
                        interaction,
                        &t!(
                            "embeds.config.setup.error_language_update_failed",
                            locale = &locale
                        ),
                    )
                    .await
                }
            }
        }
        ConfigContext::User(user_id) => {
            let repo = UserConfigRepository::new(db.clone());

            // Check if registered
            let existing = repo.get(user_id).await;
            if existing.is_none() {
                return respond_error(
                    ctx,
                    interaction,
                    &t!(
                        "embeds.config.setup.error_language_not_registered_user",
                        locale = &locale
                    ),
                )
                .await;
            }

            // If no language specified, show current setting
            if !has_argument {
                let current = existing.and_then(|c| c.language);
                let embed = embeds::language_current(current.as_deref(), false, &locale);
                let response = CreateInteractionResponseMessage::new().embed(embed);
                return interaction
                    .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                    .await;
            }

            // Update language - use the NEW language for the response
            let response_locale = language.as_deref().unwrap_or(&locale);
            match repo.update_language(user_id, language.clone()).await {
                Ok(_) => {
                    info!(user_id = %user_id, language = ?language, "Updated user language");
                    let embed = embeds::language_updated(language.as_deref(), response_locale);
                    let response = CreateInteractionResponseMessage::new().embed(embed);
                    interaction
                        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                        .await
                }
                Err(e) => {
                    error!(error = %e, "Failed to update user language");
                    respond_error(
                        ctx,
                        interaction,
                        &t!(
                            "embeds.config.setup.error_language_update_failed",
                            locale = &locale
                        ),
                    )
                    .await
                }
            }
        }
    }
}
