//! Button utilities for Discord component interactions
//!
//! Standard button ID format: `{module}_{action}[:{context_type}:{context_id}]`

/// Generate button custom_id: `{module}_{action}`
///
/// Use this for buttons that don't need to preserve context across interactions.
pub fn button_id(module: &str, action: &str) -> String {
    format!("{}_{}", module, action)
}

/// Generate button custom_id with context: `{module}_{action}:{context_type}:{id}`
///
/// Use this for buttons that need to preserve entity context (e.g., guild/user ID)
/// across interactions, since Discord doesn't maintain state between button clicks.
pub fn button_id_with_context(
    module: &str,
    action: &str,
    context_type: &str,
    id: impl ToString,
) -> String {
    format!("{}_{}:{}:{}", module, action, context_type, id.to_string())
}

/// Parse context from button custom_id.
///
/// Returns `(context_type, id)` if the custom_id matches the pattern `...:type:id`.
pub fn parse_button_context(custom_id: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = custom_id.split(':').collect();
    if parts.len() >= 3 {
        Some((parts[parts.len() - 2], parts[parts.len() - 1]))
    } else {
        None
    }
}

/// Check if button custom_id matches a specific module and action prefix.
///
/// This checks if the custom_id starts with `{module}_{action}`.
pub fn is_button(custom_id: &str, module: &str, action: &str) -> bool {
    custom_id.starts_with(&button_id(module, action))
}
