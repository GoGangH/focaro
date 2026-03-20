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

- [ ] T001 `cargo tauri init`으로 Tauri v2 프로젝트 초기화 및 헌법 I 기준 디렉토리 구조 생성 (`src-tauri/src/commands/`, `services/`, `models/`, `state/`, `src/components/`, `hooks/`, `services/`, `types/`, `stores/`, `__tests__/`)
- [ ] T002 [P] `src-tauri/Cargo.toml`에 Rust 의존성 추가: `rusqlite 0.31 (bundled)`, `r2d2 0.8`, `r2d2_sqlite 0.25`, `refinery 0.8`, `objc2 0.5`, `objc2-foundation 0.2`, `objc2-app-kit 0.2`, `tauri-specta 2`, `specta 2`, `serde`, `uuid`
- [ ] T003 [P] `package.json`에 프론트엔드 의존성 추가: `vitest`, `@testing-library/react`, `@testing-library/user-event`, `zustand`, `@tauri-apps/api`; `vite.config.ts`에 Vitest 설정 추가
- [ ] T004 [P] `src-tauri/tauri.conf.json` 설정: `LSUIElement` (독 아이콘 숨김), `dropdown` 창 (`decorations: false`, `transparent: true`, `visible: false`), `dashboard` 창 설정
- [ ] T005 [P] `capabilities/default.json` 권한 설정: Tauri v2 IPC 커맨드 허용 목록 등록

---

## Phase 2: 기반 인프라 (모든 유저 스토리 선행 필수)

**목적**: 모든 스토리가 공유하는 DB, 에러 타입, 모델, 앱 상태 구축

**⚠️ 체크포인트**: 이 Phase가 완료되어야 Phase 3+ 작업 시작 가능

### 테스트 먼저 작성 (Red) ⚠️

> **헌법 III 준수: 구현 전 반드시 실패 테스트 확인 필수**

- [ ] T006a [P] `src-tauri/src/errors.rs` 내 `#[cfg(test)]` 블록 작성: `AppError::Database`, `AppError::NoActiveSession` 등 각 variant가 `serde_json::to_string()`으로 직렬화 가능한지 확인하는 단위 테스트 작성 (컴파일 에러 → 실패 확인 후 구현)
- [ ] T006b [P] `src-tauri/src/models/` 각 파일 내 `#[cfg(test)]` 블록 작성: `Session`, `Activity`, `FocusMetrics`, `AppSettings` 등 주요 모델의 `serde_json` 직렬화 → 역직렬화 왕복 단위 테스트 (컴파일 에러 → 실패 확인 후 구현)
- [ ] T006c `src-tauri/src/services/db.rs` 내 `#[cfg(test)]` 블록 작성: 인메모리 SQLite(`":memory:"`)에서 마이그레이션 실행 후 `sessions`, `activities`, `classification_rules` 테이블 존재 및 기본 규칙 데이터 삽입 확인 단위 테스트 (컴파일 에러 → 실패 확인 후 구현)

### 구현 (Green)

- [ ] T006 `src-tauri/migrations/V1__init.sql` 작성: `sessions`, `activities`, `classification_rules` (기본 규칙 포함), `settings` 테이블 스키마 및 초기 데이터
- [ ] T007 [P] `src-tauri/migrations/V2__archive.sql` 작성: `archived_daily_summaries`, `references` 테이블 스키마
- [ ] T008 [P] `src-tauri/src/errors.rs` 작성: `AppError` enum (`Database`, `PermissionDenied`, `SessionAlreadyActive`, `NoActiveSession`, `NotFound`, `Internal`) — `serde::Serialize`, `specta::Type` 구현 (T006a 통과 후)
- [ ] T009 [P] `src-tauri/src/models/` 전체 작성: `session.rs` (Session, SessionStatus), `activity.rs` (Activity, Classification), `reference.rs` (Reference, SaveReferenceInput), `metrics.rs` (FocusMetrics, DomainSummary), `archive.rs` (ArchivedDailySummary), `settings.rs` (AppSettings) — 모두 `serde::Serialize/Deserialize`, `specta::Type` 구현 (T006b 통과 후)
- [ ] T010 `src-tauri/src/services/db.rs` 작성: `r2d2` + `rusqlite` 커넥션 풀 초기화, `refinery`로 마이그레이션 실행, `app.path().app_data_dir()` 기반 DB 파일 경로 설정 (T006c 통과 후)
- [ ] T011 [P] `src-tauri/src/state/app_state.rs` 작성: `AppState { db_pool: Pool<SqliteConnectionManager>, tracker_handle: Option<JoinHandle<()>>, current_session_id: Option<String> }` — `Mutex`로 래핑
- [ ] T012 `src-tauri/src/lib.rs` 기본 골격 작성: Tauri builder 설정, `AppState` 등록 (`manage()`), DB 초기화 호출, tauri-specta 바인딩 생성 (`src/types/bindings.ts` 출력) 설정
- [ ] T013 [P] `src/stores/appStore.ts` 작성: Zustand 기반 클라이언트 상태 (`currentSession`, `currentActivity`, `metrics`)

