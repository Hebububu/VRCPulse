# Commands (User Interface)

This directory contains specifications for **Discord Slash Commands** and user interaction flows.

## Implemented Commands

*   **[status.md](./status.md)**: Real-time status lookup (`/status`)
    *   Current server status summary
    *   On-demand dashboard generation with metrics visualization
*   `/hello`: Simple greeting command (internal/test, not documented)

## Planned Commands

*   **[config.md](./config.md)**: Guild configuration (`/config`) `[NOT IMPLEMENTED]`
    *   Registering/Unregistering notification channels
    *   Adjusting report thresholds
*   **[report.md](./report.md)**: Outage reporting (`/report`) `[NOT IMPLEMENTED]`
    *   Incident type selection
    *   Submission and storage workflow

## Admin Commands

*   **[admin/config.md](./admin/config.md)**: Bot global configuration (`/admin config`) `[DISABLED]`
    *   Polling interval management
    *   Dynamic collector settings

## Source Files

| Component | File | Lines |
|-----------|------|-------|
| Command registration | `src/commands/mod.rs` | 1-34 |
| Status command | `src/commands/status/mod.rs` | 1-13 |
| Admin commands | `src/commands/admin/mod.rs` | 1-9 |
