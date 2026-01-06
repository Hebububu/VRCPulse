# /admin config

Admin command for managing bot configuration. Currently supports polling interval settings.

---

## Status

> **[DISABLED]**: This command is implemented but currently disabled in bot registration. See `src/commands/mod.rs:11`. To enable, uncomment: `commands.extend(admin::all());`

---

## Permissions

- Requires **Administrator** permission in the guild

---

## Subcommands

### `/admin config show`

Display current polling interval settings.

**Response:**
```
[Title] Polling Intervals
[Color] Blue (0x00b0f4)
[Fields]
  Status: 60s (inline)
  Incident: 60s (inline)
  Maintenance: 60s (inline)
  Metrics: 60s (inline)
[Footer] Use /admin config set to change
```

### `/admin config set <poller> <seconds>`

Update a specific poller's interval.

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `poller` | Choice | Yes | `status`, `incident`, `maintenance`, `metrics` |
| `seconds` | Integer | Yes | Interval in seconds (60-3600) |

**Validation:**
- Minimum: 60 seconds
- Maximum: 3600 seconds (1 hour)

**Success Response:**
```
[Title] Configuration Updated
[Description] Polling interval has been changed.
[Color] Green (0x57f287)
[Fields]
  Poller: {name} (inline)
  New Interval: {seconds}s (inline)
```

**Error Response:**
```
[Title] Invalid Interval
[Description] Interval must be between 60 and 3600 seconds.
[Color] Red (0xed4245)
```

### `/admin config reset`

Reset all polling intervals to default values (60 seconds).

**Response:**
```
[Title] Configuration Reset
[Description] All polling intervals have been reset to default values.
[Color] Green (0x57f287)
[Fields]
  Status: 60s (inline)
  Incident: 60s (inline)
  Maintenance: 60s (inline)
  Metrics: 60s (inline)
```

---

## Implementation

### Source Files

| Component | File | Lines |
|-----------|------|-------|
| Command definition | `src/commands/admin/config.rs` | 12-62 |
| Handler | `src/commands/admin/config.rs` | 65-101 |
| Show handler | `src/commands/admin/config.rs` | 103-140 |
| Set handler | `src/commands/admin/config.rs` | 142-190 |
| Reset handler | `src/commands/admin/config.rs` | 192-220 |
| Config module | `src/collector/config.rs` | 1-271 |

### Database

| Table | Purpose |
|-------|---------|
| `bot_config` | Store polling intervals as key-value pairs |

**Keys:**
- `polling.status` - Status poller interval
- `polling.incident` - Incident poller interval
- `polling.maintenance` - Maintenance poller interval
- `polling.metrics` - Metrics poller interval

### Dynamic Updates

Uses `tokio::sync::watch` channels for live interval updates without restart.

- **Config init**: `src/collector/config.rs:162-199`
- **Watch channel**: `src/collector/config.rs:91-97`
- **Poll loop**: `src/collector/mod.rs:39-73`

---

## Error Handling

| Error | Response |
| :--- | :--- |
| Invalid interval range | "Interval must be between 60 and 3600 seconds" |
| Database error | "Failed to save configuration" |
| Missing permission | Discord handles (command not shown to non-admins) |

---

## Related Documents

- `docs/system/data-collector.md` - How polling intervals affect data collection
- `docs/system/database-schema.md` - `bot_config` table schema
