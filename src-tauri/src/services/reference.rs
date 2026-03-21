use crate::errors::AppError;
use crate::models::reference::{Reference, SaveReferenceInput};
use crate::services::db::DbPool;

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn unix_to_iso(unix: i64) -> String {
    chrono::DateTime::from_timestamp(unix, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_default()
}

pub fn save_reference(
    pool: &DbPool,
    session_id: &str,
    input: SaveReferenceInput,
) -> Result<Reference, AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_unix();
    let tags_json = serde_json::to_string(&input.tags.unwrap_or_default())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let conn = pool.get().map_err(AppError::from)?;
    conn.execute(
        r#"INSERT INTO "references" (id, session_id, url, title, tags, created_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
        rusqlite::params![id, session_id, input.url, input.title, tags_json, now],
    )
    .map_err(AppError::from)?;

    let tags: Vec<String> = serde_json::from_str(&tags_json)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Reference {
        id,
        session_id: session_id.to_string(),
        url: input.url,
        title: input.title,
        tags,
        created_at: unix_to_iso(now),
    })
}

pub fn get_references(
    pool: &DbPool,
    session_id: Option<&str>,
) -> Result<Vec<Reference>, AppError> {
    let conn = pool.get().map_err(AppError::from)?;

    let mut refs = Vec::new();

    if let Some(sid) = session_id {
        let mut stmt = conn
            .prepare(
                r#"SELECT id, session_id, url, title, tags, created_at
                   FROM "references" WHERE session_id = ?1
                   ORDER BY created_at DESC"#,
            )
            .map_err(AppError::from)?;

        let rows = stmt
            .query_map(rusqlite::params![sid], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, i64>(5)?,
                ))
            })
            .map_err(AppError::from)?;

        for row in rows {
            let (id, session_id, url, title, tags_json, created_at) =
                row.map_err(AppError::from)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            refs.push(Reference {
                id,
                session_id,
                url,
                title,
                tags,
                created_at: unix_to_iso(created_at),
            });
        }
    } else {
        let mut stmt = conn
            .prepare(
                r#"SELECT id, session_id, url, title, tags, created_at
                   FROM "references"
                   ORDER BY created_at DESC"#,
            )
            .map_err(AppError::from)?;

        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, i64>(5)?,
                ))
            })
            .map_err(AppError::from)?;

        for row in rows {
            let (id, session_id, url, title, tags_json, created_at) =
                row.map_err(AppError::from)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            refs.push(Reference {
                id,
                session_id,
                url,
                title,
                tags,
                created_at: unix_to_iso(created_at),
            });
        }
    }

    Ok(refs)
}

pub fn delete_reference(pool: &DbPool, id: &str) -> Result<(), AppError> {
    let conn = pool.get().map_err(AppError::from)?;
    let affected = conn
        .execute(r#"DELETE FROM "references" WHERE id = ?1"#, rusqlite::params![id])
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("reference {id} not found")));
    }
    Ok(())
}

