# 구현 계획: Activity Focus Tracker

**브랜치**: `001-activity-focus-tracker` | **날짜**: 2026-03-20 | **스펙**: [spec.md](spec.md)
**입력**: [specs/001-activity-focus-tracker/spec.md](spec.md)

---

## 요약

macOS 메뉴바에서 실행되는 활동 추적 기반 집중 관리 도구.
Tauri v2 + Rust 백엔드(활동 감지, 분류, SQLite 저장) + React 프론트엔드(메뉴바 드롭다운, 대시보드)로 구성된다.
2초 폴링으로 활성 앱과 브라우저 URL을 감지하고, domain 기반 규칙으로 Focus/Neutral/Distraction을 분류하여 세션 단위로 집계한다.

---

## 기술 컨텍스트

**언어/버전**: Rust stable 1.75+, TypeScript 5.x
**주요 의존성**:
- Tauri v2, tauri-specta v2
- **tauri-nspanel** (git, v2 branch) — NSPanel 변환, 전체화면 위 드롭다운 구현 ← 추가됨
- rusqlite 0.32 (bundled), r2d2 0.8, r2d2_sqlite 0.25
- refinery 0.8 (마이그레이션)
- objc2 0.5, objc2-foundation 0.2, objc2-app-kit 0.2 (macOS API)
- React 18, Vitest, Testing Library

**스토리지**: SQLite (`~/Library/Application Support/com.focaro.app/focaro.db`)
**테스트**: cargo test + tauri::test (Rust), Vitest + Testing Library + mockIPC (React)
**타겟 플랫폼**: macOS 12.0+ (Monterey)
**프로젝트 유형**: 데스크탑 앱 (메뉴바 앱)
**성능 목표**: CPU < 2% (백그라운드), 활동 감지 < 4초, Reference 저장 < 1초
**제약**: 로컬 전용, 계정 없음, 단일 세션, macOS 전용
**규모**: 단일 사용자, 로컬 SQLite

---

## 헌법 체크 (Constitution Check)

*게이트: Phase 0 리서치 전 통과 필수. Phase 1 설계 후 재확인.*

| 원칙 | 상태 | 근거 |
|------|------|------|
| I. Tauri 아키텍처 준수 | ✅ 통과 | 규정 디렉토리 구조 준수. commands/ ↔ services/ 분리 적용 |
| II. 타입 안전성 | ✅ 통과 | tauri-specta로 Rust → TS 자동 생성. `any` 금지. AppError에 serde::Serialize 구현 |
| III. TDD Red-Green-Refactor | ✅ 통과 | 각 서비스(classifier, metrics, tracker)의 단위 테스트를 구현 전 작성 |
| IV. DRY 원칙 | ✅ 통과 | 분류 로직은 `services/classifier.rs` 단일 위치. 타입은 tauri-specta 단일 출처 |
| V. 단순성 | ✅ 통과 | osascript 직접 실행(별도 크레이트 없음), rusqlite 직접 사용(tauri-plugin-sql 미사용) |

**복잡도 추가 사항**: 없음 (모든 선택이 현재 요구사항 최소 충족)

---

## 프로젝트 구조

### 문서 (이 피처)

```text
specs/001-activity-focus-tracker/
├── plan.md              # 이 파일
├── research.md          # Phase 0 리서치 결과
├── data-model.md        # 데이터 모델 및 SQL 스키마
├── quickstart.md        # 개발 시작 가이드
├── contracts/
│   └── ipc-commands.md  # Tauri IPC 커맨드 계약
└── tasks.md             # Phase 2 태스크 목록 (/speckit.tasks 생성)
```

### 소스 코드 (저장소 루트)

