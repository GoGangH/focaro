import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { SaveReference } from "../../components/Dropdown/SaveReference";

const REFERENCE_MOCK = {
  id: "ref-1",
  session_id: "sess-1",
  url: "https://github.com",
  title: "GitHub",
  tags: [],
  created_at: "2026-03-21T00:00:00Z",
};

describe("SaveReference", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("URL이 없으면 버튼 비활성화", () => {
    render(<SaveReference currentUrl={null} />);
    const btn = screen.getByRole("button", { name: /reference 저장/i });
    expect(btn).toBeTruthy();
    expect((btn as HTMLButtonElement).disabled).toBe(true);
  });

  it("URL이 있으면 버튼 활성화", () => {
    render(<SaveReference currentUrl="https://github.com" />);
    const btn = screen.getByRole("button", { name: /reference 저장/i });
    expect((btn as HTMLButtonElement).disabled).toBe(false);
  });

  it("버튼 클릭 시 폼 표시", async () => {
    mockIPC((cmd) => {
      if (cmd === "get_current_title") return null;
    });
    render(<SaveReference currentUrl="https://github.com" />);
    fireEvent.click(screen.getByRole("button", { name: /reference 저장/i }));
    await waitFor(() => {
      expect(screen.getByPlaceholderText("제목 입력")).toBeTruthy();
    });
  });

  it("폼 열릴 때 get_current_title로 타이틀 자동 채움", async () => {
    mockIPC((cmd) => {
      if (cmd === "get_current_title") return "GitHub";
    });
    render(<SaveReference currentUrl="https://github.com" />);
    fireEvent.click(screen.getByRole("button", { name: /reference 저장/i }));
    await waitFor(() => {
      const input = screen.getByPlaceholderText("제목 입력") as HTMLInputElement;
      expect(input.value).toBe("GitHub");
    });
  });

  it("제목 없이 저장 버튼 클릭 시 저장 안 됨", async () => {
    mockIPC((cmd) => {
      if (cmd === "get_current_title") return null;
    });
    const saved = vi.fn();
    render(<SaveReference currentUrl="https://github.com" onSaved={saved} />);
    fireEvent.click(screen.getByRole("button", { name: /reference 저장/i }));
    await waitFor(() => screen.getByPlaceholderText("제목 입력"));
    fireEvent.click(screen.getByRole("button", { name: /저장$/ }));
    expect(saved).not.toHaveBeenCalled();
  });

  it("제목 입력 후 저장 시 save_reference 커맨드 호출", async () => {
    mockIPC((cmd) => {
      if (cmd === "get_current_title") return null;
      if (cmd === "save_reference") return REFERENCE_MOCK;
    });
    const onSaved = vi.fn();
    render(<SaveReference currentUrl="https://github.com" onSaved={onSaved} />);

    fireEvent.click(screen.getByRole("button", { name: /reference 저장/i }));
    await waitFor(() => screen.getByPlaceholderText("제목 입력"));

    fireEvent.change(screen.getByPlaceholderText("제목 입력"), {
      target: { value: "GitHub" },
    });
    fireEvent.click(screen.getByRole("button", { name: /저장$/ }));

    await waitFor(() => {
      expect(onSaved).toHaveBeenCalledWith(REFERENCE_MOCK);
    });
  });

  it("저장 후 폼이 닫힘", async () => {
    mockIPC((cmd) => {
      if (cmd === "get_current_title") return null;
      if (cmd === "save_reference") return REFERENCE_MOCK;
    });
    render(<SaveReference currentUrl="https://github.com" />);

    fireEvent.click(screen.getByRole("button", { name: /reference 저장/i }));
    await waitFor(() => screen.getByPlaceholderText("제목 입력"));

    fireEvent.change(screen.getByPlaceholderText("제목 입력"), {
      target: { value: "GitHub" },
    });
    fireEvent.click(screen.getByRole("button", { name: /저장$/ }));

    await waitFor(() => {
      expect(screen.queryByPlaceholderText("제목 입력")).toBeNull();
    });
  });

  it("취소 버튼 클릭 시 폼 닫힘", async () => {
    mockIPC((cmd) => {
      if (cmd === "get_current_title") return null;
    });
    render(<SaveReference currentUrl="https://github.com" />);
    fireEvent.click(screen.getByRole("button", { name: /reference 저장/i }));
    await waitFor(() => screen.getByPlaceholderText("제목 입력"));
    fireEvent.click(screen.getByRole("button", { name: /취소/ }));
    await waitFor(() => {
      expect(screen.queryByPlaceholderText("제목 입력")).toBeNull();
    });
  });
});