pub fn update_reference(
    pool: &DbPool,
    id: &str,
    title: &str,
    tags: Vec<String>,
) -> Result<Reference, AppError> {
    let tags_json =
        serde_json::to_string(&tags).map_err(|e| AppError::Internal(e.to_string()))?;

    let conn = pool.get().map_err(AppError::from)?;
    let affected = conn
        .execute(
            r#"UPDATE "references" SET title = ?1, tags = ?2 WHERE id = ?3"#,
            rusqlite::params![title, tags_json, id],
        )
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("reference {id} not found")));
    }

    let (session_id, url, created_at): (String, String, i64) = conn
        .query_row(
            r#"SELECT session_id, url, created_at FROM "references" WHERE id = ?1"#,
            rusqlite::params![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(AppError::from)?;

    Ok(Reference {
        id: id.to_string(),
        session_id,
        url,
        title: title.to_string(),
        tags,
        created_at: unix_to_iso(created_at),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::db;
    use r2d2_sqlite::SqliteConnectionManager;

    fn setup_pool() -> DbPool {
        let manager = SqliteConnectionManager::memory();
        let pool = r2d2::Pool::new(manager).unwrap();
        let mut conn = pool.get().unwrap();
        db::run_migrations(&mut *conn).unwrap();
        drop(conn);
        pool
    }

    fn insert_session(pool: &DbPool, id: &str) {
        let now = now_unix();
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO sessions (id, started_at, ended_at, is_complete) VALUES (?1, ?2, NULL, 1)",
            rusqlite::params![id, now],
        )
        .unwrap();
    }

    #[test]
    fn test_save_reference_returns_reference() {
        let pool = setup_pool();
        insert_session(&pool, "sess-1");

        let input = SaveReferenceInput {
            url: "https://github.com".to_string(),
            title: "GitHub".to_string(),
            tags: Some(vec!["rust".to_string()]),
        };
        let r = save_reference(&pool, "sess-1", input).unwrap();

        assert!(!r.id.is_empty());
        assert_eq!(r.session_id, "sess-1");
        assert_eq!(r.url, "https://github.com");
        assert_eq!(r.title, "GitHub");
        assert_eq!(r.tags, vec!["rust"]);
        assert!(!r.created_at.is_empty());
    }

    #[test]
    fn test_save_reference_without_tags() {
        let pool = setup_pool();
        insert_session(&pool, "sess-1");

        let input = SaveReferenceInput {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            tags: None,
        };
        let r = save_reference(&pool, "sess-1", input).unwrap();
        assert!(r.tags.is_empty());
    }

    #[test]
    fn test_save_reference_multiple_tags() {
        let pool = setup_pool();
        insert_session(&pool, "sess-1");

        let input = SaveReferenceInput {
            url: "https://docs.rs".to_string(),
            title: "Rust Docs".to_string(),
            tags: Some(vec!["rust".to_string(), "docs".to_string()]),
        };
        let r = save_reference(&pool, "sess-1", input).unwrap();
        assert_eq!(r.tags.len(), 2);
        assert!(r.tags.contains(&"rust".to_string()));
        assert!(r.tags.contains(&"docs".to_string()));
    }

    #[test]
    fn test_get_references_by_session() {
        let pool = setup_pool();
        insert_session(&pool, "sess-1");
        insert_session(&pool, "sess-2");

        save_reference(
            &pool,
            "sess-1",
            SaveReferenceInput {
                url: "https://github.com".to_string(),
                title: "GitHub".to_string(),
                tags: None,
            },
        )
        .unwrap();
        save_reference(
            &pool,
            "sess-2",
            SaveReferenceInput {
                url: "https://google.com".to_string(),
                title: "Google".to_string(),
                tags: None,
            },
        )
        .unwrap();

        let refs_1 = get_references(&pool, Some("sess-1")).unwrap();
        assert_eq!(refs_1.len(), 1);
        assert_eq!(refs_1[0].url, "https://github.com");

        let refs_2 = get_references(&pool, Some("sess-2")).unwrap();
        assert_eq!(refs_2.len(), 1);
        assert_eq!(refs_2[0].url, "https://google.com");
    }

    #[test]
    fn test_get_references_all() {
        let pool = setup_pool();
        insert_session(&pool, "sess-1");
        insert_session(&pool, "sess-2");

        save_reference(
            &pool,
            "sess-1",
            SaveReferenceInput {
                url: "https://github.com".to_string(),
                title: "GitHub".to_string(),
                tags: None,
            },
        )
        .unwrap();
        save_reference(
            &pool,
            "sess-2",
            SaveReferenceInput {
                url: "https://google.com".to_string(),
                title: "Google".to_string(),
                tags: None,
            },
        )
        .unwrap();

        let all = get_references(&pool, None).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_get_references_empty_session() {
        let pool = setup_pool();
        insert_session(&pool, "sess-1");

        let refs = get_references(&pool, Some("sess-1")).unwrap();
        assert!(refs.is_empty());
    }

    #[test]
    fn test_get_references_ordered_by_created_at_desc() {
        let pool = setup_pool();
        insert_session(&pool, "sess-1");

        save_reference(
            &pool,
            "sess-1",
            SaveReferenceInput {
                url: "https://first.com".to_string(),
                title: "First".to_string(),
                tags: None,
            },
        )
        .unwrap();
        // 1초 대기 없이 순서를 보장하려면 직접 삽입
        {
            let conn = pool.get().unwrap();
            conn.execute(
                r#"INSERT INTO "references" (id, session_id, url, title, tags, created_at)
                   VALUES ('ref-2', 'sess-1', 'https://second.com', 'Second', '[]', 9999999999)"#,
                [],
            )
            .unwrap();
        }

        let refs = get_references(&pool, Some("sess-1")).unwrap();
        assert_eq!(refs[0].url, "https://second.com", "최신 항목이 첫 번째여야 함");
    }
}
