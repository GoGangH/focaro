use rusqlite::{params, Connection};

use crate::errors::AppError;
use crate::models::onboarding::{Profession, TitleRule, profession_domain_rules};
use crate::services::db::DbPool;

/// 온보딩 완료 여부 조회
pub fn is_onboarding_completed(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'onboarding_completed'",
        [],
        |r| r.get::<_, String>(0),
    )
    .map(|v| v == "true")
    .unwrap_or(false)
}

/// 온보딩 완료 처리
pub fn complete_onboarding(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('onboarding_completed', 'true')",
        [],
    )
    .map_err(AppError::from)?;
    Ok(())
}

/// 직업 기반 사전 설정 규칙 적용 (기존 규칙 유지 + 신규 추가)
pub fn apply_profession_rules(pool: &DbPool, profession: &Profession) -> Result<(), AppError> {
    let conn = pool.get().map_err(AppError::from)?;
    let rules = profession_domain_rules(profession);
    for (domain, category) in rules {
        conn.execute(
            "INSERT OR IGNORE INTO classification_rules (domain, category) VALUES (?1, ?2)",
            params![domain, category],
        )
        .map_err(AppError::from)?;
    }
    Ok(())
}

/// title_rule 추가
pub fn add_title_rule(
    conn: &Connection,
    domain: &str,
    keyword: &str,
    category: &str,
) -> Result<TitleRule, AppError> {
    conn.execute(
        "INSERT INTO title_rules (domain, keyword, category) VALUES (?1, ?2, ?3)",
        params![domain, keyword, category],
    )
    .map_err(AppError::from)?;

    let id = conn.last_insert_rowid();
    Ok(TitleRule {
        id,
        domain: domain.to_string(),
        keyword: keyword.to_string(),
        category: category.to_string(),
    })
}

/// title_rule 목록 조회
pub fn get_title_rules(conn: &Connection) -> Result<Vec<TitleRule>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, domain, keyword, category FROM title_rules ORDER BY id")
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map([], |r| {
            Ok(TitleRule {
                id: r.get(0)?,
                domain: r.get(1)?,
                keyword: r.get(2)?,
                category: r.get(3)?,
            })
        })
        .map_err(AppError::from)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// title_rule 삭제
pub fn delete_title_rule(conn: &Connection, id: i64) -> Result<(), AppError> {
    let affected = conn
        .execute("DELETE FROM title_rules WHERE id = ?1", params![id])
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(format!("title_rule id={id} 없음")));
    }
    Ok(())
}

/// 현재 활동의 분류를 즉시 변경 (이번 세션만)
/// activity_id에 해당하는 activity의 classification 업데이트
pub fn override_activity_classification(
    conn: &Connection,
    activity_id: &str,
    category: &str,
) -> Result<(), AppError> {
    let affected = conn
        .execute(
            "UPDATE activities SET classification = ?1 WHERE id = ?2",
            params![category, activity_id],
        )
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(format!("activity id={activity_id} 없음")));
    }
    Ok(())
}