```text
focaro/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                 # 데스크탑 진입점 (run() 호출만)
│   │   ├── lib.rs                  # 앱 초기화, 커맨드 등록, 트레이 설정
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── session.rs          # start_session, end_session, get_current_session
│   │   │   │                       # get_incomplete_session, resume_session, discard_incomplete_session
│   │   │   │                       # get_focus_stats, get_top_apps, get_current_app  ← 추가됨
│   │   │   │                       # open_dashboard
│   │   │   ├── activity.rs         # get_recent_activities (향후)
│   │   │   │                       # get_activity_timeline, get_top_sites (Phase 7)
│   │   │   ├── reference.rs        # save_reference, get_references
│   │   │   └── settings.rs         # get_settings, update_settings
│   │   ├── state/
│   │   │   ├── mod.rs
│   │   │   └── app_state.rs        # AppState { db_pool, tracker_handle, current_session_id }
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── tracker.rs          # 2초 폴링 루프, 활동 변경 감지
│   │   │   ├── classifier.rs       # domain → Classification 규칙 매칭
│   │   │   ├── metrics.rs          # FocusMetrics 집계 계산
│   │   │   ├── browser.rs          # AppleScript 실행 (Chrome/Safari URL)
│   │   │   ├── archive.rs          # 보관 기간 초과 데이터 집계 및 삭제
│   │   │   └── db.rs               # DB 연결 풀, 마이그레이션 실행, CRUD 쿼리
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── session.rs          # Session, SessionStatus
│   │   │   ├── activity.rs         # Activity, Classification
│   │   │   ├── reference.rs        # Reference, SaveReferenceInput
│   │   │   ├── metrics.rs          # FocusMetrics, DomainSummary
│   │   │   ├── archive.rs          # ArchivedDailySummary
│   │   │   └── settings.rs         # AppSettings
│   │   └── errors.rs               # AppError (serde::Serialize, specta::Type)
│   ├── tests/
│   │   ├── classifier_tests.rs
│   │   ├── metrics_tests.rs
│   │   └── session_commands_tests.rs
│   ├── migrations/
│   │   ├── V1__init.sql
│   │   └── V2__archive.sql
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/
│   ├── components/
│   │   ├── Dropdown/
│   │   │   ├── SessionTimer.tsx
│   │   │   ├── FocusChart.tsx
│   │   │   ├── CurrentActivity.tsx
│   │   │   ├── RecentActivities.tsx
│   │   │   └── SessionControls.tsx
│   │   └── Dashboard/
│   │       ├── ActivityTimeline.tsx
│   │       ├── TopSites.tsx
│   │       ├── FocusScore.tsx
│   │       └── SavedReferences.tsx
│   ├── pages/
│   │   ├── Dropdown.tsx
│   │   └── Dashboard.tsx
│   ├── hooks/
│   │   ├── useSession.ts
│   │   ├── useActivity.ts
│   │   └── useMetrics.ts
│   ├── services/
│   │   ├── session.ts
│   │   ├── activity.ts
│   │   ├── reference.ts
│   │   └── settings.ts
│   ├── types/
│   │   └── bindings.ts             # tauri-specta 자동 생성 (수동 편집 금지)
│   ├── stores/
│   │   └── appStore.ts
│   └── __tests__/
│       ├── components/
│       └── hooks/
│
├── capabilities/
│   └── default.json
├── package.json
└── Cargo.toml
```

**구조 결정**: Tauri v2 표준 레이아웃 (헌법 I 원칙). 단일 앱 구조, Cargo 워크스페이스 없음(현재는 불필요).

---

## 아키텍처 결정 기록 (ADR)

### ADR-001: SQLite 라이브러리
- **결정**: `rusqlite` + `r2d2` 커넥션 풀
- **이유**: 데스크탑 앱에서 동기 I/O는 충분하다. `sqlx`의 async 복잡도가 불필요하다. `tauri-plugin-sql`은 SQL을 프론트엔드에 노출하여 헌법 II 위반이다.

### ADR-002: 브라우저 URL 읽기
- **결정**: `std::process::Command`로 `osascript` 직접 실행
- **이유**: 별도 크레이트 없이 표준 라이브러리로 충분하다 (헌법 V 단순성). 500ms 타임아웃으로 블로킹 방지.

### ADR-003: macOS 활성 앱 감지
- **결정**: `objc2` 크레이트로 `NSWorkspace.shared.frontmostApplication` 호출
- **이유**: Accessibility 권한 없이 공개 API로 앱 이름 읽기 가능. `active-win-pos-rs`는 macOS 지원 불안정.

### ADR-004: 타입 안전성 브릿지
- **결정**: `tauri-specta` v2로 Rust 타입 → TypeScript 자동 생성
- **이유**: 헌법 II 원칙. Rust와 TypeScript 타입을 동기화하는 단일 출처를 보장한다.

### ADR-005: 메뉴바 아이콘 동적 텍스트
- **결정**: `tray.set_title()`로 macOS 메뉴바 텍스트 직접 업데이트
- **이유**: 이미지 생성 없이 Tauri v2 API로 직접 텍스트 표시 가능.

---

## Phase 1 설계 후 헌법 재확인

| 원칙 | 상태 | 비고 |
|------|------|------|
| I. Tauri 아키텍처 | ✅ | 구조 확정, commands/services 분리 유지 |
| II. 타입 안전성 | ✅ | AppError serde 구현, specta::Type 전 모델 적용 |
| III. TDD | ✅ | tasks.md에서 테스트 태스크가 구현 태스크보다 앞서야 함 |
| IV. DRY | ✅ | classifier 단일 모듈, 타입 단일 출처 |
| V. 단순성 | ✅ | ADR-002로 불필요 크레이트 제거 확인 |
