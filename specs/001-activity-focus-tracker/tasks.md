# 태스크: Activity Focus Tracker

**입력**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md), [contracts/ipc-commands.md](contracts/ipc-commands.md)
**사전 조건**: plan.md (필수), spec.md (필수), research.md, data-model.md, contracts/

> **⚠️ TDD 필수 (헌법 III 원칙)**: 테스트 태스크는 반드시 구현 태스크보다 **먼저** 완료해야 한다.
> Red(실패) → Green(구현) → Refactor 순서를 엄격히 준수한다.

## 포맷: `[ID] [P?] [Story?] 설명`

- **[P]**: 병렬 실행 가능 (다른 파일, 의존성 없음)
- **[Story]**: 해당 태스크가 속한 유저 스토리 (US1~US5)

---

## Phase 1: 프로젝트 설정

**목적**: Tauri v2 프로젝트 초기화 및 개발 환경 구성

- [x] T001 `cargo tauri init`으로 Tauri v2 프로젝트 초기화 및 헌법 I 기준 디렉토리 구조 생성 (`src-tauri/src/commands/`, `services/`, `models/`, `state/`, `src/components/`, `hooks/`, `services/`, `types/`, `stores/`, `__tests__/`)
- [x] T002 [P] `src-tauri/Cargo.toml`에 Rust 의존성 추가: `rusqlite 0.32 (bundled)`, `r2d2 0.8`, `r2d2_sqlite 0.25`, `refinery 0.8`, `objc2 0.5`, `objc2-foundation 0.2`, `objc2-app-kit 0.2`, `tauri-specta 2.0.0-rc.21`, `serde`, `uuid`, `chrono`, `thiserror`
- [x] T003 [P] `package.json`에 프론트엔드 의존성 추가: `vitest`, `@testing-library/react`, `@testing-library/user-event`, `zustand`, `@tauri-apps/api`; `vite.config.ts`에 Vitest 설정 추가
- [x] T004 [P] `src-tauri/tauri.conf.json` 설정: `LSUIElement` (Info.plist), `dropdown` 창 (`decorations: false`, `transparent: true`, `visible: false`), `dashboard` 창 설정
- [x] T005 [P] `capabilities/default.json` 권한 설정: Tauri v2 IPC 커맨드 허용 목록 등록

---

## Phase 2: 기반 인프라 (모든 유저 스토리 선행 필수)

**목적**: 모든 스토리가 공유하는 DB, 에러 타입, 모델, 앱 상태 구축

**⚠️ 체크포인트**: 이 Phase가 완료되어야 Phase 3+ 작업 시작 가능

- [x] T006 `src-tauri/migrations/V1__init.sql` 작성: `sessions`, `activities`, `classification_rules` (기본 규칙 포함), `settings` 테이블 스키마 및 초기 데이터
- [x] T007 [P] `src-tauri/migrations/V2__archive.sql` 작성: `archived_daily_summaries`, `references` 테이블 스키마
- [x] T008 [P] `src-tauri/src/errors.rs` 작성: `AppError` enum (`Database`, `PermissionDenied`, `SessionAlreadyActive`, `NoActiveSession`, `NotFound`, `Internal`) — `serde::Serialize`, `specta::Type` 구현
- [x] T009 [P] `src-tauri/src/models/` 전체 작성: `session.rs` (Session, SessionStatus), `activity.rs` (Activity, Classification), `reference.rs` (Reference, SaveReferenceInput), `metrics.rs` (FocusMetrics, DomainSummary), `archive.rs` (ArchivedDailySummary), `settings.rs` (AppSettings) — 모두 `serde::Serialize/Deserialize`, `specta::Type` 구현
- [x] T010 `src-tauri/src/services/db.rs` 작성: `r2d2` + `rusqlite` 커넥션 풀 초기화, `refinery`로 마이그레이션 실행, `app.path().app_data_dir()` 기반 DB 파일 경로 설정
- [x] T011 [P] `src-tauri/src/state/app_state.rs` 작성: `AppState { db_pool: Pool<SqliteConnectionManager>, tracker_handle: Option<JoinHandle<()>>, current_session_id: Option<String> }` — `Mutex`로 래핑
- [x] T012 `src-tauri/src/lib.rs` 기본 골격 작성: Tauri builder 설정, `AppState` 등록 (`manage()`), DB 초기화 호출, tauri-specta 바인딩 생성 (`src/types/bindings.ts` 출력) 설정
- [x] T013 [P] `src/stores/appStore.ts` 작성: Zustand 기반 클라이언트 상태 (`currentSession`, `currentActivity`, `metrics`)

**체크포인트**: `cargo check` 및 `cargo test` 통과, `npm test` 설정 확인

---

## Phase 3: US1 — 세션 시작 및 실시간 집중 상태 확인 (Priority: P1) 🎯 MVP

**목표**: 세션 시작/종료, 2초 활동 감지, 메뉴바 실시간 업데이트
**독립 테스트**: `cargo tauri dev` 실행 후 세션 시작 → 메뉴바 타이머 동작 확인

### 테스트 먼저 작성 (Red) ⚠️

> **반드시 구현 전 실패 확인 필수**

