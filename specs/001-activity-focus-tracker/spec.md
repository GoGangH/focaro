# Feature Specification: Activity Focus Tracker

**Feature Branch**: `001-activity-focus-tracker`
**Created**: 2026-03-20
**Status**: Draft
**Input**: macOS 메뉴바 기반 활동 추적 집중 관리 도구

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 세션 시작 및 실시간 집중 상태 확인 (Priority: P1)

사용자가 작업을 시작하면서 새 세션을 열고, 메뉴바에서 현재 집중 상태를 실시간으로 확인한다.
앱은 백그라운드에서 2초마다 현재 활성 앱과 브라우저 URL을 감지하고,
메뉴바에 세션 경과 시간과 Focus 퍼센트를 표시한다.

**Why this priority**: 이것이 앱의 핵심 가치이다. 세션 관리와 실시간 모니터링이 없으면 다른 기능은 의미가 없다.

**Independent Test**: 세션을 시작하고 2초 이상 대기한 뒤 메뉴바에 타이머와 Focus % 가 업데이트되는지 확인한다. Rust 단위 테스트로 활동 감지 및 분류 로직을 독립 검증할 수 있다.

**Acceptance Scenarios**:

1. **Given** 앱이 실행된 상태, **When** 사용자가 "세션 시작" 버튼을 누르면, **Then** 메뉴바에 타이머가 0부터 시작되고 세션이 생성된다.
2. **Given** 세션이 진행 중, **When** 2초가 경과하면, **Then** 현재 활성 앱이 감지되고 메뉴바의 타이머와 Focus % 가 갱신된다.
3. **Given** Chrome이 활성화된 상태, **When** 시스템이 활동을 감지하면, **Then** 현재 탭 URL을 읽어 domain을 추출하고 분류 규칙을 적용한다.
4. **Given** 세션이 진행 중, **When** 사용자가 "세션 종료" 버튼을 누르면, **Then** 트래커가 중지되고 종료 시간이 기록된다.

---

### User Story 2 - 드롭다운 UI에서 집중 현황 상세 확인 (Priority: P2)

사용자가 메뉴바 아이콘을 클릭하여 드롭다운을 열고, 현재 활동, 집중/산만/중립 비율 차트, 최근 활동 리스트를 확인한다.

**Why this priority**: 메뉴바 수치만으로는 맥락이 부족하다. 드롭다운이 없으면 사용자는 무엇에 시간을 쓰는지 파악하기 어렵다.

**Independent Test**: 세션 시작 후 메뉴바를 클릭하면 드롭다운이 열리고, 원형 차트와 현재 활동 정보, 최근 활동 리스트가 정상 표시되는지 확인한다. mockIPC를 사용한 React 단위 테스트로 UI 렌더링을 독립 검증할 수 있다.

**Acceptance Scenarios**:

1. **Given** 세션이 진행 중, **When** 메뉴바 아이콘을 클릭하면, **Then** 드롭다운이 열리고 현재 활동(앱 이름, 도메인), 세션 타이머, Focus %, 원형 차트가 표시된다.
2. **Given** 드롭다운이 열린 상태, **When** 활동이 변경되면, **Then** 현재 활동 정보와 차트가 실시간으로 갱신된다.
3. **Given** 드롭다운이 열린 상태, **When** 최근 활동 리스트를 보면, **Then** 직전 활동들이 앱 이름, 도메인, 지속 시간과 함께 시간순으로 표시된다.

---

### User Story 3 - 활동 분류 및 Focus Metrics 계산 (Priority: P3)

시스템이 감지된 activity의 domain을 기반으로 Focus / Neutral / Distraction 을 자동 분류하고,
세션 전체의 Focus Metrics(총 시간, 카테고리별 시간 및 퍼센트)를 계산한다.

**Why this priority**: 분류와 메트릭이 없으면 앱은 단순한 타이머에 불과하다. 의미 있는 집중 데이터를 제공하기 위해 필요하다.

**Independent Test**: 미리 정의된 domain 목록으로 분류 규칙 단위 테스트를 실행하고, 특정 시나리오에서 퍼센트 합산이 100%가 되는지 검증한다.

**Acceptance Scenarios**:

