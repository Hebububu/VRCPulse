# TODOS

## Migration Ownership _(done)_
**What:** Each binary (bot, server, desktop) must run `Migrator::up()` on startup against its local SQLite database.
**Why:** Entities live in `vrcpulse-core` but each deployment has its own SQLite file. Without explicit migration-on-startup, schema drift between deployments is possible.
**Status:** Both server (`main.rs`) and bot (`discord::setup()`) now call `Migrator::up()` on startup. Desktop has no local DB so N/A unless self-contained mode is added.

## Axum Query Timeout _(partially done)_
**What:** Add explicit query timeout for SQLite reads in the Axum web server.
**Why:** Time-series chart queries could degrade with large datasets. No timeout means a slow query blocks the Axum handler indefinitely.
**Status:** `acquire_timeout(10s)` and SQLite `PRAGMA busy_timeout=5000` are configured in `vrcpulse-core/src/database.rs`. Missing: `sqlx_slow_statements_logging_threshold` for monitoring and per-query timeout (5s) on chart data endpoints.
**Remaining:** Add slow query logging threshold and per-query timeout on chart endpoints.

## Desktop Self-Contained AI Insight
**What:** Desktop app with its own SQLite DB + collector, calling Gemini via user's Google API key.
**Why:** Currently desktop is a read-only UI shell that displays insights from the web API. Independent analysis without a web server would be valuable.
**Status:** Not started. Desktop has no local collector, no tauri-plugin-stronghold, no API key settings page.
**Depends on:** AI Insight Engine v1 (web server)

## Frontend Component Tests (Vitest)
**What:** Vitest + @testing-library/svelte component unit tests.
**Why:** 9+ frontend components/pages have no automated tests. InsightCard locale switching, MaintenanceBanner conditional render, filter behavior all rely on manual QA.
**Status:** Not started. No vitest config, no @testing-library/svelte dependency, no test files.

## Bot CI/CD Pipeline _(done)_
**What:** Add GitHub Actions workflow and update Dockerfile for `vrcpulse-bot` crate deployment.
**Why:** Root `Dockerfile` was outdated — referenced old single-crate `src/main.rs` layout.
**Status:** Done. `Dockerfile.bot` created, `docker-compose.prod.yml` manages both web + bot, `deploy.yml` updated for compose-based deploys with post-deploy verification. Old root `Dockerfile` deleted. Bot collector removed (server is single data collector).

## Mobile QA Test Automation
**What:** Playwright mobile viewport test automation (390x844).
**Why:** All UI verification depends on manual QA runs. Need regression prevention for mobile media query changes.
**Status:** Not started. No Playwright config or mobile viewport tests exist.

## RPM Package Panic Fix
**What:** Fix panic crash when launching the RPM-packaged desktop app.
**Why:** Likely caused by lack of native Wayland support in the current build configuration. The app panics on startup on Wayland-based Linux environments distributed via RPM.
**Status:** Not started. Need to investigate Wayland-related panic trace and apply appropriate fix (e.g., Wayland feature flags, fallback to XWayland).

## Migrate Frontend Dependencies to Bun
**What:** Replace npm with Bun as the frontend package manager and runtime.
**Why:** Bun offers significantly faster install and build times compared to npm. Simplifies the frontend toolchain.
**Status:** Not started. Currently using npm with `package-lock.json` in `desktop/`.