- [x] T014 [P] [US1] `src-tauri/src/services/classifier.rs` 내 `#[cfg(test)]` 블록 작성: `github.com → Focus`, `youtube.com → Distraction`, 미등록 도메인 → `Neutral`, `None` 도메인 → `Neutral` 테스트
- [x] T015 [P] [US1] `src-tauri/tests/session_commands_tests.rs` 작성: `tauri::test::mock_builder` 사용, `start_session` → Session 반환, `end_session` → `NoActiveSession` 에러(세션 없을 때), `start_session` 중복 → `SessionAlreadyActive` 에러 테스트
- [x] T016 [P] [US1] `src/__tests__/hooks/useSession.test.ts` 작성: `mockIPC()` 사용, `startSession` 호출 시 상태 업데이트, `endSession` 호출 시 세션 초기화 테스트

### 구현 (Green)

- [x] T017 [P] [US1] `src-tauri/src/services/classifier.rs` 구현: `classify(domain: Option<&str>) -> Classification` 함수, DB에서 규칙 조회하여 매칭, 없으면 `Neutral` 반환
- [x] T018 [P] [US1] `src-tauri/src/services/browser.rs` 구현: `get_browser_url(app_name: &str) -> Option<String>` 함수, `std::process::Command`로 `osascript` 실행, Chrome/Safari 각각 AppleScript, 500ms 타임아웃, 권한 거부 시 `None` 반환
- [x] T019 [US1] `src-tauri/src/services/tracker.rs` 구현: `start_tracker(app_handle, db_pool)` → `JoinHandle` 반환, 2초 폴링 루프, `NSWorkspace.frontmostApplication` 호출 (`objc2`), 활동 변경 감지, duration 계산 및 DB 저장 (T017, T018 의존)
- [x] T020 [US1] `src-tauri/src/commands/session.rs` 구현: `start_session`, `end_session`, `get_current_session`, `get_incomplete_session`, `resume_session`, `discard_incomplete_session` 커맨드 (T019 의존)
- [x] T021 [US1] `src-tauri/src/lib.rs` 트레이 설정 추가: `TrayIconBuilder`로 트레이 생성, `tray.set_title()` 1초마다 업데이트 (타이머), 세션 없을 때 `🔵` 아이콘 표시. **NSPanel 변환 추가**: `tauri-nspanel` 크레이트로 dropdown 창을 NSPanel로 변환, `NSWindowStyleMaskNonActivatingPanel`+`NSWindowCollectionBehaviorFullScreenAuxiliary` 설정으로 전체화면 앱 위 표시 구현. `window_did_resign_key` delegate로 외부 클릭 시 자동 닫힘. (T020 의존)
- [x] T022 [P] [US1] `src/services/session.ts` 작성: `invoke()` 래퍼 함수 (`startSession`, `endSession`, `getCurrentSession`, `getIncompleteSession`, `resumeSession`, `discardIncompleteSession`)
- [x] T023 [P] [US1] `src/hooks/useSession.ts` 작성: `useSession()` 커스텀 훅 — 세션 상태 구독, `startSession`/`endSession` 액션
- [x] T024 [US1] `src/components/Dropdown/SessionControls.tsx` 작성: "세션 시작" / "세션 종료" 버튼, `useSession` 훅 사용, Props 타입 명시 (`any` 금지)
- [x] T025 [US1] `src/components/Dropdown/SessionTimer.tsx` 작성: `started_at` 기반 경과 시간 실시간 표시 컴포넌트
- [x] T026 [US1] `src/pages/Dropdown.tsx` 기본 레이아웃 작성: `SessionTimer`, `SessionControls` 조합, 세션 없을 때 "시작" 버튼만 표시 (T024, T025 의존)
- [x] T027 [US1] 앱 시작 시 미완료 세션 복구 팝업 구현: `get_incomplete_session` 호출 후 "이전 세션을 이어할까요?" UI 표시 (`src/pages/Dropdown.tsx`에서 처리)

- [x] T021a 트레이 우클릭 메뉴 추가: `Menu`, `MenuItem` 사용, "focaro 종료" 항목 클릭 시 `app.exit(0)`, `show_menu_on_left_click(false)` 설정 (T021 의존)

**체크포인트**: `cargo test` 통과, 메뉴바에 타이머 표시됨, 세션 시작/종료 동작 확인

---

## Phase 4: US2 — 드롭다운 UI 집중 현황 상세 확인 (Priority: P2)

**목표**: 메뉴바 클릭 시 도넛 차트, 현재 앱, 상위 5개 활동 리스트 표시 (다크 테마)
**독립 테스트**: 메뉴바 클릭 → 드롭다운 열림, mockIPC로 UI 렌더링 단위 테스트

### 테스트 먼저 작성 (Red) ⚠️

- [x] T028 [P] [US2] `src/__tests__/components/DonutChart.test.tsx` 작성: 렌더링, 0% 빈 상태, 100% focus, size prop 테스트 (6개 테스트)
- [x] T029 [P] [US2] `src/__tests__/components/Dropdown.test.tsx` 작성: `mockIPC()` 사용, 세션 없을 때/있을 때, 복구 팝업, 세션 시작, 도넛 차트, 앱 리스트, Dashboard 버튼 테스트 (7개 테스트)
- [x] T030 [P] [US2] 통합 테스트로 대체 완료

### 구현 (Green) ✅ 완료

