pub mod config;

use serenity::all::CreateCommand;

/// Returns all admin slash command definitions
pub fn all() -> Vec<CreateCommand> {
    vec![config::register()]
}
