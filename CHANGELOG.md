# Changelog

## 2.8.0 (2026-04-07)

### Added

- Docker Compose production stack (`docker-compose.prod.yml`) managing web server and Discord bot as a unified deployment
- `Dockerfile.bot` for workspace-aware multi-stage bot builds
- Post-deploy verification in GitHub Actions (checks both containers are running after deploy)
- Deploy workflow gated on test suite passing (`needs: test`)
- Bot now runs database migrations on startup (`Migrator::up()`)

### Changed

- Deploy workflow upgraded from bare `docker run` to `docker compose` for both services
- Bot no longer runs its own data collector; web server is the single source of truth for VRChat API polling
- Removed `collector_config` from bot's `AppState` (collector removed entirely)
- `migration` and `sea-orm-migration` moved from dev-dependencies to regular dependencies in bot crate

### Removed

- Stale root `Dockerfile` (pre-workspace layout, replaced by `Dockerfile.bot`)

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
