import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { Onboarding } from "../../pages/Onboarding";

vi.mock("../../services/onboarding", () => ({
  applyProfessionRules: vi.fn().mockResolvedValue(undefined),
  completeOnboarding: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    close: vi.fn().mockResolvedValue(undefined),
    hide: vi.fn().mockResolvedValue(undefined),
  }),
}));

describe("Onboarding", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("Welcome 화면이 처음에 표시된다", () => {
    render(<Onboarding />);
    expect(screen.getByText(/환영합니다/)).toBeTruthy();
    expect(screen.getByText("시작하기")).toBeTruthy();
    expect(screen.getByText("건너뛰기")).toBeTruthy();
  });

  it("시작하기 버튼 클릭 시 직업 선택 화면으로 이동한다", () => {
    render(<Onboarding />);
    fireEvent.click(screen.getByText("시작하기"));
    expect(screen.getByText("직업을 선택해주세요")).toBeTruthy();
  });

  it("직업 목록 5개가 표시된다", () => {
    render(<Onboarding />);
    fireEvent.click(screen.getByText("시작하기"));
    expect(screen.getByText("개발자")).toBeTruthy();
    expect(screen.getByText("디자이너")).toBeTruthy();
    expect(screen.getByText("마케터")).toBeTruthy();
    expect(screen.getByText("학생")).toBeTruthy();
    expect(screen.getByText(/기타/)).toBeTruthy();
  });

  it("직업 선택 후 다음 버튼이 활성화된다", () => {
    render(<Onboarding />);
    fireEvent.click(screen.getByText("시작하기"));

    const nextBtn = screen.getByText("다음");
    expect(nextBtn).toHaveProperty("disabled", true);

    fireEvent.click(screen.getByText("개발자"));
    expect(nextBtn).toHaveProperty("disabled", false);
  });

  it("다음 클릭 시 applyProfessionRules를 호출하고 완료 화면으로 이동한다", async () => {
    const { applyProfessionRules } = await import("../../services/onboarding");
    render(<Onboarding />);
    fireEvent.click(screen.getByText("시작하기"));
    fireEvent.click(screen.getByText("개발자"));
    fireEvent.click(screen.getByText("다음"));

    await waitFor(() => {
      expect(applyProfessionRules).toHaveBeenCalledWith("developer");
      expect(screen.getByText("설정 완료!")).toBeTruthy();
    });
  });

  it("건너뛰기 클릭 시 completeOnboarding을 호출한다", async () => {
    const { completeOnboarding } = await import("../../services/onboarding");
    render(<Onboarding />);
    fireEvent.click(screen.getByText("건너뛰기"));

    await waitFor(() => {
      expect(completeOnboarding).toHaveBeenCalled();
    });
  });

  it("완료 화면에서 시작하기 버튼으로 completeOnboarding 호출한다", async () => {
    const { applyProfessionRules, completeOnboarding } = await import("../../services/onboarding");
    render(<Onboarding />);
    fireEvent.click(screen.getByText("시작하기"));
    fireEvent.click(screen.getByText("개발자"));
    fireEvent.click(screen.getByText("다음"));

    await waitFor(() => expect(screen.getByText("설정 완료!")).toBeTruthy());

    fireEvent.click(screen.getByText("시작하기"));

    await waitFor(() => {
      expect(applyProfessionRules).toHaveBeenCalledTimes(1);
      expect(completeOnboarding).toHaveBeenCalled();
    });
  });
});
