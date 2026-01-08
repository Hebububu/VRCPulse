# 📚 Documentation

Welcome to the **VRCPulse** documentation hub.
Here you can find detailed technical specifications, architecture designs, and policy definitions.

## 📂 Documentation Index

### [🎮 Commands (User Interface)](./commands/README.md)
Specifications for Discord Slash Commands and user interactions.
- [**Config Command**](./commands/config.md) `[NOT IMPLEMENTED]` - Channel and reporting interval settings.
- [**Status Command**](./commands/status.md) - Immediate status checks and graph output.
- [**Report Command**](./commands/report.md) `[NOT IMPLEMENTED]` - User-driven incident reporting.

### [⚙️ System Design (Architecture & Data)](./system/README.md)
Backend logic, database schema, and core engine designs.
- [**Database Schema**](./system/database-schema.md) - SQLite table structures and Entity definitions.
- [**Data Collector**](./system/data-collector.md) - API polling strategies and scheduling.
- [**Visualization Engine**](./system/visualization-engine.md) - Plotters-based graph rendering logic.

### [🚨 Alert Policies](./alerts/README.md)
Business logic for incident detection and alert broadcasting.
- [**User Threshold Policy**](./alerts/policy-user-threshold.md) `[NOT IMPLEMENTED]` - Accumulation-based alerting conditions.
- [**VRChat Status Policy**](./alerts/policy-vrchat-status.md) `[NOT IMPLEMENTED]` - Handling official status API events.
- [**CloudFront Policy**](./alerts/policy-cloudfront.md) `[NOT IMPLEMENTED]` - Anomaly detection in CloudFront metrics.