//! Setup handler for /config command

use rust_i18n::t;
use serenity::all::{ChannelId, CommandInteraction, Context};
use tracing::{error, info};

use crate::commands::shared::{defer, edit_error, edit_info, edit_success};
use crate::database;
use crate::i18n::resolve_locale_async;
use crate::repository::{GuildConfigRepository, UserConfigRepository};

use super::super::context::ConfigContext;
use super::super::validation::validate_channel_permissions;

/// Handle /config setup
pub async fn handle_setup(
    ctx: &Context,
    interaction: &CommandInteraction,
    config_context: ConfigContext,
    channel_id: Option<ChannelId>,
) -> Result<(), serenity::Error> {
    // Defer response since we do database operations
    defer(ctx, interaction).await?;

    let db = database::get_db(ctx).await;
    let locale = resolve_locale_async(ctx, interaction).await;

    match config_context {
        ConfigContext::Guild(guild_id) => {
            // Channel is required for guild setup
            let Some(channel_id) = channel_id else {
                return edit_error(
                    ctx,
                    interaction,
                    &t!(
                        "embeds.config.setup.error_channel_required",
                        locale = &locale
                    ),
                    &locale,
                )
                .await;
            };

            // Validate channel permissions
            if let Err(msg) = validate_channel_permissions(ctx, channel_id).await {
                return edit_error(ctx, interaction, &msg, &locale).await;
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
                    return edit_info(
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
                        return edit_error(
                            ctx,
                            interaction,
                            &t!("embeds.config.setup.error_update_failed", locale = &locale),
                            &locale,
                        )
                        .await;
                    }
                    let channel = format!("<#{}>", channel_id);
                    return edit_success(
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
                    edit_success(
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
                    edit_error(
                        ctx,
                        interaction,
                        &t!(
                            "embeds.config.setup.error_registration_failed",
                            locale = &locale
                        ),
                        &locale,
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
                return edit_info(
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
                    edit_success(
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
                    edit_error(
                        ctx,
                        interaction,
                        &t!(
                            "embeds.config.setup.error_registration_failed",
                            locale = &locale
                        ),
                        &locale,
                    )
                    .await
                }
            }
        }
    }
}
