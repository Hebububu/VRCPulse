# CloudFront Metrics Alert

Alert triggered when CloudFront performance metrics exceed thresholds.

---

## Status

> **[NOT IMPLEMENTED]**: This alert policy is planned but not yet implemented.

---

## Overview

When enabled, this alert monitors CloudFront performance metrics (API latency, error rates, auth success rates). When metrics cross configured thresholds, an alert is sent to configured guilds.

---

## Trigger Conditions

### Metric Thresholds

| Metric | Alert When | Default Threshold |
|--------|------------|-------------------|
| `api_latency` | Above threshold | TBD |
| `api_errors` | Above threshold | TBD |
| `extauth_steam` | Below threshold | TBD |
| `extauth_oculus` | Below threshold | TBD |

### Evaluation Window

- Check interval: 60 seconds (matches metrics polling)
- Evaluation period: 5 minutes (average of recent data points)

---

## Deduplication

### Strategy
Per metric_type + hourly time window

### Reference ID Format
```
cloudfront_{metric_name}_{hour_window}
```

**Examples:**
- `cloudfront_api_errors_2026-01-05T12`
- `cloudfront_extauth_steam_2026-01-05T13`

### Behavior
- Same metric can only alert once per hour per guild
- Different metrics alert independently
- Recovery alerts when metric returns to normal

---

## Alert Message

### Threshold Exceeded

```
[Title] CloudFront Metric Alert
[Description] **{metric_name}** has crossed threshold
[Color] Orange (0xf0b132) or Red (0xed4245) based on severity
[Fields]
  Metric: {metric_display_name} (inline)
  Current Value: {value} (inline)
  Threshold: {threshold} (inline)
  Trend: {direction} over last 5 minutes
[Footer] Data from CloudFront metrics
[Timestamp] Current time
```

### Recovery Alert

```
[Title] CloudFront Metric Recovered
[Description] **{metric_name}** has returned to normal
[Color] Green (0x57f287)
[Fields]
  Metric: {metric_display_name} (inline)
  Current Value: {value} (inline)
  Threshold: {threshold} (inline)
[Footer] Data from CloudFront metrics
[Timestamp] Current time
```

---

## Configuration

### Planned Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `alert.cloudfront.enabled` | false | Enable CloudFront alerts |
| `alert.cloudfront.latency_threshold` | TBD | API latency threshold (ms) |
| `alert.cloudfront.error_threshold` | TBD | Error rate threshold (%) |
| `alert.cloudfront.auth_threshold` | TBD | Auth success threshold (%) |

---

## Implementation

### Planned Source Files [NOT IMPLEMENTED]

> **Note**: These files will be created during alert system implementation.

| Component | Planned File |
|-----------|--------------|
| Alert service | `src/alerts/mod.rs` [PLANNED] |
| Metric threshold check | `src/alerts/cloudfront.rs` [PLANNED] |
| Notification builder | `src/alerts/notifications.rs` [PLANNED] |

### Data Sources

- `metric_logs`: CloudFront metric history

---

## Related Documents

- `docs/system/data-collector.md` - How metrics are collected
- `docs/system/database-schema.md` - Table definitions
- `docs/system/visualization-engine.md` - How metrics are visualized
- `docs/alerts/AGENTS.md` - Alert documentation guide
