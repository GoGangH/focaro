use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use refinery::embed_migrations;
use rusqlite::Connection;
use std::path::Path;

use crate::errors::AppError;

embed_migrations!("migrations");

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn create_pool(db_path: &Path) -> Result<DbPool, AppError> {
    let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(())
    });
    let pool = Pool::new(manager).map_err(AppError::from)?;
    Ok(pool)
}

pub fn run_migrations(conn: &mut Connection) -> Result<(), AppError> {
    migrations::runner()
        .run(conn)
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    #[test]
    fn test_migrations_create_sessions_table() {
        let mut conn = Connection::open_in_memory().unwrap();
        super::run_migrations(&mut conn).expect("마이그레이션 실행 성공해야 함");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sessions'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "sessions 테이블이 존재해야 함");
    }

    #[test]
    fn test_migrations_create_activities_table() {
        let mut conn = Connection::open_in_memory().unwrap();
        super::run_migrations(&mut conn).expect("마이그레이션 실행 성공해야 함");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='activities'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "activities 테이블이 존재해야 함");
    }

    #[test]
    fn test_migrations_insert_default_classification_rules() {
        let mut conn = Connection::open_in_memory().unwrap();
        super::run_migrations(&mut conn).expect("마이그레이션 실행 성공해야 함");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM classification_rules", [], |r| r.get(0))
            .unwrap();
        assert!(count > 0, "기본 분류 규칙이 존재해야 함");
    }
}
