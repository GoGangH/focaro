use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// macOS 알림 전송
pub fn send_notification(app: &AppHandle, title: &str, body: &str) {
    let result = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show();

    if let Err(e) = result {
        eprintln!("[notification] 전송 실패: {e}");
    }
}
