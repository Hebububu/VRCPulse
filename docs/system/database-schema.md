# Database Schema Design

This document defines the SQLite database schema for **VRCPulse**. The schema is designed to store VRChat server status history, performance metrics, official incidents, and bot configurations.

## Overview

We use **SQLite** as the primary data store and **Sea-ORM** as the Object-Relational Mapper (ORM). All mutable tables include `created_at` and `updated_at` timestamps for auditing. Immutable log tables (e.g., `status_logs`, `component_logs`, `metric_logs`) only include `created_at`.

---

## Source Files

| Component | File | Lines |
|-----------|------|-------|
| Migration (all tables) | `migration/src/m20260103_001_create_table.rs` | 1-410 |
| Guild configs entity | `src/entity/guild_configs.rs` | 1-39 |
| User reports entity | `src/entity/user_reports.rs` | 1-38 |
| Status logs entity | `src/entity/status_logs.rs` | 1-22 |
| Component logs entity | `src/entity/component_logs.rs` | 1-21 |
| Incidents entity | `src/entity/incidents.rs` | 1-32 |
| Incident updates entity | `src/entity/incident_updates.rs` | 1-38 |
| Maintenances entity | `src/entity/maintenances.rs` | 1-22 |
| Metric logs entity | `src/entity/metric_logs.rs` | 1-25 |
| Sent alerts entity | `src/entity/sent_alerts.rs` | 1-39 |
| Bot config entity | `src/entity/bot_config.rs` | 1-18 |

---

## Tables

### 1. Guild Configuration (`guild_configs`)
Stores Discord server (guild) specific settings.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `guild_id` | String | PK | Discord Guild ID |
| `channel_id` | String | Nullable | Designated channel for status updates |
| `report_interval` | Integer | Default: 60 | Interval in minutes for scheduled reports |
| `threshold` | Integer | Default: 5 | User report count to trigger a broadcast alert |
| `enabled` | Boolean | Default: true | Whether the bot is active for this guild |
| `created_at` | DateTime | | |
| `updated_at` | DateTime | | |

### 2. User Incident Reports (`user_reports`)
Stores outage reports submitted by users via `/report`.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | Integer | PK, AutoInc | Unique report ID |
| `guild_id` | String | FK (guild_configs) | Origin guild of the report |
| `user_id` | String | | Discord User ID of the reporter |
| `incident_type` | String | | e.g., 'login', 'instance', 'api' |
| `content` | Text | Nullable | Detailed description from the user |
| `status` | String | Default: 'pending' | `pending`, `acknowledged`, `dismissed` |
| `created_at` | DateTime | | |

### 3. System Status Logs (`status_logs`)
Stores overall system status snapshots from VRChat Status API.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | Integer | PK, AutoInc | |
| `indicator` | String | | `none`, `minor`, `major`, `critical` |
| `description` | Text | | Human-readable status description |
| `source_timestamp` | DateTime | Unique | `page.updated_at` from API to prevent duplicates |
| `created_at` | DateTime | | |

### 4. Component Status Logs (`component_logs`)
Stores individual component status (e.g., API, Website) for history tracking.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | Integer | PK, AutoInc | |
| `component_id` | String | | Unique ID provided by VRChat |
| `name` | String | | Component name (e.g., "API / Website") |
| `status` | String | | `operational`, `degraded_performance`, etc. |
| `source_timestamp` | DateTime | | Timestamp when this status was captured |
| `created_at` | DateTime | | |

### 5. Official Incidents (`incidents`)
Stores official incident records from VRChat.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | String | PK | VRChat Incident ID (e.g., `9cn3s26glx4d`) |
| `title` | String | | Incident title |
| `impact` | String | | Impact level |
| `status` | String | | `investigating`, `identified`, `monitoring`, `resolved` |
| `started_at` | DateTime | | |
| `resolved_at` | DateTime | Nullable | |
| `created_at` | DateTime | | |
| `updated_at` | DateTime | | |

### 6. Incident Updates (`incident_updates`)
Stores chronological updates for each incident.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | String | PK | Update ID |
| `incident_id` | String | FK (incidents) | Associated incident |
| `body` | Text | | Update message content |
| `status` | String | | Status at the time of update |
| `published_at` | DateTime | | When the update was posted |
| `created_at` | DateTime | | |
| `updated_at` | DateTime | | |

