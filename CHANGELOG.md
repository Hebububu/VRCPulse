# Changelog

## 2.7.0 (2026-04-04)

### Added

- Component status grid with hierarchical grouping (API/Website, Realtime Networking)
- Individual component status cards with 90-bucket history bars
- Component history API endpoint with bucketed status data (`/api/components`)
- i18n support for component status labels (EN, KO, JP)
- ARIA attributes and keyboard accessibility for status grid

### Fixed

- Tauri desktop app now uses `localhost` API in dev mode instead of production URL

## 2.6.0 (2026-04-04)

### Refactored

- Reorganized bot into domain-based architecture (`registration/`, `reporting/`, `onboarding/`, `status/`, `alerting/`, `discord/`, `infrastructure/`, `admin/`)
- Extracted `VrcPulseService` and shared database factory into `vrcpulse-core` for reuse between bot and server
- Moved locale files and shared utilities under their respective domain modules

### Fixed

- Bot command logic bugs in registration, reports, and button handling

### Added

- Architecture diagrams (backend/frontend flow and sequence diagrams)
- Dedicated registration repository with comprehensive data access
- Enhanced threshold alerting with expanded test coverage
- `crates.md` documenting the workspace structure

### Removed

- Legacy `src/` bot code (fully replaced by `crates/vrcpulse-bot/`)
- All `docs/` documentation (AGENTS.md, command docs, system docs, alert policy docs)
- Unused test scripts and dev helper scripts

## 2.5.0

- Previous release
