pub mod hello;

use serenity::all::{Command, Context, CreateCommand};
use tracing::info;

/// Returns all slash command definitions
pub fn all() -> Vec<CreateCommand> {
    vec![hello::register()]
}

/// Register global slash commands
pub async fn register_global(ctx: &Context) -> Result<(), serenity::Error> {
    let commands = Command::set_global_commands(&ctx.http, all()).await?;
    info!("Registered {} global commands", commands.len());
    Ok(())
}

/// Register slash commands to a specific guild (for development, instant update)
pub async fn register_guild(ctx: &Context, guild_id: u64) -> Result<(), serenity::Error> {
    let guild_id = serenity::all::GuildId::new(guild_id);
    let commands = guild_id.set_commands(&ctx.http, all()).await?;
    info!(
        "Registered {} commands to guild {}",
        commands.len(),
        guild_id
    );
    Ok(())
}
