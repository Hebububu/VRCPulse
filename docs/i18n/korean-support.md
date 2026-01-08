# Korean Language Support Implementation Plan

Implementation plan for adding Korean (ko) language support to VRCPulse.

---

## Overview

| Property | Value |
|----------|-------|
| **Target Languages** | English (en), Korean (ko) |
| **i18n Library** | `rust-i18n` |
| **Resolution Priority** | Auto-detect (Discord) > Guild preference > User preference > Default (en) |
| **Total Strings** | ~130 user-facing strings |
| **Admin Commands** | English only (not localized) |

---

## Architecture

### Language Resolution Flow

```
Discord Interaction
        |
        v
[1] Discord Locale (interaction.locale)
        |
        | (if available, use it)
        v
[2] Guild Config Language (guild_configs.language)
        |
        | (fallback for guild context)
        v
[3] User Config Language (user_configs.language)
        |
        | (fallback for DM context)
        v
[4] Default: "en"
```

### File Structure (Planned)

```
src/
├── i18n/
│   ├── mod.rs              # i18n initialization, locale resolver
│   └── keys.rs             # Translation key constants (optional)
├── commands/
│   └── config/
│       └── language.rs     # /config language handler (NEW)
└── visualization/
    └── fonts/              # Font loading (NEW)

locales/
├── en.json                 # English translations
└── ko.json                 # Korean translations

assets/
└── fonts/
    └── NotoSansKR-*.ttf    # Noto Sans KR font files
```

---

## Stages

### Stage 1: Setup rust-i18n Infrastructure

**Goal**: Establish translation infrastructure with rust-i18n crate

**Status**: [x] Complete

**Tasks**:
- [x] Add `rust-i18n` dependency to `Cargo.toml`
- [x] Create `locales/en.json` with all English strings
- [x] Create `locales/ko.json` with Korean translations
- [x] Create `src/i18n/mod.rs` with initialization and helper functions
- [x] Add i18n initialization to `main.rs`

**Files to Create**:
| File | Description |
|------|-------------|
| `locales/en.json` | English translation file |
| `locales/ko.json` | Korean translation file |
| `src/i18n/mod.rs` | i18n module with locale resolver |

**Success Criteria**:
- `cargo build` succeeds with rust-i18n
- Can call `t!("key")` and get translated string
- Locale can be set dynamically per-request

**Translation Key Structure**:
```json
{
  "commands": {
    "status": {
      "name": "status",
      "description": "View VRChat status dashboard"
    },
    "config": {
      "name": "config",
      "description": "Configure VRCPulse settings"
    }
  },
  "embeds": {
    "dashboard": {
      "title": "VRChat Status Dashboard",
      "system_status": "System Status",
      "online_users": "Online Users"
    }
  },
  "errors": {
    "unknown_command": "Unknown command",
    "dashboard_failed": "Failed to generate dashboard. Please try again later."
  }
}
```

---

### Stage 2: Database Migration + /config language Command

**Goal**: Add language preference storage and configuration command

**Status**: [x] Complete

**Tasks**:
- [x] Create migration `m20260108_001_add_language_column.rs`
- [x] Add `language` column to `guild_configs` table
- [x] Add `language` column to `user_configs` table
- [x] Update entities manually (language field added)
- [x] Add `update_language()` methods to repositories
- [x] Implement `/config language` subcommand
- [x] Add language embed builders

**Files to Create/Modify**:
| File | Action |
|------|--------|
| `migration/src/m20260108_001_add_language_column.rs` | CREATE |
| `migration/src/lib.rs` | MODIFY - register migration |
| `src/entity/guild_configs.rs` | REGENERATE |
| `src/entity/user_configs.rs` | REGENERATE |
| `src/repository/config.rs` | MODIFY - add language methods |
| `src/commands/config/mod.rs` | MODIFY - add subcommand |
| `src/commands/config/handlers.rs` | MODIFY - add handler |
| `src/commands/config/embeds.rs` | MODIFY - add embeds |

**Database Schema Change**:
```sql
-- guild_configs
ALTER TABLE guild_configs ADD COLUMN language TEXT DEFAULT NULL;

-- user_configs
ALTER TABLE user_configs ADD COLUMN language TEXT DEFAULT NULL;
```

**Command Structure**:
```
/config language [code]
  - code (optional): Language code (en, ko)
  - If omitted: shows current setting and available options
  - If provided: updates preference
```

