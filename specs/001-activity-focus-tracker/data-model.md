# 데이터 모델: Activity Focus Tracker

**브랜치**: `001-activity-focus-tracker`
**날짜**: 2026-03-20

---

## 엔티티 관계도

```
Session 1 ──< Activity (n)
Session 1 ──< Reference (n)
ClassificationRule (독립)
AppSettings (독립, 단일 행 키-값)
ArchivedDailySummary (독립, 날짜별 집계)
```

---

## SQLite 스키마

### sessions

```sql
CREATE TABLE sessions (
    id          TEXT    PRIMARY KEY,              -- UUID v4
    started_at  INTEGER NOT NULL,                 -- Unix timestamp (초)
    ended_at    INTEGER,                          -- NULL = 진행 중
    is_complete INTEGER NOT NULL DEFAULT 0        -- 0=미완료, 1=정상종료
);
```

**상태 전이**:
- `ended_at IS NULL AND is_complete = 0` → 진행 중 (또는 크래시 미완료)
- `ended_at IS NOT NULL AND is_complete = 1` → 정상 종료
- `ended_at IS NOT NULL AND is_complete = 0` → 크래시 종료 (사용자가 "종료" 선택)

---

### activities

```sql
CREATE TABLE activities (
    id              TEXT    PRIMARY KEY,           -- UUID v4
    session_id      TEXT    NOT NULL,
    app_name        TEXT    NOT NULL,
    url             TEXT,                          -- NULL = 비브라우저 앱 또는 권한 거부
    domain          TEXT,                          -- NULL = 비브라우저 앱 또는 권한 거부
    classification  TEXT    NOT NULL
                    CHECK (classification IN ('Focus', 'Neutral', 'Distraction')),
    started_at      INTEGER NOT NULL,              -- Unix timestamp (초)
    duration_secs   INTEGER,                       -- NULL = 현재 진행 중인 마지막 활동
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_activities_session_id ON activities(session_id);
CREATE INDEX idx_activities_domain ON activities(domain);
CREATE INDEX idx_activities_started_at ON activities(started_at);
```

**동일 활동 판단 기준**: `app_name` + `url` (url이 NULL이면 `app_name`만 비교)

---

### classification_rules

```sql
CREATE TABLE classification_rules (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    domain    TEXT    NOT NULL UNIQUE,
    category  TEXT    NOT NULL
              CHECK (category IN ('Focus', 'Neutral', 'Distraction'))
);

-- 기본 규칙 (초기 마이그레이션에서 삽입)
INSERT INTO classification_rules (domain, category) VALUES
    ('github.com',          'Focus'),
    ('stackoverflow.com',   'Focus'),
    ('docs.rs',             'Focus'),
    ('developer.apple.com', 'Focus'),
    ('figma.com',           'Focus'),
    ('notion.so',           'Focus'),
    ('linear.app',          'Focus'),
    ('youtube.com',         'Distraction'),
    ('twitter.com',         'Distraction'),
    ('x.com',               'Distraction'),
    ('instagram.com',       'Distraction'),
    ('reddit.com',          'Distraction'),
    ('facebook.com',        'Distraction'),
    ('tiktok.com',          'Distraction'),
    ('netflix.com',         'Distraction');
```

---

### references

```sql
CREATE TABLE references (
    id          TEXT    PRIMARY KEY,               -- UUID v4
    session_id  TEXT    NOT NULL,
    url         TEXT    NOT NULL,
    title       TEXT    NOT NULL,
    tags        TEXT    NOT NULL DEFAULT '[]',     -- JSON 배열 문자열
    created_at  INTEGER NOT NULL,                  -- Unix timestamp (초)
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_references_session_id ON references(session_id);
```

---

### archived_daily_summaries

```sql
CREATE TABLE archived_daily_summaries (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    date                TEXT    NOT NULL UNIQUE,   -- ISO 날짜 YYYY-MM-DD
    total_secs          INTEGER NOT NULL DEFAULT 0,
    focus_secs          INTEGER NOT NULL DEFAULT 0,
    neutral_secs        INTEGER NOT NULL DEFAULT 0,
    distraction_secs    INTEGER NOT NULL DEFAULT 0,
    top_domains         TEXT    NOT NULL DEFAULT '[]'
    -- JSON: [{"domain":"github.com","secs":3600,"category":"Focus"}, ...]
);
```

---

### settings

```sql
CREATE TABLE settings (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);

-- 기본 설정 (초기 마이그레이션)
INSERT INTO settings (key, value) VALUES
    ('raw_data_retention_days', '30');
```

---

## Rust 구조체 (models/)

### Session

```rust
// src-tauri/src/models/session.rs
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Session {
    pub id: String,           // UUID
    pub started_at: i64,      // Unix timestamp
    pub ended_at: Option<i64>,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum SessionStatus {
    Active,
    Completed,
    CrashedIncomplete,
}
```

### Activity

```rust
// src-tauri/src/models/activity.rs
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Activity {
    pub id: String,
    pub session_id: String,
    pub app_name: String,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub classification: Classification,
    pub started_at: i64,
    pub duration_secs: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum Classification {
    Focus,
    Neutral,
    Distraction,
}
```

### FocusMetrics

```rust
// src-tauri/src/models/metrics.rs
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FocusMetrics {
    pub session_id: String,
    pub total_secs: i64,
    pub focus_secs: i64,
    pub neutral_secs: i64,
    pub distraction_secs: i64,
    pub focus_pct: f64,       // 0.0 ~ 100.0
    pub neutral_pct: f64,
    pub distraction_pct: f64,
}
```

### Reference

```rust
// src-tauri/src/models/reference.rs
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Reference {
    pub id: String,
    pub session_id: String,
    pub url: String,
    pub title: String,
    pub tags: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize, Type)]
pub struct SaveReferenceInput {
    pub url: String,
    pub title: String,
    pub tags: Vec<String>,
}
```

### ArchivedDailySummary

```rust
// src-tauri/src/models/archive.rs
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ArchivedDailySummary {
    pub date: String,             // YYYY-MM-DD
    pub total_secs: i64,
    pub focus_secs: i64,
    pub neutral_secs: i64,
    pub distraction_secs: i64,
    pub top_domains: Vec<DomainSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DomainSummary {
    pub domain: String,
    pub secs: i64,
    pub category: Classification,
}
```

### AppSettings

```rust
// src-tauri/src/models/settings.rs
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppSettings {
    pub raw_data_retention_days: u32,   // 기본값: 30
}
```

### AppError

```rust
// src-tauri/src/errors.rs
#[derive(Debug, Serialize, Type)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    Database(String),
    PermissionDenied(String),
    SessionAlreadyActive,
    NoActiveSession,
    NotFound(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AppError {}
```

---

## 마이그레이션 파일

```
src-tauri/migrations/
├── V1__init.sql          # sessions, activities, classification_rules, settings 기본 스키마 + 초기 데이터
└── V2__archive.sql       # archived_daily_summaries, references 스키마
```

---

## 비즈니스 규칙 요약

| 규칙 | 설명 |
|------|------|
| 분류 기본값 | domain이 없거나 규칙에 없으면 `Neutral` |
| duration 계산 | 활동 변경 시: `현재 timestamp - 이전 started_at`. 세션 종료 시: `ended_at - 마지막 started_at` |
| 퍼센트 계산 | `total_secs == 0`이면 모든 퍼센트 0.0 반환 (0 나누기 방지) |
| 아카이빙 트리거 | 앱 시작 시 보관 기간 초과 날짜를 집계 후 원본 삭제 |
| 세션 상태 | 동시에 하나의 세션만 `ended_at IS NULL` 가능 |
