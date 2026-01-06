# AGENTS.md

Development guidelines for agentic coding assistants working on VRCPulse.

## Project Overview

VRCPulse is a Discord bot monitoring VRChat server status with real-time visualized dashboards.

- **Language**: Rust (Edition 2024)
- **Framework**: Serenity (Discord), SeaORM (Database), Plotters (Visualization)
- **Database**: SQLite
- **Runtime**: Tokio (async)

## Build, Test, and Lint Commands

### Essential Commands

```bash
# Build
cargo build                    # Development build
cargo build --release          # Optimized production build

# Run
cargo run                      # Start the bot
cargo run --example chart_test # Test chart generation

# Code Quality (Run ALL three before committing)
cargo fmt                      # Format code
cargo clippy                   # Lint code
cargo check                    # Fast type check

# Testing
cargo test                     # Run all tests
cargo test test_name           # Run specific test by name
cargo test -- --nocapture      # Show println! output during tests

# Database
sea-orm-cli migrate up         # Run migrations
sea-orm-cli generate entity    # Generate entity models from schema
```

### Running a Single Test

```bash
# By test function name
cargo test test_poll_status

# By module path
cargo test collector::status::tests

# With output
cargo test test_name -- --nocapture
```

## Code Style Guidelines

### Import Organization

**Order**: External crates first, then internal modules, alphabetically within each group.

```rust
// External dependencies (alphabetical)
use chrono::Utc;
use reqwest::Client;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection};
use serenity::all::{CommandInteraction, Context};
use tracing::{debug, info, warn};

// Internal modules (alphabetical)
use crate::entity::{status_logs, component_logs};
use crate::error::Result;
use super::client::fetch_json;
```

**Entity imports** use prelude:

```rust
use sea_orm::entity::prelude::*;
```

### Naming Conventions

- **Files/Modules**: `snake_case` (e.g., `status_logs.rs`, `data_collector.rs`)
- **Structs/Enums**: `PascalCase` (e.g., `AppState`, `AppError`)
- **Functions/Variables**: `snake_case` (e.g., `poll_status()`, `db_connection`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `MIN_INTERVAL`, `API_URL`)
- **Database Tables**: `snake_case` (e.g., `status_logs`, `metric_logs`)

### Formatting

- **Indentation**: 4 spaces (Rust standard)
- **Line Length**: 100 characters (rustfmt default)
- **Always run**: `cargo fmt` before committing

### Type Safety

**NEVER**:
- Use `unwrap()` in production code (use `?` or `expect()` with context)
- Use `unsafe` blocks without strong justification
- Manually edit `Cargo.toml` for dependencies (use `cargo add`)

**ALWAYS**:
- Use proper error types (avoid `Box<dyn Error>`)
- Prefer `Result<T>` over `Option<T>` for fallible operations
- Use generics/traits over dynamic dispatch when possible

### Error Handling

