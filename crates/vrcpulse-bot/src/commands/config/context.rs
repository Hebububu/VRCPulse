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
