# Database Schema Design

This document defines the SQLite database schema for **VRCPulse**. The schema is designed to store VRChat server status history, performance metrics, official incidents, and bot configurations.

## Overview

We use **SQLite** as the primary data store and **Sea-ORM** as the Object-Relational Mapper (ORM). All mutable tables include `created_at` and `updated_at` timestamps for auditing. Immutable log tables (e.g., `status_logs`, `component_logs`, `metric_logs`) only include `created_at`.

---

## Source Files

| Component | File |
|-----------|------|
| Migration (all tables) | `migration/src/m20260103_001_create_table.rs` |
| Entity modules | `src/entity/*.rs` |
| Entity prelude | `src/entity/prelude.rs` |

---

## Tables

### 1. Guild Configuration (`guild_configs`)
Stores Discord server (guild) registration and settings.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `guild_id` | String | PK | Discord Guild ID |
| `channel_id` | String | Nullable | Designated channel for alerts |
| `enabled` | Boolean | Default: true | Whether alerts are active for this guild |
| `created_at` | DateTime | | Registration timestamp |
| `updated_at` | DateTime | | Last modification |

> **Note**: Threshold and interval settings are global (see `bot_config` table).

### 2. User Configuration (`user_configs`)
Stores user registration for DM alerts (user-install context).

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `user_id` | String | PK | Discord User ID |
| `enabled` | Boolean | Default: true | Whether DM alerts are active |
| `created_at` | DateTime | | Registration timestamp |
| `updated_at` | DateTime | | Last modification |

### 3. User Incident Reports (`user_reports`)
Stores outage reports submitted by users via `/report`.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | Integer | PK, AutoInc | Unique report ID |
| `guild_id` | String | Nullable | Origin guild (null for user-install) |
| `user_id` | String | | Discord User ID of the reporter |
| `incident_type` | String | | e.g., 'login', 'instance', 'api' |
| `content` | Text | Nullable | Detailed description from the user |
| `status` | String | Default: 'active' | Report status |
| `created_at` | DateTime | | |

**Indexes**:
- `idx_user_reports_type_created`: `(incident_type, created_at)` for threshold queries
- `idx_user_reports_user_type_created`: `(user_id, incident_type, created_at)` for duplicate check

### 4. System Status Logs (`status_logs`)
Stores overall system status snapshots from VRChat Status API.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | Integer | PK, AutoInc | |
| `indicator` | String | | `none`, `minor`, `major`, `critical` |
| `description` | Text | | Human-readable status description |
| `source_timestamp` | DateTime | Unique | `page.updated_at` from API to prevent duplicates |
| `created_at` | DateTime | | |

### 5. Component Status Logs (`component_logs`)
Stores individual component status (e.g., API, Website) for history tracking.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | Integer | PK, AutoInc | |
| `component_id` | String | | Unique ID provided by VRChat |
| `name` | String | | Component name (e.g., "API / Website") |
| `status` | String | | `operational`, `degraded_performance`, etc. |
| `source_timestamp` | DateTime | | Timestamp when this status was captured |
| `created_at` | DateTime | | |

### 6. Official Incidents (`incidents`)
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

### 7. Incident Updates (`incident_updates`)
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

### 8. Scheduled Maintenances (`maintenances`)
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

### 9. Performance Metrics (`metric_logs`)
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

### 10. Sent Alerts (`sent_alerts`)
Tracks which alerts have been sent to prevent duplicate notifications.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | Integer | PK, AutoInc | |
| `guild_id` | String | Nullable | Target guild (null for user alerts) |
| `user_id` | String | Nullable | Target user (null for guild alerts) |
| `alert_type` | String | | `incident`, `maintenance`, `threshold` |
| `reference_id` | String | | ID of the incident/maintenance/time-block |
| `notified_at` | DateTime | | When the alert was sent |
| `created_at` | DateTime | | |

> **Note**: Composite unique constraint on `(guild_id, user_id, alert_type, reference_id)` prevents duplicate alerts. Either `guild_id` or `user_id` is set, not both.

### 11. Bot Configuration (`bot_config`)
Global bot configuration storage (key-value pairs).

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `key` | String | PK | Configuration key |
| `value` | String | | Configuration value |
| `updated_at` | DateTime | | Last modification time |

**Seeded Keys:**
| Key | Default Value | Description |
| :--- | :--- | :--- |
| `polling.status` | `60` | Status poller interval (seconds) |
| `polling.incident` | `60` | Incident poller interval (seconds) |
| `polling.maintenance` | `60` | Maintenance poller interval (seconds) |
| `polling.metrics` | `60` | Metrics poller interval (seconds) |
| `report_threshold` | `1` | Reports needed to trigger alert |
| `report_interval` | `60` | Time window for counting reports (minutes) |

### 12. Command Logs (`command_logs`)
Audit trail for slash command executions.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | Integer | PK, AutoInc | |
| `command_name` | String | | Command name (e.g., 'config') |
| `subcommand` | String | Nullable | Subcommand if any (e.g., 'setup') |
| `user_id` | String | | User who executed |
| `guild_id` | String | Nullable | Guild context (null for DMs) |
| `channel_id` | String | Nullable | Channel where executed |
| `executed_at` | DateTime | | Execution timestamp |

**Indexes**:
- `idx_command_logs_user_id`: For user activity queries
- `idx_command_logs_guild_id`: For guild activity queries

---

## Optimization & Integrity

### Unique Constraints
- **`status_logs.source_timestamp`**: Prevents duplicate snapshots.
- **`metric_logs(metric_name, timestamp)`**: Prevents duplicate metric points.

### Relationships
- `incidents` (1) ↔ (N) `incident_updates`

> **Note**: `user_reports` and `sent_alerts` previously had FKs to `guild_configs`, but these were removed to support user-install context where `guild_id` is null.

### Indexes

All indexes are created in migration. Key indexes:

```sql
-- User reports: threshold queries by incident type
CREATE INDEX idx_user_reports_type_created
ON user_reports(incident_type, created_at);

-- User reports: duplicate/cooldown check
CREATE INDEX idx_user_reports_user_type_created
ON user_reports(user_id, incident_type, created_at);

-- Component logs: query history by component
CREATE INDEX idx_component_logs_component_time
ON component_logs(component_id, source_timestamp);

-- Metric logs: time-series range queries (unique)
CREATE UNIQUE INDEX idx_metric_logs_name_time
ON metric_logs(metric_name, timestamp);

-- Sent alerts: deduplication (unique)
CREATE UNIQUE INDEX idx_sent_alerts_lookup
ON sent_alerts(guild_id, user_id, alert_type, reference_id);

-- Command logs: activity queries
CREATE INDEX idx_command_logs_user_id ON command_logs(user_id);
CREATE INDEX idx_command_logs_guild_id ON command_logs(guild_id);
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