//! Handler functions for /config subcommands

mod language;
mod setup;
mod show;
mod unregister;

pub use language::handle_language;
pub use setup::handle_setup;
pub use show::handle_show;
pub use unregister::{handle_unregister, handle_unregister_cancel, handle_unregister_confirm};

use crate::commands::shared::is_button;

// =============================================================================
// Button Configuration
// =============================================================================

/// Module name for config command buttons
const MODULE: &str = "config";

/// Action name for unregister confirmation button
const ACTION_UNREGISTER_CONFIRM: &str = "unregister_confirm";

/// Action name for unregister cancel button
const ACTION_UNREGISTER_CANCEL: &str = "unregister_cancel";

/// Generate button ID for unregister confirmation
pub fn unregister_confirm_button_id(context_type: &str, id: impl ToString) -> String {
    crate::commands::shared::button_id_with_context(
        MODULE,
        ACTION_UNREGISTER_CONFIRM,
        context_type,
        id,
    )
}

/// Generate button ID for unregister cancel
pub fn unregister_cancel_button_id(context_type: &str, id: impl ToString) -> String {
    crate::commands::shared::button_id_with_context(
        MODULE,
        ACTION_UNREGISTER_CANCEL,
        context_type,
        id,
    )
}

/// Check if button ID matches unregister confirmation
pub fn is_confirm_button(custom_id: &str) -> bool {
    is_button(custom_id, MODULE, ACTION_UNREGISTER_CONFIRM)
}

/// Check if button ID matches unregister cancel
pub fn is_cancel_button(custom_id: &str) -> bool {
    is_button(custom_id, MODULE, ACTION_UNREGISTER_CANCEL)
}
