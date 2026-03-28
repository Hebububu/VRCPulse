//! Response utilities for Discord interactions

use rust_i18n::t;
use serenity::all::{
    CommandInteraction, ComponentInteraction, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use super::embeds;

// =============================================================================
// Command Interaction Responses
// =============================================================================

/// Send a success response to a command interaction
#[allow(dead_code)]
pub async fn respond_success(
    ctx: &Context,
    interaction: &CommandInteraction,
    title: &str,
    description: &str,
) -> Result<(), serenity::Error> {
    let embed = embeds::success_embed(title, description).timestamp(Timestamp::now());

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Send an info response to a command interaction
#[allow(dead_code)]
pub async fn respond_info(
    ctx: &Context,
    interaction: &CommandInteraction,
    title: &str,
    description: &str,
) -> Result<(), serenity::Error> {
    let embed = embeds::info_embed(title, description);

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Send an error response to a command interaction (ephemeral)
pub async fn respond_error(
    ctx: &Context,
    interaction: &CommandInteraction,
    message: &str,
    locale: &str,
) -> Result<(), serenity::Error> {
    let title = t!("embeds.dashboard.error_title", locale = locale);
    let embed = embeds::error_embed(title, message);

    let response = CreateInteractionResponseMessage::new()
        .embed(embed)
        .ephemeral(true);

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

// =============================================================================
// Component Interaction Responses (Buttons)
// =============================================================================

/// Send an error response to a button interaction (updates the message)
pub async fn respond_button_error(
    ctx: &Context,
    interaction: &ComponentInteraction,
    message: &str,
    locale: &str,
) -> Result<(), serenity::Error> {
    let title = t!("embeds.dashboard.error_title", locale = locale);
    let embed = embeds::error_embed(title, message);

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
