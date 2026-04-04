//! Embed builders for /config command responses

mod guild;
mod language;
mod unregister;
mod user;

pub use guild::{show_guild_active, show_guild_disabled, show_guild_intro};
pub use language::{language_current, language_updated};
pub use unregister::{
    unregister_cancelled, unregister_confirm, unregister_error, unregister_success,
};
pub use user::{show_user_active, show_user_disabled, show_user_intro};
