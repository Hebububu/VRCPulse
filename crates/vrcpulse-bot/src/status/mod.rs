//! Status commands module

mod dashboard;
pub mod visualization;

use serenity::all::CreateCommand;

/// Returns all status command definitions
pub fn all() -> Vec<CreateCommand> {
    vec![dashboard::register()]
}

pub use dashboard::run;
