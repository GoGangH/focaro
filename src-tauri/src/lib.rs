// focaro — macOS 메뉴바 기반 활동 추적 집중 관리 도구

pub mod commands;
pub mod errors;
pub mod models;
pub mod services;
pub mod state;

use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Manager, WebviewWindow};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

use services::db;
use state::app_state::AppState;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("앱 데이터 디렉토리를 찾을 수 없습니다");
            std::fs::create_dir_all(&app_data_dir)?;

            let db_path = app_data_dir.join("focaro.db");
            let pool = db::create_pool(&db_path).expect("DB 연결 풀 생성 실패");

            let mut conn = pool.get().expect("DB 연결 획득 실패");
            db::run_migrations(&mut conn).expect("마이그레이션 실패");
            drop(conn);

            let app_state = AppState::new(pool);
            app.manage(Mutex::new(app_state));

            #[cfg(target_os = "macos")]
            request_accessibility_permission();

            let dropdown = app.get_webview_window("dropdown").unwrap();

            // 드롭다운 창 레벨 + collection behavior 설정 (setup에서 한 번)
            #[cfg(target_os = "macos")]
            init_dropdown_window(&dropdown);

            // shown_at: 트레이 클릭으로 창이 표시된 시각
            let shown_at: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));

            // 전역 마우스 모니터 — 반드시 main thread에서 등록해야 함
            // setup 클로저는 main thread에서 실행되므로 여기서 등록
            #[cfg(target_os = "macos")]
            register_global_mouse_monitor(dropdown.clone(), shown_at.clone());

            let tray_app = app.handle().clone();
            let shown_at_tray = shown_at.clone();
            let _tray = TrayIconBuilder::with_id("tray")
                .title("🔵")
                .on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        if let Some(window) = tray_app.get_webview_window("dropdown") {
                            if window.is_visible().unwrap_or(false) {
                                *shown_at_tray.lock().unwrap() = None;
                                let _ = window.hide();
                            } else {
                                *shown_at_tray.lock().unwrap() = Some(Instant::now());
                                let rect = tray.rect().ok().flatten();
                                show_dropdown(&window, rect);
                            }
                        }
                    }
                })
                .build(app)?;

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    let title = {
                        let state = app_handle.state::<Mutex<AppState>>();
                        let state = state.lock().unwrap();
                        let session_id = state.current_session_id.lock().unwrap().clone();
                        if let Some(sid) = session_id {
                            let pool = state.db_pool.clone();
                            drop(state);
                            compute_tray_title(&pool, &sid)
                        } else {
                            None
                        }
                    };
                    if let Some(tray) = app_handle.tray_by_id("tray") {
                        // 세션 없을 때 None → "🔵" 유지 (None이면 트레이 항목이 사라짐)
                        let display = title.unwrap_or_else(|| "🔵".to_string());
                        let _ = tray.set_title(Some(display.as_str()));
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session::start_session,
            commands::session::end_session,
            commands::session::get_current_session,
            commands::session::get_incomplete_session,
            commands::session::resume_session,
            commands::session::discard_incomplete_session,
            commands::session::open_dashboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 드롭다운을 트레이 아이콘 rect 바로 아래 중앙에 표시
/// NSWindow 조작은 반드시 메인 스레드에서 실행해야 하므로 run_on_main_thread 사용
fn show_dropdown(window: &WebviewWindow, rect: Option<tauri::Rect>) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let win_width = 320.0_f64;

    let (x, y) = if let Some(r) = rect {
        let ix = r.position.to_logical::<f64>(scale).x;
        let iy = r.position.to_logical::<f64>(scale).y;
        let iw = r.size.to_logical::<f64>(scale).width;
        let ih = r.size.to_logical::<f64>(scale).height;
        (ix + iw / 2.0 - win_width / 2.0, iy + ih + 2.0)
    } else {
        (100.0, 24.0)
    };

    let _ = window.set_position(tauri::LogicalPosition::new(x, y));

    // NSWindow 조작을 메인 스레드에서 실행 (트레이 이벤트는 백그라운드 스레드)
    let w = window.clone();
    let _ = window.app_handle().run_on_main_thread(move || {
        unsafe {
            use objc2::msg_send;
            use objc2::runtime::AnyObject;
            if let Ok(ptr) = w.ns_window() {
                let ns_win = ptr as *mut AnyObject;
                let _: () = msg_send![ns_win, setLevel: 101_i64];
                // orderFrontRegardless: 앱 활성화 여부와 무관하게 최상위에 표시
                let _: () = msg_send![ns_win, orderFrontRegardless];
            }
        }
    });
}

