# /report

User incident reporting command for VRChat issues.

---

## Status

> **[NOT IMPLEMENTED]**: This command is documented but not yet implemented. The database schema is in place, but the Discord slash command handler does not exist.

---

## Usage

```
/report <incident_type> [details]
```

## Parameters

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `incident_type` | Choice | Yes | Type of issue being reported |
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

### 1. Guild Configuration Check

Verify guild is configured before accepting reports.

- **Requirement**: `guild_configs` entry must exist for the guild
- **Error if missing**: "Guild not configured. Admin must run `/config setup` first."

### 2. Duplicate Prevention

Prevent spam from same user reporting same issue type.

- **Window**: 5 minutes
- **Key**: `(guild_id, user_id, incident_type)`
- **Error if duplicate**: "You already reported this issue recently."

### 3. Store Report

Insert report into `user_reports` table.

### 4. User Confirmation

Return success embed to user.

---

## Response

### Success Response

```
[Title] Report Submitted
[Description] Thank you for reporting {incident_type}.
[Color] Green (0x57f287)
[Footer] If multiple users report this issue, an alert will be sent.
```

### Error Responses

| Situation | Title | Description |
| :--- | :--- | :--- |
| Guild not configured | Configuration Required | An administrator must run `/config setup` first. |
| Duplicate report (5 min) | Report Cooldown | You already reported this issue recently. |
| Details too long | Validation Error | Details must be under 500 characters. |

---

## Implementation

### Source Files

> **[PLANNED]**: Command files do not exist yet.

| Component | Planned File |
|-----------|--------------|
| Command definition | `src/commands/report.rs` [PLANNED] |

### Database Tables (Schema Ready)

| Table | Purpose | Entity |
|-------|---------|--------|
| `user_reports` | Store submitted reports | `src/entity/user_reports.rs:1-38` |
| `guild_configs` | Check guild configuration | `src/entity/guild_configs.rs:1-39` |

**Migration**: `migration/src/m20260103_001_create_table.rs:26-47`

---

## Implementation Checklist

- [ ] Create command definition in `src/commands/report.rs`
- [ ] Add slash command registration in `src/commands/mod.rs`
- [ ] Implement guild config check
- [ ] Implement duplicate prevention (5-min window)
- [ ] Implement report insertion
- [ ] Create user confirmation embed
- [ ] Add handler in `src/main.rs`

---

## Related Documents

- `docs/commands/config.md` - Guild configuration command (prerequisite)
- `docs/alerts/policy-user-threshold.md` - Alert triggering logic
- `docs/system/database-schema.md` - Table definitions
