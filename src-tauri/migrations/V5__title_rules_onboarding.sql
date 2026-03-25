-- V5: title_rules 테이블 + 온보딩 설정 추가

CREATE TABLE IF NOT EXISTS title_rules (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    domain   TEXT NOT NULL,
    keyword  TEXT NOT NULL,  -- 소문자 변환 후 title contains 매칭
    category TEXT NOT NULL
             CHECK (category IN ('Focus', 'Neutral', 'Distraction'))
);

CREATE INDEX IF NOT EXISTS idx_title_rules_domain ON title_rules(domain);

INSERT OR IGNORE INTO settings (key, value) VALUES ('onboarding_completed', 'false');
INSERT OR IGNORE INTO settings (key, value) VALUES ('shortcut_session_start', 'CmdOrCtrl+Shift+S');
INSERT OR IGNORE INTO settings (key, value) VALUES ('shortcut_session_end', 'CmdOrCtrl+Shift+E');