**체크포인트**: `cargo check` 및 `cargo test` 통과 (T006a~T006c 포함), `npm test` 설정 확인

---

## Phase 3: US1 — 세션 시작 및 실시간 집중 상태 확인 (Priority: P1) 🎯 MVP

**목표**: 세션 시작/종료, 2초 활동 감지, 메뉴바 실시간 업데이트
**독립 테스트**: `cargo tauri dev` 실행 후 세션 시작 → 메뉴바 타이머 동작 확인

### 테스트 먼저 작성 (Red) ⚠️

> **반드시 구현 전 실패 확인 필수**

- [ ] T014 [P] [US1] `src-tauri/src/services/classifier.rs` 내 `#[cfg(test)]` 블록 작성: `github.com → Focus`, `youtube.com → Distraction`, 미등록 도메인 → `Neutral`, `None` 도메인 → `Neutral` 테스트
- [ ] T015 [P] [US1] `src-tauri/tests/session_commands_tests.rs` 작성: `tauri::test::mock_builder` 사용, `start_session` → Session 반환, `end_session` → `NoActiveSession` 에러(세션 없을 때), `start_session` 중복 → `SessionAlreadyActive` 에러 테스트
- [ ] T016 [P] [US1] `src/__tests__/hooks/useSession.test.ts` 작성: `mockIPC()` 사용, `startSession` 호출 시 상태 업데이트, `endSession` 호출 시 세션 초기화 테스트

### 구현 (Green)

- [ ] T017 [P] [US1] `src-tauri/src/services/classifier.rs` 구현: `classify(domain: Option<&str>) -> Classification` 함수, DB에서 규칙 조회하여 매칭, 없으면 `Neutral` 반환
- [ ] T018 [P] [US1] `src-tauri/src/services/browser.rs` 구현: `get_browser_url(app_name: &str) -> Option<String>` 함수, `std::process::Command`로 `osascript` 실행, Chrome/Safari 각각 AppleScript, 500ms 타임아웃, 권한 거부 시 `None` 반환
- [ ] T019 [US1] `src-tauri/src/services/tracker.rs` 구현: `start_tracker(app_handle, db_pool)` → `JoinHandle` 반환, 2초 폴링 루프, `NSWorkspace.frontmostApplication` 호출 (`objc2`), 활동 변경 감지, duration 계산 및 DB 저장 (T017, T018 의존)
- [ ] T020 [US1] `src-tauri/src/commands/session.rs` 구현: `start_session`, `end_session`, `get_current_session`, `get_incomplete_session`, `resume_session`, `discard_incomplete_session` 커맨드 (T019 의존)
- [ ] T021 [US1] `src-tauri/src/lib.rs` 트레이 설정 추가: `TrayIconBuilder`로 트레이 생성, `tray.set_title()` 2초마다 업데이트 (타이머 + Focus %), 세션 없을 때 아이콘만 표시 (T020 의존)
- [ ] T022 [P] [US1] `src/services/session.ts` 작성: `invoke()` 래퍼 함수 (`startSession`, `endSession`, `getCurrentSession`, `getIncompleteSession`, `resumeSession`, `discardIncompleteSession`)
- [ ] T023 [P] [US1] `src/hooks/useSession.ts` 작성: `useSession()` 커스텀 훅 — 세션 상태 구독, `startSession`/`endSession` 액션
- [ ] T024 [US1] `src/components/Dropdown/SessionControls.tsx` 작성: "세션 시작" / "세션 종료" 버튼, `useSession` 훅 사용, Props 타입 명시 (`any` 금지)
- [ ] T025 [US1] `src/components/Dropdown/SessionTimer.tsx` 작성: `started_at` 기반 경과 시간 실시간 표시 컴포넌트
- [ ] T026 [US1] `src/pages/Dropdown.tsx` 기본 레이아웃 작성: `SessionTimer`, `SessionControls` 조합, 세션 없을 때 "시작" 버튼만 표시 (T024, T025 의존)
- [ ] T027 [US1] 미완료 세션 복구 팝업 기본 구현: `src/pages/Dropdown.tsx` 마운트 시 `getIncompleteSession()` 호출 → 결과 있으면 "이전 세션을 이어할까요?" 확인 다이얼로그 표시, "이어가기" → `resumeSession()`, "종료" → `discardIncompleteSession()` 호출

