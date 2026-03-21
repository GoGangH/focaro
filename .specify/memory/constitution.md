<!--
SYNC IMPACT REPORT
==================
Version change: 1.0.1 → 1.1.0 (Git 브랜치 원칙 추가)
Modified principles: 없음
Added sections:
  - VI. Git 브랜치 및 협업 원칙 (NON-NEGOTIABLE)
  - 개발 워크플로우: 이슈 → 브랜치 → PR 순서 명시

Removed sections: 없음

Templates requiring updates:
  ✅ CLAUDE.md — Git 워크플로우 섹션 추가

Deferred TODOs: 없음
-->

# Focaro Constitution

## 핵심 원칙

### I. Tauri 아키텍처 준수

이 프로젝트는 macOS를 주 대상으로 하는 Tauri v2 데스크탑 애플리케이션이다.
모든 코드와 구조는 아래 파일 레이아웃을 반드시 따른다.

```text
focaro/
├── src-tauri/                  # Rust 백엔드
│   ├── src/
│   │   ├── main.rs             # 데스크탑 진입점 (app_lib::run() 호출만)
│   │   ├── lib.rs              # 핵심 로직 및 모바일 진입점
│   │   ├── commands/           # #[tauri::command] 핸들러 모음
│   │   ├── state/              # Tauri State<T> 기반 애플리케이션 상태
│   │   ├── services/           # 비즈니스 로직 (command와 분리)
│   │   ├── models/             # serde::Serialize/Deserialize 데이터 모델
│   │   └── errors.rs           # 커스텀 에러 타입 (serde::Serialize 구현)
│   ├── tests/                  # Rust 통합 테스트
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                        # React 프론트엔드
│   ├── components/             # 재사용 가능한 UI 컴포넌트
│   ├── pages/                  # 페이지 단위 컴포넌트
│   ├── hooks/                  # 커스텀 React 훅
│   ├── services/               # IPC 서비스 레이어 (invoke 래퍼)
│   ├── types/                  # TypeScript 타입 (자동 생성 + 수동)
│   ├── stores/                 # 클라이언트 상태 관리
│   └── __tests__/              # 프론트엔드 테스트
│
├── capabilities/               # Tauri 권한 정의 파일
├── package.json
└── Cargo.toml (워크스페이스)
```

**규칙**:
- Rust 비즈니스 로직은 `services/`에, IPC 진입점만 `commands/`에 둔다.
- 상태는 `tauri::State<Mutex<T>>`를 통해 관리하며 `Arc`로 별도 감쌀 필요 없다.
- 프론트엔드에서 `invoke()`를 직접 호출하는 것을 금지하며, 반드시 `src/services/` 래퍼를 경유한다.

### II. 타입 안전성 (NON-NEGOTIABLE)

Rust-TypeScript 경계를 포함한 모든 코드는 타입 안전성을 최우선으로 한다.

**Rust 측**:
- 커맨드 인자는 `serde::Deserialize`를 구현해야 한다.
- 커맨드 반환값은 `serde::Serialize`를 구현해야 한다.
- 에러 타입은 `serde::Serialize`를 구현한 커스텀 타입이어야 한다 (`anyhow::Error`를 그대로 반환 금지).
- `unwrap()` / `expect()` 사용은 테스트 코드를 제외하고 금지한다.

**TypeScript 측**:
- `any` 타입 사용은 금지한다. 불가피한 경우 `unknown`을 사용하고 타입 가드를 적용한다.
- `tauri-specta` 또는 `tauri-typegen`을 통해 Rust 타입에서 TypeScript 타입을 자동 생성한다.
- React 컴포넌트 Props는 모두 명시적 타입 선언이 필요하다.

**이유**: 컴파일 타임에 경계 오류를 잡아 런타임 장애를 예방한다.

### III. 테스트 우선 개발 / Red-Green-Refactor (NON-NEGOTIABLE)

모든 기능은 테스트가 먼저 작성되어야 하며, Red-Green-Refactor 사이클을 엄격히 준수한다.

**사이클**:
1. **Red**: 실패하는 테스트를 먼저 작성한다.
2. **Green**: 테스트를 통과하는 최소한의 코드를 구현한다.
3. **Refactor**: 중복 제거 및 품질 개선 후 테스트가 여전히 통과하는지 확인한다.

