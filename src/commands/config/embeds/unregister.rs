//! Unregister flow embed builders for /config command

use rust_i18n::t;
use serenity::all::{CreateEmbed, CreateEmbedFooter};

use crate::commands::shared::embeds;

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

    embeds::warning_embed(
        t!("embeds.config.unregister.confirm.title", locale = locale),
        description,
    )
    .footer(CreateEmbedFooter::new(t!(
        "embeds.config.unregister.confirm.footer",
        locale = locale
    )))
}

/// Build success embed after unregistering
pub fn unregister_success(locale: &str) -> CreateEmbed {
    embeds::success_embed(
        t!("embeds.config.unregister.success.title", locale = locale),
        t!(
            "embeds.config.unregister.success.description",
            locale = locale
        ),
    )
}

/// Build cancelled embed for unregister action
pub fn unregister_cancelled(locale: &str) -> CreateEmbed {
    embeds::info_embed(
        t!("embeds.config.unregister.cancelled.title", locale = locale),
        t!(
            "embeds.config.unregister.cancelled.description",
            locale = locale
        ),
    )
}

/// Build error embed for failed unregister
pub fn unregister_error(locale: &str) -> CreateEmbed {
    embeds::error_embed(
        t!("embeds.config.unregister.error.title", locale = locale),
        t!(
            "embeds.config.unregister.error.description",
            locale = locale
        ),
    )
}
