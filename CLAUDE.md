# focaro Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-03-20

## Active Technologies

- **Rust** stable 1.75+ (백엔드, `src-tauri/`)
- **TypeScript** 5.x + **React** 18 (프론트엔드, `src/`)
- **Tauri** v2 (데스크탑 프레임워크)
- **SQLite** via rusqlite 0.31 + r2d2 커넥션 풀
- **tauri-specta** v2 (Rust → TypeScript 타입 자동 생성)

## Project Structure

```text
src-tauri/src/
├── commands/    # Tauri IPC 핸들러 (#[tauri::command])
├── services/    # 비즈니스 로직 (commands와 분리)
├── models/      # serde::Serialize/Deserialize 데이터 구조체
├── state/       # tauri::State<Mutex<T>> 앱 상태
└── errors.rs    # AppError (serde::Serialize 구현)

src/
├── components/  # 재사용 UI 컴포넌트
├── pages/       # 페이지 단위 컴포넌트
├── hooks/       # 커스텀 React 훅
├── services/    # invoke() 래퍼 (직접 invoke 금지)
├── types/       # tauri-specta 자동 생성 바인딩
└── stores/      # 클라이언트 상태
```

## Commands

```bash
# 개발 실행
cargo tauri dev

# Rust 테스트
cd src-tauri && cargo test

# 프론트엔드 테스트
npm test

# 빌드
cargo tauri build
```

## Code Style

- **Rust**: `unwrap()` / `expect()` 프로덕션 코드 금지. 에러는 `AppError`로 반환
- **TypeScript**: `any` 타입 금지. `unknown` + 타입 가드 사용
- **IPC**: 프론트엔드에서 `invoke()` 직접 호출 금지 → `src/services/` 경유 필수
- **타입**: Rust 타입이 단일 출처. `src/types/bindings.ts`는 자동 생성, 수동 편집 금지

## Git 브랜치 원칙 (NON-NEGOTIABLE)

```
main ← develop ← feat/#{issue}-{name}
```

**흐름**: Issue 생성 → develop에서 분기 → 작업 → develop으로 PR → main은 릴리즈만

**브랜치 명명**:
```
feat/#12-session-management    (develop에서 분기)
fix/#34-tracker-cpu-spike      (develop에서 분기)
hotfix/#99-critical-fix        (main에서 분기 → main+develop PR)
chore/#5-project-setup
```

**커밋 메시지**:
```
feat(#12): 세션 시작/종료 커맨드 구현
test(#12): classifier 단위 테스트 작성
```

**PR 필수 요건**:
- 대상 브랜치: `develop` (릴리즈 시에만 `main`)
- 제목: `[#12] 세션 관리 기능 구현`
- 본문: `Closes #12` 또는 `Fixes #12` 포함

## Constitution Check (PR 게이트)

- [ ] 연결된 GitHub Issue가 존재하는가?
- [ ] 브랜치 이름이 `{type}/#{issue}-{description}` 형식인가?
- [ ] PR 제목에 이슈 번호가 포함되어 있는가?
- [ ] PR 본문에 `Closes #` 또는 `Fixes #`가 있는가?
- [ ] 모든 신규 기능에 테스트가 있는가?
- [ ] 테스트가 구현 전에 작성되었는가?
- [ ] `any` 타입 미사용?
- [ ] `unwrap()` / `expect()` 프로덕션 코드 미사용?
- [ ] 에러 타입이 `serde::Serialize` 구현?
- [ ] 중복 코드 없음?
- [ ] 현재 요구사항에만 필요한 구현인가?

## Recent Changes

- 001-activity-focus-tracker: 초기 피처 계획 수립 (2026-03-20)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