/// 드롭다운 창 레벨 + collection behavior 초기 설정
#[cfg(target_os = "macos")]
fn init_dropdown_window(window: &WebviewWindow) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    unsafe {
        if let Ok(ptr) = window.ns_window() {
            let ns_win = ptr as *mut AnyObject;
            let _: () = msg_send![ns_win, setLevel: 101_i64];
            // CanJoinAllSpaces(1) | Stationary(16) | FullScreenAuxiliary(256) = 273
            // Stationary: 스페이스 전환 시 창 유지
            // FullScreenAuxiliary: 전체화면 앱 위에도 보조 창으로 표시 가능
            let _: () = msg_send![ns_win, setCollectionBehavior: 273_u64];
        }
    }
}

/// 전역 마우스 클릭 모니터 등록 (main thread에서 호출해야 함)
/// 창 밖 클릭 시 드롭다운 자동 닫힘
#[cfg(target_os = "macos")]
fn register_global_mouse_monitor(window: WebviewWindow, shown_at: Arc<Mutex<Option<Instant>>>) {
    use block2::RcBlock;
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let block = RcBlock::new(move |_event: *mut AnyObject| {
        if !window.is_visible().unwrap_or(false) {
            return;
        }
        let elapsed = shown_at
            .lock()
            .unwrap()
            .map(|t| t.elapsed())
            .unwrap_or(std::time::Duration::ZERO);

        // 창이 표시된 지 500ms 이후의 클릭만 닫힘으로 처리
        // (트레이 아이콘 클릭 자체가 global monitor에 잡히는 것 방지)
        if elapsed > std::time::Duration::from_millis(500) {
            let _ = window.hide();
        }
    });

    unsafe {
        let mask: u64 = 1 << 1; // NSLeftMouseDown
        let _monitor: *mut AnyObject = msg_send![
            objc2::class!(NSEvent),
            addGlobalMonitorForEventsMatchingMask: mask,
            handler: &*block
        ];
        // block과 monitor를 앱 수명 동안 유지
        std::mem::forget(block);
        // _monitor는 Objective-C 런타임이 관리하므로 별도 유지 불필요
    }
}

/// Accessibility 권한 요청 (앱 시작 시 시스템 팝업)
#[cfg(target_os = "macos")]
fn request_accessibility_permission() {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
    }
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    use objc2_foundation::NSString;
    unsafe {
        let key = NSString::from_str("AXTrustedCheckOptionPrompt");
        let val: *mut AnyObject = msg_send![objc2::class!(NSNumber), numberWithBool: true];
        let dict: *mut AnyObject = msg_send![
            objc2::class!(NSDictionary),
            dictionaryWithObject: val,
            forKey: &*key
        ];
        AXIsProcessTrustedWithOptions(dict as *const std::ffi::c_void);
    }
}

fn compute_tray_title(pool: &services::db::DbPool, session_id: &str) -> Option<String> {
    let conn = pool.get().ok()?;

    let started_at: i64 = conn
        .query_row(
            "SELECT started_at FROM sessions WHERE id = ?1",
            rusqlite::params![session_id],
            |r| r.get(0),
        )
        .ok()?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let elapsed = (now - started_at).max(0);
    let hours = elapsed / 3600;
    let minutes = (elapsed % 3600) / 60;
    let seconds = elapsed % 60;

    let time_str = if hours > 0 {
        format!("{}h {:02}m", hours, minutes)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    };

    Some(format!("🟢 {}", time_str))
}
