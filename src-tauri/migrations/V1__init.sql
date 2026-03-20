-- V1: 초기 스키마 — sessions, activities, classification_rules, settings

CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT    PRIMARY KEY,
    started_at  INTEGER NOT NULL,
    ended_at    INTEGER,
    is_complete INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS activities (
    id              TEXT    PRIMARY KEY,
    session_id      TEXT    NOT NULL,
    app_name        TEXT    NOT NULL,
    url             TEXT,
    domain          TEXT,
    classification  TEXT    NOT NULL
                    CHECK (classification IN ('Focus', 'Neutral', 'Distraction')),
    started_at      INTEGER NOT NULL,
    duration_secs   INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_activities_session_id ON activities(session_id);
CREATE INDEX IF NOT EXISTS idx_activities_domain     ON activities(domain);
CREATE INDEX IF NOT EXISTS idx_activities_started_at ON activities(started_at);

CREATE TABLE IF NOT EXISTS classification_rules (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    domain    TEXT    NOT NULL UNIQUE,
    category  TEXT    NOT NULL
              CHECK (category IN ('Focus', 'Neutral', 'Distraction'))
);

INSERT OR IGNORE INTO classification_rules (domain, category) VALUES
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

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO settings (key, value) VALUES ('retention_days', '30');
