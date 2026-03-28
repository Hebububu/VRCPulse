//! Show handler for /config command

use serenity::all::{CommandInteraction, Context};

use crate::commands::shared::{defer, edit_embed};
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

    edit_embed(ctx, interaction, embed).await
}
