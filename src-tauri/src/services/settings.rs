use rusqlite::{params, Connection};

use crate::errors::AppError;
use crate::models::settings::{AppSettings, ClassificationRule};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AppRule {
    pub id: i64,
    pub app_name: String,
    pub category: String,
}

pub fn get_settings(conn: &Connection) -> Result<AppSettings, AppError> {
    let retention_days: i64 = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'retention_days'",
            [],
            |r| r.get::<_, String>(0),
        )
        .map(|v| v.parse().unwrap_or(30))
        .unwrap_or(30);

    let shortcut_save_ref: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'shortcut_save_ref'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "CmdOrCtrl+Shift+R".to_string());

    let shortcut_session_start: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'shortcut_session_start'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "CmdOrCtrl+Shift+S".to_string());

    let shortcut_session_end: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'shortcut_session_end'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "CmdOrCtrl+Shift+E".to_string());

    Ok(AppSettings {
        retention_days,
        shortcut_save_ref,
        shortcut_session_start,
        shortcut_session_end,
    })
}

pub fn update_settings(conn: &Connection, settings: &AppSettings) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('retention_days', ?1)",
        params![settings.retention_days.to_string()],
    )
    .map_err(AppError::from)?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('shortcut_save_ref', ?1)",
        params![settings.shortcut_save_ref],
    )
    .map_err(AppError::from)?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('shortcut_session_start', ?1)",
        params![settings.shortcut_session_start],
    )
    .map_err(AppError::from)?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('shortcut_session_end', ?1)",
        params![settings.shortcut_session_end],
    )
    .map_err(AppError::from)?;

    Ok(())
}

pub fn get_classification_rules(conn: &Connection) -> Result<Vec<ClassificationRule>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, domain, category FROM classification_rules ORDER BY id")
        .map_err(AppError::from)?;

    let rules = stmt
        .query_map([], |r| {
            Ok(ClassificationRule {
                id: r.get(0)?,
                domain: r.get(1)?,
                category: r.get(2)?,
            })
        })
        .map_err(AppError::from)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rules)
}

pub fn add_classification_rule(
    conn: &Connection,
    domain: &str,
    category: &str,
) -> Result<ClassificationRule, AppError> {
    conn.execute(
        "INSERT INTO classification_rules (domain, category) VALUES (?1, ?2)",
        params![domain, category],
    )
    .map_err(AppError::from)?;

    let id = conn.last_insert_rowid();
    Ok(ClassificationRule {
        id,
        domain: domain.to_string(),
        category: category.to_string(),
    })
}

pub fn delete_classification_rule(conn: &Connection, id: i64) -> Result<(), AppError> {
    let affected = conn
        .execute("DELETE FROM classification_rules WHERE id = ?1", params![id])
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(format!("규칙 id={id} 없음")));
    }
    Ok(())
}

pub fn get_app_rules(conn: &Connection) -> Result<Vec<AppRule>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, app_name, category FROM app_rules ORDER BY app_name ASC")
        .map_err(AppError::from)?;

    let rules = stmt
        .query_map([], |r| {
            Ok(AppRule {
                id: r.get(0)?,
                app_name: r.get(1)?,
                category: r.get(2)?,
            })
        })
        .map_err(AppError::from)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rules)
}

pub fn add_app_rule(
    conn: &Connection,
    app_name: &str,
    category: &str,
) -> Result<AppRule, AppError> {
    conn.execute(
        "INSERT INTO app_rules (app_name, category) VALUES (?1, ?2)
         ON CONFLICT(app_name) DO UPDATE SET category = excluded.category",
        params![app_name, category],
    )
    .map_err(AppError::from)?;

    let id = conn.last_insert_rowid();
    Ok(AppRule {
        id,
        app_name: app_name.to_string(),
        category: category.to_string(),
    })
}

pub fn delete_app_rule(conn: &Connection, id: i64) -> Result<(), AppError> {
    let affected = conn
        .execute("DELETE FROM app_rules WHERE id = ?1", params![id])
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(format!("앱 규칙 id={id} 없음")));
    }
    Ok(())
}
