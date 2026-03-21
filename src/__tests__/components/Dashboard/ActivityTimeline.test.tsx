import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ActivityTimeline } from "../../../components/Dashboard/ActivityTimeline";
import type { Activity } from "../../../types/bindings";

const ACTIVITIES: Activity[] = [
  {
    id: "a1",
    session_id: "s1",
    app_name: "Google Chrome",
    url: "https://github.com",
    domain: "github.com",
    classification: "Focus",
    started_at: "2026-03-21T10:00:00Z",
    duration_secs: 60,
  },
  {
    id: "a2",
    session_id: "s1",
    app_name: "Slack",
    url: null,
    domain: null,
    classification: "Neutral",
    started_at: "2026-03-21T10:01:00Z",
    duration_secs: 30,
  },
  {
    id: "a3",
    session_id: "s1",
    app_name: "YouTube",
    url: "https://youtube.com/watch?v=xxx",
    domain: "youtube.com",
    classification: "Distraction",
    started_at: "2026-03-21T10:01:30Z",
    duration_secs: 45,
  },
];

describe("ActivityTimeline", () => {
  it("활동 목록을 렌더링", () => {
    render(<ActivityTimeline activities={ACTIVITIES} />);
    expect(screen.getByText("Google Chrome")).toBeTruthy();
    expect(screen.getByText("Slack")).toBeTruthy();
    expect(screen.getByText("YouTube")).toBeTruthy();
  });

  it("빈 상태 메시지 표시", () => {
    render(<ActivityTimeline activities={[]} />);
    expect(screen.getByText(/활동 없음|no activity/i)).toBeTruthy();
  });

  it("도메인 표시", () => {
    render(<ActivityTimeline activities={ACTIVITIES} />);
    expect(screen.getByText("github.com")).toBeTruthy();
    expect(screen.getByText("youtube.com")).toBeTruthy();
  });

  it("classification별 색상 마커 렌더링", () => {
    const { container } = render(<ActivityTimeline activities={ACTIVITIES} />);
    const dots = container.querySelectorAll(".timeline-dot");
    expect(dots.length).toBe(3);
  });

  it("duration 표시", () => {
    render(<ActivityTimeline activities={ACTIVITIES} />);
    // 60초 = "1분"
    expect(screen.getByText("1분")).toBeTruthy();
  });
});