1. **Given** github.com 활동이 기록됨, **When** 분류 규칙이 적용되면, **Then** Focus 로 분류된다.
2. **Given** youtube.com 활동이 기록됨, **When** 분류 규칙이 적용되면, **Then** Distraction 으로 분류된다.
3. **Given** 규칙에 없는 도메인이 기록됨, **When** 분류 규칙이 적용되면, **Then** Neutral 로 분류된다.
4. **Given** 세션이 종료됨, **When** Focus Metrics를 조회하면, **Then** focus + neutral + distraction 퍼센트의 합이 100%이다.
5. **Given** 세션 시간이 0초인 경우, **When** 퍼센트를 계산하면, **Then** 모든 값이 0으로 반환된다.

---

### User Story 4 - Reference 저장 (Priority: P4)

사용자가 현재 브라우저에서 열린 URL을 제목과 태그를 입력하여 저장하고, 나중에 Dashboard에서 확인한다.

**Why this priority**: 핵심 추적 기능과 독립적이며 부가 편의 기능이다. P1~P3 완료 후 추가된다.

**Independent Test**: 브라우저 활동 중 Reference 저장 버튼을 눌러 제목을 입력하고 저장하면, Dashboard의 Saved References 목록에 해당 URL과 제목이 나타나는지 확인한다.

**Acceptance Scenarios**:

1. **Given** 브라우저가 활성화된 세션 진행 중, **When** "Reference 저장" 버튼을 누르면, **Then** 현재 URL이 자동으로 채워진 저장 폼이 표시된다.
2. **Given** 저장 폼이 열린 상태, **When** 제목을 입력하고 저장하면, **Then** URL, 제목, 태그, 세션 ID, 저장 시각이 함께 저장된다.
3. **Given** Reference가 저장된 상태, **When** Dashboard를 열면, **Then** Saved References 목록에서 확인할 수 있다.

---

### User Story 5 - Dashboard에서 이력 분석 (Priority: P5)

사용자가 Dashboard를 열어 Activity Timeline, Top Sites, Focus Score, 날짜별 활동 기록을 확인한다.

**Why this priority**: 이력 분석은 데이터가 충분히 쌓인 뒤 의미가 생기는 확장 기능이다.

**Independent Test**: 여러 세션 데이터가 존재하는 상태에서 Dashboard를 열면 Timeline, Top Sites, Focus Score가 정상 표시되는지 확인한다.

**Acceptance Scenarios**:

1. **Given** 하나 이상의 세션이 완료된 상태, **When** Dashboard를 열면, **Then** Activity Timeline에 활동이 시간순으로 표시된다.
2. **Given** Dashboard가 열린 상태, **When** Top Sites를 보면, **Then** 도메인별 누적 사용 시간이 내림차순으로 정렬되어 표시된다.
3. **Given** Dashboard가 열린 상태, **When** 날짜 필터를 변경하면, **Then** 해당 날짜의 활동 기록만 표시된다.

---

### Edge Cases

