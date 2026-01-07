//! Shared utilities for Discord command responses

use serenity::all::{
    Colour, CommandInteraction, ComponentInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp,
};

// =============================================================================
// Colors
// =============================================================================

/// Standard colors for embed responses
pub mod colors {
    /// Brand color (blue)
    pub const BRAND: u32 = 0x00b0f4;
    /// Success color (green)
    pub const SUCCESS: u32 = 0x57f287;
    /// Error color (red)
    pub const ERROR: u32 = 0xed4245;
    /// Warning color (yellow)
    pub const WARNING: u32 = 0xfee75c;
}

// =============================================================================
// Command Interaction Responses
// =============================================================================

/// Send a success response to a command interaction
pub async fn respond_success(
    ctx: &Context,
    interaction: &CommandInteraction,
    title: &str,
    description: &str,
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::new(colors::SUCCESS))
        .timestamp(Timestamp::now());

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

/// Send an info response to a command interaction
pub async fn respond_info(
    ctx: &Context,
    interaction: &CommandInteraction,
    title: &str,
    description: &str,
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::new(colors::BRAND));

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
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title("Error")
        .description(message)
        .color(Colour::new(colors::ERROR));

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
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title("Error")
        .description(message)
        .color(Colour::new(colors::ERROR));

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
