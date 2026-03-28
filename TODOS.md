# TODOS

## Migration Ownership
**What:** Each binary (bot, server, desktop) must run `Migrator::up()` on startup against its local SQLite database.
**Why:** Entities live in `vrcpulse-core` but each deployment has its own SQLite file. Without explicit migration-on-startup, schema drift between deployments is possible.
**Context:** The migration crate is shared at workspace root. Core exports a `run_migrations(db: &DatabaseConnection)` helper. Each binary calls it before starting the collector or serving requests.
**Depends on:** Phase 1 (core extraction)

## Axum Query Timeout
**What:** Add explicit query timeout for SQLite reads in the Axum web server.
**Why:** Time-series chart queries could degrade with large datasets. No timeout means a slow query blocks the Axum handler indefinitely.
**Context:** Use `sea_orm::ConnectOptions::sqlx_slow_statements_logging_threshold` for monitoring. Consider adding a 5s timeout on chart data queries. Low risk at current data volume but prevents future surprises.
**Depends on:** Phase 3 (Axum server)