**Success Criteria**:
- Migration runs successfully: `sea-orm-cli migrate up`
- `/config language` shows current setting
- `/config language ko` updates preference
- Language preference persists in database

---

### Stage 3: Localize Discord Slash Commands (Built-in)

**Goal**: Use Serenity's built-in localization for command names/descriptions

**Status**: [ ] Not started

**Tasks**:
- [ ] Add `.name_localized("ko", ...)` to all public commands
- [ ] Add `.description_localized("ko", ...)` to all public commands
- [ ] Add localized option names and descriptions
- [ ] Add localized choice names

**Files to Modify**:
| File | Lines (approx) | Content |
|------|----------------|---------|
| `src/commands/hello.rs` | 7-8 | `/hello` command |
| `src/commands/status/dashboard.rs` | 15-17 | `/status` command |
| `src/commands/config/mod.rs` | 25-55 | `/config` command + subcommands |
| `src/commands/report.rs` | 64-88 | `/report` command + options |

**Example Pattern**:
```rust
CreateCommand::new("status")
    .description("View VRChat status dashboard")
    .name_localized("ko", "상태")
    .description_localized("ko", "VRChat 상태 대시보드 보기")
```

**Commands to Localize**:
| Command | Korean Name | Korean Description |
|---------|-------------|-------------------|
| `/status` | `/상태` | VRChat 상태 대시보드 보기 |
| `/config` | `/설정` | VRCPulse 설정 구성 |
| `/config setup` | 설정 | 알림 채널 설정 |
| `/config show` | 보기 | 현재 설정 확인 |
| `/config unregister` | 해제 | 알림 비활성화 |
| `/config language` | 언어 | 언어 설정 |
| `/report` | `/신고` | 문제 신고하기 |
| `/hello` | `/인사` | 인사하기 |

**Success Criteria**:
- Discord shows Korean command names for Korean locale users
- Command descriptions appear in Korean
- Option names and choices appear in Korean

---

### Stage 4: Localize Command Responses

**Goal**: Translate all embed messages and responses

**Status**: [ ] Not started

**Tasks**:
- [ ] Create locale resolver function (get locale from interaction)
- [ ] Update all embed builders to accept locale parameter
- [ ] Replace hardcoded strings with `t!()` macro calls
- [ ] Update error response functions

**Files to Modify**:
| File | String Count (approx) |
|------|----------------------|
| `src/commands/status/dashboard.rs` | ~15 strings |
| `src/commands/config/embeds.rs` | ~40 strings |
| `src/commands/config/handlers.rs` | ~20 strings |
| `src/commands/report.rs` | ~25 strings |
| `src/commands/hello.rs` | ~2 strings |
| `src/commands/shared.rs` | ~5 strings |
| `src/main.rs` (guild join embed) | ~10 strings |

**Locale Resolver Pattern**:
```rust
// src/i18n/mod.rs
pub async fn resolve_locale(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> String {
    // 1. Discord locale (highest priority)
    let discord_locale = interaction.locale.as_str();
    if is_supported(discord_locale) {
        return discord_locale.to_string();
    }
    
    // 2. Guild preference (if in guild context)
    if let Some(guild_id) = interaction.guild_id {
        if let Some(lang) = get_guild_language(ctx, guild_id).await {
            return lang;
        }
    }
    
    // 3. User preference
    if let Some(lang) = get_user_language(ctx, interaction.user.id).await {
        return lang;
    }
    
    // 4. Default
    "en".to_string()
}
```

**Embed Builder Pattern**:
```rust
// Before
pub fn dashboard_title() -> String {
    "VRChat Status Dashboard".to_string()
}

// After
pub fn dashboard_title(locale: &str) -> String {
    t!("embeds.dashboard.title", locale = locale).to_string()
}
```

**Success Criteria**:
- All user-facing strings use translation keys
- Korean users see Korean responses
- English users see English responses
- Fallback works correctly

---

### Stage 5: Refactor Visualization Engine + Korean Font Support

**Goal**: Support Korean text rendering in charts with Noto Sans KR

**Status**: [ ] Not started

**Tasks**:
- [ ] Download and add Noto Sans KR font files to `assets/fonts/`
- [ ] Refactor font loading in visualization module
- [ ] Create font provider that selects font based on locale
- [ ] Update chart text rendering to use locale-aware fonts
- [ ] Translate chart labels

