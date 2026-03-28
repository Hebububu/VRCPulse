//! Guild-related embed builders for /config command

use rust_i18n::t;
use serenity::all::{Colour, CreateEmbed, CreateEmbedFooter};

use crate::commands::shared::colors;
use crate::entity::guild_configs;
use crate::i18n::get_language_display_name;

/// Build embed for active guild configuration
pub fn show_guild_active(config: &guild_configs::Model, locale: &str) -> CreateEmbed {
    let channel_display = config
        .channel_id
        .as_ref()
        .map(|id| format!("<#{}>", id))
        .unwrap_or_else(|| {
            t!(
                "embeds.config.show.guild_active.field_channel_not_set",
                locale = locale
            )
            .to_string()
        });

    let language_display = get_language_display_name(config.language.as_deref(), locale);

    CreateEmbed::default()
        .title(t!("embeds.config.show.guild_active.title", locale = locale))
        .color(Colour::new(colors::BRAND))
        .field(
            t!(
                "embeds.config.show.guild_active.field_status",
                locale = locale
            ),
            t!(
                "embeds.config.show.guild_active.field_status_value",
                locale = locale
            ),
            true,
        )
        .field(
            t!(
                "embeds.config.show.guild_active.field_channel",
                locale = locale
            ),
            channel_display,
            true,
        )
        .field(
            t!(
                "embeds.config.show.guild_active.field_language",
                locale = locale
            ),
            language_display,
            true,
        )
        .field(
            t!(
                "embeds.config.show.guild_active.field_registered",
                locale = locale
            ),
            format!("<t:{}:R>", config.created_at.timestamp()),
            true,
        )
        .footer(CreateEmbedFooter::new(t!(
            "embeds.config.show.guild_active.footer",
            locale = locale
        )))
}

/// Build embed for disabled guild configuration
pub fn show_guild_disabled(config: &guild_configs::Model, locale: &str) -> CreateEmbed {
    let channel_display = config
        .channel_id
        .as_ref()
        .map(|id| format!("<#{}>", id))
        .unwrap_or_else(|| {
            t!(
                "embeds.config.show.guild_active.field_channel_not_set",
                locale = locale
            )
            .to_string()
        });

    let time = format!("<t:{}:R>", config.updated_at.timestamp());

    CreateEmbed::default()
        .title(t!(
            "embeds.config.show.guild_disabled.title",
            locale = locale
        ))
        .description(t!(
            "embeds.config.show.guild_disabled.description",
            locale = locale,
            time = time
        ))
        .color(Colour::new(colors::WARNING))
        .field(
            t!(
                "embeds.config.show.guild_disabled.field_previous_channel",
                locale = locale
            ),
            channel_display,
            true,
        )
        .field(
            t!(
                "embeds.config.show.guild_disabled.field_originally_registered",
                locale = locale
            ),
            format!("<t:{}:R>", config.created_at.timestamp()),
            true,
        )
}

/// Build intro embed for unregistered guild
pub fn show_guild_intro(locale: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(t!("embeds.config.show.guild_intro.title", locale = locale))
        .description(t!(
            "embeds.config.show.guild_intro.description",
            locale = locale
        ))
        .color(Colour::new(colors::BRAND))
        .field(
            t!(
                "embeds.config.show.guild_intro.field_getting_started",
                locale = locale
            ),
            t!(
                "embeds.config.show.guild_intro.field_getting_started_value",
                locale = locale
            ),
            false,
        )
        .field(
            t!(
                "embeds.config.show.guild_intro.field_commands",
                locale = locale
            ),
            t!(
                "embeds.config.show.guild_intro.field_commands_value",
                locale = locale
            ),
            false,
        )
        .footer(CreateEmbedFooter::new(t!(
            "embeds.config.show.guild_intro.footer",
            locale = locale
        )))
}
