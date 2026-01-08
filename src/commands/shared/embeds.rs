//! Shared embed builders for consistent styling
//!
//! These functions create basic embeds with standard colors.
//! Use these for simple title+description embeds. For complex embeds
//! with fields, footers, or custom content, build the embed directly.

use serenity::all::{Colour, CreateEmbed};

use super::colors;

/// Create a success embed (green)
///
/// Use for successful operations like registration, updates, etc.
pub fn success_embed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::new(colors::SUCCESS))
}

/// Create an info embed (brand blue)
///
/// Use for informational messages, cancellations, neutral status.
pub fn info_embed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::new(colors::BRAND))
}

/// Create an error embed (red)
///
/// Use for error messages, failed operations.
pub fn error_embed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::new(colors::ERROR))
}

/// Create a warning embed (yellow)
///
/// Use for warnings, confirmations before destructive actions.
pub fn warning_embed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::new(colors::WARNING))
}
