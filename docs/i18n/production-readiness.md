# i18n Production Readiness Plan

Complete remaining i18n work to make the feature production-ready.

---

## Overview

| Property | Value |
|----------|-------|
| **Current Coverage** | ~85% (commands, responses localized) |
| **Target Coverage** | 100% user-facing strings |
| **Blocking Issues** | Alert system, guild intro, shared errors |
| **Estimated Effort** | ~2-3 hours |

---

## Stage 1: Add String-Based Locale Resolution Functions

**Goal**: Add convenience functions for alert system that accepts string IDs

**Status**: [x] Complete

**File**: `src/i18n/mod.rs`

**Tasks**:
- [ ] Add `resolve_guild_locale_by_id(db, guild_id: &str) -> String`
- [ ] Add `resolve_user_locale_by_id(db, user_id: &str) -> String`
- [ ] Add unit tests for new functions

**Implementation**:
```rust
// After line 156, add:

/// Resolve locale for alert sending (guild context, string ID)
pub async fn resolve_guild_locale_by_id(db: &DatabaseConnection, guild_id: &str) -> String {
    match guild_id.parse::<u64>() {
        Ok(id) => resolve_guild_locale(db, GuildId::new(id)).await,
        Err(_) => DEFAULT_LOCALE.to_string(),
    }
}

/// Resolve locale for alert sending (user DM context, string ID)
pub async fn resolve_user_locale_by_id(db: &DatabaseConnection, user_id: &str) -> String {
    match user_id.parse::<u64>() {
        Ok(id) => resolve_user_locale(db, UserId::new(id)).await,
        Err(_) => DEFAULT_LOCALE.to_string(),
    }
}
```

**Success Criteria**:
- Functions compile and work with string IDs
- Parse errors fallback to "en"
- Tests pass

---

## Stage 2: Localize Alert System

**Goal**: Use i18n for all alert embed strings

**Status**: [x] Complete

**File**: `src/alerts/threshold.rs`

**Tasks**:
- [ ] Add `use rust_i18n::t;` import
- [ ] Add `use crate::i18n::resolve_guild_locale_by_id;`
- [ ] Add `use crate::i18n::resolve_user_locale_by_id;`
- [ ] Remove `INCIDENT_TYPES` constant (lines 25-32)
- [ ] Refactor `get_incident_display_name()` to accept locale
- [ ] Refactor `build_alert_embed()` to accept locale
- [ ] Update `send_guild_alert()` to resolve and pass locale
- [ ] Update `send_user_alert()` to resolve and pass locale

**Changes Detail**:

| Location | Current | Change To |
|----------|---------|-----------|
| Line 25-32 | `INCIDENT_TYPES` constant | DELETE - use translation keys |
| Line 383-389 | `get_incident_display_name(incident_type: &str)` | `get_incident_display_name(incident_type: &str, locale: &str)` |
| Line 391 | `build_alert_embed(...)` | Add `locale: &str` parameter |
| Line 402 | `"No recent reports"` | `t!("embeds.alerts.threshold.no_recent_reports", locale = locale)` |
| Line 410 | `"- Just now"` | `format!("- {}", t!("time.just_now", locale = locale))` |
| Line 412 | `"- 1 min ago"` | `format!("- {}", t!("time.min_ago_one", locale = locale))` |
| Line 414 | `format!("- {} min ago", mins)` | `format!("- {}", t!("time.min_ago_many", n = mins, locale = locale))` |
| Line 422 | `"High Report Volume Detected"` | `t!("embeds.alerts.threshold.title", locale = locale)` |
| Line 423-426 | Description format | Use `t!("embeds.alerts.threshold.description", ...)` |
| Line 428 | `"Recent Reports"` | `t!("embeds.alerts.threshold.field_recent_reports", locale = locale)` |
| Line 429-431 | Footer | `t!("embeds.alerts.threshold.footer", locale = locale)` |

**In send_guild_alert() (line ~275)**:
```rust
// Add before build_alert_embed call:
let locale = crate::i18n::resolve_guild_locale_by_id(db, &guild.guild_id).await;
let embed = build_alert_embed(incident_type, count, interval, recent_reports, &locale);
```

