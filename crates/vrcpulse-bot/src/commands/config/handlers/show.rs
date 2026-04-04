//! Show handler for /config command

use serenity::all::{CommandInteraction, Context};

use crate::commands::shared::{defer, edit_embed, edit_error};
use crate::database;
use crate::i18n::resolve_locale_async;
use crate::repository::{GuildConfigRepository, UserConfigRepository};

use super::super::context::ConfigContext;
use super::super::embeds;

/// Handle /config show
pub async fn handle_show(
    ctx: &Context,
    interaction: &CommandInteraction,
    config_context: ConfigContext,
) -> Result<(), serenity::Error> {
    // Defer response since we do database operations
    defer(ctx, interaction).await?;

    let db = database::get_db(ctx).await;
    let locale = resolve_locale_async(ctx, interaction).await;

    let embed = match config_context {
        ConfigContext::Guild(guild_id) => {
            let repo = GuildConfigRepository::new(db);
            match repo.get(guild_id).await {
                Ok(Some(c)) if c.enabled => embeds::show_guild_active(&c, &locale),
                Ok(Some(c)) => embeds::show_guild_disabled(&c, &locale),
                Ok(None) => embeds::show_guild_intro(&locale),
                Err(e) => {
                    tracing::error!(error = %e, "Failed to query guild config");
                    return edit_error(
                        ctx,
                        interaction,
                        &rust_i18n::t!("embeds.config.setup.error_registration_failed", locale = &locale),
                        &locale,
                    )
                    .await;
                }
            }
        }
        ConfigContext::User(user_id) => {
            let repo = UserConfigRepository::new(db);
            match repo.get(user_id).await {
                Ok(Some(c)) if c.enabled => embeds::show_user_active(&c, &locale),
                Ok(Some(c)) => embeds::show_user_disabled(&c, &locale),
                Ok(None) => embeds::show_user_intro(&locale),
                Err(e) => {
                    tracing::error!(error = %e, "Failed to query user config");
                    return edit_error(
                        ctx,
                        interaction,
                        &rust_i18n::t!("embeds.config.setup.error_registration_failed", locale = &locale),
                        &locale,
                    )
                    .await;
                }
            }
        }
    };

    edit_embed(ctx, interaction, embed).await
}
