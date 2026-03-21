import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { SaveReferencePage } from "../../pages/SaveReferencePage";

const REFERENCE_MOCK = {
  id: "ref-1",
  session_id: "sess-1",
  url: "https://github.com",
  title: "GitHub",
  tags: [],
  created_at: "2026-03-21T00:00:00Z",
};

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ close: vi.fn() }),
}));

describe("SaveReferencePage (팝업 창 전용)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("폼이 렌더링됨", () => {
    mockIPC((cmd) => {
      if (cmd === "get_current_url") return "https://github.com";
      if (cmd === "get_current_title") return "GitHub";
    });
    render(<SaveReferencePage />);
    expect(screen.getByPlaceholderText(/제목/i)).toBeTruthy();
    expect(screen.getByPlaceholderText(/태그/i)).toBeTruthy();
  });

  it("제목 없이 저장 불가", () => {
    mockIPC((cmd) => {
      if (cmd === "get_current_url") return "https://github.com";
      if (cmd === "get_current_title") return null;
    });
    render(<SaveReferencePage />);
    const saveBtn = screen.getByRole("button", { name: /저장/i });
    expect((saveBtn as HTMLButtonElement).disabled).toBe(true);
  });

  it("제목 입력 후 저장 버튼 활성화", async () => {
    mockIPC((cmd) => {
      if (cmd === "get_current_url") return "https://github.com";
      if (cmd === "get_current_title") return null;
    });
    render(<SaveReferencePage />);
    // loading 완료 대기
    await waitFor(() => {
      expect((screen.getByPlaceholderText(/제목/i) as HTMLInputElement).disabled).toBe(false);
    });
    const titleInput = screen.getByPlaceholderText(/제목/i);
    fireEvent.change(titleInput, { target: { value: "My Ref" } });
    const saveBtn = screen.getByRole("button", { name: /저장/i });
    expect((saveBtn as HTMLButtonElement).disabled).toBe(false);
  });

  it("저장 성공 후 save_reference 커맨드 호출", async () => {
    let called = false;
    mockIPC((cmd) => {
      if (cmd === "get_current_url") return "https://github.com";
      if (cmd === "get_current_title") return "GitHub";
      if (cmd === "save_reference") {
        called = true;
        return REFERENCE_MOCK;
      }
    });
    render(<SaveReferencePage />);
    await waitFor(() => {
      const titleInput = screen.getByPlaceholderText(/제목/i);
      expect((titleInput as HTMLInputElement).value).toBeTruthy();
    });

    const saveBtn = screen.getByRole("button", { name: /저장/i });
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(called).toBe(true);
    });
  });
});
