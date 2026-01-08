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

/// Default locale
pub const DEFAULT_LOCALE: &str = "en";

/// Convert Discord locale to our locale format
///
/// Discord sends: "ko", "en-US", "en-GB", "ja", etc.
/// We support: "en", "ko"
///
/// Returns "ko" if Korean, "en" otherwise.
fn to_locale(discord_locale: &str) -> &'static str {
    if discord_locale == "ko" { "ko" } else { "en" }
}

/// Resolve the locale for a command interaction (sync version)
///
/// Uses only Discord locale, no database lookup.
/// For full resolution with database fallback, use `resolve_locale_async`.
pub fn resolve_locale(interaction: &CommandInteraction) -> String {
    to_locale(&interaction.locale).to_string()
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
            return to_locale(&interaction.locale).to_string();
        }
    };

    // 1. Check guild preference first (if in guild context)
    if let Some(guild_id) = interaction.guild_id
        && let Some(lang) = get_guild_language(&db, guild_id).await
    {
        return lang;
    }

    // 2. Check user preference
    if let Some(lang) = get_user_language(&db, interaction.user.id).await {
        return lang;
    }

    // 3. Fall back to Discord locale
    to_locale(&interaction.locale).to_string()
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
            return to_locale(&interaction.locale).to_string();
        }
    };

    // 1. Check guild preference first (if in guild context)
    if let Some(guild_id) = interaction.guild_id
        && let Some(lang) = get_guild_language(&db, guild_id).await
    {
        return lang;
    }

    // 2. Check user preference
    if let Some(lang) = get_user_language(&db, interaction.user.id).await {
        return lang;
    }

    // 3. Fall back to Discord locale
    to_locale(&interaction.locale).to_string()
}

/// Resolve locale for alert sending (guild context)
pub async fn resolve_guild_locale(db: &DatabaseConnection, guild_id: GuildId) -> String {
    if let Some(lang) = get_guild_language(db, guild_id).await {
        return lang;
    }
    DEFAULT_LOCALE.to_string()
}

/// Resolve locale for alert sending (user DM context)
pub async fn resolve_user_locale(db: &DatabaseConnection, user_id: UserId) -> String {
    if let Some(lang) = get_user_language(db, user_id).await {
        return lang;
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
// Display Helpers
// =============================================================================

/// Get localized display name for a language code
///
/// Returns a human-readable name for the language code in the user's locale.
/// - `Some("en")` -> "English" (or localized equivalent)
/// - `Some("ko")` -> "Korean" (or localized equivalent)
/// - `None` -> "Auto-detect" (or localized equivalent)
pub fn get_language_display_name(code: Option<&str>, locale: &str) -> String {
    use rust_i18n::t;

    match code {
        Some("en") => t!("embeds.config.language.names.en", locale = locale).to_string(),
        Some("ko") => t!("embeds.config.language.names.ko", locale = locale).to_string(),
        None => t!("embeds.config.language.names.auto", locale = locale).to_string(),
        Some(other) => other.to_string(),
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
