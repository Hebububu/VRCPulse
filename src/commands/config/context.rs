//! Configuration context detection and parsing

use serenity::all::{CommandInteraction, GuildId, UserId};

/// Configuration context (guild or user)
#[derive(Debug, Clone)]
pub enum ConfigContext {
    Guild(GuildId),
    User(UserId),
}

/// Determine if this is a guild or user install context
pub fn determine_context(interaction: &CommandInteraction) -> ConfigContext {
    // If guild_id is present, it's a guild context
    if let Some(guild_id) = interaction.guild_id {
        ConfigContext::Guild(guild_id)
    } else {
        // User install (DM context)
        ConfigContext::User(interaction.user.id)
    }
}

/// Parse context from button custom_id
/// Format: "prefix:guild:123456" or "prefix:user:123456"
pub fn parse_button_context(custom_id: &str) -> Option<ConfigContext> {
    let parts: Vec<&str> = custom_id.split(':').collect();
    if parts.len() >= 3 {
        let context_type = parts[parts.len() - 2];
        let id_str = parts[parts.len() - 1];

        match context_type {
            "guild" => id_str
                .parse::<u64>()
                .ok()
                .map(|id| ConfigContext::Guild(GuildId::new(id))),
            "user" => id_str
                .parse::<u64>()
                .ok()
                .map(|id| ConfigContext::User(UserId::new(id))),
            _ => None,
        }
    } else {
        None
    }
}