- [x] T031 [P] [US2] `src-tauri/src/commands/session.rs` 추가: `get_focus_stats(session_id)` — 세션의 Focus/Neutral/Distraction 누적 시간 반환, `get_top_apps(session_id)` — 앱별 누적 시간 + 분류 + 퍼센트 반환 (상위 10개), `get_current_app()` — 현재 활성 앱 이름 반환 (objc2 NSWorkspace 사용)
- [x] T032 [P] [US2] `src/services/session.ts` 업데이트: `getFocusStats(sessionId)`, `getTopApps(sessionId)`, `getCurrentApp()` invoke 래퍼 추가
- [x] T032a [P] [US2] `src/types/bindings.ts` 업데이트: `FocusStats`, `AppStat` 타입 추가
- [x] T033 [US2] `src-tauri/src/lib.rs` 드롭다운 NSPanel 설정 완료 (T021에서 구현): `TrayIconEvent::Click(Left, Up)` 핸들러에서 NSPanel show/hide 토글, `window_did_resign_key` delegate로 외부 클릭 시 자동 닫힘
- [x] T034 [US2] `src/components/Dropdown/DonutChart.tsx` 구현: SVG 기반 도넛 차트, Focus/Neutral/Distraction 색상 구분 (`#30d158`, `#636366`, `#ff453a`), 중앙에 Focus % 표시
- [x] T035 [P] [US2] 현재 앱 표시: `Dropdown.tsx`에 인라인 통합 — `getCurrentApp()` 폴링(5초), 앱 이름 표시
- [x] T036 [P] [US2] 최근 활동 리스트: `Dropdown.tsx`에 인라인 통합 — `getTopApps(sessionId)` 5초 폴링, 상위 5개 앱 표시 (이름 + 퍼센트 + 분류 색상 dot)
- [x] T037 [P] [US2] 폴링 로직: `Dropdown.tsx` 내 `useEffect`로 5초마다 `getFocusStats`, `getTopApps`, `getCurrentApp` 동시 호출; 로컬 1초 타이머로 elapsed 증가
- [x] T038 [US2] `src/pages/Dropdown.tsx` 완전 재작성: 다크 테마, 타이머 헤더, 세션 버튼, `DonutChart`, 현재 앱, 앱 리스트, Dashboard 버튼 통합 (T034~T037 의존)
- [x] T038a `src/App.css` 다크 테마 전면 재작성: `#1c1c1e` 배경, blur 효과, 컴포넌트별 다크 스타일

**체크포인트**: 드롭다운 클릭 시 도넛 차트와 현재 앱/활동 정보 표시됨, 전체화면 앱 위에도 표시됨

---

## Phase 5: US3 — 활동 분류 및 Focus Metrics 계산 (Priority: P3)

**목표**: 세션 전체 Focus/Neutral/Distraction 시간 집계 및 퍼센트 계산
**독립 테스트**: `cargo test` — 퍼센트 합 100%, 0초 세션 에러 없음

### 테스트 먼저 작성 (Red) ⚠️

- [x] T039 [P] [US3] `src-tauri/src/services/metrics.rs` 신규 생성 + `#[cfg(test)]` 단위 테스트: 빈 세션, 활동 집계, 퍼센트 합 100%, top_apps 정렬/퍼센트/분류 (6개 단위 테스트)
- [x] T040 [P] [US3] `src-tauri/tests/metrics_tests.rs` 통합 테스트: 복수 분류, 세션 격리, total_secs, 10개 제한, 동일 앱 합산, 퍼센트 합 100% (6개 통합 테스트)

### 구현 (Green)

- [x] T041 [US3] `get_focus_stats` 구현 완료 (T031에서 session.rs에 통합): DB에서 classification별 SUM(duration_secs) 집계, total_secs == 0 가드 포함 (T039 의존)
- [x] T042 [US3] `get_focus_stats` 커맨드 등록 완료 (T031에서 session.rs에 통합, activity.rs 미사용)
- [x] T043 [P] [US3] 폴링 로직 완료 (T037에서 Dropdown.tsx에 통합): 5초마다 `getFocusStats` 호출
- [x] T044 [US3] `src/pages/Dropdown.tsx`에서 `getFocusStats` → `DonutChart`에 데이터 전달 완료 (T038에서 통합)

**체크포인트**: `cargo test` 통과, 드롭다운의 원형 차트에 실제 퍼센트 표시됨

---

## Phase 6: US4 — Reference 저장 (Priority: P4)

**목표**: 현재 브라우저 URL을 제목/태그와 함께 저장
**독립 테스트**: 브라우저 활동 중 저장 버튼 클릭 → DB에 Reference 저장됨

### 테스트 먼저 작성 (Red) ⚠️

- [x] T045 [P] [US4] `src-tauri/src/services/reference.rs` 신규: `save_reference`, `get_references` 서비스 + 단위 테스트 7개 (저장/태그/조회/정렬/격리)
- [x] T046 [P] [US4] `src/__tests__/components/SaveReference.test.tsx`: mockIPC 기반 8개 테스트 (비활성, 폼 열림, URL 자동채움, 제목 없을 때, 저장, 폼 닫힘, 취소)

### 구현 (Green)

