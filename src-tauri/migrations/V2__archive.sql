-- V2: 아카이브 및 references 테이블 추가

CREATE TABLE IF NOT EXISTS archived_daily_summaries (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    date                TEXT    NOT NULL UNIQUE,
    total_secs          INTEGER NOT NULL DEFAULT 0,
    focus_secs          INTEGER NOT NULL DEFAULT 0,
    neutral_secs        INTEGER NOT NULL DEFAULT 0,
    distraction_secs    INTEGER NOT NULL DEFAULT 0,
    top_domains_json    TEXT    NOT NULL DEFAULT '[]'
);

CREATE TABLE IF NOT EXISTS "references" (
    id          TEXT    PRIMARY KEY,
    session_id  TEXT    NOT NULL,
    url         TEXT    NOT NULL,
    title       TEXT    NOT NULL,
    tags        TEXT    NOT NULL DEFAULT '[]',
    created_at  INTEGER NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_references_session_id ON "references"(session_id);
