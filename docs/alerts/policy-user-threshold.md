# User Report Threshold Alert

Alert triggered when multiple users report the same incident type within a time window.

---

## Status

> **[NOT IMPLEMENTED]**: This alert policy is planned but not yet implemented.

---

## Overview

When enabled, this alert monitors user-submitted reports via `/report` command. When the number of reports for a specific incident type exceeds the configured threshold within a time window, an alert is sent to the guild's configured channel.

---

## Trigger Conditions

### Threshold Check

Alert fires when:
- Report count >= `guild_configs.threshold` (default: 5)
- Within time window (default: 60 minutes, configurable via `guild_configs.report_interval`)
- Per incident type (login, instance, api, auth, download, other)

**Migration defaults**: `migration/src/m20260103_001_create_table.rs:17-18`

---

## Deduplication

### Strategy
Per incident_type + hourly time window

### Reference ID Format
```
threshold_{incident_type}_{hour_window}
```

**Examples:**
- `threshold_login_2026-01-05T12`
- `threshold_api_2026-01-05T13`

### Behavior
- Same incident_type can only alert once per hour per guild
- Different incident_types alert independently
- New hour = new alert window

---

## Alert Message

### Embed Format

```
[Title] High Report Volume Detected
[Description] **{count}** users reported **{incident_type}** in the last {window} minutes
[Color] Orange (0xf0b132)
[Fields]
  Incident Type: {display_name} (inline)
  Report Count: {count} (inline)
  Time Window: {window} minutes (inline)
  Recent Reports: (not inline)
    - @User1 - 2 min ago: "{content preview}"
    - @User2 - 5 min ago
    - @User3 - 8 min ago
[Footer] Check /status for official VRChat status
[Timestamp] Current time
```

---

## Configuration

### Guild Settings (`guild_configs` table)

| Column | Default | Range | Description |
|--------|---------|-------|-------------|
| `threshold` | 5 | 1-100 | Reports needed to trigger |
| `report_interval` | 60 | 5-1440 | Time window (minutes) |
| `channel_id` | null | N/A | Alert destination channel |
| `enabled` | true | N/A | Enable/disable alerts |

**Source**: `migration/src/m20260103_001_create_table.rs:9-24`

---

## Implementation

### Planned Source Files [NOT IMPLEMENTED]

> **Note**: These files will be created during alert system implementation.

| Component | Planned File |
|-----------|--------------|
| Alert service | `src/alerts/mod.rs` [PLANNED] |
| Threshold check | `src/alerts/threshold.rs` [PLANNED] |
| Notification builder | `src/alerts/notifications.rs` [PLANNED] |

### Database Tables

- `user_reports`: Source of report data
- `guild_configs`: Threshold and channel configuration
- `sent_alerts`: Deduplication tracking

---

## Related Documents

- `docs/commands/report.md` - /report command (creates user reports)
- `docs/system/database-schema.md` - Table definitions
- `docs/alerts/AGENTS.md` - Alert documentation guide