**Files to Create/Modify**:
| File | Action |
|------|--------|
| `assets/fonts/NotoSansKR-Regular.ttf` | ADD |
| `assets/fonts/NotoSansKR-Bold.ttf` | ADD |
| `src/visualization/fonts.rs` | CREATE - font loading module |
| `src/visualization/mod.rs` | MODIFY - export fonts module |
| `src/visualization/dashboard.rs` | MODIFY - use locale-aware rendering |

**Chart Labels to Translate**:
| English | Korean |
|---------|--------|
| Online Users | 온라인 사용자 |
| API Latency | API 지연시간 |
| API Requests | API 요청 |
| API Error Rate | API 오류율 |
| Steam Auth Success Rate | Steam 인증 성공률 |
| Meta Auth Success Rate | Meta 인증 성공률 |

**Font Loading Pattern**:
```rust
// src/visualization/fonts.rs
pub struct FontProvider {
    en_regular: FontFamily,
    en_bold: FontFamily,
    ko_regular: FontFamily,
    ko_bold: FontFamily,
}

impl FontProvider {
    pub fn get_font(&self, locale: &str, bold: bool) -> &FontFamily {
        match (locale, bold) {
            ("ko", true) => &self.ko_bold,
            ("ko", false) => &self.ko_regular,
            (_, true) => &self.en_bold,
            (_, false) => &self.en_regular,
        }
    }
}
```

**Success Criteria**:
- Charts render correctly with Korean text
- No missing glyphs or tofu characters
- Font files are bundled correctly
- Chart generation performance is acceptable

---

### Stage 6: Localize Alert Messages

**Goal**: Translate alert notification messages

**Status**: [ ] Not started

**Tasks**:
- [ ] Update alert embed builders to accept locale
- [ ] Translate incident type names
- [ ] Translate alert titles and descriptions
- [ ] Resolve locale for alert destination (guild/user)

**Files to Modify**:
| File | Content |
|------|---------|
| `src/alerts/threshold.rs` | Alert embeds, incident type names |

**Alert Strings to Translate**:
| English | Korean |
|---------|--------|
| High Report Volume Detected | 높은 신고량 감지됨 |
| Recent Reports | 최근 신고 |
| API Issues | API 문제 |
| Authentication Issues | 인증 문제 |
| Connection Issues | 연결 문제 |
| {n} minutes ago | {n}분 전 |
| just now | 방금 |

**Locale Resolution for Alerts**:
```rust
// For guild alerts: use guild_configs.language
// For DM alerts: use user_configs.language
// Fallback: "en"
```

**Success Criteria**:
- Alert messages appear in configured language
- Incident type names are translated
- Time formatting is localized

---

## Translation Reference

### Complete String List

**Commands (Discord built-in localization)**:
```json
{
  "commands.status.name": "상태",
  "commands.status.description": "VRChat 상태 대시보드 보기",
  "commands.config.name": "설정",
  "commands.config.description": "VRCPulse 설정 구성",
  "commands.config.setup.name": "설정",
  "commands.config.setup.description": "VRCPulse 알림 설정",
  "commands.config.show.name": "보기",
  "commands.config.show.description": "현재 설정 확인",
  "commands.config.unregister.name": "해제",
  "commands.config.unregister.description": "VRCPulse 알림 비활성화",
  "commands.config.language.name": "언어",
  "commands.config.language.description": "언어 설정",
  "commands.report.name": "신고",
  "commands.report.description": "VRChat 문제 신고",
  "commands.hello.name": "인사",
  "commands.hello.description": "인사하기"
}
```

**Dashboard Embed**:
```json
{
  "embeds.dashboard.title": "VRChat 상태 대시보드",
  "embeds.dashboard.system_status": "시스템 상태",
  "embeds.dashboard.online_users": "온라인 사용자",
  "embeds.dashboard.api_error_rate": "API 오류율",
  "embeds.dashboard.steam_auth": "Steam 인증",
  "embeds.dashboard.meta_auth": "Meta 인증",
  "embeds.dashboard.last_12_hours": "최근 12시간",
  "embeds.dashboard.components": "컴포넌트",
  "embeds.dashboard.no_data": "데이터 없음",
  "embeds.dashboard.api_website": "API / 웹사이트",
  "embeds.dashboard.realtime_networking": "실시간 네트워킹",
  "embeds.dashboard.unknown": "알 수 없음"
}
```

