# Command Documentation Guide

Guidelines for writing and maintaining Discord slash command documentation.

## Purpose

This document defines principles for maintaining consistency in command documentation including usage, parameters, responses, and implementation references.

---

## Document Structure

### File Naming Convention

```
docs/commands/
├── AGENTS.md                 # This guide
├── status.md                 # /status command
├── config.md                 # /config command [NOT IMPLEMENTED]
├── report.md                 # /report command [NOT IMPLEMENTED]
└── admin/                    # Admin-only commands
    └── config.md             # /admin config command [DISABLED]
```

**Naming Rules:**
- **Use kebab-case**: `user-report.md` [o]
- **Command-first naming**: Match the slash command name
- **Group admin commands**: Place in `admin/` subdirectory
- **One file per command**: Don't combine multiple commands

### Document Layout

```markdown
# /{command-name}

Brief description of what the command does.

---

## Status

> **[STATUS]**: Implementation status note if needed.

---

## Usage

```
/command <required_param> [optional_param]
```

## Parameters

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `param_name` | Type | Yes/No | Description |

---

## Behavior Flow

### 1. Step Name
Description with code reference.

```rust
// Pseudocode or reference
```

### 2. Next Step
...

---

## Response

### Success Response

```
Embed title
Embed description
Field: Value
```

### Error Responses

| Situation | Response |
| :--- | :--- |
| Error case | Error message |

---

## Implementation

### Source Files

| Component | File | Lines |
|-----------|------|-------|
| Command definition | `src/commands/xyz.rs` [PLANNED] | 12-45 |
| Handler | `src/commands/xyz.rs` [PLANNED] | 47-120 |

---

## Related Documents

- Link to related docs
```

---

## Required Sections

### 1. Usage Section

Show exact command syntax:

```markdown
## Usage

```
/report <incident_type> [details]
```
```

- Use `<param>` for required parameters
- Use `[param]` for optional parameters
- Show actual parameter names from code

### 2. Parameters Table

```markdown
## Parameters

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `incident_type` | Choice | Yes | Type of issue being reported |
| `details` | String | No | Additional context (max 500 chars) |
```

For Choice types, list options:

```markdown
### Incident Types (Choices)

| Value | Display Name |
| :--- | :--- |
| `login` | Login Issues |
| `api` | API/Website Issues |
```

### 3. Behavior Flow

Document the command's logic flow with code references:

```markdown
## Behavior Flow

### 1. Load Status Data

Fetch current status from database.

- **Implementation**: `src/commands/status/dashboard.rs:75-95`

### 2. Generate Dashboard

Create visualization PNG.

- **Generator**: `src/visualization/dashboard.rs:18-246`
```

### 4. Response Section

Document both success and error responses:

```markdown
## Response

### Success Response

```
[Title] Report Submitted
[Description] Thank you for reporting Login Issues.
[Color] Green (0x57f287)
[Footer] If multiple users report this issue, an alert will be sent.
```

### Error Responses

| Situation | Title | Description |
| :--- | :--- | :--- |
| Guild not configured | Configuration Required | An administrator must run `/config setup` first. |
| Duplicate report | Report Cooldown | You already reported this issue recently. |
```

### 5. Implementation References

**Reference code locations, don't copy code:**

```markdown
## Implementation

### Source Files

| Component | File | Lines |
|-----------|------|-------|
| Command definition | `src/commands/status/dashboard.rs` | 15-18 |
| Handler logic | `src/commands/status/dashboard.rs` | 21-148 |
| Component formatting | `src/commands/status/dashboard.rs` | 169-232 |

### Database Tables

- `status_logs`: `src/entity/status_logs.rs:1-22`
- `component_logs`: `src/entity/component_logs.rs:1-21`
```

---

## Writing Principles

### 1. Reference, Don't Duplicate

**Docs should reference code locations, not copy code.**

**Do This:**
```markdown
### Command Registration
- **Definition**: `src/commands/status/dashboard.rs:15-18`
- **Handler**: `src/commands/status/dashboard.rs:21-148`
```

**Don't Do This:**
```markdown
### Command Registration
(copying 30+ lines of Rust code that will become outdated)
```

### 2. Implementation First

- Documentation must reflect **current implementation**
- If command behavior changes, update docs immediately
- Mark unimplemented commands with `[NOT IMPLEMENTED]`

### 3. No Emojis

Use text markers for status and indicators:

```markdown
## Status

> **[IMPLEMENTED]**: This command is fully functional.

> **[NOT IMPLEMENTED]**: This command is documented but not yet coded.

> **[DISABLED]**: This command exists but is disabled in registration.
```

**Exception**: Discord response examples may include emojis when they represent actual bot output (e.g., status indicators like `🟢 All Systems Operational`).

### 4. Document Error Cases

Every command should document:
- All possible error responses
- Error message text (must match code)
- HTTP-like status concept (success/failure)

---

## Validation Checklist

Check after writing/modifying command documentation:

### Accuracy
- [ ] Command syntax matches actual implementation
- [ ] Parameter names match code exactly
- [ ] Choice values match code exactly
- [ ] Error messages match code exactly
- [ ] File:line references are accurate

### Completeness
- [ ] All parameters documented
- [ ] All error cases documented
- [ ] Implementation references included
- [ ] Related documents linked

### Consistency
- [ ] File name matches command name (kebab-case)
- [ ] Same section structure as other command docs
- [ ] No emojis (use [x], [ ], [o], [STATUS])
- [ ] Tables use consistent formatting

---

## Source Files to Verify Against

| Documentation Topic | Verify Against |
|--------------------|----------------|
| Command definition | `src/commands/{cmd}.rs` [PLANNED] - `register()` function |
| Handler logic | `src/commands/{cmd}.rs` [PLANNED] - `run()` function |
| Subcommands | `src/commands/{cmd}/mod.rs` [PLANNED] |
| Admin commands | `src/commands/admin/*.rs` [PLANNED] |
| Registration | `src/commands/mod.rs` |
| Event handler | `src/main.rs` - `interaction_create()` |

---

## Common Mistakes and Prevention

| Mistake | Prevention |
|---------|------------|
| Wrong parameter syntax | Check `register()` function in source |
| Missing error cases | Check all `respond_error()` calls in handler |
| Outdated choice values | Verify against `add_string_choice()` calls |
| Copy-pasting code | Use `file:line` references instead |
| Using emojis | Use `[x]`, `[o]`, `[STATUS]` markers |

---

## Anti-Patterns

### NEVER
- Copy command handler code into docs
- Use emojis for status indicators
- Document planned parameters as implemented
- Skip error response documentation

### ALWAYS
- Reference code with `file:line` format
- Document all error cases
- Update docs when command changes
- Use consistent table formatting

---

## Related Documents

- `docs/AGENTS.md` - Documentation index
- `docs/system/AGENTS.md` - System documentation guide
- `docs/alerts/AGENTS.md` - Alert documentation guide
- `AGENTS.md` (root) - Project-wide conventions
