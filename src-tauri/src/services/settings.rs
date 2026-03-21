use rusqlite::{params, Connection};

use crate::errors::AppError;
use crate::models::settings::{AppSettings, ClassificationRule};

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

    Ok(AppSettings {
        retention_days,
        shortcut_save_ref,
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
