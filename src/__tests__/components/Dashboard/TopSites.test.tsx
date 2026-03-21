import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { TopSites } from "../../../components/Dashboard/TopSites";
import type { DomainSummary } from "../../../types/bindings";

const SITES: DomainSummary[] = [
  { domain: "github.com", total_secs: 3600, classification: "Focus" },
  { domain: "youtube.com", total_secs: 1800, classification: "Distraction" },
  { domain: "notion.so", total_secs: 900, classification: "Focus" },
];

describe("TopSites", () => {
  it("도메인 목록 렌더링", () => {
    render(<TopSites sites={SITES} />);
    expect(screen.getByText("github.com")).toBeTruthy();
    expect(screen.getByText("youtube.com")).toBeTruthy();
    expect(screen.getByText("notion.so")).toBeTruthy();
  });

  it("빈 상태 메시지 표시", () => {
    render(<TopSites sites={[]} />);
    expect(screen.getByText(/사이트 없음|no sites/i)).toBeTruthy();
  });

  it("시간을 사람이 읽기 좋은 포맷으로 표시", () => {
    render(<TopSites sites={SITES} />);
    // 3600초 = "1h 00m" 또는 "60분"
    expect(screen.getByText(/1h|60분|1시간/)).toBeTruthy();
  });

  it("classification별 색상 마커 렌더링", () => {
    const { container } = render(<TopSites sites={SITES} />);
    const dots = container.querySelectorAll(".site-dot");
    expect(dots.length).toBe(3);
  });

  it("비율 바 렌더링", () => {
    const { container } = render(<TopSites sites={SITES} />);
    const bars = container.querySelectorAll(".site-bar");
    expect(bars.length).toBe(3);
  });
});