- 세션 진행 중 컴퓨터가 절전 모드가 되면? → 복귀 시 절전 시점까지만 마지막 활동을 기록하고, 절전 구간은 활동 없음으로 처리한다.
- 브라우저가 활성화되어 있지만 URL을 읽을 수 없으면(권한 거부)? → url 및 domain을 null로 처리하고 app_name만 기록한다.
- Accessibility 권한이 없으면? → 활동 추적을 중단하고 사용자에게 권한 설정 안내 메시지를 표시한다.
- 총 세션 시간이 0초이면 퍼센트 계산 시? → 모든 퍼센트를 0으로 반환하여 0으로 나누기를 방지한다.
- 앱이 2초 이내에 빠르게 전환되면? → 스냅샷 시점 기준 하나의 활동만 기록한다(2초 단위 샘플링).
- 앱이 크래시되거나 강제 종료된 경우? → 재시작 시 미완료 세션을 감지하여 사용자에게 "이전 세션을 이어할까요?" 팝업을 표시한다. 사용자가 이어가기를 선택하면 세션이 재개되고, 거부하면 미완료 세션은 크래시 시점 기준으로 종료 처리된다.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: 시스템은 사용자가 세션을 시작할 수 있어야 한다.
- **FR-002**: 시스템은 사용자가 세션을 종료할 수 있어야 한다.
- **FR-003**: 시스템은 세션 시작 시 고유 세션 식별자와 시작 시각을 기록해야 한다.
- **FR-004**: 시스템은 세션 종료 시 종료 시각을 기록해야 한다.
- **FR-005**: 시스템은 세션 진행 중 2초 간격으로 현재 활성 애플리케이션을 감지해야 한다.
- **FR-006**: 시스템은 활성 애플리케이션이 Google Chrome 또는 Safari일 경우 현재 탭 URL을 읽어야 한다.
- **FR-007**: 시스템은 URL에서 도메인을 추출해야 한다.
- **FR-008**: 시스템은 activity 데이터로 app_name(필수), url(선택), domain(선택), timestamp(필수)를 저장해야 한다.
- **FR-009**: 시스템은 이전 활동과 현재 활동을 비교하여 변경 여부를 감지해야 한다. (동일 판단 기준: app_name + url. 비브라우저 앱의 경우 url=null이므로 app_name만으로 동일성을 판단한다.)
- **FR-010**: 시스템은 활동이 변경될 때 이전 활동의 지속 시간(duration)을 계산하고 기록해야 한다. 세션 종료 시에는 종료 시각을 기준으로 마지막 진행 중 활동의 duration을 자동 계산하여 기록해야 한다.
- **FR-011**: 시스템은 기록된 모든 activity를 Focus / Neutral / Distraction 중 하나로 분류해야 한다.
- **FR-012**: 분류는 domain 기반 규칙 매칭을 따르며, 규칙에 없는 경우 Neutral로 분류해야 한다.
- **FR-013**: 시스템은 세션 동안 카테고리별 누적 시간(Focus, Neutral, Distraction)과 각 퍼센트를 계산해야 한다.
- **FR-014**: 메뉴바에 현재 세션 경과 시간과 Focus 퍼센트를 항상 표시해야 한다. 세션이 없는 대기 상태에서는 아이콘만 표시하며, 클릭 시 드롭다운에서 세션 시작 버튼을 제공해야 한다. (표시 형식 예시: `🟢 1h 24m | Focus 78%`)
- **FR-015**: 메뉴바 클릭 시 드롭다운이 열려야 하며, 원형 차트, 현재 활동 정보, 최근 활동 리스트(최근 3개), 세션 제어 버튼, Dashboard 열기 버튼이 포함되어야 한다.
- **FR-016**: 사용자는 브라우저 활동 중 현재 URL을 제목과 선택적 태그와 함께 Reference로 저장할 수 있어야 한다.
- **FR-017**: Dashboard에는 Activity Timeline, Top Sites, Focus Score, Saved References 목록, 날짜별 활동 기록이 표시되어야 한다.
- **FR-018**: 모든 데이터는 로컬에만 저장되어야 하며 외부로 전송되어서는 안 된다.
- **FR-019**: 앱은 백그라운드에서 지속 실행되는 메뉴바 앱으로 동작해야 한다.
- **FR-020**: 브라우저 URL 읽기 실패 시 url 필드는 null로 처리하고 app_name만 기록해야 한다.
- **FR-021**: Accessibility 권한이 없는 경우 활동 추적을 중단하고 사용자에게 권한 요청 안내를 표시해야 한다.
- **FR-022**: 앱 재시작 시 미완료 세션(종료 시각 없음)이 존재하면, 사용자에게 이어가기 또는 종료 선택 팝업을 표시해야 한다. 종료 선택 시 크래시 시점(마지막 감지 활동 timestamp)을 세션 종료 시각으로 기록해야 한다.
- **FR-023**: 사용자는 로우 Activity 데이터 보관 기간을 설정할 수 있어야 한다 (기본값: 30일).
- **FR-024**: 설정된 보관 기간이 지난 로우 Activity 데이터는 일별 집계 데이터(ArchivedDailySummary)로 변환 후 원본을 삭제해야 한다. 아카이빙은 앱 시작 시 자동으로 실행된다(MVP 범위).
- **FR-025**: Dashboard의 날짜별 활동 기록은 보관 기간 내 날짜는 로우 데이터를, 보관 기간 초과 날짜는 ArchivedDailySummary를 사용하여 표시해야 한다.