- [x] T047 [P] [US4] `src-tauri/src/commands/reference.rs` 구현: `save_reference` (NoActiveSession 가드), `get_references(session_id: Option<String>)` — services/reference.rs 위임
- [x] T047a `src-tauri/src/commands/session.rs`: `get_current_url` 커맨드 추가 — 현재 앱이 Chrome/Safari일 때 URL 반환 (AppleScript)
- [x] T048 [P] [US4] `src/services/reference.ts`: `saveReference`, `getReferences` invoke 래퍼; `src/services/session.ts`: `getCurrentUrl` 추가
- [x] T049 [US4] `src/components/Dropdown/SaveReference.tsx` 구현: 버튼(URL 없으면 비활성), 인라인 폼(URL 읽기전용, 제목 입력, 태그 입력), 저장/취소 버튼
- [x] T050 [US4] `src/pages/Dropdown.tsx` 업데이트: 세션 활성 중 `SaveReference` 통합, `getCurrentUrl` 5초 폴링으로 URL 갱신

**체크포인트**: `cargo test`, `npm test` 통과, 브라우저 URL이 자동 채워지고 저장됨

---

## Phase 7: US5 — Dashboard 이력 분석 (Priority: P5)

**목표**: Activity Timeline, Top Sites, Focus Score, Saved References, 날짜별 기록 조회
**독립 테스트**: 여러 세션 데이터 존재 시 Dashboard 열기 → 모든 섹션 정상 표시

### 테스트 먼저 작성 (Red) ⚠️

- [x] T051 [P] [US5] `src-tauri/tests/activity_tests.rs` 신규 작성: `query_activity_timeline`, `query_top_sites`, `query_daily_focus_stats` 서비스 테스트 12개 (날짜별 필터링, 도메인 집계, 정렬, limit 적용)
- [x] T052 [P] [US5] `src/__tests__/components/Dashboard/ActivityTimeline.test.tsx` 작성: 활동 목록 렌더링, 빈 상태, 도메인 표시, timeline-dot, duration 포맷 테스트 (5개)
- [x] T053 [P] [US5] `src/__tests__/components/Dashboard/TopSites.test.tsx` 작성: 도메인 목록, 빈 상태, 시간 포맷, site-dot, site-bar 테스트 (5개)

### 구현 (Green)

- [x] T054 [P] [US5] `src-tauri/src/services/archive.rs` 구현: `archive_old_data(pool, retention_days)` — 보관 기간 초과 날짜 Activity를 ArchivedDailySummary로 집계 후 원본 삭제
- [x] T055 [P] [US5] `src-tauri/src/services/activity.rs` + `commands/activity.rs` 신규: `get_activity_timeline`, `get_top_sites`, `get_daily_focus_stats` 커맨드 + `open_url` 추가
- [x] T056 [P] [US5] `src/components/Dashboard/ActivityTimeline.tsx` 구현: 활동 목록 시간순, Classification 색상 dot, duration/domain 표시
- [x] T057 [P] [US5] `src/components/Dashboard/TopSites.tsx` 구현: 도메인별 바 차트 (최장 기준 비율), 색상 구분
- [x] T058 [P] [US5] `src/components/Dashboard/FocusScore.tsx` 구현: Focus % 큰 숫자 + 3개 카테고리 바 + 총 시간
- [x] T059 [P] [US5] `src/components/Dashboard/SavedReferences.tsx` 구현: Reference 목록, URL 클릭 시 `open_url` 커맨드로 브라우저 열기
- [x] T060 [US5] `src/pages/Dashboard.tsx` 완전 재구현: 탭(타임라인/TopSites/FocusScore/References) + 날짜 필터 + 데이터 로드
- [x] T061 [US5] `open_dashboard` 커맨드 기존 구현 확인 (T031에서 완료), `src/services/activity.ts` 신규 invoke 래퍼 추가

**체크포인트**: `cargo test`, `npm test` 통과, Dashboard 열리고 날짜 필터 동작 확인

---

## Phase 8: US6 — 설정 및 Reference 팝업 (Priority: P6)

**목표**: Reference 저장 별도 팝업 창, 전역 단축키(`⌘⇧R`), 설정 창 (단축키/보관기간/분류규칙/자동실행)
**독립 테스트**: 설정에서 분류 규칙 추가 → 이후 활동에 즉시 반영됨, `⌘⇧R` 팝업 열림 확인

### 테스트 먼저 작성 (Red) ⚠️

- [x] T062a [P] [US6] `src-tauri/src/commands/settings.rs` 신규 테스트: `get_settings`, `update_settings`, `get_classification_rules`, `add_classification_rule`, `delete_classification_rule` — 단위 테스트 5개
- [x] T062b [P] [US6] `src/__tests__/pages/Settings.test.tsx` 작성: 단축키 변경, 보관 기간 선택, 규칙 추가/삭제 렌더링 테스트 (mockIPC)
- [x] T062c [P] [US6] `src/__tests__/pages/SaveReference.test.tsx` 작성 (팝업 창 버전): 폼 렌더링, URL 자동채움, 제목 없이 저장 불가, 저장 후 창 닫힘 테스트

### 구현 (Green)

