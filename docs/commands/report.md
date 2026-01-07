# /report

User incident reporting command for VRChat issues.

---

## Status

> **[IMPLEMENTED]**: This command is fully implemented.

---

## Usage

```
/report <type> [details]
```

## Parameters

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `type` | Choice | Yes | Type of issue being reported |
| `details` | String | No | Additional context (max 500 chars) |

### Incident Types (Choices)

| Value | Display Name |
| :--- | :--- |
| `login` | Login Issues |
| `instance` | Instance/World Loading |
| `api` | API/Website Issues |
| `auth` | Authentication Issues |
| `download` | Content Download Issues |
| `other` | Other Issues |

---

## Behavior Flow

### 1. Registration Check

- **Guild context**: Requires `guild_configs.enabled = true`
- **User context**: Requires `user_configs.enabled = true`
- **Unregistered user**: Shows intro embed with setup instructions

### 2. Duplicate Prevention

- **Window**: 5 minutes
- **Scope**: Per user globally (any report within window triggers cooldown)
- **Response**: Shows when user can report again

### 3. Details Validation

- **Max length**: 500 characters
- **Error**: Shows character count if exceeded

### 4. Store Report

- Inserts into `user_reports` table
- `status` = `active`
- `guild_id` = null for user-install context

### 5. Response

- Shows success with count of similar reports
- Anonymous (no guild/user names shown)

---

## Response Examples

### Success
```
Report Submitted

Thank you for reporting Login Issues.

6 others reported this issue in the last 60 minutes.

[Footer] Your report helps us detect widespread issues.
```

### Duplicate (cooldown)
```
Report Cooldown

You recently submitted a report.
You can report again <t:1736251234:R>.
```

### User Not Registered (user-install)
```
Welcome to VRCPulse!

VRCPulse monitors VRChat server status and alerts you when issues occur.

Getting Started:
1. Run /config setup to register for DM alerts
2. Check current VRChat status with /status

[Footer] Run /config setup to start receiving alerts and submit reports!
```

---

## Global Report Pool

Reports from all sources (guilds + user-install) contribute to a single global count per incident type. Guild/user identities are kept anonymous in responses.

---

## Configuration

Global settings in `bot_config` table:

| Key | Default | Description |
|-----|---------|-------------|
| `report_threshold` | 5 | Reports needed to trigger alert |
| `report_interval` | 60 | Time window (minutes) for counting |

---

## Implementation

### Source Files

| Component | File |
|-----------|------|
| Command definition | `src/commands/report.rs` |
| Handler | `src/commands/report.rs` |
| Registration check | `src/commands/report.rs` |
| Atomic insert with race handling | `src/commands/report.rs` |
| Report count query | `src/commands/report.rs` |

### Database Tables

| Table | Purpose |
|-------|---------|
| `user_reports` | Store submitted reports |
| `guild_configs` | Check guild registration |
| `user_configs` | Check user registration |
| `bot_config` | Global threshold settings |

### Status Field

| Status | Description |
|--------|-------------|
| `active` | Active report (used for counting and alerts) |

> **Note**: Currently only `active` status is used. Future versions may add `resolved` or `expired` for report lifecycle management.

---

## Related Documents

- `docs/commands/config.md` - Registration command (prerequisite)
- `docs/alerts/policy-user-threshold.md` - Alert triggering logic
