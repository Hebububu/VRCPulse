//! Guild introduction embed
//!
//! Creates the welcome embed sent when the bot joins a new guild.

use rust_i18n::t;
use serenity::all::{
    ButtonStyle, Colour, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
};

use crate::commands::shared::colors;

/// Button custom IDs
pub const BUTTON_VIEW_KOREAN: &str = "intro_view_korean";
pub const BUTTON_SET_KOREAN: &str = "intro_set_korean";

/// Create the introduction embed for new guilds
pub fn create_intro_embed(locale: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(t!("embeds.intro.guild_join.title", locale = locale))
        .description(t!("embeds.intro.guild_join.description", locale = locale))
        .color(Colour::new(colors::BRAND))
        .field(
            t!(
                "embeds.intro.guild_join.field_getting_started",
                locale = locale
            ),
            t!(
                "embeds.intro.guild_join.field_getting_started_value",
                locale = locale
            ),
            false,
        )
        .field(
            t!("embeds.intro.guild_join.field_commands", locale = locale),
            t!(
                "embeds.intro.guild_join.field_commands_value",
                locale = locale
            ),
            false,
        )
        .footer(CreateEmbedFooter::new(t!(
            "embeds.intro.guild_join.footer",
            locale = locale
        )))
}

/// Create the "View in Korean" button
fn create_view_korean_button() -> CreateButton {
    CreateButton::new(BUTTON_VIEW_KOREAN)
        .label("한국어 설명 보기")
        .style(ButtonStyle::Secondary)
}

/// Create the "Set language to Korean" button
fn create_set_korean_button() -> CreateButton {
    CreateButton::new(BUTTON_SET_KOREAN)
        .label("봇 언어를 한국어로 설정")
        .style(ButtonStyle::Primary)
}

/// Create the initial intro message based on guild's Discord locale
///
/// Used when bot joins a guild or on first command from pending guild.
/// Discord sends "ko" for Korean, "en-US"/"en-GB" for English, etc.
///
/// - If locale is "ko": Korean intro (no button needed)
/// - Otherwise: English intro with "한국어 설명 보기" button
pub fn create_intro_message(discord_locale: &str) -> CreateMessage {
    use tracing::debug;

    debug!(discord_locale = %discord_locale, "Creating intro message");

    if discord_locale == "ko" {
        // Korean locale: Korean intro, no button needed
        debug!("Using Korean intro (no button)");
        let embed = create_intro_embed("ko");
        CreateMessage::new().embed(embed)
    } else {
        // Non-Korean locale: English intro with button to view in Korean
        debug!("Using English intro with Korean button");
        let embed = create_intro_embed("en");
        let button = create_view_korean_button();
        let action_row = CreateActionRow::Buttons(vec![button]);
        CreateMessage::new()
            .embed(embed)
            .components(vec![action_row])
    }
}

/// Create the Korean intro response with "Set language to Korean" button
///
/// Used when user clicks "한국어 설명 보기" button.
/// Returns a public response.
pub fn create_korean_intro_response() -> CreateInteractionResponse {
    let embed = create_intro_embed("ko");
    let button = create_set_korean_button();
    let action_row = CreateActionRow::Buttons(vec![button]);

    let message = CreateInteractionResponseMessage::new()
        .embed(embed)
        .components(vec![action_row]);

    CreateInteractionResponse::Message(message)
}

/// Create the confirmation response after setting language to Korean
///
/// Returns a public confirmation message.
pub fn create_set_korean_success_response() -> CreateInteractionResponse {
    let message = CreateInteractionResponseMessage::new()
        .content("설정 완료! 봇 언어가 한국어로 설정되었습니다.");

    CreateInteractionResponse::Message(message)
}

/// Create the error response when non-admin tries to set language
///
/// Returns an ephemeral error message.
pub fn create_admin_only_error_response() -> CreateInteractionResponse {
    let message = CreateInteractionResponseMessage::new()
        .content("관리자만 설정할 수 있습니다.")
        .ephemeral(true);

    CreateInteractionResponse::Message(message)
}