### Key Entities

- **Session**: 사용자의 한 작업 세션. 시작 시각, 종료 시각, 고유 식별자를 가진다.
- **Activity**: 2초 단위 스냅샷 기록. app_name, url, domain, timestamp, duration, classification(Focus/Neutral/Distraction)을 가진다. 하나의 Session에 속한다.
- **ClassificationRule**: domain과 카테고리(Focus/Neutral/Distraction)의 매핑 규칙. 예: "github.com → Focus".
- **FocusMetrics**: 세션 단위로 집계된 총 시간, 카테고리별 시간 및 퍼센트. Session으로부터 파생된다.
- **Reference**: 사용자가 수동 저장한 URL. url, title, tags, session_id, created_at을 가진다.
- **ArchivedDailySummary**: 보관 기간이 지난 로우 Activity를 날짜 단위로 집계한 요약 데이터. 날짜, 총 세션 시간, Focus/Neutral/Distraction 누적 시간, Top 도메인 목록을 포함한다. Dashboard의 날짜별 기록은 보관 기간 내는 로우 데이터, 이후는 이 요약 데이터를 사용한다.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 세션 시작 후 5초 이내에 메뉴바에 실시간 타이머와 Focus % 가 표시되어야 한다.
- **SC-002**: 활동 변경 감지는 변경 발생 후 4초(2사이클) 이내에 반영되어야 한다.
- **SC-003**: 세션 동안 집계된 Focus, Neutral, Distraction 퍼센트의 합이 항상 100%이어야 한다.
- **SC-004**: 정의된 domain 분류 규칙에 대한 분류 정확도가 100%이어야 한다.
- **SC-005**: 앱이 백그라운드에서 실행 중일 때 평균 CPU 사용량이 2% 미만이어야 한다. (측정 조건: 2초 폴링 활성 세션 진행 중, macOS 활성 모니터 기준)
- **SC-006**: Reference 저장 동작이 사용자 입력 후 1초 이내에 완료되어야 한다.
- **SC-007**: 세션 관리, 활동 추적, 분류, 메트릭 계산 등 모든 핵심 비즈니스 로직이 자동화된 테스트로 검증 가능해야 한다.
- **SC-008**: 세션 시간이 0초인 경우 퍼센트 계산이 오류 없이 완료되어야 한다.

## Clarifications

### Session 2026-03-20

- Q: 앱 크래시/재시작 시 진행 중이던 세션 처리 방식은? → A: 앱 재시작 시 미완료 세션이 있으면 사용자에게 "이전 세션을 이어할까요?" 확인 팝업을 표시한다.
- Q: 세션 종료 시 아직 duration이 계산되지 않은 마지막 활동 처리 방식은? → A: 세션 종료 시각을 기준으로 마지막 활동의 duration을 자동 계산하여 기록한다.
- Q: 세션 미시작(대기) 상태에서 메뉴바 표시 방식은? → A: 아이콘만 표시하고 텍스트는 없다. 클릭 시 드롭다운에서 세션 시작 버튼을 확인할 수 있다.
- Q: 활동 데이터 보존 기간 및 스토리지 관리 방식은? → A: 사용자가 설정 가능한 보관 기간(기본값 30일) 이후, 로우 Activity 데이터를 일별 집계 데이터(ArchivedDailySummary)로 변환 보관한다. 원본 로우 데이터는 삭제하여 스토리지를 절약한다.
- Q: 드롭다운 최근 활동 리스트 표시 개수는? → A: 최근 3개.

## Assumptions

- macOS Monterey (12.0) 이상을 지원 대상으로 가정한다.
- 초기 분류 규칙(github.com → Focus, youtube.com → Distraction 등)은 기본값으로 제공되며, MVP에서 사용자 커스터마이징은 지원하지 않는다.
- 세션은 동시에 하나만 실행된다(다중 세션 미지원).
- 브라우저 외 일반 앱(Xcode, Figma 등)은 url/domain 없이 app_name만 기록하며 Neutral로 분류된다.
- 로우 Activity 데이터 기본 보관 기간은 30일이며, 사용자가 변경할 수 있다. 기간 초과 시 일별 집계(ArchivedDailySummary)로 자동 변환된다.