**In send_user_alert() (line ~345)**:
```rust
// Add before build_alert_embed call:
let locale = crate::i18n::resolve_user_locale_by_id(db, &user.user_id).await;
let embed = build_alert_embed(incident_type, count, interval, recent_reports, &locale);
```

**Translation Keys** (already exist in `locales/*.json`):
- `embeds.alerts.threshold.title`
- `embeds.alerts.threshold.description`
- `embeds.alerts.threshold.field_recent_reports`
- `embeds.alerts.threshold.no_recent_reports`
- `embeds.alerts.threshold.footer`
- `time.just_now`
- `time.min_ago_one`
- `time.min_ago_many`
- `incident_types.{login,instance,api,auth,download,other}`

**Success Criteria**:
- Alert embeds render in configured language
- Korean guilds receive Korean alerts
- English guilds receive English alerts
- Fallback to English works

---

## Stage 3: Localize Guild Join Intro

**Goal**: Use Discord server locale for guild join message

**Status**: [x] Complete

**File**: `src/main.rs`

**Tasks**:
- [ ] Add `use rust_i18n::t;` import (if not present)
- [ ] Add `use crate::i18n::normalize_locale;` import
- [ ] Modify `create_intro_embed()` to accept locale parameter
- [ ] Update caller to pass guild's preferred_locale

**Current** (lines 189-210):
```rust
fn create_intro_embed() -> CreateEmbed {
    CreateEmbed::default()
        .title("Welcome to VRCPulse!")
        // ... hardcoded strings
}
```

**After**:
```rust
fn create_intro_embed(locale: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(t!("embeds.intro.guild_join.title", locale = locale))
        .description(t!("embeds.intro.guild_join.description", locale = locale))
        .color(Colour::new(colors::BRAND))
        .field(
            t!("embeds.intro.guild_join.field_getting_started", locale = locale),
            t!("embeds.intro.guild_join.field_getting_started_value", locale = locale),
            false,
        )
        .field(
            t!("embeds.intro.guild_join.field_commands", locale = locale),
            t!("embeds.intro.guild_join.field_commands_value", locale = locale),
            false,
        )
        .footer(CreateEmbedFooter::new(
            t!("embeds.intro.guild_join.footer", locale = locale),
        ))
}
```

**Caller Update** (in guild_create event handler):
```rust
// Get guild's preferred locale, normalize, fallback to "en"
let locale = guild.preferred_locale
    .as_ref()
    .map(|l| normalize_locale(l))
    .unwrap_or(crate::i18n::DEFAULT_LOCALE);

let embed = create_intro_embed(locale);
```

**Translation Keys** (already exist):
- `embeds.intro.guild_join.title`
- `embeds.intro.guild_join.description`
- `embeds.intro.guild_join.field_getting_started`
- `embeds.intro.guild_join.field_getting_started_value`
- `embeds.intro.guild_join.field_commands`
- `embeds.intro.guild_join.field_commands_value`
- `embeds.intro.guild_join.footer`

**Success Criteria**:
- Korean Discord servers see Korean intro
- English Discord servers see English intro
- Unsupported locales fallback to English

---

## Stage 4: Localize Shared Error Functions

**Goal**: Add locale parameter to shared error response functions

**Status**: [x] Complete

**File**: `src/commands/shared.rs`

**Tasks**:
- [ ] Add `use rust_i18n::t;` import
- [ ] Add locale param to `respond_error()`
- [ ] Add locale param to `respond_button_error()`
- [ ] Use `t!("embeds.dashboard.error_title", locale = locale)` for title
- [ ] Update all 25 callers of `respond_error()`
- [ ] Update all 4 callers of `respond_button_error()`

**Function Signature Changes**:

```rust
// Before
pub async fn respond_error(
    ctx: &Context,
    interaction: &CommandInteraction,
    message: &str,
) -> Result<(), serenity::Error>

// After
pub async fn respond_error(
    ctx: &Context,
    interaction: &CommandInteraction,
    message: &str,
    locale: &str,
) -> Result<(), serenity::Error>
```

