# 리서치: Activity Focus Tracker

**브랜치**: `001-activity-focus-tracker`
**날짜**: 2026-03-20

---

## 1. macOS 활성 앱 감지

### 결정
`objc2` 크레이트로 `NSWorkspace.shared.frontmostApplication`을 호출한다.

### 근거
- NSWorkspace는 macOS 공개 API로, 별도 Accessibility 권한 없이 활성 앱 이름을 읽을 수 있다.
- `objc2`는 `cocoa`(구버전)를 대체하는 현대적 Objective-C 바인딩이다.
- `active-win-pos-rs`는 크로스플랫폼이지만 macOS 지원이 불안정하다.

### 핵심 API
```
NSWorkspace.shared.frontmostApplication → NSRunningApplication
NSRunningApplication.localizedName → String (앱 이름)
NSRunningApplication.bundleIdentifier → String
```

### 사용 크레이트
```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-foundation = "0.2"
objc2-app-kit = "0.2"
```

### 필요 권한
- NSWorkspace 방식: **별도 권한 불필요**
- Accessibility API (창 세부 정보 필요 시): System Preferences → Security & Privacy → Accessibility 권한

---

## 2. 브라우저 URL 읽기 (Chrome / Safari)

### 결정
`std::process::Command`로 `osascript`를 직접 실행한다. 별도 크레이트 미사용.

### 근거
- `osascript` 크레이트는 의존성 추가 대비 이점이 없다 (헌법 V. 단순성).
- `std::process::Command`로 충분하며 타임아웃 제어가 용이하다.

### AppleScript 쿼리

**Chrome:**
```applescript
tell application "Google Chrome"
    if (count of windows) = 0 then error "no window"
    get URL of active tab of front window
end tell
```

**Safari:**
```applescript
tell application "Safari"
    if (count of windows) = 0 then error "no window"
    get URL of current tab of front window
end tell
```

### 에러 처리
| 상황 | 처리 |
|------|------|
| 브라우저 미실행 | `url = None` 으로 처리 |
| Automation 권한 거부 | `url = None`, 사용자에게 권한 요청 안내 |
| 타임아웃(500ms 초과) | `url = None` |

### 필요 권한
- macOS Catalina 이상: System Preferences → Security & Privacy → **Automation** 권한
- 첫 실행 시 macOS가 자동으로 권한 다이얼로그를 표시한다.

---

## 3. SQLite 스토리지

### 결정
`rusqlite` + `r2d2` 커넥션 풀, 마이그레이션은 `refinery`.

### 근거
| 옵션 | 판단 |
|------|------|
| `rusqlite` + `r2d2` | ✅ 채택. 동기 I/O, 데스크탑 앱에 적합, 의존성 최소 |
| `sqlx` | ❌ async-first라 Tauri 이벤트 루프와 복잡도 증가, 데스크탑 앱에 과도함 |
| `tauri-plugin-sql` | ❌ 프론트엔드에서 SQL 직접 실행 → 비즈니스 로직이 TS로 노출, 헌법 II 위반 |

### DB 파일 위치
```rust
// ~/Library/Application Support/focaro/focaro.db
let db_path = app.path().app_data_dir()?.join("focaro.db");
```

### 마이그레이션
`refinery` 크레이트 사용, `src-tauri/migrations/` 디렉토리에 SQL 파일 관리.

### Cargo.toml
```toml
rusqlite = { version = "0.31", features = ["bundled"] }
r2d2 = "0.8"
r2d2_sqlite = "0.25"
refinery = { version = "0.8", features = ["rusqlite"] }
```

---

## 4. Tauri v2 메뉴바 앱 구현

### 결정
`TrayIconBuilder` API 사용. 드롭다운은 별도 `WebviewWindow`로 구현.

### 핵심 사항

**트레이 아이콘 동적 텍스트**:
- `tray.set_title(Some("🟢 1h24m"))` — macOS에서 트레이 텍스트 표시
- `tray.set_tooltip(Some("Focus 78%"))` — 툴팁으로 보조 정보 표시

**드롭다운 창**:
- `decorations: false`, `transparent: true`, `resizable: false`
- 포커스 잃을 때(`WindowEvent::Focused(false)`) 자동 숨김
- 트레이 아이콘 위치 기준으로 창 위치 계산

**독 아이콘 숨김 (LSUIElement)**:
```xml
<!-- src-tauri/Info.plist -->
<key>LSUIElement</key>
<true/>
```
또는 `tauri.conf.json`에서 `"activationPolicy": "accessory"` 설정.

**다중 창 구조**:
| 창 레이블 | 용도 | 기본 상태 |
|-----------|------|-----------|
| `dropdown` | 메뉴바 클릭 시 팝오버 | hidden |
| `dashboard` | 확장 대시보드 | hidden |

### 관련 Tauri v2 API
```rust
TrayIconBuilder::new()          // 트레이 아이콘 생성
tray.set_title(text)            // 메뉴바 텍스트 업데이트
tray.set_tooltip(text)          // 툴팁
TrayIconEvent::Click { .. }     // 클릭 이벤트
app.get_webview_window(label)   // 창 접근
window.show() / window.hide()   // 창 표시/숨김
WindowEvent::Focused(false)     // 포커스 손실 감지
```

---

## 5. 타입 안전성 — tauri-specta 설정

### 결정
`tauri-specta`로 Rust 타입 → TypeScript 타입 자동 생성.

### 설정
```toml
# Cargo.toml
tauri-specta = { version = "2", features = ["derive", "typescript"] }
specta = "2"
```

빌드 시 `src/types/bindings.ts` 자동 생성. 프론트엔드는 이 파일의 타입만 사용한다.

---

## 6. 헌법 준수 확인 (Phase 0)

| 원칙 | 상태 | 비고 |
|------|------|------|
| I. Tauri 아키텍처 | ✅ | 규정 디렉토리 구조 준수 |
| II. 타입 안전성 | ✅ | tauri-specta, serde 사용 |
| III. TDD Red-Green | ✅ | 각 서비스 단위 테스트 먼저 작성 |
| IV. DRY | ✅ | 비즈니스 로직은 services/, 중복 없음 |
| V. 단순성 | ✅ | osascript 직접 호출, 불필요 크레이트 제거 |
