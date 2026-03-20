# Focaro 구현 작업 순서

> 이 문서는 매 기능 작업 시 참고하는 체크리스트입니다.
> 순서를 반드시 지켜야 하며, 이슈 없이 작업을 시작할 수 없습니다.

---

## 브랜치 구조

```
main        ← 프로덕션 (릴리즈 태그만)
  └─ develop  ← 통합 브랜치 (모든 기능 PR 대상)
       └─ feat/#12-session-management  ← 기능 브랜치
       └─ fix/#34-tracker-bug          ← 버그 수정 브랜치
       └─ chore/#5-project-setup       ← 설정/문서 브랜치
```

---

## 작업 시작 전 (매번 필수)

### 1. GitHub Issue 생성

```
제목: [feat] 세션 관리 기능 구현
내용:
  ## 개요
  (작업 내용 요약)

  ## 관련 문서
  - specs/001-activity-focus-tracker/spec.md
  - specs/001-activity-focus-tracker/tasks.md (T001~T027)

  ## 완료 조건
  - [ ] cargo test 통과
  - [ ] npm test 통과
```

### 2. develop에서 브랜치 생성

```bash
git checkout develop
git pull origin develop
git checkout -b feat/#12-session-management
```

브랜치 타입:
| 타입 | 용도 | 분기 기준 |
|------|------|-----------|
| `feat/` | 새 기능 | develop |
| `fix/` | 버그 수정 | develop |
| `hotfix/` | 긴급 수정 | main |
| `chore/` | 설정/문서 | develop |
| `refactor/` | 리팩터 | develop |
| `test/` | 테스트 추가 | develop |

---

## 개발 사이클 (TDD 필수)

### 3. 테스트 먼저 작성 (Red)

```bash
# Rust 단위 테스트
# src-tauri/src/services/classifier.rs 내 #[cfg(test)] 블록 작성

# 테스트 실패 확인 (반드시 FAILED 확인 후 구현 시작)
cd src-tauri && cargo test
```

### 4. 구현 (Green)

```bash
# 테스트를 통과하는 최소 코드 작성
# 구현 후 테스트 통과 확인
cargo test
npm test
```

### 5. 리팩터

```bash
# 중복 제거, any 타입 제거, unwrap() 제거
cargo test && npm test  # 리팩터 후 재확인
```

---

## 커밋

```bash
# 커밋 메시지 형식
git add <파일>
git commit -m "feat(#12): 세션 시작/종료 커맨드 구현"

# 타입별 예시
feat(#12): 세션 시작/종료 커맨드 구현
test(#12): session 커맨드 단위 테스트 작성 (Red)
fix(#34): 트래커 2초 폴링 타이머 누수 수정
chore(#5): Cargo.toml 의존성 추가
refactor(#20): classifier 서비스 DB 쿼리 최적화
```

---

## Pull Request 제출

### 6. PR 생성

```bash
git push -u origin feat/#12-session-management

gh pr create \
  --base develop \
  --title "[#12] 세션 관리 기능 구현" \
  --body "## 변경 내용
- start_session, end_session 커맨드 구현
- 2초 폴링 트래커 구현

## 테스트
- \`cargo test\` 통과
- \`npm test\` 통과

## Constitution Check
- [x] 연결된 GitHub Issue 존재 (#12)
- [x] 브랜치 이름 형식 준수
- [x] 테스트 먼저 작성 (커밋 히스토리 확인 가능)
- [x] any 타입 미사용
- [x] unwrap() 프로덕션 코드 미사용
- [x] 에러 타입 serde::Serialize 구현
- [x] 중복 코드 없음
- [x] YAGNI 준수

Closes #12"
```

### 7. PR 병합 후 정리

```bash
git checkout develop
git pull origin develop
git branch -d feat/#12-session-management
```

---

## 릴리즈 (develop → main)

```bash
# develop에서 충분히 검증 후
gh pr create \
  --base main \
  --head develop \
  --title "Release v0.1.0" \
  --body "## 포함 기능
- #12 세션 관리
- #13 활동 트래킹
- #14 Focus Metrics

Closes #12
Closes #13
Closes #14"

# 병합 후 태그
git checkout main && git pull
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

---

## 현재 피처 tasks.md 기준 작업 순서

`specs/001-activity-focus-tracker/tasks.md` 참조.
각 Phase는 GitHub Issue 1개에 대응하거나, 태스크 그룹별로 이슈를 분리할 수 있습니다.

### 권장 이슈 분리 기준

| 이슈 | 범위 | tasks.md |
|------|------|----------|
| #1 프로젝트 초기 설정 | Phase 1 | T001~T005 |
| #2 기반 인프라 구축 | Phase 2 | T006~T013 |
| #3 세션 관리 및 트래킹 (MVP) | Phase 3 (US1) | T014~T027 |
| #4 드롭다운 UI | Phase 4 (US2) | T028~T038 |
| #5 Focus Metrics 계산 | Phase 5 (US3) | T039~T044 |
| #6 Reference 저장 | Phase 6 (US4) | T045~T050 |
| #7 Dashboard | Phase 7 (US5) | T051~T061 |
| #8 설정 및 크래시 복구 | Phase 8 | T062~T065 |
| #9 권한 처리 및 마무리 | Phase 9 | T066~T070 |

---

## 빠른 참조

```bash
# 새 기능 시작
git checkout develop && git pull
git checkout -b feat/#{이슈번호}-{설명}

# 테스트 (Rust)
cd src-tauri && cargo test

# 테스트 (React)
npm test

# 개발 실행
cargo tauri dev

# PR 생성
gh pr create --base develop --title "[#{번호}] 제목" --body "Closes #{번호}"
```
