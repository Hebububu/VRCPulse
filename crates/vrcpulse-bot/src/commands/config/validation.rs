//! Permission validation for config commands

use serenity::all::{ChannelId, Context, GuildChannel, GuildId, Permissions, UserId};
use tracing::error;

// =============================================================================
// Channel Validation
// =============================================================================

/// Validate bot has required permissions in the target channel
pub async fn validate_channel_permissions(
    ctx: &Context,
    channel_id: ChannelId,
) -> Result<(), String> {
    // Get channel
    let channel = channel_id
        .to_channel(&ctx.http)
        .await
        .map_err(|_| "Could not access that channel. Please check it exists and I can see it.")?;

    let guild_channel = channel
        .guild()
        .ok_or("That doesn't appear to be a server channel.")?;

    // Get bot's permissions in the channel
    let bot_id = ctx.cache.current_user().id;
    let permissions = get_channel_permissions(ctx, &guild_channel, bot_id).await?;

    // Check required permissions
    if !permissions.send_messages() {
        return Err(
            "I don't have permission to send messages in that channel. Please give me the **Send Messages** permission."
                .to_string(),
        );
    }

    if !permissions.embed_links() {
        return Err(
            "I don't have permission to send embeds in that channel. Please give me the **Embed Links** permission."
                .to_string(),
        );
    }

    Ok(())
}

/// Get bot's permissions in a channel
async fn get_channel_permissions(
    ctx: &Context,
    channel: &GuildChannel,
    user_id: UserId,
) -> Result<Permissions, String> {
    let guild_id = channel.guild_id;

    // Try to get from cache first
    if let Some(guild) = ctx.cache.guild(guild_id)
        && let Some(member) = guild.members.get(&user_id)
    {
        return Ok(guild.user_permissions_in(channel, member));
    }

    // Fallback: fetch member
    let member = guild_id
        .member(&ctx.http, user_id)
        .await
        .map_err(|_| "Could not verify my permissions in that channel.")?;

    let guild = ctx
        .cache
        .guild(guild_id)
        .ok_or("Could not access guild information.")?;

    Ok(guild.user_permissions_in(channel, &member))
}

// =============================================================================
// Admin Validation
// =============================================================================

/// Result of admin permission check
pub enum AdminCheckResult {
    /// User is an administrator
    IsAdmin,
    /// User is not an administrator
    NotAdmin,
    /// Could not verify permissions (API error, cache miss, etc.)
    CouldNotVerify(String),
}

/// Validate that a user has ADMINISTRATOR permission in a guild
pub async fn validate_guild_admin(
    ctx: &Context,
    guild_id: GuildId,
    user_id: UserId,
) -> AdminCheckResult {
    // Try cache first
    if let Some(guild) = ctx.cache.guild(guild_id)
        && let Some(member) = guild.members.get(&user_id)
    {
        let perms = guild.member_permissions(member);
        return if perms.administrator() {
            AdminCheckResult::IsAdmin
        } else {
            AdminCheckResult::NotAdmin
        };
    }

    // Fallback: fetch member and check permissions
    match guild_id.member(&ctx.http, user_id).await {
        Ok(member) => {
            if let Some(guild) = ctx.cache.guild(guild_id) {
                let perms = guild.member_permissions(&member);
                return if perms.administrator() {
                    AdminCheckResult::IsAdmin
                } else {
                    AdminCheckResult::NotAdmin
                };
            }
            AdminCheckResult::CouldNotVerify("Guild not in cache after member fetch".to_string())
        }
        Err(e) => {
            error!(guild_id = %guild_id, user_id = %user_id, error = %e, "Failed to fetch member for admin check");
            AdminCheckResult::CouldNotVerify(format!("API error: {}", e))
        }
    }
}