- [x] T063 [US6] `src-tauri/migrations/V3__shortcut.sql` 작성: `settings` 테이블에 `shortcut_save_ref TEXT DEFAULT 'CmdOrCtrl+Shift+R'` 컬럼 추가 (마이그레이션)
- [x] T064 [P] [US6] `src-tauri/src/commands/settings.rs` 구현: `get_settings()`, `update_settings(settings)`, `get_classification_rules()`, `add_classification_rule(domain, classification)`, `delete_classification_rule(id)` 커맨드
- [x] T065 [P] [US6] `src-tauri/src/services/shortcut.rs` 구현: `register_shortcut(app, shortcut_str)` — `tauri-plugin-global-shortcut`로 `⌘⇧R` 기본 등록, 설정 변경 시 재등록
- [x] T066 [P] [US6] `src/services/settings.ts` 작성: `getSettings()`, `updateSettings(settings)`, `getClassificationRules()`, `addClassificationRule(domain, classification)`, `deleteClassificationRule(id)` invoke 래퍼
- [x] T067 [P] [US6] `src-tauri/tauri.conf.json` 업데이트: `save-reference` 창 추가 (`width: 480, height: 300, decorations: true, visible: false`), `settings` 창 추가 (`width: 600, height: 500, decorations: true, visible: false`)
- [x] T068 [US6] `src-tauri/src/commands/settings.rs`에 `open_save_reference_window`, `open_settings_window` 커맨드 추가; `src-tauri/src/lib.rs`에 트레이 우클릭 메뉴 "설정 열기" 항목 연결
- [x] T069 [P] [US6] `src/pages/SaveReference.tsx` 구현 (팝업 창 전용): URL 자동채움 (쿼리 파라미터로 전달), 제목 입력, 태그 입력, 저장 후 창 닫힘 (`getCurrentWindow().close()`)
- [x] T070 [P] [US6] `src/components/Settings/ShortcutSettings.tsx` 구현: 현재 단축키 표시, 새 단축키 입력(키 캡처), 저장 시 `updateSettings()` 호출
- [x] T071 [P] [US6] `src/components/Settings/RetentionSettings.tsx` 구현: 7일/30일/90일/무제한 라디오 선택, 저장
- [x] T072 [P] [US6] `src/components/Settings/RulesSettings.tsx` 구현: 규칙 목록 표시, 도메인+분류 추가 폼, 규칙 삭제 버튼 (즉시 반영)
- [x] T073 [P] [US6] `src/components/Settings/AutoLaunchSettings.tsx` 구현: 자동 실행 토글 (macOS `loginItems` API)
- [x] T074 [US6] `src/pages/Settings.tsx` 구현: 탭 또는 섹션으로 ShortcutSettings, RetentionSettings, RulesSettings, AutoLaunchSettings 통합
- [x] T075 [US6] `src/pages/Dropdown.tsx` 업데이트: "Reference 저장" 버튼 클릭 시 인라인 폼 대신 `openSaveReferenceWindow()` 호출로 변경

**체크포인트**: `cargo test`, `npm test` 통과, `⌘⇧R` 단축키로 팝업 열림, 설정 창에서 규칙 추가 후 분류 즉시 반영

---

## Phase 9: 아카이빙 및 크래시 복구

**목표**: 보관 기간 초과 데이터 자동 아카이빙, 앱 시작 시 미완료 세션 처리

- [x] T076 `src-tauri/src/lib.rs` 업데이트: 앱 시작 시 `archive_old_data()` 호출 (설정된 보관 기간 기준), 이후 `get_incomplete_session()` 확인
- [x] T077 `src/pages/Dropdown.tsx` 확인: 앱 초기화 시 `getIncompleteSession()` 호출, 미완료 세션 있으면 "이전 세션을 이어할까요?" 확인 다이얼로그 표시 (이미 T065에서 구현됨)

---

## Phase 10: 마무리 및 권한 처리

**목적**: 권한 오류 처리, 전체 테스트 검증, quickstart.md 기준 검증

- [x] T078 [P] Accessibility 권한 없을 때 트래커 중단 및 사용자 안내 UI 추가 (`src-tauri/src/services/tracker.rs`, `src/pages/Dropdown.tsx`)
- [x] T079 [P] Automation 권한 거부 시 `url = null` 처리 확인 및 에러 로그 추가 (`src-tauri/src/services/browser.rs`)
- [x] T080 [P] `cargo test --all` 전체 Rust 테스트 실행 및 미통과 테스트 수정
- [x] T081 [P] `npm test` 전체 React 테스트 실행 및 미통과 테스트 수정
- [x] T082 `specs/001-activity-focus-tracker/quickstart.md` 검증 체크리스트 기준으로 실제 앱 동작 확인

---

---

## ✅ 완료된 Phase (Phase 1~10)

Phase 1~10 전체 구현 완료 (2026-03-21). 이하는 v2 확장 Phase.

---

## Phase A: 온보딩 + 설정 완성 + 실시간 분류 변경

**목표**: 첫 실행 경험 개선, 설정 기능 완성, Dropdown에서 즉각적인 분류 수정

**체크포인트**: 온보딩 완료 후 직업 기반 규칙 적용 확인, Quick Override로 분류 변경 시 즉시 반영 및 영구 저장 확인

### 테스트 먼저 작성 (Red)

- [x] TA001 [P] `src-tauri/src/commands/onboarding.rs` 테스트: `get_onboarding_status`, `complete_onboarding`, `apply_profession_rules` 단위 테스트
- [x] TA002 [P] `src-tauri/src/services/classifier.rs` 테스트: title_rules 우선 적용, 이중용도 도메인 title 매칭 테스트
- [x] TA003 [P] `src/__tests__/pages/Onboarding.test.tsx`: 직업 선택 렌더링, 스킵 동작, 완료 후 커맨드 호출 테스트
- [x] TA004 [P] `src/__tests__/components/Dropdown/QuickOverride.test.tsx`: 분류 배지 클릭 → 선택기 표시, "이번만"/"항상" 동작 테스트

### 구현 (Green)

