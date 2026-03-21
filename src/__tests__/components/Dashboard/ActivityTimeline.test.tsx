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
    title: "GitHub",
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
    title: null,
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
    title: "Some Video - YouTube",
    classification: "Distraction",
    started_at: "2026-03-21T10:01:30Z",
    duration_secs: 45,
  },
];

describe("ActivityTimeline", () => {
  it("활동 목록을 렌더링", () => {
    render(<ActivityTimeline activities={ACTIVITIES} sessionEvents={[]} />);
    // title이 있으면 title을, 없으면 app_name 표시
    expect(screen.getByText("GitHub")).toBeTruthy();
    expect(screen.getByText("Slack")).toBeTruthy();
    expect(screen.getByText("Some Video - YouTube")).toBeTruthy();
  });

  it("빈 상태 메시지 표시", () => {
    render(<ActivityTimeline activities={[]} sessionEvents={[]} />);
    expect(screen.getByText(/활동 없음|no activity/i)).toBeTruthy();
  });

  it("도메인 표시", () => {
    render(<ActivityTimeline activities={ACTIVITIES} sessionEvents={[]} />);
    expect(screen.getByText("github.com")).toBeTruthy();
    expect(screen.getByText("youtube.com")).toBeTruthy();
  });

  it("classification별 색상 마커 렌더링", () => {
    const { container } = render(<ActivityTimeline activities={ACTIVITIES} sessionEvents={[]} />);
    const dots = container.querySelectorAll(".timeline-dot");
    expect(dots.length).toBe(3);
  });

  it("duration 표시", () => {
    render(<ActivityTimeline activities={ACTIVITIES} sessionEvents={[]} />);
    // 60초 = "1분"
    expect(screen.getByText("1분")).toBeTruthy();
  });

  it("세션 이벤트 렌더링", () => {
    render(
      <ActivityTimeline
        activities={[]}
        sessionEvents={[
          { session_id: "s1", event_type: "start", timestamp: "2026-03-21T09:00:00Z" },
          { session_id: "s1", event_type: "end", timestamp: "2026-03-21T11:00:00Z" },
        ]}
      />
    );
    expect(screen.getByText("세션 시작")).toBeTruthy();
    expect(screen.getByText("세션 종료")).toBeTruthy();
  });
});
