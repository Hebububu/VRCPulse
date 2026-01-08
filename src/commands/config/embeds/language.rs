//! Language setting embed builders for /config command

use rust_i18n::t;
use serenity::all::{CreateEmbed, CreateEmbedFooter};

use crate::commands::shared::embeds;
use crate::i18n::get_language_display_name;

/// Build embed showing current language setting
pub fn language_current(current: Option<&str>, is_guild: bool, locale: &str) -> CreateEmbed {
    let display_name = get_language_display_name(current, locale);
    let context = if is_guild { "server" } else { "account" };

    embeds::info_embed(
        t!("embeds.config.language.current.title", locale = locale),
        t!(
            "embeds.config.language.current.description",
            locale = locale,
            context = context,
            language = display_name
        ),
    )
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

    embeds::success_embed(
        t!("embeds.config.language.updated.title", locale = locale),
        t!(
            "embeds.config.language.updated.description",
            locale = locale,
            language = display_name
        ),
    )
}