- [x] TA005 `src-tauri/migrations/V5__title_rules_onboarding.sql`: `title_rules` 테이블 + `onboarding_completed` 기본 settings 값 추가
- [x] TA006 [P] `src-tauri/src/services/classifier.rs` 업데이트: title_rules 조회 및 우선 적용 로직 추가 (domain + title keyword contains 매칭)
- [x] TA007 [P] `src-tauri/src/commands/onboarding.rs` 구현: `get_onboarding_status()`, `complete_onboarding(profession)`, `apply_profession_rules(profession)`, `add_title_rule(domain, keyword, category)`, `get_title_rules()`, `delete_title_rule(id)`
- [x] TA008 [P] `src-tauri/src/services/settings.rs` 업데이트: title_rules CRUD 서비스 함수 추가
- [x] TA009 `src-tauri/tauri.conf.json` 업데이트: `onboarding` 창 추가 (`width: 520, height: 460, decorations: true, visible: false`)
- [x] TA010 `src-tauri/src/lib.rs` 업데이트: 앱 시작 시 `onboarding_completed` 확인 → 미완료 시 onboarding 창 표시, 새 커맨드 등록
- [x] TA011 [P] `src-tauri/src/commands/session.rs` 업데이트: 세션 시작/종료 전역 단축키 커맨드 추가 (`register_session_shortcuts`)
- [x] TA012 [P] `src/pages/Onboarding.tsx` 구현: Step 1(Welcome) → Step 2(직업 선택) → Step 3(규칙 미리보기+완료) 3단계 플로우
- [x] TA013 [P] `src/components/Dropdown/QuickOverride.tsx` 구현: 분류 배지 클릭 → Focus/Neutral/Distraction 선택 → "이번만"/"항상 이렇게" 선택, title_rule 또는 domain_rule 저장
- [x] TA014 `src/pages/Dropdown.tsx` 업데이트: 현재 활동에 QuickOverride 컴포넌트 연결, 트레이 아이콘 상태 업데이트 로직 연결
- [x] TA015 [P] `src/services/onboarding.ts`: `getOnboardingStatus()`, `completeOnboarding(profession)`, `addTitleRule()`, `getTitleRules()`, `deleteTitleRule()` 서비스 레이어
- [x] TA016 `src/App.tsx` 업데이트: `onboarding` 창 라우팅 추가
- [x] TA017 [P] `src-tauri/src/lib.rs` 트레이 아이콘 갱신: 1초 루프에서 현재 분류 상태에 따라 🔵/🟢/🟡/🔴 업데이트
- [x] TA018 [P] `src/components/Settings/AutoLaunchSettings.tsx` 구현 (`tauri-plugin-autostart` 사용), Settings.tsx에 통합
- [x] TA019 `src/App.css` 업데이트: 온보딩 스타일, QuickOverride 스타일

---

## Phase B: 목표 + 알림

**목표**: 사용자가 의도를 갖고 세션을 운영할 수 있도록

- [X] TB001 [P] `src-tauri/migrations/V6__goals.sql`: `session_goals` 테이블 추가 (`target_secs`, `date`)
- [X] TB002 [P] `src-tauri/src/commands/goal.rs`: `set_daily_goal(secs)`, `get_daily_goal()`, `get_goal_progress(date)` 커맨드
- [X] TB003 [P] `src/components/Dropdown/GoalProgress.tsx`: 목표 시간 설정 + 달성률 프로그레스 바
- [X] TB004 [P] `src-tauri/src/services/notification.rs`: `send_notification(title, body)` — macOS 알림 (`tauri-plugin-notification`)
- [X] TB005 [P] 세션 루프에 알림 로직 추가: 집중도 임계값(기본 40%) 이하 10분 지속 시 알림, 목표 달성 시 알림

---

## Phase C: Dashboard 강화

**목표**: 데이터 기반 인사이트 제공

- [X] TC001 [P] `src-tauri/src/services/activity.rs` 추가: `query_weekly_report(start_date)`, `query_trend(days)` 쿼리
- [X] TC002 [P] `src/components/Dashboard/WeeklyReport.tsx`: 요일별 Focus 시간 바 차트 (이번 주 vs 지난 주)
- [X] TC003 [P] `src/components/Dashboard/TrendChart.tsx`: 최근 30일 Focus % 꺾은선 그래프
- [X] TC004 [P] `src/components/Dashboard/HabitInsights.tsx`: 요일/시간대별 패턴 분석 텍스트 요약
- [X] TC005 [P] `src/components/Dashboard/SavedReferences.tsx` 업데이트: 검색 입력창 + 태그 클릭 필터
- [X] TC006 `src/pages/Dashboard.tsx` 업데이트: 주간/트렌드 탭 추가, 인사이트 섹션 추가

---

## Phase D: 내보내기

**목표**: 데이터 주권 + 외부 활용

- [x] TD001 [P] `src-tauri/src/commands/export.rs`: `export_data(start_date, end_date, format)` — CSV/JSON 생성 후 Downloads 폴더 저장 + opener로 폴더 열기
- [x] TD002 [P] `src/components/Dashboard/ExportButton.tsx`: 날짜 범위 선택 + 형식 선택 (CSV/JSON) + 내보내기 버튼
- [x] TD003 `src/pages/Dashboard.tsx` 업데이트: ExportButton 추가

---

## Phase E: 앱 이름 기반 분류 규칙 (Issue #29)

