# /config

Guild configuration for VRCPulse bot settings.

---

## Status

> **[NOT IMPLEMENTED]**: This command is planned but not yet implemented.

---

## Overview

This command will allow guild administrators to configure VRCPulse settings for their server, including:
- Setting the notification channel for alerts
- Adjusting report thresholds
- Enabling/disabling bot features

---

## Planned Usage

```
/config setup <channel>
/config threshold <count>
/config interval <minutes>
/config disable
```

---

## Planned Parameters

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `channel` | Channel | Yes (setup) | Channel for status alerts |
| `count` | Integer | Yes (threshold) | Reports needed to trigger alert |
| `minutes` | Integer | Yes (interval) | Time window for report counting |

---

## Implementation

### Planned Source Files [NOT IMPLEMENTED]

> **Note**: Command file will be created during implementation.

| Component | Planned File |
|-----------|--------------|
| Command definition | `src/commands/config.rs` [PLANNED] |
| Guild config entity | `src/entity/guild_configs.rs` (exists) |

### Database Tables

- `guild_configs`: Stores per-guild configuration

**Migration**: `migration/src/m20260103_001_create_table.rs:9-24`

---

## Related Documents

- `docs/commands/report.md` - /report command (requires guild config)
- `docs/alerts/policy-user-threshold.md` - Alert triggered by user reports
- `docs/system/database-schema.md` - Table definitions