**Use custom error types with `thiserror`**:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to load config: {0}")]
    Config(#[from] envy::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

pub type Result<T> = std::result::Result<T, AppError>;
```

**Early returns for errors**:

```rust
async fn poll(client: &Client, db: &DatabaseConnection) -> Result<()> {
    let response = match fetch_json(client, &url).await {
        Ok(r) => r,
        Err(e) => {
            warn!("API fetch failed: {}", e);
            return Err(e);
        }
    };
    
    // Continue with success path
    Ok(())
}
```

**Structured logging with context**:

```rust
use tracing::{debug, info, warn, error};

info!(
    poller = name,
    interval_secs = duration.as_secs(),
    "Polling interval updated"
);

error!(error = %e, "Failed to generate dashboard");
```

## Common Patterns

### Database Operations

**Deduplication check before insert**:

```rust
let existing = entity::Entity::find()
    .filter(Column::UniqueField.eq(value))
    .one(db)
    .await?;

if existing.is_none() {
    let record = entity::ActiveModel {
        field: Set(value),
        created_at: Set(Utc::now()),
        ..Default::default()
    };
    record.insert(db).await?;
}
```

**Upsert pattern**:

```rust
let existing = entity::Entity::find_by_id(&id).one(db).await?;

match existing {
    Some(existing) => {
        if needs_update {
            let mut active: entity::ActiveModel = existing.into();
            active.field = Set(new_value);
            active.update(db).await?;
        }
    }
    None => {
        let active = entity::ActiveModel {
            field: Set(value),
            ..Default::default()
        };
        active.insert(db).await?;
    }
}
```

### Async Patterns

**Tokio runtime and async traits**:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // ...
}

// Serenity async trait
#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        // ...
    }
}
```

**Parallel tasks with tokio**:

```rust
// Wait for all tasks
tokio::join!(
    poll_status(&client, &db),
    poll_metrics(&client, &db),
);

// Background tasks
tokio::spawn(async move {
    collector::start(client, db).await
});
```

### State Management (Serenity)

**Store in TypeMap**:

```rust
{
    let mut data = client.data.write().await;
    data.insert::<AppStateKey>(app_state);
}
```

**Retrieve from TypeMap**:

```rust
let data = ctx.data.read().await;
let state = data.get::<AppStateKey>().expect("AppState not found");
let state = state.read().await;
let db = state.database.as_ref();
```

## Project Structure

```
src/
├── main.rs              # Entry point, Discord client setup
├── config.rs            # Environment configuration
├── error.rs             # Error types
├── logging.rs           # Tracing initialization
├── state.rs             # Application state
├── collector/           # Data collection from APIs
│   ├── mod.rs          # Orchestration
│   ├── client.rs       # HTTP utilities
│   ├── status.rs       # VRChat status polling
│   ├── incident.rs     # Incident tracking
│   └── metrics.rs      # CloudFront metrics
├── commands/            # Discord slash commands
│   ├── mod.rs          # Command registration
│   ├── admin/          # Admin commands
│   └── status/         # Status commands
├── entity/              # SeaORM generated entities (DO NOT EDIT)
└── visualization/       # Chart generation
    ├── dashboard.rs    # Dashboard rendering
    ├── query.rs        # Database queries
    └── theme.rs        # Color schemes

migration/               # Database migrations
docs/                    # Technical documentation
```

## Development Workflow

### Before Committing

**Run ALL three** (in order):

```bash
cargo fmt      # Format code
cargo clippy   # Lint and fix warnings
cargo check    # Verify compilation
```

### Commit Convention

Use **Conventional Commits** format:

```
feat(collector): add CloudFront metrics polling
fix(status): correct timestamp deduplication logic
docs(readme): update installation instructions
chore(deps): add chrono dependency
```

**Rules**:
- Use lowercase type (feat, fix, docs, chore, refactor, test, style)
- Include scope in parentheses: `feat(module):`
- No co-authoring messages
- Keep first line under 72 characters

### Adding Dependencies

**ALWAYS use `cargo add`**, never manually edit `Cargo.toml`:

```bash
cargo add serde --features derive
cargo add tokio --features full
```

## Testing Guidelines

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let result = function_to_test();
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await.unwrap();
        assert!(result.is_valid());
    }
}
```

### Test Principles

- Test **behavior**, not implementation
- One assertion per test when possible
- Clear test names describing the scenario
- Tests must be deterministic
- **NEVER disable tests** - fix them instead

## Architecture Principles

### Core Principles

- **Separation of concerns**: Each module has a single responsibility
- **Type safety**: Leverage Rust's type system for correctness
- **Async-first**: All I/O operations are async (Tokio)
- **Database-driven config**: Store configuration in database for hot-reload
- **Defensive programming**: Early returns, explicit errors, deduplication

### When Stuck (After 3 Attempts)

**CRITICAL**: Maximum 3 attempts per issue, then STOP and reassess.

1. Document what failed (error messages, what you tried)
2. Research alternatives (find 2-3 similar implementations)
3. Question fundamentals (is this the right approach?)
4. Try a different angle (different library, simpler pattern)

## Environment Configuration

```bash
# .env file (create from .env.example)
DISCORD_TOKEN=your_discord_bot_token_here
TEST_GUILD_ID=123456789012345678  # Optional, for development
DATABASE_URL=sqlite://data/vrcpulse.db?mode=rwc
RUST_LOG=info,vrc_pulse=debug
```

## Documentation

### Documentation Structure

```
docs/
├── AGENTS.md                 # Documentation index & principles
├── commands/                 # Discord command specifications
│   └── AGENTS.md             # Command docs guide
├── system/                   # System architecture
│   └── AGENTS.md             # System docs guide
└── alerts/                   # Alert system specifications
    └── AGENTS.md             # Alert docs guide
```

**Before writing docs, read the relevant AGENTS.md guide first.**

### Documentation Principles

#### 1. Reference, Don't Duplicate (Most Important)

**Docs should reference code locations, not copy code.**

```markdown
## Do This:
- **Handler**: `src/commands/status/dashboard.rs:21-148`
- **Query**: `src/visualization/query.rs:15-67`

## Don't Do This:
(copying 100+ lines of code that will become outdated)
```

#### 2. No Emojis Allowed

**CRITICAL: No emojis in documentation or code comments.**

| Instead of | Use |
|------------|-----|
| Checkmark emoji | `[x]` or `[o]` |
| X mark emoji | `[ ]` |
| Warning emoji | `**WARNING**:` or `> **Note**:` |
| Status emojis | `[DONE]`, `[PENDING]`, `[PLANNED]` |

**Exceptions**:
- `README.md` files (user-facing) may use emojis for better presentation
- **Discord response examples**: When documenting actual Discord bot responses that contain emojis (e.g., status indicators), keep the emojis as they represent the real output

#### 3. Implementation First

- Documentation must reflect **current implementation**
- If docs and code differ, docs are wrong
- Mark unimplemented features with `[PLANNED]` or `[NOT IMPLEMENTED]`

#### 4. File References with Line Numbers

Always include file paths with line numbers:

```markdown
### Implementation
- **File**: `src/collector/status.rs:12-71`
- **Entity**: `src/entity/status_logs.rs:6-18`
```

### Documentation Links

- **Index**: `docs/AGENTS.md`
- **Commands**: `docs/commands/AGENTS.md`
- **System**: `docs/system/AGENTS.md`
- **Alerts**: `docs/alerts/AGENTS.md`

## Quick Reference

```bash
# Initial setup
cp .env.example .env
sea-orm-cli migrate up
cargo run

# Pre-commit checks
cargo fmt && cargo clippy && cargo check

# Add dependency
cargo add <crate-name>

# Run example
cargo run --example chart_test
```
