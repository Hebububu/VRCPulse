# Documentation Guide

Guidelines for writing and maintaining project documentation.

## Purpose

This document provides an index of all documentation areas and defines general principles for documentation in this project.

---

## Documentation Structure

```
docs/
├── AGENTS.md                 # This guide (index)
├── commands/                 # Discord command specifications
│   ├── AGENTS.md             # Command docs guide
│   ├── status.md             # /status command
│   ├── config.md             # /config command [NOT IMPLEMENTED]
│   ├── report.md             # /report command [NOT IMPLEMENTED]
│   └── admin/                # Admin commands
│       └── config.md         # /admin config command [DISABLED]
├── system/                   # System architecture
│   ├── AGENTS.md             # System docs guide
│   ├── data-collector.md     # Data collection service
│   ├── database-schema.md    # Database schema
│   └── visualization-engine.md
├── alerts/                   # Alert system specifications
│   ├── AGENTS.md             # Alert docs guide
│   ├── policy-user-threshold.md   # [NOT IMPLEMENTED]
│   ├── policy-vrchat-status.md    # [NOT IMPLEMENTED]
│   └── policy-cloudfront.md       # [NOT IMPLEMENTED]
└── README.md                 # Docs index for users
```

---

## Specialized Guides

Each documentation area has its own AGENTS.md with detailed conventions:

| Area | Guide | Purpose |
|------|-------|---------|
| **Commands** | `docs/commands/AGENTS.md` | Discord slash command specifications |
| **System** | `docs/system/AGENTS.md` | Architecture, services, database design |
| **Alerts** | `docs/alerts/AGENTS.md` | Alert policies, thresholds, notifications |

**When to consult**:
- Before writing command docs -> Read `docs/commands/AGENTS.md`
- Before writing system docs -> Read `docs/system/AGENTS.md`
- Before writing alert docs -> Read `docs/alerts/AGENTS.md`

---

## Core Principles

### 1. Reference, Don't Duplicate (Most Important)

**Docs should reference code locations, not copy code.**

- Code snippets in docs become stale when implementation changes
- File references with line numbers stay accurate and are easy to verify
- When code changes, only update line numbers (not entire code blocks)

**Do This:**
```markdown
### Implementation
- **Handler**: `src/commands/status/dashboard.rs:21-148`
- **Visualization**: `src/visualization/dashboard.rs:45-120`
- **Database query**: `src/visualization/query.rs:15-67`
```

**Don't Do This:**
```markdown
### Implementation
(copying 100+ lines of Rust code that will become outdated)
```

### 2. Implementation First

- Documentation must reflect **current implementation**
- If docs and code differ, docs are wrong
- Update docs when implementation changes
- Mark unimplemented features with `[PLANNED]` or `[NOT IMPLEMENTED]`

### 3. No Emojis

**CRITICAL: No emojis allowed in documentation.**

Use text markers instead:

| Instead of | Use |
|------------|-----|
| Checkmark emoji | `[x]` or `[o]` |
| X mark emoji | `[ ]` or `[x]` (for "don't do") |
| Warning emoji | `**WARNING**:` or `> **Note**:` |
| Status emojis | `[DONE]`, `[PENDING]`, `[PLANNED]` |

**Exceptions**:
- `README.md` files (user-facing) may use emojis for better presentation
- **Discord response examples**: When documenting actual Discord bot responses that contain emojis (e.g., status indicators like green/red circles), keep the emojis as they represent the real output

**Example:**
```markdown
## Implementation Status
| Feature | Status |
|---------|--------|
| Status polling | [x] Complete |
| Alert service | [ ] Not started |
| User reports | [o] In progress |
```

### 4. File Naming Convention

- **Use kebab-case**: `data-collector.md` [o]
- **Be descriptive**: Name should indicate the topic
- **Avoid status words**: No "fix", "todo", "wip" in filenames
- **Use feature names**: Not implementation details

### 5. Include File References

Always include file paths with line numbers:

```markdown
### Implementation
- **File**: `src/collector/status.rs:12-71`
- **Entity**: `src/entity/status_logs.rs:6-18`
- **Migration**: `migration/src/m20260103_001_create_table.rs:61-74`
```

### 6. Language Convention

- **All content**: English
- **Code/field names**: English
- **AGENTS.md files**: English

---

## Documentation Types

| Type | Location | Purpose |
|------|----------|---------|
| Command Specs | `docs/commands/` | Discord slash command documentation |
| System Docs | `docs/system/` | Architecture, services, data flow |
| Alert Policies | `docs/alerts/` | Alert triggers, thresholds, messages |

---

## Source Files to Verify Against

When writing or updating documentation, verify against these source files:

| Documentation Topic | Verify Against |
|--------------------|----------------|
| Command specifications | `src/commands/**/*.rs` [PLANNED] |
| Data collector | `src/collector/*.rs` [PLANNED] |
| Database schema | `migration/src/*.rs`, `src/entity/*.rs` [PLANNED] |
| Visualization | `src/visualization/*.rs` [PLANNED] |
| Alert service | `src/alerts/*.rs` (when implemented) [PLANNED] |
| Configuration | `src/config.rs`, `.env.example` |

---

## Validation Checklist

Check after writing/modifying documentation:

### Accuracy
- [ ] All `file:line` references are accurate and verified
- [ ] Response examples match actual implementation
- [ ] Feature descriptions match current behavior
- [ ] No emojis in the document

### Completeness
- [ ] Source files section with accurate line numbers
- [ ] Error handling documented
- [ ] Related documents section included
- [ ] Implementation status clearly marked

### Consistency
- [ ] File names use kebab-case
- [ ] Same section structure as other docs in category
- [ ] Cross-references use correct file names
- [ ] Status markers use `[x]`, `[ ]`, `[o]` not emojis

### Maintenance
- [ ] Remove planning sections when implementation is complete
- [ ] Update line numbers after code changes
- [ ] Verify mermaid diagrams render correctly

---

## Common Mistakes and Prevention

| Mistake | Prevention |
|---------|------------|
| Copying code blocks into docs | Use `file:line` references only |
| Using emojis for status | Use `[x]`, `[ ]`, `[o]`, `[DONE]`, `[PLANNED]` |
| Outdated line numbers | Re-verify after any code changes |
| Keeping completed plan docs | Convert to implementation docs or delete |
| Inconsistent file naming | Always use kebab-case |

---

## Anti-Patterns

### NEVER
- Use emojis in documentation
- Copy entire code blocks into docs (reference instead)
- Document planned features as implemented
- Use filenames with status words (fix, todo, wip)
- Leave outdated line numbers

### ALWAYS
- Reference code with `file:line` format
- Mark implementation status clearly
- Update docs when code changes
- Use validation checklist before committing
- Keep documentation concise and scannable

---

## Related Documents

- `AGENTS.md` (root) - Project-wide development guidelines
- `docs/commands/AGENTS.md` - Command documentation guide
- `docs/system/AGENTS.md` - System documentation guide
- `docs/alerts/AGENTS.md` - Alert documentation guide
