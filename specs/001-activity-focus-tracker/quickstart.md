# 빠른 시작 가이드: Activity Focus Tracker

**브랜치**: `001-activity-focus-tracker`
**날짜**: 2026-03-20

---

## 사전 요구사항

| 도구 | 버전 | 설치 방법 |
|------|------|-----------|
| Rust | stable 1.75+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | 20+ | `brew install node` |
| Xcode Command Line Tools | 최신 | `xcode-select --install` |
| Tauri CLI | v2 | `cargo install tauri-cli --version "^2"` |

---

## 프로젝트 초기화 (신규 설정 시)

```bash
# 1. 저장소 클론 후 브랜치 전환
git clone <repo-url> focaro
cd focaro
git checkout 001-activity-focus-tracker

# 2. 프론트엔드 의존성 설치
npm install

# 3. Rust 의존성 확인
cd src-tauri && cargo check && cd ..

# 4. 개발 서버 실행
cargo tauri dev
```

---

## 디렉토리 구조

```
focaro/
├── src-tauri/                      # Rust 백엔드
│   ├── src/
│   │   ├── main.rs                 # 데스크탑 진입점
│   │   ├── lib.rs                  # 앱 초기화, 커맨드 등록
│   │   ├── commands/               # Tauri IPC 핸들러
│   │   │   ├── mod.rs
│   │   │   ├── session.rs          # start_session, end_session 등
│   │   │   ├── activity.rs         # get_recent_activities, get_focus_metrics 등
│   │   │   ├── reference.rs        # save_reference, get_references
│   │   │   └── settings.rs         # get_settings, update_settings
│   │   ├── state/
│   │   │   ├── mod.rs
│   │   │   └── app_state.rs        # AppState (DB 풀, 트래커 상태)
│   │   ├── services/               # 비즈니스 로직
│   │   │   ├── mod.rs
│   │   │   ├── tracker.rs          # 2초 폴링 트래커
│   │   │   ├── classifier.rs       # domain → Focus/Neutral/Distraction
│   │   │   ├── metrics.rs          # FocusMetrics 계산
│   │   │   ├── browser.rs          # AppleScript로 브라우저 URL 읽기
│   │   │   ├── archive.rs          # 데이터 아카이빙 서비스
│   │   │   └── db.rs               # SQLite 연결, 쿼리
│   │   ├── models/                 # 데이터 구조체
│   │   │   ├── mod.rs
│   │   │   ├── session.rs
│   │   │   ├── activity.rs
│   │   │   ├── reference.rs
│   │   │   ├── metrics.rs
│   │   │   ├── archive.rs
│   │   │   └── settings.rs
│   │   └── errors.rs               # AppError (serde::Serialize)
│   ├── tests/                      # Rust 통합 테스트
│   │   ├── classifier_tests.rs
│   │   ├── metrics_tests.rs
│   │   └── session_commands_tests.rs
│   ├── migrations/                 # SQL 마이그레이션
│   │   ├── V1__init.sql
│   │   └── V2__archive.sql
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                            # React 프론트엔드
│   ├── components/
│   │   ├── Dropdown/               # 메뉴바 드롭다운 컴포넌트
│   │   │   ├── SessionTimer.tsx
│   │   │   ├── FocusChart.tsx      # 원형 차트
│   │   │   ├── CurrentActivity.tsx
│   │   │   ├── RecentActivities.tsx
│   │   │   └── SessionControls.tsx
│   │   └── Dashboard/             # 대시보드 컴포넌트
│   │       ├── ActivityTimeline.tsx
│   │       ├── TopSites.tsx
│   │       ├── FocusScore.tsx
│   │       └── SavedReferences.tsx
│   ├── pages/
│   │   ├── Dropdown.tsx            # 메뉴바 클릭 시 팝오버 페이지
│   │   └── Dashboard.tsx           # 대시보드 페이지
│   ├── hooks/
│   │   ├── useSession.ts           # 세션 상태 훅
│   │   ├── useActivity.ts          # 실시간 활동 훅
│   │   └── useMetrics.ts           # FocusMetrics 훅
│   ├── services/                   # invoke() 래퍼 (직접 invoke 금지)
│   │   ├── session.ts
│   │   ├── activity.ts
│   │   ├── reference.ts
│   │   └── settings.ts
│   ├── types/
│   │   └── bindings.ts             # tauri-specta 자동 생성 (수동 편집 금지)
│   ├── stores/
│   │   └── appStore.ts             # 클라이언트 상태 (Zustand)
│   └── __tests__/
│       ├── components/
│       └── hooks/
│
├── capabilities/
│   └── default.json                # Tauri 권한 정의
├── package.json
└── Cargo.toml                      # 워크스페이스
```

---

## 핵심 개발 명령어

```bash
# 개발 실행
cargo tauri dev

# Rust 테스트
cd src-tauri && cargo test

# 프론트엔드 테스트
npm test

# 프론트엔드 테스트 (watch 모드)
npm run test:watch

# 타입 바인딩 재생성 (tauri-specta)
cargo tauri dev  # 개발 시 자동 생성됨

# 빌드 (macOS .app)
cargo tauri build
```

---

## macOS 권한 설정

앱 첫 실행 시 또는 개발 중 다음 권한이 필요하다.

| 권한 | 경로 | 필요한 기능 |
|------|------|-------------|
| Accessibility | 시스템 환경설정 → 개인 정보 보호 → 손쉬운 사용 | 활성 앱 감지 |
| Automation (Chrome) | 시스템 환경설정 → 개인 정보 보호 → 자동화 | Chrome URL 읽기 |
| Automation (Safari) | 시스템 환경설정 → 개인 정보 보호 → 자동화 | Safari URL 읽기 |

> **개발 시 주의**: `cargo tauri dev`로 실행한 앱은 번들 앱이 아니므로
> 터미널(또는 IDE)에 Accessibility 권한을 부여해야 할 수 있다.

---

## Red-Green-Refactor 개발 워크플로우

```bash
# 1. 테스트 파일 먼저 작성 (Red)
# src-tauri/src/services/classifier.rs 의 테스트를 먼저 작성

# 2. 테스트 실패 확인
cd src-tauri && cargo test -- --nocapture
# expected: 컴파일 에러 또는 test FAILED

# 3. 최소 구현 (Green)
# classifier.rs 구현

# 4. 테스트 통과 확인
cargo test

# 5. 리팩터 후 재확인
cargo test && npm test
```

---

## DB 위치 (개발 및 운영)

```
개발: ~/Library/Application Support/com.focaro.dev/focaro.db
운영: ~/Library/Application Support/com.focaro.app/focaro.db
```

SQLite 직접 확인:
```bash
sqlite3 ~/Library/Application\ Support/com.focaro.dev/focaro.db
.tables
SELECT * FROM sessions LIMIT 5;
```

---

## 검증 체크리스트

기능 구현 완료 후 다음을 확인한다.

- [ ] `cargo test` 모두 통과
- [ ] `npm test` 모두 통과
- [ ] `cargo tauri dev`로 앱 실행 확인
- [ ] 메뉴바에 아이콘만 표시됨 (세션 없음)
- [ ] 세션 시작 후 타이머가 메뉴바에 표시됨
- [ ] Chrome/Safari URL이 정상 감지됨 (Automation 권한 필요)
- [ ] 분류 규칙이 올바르게 적용됨
- [ ] 세션 종료 후 Focus Metrics 합계 = 100%
- [ ] 앱 재시작 시 미완료 세션 팝업 표시됨
