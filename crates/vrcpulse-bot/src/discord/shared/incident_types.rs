//! Shared utilities for incident type display names

use rust_i18n::t;

/// Available incident type keys for reporting
pub const INCIDENT_TYPE_KEYS: &[&str] = &["login", "instance", "api", "auth", "download", "other"];

/// Get display name for incident type using i18n (default locale)
pub fn display_name(incident_type: &str) -> String {
    match incident_type {
        "login" => t!("incident_types.login").to_string(),
        "instance" => t!("incident_types.instance").to_string(),
        "api" => t!("incident_types.api").to_string(),
        "auth" => t!("incident_types.auth").to_string(),
        "download" => t!("incident_types.download").to_string(),
        "other" => t!("incident_types.other").to_string(),
        _ => incident_type.to_string(),
    }
}

/// Get localized display name for incident type
pub fn display_name_localized(incident_type: &str, locale: &str) -> String {
    let key = format!("incident_types.{}", incident_type);
    let translated = t!(&key, locale = locale);
    // If translation key doesn't exist, rust-i18n returns the key itself
    if translated.contains("incident_types.") {
        incident_type.to_string()
    } else {
        translated.to_string()
    }
}
