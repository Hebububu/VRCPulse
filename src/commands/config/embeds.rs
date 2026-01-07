//! Embed builders for /config command responses

use serenity::all::{Colour, CreateEmbed, CreateEmbedFooter};

use crate::commands::shared::colors;
use crate::entity::{guild_configs, user_configs};

// =============================================================================
// Show Handler Embeds - Guild
// =============================================================================

/// Build embed for active guild configuration
pub fn show_guild_active(config: &guild_configs::Model) -> CreateEmbed {
    let channel_display = config
        .channel_id
        .as_ref()
        .map(|id| format!("<#{}>", id))
        .unwrap_or_else(|| "Not set".to_string());

    CreateEmbed::default()
        .title("VRCPulse Configuration")
        .color(Colour::new(colors::BRAND))
        .field("Status", "Active", true)
        .field("Channel", channel_display, true)
        .field(
            "Registered",
            format!("<t:{}:R>", config.created_at.timestamp()),
            true,
        )
        .footer(CreateEmbedFooter::new(
            "Use /config unregister to disable alerts",
        ))
}

/// Build embed for disabled guild configuration
pub fn show_guild_disabled(config: &guild_configs::Model) -> CreateEmbed {
    let channel_display = config
        .channel_id
        .as_ref()
        .map(|id| format!("<#{}>", id))
        .unwrap_or_else(|| "Not set".to_string());

    CreateEmbed::default()
        .title("VRCPulse - Unregistered")
        .description(format!(
            "This server was unregistered <t:{}:R>.\nRun `/config setup #channel` to re-enable alerts.",
            config.updated_at.timestamp()
        ))
        .color(Colour::new(colors::WARNING))
        .field("Previous Channel", channel_display, true)
        .field(
            "Originally Registered",
            format!("<t:{}:R>", config.created_at.timestamp()),
            true,
        )
}

/// Build intro embed for unregistered guild
pub fn show_guild_intro() -> CreateEmbed {
    CreateEmbed::default()
        .title("Welcome to VRCPulse!")
        .description("VRCPulse monitors VRChat server status and alerts you when issues occur.")
        .color(Colour::new(colors::BRAND))
        .field(
            "Getting Started",
            "1. Run `/config setup #channel` to register this server\n2. Check current VRChat status with `/status`",
            false,
        )
        .field(
            "Commands",
            "- `/config setup <channel>` - Register and set alert channel\n- `/config show` - View current settings\n- `/config unregister` - Disable alerts",
            false,
        )
        .footer(CreateEmbedFooter::new(
            "This server isn't registered yet. Run /config setup #channel to get started!",
        ))
}

// =============================================================================
// Show Handler Embeds - User
// =============================================================================

/// Build embed for active user configuration
pub fn show_user_active(config: &user_configs::Model) -> CreateEmbed {
    CreateEmbed::default()
        .title("VRCPulse Configuration")
        .color(Colour::new(colors::BRAND))
        .field("Status", "Active", true)
        .field("Delivery", "Direct Messages", true)
        .field(
            "Registered",
            format!("<t:{}:R>", config.created_at.timestamp()),
            true,
        )
        .footer(CreateEmbedFooter::new(
            "Use /config unregister to disable alerts",
        ))
}

/// Build embed for disabled user configuration
pub fn show_user_disabled(config: &user_configs::Model) -> CreateEmbed {
    CreateEmbed::default()
        .title("VRCPulse - Unregistered")
        .description(format!(
            "You unregistered <t:{}:R>.\nRun `/config setup` to re-enable DM alerts.",
            config.updated_at.timestamp()
        ))
        .color(Colour::new(colors::WARNING))
        .field(
            "Originally Registered",
            format!("<t:{}:R>", config.created_at.timestamp()),
            true,
        )
}

/// Build intro embed for unregistered user
pub fn show_user_intro() -> CreateEmbed {
    CreateEmbed::default()
        .title("Welcome to VRCPulse!")
        .description("VRCPulse monitors VRChat server status and alerts you when issues occur.")
        .color(Colour::new(colors::BRAND))
        .field(
            "Getting Started",
            "1. Run `/config setup` to register for DM alerts\n2. Check current VRChat status with `/status`",
            false,
        )
        .field(
            "Commands",
            "- `/config setup` - Register for DM alerts\n- `/config show` - View current settings\n- `/config unregister` - Disable alerts",
            false,
        )
        .footer(CreateEmbedFooter::new(
            "You aren't registered yet. Run /config setup to get started!",
        ))
}

// =============================================================================
// Unregister Handler Embeds
// =============================================================================

/// Build confirmation embed for unregister action
pub fn unregister_confirm(name: &str, is_guild: bool) -> CreateEmbed {
    let description = if is_guild {
        format!(
            "Are you sure you want to unregister **{}**?\n\nThis will stop all VRCPulse alerts for this server.",
            name
        )
    } else {
        "Are you sure you want to unregister?\n\nThis will stop all VRCPulse DM alerts.".to_string()
    };

    CreateEmbed::default()
        .title("Confirm Unregister")
        .description(description)
        .color(Colour::new(colors::WARNING))
        .footer(CreateEmbedFooter::new(
            "This confirmation expires in 15 minutes",
        ))
}

/// Build success embed after unregistering
pub fn unregister_success() -> CreateEmbed {
    CreateEmbed::default()
        .title("Unregistered")
        .description(
            "VRCPulse alerts have been disabled.\n\nYou can re-register anytime with `/config setup`.",
        )
        .color(Colour::new(colors::SUCCESS))
}

/// Build cancelled embed for unregister action
pub fn unregister_cancelled() -> CreateEmbed {
    CreateEmbed::default()
        .title("Cancelled")
        .description("Unregister cancelled. Your configuration remains active.")
        .color(Colour::new(colors::BRAND))
}

/// Build error embed for failed unregister
pub fn unregister_error() -> CreateEmbed {
    CreateEmbed::default()
        .title("Error")
        .description("Failed to unregister. Please try again.")
        .color(Colour::new(colors::ERROR))
}