**체크포인트**: `cargo test` 통과, 메뉴바에 타이머 표시됨, 세션 시작/종료 동작 확인

---

## Phase 4: US2 — 드롭다운 UI 집중 현황 상세 확인 (Priority: P2)

**목표**: 메뉴바 클릭 시 원형 차트, 현재 활동, 최근 3개 활동 표시
**독립 테스트**: 메뉴바 클릭 → 드롭다운 열림, mockIPC로 UI 렌더링 단위 테스트

### 테스트 먼저 작성 (Red) ⚠️

- [ ] T028 [P] [US2] `src/__tests__/components/FocusChart.test.tsx` 작성: `mockIPC()` 사용, `{ focus: 60, neutral: 20, distraction: 20 }` 데이터로 차트 렌더링 확인, 0% 일 때 빈 상태 렌더링 테스트
- [ ] T029 [P] [US2] `src/__tests__/components/CurrentActivity.test.tsx` 작성: 브라우저 활동(도메인 있음) / 일반 앱(도메인 없음) 렌더링 테스트
- [ ] T030 [P] [US2] `src/__tests__/components/RecentActivities.test.tsx` 작성: 3개 활동 목록 렌더링, 빈 목록 상태 테스트

### 구현 (Green)

- [ ] T031 [P] [US2] `src-tauri/src/commands/activity.rs` 구현: `get_recent_activities(limit: u32)` 커맨드 — 현재 세션의 최근 N개 활동 반환
- [ ] T032 [P] [US2] `src/services/activity.ts` 작성: `getRecentActivities(limit)`, `getFocusMetrics(sessionId)` invoke 래퍼
- [ ] T033 [US2] `src-tauri/src/lib.rs` 드롭다운 창 설정: `TrayIconEvent::Click` 핸들러에서 `dropdown` 창 show/hide 토글, `WindowEvent::Focused(false)` 시 자동 숨김
- [ ] T034 [US2] `src/components/Dropdown/FocusChart.tsx` 구현: SVG 원형 차트 (Focus/Neutral/Distraction 비율), `FocusMetrics` Props 타입 명시
- [ ] T035 [P] [US2] `src/components/Dropdown/CurrentActivity.tsx` 구현: 현재 앱 이름, 도메인(있을 때만), Classification 배지 표시
- [ ] T036 [P] [US2] `src/components/Dropdown/RecentActivities.tsx` 구현: 최근 3개 활동 목록 (앱 이름, 도메인, 지속 시간), `Activity[]` Props 타입 명시
- [ ] T037 [P] [US2] `src/hooks/useActivity.ts` 작성: 2초마다 `getRecentActivities(3)` 폴링, 현재 활동 상태 관리
- [ ] T038 [US2] `src/pages/Dropdown.tsx` 업데이트: `FocusChart`, `CurrentActivity`, `RecentActivities` 통합 (T034, T035, T036, T037 의존)

**체크포인트**: `npm test` 통과, 드롭다운 클릭 시 원형 차트와 활동 정보 표시됨

---

## Phase 5: US3 — 활동 분류 및 Focus Metrics 계산 (Priority: P3)

**목표**: 세션 전체 Focus/Neutral/Distraction 시간 집계 및 퍼센트 계산
**독립 테스트**: `cargo test` — 퍼센트 합 100%, 0초 세션 에러 없음

### 테스트 먼저 작성 (Red) ⚠️

- [ ] T039 [P] [US3] `src-tauri/src/services/metrics.rs` 내 `#[cfg(test)]` 블록 작성: focus 60s + neutral 30s + distraction 30s → 각 퍼센트 50%, 25%, 25%, 합 100% 테스트; total_secs=0 → 모든 퍼센트 0.0 테스트
- [ ] T040 [P] [US3] `src-tauri/tests/metrics_tests.rs` 작성: `get_focus_metrics` 커맨드 통합 테스트 — 여러 활동 저장 후 메트릭 조회

### 구현 (Green)