**목표**: 웹사이트뿐 아니라 네이티브 앱(Xcode, Slack 등)도 Focus/Neutral/Distraction으로 분류 가능하게 함

**분류 우선순위 (변경 후)**:
```
title_rules (domain+keyword) → domain_rules (domain) → app_rules (app_name) → 내장 기본값 → Neutral
```

### 백엔드

- [x] TE001 `src-tauri/migrations/V7__app_rules.sql`: `app_rules` 테이블 생성
- [x] TE002 [P] `src-tauri/src/services/classifier.rs` 업데이트: `app_name` + `app_rules` 파라미터 추가, domain 없는 네이티브 앱에만 적용 (우선순위: title_rules → domain_rules → 내장 기본값 → app_rules → Neutral)
- [x] TE003 [P] `src-tauri/src/services/tracker.rs`: `load_app_rules` 추가, `poll_once`에서 호출
- [x] TE004 [P] `src-tauri/src/commands/activity.rs`: `get_tracked_apps` 커맨드 추가
- [x] TE005 [P] `src-tauri/src/commands/settings.rs` + `services/settings.rs`: `get_app_rules`, `add_app_rule`, `delete_app_rule` 추가
- [x] TE006 `src-tauri/src/lib.rs`: 커맨드 등록

### 프론트엔드

- [x] TE007 [P] `src/types/bindings.ts`: `AppRule` 인터페이스 추가
- [x] TE008 [P] `src/services/settings.ts`: `getAppRules`, `addAppRule`, `deleteAppRule` 추가
- [x] TE009 [P] `src/services/activity.ts`: `getTrackedApps()` 추가
- [x] TE010 `src/components/Settings/AppRuleSettings.tsx`: 앱 규칙 설정 컴포넌트 구현 (추적앱 드롭다운 + 직접입력, 규칙 목록 + 삭제)
- [x] TE011 `src/pages/Settings.tsx`: `<AppRuleSettings />` 섹션 추가

---

## Phase F: 타이틀 규칙 설정 UI (Issue #32)

**목표**: `title_rules` 테이블이 DB에 존재하지만 설정 UI가 없어 Quick Override로만 추가 가능. 설정 페이지에서 직접 관리(조회/삭제)할 수 있도록 구현.

### 백엔드

- [x] TF001 [P] `src-tauri/src/commands/settings.rs` + `services/settings.rs`: `get_title_rules`, `delete_title_rule` 커맨드 추가
- [x] TF002 `src-tauri/src/lib.rs`: 커맨드 등록

### 프론트엔드

- [x] TF003 [P] `src/types/bindings.ts`: `TitleRule` 인터페이스 추가 (`id`, `domain`, `keyword`, `category`)
- [x] TF004 [P] `src/services/settings.ts`: `getTitleRules`, `deleteTitleRule` 추가
- [x] TF005 `src/components/Settings/TitleRuleSettings.tsx`: 타이틀 규칙 목록(도메인+키워드+분류) + 삭제 버튼
- [x] TF006 `src/pages/Settings.tsx`: `<TitleRuleSettings />` 섹션 추가

---

## Phase G: 시간대별 히트맵 및 요일 패턴 시각화 (Issue #33)

**목표**: 대시보드에 시간대별·요일별 집중도 패턴을 시각화하여 언제 가장 집중하는지 파악 가능하게 함.

### 백엔드

- [x] TG001 `src-tauri/src/commands/activity.rs`: `get_hourly_heatmap(days: u32)` 커맨드 — 시간(0~23) × 요일(0~6) 별 평균 Focus % 반환
- [x] TG002 `src-tauri/src/commands/activity.rs`: `get_weekday_stats(days: u32)` 커맨드 — 요일별 평균 Focus 시간(분) 반환
- [x] TG003 `src-tauri/src/lib.rs`: 커맨드 등록

### 프론트엔드

- [x] TG004 [P] `src/types/bindings.ts`: `HeatmapCell` (`hour`, `weekday`, `focus_pct`), `WeekdayStat` (`weekday`, `avg_focus_mins`) 인터페이스 추가
- [x] TG005 [P] `src/services/activity.ts`: `getHourlyHeatmap`, `getWeekdayStats` 추가
- [x] TG006 `src/components/Dashboard/PatternView.tsx`: 히트맵 그리드(24×7) + 요일별 바 차트 구현
- [x] TG007 `src/pages/Dashboard.tsx`: `패턴` 탭 추가 및 `PatternView` 연결

---

## Phase H: 세션 목표 달성 히스토리 (Issue #34)

**목표**: 매일 목표 달성 여부를 기록하고 대시보드에서 스트릭 및 달성 캘린더를 확인할 수 있도록 함.

### 백엔드

- [x] TH001 `src-tauri/migrations/V8__goal_history.sql`: `goal_history` 테이블 생성 (`date TEXT PK`, `target_secs INT`, `actual_secs INT`, `achieved BOOL`)
- [x] TH002 `src-tauri/src/services/goal.rs`: `record_daily_goal_result(date, target_secs, actual_secs)` 함수 추가
- [x] TH003 `src-tauri/src/commands/session.rs`: 세션 종료 시 `record_daily_goal_result` 호출
- [x] TH004 [P] `src-tauri/src/commands/activity.rs`: `get_goal_history(days: u32)` 커맨드 — 날짜별 달성 여부 목록 반환
- [x] TH005 `src-tauri/src/lib.rs`: 커맨드 등록

### 프론트엔드

