//! Embed builders for /config command responses

use rust_i18n::t;
use serenity::all::{Colour, CreateEmbed, CreateEmbedFooter};

use crate::commands::shared::colors;
use crate::entity::{guild_configs, user_configs};

// =============================================================================
// Show Handler Embeds - Guild
// =============================================================================

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

// =============================================================================
// Show Handler Embeds - User
// =============================================================================

/// Build embed for active user configuration
pub fn show_user_active(config: &user_configs::Model, locale: &str) -> CreateEmbed {
    let language_display = get_language_display_name(config.language.as_deref(), locale);

    CreateEmbed::default()
        .title(t!("embeds.config.show.user_active.title", locale = locale))
        .color(Colour::new(colors::BRAND))
        .field(
            t!(
                "embeds.config.show.user_active.field_status",
                locale = locale
            ),
            t!(
                "embeds.config.show.user_active.field_status_value",
                locale = locale
            ),
            true,
        )
        .field(
            t!(
                "embeds.config.show.user_active.field_delivery",
                locale = locale
            ),
            t!(
                "embeds.config.show.user_active.field_delivery_value",
                locale = locale
            ),
            true,
        )
        .field(
            t!(
                "embeds.config.show.user_active.field_language",
                locale = locale
            ),
            language_display,
            true,
        )
        .field(
            t!(
                "embeds.config.show.user_active.field_registered",
                locale = locale
            ),
            format!("<t:{}:R>", config.created_at.timestamp()),
            true,
        )
        .footer(CreateEmbedFooter::new(t!(
            "embeds.config.show.user_active.footer",
            locale = locale
        )))
}

/// Build embed for disabled user configuration
pub fn show_user_disabled(config: &user_configs::Model, locale: &str) -> CreateEmbed {
    let time = format!("<t:{}:R>", config.updated_at.timestamp());

    CreateEmbed::default()
        .title(t!(
            "embeds.config.show.user_disabled.title",
            locale = locale
        ))
        .description(t!(
            "embeds.config.show.user_disabled.description",
            locale = locale,
            time = time
        ))
        .color(Colour::new(colors::WARNING))
        .field(
            t!(
                "embeds.config.show.user_disabled.field_originally_registered",
                locale = locale
            ),
            format!("<t:{}:R>", config.created_at.timestamp()),
            true,
        )
}

/// Build intro embed for unregistered user
pub fn show_user_intro(locale: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(t!("embeds.config.show.user_intro.title", locale = locale))
        .description(t!(
            "embeds.config.show.user_intro.description",
            locale = locale
        ))
        .color(Colour::new(colors::BRAND))
        .field(
            t!(
                "embeds.config.show.user_intro.field_getting_started",
                locale = locale
            ),
            t!(
                "embeds.config.show.user_intro.field_getting_started_value",
                locale = locale
            ),
            false,
        )
        .field(
            t!(
                "embeds.config.show.user_intro.field_commands",
                locale = locale
            ),
            t!(
                "embeds.config.show.user_intro.field_commands_value",
                locale = locale
            ),
            false,
        )
        .footer(CreateEmbedFooter::new(t!(
            "embeds.config.show.user_intro.footer",
            locale = locale
        )))
}

// =============================================================================
// Unregister Handler Embeds
// =============================================================================

/// Build confirmation embed for unregister action
pub fn unregister_confirm(name: &str, is_guild: bool, locale: &str) -> CreateEmbed {
    let description = if is_guild {
        t!(
            "embeds.config.unregister.confirm.description_guild",
            locale = locale,
            name = name
        )
        .to_string()
    } else {
        t!(
            "embeds.config.unregister.confirm.description_user",
            locale = locale
        )
        .to_string()
    };

    CreateEmbed::default()
        .title(t!(
            "embeds.config.unregister.confirm.title",
            locale = locale
        ))
        .description(description)
        .color(Colour::new(colors::WARNING))
        .footer(CreateEmbedFooter::new(t!(
            "embeds.config.unregister.confirm.footer",
            locale = locale
        )))
}

/// Build success embed after unregistering
pub fn unregister_success(locale: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(t!(
            "embeds.config.unregister.success.title",
            locale = locale
        ))
        .description(t!(
            "embeds.config.unregister.success.description",
            locale = locale
        ))
        .color(Colour::new(colors::SUCCESS))
}

/// Build cancelled embed for unregister action
pub fn unregister_cancelled(locale: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(t!(
            "embeds.config.unregister.cancelled.title",
            locale = locale
        ))
        .description(t!(
            "embeds.config.unregister.cancelled.description",
            locale = locale
        ))
        .color(Colour::new(colors::BRAND))
}

/// Build error embed for failed unregister
pub fn unregister_error(locale: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(t!("embeds.config.unregister.error.title", locale = locale))
        .description(t!(
            "embeds.config.unregister.error.description",
            locale = locale
        ))
        .color(Colour::new(colors::ERROR))
}

// =============================================================================
// Language Handler Embeds
// =============================================================================

/// Get display name for language code using i18n
fn get_language_display_name(code: Option<&str>, locale: &str) -> String {
    match code {
        Some("en") => t!("embeds.config.language.names.en", locale = locale).to_string(),
        Some("ko") => t!("embeds.config.language.names.ko", locale = locale).to_string(),
        None => t!("embeds.config.language.names.auto", locale = locale).to_string(),
        Some(other) => other.to_string(),
    }
}

/// Build embed showing current language setting
pub fn language_current(current: Option<&str>, is_guild: bool, locale: &str) -> CreateEmbed {
    let display_name = get_language_display_name(current, locale);
    let context = if is_guild { "server" } else { "account" };

    CreateEmbed::default()
        .title(t!("embeds.config.language.current.title", locale = locale))
        .description(t!(
            "embeds.config.language.current.description",
            locale = locale,
            context = context,
            language = display_name
        ))
        .color(Colour::new(colors::BRAND))
        .field(
            t!(
                "embeds.config.language.current.field_available",
                locale = locale
            ),
            t!(
                "embeds.config.language.current.field_available_value",
                locale = locale
            ),
            false,
        )
        .footer(CreateEmbedFooter::new(t!(
            "embeds.config.language.current.footer",
            locale = locale
        )))
}

/// Build embed confirming language update
pub fn language_updated(language: Option<&str>, locale: &str) -> CreateEmbed {
    let display_name = get_language_display_name(language, locale);

    CreateEmbed::default()
        .title(t!("embeds.config.language.updated.title", locale = locale))
        .description(t!(
            "embeds.config.language.updated.description",
            locale = locale,
            language = display_name
        ))
        .color(Colour::new(colors::SUCCESS))
}