### 7. Scheduled Maintenances (`maintenances`)
Stores planned maintenance information.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | String | PK | Maintenance ID |
| `title` | String | | Maintenance title |
| `status` | String | | `scheduled`, `in_progress`, `completed` |
| `scheduled_for` | DateTime | | Start time |
| `scheduled_until` | DateTime | | End time |
| `created_at` | DateTime | | |
| `updated_at` | DateTime | | |

### 8. Performance Metrics (`metric_logs`)
Stores time-series data from CloudFront Metrics.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | Integer | PK, AutoInc | |
| `metric_name` | String | | `api_latency`, `visits`, `api_errors`, etc. |
| `value` | Double | | Measured value |
| `unit` | String | | `ms`, `count`, `percent`, etc. |
| `interval_sec` | Integer | | Data resolution (e.g., 60 for 1-minute data) |
| `timestamp` | DateTime | | Original timestamp from the source |
| `created_at` | DateTime | | |

> **Note**: Composite unique constraint on `(metric_name, timestamp)` prevents duplicate data points.

### 9. Sent Alerts (`sent_alerts`)
Tracks which alerts have been sent to which guilds to prevent duplicate notifications.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | Integer | PK, AutoInc | |
| `guild_id` | String | FK (guild_configs) | Target guild |
| `alert_type` | String | | `incident`, `maintenance`, `threshold` |
| `reference_id` | String | | ID of the incident/maintenance/etc. |
| `notified_at` | DateTime | | When the alert was sent |
| `created_at` | DateTime | | |

> **Note**: Composite unique constraint on `(guild_id, alert_type, reference_id)` prevents duplicate alerts.

### 10. Bot Configuration (`bot_config`)
Global bot configuration storage (key-value pairs).

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `key` | String | PK | Configuration key (e.g., `polling.status`) |
| `value` | String | | Configuration value (JSON-serialized if needed) |
| `updated_at` | DateTime | | Last modification time |

**Default Keys:**
| Key | Default Value | Description |
| :--- | :--- | :--- |
| `polling.status` | `60` | Status poller interval (seconds) |
| `polling.incident` | `60` | Incident poller interval (seconds) |
| `polling.maintenance` | `60` | Maintenance poller interval (seconds) |
| `polling.metrics` | `60` | Metrics poller interval (seconds) |

**Source**: `migration/src/m20260103_001_create_table.rs:246-249`

---

## Optimization & Integrity

### Unique Constraints
- **`status_logs.source_timestamp`**: Prevents duplicate snapshots.
- **`metric_logs(metric_name, timestamp)`**: Prevents duplicate metric points.

### Relationships
- `incidents` (1) ↔ (N) `incident_updates`
- `guild_configs` (1) ↔ (N) `user_reports`
- `guild_configs` (1) ↔ (N) `sent_alerts`

### Indexes

The following indexes are recommended for query performance:

```sql
-- User reports: count reports within time window per guild
CREATE INDEX idx_user_reports_guild_created
ON user_reports(guild_id, created_at);

-- Component logs: query history by component
CREATE INDEX idx_component_logs_component_time
ON component_logs(component_id, source_timestamp);

-- Metric logs: time-series range queries
CREATE INDEX idx_metric_logs_name_time
ON metric_logs(metric_name, timestamp);

-- Sent alerts: check if alert was already sent
CREATE INDEX idx_sent_alerts_lookup
ON sent_alerts(guild_id, alert_type, reference_id);
```

---

## Data Retention Policy

Time-series data can grow indefinitely. The following retention policies are recommended:

| Table | Retention Period | Rationale |
| :--- | :--- | :--- |
| `metric_logs` | 90 days | Sufficient for trend analysis and dashboards |
| `status_logs` | 180 days | Longer history for system health review |
| `component_logs` | 180 days | Matches status_logs |
| `user_reports` | 365 days | May be needed for pattern analysis |
| `sent_alerts` | 30 days | Only needed to prevent recent duplicates |

Cleanup can be implemented via:
- Scheduled task (e.g., Tokio cron job)
- Sea-ORM migration script
- Manual SQL: `DELETE FROM metric_logs WHERE created_at < datetime('now', '-90 days');`