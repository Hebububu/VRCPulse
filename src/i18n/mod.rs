//! Internationalization (i18n) module
//!
//! Provides translation support using rust-i18n.
//!
//! # Language Resolution Priority
//! 1. Guild preference (from guild_configs.language) - if in guild context
//! 2. User preference (from user_configs.language)
//! 3. Discord locale (from interaction)
//! 4. Default: "en"

use sea_orm::{DatabaseConnection, EntityTrait};
use serenity::all::{CommandInteraction, ComponentInteraction, Context, GuildId, UserId};

use crate::entity::{guild_configs, user_configs};
use crate::state::AppStateKey;

/// Supported locales
pub const SUPPORTED_LOCALES: &[&str] = &["en", "ko"];

/// Default locale
pub const DEFAULT_LOCALE: &str = "en";

/// Check if a locale is supported
pub fn is_supported(locale: &str) -> bool {
    // Discord sends locales like "ko" or "en-US", we only care about the language part
    let lang = locale.split('-').next().unwrap_or(locale);
    SUPPORTED_LOCALES.contains(&lang)
}

/// Normalize Discord locale to our supported format
/// Discord sends "en-US", "ko", etc. We normalize to "en", "ko"
pub fn normalize_locale(locale: &str) -> &str {
    let lang = locale.split('-').next().unwrap_or(locale);
    if SUPPORTED_LOCALES.contains(&lang) {
        lang
    } else {
        DEFAULT_LOCALE
    }
}

/// Resolve the locale for a command interaction (sync version)
///
/// Uses only Discord locale, no database lookup.
/// For full resolution with database fallback, use `resolve_locale_async`.
pub fn resolve_locale(interaction: &CommandInteraction) -> String {
    let discord_locale = interaction.locale.as_str();
    normalize_locale(discord_locale).to_string()
}

/// Resolve the locale for a command interaction (async version with database lookup)
///
/// Priority:
/// 1. Guild preference (from guild_configs.language) - if in guild context and set
/// 2. User preference (from user_configs.language) - if set
/// 3. Discord locale (from interaction)
/// 4. Default: "en"
pub async fn resolve_locale_async(ctx: &Context, interaction: &CommandInteraction) -> String {
    // Get database connection
    let db = match get_db(ctx).await {
        Some(db) => db,
        None => {
            // No database, fall back to Discord locale
            return normalize_locale(interaction.locale.as_str()).to_string();
        }
    };

    // 1. Check guild preference first (if in guild context)
    if let Some(guild_id) = interaction.guild_id {
        if let Some(lang) = get_guild_language(&db, guild_id).await {
            if is_supported(&lang) {
                return lang;
            }
        }
    }

    // 2. Check user preference
    if let Some(lang) = get_user_language(&db, interaction.user.id).await {
        if is_supported(&lang) {
            return lang;
        }
    }

    // 3. Fall back to Discord locale
    let discord_locale = interaction.locale.as_str();
    let normalized = normalize_locale(discord_locale);
    if is_supported(normalized) {
        return normalized.to_string();
    }

    // 4. Default
    DEFAULT_LOCALE.to_string()
}

/// Resolve the locale for a component interaction (button, select menu, etc.)
///
/// Priority:
/// 1. Guild preference (from guild_configs.language) - if in guild context
/// 2. User preference (from user_configs.language)
/// 3. Discord locale (from interaction)
/// 4. Default: "en"
pub async fn resolve_locale_component(ctx: &Context, interaction: &ComponentInteraction) -> String {
    // Get database connection
    let db = match get_db(ctx).await {
        Some(db) => db,
        None => {
            // No database, fall back to Discord locale
            return normalize_locale(&interaction.locale).to_string();
        }
    };

    // 1. Check guild preference first (if in guild context)
    if let Some(guild_id) = interaction.guild_id {
        if let Some(lang) = get_guild_language(&db, guild_id).await {
            if is_supported(&lang) {
                return lang;
            }
        }
    }

    // 2. Check user preference
    if let Some(lang) = get_user_language(&db, interaction.user.id).await {
        if is_supported(&lang) {
            return lang;
        }
    }

    // 3. Fall back to Discord locale
    let discord_locale = &interaction.locale;
    let normalized = normalize_locale(discord_locale);
    if is_supported(normalized) {
        return normalized.to_string();
    }

    // 4. Default
    DEFAULT_LOCALE.to_string()
}

