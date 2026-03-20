import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { DonutChart } from "../../components/Dropdown/DonutChart";

describe("DonutChart", () => {
  it("focus/neutral/distraction 데이터로 렌더링", () => {
    const { container } = render(
      <DonutChart focus={60} neutral={20} distraction={20} />
    );
    const circles = container.querySelectorAll("circle");
    // 배경 원 + 3개 세그먼트 = 4개
    expect(circles.length).toBe(4);
  });

  it("중앙에 focus 퍼센트 표시", () => {
    render(<DonutChart focus={60} neutral={20} distraction={20} />);
    // 60 / (60+20+20) = 60%
    expect(screen.getByText("60%")).toBeTruthy();
    expect(screen.getByText("focus")).toBeTruthy();
  });

  it("총합 0일 때 0% 표시", () => {
    render(<DonutChart focus={0} neutral={0} distraction={0} />);
    expect(screen.getByText("0%")).toBeTruthy();
  });

  it("총합 0일 때 배경 원만 렌더링 (세그먼트 없음)", () => {
    const { container } = render(
      <DonutChart focus={0} neutral={0} distraction={0} />
    );
    // 배경 원 + 빈 상태 원 = 2개 (세그먼트 없음)
    const circles = container.querySelectorAll("circle");
    expect(circles.length).toBe(2);
  });

  it("focus 100%일 때 focus 세그먼트만 렌더링", () => {
    const { container } = render(
      <DonutChart focus={100} neutral={0} distraction={0} />
    );
    expect(screen.getByText("100%")).toBeTruthy();
    // 배경 + focus = 2개
    const circles = container.querySelectorAll("circle");
    expect(circles.length).toBe(2);
  });

  it("size prop으로 SVG 크기 조정 가능", () => {
    const { container } = render(
      <DonutChart focus={50} neutral={30} distraction={20} size={120} />
    );
    const svg = container.querySelector("svg");
    expect(svg?.getAttribute("width")).toBe("120");
    expect(svg?.getAttribute("height")).toBe("120");
  });
});
