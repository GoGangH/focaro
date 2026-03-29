-- V8: 일일 목표 달성 이력 테이블
CREATE TABLE IF NOT EXISTS goal_history (
    date        TEXT PRIMARY KEY,  -- YYYY-MM-DD
    target_secs INTEGER NOT NULL,
    actual_secs INTEGER NOT NULL,
    achieved    INTEGER NOT NULL DEFAULT 0  -- BOOL: 0 or 1
);
