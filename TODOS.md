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

## Desktop Self-Contained AI Insight
**What:** 데스크톱 앱이 자체 SQLite DB + collector를 가지고, 유저의 Google API key로 직접 Gemini를 호출하는 기능.
**Why:** 현재 데스크톱은 웹 API의 insight를 읽기 전용으로 표시만 함. 유저가 자신의 API key로 독립적인 분석을 받을 수 있으면 웹 서버 없이도 동작 가능.
**Context:** Settings 페이지에 Google AI Studio API Key 입력 필드 추가. tauri-plugin-stronghold로 키 암호화 저장 (tauri-plugin-store는 평문 저장이라 부적합). /api/insights/features 엔드포인트를 서버에 추가해서 데스크톱이 feature snapshot을 받아온 뒤 자체 Gemini 호출 가능. 또는 데스크톱에 자체 collector를 넣어서 완전 독립 동작.
**Depends on:** AI Insight Engine v1 (웹 서버 구현)

## Mobile QA Test Automation
**What:** Playwright 또는 Vitest + @testing-library/svelte로 모바일 viewport 테스트 자동화.
**Why:** 현재 프론트엔드 테스트 프레임워크가 없어서 모든 UI 검증이 수동 /qa 실행에 의존. 모바일 미디어 쿼리 변경 시 회귀 방지 필요.
**Context:** 390x844 viewport에서 주요 동선(Dashboard→IncidentList→IncidentDetail) 테스트. StatusBar 2줄 wrap, InsightCard 최상단 배치, 차트 높이, 터치 타겟 44px 검증. Playwright가 적합 (다중 viewport + 스크린샷 비교).
**Depends on:** 없음