**Config Embeds**:
```json
{
  "embeds.config.setup_success_title": "설정 완료",
  "embeds.config.setup_success_description": "VRCPulse 알림이 활성화되었습니다",
  "embeds.config.show_title": "현재 설정",
  "embeds.config.unregister_confirm_title": "알림 해제 확인",
  "embeds.config.unregister_confirm_description": "정말로 VRCPulse 알림을 비활성화하시겠습니까?",
  "embeds.config.unregister_success_title": "알림 해제됨",
  "embeds.config.language_current": "현재 언어: {language}",
  "embeds.config.language_updated": "언어가 {language}(으)로 변경되었습니다"
}
```

**Buttons**:
```json
{
  "buttons.cancel": "취소",
  "buttons.confirm": "확인",
  "buttons.yes_unregister": "예, 해제합니다"
}
```

**Errors**:
```json
{
  "errors.unknown_command": "알 수 없는 명령어",
  "errors.dashboard_failed": "대시보드 생성에 실패했습니다. 나중에 다시 시도해주세요.",
  "errors.permission_denied": "권한이 없습니다",
  "errors.not_configured": "설정되지 않음"
}
```

**Charts**:
```json
{
  "charts.online_users": "온라인 사용자",
  "charts.api_latency": "API 지연시간",
  "charts.api_requests": "API 요청",
  "charts.api_error_rate": "API 오류율",
  "charts.steam_auth_rate": "Steam 인증 성공률",
  "charts.meta_auth_rate": "Meta 인증 성공률"
}
```

**Alerts**:
```json
{
  "alerts.high_volume_title": "높은 신고량 감지됨",
  "alerts.recent_reports": "최근 신고",
  "alerts.incident_types.api": "API 문제",
  "alerts.incident_types.auth": "인증 문제",
  "alerts.incident_types.connection": "연결 문제",
  "alerts.time.just_now": "방금",
  "alerts.time.minutes_ago": "{n}분 전",
  "alerts.time.hours_ago": "{n}시간 전"
}
```

---

## Testing Checklist

### Stage 1
- [ ] `rust-i18n` compiles and initializes
- [ ] `t!("key")` returns English by default
- [ ] `t!("key", locale = "ko")` returns Korean

### Stage 2
- [ ] Migration runs: `sea-orm-cli migrate up`
- [ ] `/config language` shows current setting
- [ ] `/config language ko` persists to database
- [ ] `/config language en` persists to database

### Stage 3
- [ ] Discord shows Korean command names for `ko` locale
- [ ] Discord shows English command names for `en` locale
- [ ] All option names are localized

### Stage 4
- [ ] `/status` returns Korean embed for Korean users
- [ ] `/config show` returns Korean embed for Korean users
- [ ] Error messages are localized
- [ ] Fallback chain works correctly

### Stage 5
- [ ] Charts render Korean text without tofu
- [ ] Font loading doesn't significantly impact performance
- [ ] All chart labels are translated

### Stage 6
- [ ] Alert embeds use guild language preference
- [ ] DM alerts use user language preference
- [ ] Incident type names are translated
- [ ] Time formatting is localized

---

## Dependencies

```toml
# Cargo.toml additions
[dependencies]
rust-i18n = "3"
```

---

## Source Files Reference

| Purpose | Current File | Lines |
|---------|--------------|-------|
| Command definitions | `src/commands/config/mod.rs` | 25-55 |
| Config handlers | `src/commands/config/handlers.rs` | 1-415 |
| Config embeds | `src/commands/config/embeds.rs` | 1-185 |
| Dashboard handler | `src/commands/status/dashboard.rs` | 1-240 |
| Report command | `src/commands/report.rs` | 1-533 |
| Alert threshold | `src/alerts/threshold.rs` | 1-433 |
| Visualization | `src/visualization/dashboard.rs` | 1-200 |
| Guild config entity | `src/entity/guild_configs.rs` | 1-25 |
| User config entity | `src/entity/user_configs.rs` | 1-20 |
| Config repository | `src/repository/config.rs` | 1-175 |
| Initial migration | `migration/src/m20260103_001_create_table.rs` | 1-492 |

---

## Related Documents

- `AGENTS.md` - Project development guidelines
- `docs/commands/AGENTS.md` - Command documentation guide
- `docs/system/visualization-engine.md` - Visualization system docs
