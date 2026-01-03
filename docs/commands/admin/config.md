# /admin config

Admin command for managing bot configuration. Currently supports polling interval settings.

---

## Permissions

- Requires **Administrator** permission in the guild
- Bot owner can use this command in any guild (optional, for future)

---

## Subcommands

### `/admin config polling`

Manage data collector polling intervals.

#### `/admin config polling show`

Display current polling interval settings.

**Response Embed:**

```rust
CreateEmbed::default()
    .title("Polling Intervals")
    .color(Colour::new(0x00b0f4))
    .fields(vec![
        ("Status", "60s", true),
        ("Incident", "60s", true),
        ("Maintenance", "60s", true),
        ("Metrics", "60s", true),
    ])
    .footer(CreateEmbedFooter::new("Use /admin config polling set to change"))
```

#### `/admin config polling set <poller> <seconds>`

Update a specific poller's interval.

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `poller` | String (Choice) | Yes | `status`, `incident`, `maintenance`, `metrics` |
| `seconds` | Integer | Yes | Interval in seconds (min: 60, max: 3600) |

**Validation:**
- All pollers: minimum 60 seconds
- All pollers: maximum 3600 seconds (1 hour)

**Response Embed (Success):**

```rust
CreateEmbed::default()
    .title("Configuration Updated")
    .description("Polling interval has been changed.")
    .color(Colour::new(0x57f287)) // Green
    .fields(vec![
        ("Poller", "incident", true),
        ("New Interval", "20s", true),
    ])
    .timestamp(Timestamp::now())
```

**Response Embed (Error):**

```rust
CreateEmbed::default()
    .title("Invalid Interval")
    .description("Interval must be between 60 and 3600 seconds for `incident` poller.")
    .color(Colour::new(0xed4245)) // Red
```

#### `/admin config polling reset`

Reset all polling intervals to default values (60 seconds).

**Response Embed:**

```rust
CreateEmbed::default()
    .title("Configuration Reset")
    .description("All polling intervals have been reset to default values.")
    .color(Colour::new(0x57f287)) // Green
    .fields(vec![
        ("Status", "60s", true),
        ("Incident", "60s", true),
        ("Maintenance", "60s", true),
        ("Metrics", "60s", true),
    ])
    .timestamp(Timestamp::now())
```

---

## Database Schema

### `bot_config` Table

Global bot configuration storage (key-value).

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `key` | String | PK | Configuration key |
| `value` | String | | JSON-serialized value |
| `updated_at` | DateTime | | Last modification time |

**Polling Interval Keys:**
- `polling.status` → `"60"`
- `polling.incident` → `"60"`
- `polling.maintenance` → `"60"`
- `polling.metrics` → `"60"`

---

## Implementation Notes

### Dynamic Interval Updates

The collector should check the database for interval changes:

1. **Option A: Reload on next tick**
   - Each poll loop reads current interval from DB before sleeping
   - Simple but adds DB query per tick

2. **Option B: Watch channel** (Recommended)
   - Use `tokio::sync::watch` channel
   - Command updates DB and sends new value to channel
   - Collector receives update and adjusts interval immediately

```rust
// Pseudocode for Option B
pub struct CollectorConfig {
    pub status_interval: watch::Receiver<Duration>,
    pub incident_interval: watch::Receiver<Duration>,
    // ...
}

// In command handler
config_tx.send(new_interval)?;
db.update_config("polling.status", new_interval).await?;
```

### Startup Behavior

On bot startup:
1. Load intervals from `bot_config` table
2. If key not found, return error (migration seeds default values)
3. Initialize watch channels with loaded values

---

## Error Handling

| Error | Response |
| :--- | :--- |
| Invalid interval range | "❌ Interval must be between 60 and 3600 seconds" |
| Database error | "❌ Failed to save configuration" |
| Missing config key | Startup error (run migration to seed defaults) |
| Missing permission | "❌ You need Administrator permission to use this command" |