**Rust 테스트**:
- 단위 테스트: `src-tauri/src/` 내 각 모듈에 `#[cfg(test)]` 블록으로 작성한다.
- 통합 테스트: `src-tauri/tests/`에서 `tauri::test::mock_builder`를 사용한다.
- 커맨드 테스트: `get_ipc_response`로 IPC 응답을 검증한다.

**React 테스트**:
- 단위/컴포넌트 테스트: Vitest + Testing Library로 `src/__tests__/`에 작성한다.
- IPC 모킹: `mockIPC()`로 Rust 백엔드 없이 프론트엔드를 독립 테스트한다.

**이유**: 테스트 없는 구현은 허용되지 않는다. 테스트가 먼저 실패해야 구현 의미가 있다.

### IV. DRY 원칙 — 코드 중복 제거

동일한 로직이 두 곳 이상에 나타나는 것을 허용하지 않는다.

**규칙**:
- 공통 Rust 로직은 `services/` 또는 별도 모듈로 추출한다.
- 공통 React 로직은 커스텀 훅(`hooks/`) 또는 유틸 함수로 추출한다.
- 타입 정의는 단일 출처(Single Source of Truth)를 유지한다: Rust 타입 → 자동 생성된 TypeScript 타입.
- 복사-붙여넣기 코드는 PR 리뷰에서 거부된다.

**이유**: 중복 코드는 버그 수정과 변경을 두 배로 만들어 유지보수 비용을 가중시킨다.

### VI. Git 브랜치 및 협업 원칙 (NON-NEGOTIABLE)

모든 작업은 GitHub Issue와 연결된 브랜치에서 진행하며, PR을 통해 병합한다.

**브랜치 구조 (Git Flow)**:
```
main        ← 프로덕션 전용. 릴리즈 태그만 병합. 직접 커밋 금지.
develop     ← 통합 브랜치. 모든 기능 브랜치의 병합 대상.
feat/...    ← 기능 브랜치. develop에서 분기, develop으로 PR.
fix/...     ← 버그 수정. develop에서 분기.
hotfix/...  ← 긴급 수정. main에서 분기 → main + develop 모두 PR.
chore/...   ← 설정/문서. develop에서 분기.
```

**작업 시작 전**:
- 모든 작업은 **GitHub Issue를 먼저 생성**한다. 이슈 없이 브랜치를 만들거나 코드를 작성하는 것을 금지한다.
- 이슈 제목은 작업 내용을 명확히 서술한다.
- 이슈에 관련 스펙/플랜 문서 링크를 첨부한다.
- 작업 브랜치는 반드시 **`develop`에서 분기**한다 (`hotfix` 제외).

**브랜치 명명 규칙**:
```
{type}/#{issue-number}-{short-description}

예시:
  feat/#12-session-management
  fix/#34-tracker-cpu-spike
  chore/#5-project-setup
  hotfix/#99-critical-data-loss
```

브랜치 타입:
- `feat/` — 새 기능 구현
- `fix/` — 버그 수정
- `hotfix/` — 프로덕션 긴급 수정 (main에서 분기)
- `chore/` — 설정, 도구, 문서 등 비기능 작업
- `refactor/` — 동작 변경 없는 코드 개선
- `test/` — 테스트 추가/수정

**PR 규칙**:
- 기능/수정 브랜치 → **`develop`** 으로 PR
- 릴리즈 준비 완료 시 `develop` → **`main`** 으로 PR (버전 태그 포함)
- `hotfix` → **`main`** PR 후, 동일 내용을 **`develop`** 에도 PR
- PR 제목 형식: `[#12] 세션 관리 기능 구현`
- PR 본문에 `Closes #12` 또는 `Fixes #12`를 명시한다.
- PR은 Constitution Check를 모두 통과해야 병합할 수 있다.

**커밋 메시지 규칙**:
```
{type}(#{issue}): {설명}

예시:
  feat(#12): 세션 시작/종료 커맨드 구현
  test(#12): classifier 단위 테스트 작성
  fix(#34): 트래커 CPU 과부하 수정
```

**이유**: 이슈 없는 작업은 추적이 불가능하고, 변경 이유를 나중에 파악할 수 없다. `main`-`develop` 구조는 프로덕션 안정성을 보장하며 모든 변경은 이슈로 시작하여 PR로 끝난다.

### V. 단순성 (YAGNI)

현재 요구사항에 필요한 것만 구현한다.