/// Resolve locale for alert sending (guild context)
pub async fn resolve_guild_locale(db: &DatabaseConnection, guild_id: GuildId) -> String {
    if let Some(lang) = get_guild_language(db, guild_id).await {
        if is_supported(&lang) {
            return lang;
        }
    }
    DEFAULT_LOCALE.to_string()
}

/// Resolve locale for alert sending (user DM context)
pub async fn resolve_user_locale(db: &DatabaseConnection, user_id: UserId) -> String {
    if let Some(lang) = get_user_language(db, user_id).await {
        if is_supported(&lang) {
            return lang;
        }
    }
    DEFAULT_LOCALE.to_string()
}

/// Resolve locale for alert sending (guild context, string ID)
///
/// Convenience function that accepts a string ID instead of GuildId.
/// Falls back to default locale on parse error.
pub async fn resolve_guild_locale_by_id(db: &DatabaseConnection, guild_id: &str) -> String {
    match guild_id.parse::<u64>() {
        Ok(id) => resolve_guild_locale(db, GuildId::new(id)).await,
        Err(_) => DEFAULT_LOCALE.to_string(),
    }
}

/// Resolve locale for alert sending (user DM context, string ID)
///
/// Convenience function that accepts a string ID instead of UserId.
/// Falls back to default locale on parse error.
pub async fn resolve_user_locale_by_id(db: &DatabaseConnection, user_id: &str) -> String {
    match user_id.parse::<u64>() {
        Ok(id) => resolve_user_locale(db, UserId::new(id)).await,
        Err(_) => DEFAULT_LOCALE.to_string(),
    }
}

// =============================================================================
// Database Helpers
// =============================================================================

async fn get_db(ctx: &Context) -> Option<std::sync::Arc<DatabaseConnection>> {
    let data = ctx.data.read().await;
    let state = data.get::<AppStateKey>()?;
    Some(state.read().await.database.clone())
}

async fn get_guild_language(db: &DatabaseConnection, guild_id: GuildId) -> Option<String> {
    guild_configs::Entity::find_by_id(guild_id.to_string())
        .one(db)
        .await
        .ok()
        .flatten()
        .and_then(|c| c.language)
}

async fn get_user_language(db: &DatabaseConnection, user_id: UserId) -> Option<String> {
    user_configs::Entity::find_by_id(user_id.to_string())
        .one(db)
        .await
        .ok()
        .flatten()
        .and_then(|c| c.language)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported() {
        assert!(is_supported("en"));
        assert!(is_supported("ko"));
        assert!(!is_supported("ja"));
        assert!(!is_supported("fr"));
    }

    #[test]
    fn test_normalize_locale() {
        assert_eq!(normalize_locale("en"), "en");
        assert_eq!(normalize_locale("en-US"), "en");
        assert_eq!(normalize_locale("en-GB"), "en");
        assert_eq!(normalize_locale("ko"), "ko");
        assert_eq!(normalize_locale("ja"), "en"); // unsupported -> default
        assert_eq!(normalize_locale("fr-FR"), "en"); // unsupported -> default
    }

    #[test]
    fn test_critical_translation_keys_exist() {
        use rust_i18n::t;

        let critical_keys = [
            "embeds.dashboard.title",
            "embeds.alerts.threshold.title",
            "embeds.intro.guild_join.title",
            "errors.generic",
            "time.just_now",
            "time.min_ago_one",
            "incident_types.login",
        ];

        for locale in SUPPORTED_LOCALES {
            for key in &critical_keys {
                let translated = t!(*key, locale = locale);
                // rust-i18n returns the key itself if not found
                assert_ne!(
                    translated.as_ref(),
                    *key,
                    "Missing key {} for locale {}",
                    key,
                    locale
                );
            }
        }
    }
}
