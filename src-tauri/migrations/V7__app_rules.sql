CREATE TABLE IF NOT EXISTS app_rules (
  id       INTEGER PRIMARY KEY AUTOINCREMENT,
  app_name TEXT    NOT NULL UNIQUE,
  category TEXT    NOT NULL CHECK (category IN ('Focus', 'Neutral', 'Distraction'))
);

CREATE INDEX IF NOT EXISTS idx_app_rules_app_name ON app_rules(app_name);
