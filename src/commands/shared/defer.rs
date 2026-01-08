//! Deferred response utilities for Discord interactions
//!
//! Use these when your handler needs to do slow operations (database, API calls)
//! before responding. Discord requires a response within 3 seconds, so defer first.

use rust_i18n::t;
use serenity::all::{
    CommandInteraction, ComponentInteraction, Context, CreateEmbed, CreateInteractionResponse,
    EditInteractionResponse, Timestamp,
};

use super::embeds;

// =============================================================================
// Deferred Command Interaction Responses
// =============================================================================

/// Defer a command interaction response
///
/// Call this first if your handler will take more than 3 seconds.
/// Then use `edit_*` functions to send the actual response.
pub async fn defer(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    interaction.defer(&ctx.http).await
}

/// Defer a command interaction response (ephemeral)
///
/// Same as `defer` but the response will only be visible to the user.
pub async fn defer_ephemeral(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<(), serenity::Error> {
    interaction.defer_ephemeral(&ctx.http).await
}

/// Edit a deferred response with a success embed
pub async fn edit_success(
    ctx: &Context,
    interaction: &CommandInteraction,
    title: &str,
    description: &str,
) -> Result<(), serenity::Error> {
    let embed = embeds::success_embed(title, description).timestamp(Timestamp::now());
    edit_embed(ctx, interaction, embed).await
}

/// Edit a deferred response with an info embed
pub async fn edit_info(
    ctx: &Context,
    interaction: &CommandInteraction,
    title: &str,
    description: &str,
) -> Result<(), serenity::Error> {
    let embed = embeds::info_embed(title, description);
    edit_embed(ctx, interaction, embed).await
}

/// Edit a deferred response with an error embed
pub async fn edit_error(
    ctx: &Context,
    interaction: &CommandInteraction,
    message: &str,
    locale: &str,
) -> Result<(), serenity::Error> {
    let title = t!("embeds.dashboard.error_title", locale = locale);
    let embed = embeds::error_embed(title, message);
    edit_embed(ctx, interaction, embed).await
}

/// Edit a deferred response with a custom embed
pub async fn edit_embed(
    ctx: &Context,
    interaction: &CommandInteraction,
    embed: CreateEmbed,
) -> Result<(), serenity::Error> {
    let response = EditInteractionResponse::new().embed(embed);
    interaction.edit_response(&ctx.http, response).await?;
    Ok(())
}

/// Edit a deferred response with a custom embed and components (buttons, etc.)
pub async fn edit_embed_components(
    ctx: &Context,
    interaction: &CommandInteraction,
    embed: CreateEmbed,
    components: Vec<serenity::all::CreateActionRow>,
) -> Result<(), serenity::Error> {
    let response = EditInteractionResponse::new()
        .embed(embed)
        .components(components);
    interaction.edit_response(&ctx.http, response).await?;
    Ok(())
}

// =============================================================================
// Deferred Component Interaction Responses (Buttons, Select Menus)
// =============================================================================

/// Defer a component interaction that will update the original message
///
/// Call this first if your button/select handler will take more than 3 seconds.
/// Then use `edit_component_*` functions to update the message.
/// Uses `Acknowledge` which doesn't show a loading state to the user.
pub async fn defer_component_update(
    ctx: &Context,
    interaction: &ComponentInteraction,
) -> Result<(), serenity::Error> {
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
        .await
}

/// Edit a deferred component response with a custom embed (removes components)
pub async fn edit_component_embed(
    ctx: &Context,
    interaction: &ComponentInteraction,
    embed: CreateEmbed,
) -> Result<(), serenity::Error> {
    let response = EditInteractionResponse::new()
        .embed(embed)
        .components(vec![]);
    interaction.edit_response(&ctx.http, response).await?;
    Ok(())
}

/// Edit a deferred component response with an error embed (removes components)
pub async fn edit_component_error(
    ctx: &Context,
    interaction: &ComponentInteraction,
    message: &str,
    locale: &str,
) -> Result<(), serenity::Error> {
    let title = t!("embeds.dashboard.error_title", locale = locale);
    let embed = embeds::error_embed(title, message);
    edit_component_embed(ctx, interaction, embed).await
}
