// focaro — macOS 메뉴바 기반 활동 추적 집중 관리 도구

pub mod commands;
pub mod errors;
pub mod models;
pub mod services;
pub mod state;

use std::sync::Mutex;
use tauri::{Manager, WebviewWindow};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

#[cfg(target_os = "macos")]
use tauri_nspanel::ManagerExt;

use services::db;
use state::app_state::AppState;

pub fn run() {
    let mut builder = tauri::Builder::default();

    // opener 플러그인 — URL/파일을 기본 앱으로 열기, 크로스플랫폼 지원
    builder = builder.plugin(tauri_plugin_opener::init());

    // NSPanel 플러그인 — 전체화면 앱 위에 표시되는 메뉴바 드롭다운 구현
    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
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

            // 메뉴바 전용 앱: 독 아이콘 숨김, 앱 활성화 없이 창 표시 가능
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            #[cfg(target_os = "macos")]
            request_accessibility_permission();

            let dropdown = app.get_webview_window("dropdown").unwrap();

            // NSWindow → NSPanel 변환 + 전체화면 위에 뜨도록 설정
            #[cfg(target_os = "macos")]
            init_menubar_panel(app.handle(), &dropdown);

            // 대시보드 창: X로 닫아도 파괴되지 않고 숨기기만 함
            // → 재오픈 시 get_webview_window("dashboard")가 항상 Some을 반환
            if let Some(dashboard) = app.get_webview_window("dashboard") {
                let win = dashboard.clone();
                dashboard.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win.hide();
                    }
                });
            }

            // 트레이 우클릭 메뉴
            let quit_item = MenuItem::with_id(app, "quit", "focaro 종료", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&quit_item])?;

            // 트레이 클릭 핸들러
            let tray_app = app.handle().clone();
            let _tray = TrayIconBuilder::with_id("tray")
                .title("🔵")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    if event.id() == "quit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let rect = tray.rect().ok().flatten();
                        toggle_panel(&tray_app, rect);
                    }
                })
                .build(app)?;

            // 트레이 타이틀 1초 갱신 루프
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
            commands::session::get_focus_stats,
            commands::session::get_top_apps,
            commands::session::get_current_app,
            commands::session::get_current_url,
            commands::session::get_current_title,
            commands::reference::save_reference,
            commands::reference::get_references,
            commands::reference::delete_reference,
            commands::reference::update_reference,
            commands::activity::get_activity_timeline,
            commands::activity::get_top_sites,
            commands::activity::get_daily_focus_stats,
            commands::activity::get_session_events,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// NSWindow를 NSPanel로 변환하여 전체화면 앱 위에서도 표시되도록 설정
/// moa(Nexters/moa) 프로젝트와 동일한 패턴
#[cfg(target_os = "macos")]
fn init_menubar_panel(app_handle: &tauri::AppHandle, window: &WebviewWindow) {
    use tauri_nspanel::{
        cocoa::appkit::{NSMainMenuWindowLevel, NSWindowCollectionBehavior},
        panel_delegate, WebviewWindowExt,
    };

    let panel = window.to_panel().unwrap();

    // 메뉴바 레벨 바로 위 (25) — NSPopUpMenuWindowLevel(101)보다 낮아도
    // NSPanel + NonActivatingPanel 조합이 전체화면 위에 뜨는 것을 보장
    panel.set_level(NSMainMenuWindowLevel + 1);

    // NonActivatingPanel: 패널이 키 윈도우가 되어도 앱이 활성화되지 않음
    // 다른 앱이 포커스를 잃지 않고 드롭다운만 표시됨
    #[allow(non_upper_case_globals)]
    const NSWindowStyleMaskNonActivatingPanel: i32 = 1 << 7;
    panel.set_style_mask(NSWindowStyleMaskNonActivatingPanel);

    // CanJoinAllSpaces: 모든 스페이스(전체화면 포함)에 표시
    // Stationary: 스페이스 전환 시 위치 유지
    // FullScreenAuxiliary: 전체화면 앱 위에 보조 창으로 표시 (핵심)
    panel.set_collection_behaviour(
        NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary,
    );

    // 패널 밖 클릭 → window_did_resign_key → 자동 닫힘
    // 글로벌 마우스 모니터 불필요, NSWindowDelegate로 처리
    let handle = app_handle.clone();
    let delegate = panel_delegate!(FocaroPanelDelegate { window_did_resign_key });
    delegate.set_listener(Box::new(move |event: String| {
        if event == "window_did_resign_key" {
            if let Ok(panel) = handle.get_webview_panel("dropdown") {
                panel.order_out(None);
            }
        }
    }));
    panel.set_delegate(delegate);
}

/// 드롭다운 패널 토글 (트레이 클릭 시 호출)
fn toggle_panel(app_handle: &tauri::AppHandle, rect: Option<tauri::Rect>) {
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = app_handle.get_webview_panel("dropdown") {
            if panel.is_visible() {
                panel.order_out(None);
                return;
            }
        }

        // 트레이 아이콘 아래에 위치 지정 후 패널 표시
        if let Some(window) = app_handle.get_webview_window("dropdown") {
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

            let app = app_handle.clone();
            let _ = window.app_handle().run_on_main_thread(move || {
                if let Some(w) = app.get_webview_window("dropdown") {
                    let _ = w.set_position(tauri::LogicalPosition::new(x, y));
                }
                if let Ok(panel) = app.get_webview_panel("dropdown") {
                    panel.show();
                }
            });
        }
    }
}

/// Accessibility 권한 요청
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
