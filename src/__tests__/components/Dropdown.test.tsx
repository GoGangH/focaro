import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { Dropdown } from "../../pages/Dropdown";

// getCurrentWindow().hide() 모킹
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    label: "dropdown",
    hide: vi.fn().mockResolvedValue(undefined),
  }),
}));

const SESSION_MOCK = {
  id: "sess-1",
  started_at: new Date(Date.now() - 60000).toISOString(),
  ended_at: null,
  status: "Active" as const,
};

const FOCUS_STATS_MOCK = {
  total_secs: 60,
  focus_secs: 36,
  neutral_secs: 12,
  distraction_secs: 12,
};

const TOP_APPS_MOCK = [
  { app_name: "Google Chrome", duration_secs: 36, classification: "Focus", percentage: 60 },
  { app_name: "Slack", duration_secs: 24, classification: "Neutral", percentage: 40 },
];

function setupIPC({
  incomplete = null,
  session = null,
  focusStats = FOCUS_STATS_MOCK,
  topApps = TOP_APPS_MOCK,
  currentApp = "Google Chrome",
}: {
  incomplete?: typeof SESSION_MOCK | null;
  session?: typeof SESSION_MOCK | null;
  focusStats?: typeof FOCUS_STATS_MOCK;
  topApps?: typeof TOP_APPS_MOCK;
  currentApp?: string | null;
} = {}) {
  mockIPC((cmd) => {
    switch (cmd) {
      case "get_incomplete_session": return incomplete;
      case "get_current_session": return session;
      case "start_session": return SESSION_MOCK;
      case "end_session": return null;
      case "get_focus_stats": return focusStats;
      case "get_top_apps": return topApps;
      case "get_current_app": return currentApp;
      case "get_current_url": return null;
      case "get_current_title": return null;
      case "check_accessibility_permission": return true;
      case "open_dashboard": return undefined;
      case "get_goal_progress": return { date: "2026-03-25", target_secs: 7200, actual_focus_secs: 0, progress_pct: 0 };
      case "get_daily_goal": return { date: "2026-03-25", target_secs: 7200 };
      default: return undefined;
    }
  });
}

describe("Dropdown", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("세션 없을 때 '세션 시작' 버튼 표시", async () => {
    setupIPC();
    render(<Dropdown />);
    await waitFor(() => {
      expect(screen.getByText("세션 시작")).toBeTruthy();
    });
  });

  it("미완료 세션 있을 때 복구 팝업 표시", async () => {
    setupIPC({ incomplete: SESSION_MOCK });
    render(<Dropdown />);
    await waitFor(() => {
      expect(screen.getByText("이전 세션이 종료되지 않았습니다")).toBeTruthy();
      expect(screen.getByText("이어가기")).toBeTruthy();
      expect(screen.getByText("종료")).toBeTruthy();
    });
  });

  it("세션 없을 때 타이머 '--:--' 표시", async () => {
    setupIPC();
    render(<Dropdown />);
    await waitFor(() => {
      expect(screen.getByText("--:--")).toBeTruthy();
    });
  });

  it("'세션 시작' 클릭 시 세션 시작됨", async () => {
    setupIPC();
    render(<Dropdown />);
    await waitFor(() => screen.getByText("세션 시작"));
    fireEvent.click(screen.getByText("세션 시작"));
    await waitFor(() => {
      expect(screen.getByText("세션 종료")).toBeTruthy();
    });
  });

  it("세션 활성 상태에서 도넛 차트 렌더링", async () => {
    setupIPC({ incomplete: null });
    render(<Dropdown />);
    // 세션 시작
    await waitFor(() => screen.getByText("세션 시작"));
    fireEvent.click(screen.getByText("세션 시작"));
    await waitFor(() => {
      const svg = document.querySelector("svg");
      expect(svg).toBeTruthy();
    });
  });

  it("세션 활성 상태에서 앱 리스트 표시", async () => {
    setupIPC({ incomplete: null });
    render(<Dropdown />);
    await waitFor(() => screen.getByText("세션 시작"));
    fireEvent.click(screen.getByText("세션 시작"));
    await waitFor(() => {
      const items = screen.getAllByText("Google Chrome");
      expect(items.length).toBeGreaterThanOrEqual(1);
    });
  });

  it("Dashboard 열기 버튼 존재", async () => {
    setupIPC();
    render(<Dropdown />);
    await waitFor(() => {
      expect(screen.getByText("Dashboard 열기")).toBeTruthy();
    });
  });
});
