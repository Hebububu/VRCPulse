# /status

Display VRChat status dashboard with real-time metrics visualization.

---

## Status

> **[IMPLEMENTED]**: This command is fully functional.

---

## Usage

```
/status
```

**No parameters required**

---

## Response

The command responds with an embedded message containing:

### Dashboard Image (PNG attachment)

A multi-chart visualization showing:
1. **Online Users** - Concurrent user count over time
2. **API Latency** - Response time in milliseconds
3. **API Requests** - Request volume
4. **API Error Rate** - Percentage of failed requests
5. **Steam Auth Success Rate** - Authentication success percentage
6. **Meta Auth Success Rate** - Oculus/Meta authentication success percentage

Data is visualized for the **last 12 hours** with 5-minute downsampling.

### Embed Fields

| Field | Description | Format |
| :--- | :--- | :--- |
| **System Status** | Overall VRChat status | `{emoji} {description}` |
| **Online Users** | Average and peak concurrent users | `{avg}k (avg) / {max}k (max)` or raw numbers if < 1000 |
| **API Error Rate** | Average API error percentage | `{rate}%` (4 decimal places) |
| **Steam Auth** | Steam authentication success rate | `{rate}%` (1 decimal place) |
| **Meta Auth** | Meta/Oculus authentication success rate | `{rate}%` (1 decimal place) |
| **Component Groups** | Status of individual VRChat services | Two groups (see below) |

### Component Groups

Components are organized into two groups:

**API / Website** (group ID: `64b3rr3cxgk5`)
- Authentication / Login
- Social / Friends List
- SDK Asset Uploads
- Realtime Player State Changes

**Realtime Networking** (group ID: `t1jm7fqqq43h`)
- USA, West (San José)
- USA, East (Washington D.C.)
- Europe (Amsterdam)
- Japan (Tokyo)

### Status Indicators

| Status | Emoji |
| :--- | :--- |
| `operational` | 🟢 |
| `degraded_performance` | 🟡 |
| `partial_outage` | 🟠 |
| `major_outage` | 🔴 |
| `under_maintenance` | 🔵 |
| Unknown | ⚪ |

### Embed Colors

| Status Indicator | Color | Hex |
| :--- | :--- | :--- |
| `none` (Operational) | Green | `0x57f287` |
| `minor` | Yellow | `0xfee75c` |
| `major` | Orange | `0xf0b132` |
| `critical` | Red | `0xed4245` |
| Unknown | Blue | `0x00b0f4` |

---

## Error Handling

| Situation | Response |
| :--- | :--- |
| Dashboard generation fails | Red embed: "Failed to generate dashboard. Please try again later." |

The command uses deferred responses (`interaction.defer()`) to handle the time required for chart generation.

---

## Implementation

### Source Files

| Component | File | Lines |
|-----------|------|-------|
| Command definition | `src/commands/status/dashboard.rs` | 15-18 |
| Handler logic | `src/commands/status/dashboard.rs` | 21-148 |
| Status color mapping | `src/commands/status/dashboard.rs` | 61-73 |
| Component group constants | `src/commands/status/dashboard.rs` | 150-167 |
| Component formatting | `src/commands/status/dashboard.rs` | 169-232 |
| Dashboard generation | `src/visualization/dashboard.rs` | 1-246 |
| Data queries | `src/visualization/query.rs` | 1-130 |
| Theme constants | `src/visualization/theme.rs` | 1-34 |

### Data Sources

| Data | Table | Entity |
|------|-------|--------|
| System Status | `status_logs` | `src/entity/status_logs.rs:1-22` |
| Component Status | `component_logs` | `src/entity/component_logs.rs:1-21` |
| Metrics | `metric_logs` | `src/entity/metric_logs.rs:1-25` |

---

## Related Documents

- `docs/system/visualization-engine.md` - Chart rendering details
- `docs/system/data-collector.md` - How data is collected
- `docs/system/database-schema.md` - Table definitions