- [ ] T041 [US3] `src-tauri/src/services/metrics.rs` 구현: `calculate_metrics(session_id, db) -> FocusMetrics` 함수, `total_secs == 0` 가드, 부동소수점 정밀도 처리 (T039 의존)
- [ ] T042 [US3] `src-tauri/src/commands/activity.rs` 업데이트: `get_focus_metrics(session_id: String)` 커맨드 추가 (T041 의존)
- [ ] T043 [P] [US3] `src/hooks/useMetrics.ts` 작성: 5초마다 `getFocusMetrics` 폴링, `FocusMetrics` 상태 관리
- [ ] T044 [US3] `src/pages/Dropdown.tsx` 업데이트: `useMetrics` 훅으로 `FocusChart`에 실제 메트릭 데이터 전달 (T043 의존)

**체크포인트**: `cargo test` 통과, 드롭다운의 원형 차트에 실제 퍼센트 표시됨

---

## Phase 6: US4 — Reference 저장 (Priority: P4)

**목표**: 현재 브라우저 URL을 제목/태그와 함께 저장
**독립 테스트**: 브라우저 활동 중 저장 버튼 클릭 → DB에 Reference 저장됨

### 테스트 먼저 작성 (Red) ⚠️

- [ ] T045 [P] [US4] `src-tauri/tests/session_commands_tests.rs` 추가: `save_reference` 커맨드 테스트 — 세션 없을 때 `NoActiveSession` 에러, 정상 저장 시 `Reference` 반환
- [ ] T046 [P] [US4] `src/__tests__/components/SaveReference.test.tsx` 작성: `mockIPC()` 사용, URL 자동 채워짐, 제목 입력 후 저장 버튼 동작 테스트

### 구현 (Green)

- [ ] T047 [P] [US4] `src-tauri/src/commands/reference.rs` 구현: `save_reference(input: SaveReferenceInput)`, `get_references(session_id: Option<String>)` 커맨드
- [ ] T048 [P] [US4] `src/services/reference.ts` 작성: `saveReference(input)`, `getReferences(sessionId?)` invoke 래퍼
- [ ] T049 [US4] `src/components/Dropdown/SaveReference.tsx` 구현: "Reference 저장" 버튼, 클릭 시 현재 URL 자동 채워진 인라인 폼 표시, 제목 입력 필드, 태그 입력 (선택), 저장 버튼
- [ ] T050 [US4] `src/pages/Dropdown.tsx` 업데이트: `SaveReference` 컴포넌트 통합, 브라우저 활동 중일 때만 버튼 활성화 (T049 의존)

**체크포인트**: `cargo test`, `npm test` 통과, 브라우저 URL이 자동 채워지고 저장됨

---

## Phase 7: US5 — Dashboard 이력 분석 (Priority: P5)

**목표**: Activity Timeline, Top Sites, Focus Score, Saved References, 날짜별 기록 조회
**독립 테스트**: 여러 세션 데이터 존재 시 Dashboard 열기 → 모든 섹션 정상 표시

### 테스트 먼저 작성 (Red) ⚠️

- [ ] T051 [P] [US5] `src-tauri/tests/metrics_tests.rs` 추가: `get_activity_timeline`, `get_top_sites` 커맨드 테스트 — 날짜별 필터링, 보관 기간 초과 시 ArchivedDailySummary 반환
- [ ] T052 [P] [US5] `src/__tests__/components/ActivityTimeline.test.tsx` 작성: `mockIPC()` 사용, 활동 목록 시간순 렌더링, 빈 상태 렌더링 테스트
- [ ] T053 [P] [US5] `src/__tests__/components/TopSites.test.tsx` 작성: 도메인 사용 시간 내림차순 정렬 렌더링 테스트

### 구현 (Green)

- [ ] T054 [P] [US5] `src-tauri/src/services/archive.rs` 구현: `archive_old_data(db, retention_days)` — 보관 기간 초과 날짜의 Activity를 `ArchivedDailySummary`로 집계 후 원본 삭제
- [ ] T055 [P] [US5] `src-tauri/src/commands/activity.rs` 업데이트: `get_activity_timeline(date: String)`, `get_top_sites(date: String, limit: u32)` 커맨드 추가 — 보관 기간 내외 분기 처리
- [ ] T056 [P] [US5] `src/components/Dashboard/ActivityTimeline.tsx` 구현: 활동 목록 시간순 표시, Classification별 색상 구분
- [ ] T057 [P] [US5] `src/components/Dashboard/TopSites.tsx` 구현: 도메인별 누적 시간 바 차트 (사용 시간 기준 정렬)
- [ ] T058 [P] [US5] `src/components/Dashboard/FocusScore.tsx` 구현: Focus % 요약 표시, 날짜 필터 컨트롤
- [ ] T059 [P] [US5] `src/components/Dashboard/SavedReferences.tsx` 구현: `Reference[]` 목록 표시, URL 클릭 시 브라우저 열기
- [ ] T060 [US5] `src/pages/Dashboard.tsx` 작성: 모든 Dashboard 컴포넌트 통합, 날짜 필터 상태 관리 (T056~T059 의존)
- [ ] T061 [US5] Dashboard 창 열기 연결: `src-tauri/src/commands/activity.rs`에 `open_dashboard` 커맨드 추가 (session.rs 책임 범위 외), `src/components/Dropdown/SessionControls.tsx`에 "Dashboard 열기" 버튼 추가

