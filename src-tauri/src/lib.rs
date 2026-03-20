// focaro - macOS 메뉴바 기반 활동 추적 집중 관리 도구
// Phase 1 초기 골격: 의존성 구조 확인용

pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