```rust
// Before
pub async fn respond_button_error(
    ctx: &Context,
    interaction: &ComponentInteraction,
    message: &str,
) -> Result<(), serenity::Error>

// After
pub async fn respond_button_error(
    ctx: &Context,
    interaction: &ComponentInteraction,
    message: &str,
    locale: &str,
) -> Result<(), serenity::Error>
```

**Callers to Update**:

| File | Function | Count |
|------|----------|-------|
| `src/commands/config/mod.rs` | `respond_error` | 2 |
| `src/commands/config/handlers.rs` | `respond_error` | 10 |
| `src/commands/config/handlers.rs` | `respond_button_error` | 4 |
| `src/commands/report.rs` | `respond_error` | 4 |
| `src/commands/admin/config.rs` | `respond_error` | 9 |

**Note**: Admin commands use English-only, pass `"en"` as locale.

**Success Criteria**:
- Error titles show in user's language
- All callers compile with new signature
- `cargo clippy` passes

---

## Stage 5: Add i18n Tests

**Goal**: Comprehensive test coverage for i18n functionality

**Status**: [x] Complete

**File**: `src/i18n/mod.rs`

**Tasks**:
- [ ] Add test for `resolve_guild_locale_by_id` with valid ID
- [ ] Add test for `resolve_guild_locale_by_id` with invalid ID (fallback)
- [ ] Add test for `resolve_user_locale_by_id` with valid ID
- [ ] Add test for `resolve_user_locale_by_id` with invalid ID (fallback)
- [ ] Add test verifying critical translation keys exist in all locales

**Test Cases**:
```rust
#[test]
fn test_resolve_guild_locale_by_id_invalid() {
    // Invalid ID should fallback to DEFAULT_LOCALE
    // Note: Can't test DB lookup without mock, but can test parse failure
}

#[test]
fn test_critical_translation_keys_exist() {
    use rust_i18n::t;
    
    let critical_keys = [
        "embeds.dashboard.title",
        "embeds.alerts.threshold.title",
        "embeds.intro.guild_join.title",
        "errors.generic",
    ];
    
    for locale in SUPPORTED_LOCALES {
        for key in &critical_keys {
            let translated = t!(key, locale = locale);
            // rust-i18n returns the key if not found
            assert!(!translated.contains('.'), 
                "Missing key {} for locale {}", key, locale);
        }
    }
}
```

**Success Criteria**:
- All new tests pass
- Critical translation keys verified for all locales

---

## Stage 6: Verify and Clean Up

**Goal**: Final verification and cleanup

**Status**: [x] Complete

**Tasks**:
- [ ] Run `cargo fmt`
- [ ] Run `cargo clippy` - fix any warnings
- [ ] Run `cargo test` - all tests pass
- [ ] Run `cargo build --release` - compiles successfully
- [ ] Manual test: Korean Discord user sees Korean alerts
- [ ] Manual test: English Discord user sees English alerts
- [ ] Manual test: Guild join shows correct language
- [ ] Update `korean-support.md` stage statuses

**Success Criteria**:
- All automated checks pass
- Manual testing confirms correct behavior
- Documentation updated

---

## Files Modified Summary

| File | Stage | Changes |
|------|-------|---------|
| `src/i18n/mod.rs` | 1, 5 | Add string ID functions, tests |
| `src/alerts/threshold.rs` | 2 | Full i18n integration |
| `src/main.rs` | 3 | Localize guild intro |
| `src/commands/shared.rs` | 4 | Add locale param |
| `src/commands/config/mod.rs` | 4 | Update respond_error calls |
| `src/commands/config/handlers.rs` | 4 | Update respond_error/button calls |
| `src/commands/report.rs` | 4 | Update respond_error calls |
| `src/commands/admin/config.rs` | 4 | Update respond_error calls (en) |

---

## Rollback Plan

If issues found in production:
1. Alert locale can be hardcoded to "en" in `threshold.rs`
2. Guild intro can revert to hardcoded English
3. Shared error title can be hardcoded back to "Error"

No database changes required - safe to rollback code only.
