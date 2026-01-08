//! Unregister handler for /config command

use rust_i18n::t;
use serenity::all::{
    ButtonStyle, CommandInteraction, ComponentInteraction, Context, CreateActionRow, CreateButton,
};
use tracing::error;

use crate::commands::shared::{
    defer_component_update, defer_ephemeral, edit_component_embed, edit_component_error,
    edit_embed_components, edit_error, parse_button_context,
};
use crate::database;
use crate::i18n::{resolve_locale_async, resolve_locale_component};
use crate::repository::{GuildConfigRepository, UserConfigRepository};

use super::super::context::ConfigContext;
use super::super::embeds;
use super::super::validation::{AdminCheckResult, validate_guild_admin};
use super::{unregister_cancel_button_id, unregister_confirm_button_id};

/// Handle /config unregister - show confirmation buttons
pub async fn handle_unregister(
    ctx: &Context,
    interaction: &CommandInteraction,
    config_context: ConfigContext,
) -> Result<(), serenity::Error> {
    // Defer ephemeral since we do database operations and response should be private
    defer_ephemeral(ctx, interaction).await?;

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
        return edit_error(
            ctx,
            interaction,
            &t!("embeds.config.errors.not_registered", locale = &locale),
            &locale,
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

    // Generate button IDs with context
    let (context_type, context_id) = match &config_context {
        ConfigContext::Guild(guild_id) => ("guild", guild_id.to_string()),
        ConfigContext::User(user_id) => ("user", user_id.to_string()),
    };

    let buttons = CreateActionRow::Buttons(vec![
        CreateButton::new(unregister_cancel_button_id(context_type, &context_id))
            .label(t!("buttons.cancel", locale = &locale))
            .style(ButtonStyle::Secondary),
        CreateButton::new(unregister_confirm_button_id(context_type, &context_id))
            .label(t!("buttons.yes_unregister", locale = &locale))
            .style(ButtonStyle::Danger),
    ]);

    edit_embed_components(ctx, interaction, embed, vec![buttons]).await
}

/// Handle unregister confirmation button
pub async fn handle_unregister_confirm(
    ctx: &Context,
    interaction: &ComponentInteraction,
) -> Result<(), serenity::Error> {
    // Defer first to acknowledge within 3 seconds
    defer_component_update(ctx, interaction).await?;

    let db = database::get_db(ctx).await;
    let locale = resolve_locale_component(ctx, interaction).await;

    // Parse context from button custom_id using shared utility
    let config_context = parse_button_context(&interaction.data.custom_id)
        .and_then(|(context_type, id_str)| parse_config_context(context_type, id_str));

    // SECURITY: Validate the user has permission to perform this action
    let validated_context = match config_context {
        Some(ConfigContext::Guild(guild_id)) => {
            // User must have ADMINISTRATOR permission in this guild
            match validate_guild_admin(ctx, guild_id, interaction.user.id).await {
                AdminCheckResult::IsAdmin => Some(ConfigContext::Guild(guild_id)),
                AdminCheckResult::NotAdmin => {
                    return edit_component_error(
                        ctx,
                        interaction,
                        &t!("embeds.config.errors.no_permission", locale = &locale),
                        &locale,
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
                    return edit_component_error(
                        ctx,
                        interaction,
                        &t!("embeds.config.errors.could_not_verify", locale = &locale),
                        &locale,
                    )
                    .await;
                }
            }
        }
        Some(ConfigContext::User(user_id)) => {
            // User can only unregister their own account
            if user_id != interaction.user.id {
                return edit_component_error(
                    ctx,
                    interaction,
                    &t!("embeds.config.errors.only_own_account", locale = &locale),
                    &locale,
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
            return edit_component_error(
                ctx,
                interaction,
                &t!(
                    "embeds.config.errors.invalid_button_state",
                    locale = &locale
                ),
                &locale,
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

    edit_component_embed(ctx, interaction, embed).await
}

/// Handle unregister cancel button
pub async fn handle_unregister_cancel(
    ctx: &Context,
    interaction: &ComponentInteraction,
) -> Result<(), serenity::Error> {
    // Defer first to acknowledge within 3 seconds
    defer_component_update(ctx, interaction).await?;

    let locale = resolve_locale_component(ctx, interaction).await;

    edit_component_embed(ctx, interaction, embeds::unregister_cancelled(&locale)).await
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Parse ConfigContext from context type and ID string
fn parse_config_context(context_type: &str, id_str: &str) -> Option<ConfigContext> {
    use serenity::all::{GuildId, UserId};

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
}
