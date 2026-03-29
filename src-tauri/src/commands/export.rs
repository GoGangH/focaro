use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

use crate::errors::AppError;
use crate::services::{activity as activity_svc, db::DbPool};
use crate::state::app_state::AppState;

fn get_pool(state: &State<'_, Mutex<AppState>>) -> DbPool {
    state.lock().unwrap().db_pool.clone()
}

fn format_csv(rows: &[activity_svc::ActivityRow]) -> String {
    let mut out = String::from(
        "date,app_name,domain,title,classification,duration_secs,url\n",
    );
    for r in rows {
        let domain = r.domain.as_deref().unwrap_or("");
        let url = r.url.as_deref().unwrap_or("");
        let title = r.title.as_deref().unwrap_or("");
        let duration = r.duration_secs.unwrap_or(0);
        // Escape fields that may contain commas/quotes
        let escape = |s: &str| {
            if s.contains(',') || s.contains('"') || s.contains('\n') {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.to_string()
            }
        };
        out.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            &r.started_at[..10],
            escape(&r.app_name),
            escape(domain),
            escape(title),
            r.classification,
            duration,
            escape(url),
        ));
    }
    out
}

fn format_json(rows: &[activity_svc::ActivityRow]) -> Result<String, AppError> {
    #[derive(serde::Serialize)]
    struct ExportRow<'a> {
        date: &'a str,
        app_name: &'a str,
        domain: Option<&'a str>,
        title: Option<&'a str>,
        classification: &'a str,
        duration_secs: Option<i64>,
        url: Option<&'a str>,
    }

    let items: Vec<ExportRow> = rows
        .iter()
        .map(|r| ExportRow {
            date: &r.started_at[..10],
            app_name: &r.app_name,
            domain: r.domain.as_deref(),
            title: r.title.as_deref(),
            classification: &r.classification,
            duration_secs: r.duration_secs,
            url: r.url.as_deref(),
        })
        .collect();

    serde_json::to_string_pretty(&items).map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn export_data(
    start_date: String,
    end_date: String,
    format: String,
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
) -> Result<String, AppError> {
    let pool = get_pool(&state);
    let rows = activity_svc::query_export_activities(&pool, &start_date, &end_date)?;

    if rows.is_empty() {
        return Err(AppError::NotFound("해당 기간에 활동 데이터가 없습니다".into()));
    }

    let (content, ext) = match format.as_str() {
        "json" => (format_json(&rows)?, "json"),
        _ => (format_csv(&rows), "csv"),
    };

    let downloads = app
        .path()
        .download_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("focaro-export-{}-to-{}-{}.{}", start_date, end_date, timestamp, ext);
    let full_path = downloads.join(&filename);

    std::fs::write(&full_path, content)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(full_path.to_string_lossy().to_string())
}
