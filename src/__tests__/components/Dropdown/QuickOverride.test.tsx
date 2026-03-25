import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { QuickOverride } from "../../../components/Dropdown/QuickOverride";

vi.mock("../../../services/onboarding", () => ({
  addTitleRule: vi.fn().mockResolvedValue({ id: 1, domain: "youtube.com", keyword: "tutorial", category: "Focus" }),
}));

const DEFAULT_PROPS = {
  domain: "youtube.com",
  title: "Rust Tutorial 2024",
  currentCategory: "Distraction",
  onClose: vi.fn(),
};

describe("QuickOverride", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("도메인과 타이틀이 표시된다", () => {
    render(<QuickOverride {...DEFAULT_PROPS} />);
    expect(screen.getByText("youtube.com")).toBeTruthy();
    expect(screen.getByText("Rust Tutorial 2024")).toBeTruthy();
  });

  it("현재 분류(Distraction)는 버튼으로 표시되지 않는다", () => {
    render(<QuickOverride {...DEFAULT_PROPS} />);
    expect(screen.queryByRole("button", { name: /방해/ })).toBeNull();
    expect(screen.getByRole("button", { name: /집중/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /기타/ })).toBeTruthy();
  });

  it("이번만 / 항상 이렇게 모드 버튼이 있다", () => {
    render(<QuickOverride {...DEFAULT_PROPS} />);
    expect(screen.getByText("이번만")).toBeTruthy();
    expect(screen.getByText("항상 이렇게")).toBeTruthy();
  });

  it("기본 모드는 이번만이다", () => {
    render(<QuickOverride {...DEFAULT_PROPS} />);
    const onceBtn = screen.getByText("이번만");
    expect(onceBtn.className).toContain("active");
  });

  it("항상 이렇게 모드에서 분류 변경 시 addTitleRule을 호출한다", async () => {
    const { addTitleRule } = await import("../../../services/onboarding");
    render(<QuickOverride {...DEFAULT_PROPS} />);

    fireEvent.click(screen.getByText("항상 이렇게"));
    fireEvent.click(screen.getByRole("button", { name: /집중/ }));

    await waitFor(() => {
      expect(addTitleRule).toHaveBeenCalledWith(
        "youtube.com",
        expect.any(String),
        "Focus"
      );
    });
  });

  it("이번만 모드에서는 addTitleRule을 호출하지 않는다", async () => {
    const { addTitleRule } = await import("../../../services/onboarding");
    render(<QuickOverride {...DEFAULT_PROPS} />);

    // 기본 모드 "이번만"
    fireEvent.click(screen.getByRole("button", { name: /집중/ }));

    await waitFor(() => {
      expect(addTitleRule).not.toHaveBeenCalled();
    });
  });

  it("닫기 버튼 클릭 시 onClose를 호출한다", () => {
    const onClose = vi.fn();
    render(<QuickOverride {...DEFAULT_PROPS} onClose={onClose} />);
    fireEvent.click(screen.getByText("×"));
    expect(onClose).toHaveBeenCalled();
  });

  it("domain이 null이면 현재 사이트로 표시된다", () => {
    render(<QuickOverride {...DEFAULT_PROPS} domain={null} />);
    expect(screen.getByText("현재 사이트")).toBeTruthy();
  });
});
