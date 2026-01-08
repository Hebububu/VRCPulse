pub mod config;
mod embeds;

use serenity::all::CreateCommand;

/// Returns all admin slash command definitions
pub fn all() -> Vec<CreateCommand> {
    vec![config::register()]
}
