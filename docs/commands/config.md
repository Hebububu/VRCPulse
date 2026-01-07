# /config

Guild and user registration for VRCPulse alerts.

---

## Status

> **[IMPLEMENTED]**: This command is fully implemented.

---

## Overview

The `/config` command allows guilds and users to register for VRCPulse alerts. Registered guilds receive alerts in a designated channel, while user-install users receive alerts via DM.

**Features**:
- Register guild with notification channel
- Register user for DM alerts (user-install)
- View current configuration
- Unregister (soft delete, preserves history)
- Automatic welcome message on bot join

---

## Usage

```
/config setup [channel]    - Register for alerts (channel required for guilds)
/config show               - View current configuration
/config unregister         - Disable alerts (button confirmation)
```

---

## Parameters

| Subcommand | Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- | :--- |
| `setup` | `channel` | Channel | Guild: Yes, User: No | Channel for alerts (guild only) |
| `show` | - | - | - | No parameters |
| `unregister` | - | - | - | No parameters |

---

## Behavior

### /config setup

**Guild Context**:
1. Validates channel parameter is provided
2. Validates bot has `SEND_MESSAGES` and `EMBED_LINKS` permissions
3. Creates or re-enables guild config
4. If already registered with different channel, updates channel

**User Context** (user-install):
1. Creates or re-enables user config
2. Alerts sent via DM (no channel needed)

### /config show

| State | Response |
|-------|----------|
| Never registered | Welcome intro + getting started guide |
| Previously registered (disabled) | Shows previous settings + re-enable prompt |
| Currently registered | Shows current config (channel, registration date) |

### /config unregister

1. Shows confirmation with Cancel/Confirm buttons
2. On confirm: Sets `enabled=false` (soft delete)
3. Historical data (reports, alerts) preserved

---

## Permissions

- **Guild context**: Requires `ADMINISTRATOR` permission
- **User context**: No special permissions required

---

## Implementation

### Source Files

| Component | File |
|-----------|------|
| Command definition & registration | `src/commands/config.rs` |
| Command handler & context detection | `src/commands/config.rs` |
| Setup/Show/Unregister handlers | `src/commands/config.rs` |
| Button handlers (confirm/cancel) | `src/commands/config.rs` |
| Database operations | `src/commands/config.rs` |
| Channel & permission validation | `src/commands/config.rs` |
| Guild join intro | `src/main.rs` |

### Database Tables

- `guild_configs`: Guild registration and settings
- `user_configs`: User registration (for user-install)

**Migration**: `migration/src/m20260103_001_create_table.rs` (all tables in single migration)

### Routing

Command registered in `src/commands/mod.rs` and routed in `src/main.rs` (interaction_create handler).

Button interactions handled in `src/main.rs` (component interaction handler).

---

## Guild Join Introduction

When the bot joins a new guild:

1. If guild has system channel: Send welcome embed immediately
2. If no system channel: Log and skip (users discover bot via `/config show`)

**Implementation**: `src/main.rs` (guild_create handler)

---

## Related Documents

- `docs/alerts/policy-user-threshold.md` - Alert triggering (uses guild config)
- `docs/system/database-schema.md` - Table definitions