**체크포인트**: `cargo test`, `npm test` 통과, Dashboard 열리고 날짜 필터 동작 확인

---

## Phase 8: 설정 및 크래시 복구

**목표**: 사용자 설정 (보관 기간), 앱 시작 시 미완료 세션 처리

- [ ] T062 [P] `src-tauri/src/commands/settings.rs` 구현: `get_settings()`, `update_settings(settings: AppSettings)` 커맨드
- [ ] T063 [P] `src/services/settings.ts` 작성: `getSettings()`, `updateSettings(settings)` invoke 래퍼
- [ ] T064 `src-tauri/src/lib.rs` 업데이트: 앱 시작 시 `get_settings()`로 보관 기간 조회 후 `archive_old_data(db, retention_days)` 호출 — FR-024 아카이빙 트리거 완성 (T054 의존)
- [ ] T065 `src/pages/Dropdown.tsx` 업데이트: T027의 복구 팝업과 T064의 아카이빙 완료 후 초기화 순서 조정 — 아카이빙 → 미완료 세션 확인 → 팝업 표시 순서로 연결, 설정 보관 기간에 따라 동적 처리 확인

---

## Phase 9: 마무리 및 권한 처리

**목적**: 권한 오류 처리, 전체 테스트 검증, quickstart.md 기준 검증

- [ ] T066 [P] Accessibility 권한 없을 때 트래커 중단 및 사용자 안내 UI 추가 (`src-tauri/src/services/tracker.rs`, `src/pages/Dropdown.tsx`)
- [ ] T067 [P] Automation 권한 거부 시 `url = null` 처리 확인 및 에러 로그 추가 (`src-tauri/src/services/browser.rs`)
- [ ] T068 [P] `cargo test --all` 전체 Rust 테스트 실행 및 미통과 테스트 수정
- [ ] T069 [P] `npm test` 전체 React 테스트 실행 및 미통과 테스트 수정
- [ ] T070 `specs/001-activity-focus-tracker/quickstart.md` 검증 체크리스트 기준으로 실제 앱 동작 확인
- [ ] T071 [P] SC-005 CPU 사용량 검증: `cargo tauri dev` 실행 후 세션 시작 상태에서 macOS 활성 모니터로 2분간 CPU 평균 측정 → 2% 미만 확인, 초과 시 트래커 루프 최적화
- [ ] T072 [P] SC-006 Reference 저장 응답시간 검증: 브라우저 활동 중 "Reference 저장" 버튼 클릭부터 성공 피드백까지 1초 이내 완료 확인 (수동 측정), 초과 시 DB 쿼리 최적화

---

## 의존성 및 실행 순서

### Phase 의존성

- **Phase 1 (설정)**: 의존성 없음 — 즉시 시작
- **Phase 2 (기반)**: Phase 1 완료 후 시작 — 모든 Phase 3+ 차단. **Red 태스크(T006a~T006c)를 Green(T006~T013) 이전에 완료해야 함.**
- **Phase 3 (US1)**: Phase 2 완료 후 시작 — MVP
- **Phase 4 (US2)**: Phase 3 완료 후 시작 (트래커, 세션 필요)
- **Phase 5 (US3)**: Phase 3 완료 후 시작 (분류 서비스 필요) — Phase 4와 병렬 가능
- **Phase 6 (US4)**: Phase 3 완료 후 시작 — Phase 4/5와 병렬 가능
- **Phase 7 (US5)**: Phase 3, 5 완료 후 시작 (아카이빙은 메트릭 의존)
- **Phase 8 (설정)**: Phase 3 완료 후 시작 — 대부분 Phase와 병렬 가능
- **Phase 9 (마무리)**: 모든 Phase 완료 후

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