- [x] TH006 [P] `src/types/bindings.ts`: `GoalHistoryEntry` (`date`, `target_secs`, `actual_secs`, `achieved`) 인터페이스 추가
- [x] TH007 [P] `src/services/activity.ts`: `getGoalHistory(days)` 추가
- [x] TH008 `src/components/Dashboard/GoalHistory.tsx`: 최근 30일 달성 캘린더 + 연속 달성 스트릭 표시
- [x] TH009 `src/pages/Dashboard.tsx`: Focus Score 탭에 `GoalHistory` 컴포넌트 추가
- [x] TH010 `src/components/Dropdown/GoalProgress.tsx`: 연속 달성 일수 표시 추가

---

## Phase I: UI 폴리싱 및 안정성 개선 (Issue #35)

**목표**: 전반적인 UI 완성도 향상 및 알려진 UX 이슈 수정.

### 대시보드

- [x] TI001 `src/pages/Dashboard.tsx` + `src/App.css`: 탭 콘텐츠 로딩 시 스켈레톤 UI (현재 빈 화면)
- [x] TI002 각 탭 컴포넌트: 데이터 없을 때 통일된 빈 상태 메시지 및 아이콘
- [x] TI003 `src/pages/Dashboard.tsx`: 날짜 네비게이션 키보드 단축키 (← →)

### 드롭다운

- [x] TI004 `src/pages/Dropdown.tsx` + `src/App.css`: 세션 없을 때 오늘 누적 집중 시간 간략 표시

### 설정

- [x] TI005 `src/pages/Settings.tsx` + `src/App.css`: 저장 성공/실패 토스트 알림 컴포넌트
- [x] TI006 `src/pages/Settings.tsx`: 도메인/앱 규칙 추가 시 중복 체크 및 에러 피드백

### 성능

- [x] TI007 `src/pages/Dashboard.tsx`: 같은 탭 재클릭 시 리페치 방지 (이미 로딩 중이면 skip)
- [x] TI008 `src/components/Settings/*.tsx`: 설정 창 데이터 마운트 시 1회만 로드, 불필요한 재요청 제거

---

## 의존성 및 실행 순서

### Phase 의존성

- **Phase 1 (설정)**: 의존성 없음 — 즉시 시작
- **Phase 2 (기반)**: Phase 1 완료 후 시작 — 모든 Phase 3+ 차단
- **Phase 3 (US1)**: Phase 2 완료 후 시작 — MVP
- **Phase 4 (US2)**: Phase 3 완료 후 시작 (트래커, 세션 필요)
- **Phase 5 (US3)**: Phase 3 완료 후 시작 (분류 서비스 필요) — Phase 4와 병렬 가능
- **Phase 6 (US4)**: Phase 3 완료 후 시작 — Phase 4/5와 병렬 가능
- **Phase 7 (US5)**: Phase 3, 5 완료 후 시작 (아카이빙은 메트릭 의존)
- **Phase 8 (US6 설정)**: Phase 3, 6 완료 후 시작 — Reference 팝업, 전역 단축키, 설정 창
- **Phase 9 (아카이빙/크래시복구)**: Phase 7, 8 완료 후 시작
- **Phase 10 (마무리)**: 모든 Phase 완료 후

### 유저 스토리 의존성

- **US1 (P1)**: Phase 2 완료 후 독립 — MVP 완성
- **US2 (P2)**: US1 완료 필요 (트래커, 세션 상태)
- **US3 (P3)**: US1 완료 필요 (분류 서비스 공유)
- **US4 (P4)**: US1 완료 필요 (현재 URL은 트래커 상태 의존)
- **US5 (P5)**: US1 + US3 완료 필요 (아카이빙은 메트릭 집계 의존)

### 스토리 내 병렬 실행 예시 (US1)

```bash
# Red 단계 — 동시 작성
T014: classifier 단위 테스트
T015: session 커맨드 통합 테스트
T016: useSession 훅 테스트

# Green 단계 — 순서 있음
T017 → T018 → T019 (tracker → session cmd → tray)
T022 → T023 → T024 → T025 → T026 → T027 (services → hooks → components → page)
```

---

## 구현 전략

### MVP 우선 (US1만)

1. Phase 1 완료 (설정)
2. Phase 2 완료 (기반) — **모든 스토리의 전제**
3. Phase 3 완료 (US1) — **STOP: 독립 검증**
   - `cargo tauri dev` 실행
   - 세션 시작 → 메뉴바 타이머 확인
   - Chrome 사용 중 URL 감지 확인
4. MVP 확인 후 다음 우선순위로 진행

### 점진적 배포

```
Phase 1+2 완료 → Phase 3 (US1) → MVP 시연
→ Phase 4 (US2) → 드롭다운 추가
→ Phase 5 (US3) → Focus Metrics 추가
→ Phase 6 (US4) → Reference 저장 추가
→ Phase 7 (US5) → Dashboard 추가
```

---

## 노트

- **[P]** = 다른 파일, 의존성 없는 태스크 → 병렬 실행 가능
- **[Story]** = 해당 태스크의 유저 스토리 추적 레이블
- TDD 필수: 테스트 Red 확인 후 구현 시작 (헌법 III 원칙)
- `src/types/bindings.ts`는 tauri-specta 자동 생성 파일 — 수동 편집 금지
- 각 Phase 완료 후 `cargo test && npm test` 실행하여 회귀 없음 확인
