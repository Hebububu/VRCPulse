//! Language handler for /config command

use rust_i18n::t;
use tracing::{error, info};

use serenity::all::{CommandInteraction, Context};

use crate::commands::shared::{defer, edit_embed, edit_error};
use crate::database;
use crate::i18n::resolve_locale_async;
use crate::repository::{GuildConfigRepository, UserConfigRepository};

use super::super::context::ConfigContext;
use super::super::embeds;

/// Handle /config language
pub async fn handle_language(
    ctx: &Context,
    interaction: &CommandInteraction,
    config_context: ConfigContext,
    language_code: Option<String>,
) -> Result<(), serenity::Error> {
    // Defer response since we do database operations
    defer(ctx, interaction).await?;

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
                return edit_error(
                    ctx,
                    interaction,
                    &t!(
                        "embeds.config.setup.error_language_not_registered_guild",
                        locale = &locale
                    ),
                    &locale,
                )
                .await;
            }

            // If no language specified, show current setting
            if !has_argument {
                let current = existing.and_then(|c| c.language);
                let embed = embeds::language_current(current.as_deref(), true, &locale);
                return edit_embed(ctx, interaction, embed).await;
            }

            // Update language - use the NEW language for the response
            let response_locale = language.as_deref().unwrap_or(&locale);
            match repo.update_language(guild_id, language.clone()).await {
                Ok(_) => {
                    info!(guild_id = %guild_id, language = ?language, "Updated guild language");
                    let embed = embeds::language_updated(language.as_deref(), response_locale);
                    edit_embed(ctx, interaction, embed).await
                }
                Err(e) => {
                    error!(error = %e, "Failed to update guild language");
                    edit_error(
                        ctx,
                        interaction,
                        &t!(
                            "embeds.config.setup.error_language_update_failed",
                            locale = &locale
                        ),
                        &locale,
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
                return edit_error(
                    ctx,
                    interaction,
                    &t!(
                        "embeds.config.setup.error_language_not_registered_user",
                        locale = &locale
                    ),
                    &locale,
                )
                .await;
            }

            // If no language specified, show current setting
            if !has_argument {
                let current = existing.and_then(|c| c.language);
                let embed = embeds::language_current(current.as_deref(), false, &locale);
                return edit_embed(ctx, interaction, embed).await;
            }

            // Update language - use the NEW language for the response
            let response_locale = language.as_deref().unwrap_or(&locale);
            match repo.update_language(user_id, language.clone()).await {
                Ok(_) => {
                    info!(user_id = %user_id, language = ?language, "Updated user language");
                    let embed = embeds::language_updated(language.as_deref(), response_locale);
                    edit_embed(ctx, interaction, embed).await
                }
                Err(e) => {
                    error!(error = %e, "Failed to update user language");
                    edit_error(
                        ctx,
                        interaction,
                        &t!(
                            "embeds.config.setup.error_language_update_failed",
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
