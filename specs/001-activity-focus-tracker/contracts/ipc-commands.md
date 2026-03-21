# IPC 커맨드 계약: Activity Focus Tracker

**브랜치**: `001-activity-focus-tracker`
**날짜**: 2026-03-20

모든 커맨드는 Tauri `invoke()` 프로토콜을 따른다.
- 성공: `Result<T>` → TypeScript `Promise<T>`
- 실패: `Result<_, AppError>` → TypeScript `Promise<never>` (catch로 처리)

프론트엔드는 `src/services/` 래퍼를 통해서만 호출한다 (헌법 I 원칙).

---

## 세션 커맨드 (`commands/session.rs`)

### `start_session`

```
입력: 없음
출력: Session
에러: SessionAlreadyActive
```

세션을 시작한다. 이미 활성 세션이 있으면 `SessionAlreadyActive` 에러를 반환한다.

---

### `end_session`

```
입력: 없음
출력: Session  (ended_at, is_complete=true 포함)
에러: NoActiveSession
```

현재 활성 세션을 종료한다. 마지막 진행 중 activity의 duration을 세션 종료 시각 기준으로 계산하여 기록한다.

---

### `get_current_session`

```
입력: 없음
출력: Option<Session>
에러: 없음
```

현재 활성 세션을 반환한다. 세션이 없으면 `null`을 반환한다.

---

### `get_incomplete_session`

```
입력: 없음
출력: Option<Session>
에러: 없음
```

앱 재시작 시 미완료 세션(크래시된 세션)을 반환한다. 없으면 `null`.

---

### `resume_session`

```
입력: { session_id: string }
출력: Session
에러: NotFound, SessionAlreadyActive
```

미완료 세션을 재개한다 (트래커 재시작).

---

### `discard_incomplete_session`

```
입력: { session_id: string }
출력: Session  (is_complete=false, ended_at=마지막_activity_timestamp)
에러: NotFound
```

미완료 세션을 크래시 종료로 처리한다.

---

## 활동 커맨드 (`commands/activity.rs`)

### `get_recent_activities`

```
입력: { limit: number }   -- 드롭다운용, 보통 3
출력: Activity[]
에러: NoActiveSession
```

현재 세션의 최근 활동 목록을 반환한다.

---

### `get_focus_metrics`

```
입력: { session_id: string }
출력: FocusMetrics
에러: NotFound
```

지정 세션의 Focus Metrics를 반환한다.

---

### `get_activity_timeline`

```
입력: { date: string }    -- YYYY-MM-DD
출력: Activity[] | ArchivedDailySummary
에러: 없음
```

날짜가 보관 기간 내이면 `Activity[]`, 초과이면 `ArchivedDailySummary`를 반환한다.

---

### `get_top_sites`

```
입력: { date: string, limit: number }
출력: DomainSummary[]
에러: 없음
```

날짜 기준 사용 시간 상위 도메인 목록을 반환한다.

---

## 레퍼런스 커맨드 (`commands/reference.rs`)

### `save_reference`

```
입력: SaveReferenceInput { url: string, title: string, tags: string[] }
출력: Reference
에러: NoActiveSession
```

현재 세션에 레퍼런스를 저장한다.

---

### `get_references`

```
입력: { session_id?: string }   -- null이면 전체
출력: Reference[]
에러: 없음
```

---

## 설정 커맨드 (`commands/settings.rs`)

### `get_settings`

```
입력: 없음
출력: AppSettings
에러: 없음
```

### `update_settings`

```
입력: AppSettings { raw_data_retention_days: number }
출력: AppSettings
에러: 없음
```

---

## TypeScript 서비스 레이어 (`src/services/`)

```typescript
// src/services/session.ts
import { invoke } from '@tauri-apps/api/core';
import type { Session, AppError } from '../types/bindings';

export const startSession = (): Promise<Session> =>
  invoke('start_session');

export const endSession = (): Promise<Session> =>
  invoke('end_session');

export const getCurrentSession = (): Promise<Session | null> =>
  invoke('get_current_session');

export const getIncompleteSession = (): Promise<Session | null> =>
  invoke('get_incomplete_session');

export const resumeSession = (sessionId: string): Promise<Session> =>
  invoke('resume_session', { sessionId });

export const discardIncompleteSession = (sessionId: string): Promise<Session> =>
  invoke('discard_incomplete_session', { sessionId });
```

---

## 에러 타입 (`AppError`)

```typescript
// src/types/bindings.ts (tauri-specta 자동 생성)
export type AppError =
  | { kind: 'Database'; message: string }
  | { kind: 'PermissionDenied'; message: string }
  | { kind: 'SessionAlreadyActive' }
  | { kind: 'NoActiveSession' }
  | { kind: 'NotFound'; message: string }
  | { kind: 'Internal'; message: string };
```
