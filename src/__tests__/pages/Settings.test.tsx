import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { Settings } from "../../pages/Settings";

const SETTINGS_MOCK = {
  retention_days: 30,
  shortcut_save_ref: "CmdOrCtrl+Shift+R",
};

const RULES_MOCK = [
  { id: 1, domain: "github.com", category: "Focus" },
  { id: 2, domain: "youtube.com", category: "Distraction" },
];

describe("Settings 페이지", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockIPC((cmd) => {
      if (cmd === "get_settings") return SETTINGS_MOCK;
      if (cmd === "get_classification_rules") return RULES_MOCK;
      if (cmd === "update_settings") return null;
      if (cmd === "add_classification_rule") return { id: 99, domain: "example.com", category: "Focus" };
      if (cmd === "delete_classification_rule") return null;
    });
  });

  it("설정 페이지가 렌더링됨", async () => {
    render(<Settings />);
    await waitFor(() => {
      expect(screen.getByText(/보관 기간/i)).toBeTruthy();
    });
  });

  it("현재 단축키가 표시됨", async () => {
    render(<Settings />);
    await waitFor(() => {
      expect(screen.getByText("CmdOrCtrl+Shift+R")).toBeTruthy();
    });
  });

  it("분류 규칙 목록이 표시됨", async () => {
    render(<Settings />);
    await waitFor(() => {
      expect(screen.getByText("github.com")).toBeTruthy();
      expect(screen.getByText("youtube.com")).toBeTruthy();
    });
  });

  it("보관 기간 변경 후 저장 가능", async () => {
    let savedSettings: unknown = null;
    mockIPC((cmd, args) => {
      if (cmd === "get_settings") return SETTINGS_MOCK;
      if (cmd === "get_classification_rules") return RULES_MOCK;
      if (cmd === "update_settings") {
        savedSettings = (args as { settings: unknown }).settings;
        return null;
      }
    });
    render(<Settings />);
    await waitFor(() => screen.getByText(/보관 기간/i));

    const select7 = screen.getByRole("radio", { name: "7일" });
    fireEvent.click(select7);

    const saveBtn = screen.getByRole("button", { name: /저장/i });
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(savedSettings).toBeTruthy();
      expect((savedSettings as { retention_days: number }).retention_days).toBe(7);
    });
  });

  it("분류 규칙 삭제 버튼 존재", async () => {
    render(<Settings />);
    await waitFor(() => screen.getByText("github.com"));
    const deleteBtns = screen.getAllByRole("button", { name: /삭제/i });
    expect(deleteBtns.length).toBeGreaterThan(0);
  });

  it("새 분류 규칙 추가 폼 존재", async () => {
    render(<Settings />);
    await waitFor(() => screen.getByPlaceholderText(/도메인/i));
    expect(screen.getByPlaceholderText(/도메인/i)).toBeTruthy();
  });
});