**규칙**:
- 미래 기능을 위한 추상화나 플러그인 시스템을 미리 만들지 않는다.
- 한 곳에서만 쓰이는 헬퍼 함수/컴포넌트는 인라인으로 유지한다.
- 설정 가능성(configurability)은 실제로 두 가지 이상의 값이 필요할 때만 추가한다.
- 아키텍처 복잡도 증가는 헌법 거버넌스 검토를 거쳐야 한다.

**이유**: 조기 추상화는 유지비용이 크고 실제 요구사항과 어긋나기 쉽다.

## 기술 스택 및 프로젝트 구조

| 영역 | 기술 | 비고 |
|------|------|------|
| 데스크탑 프레임워크 | Tauri v2 | macOS 우선 타겟 |
| 백엔드 언어 | Rust (stable) | `src-tauri/` |
| 프론트엔드 | React + TypeScript | `src/` |
| 타입 공유 | tauri-specta 또는 tauri-typegen | Rust → TS 자동 생성 |
| Rust 테스트 | cargo test + tauri::test | 단위 및 통합 |
| 프론트엔드 테스트 | Vitest + Testing Library | mockIPC 활용 |
| E2E 테스트 | tauri-driver + WebDriver | 필요 시 추가 |
| 상태 관리 (Rust) | tauri::State<Mutex<T>> | 전역 앱 상태 |
| 상태 관리 (React) | 훅 기반 또는 Zustand | 로컬/클라이언트 상태 |

**의존성 추가 원칙**: 새 크레이트/패키지 추가 시 기존 기능으로 대체 불가능함을 먼저 검토한다.

## 개발 워크플로우

### 기능 개발 순서

1. **이슈 생성**: GitHub Issue를 먼저 생성한다. 스펙/플랜 링크 첨부.
2. **브랜치 생성**: `feat/#{issue}-{description}` 형식으로 브랜치를 만든다.
3. **스펙 작성**: `/speckit.spec`으로 기능 요구사항 및 인수 조건 정의.
4. **계획 수립**: `/speckit.plan`으로 아키텍처 결정 및 파일 구조 설계.
5. **테스트 작성 (Red)**: 구현 전 실패하는 테스트를 먼저 작성한다.
6. **구현 (Green)**: 테스트를 통과하는 최소 코드를 작성한다.
7. **리팩터**: 중복 제거, 타입 안전성 강화, 단순화.
8. **PR 제출**: `[#{issue}] 제목` 형식, `Closes #{issue}` 본문 포함, Constitution Check 통과.

### Constitution Check (PR 게이트)

다음 항목을 모두 통과해야 PR이 승인될 수 있다.

- [ ] 연결된 GitHub Issue가 존재하는가?
- [ ] 브랜치 이름이 `{type}/#{issue}-{description}` 형식인가?
- [ ] PR 제목에 이슈 번호가 포함되어 있는가? (`[#{issue}] 제목`)
- [ ] PR 본문에 `Closes #` 또는 `Fixes #`가 있는가?
- [ ] 모든 신규 기능에 테스트가 있는가?
- [ ] 테스트가 구현 전에 작성되었는가 (커밋 히스토리 확인)?
- [ ] `any` 타입이 사용되지 않았는가?
- [ ] `unwrap()` / `expect()`가 프로덕션 코드에 없는가?
- [ ] 에러 타입이 `serde::Serialize`를 구현하는가?
- [ ] 중복 코드가 없는가?
- [ ] 현재 요구사항에 필요하지 않은 추상화가 없는가?

## 거버넌스

이 헌법은 모든 다른 관행과 문서보다 우선한다.

**개정 절차**:
1. 변경 사항을 `/speckit.constitution` 커맨드로 제안한다.
2. 변경 이유와 영향 범위를 Sync Impact Report에 명시한다.
3. 의미 있는 원칙 변경은 팀(또는 프로젝트 오너)의 명시적 승인이 필요하다.

**버전 관리 정책**:
- **MAJOR**: 원칙 삭제 또는 비호환 재정의.
- **MINOR**: 새 원칙 또는 섹션 추가, 실질적 가이던스 확장.
- **PATCH**: 명확화, 표현 수정, 오타 수정.

**준수 검토**:
- 모든 PR은 Constitution Check를 통과해야 한다.
- 복잡도 증가는 Complexity Tracking 표에 정당화 사유를 기록한다.
- 이 헌법에 위반되는 코드는 병합이 거부된다.

**Version**: 1.1.0 | **Ratified**: 2026-03-20 | **Last Amended**: 2026-03-20
