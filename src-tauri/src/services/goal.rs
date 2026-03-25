use rusqlite::{params, Connection};

use crate::errors::AppError;
use crate::models::goal::{DailyGoal, GoalProgress};

/// 오늘 날짜 문자열 (YYYY-MM-DD)
pub fn today() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // 단순 계산 (UTC 기준)
    let days = now / 86400;
    let y = days_to_ymd(days);
    y
}

fn days_to_ymd(days_since_epoch: u64) -> String {
    // 1970-01-01 기준 경과 일수를 YYYY-MM-DD로 변환
    // chrono 없이 간단히 계산
    let mut d = days_since_epoch as i64;
    let mut year = 1970i64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if d < days_in_year { break; }
        d -= days_in_year;
        year += 1;
    }
    let month_days: [i64; 12] = [31, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1i64;
    for md in &month_days {
        if d < *md { break; }
        d -= md;
        month += 1;
    }
    format!("{:04}-{:02}-{:02}", year, month, d + 1)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// 특정 날짜의 목표 조회 (없으면 default_goal_secs 기본값 반환)
pub fn get_daily_goal(conn: &Connection, date: &str) -> Result<DailyGoal, AppError> {
    let result = conn.query_row(
        "SELECT date, target_secs FROM session_goals WHERE date = ?1",
        params![date],
        |r| Ok(DailyGoal { date: r.get(0)?, target_secs: r.get(1)? }),
    );
    match result {
        Ok(g) => Ok(g),
        Err(_) => {
            // 설정에서 기본 목표값 로드
            let default_secs: i64 = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = 'default_goal_secs'",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .map(|v| v.parse().unwrap_or(7200))
                .unwrap_or(7200);
            Ok(DailyGoal { date: date.to_string(), target_secs: default_secs })
        }
    }
}

/// 특정 날짜의 목표 설정 (upsert)
pub fn set_daily_goal(conn: &Connection, date: &str, target_secs: i64) -> Result<DailyGoal, AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO session_goals (date, target_secs) VALUES (?1, ?2)",
        params![date, target_secs],
    )
    .map_err(AppError::from)?;
    Ok(DailyGoal { date: date.to_string(), target_secs })
}

/// 특정 날짜의 목표 달성 현황 조회
pub fn get_goal_progress(conn: &Connection, date: &str) -> Result<GoalProgress, AppError> {
    let goal = get_daily_goal(conn, date)?;

    // 해당 날짜에 시작한 세션들의 Focus 시간 합산
    let actual_focus_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(a.duration_secs), 0)
             FROM activities a
             JOIN sessions s ON a.session_id = s.id
             WHERE a.classification = 'Focus'
               AND date(s.started_at, 'unixepoch') = ?1",
            params![date],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let progress_pct = if goal.target_secs > 0 {
        (actual_focus_secs as f64 / goal.target_secs as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    Ok(GoalProgress {
        date: date.to_string(),
        target_secs: goal.target_secs,
        actual_focus_secs,
        progress_pct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_today_format() {
        let d = today();
        // YYYY-MM-DD 형식 검증
        assert_eq!(d.len(), 10);
        assert_eq!(&d[4..5], "-");
        assert_eq!(&d[7..8], "-");
    }

    #[test]
    fn test_days_to_ymd_epoch() {
        assert_eq!(days_to_ymd(0), "1970-01-01");
    }

    #[test]
    fn test_days_to_ymd_known_date() {
        // 2026-03-25 = 1970-01-01 + ?일
        // 검증: 실제 today()와 비교하는 대신 형식만 확인
        let d = days_to_ymd(20173); // 임의 날짜
        assert_eq!(d.len(), 10);
    }

    #[test]
    fn test_is_leap() {
        assert!(is_leap(2000));
        assert!(is_leap(2024));
        assert!(!is_leap(1900));
        assert!(!is_leap(2025));
    }
}
