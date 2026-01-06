# VRChat Status Change Alert

Alert triggered when VRChat's official status changes.

---

## Status

> **[NOT IMPLEMENTED]**: This alert policy is planned but not yet implemented.

---

## Overview

When enabled, this alert monitors VRChat's official status API. When the system status changes (e.g., from `none` to `minor`, `major`, or `critical`), an alert is sent to configured guilds.

---

## Trigger Conditions

### Status Change Detection

Alert fires when:
- System indicator changes from previous state
- New incidents are created or updated
- Scheduled maintenances begin or complete

### Monitored States

| Indicator | Severity | Description |
|-----------|----------|-------------|
| `none` | Normal | All systems operational |
| `minor` | Low | Minor performance impact |
| `major` | Medium | Significant service degradation |
| `critical` | High | Major outage |

---

## Deduplication

### Strategy
Per incident/maintenance ID

### Reference ID Format
```
status_{incident_id}_{status}
```

**Examples:**
- `status_abc123_investigating`
- `status_abc123_resolved`
- `maintenance_xyz789_in_progress`

### Behavior
- Each incident status change generates one alert
- Resolved status always triggers alert
- Same incident won't re-alert for same status

---

## Alert Message

### Incident Alert

```
[Title] VRChat Incident: {title}
[Description] {impact} impact incident detected
[Color] Based on impact (Yellow/Orange/Red)
[Fields]
  Status: {status} (inline)
  Impact: {impact} (inline)
  Started: {started_at} (inline)
  Latest Update: {update_body}
[Footer] Source: status.vrchat.com
[Timestamp] Incident start time
```

### Maintenance Alert

```
[Title] VRChat Scheduled Maintenance
[Description] {title}
[Color] Blue (0x5865f2)
[Fields]
  Status: {status} (inline)
  Scheduled: {scheduled_for} to {scheduled_until}
[Footer] Source: status.vrchat.com
[Timestamp] Current time
```

---

## Configuration

### Guild Settings

| Column | Description |
|--------|-------------|
| `channel_id` | Alert destination channel |
| `enabled` | Enable/disable alerts |

---

## Implementation

### Planned Source Files [NOT IMPLEMENTED]

> **Note**: These files will be created during alert system implementation.

| Component | Planned File |
|-----------|--------------|
| Alert service | `src/alerts/mod.rs` [PLANNED] |
| Status change detection | `src/alerts/status.rs` [PLANNED] |
| Notification builder | `src/alerts/notifications.rs` [PLANNED] |

### Data Sources

- `status_logs`: System status history
- `incidents`: Official incident records
- `incident_updates`: Incident update history
- `maintenances`: Scheduled maintenance records

---

## Related Documents

- `docs/system/data-collector.md` - How status data is collected
- `docs/system/database-schema.md` - Table definitions
- `docs/alerts/AGENTS.md` - Alert documentation guide
