-- V6: 일일 집중 목표 테이블 추가
CREATE TABLE IF NOT EXISTS session_goals (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    date        TEXT NOT NULL UNIQUE,  -- YYYY-MM-DD
    target_secs INTEGER NOT NULL DEFAULT 7200  -- 기본 2시간
);

INSERT OR IGNORE INTO settings (key, value) VALUES ('default_goal_secs', '7200');
INSERT OR IGNORE INTO settings (key, value) VALUES ('notify_low_focus_threshold', '40');
INSERT OR IGNORE INTO settings (key, value) VALUES ('notify_enabled', 'true');
