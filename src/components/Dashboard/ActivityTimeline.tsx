import type { Activity, Classification, SessionEvent } from "../../types/bindings";

function classColor(cls: Classification): string {
  if (cls === "Focus") return "#30d158";
  if (cls === "Distraction") return "#ff453a";
  return "#636366";
}

function formatDuration(secs: number | null): string {
  if (!secs || secs <= 0) return "0초";
  if (secs < 60) return `${secs}초`;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  if (m < 60) return s > 0 ? `${m}분 ${s}초` : `${m}분`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  return rm > 0 ? `${h}시간 ${rm}분` : `${h}시간`;
}

function formatTime(isoStr: string): string {
  const d = new Date(isoStr);
  return d.toLocaleTimeString("ko-KR", { hour: "2-digit", minute: "2-digit" });
}

type TimelineItem =
  | { kind: "activity"; data: Activity }
  | { kind: "event"; data: SessionEvent };

function eventLabel(type: SessionEvent["event_type"]): string {
  if (type === "start") return "세션 시작";
  if (type === "end") return "세션 종료";
  return "세션 비정상 종료";
}

function eventIcon(type: SessionEvent["event_type"]): string {
  if (type === "start") return "▶";
  if (type === "end") return "■";
  return "⚠";
}

function eventColor(type: SessionEvent["event_type"]): string {
  if (type === "start") return "#3b82f6";
  if (type === "end") return "#8e8e93";
  return "#ff9f0a";
}

interface Props {
  activities: Activity[];
  sessionEvents: SessionEvent[];
}

export function ActivityTimeline({ activities, sessionEvents }: Props) {
  // 활동과 세션 이벤트를 합쳐서 시간 역순 정렬
  const items: TimelineItem[] = [
    ...activities.map((a) => ({ kind: "activity" as const, data: a, ts: a.started_at })),
    ...sessionEvents.map((e) => ({ kind: "event" as const, data: e, ts: e.timestamp })),
  ]
    .sort((a, b) => new Date(b.ts).getTime() - new Date(a.ts).getTime())
    .map(({ kind, data }) =>
      kind === "activity"
        ? { kind: "activity" as const, data: data as Activity }
        : { kind: "event" as const, data: data as SessionEvent }
    );

  if (items.length === 0) {
    return <p className="dash-empty">활동 없음</p>;
  }

  return (
    <div className="timeline">
      {items.map((item, idx) => {
        if (item.kind === "event") {
          const ev = item.data;
          return (
            <div key={`evt-${ev.session_id}-${ev.event_type}`} className="timeline-item timeline-item--event">
              <span
                className="timeline-dot"
                style={{ background: eventColor(ev.event_type), fontSize: 10 }}
              >
                {eventIcon(ev.event_type)}
              </span>
              <div className="timeline-content">
                <div className="timeline-header">
                  <span className="timeline-event-label" style={{ color: eventColor(ev.event_type) }}>
                    {eventLabel(ev.event_type)}
                  </span>
                  <span className="timeline-time">{formatTime(ev.timestamp)}</span>
                </div>
              </div>
            </div>
          );
        }

        const act = item.data;
        const displayTitle = act.title || act.domain || act.app_name;
        const showSubtitle = act.title && (act.domain || act.app_name !== act.domain);

        return (
          <div key={act.id ?? idx} className="timeline-item">
            <span
              className="timeline-dot"
              style={{ background: classColor(act.classification) }}
            />
            <div className="timeline-content">
              <div className="timeline-header">
                <span className="timeline-app">{displayTitle}</span>
                <span className="timeline-time">{formatTime(act.started_at)}</span>
              </div>
              {showSubtitle && (
                <div className="timeline-domain">{act.domain ?? act.app_name}</div>
              )}
              {!act.title && act.domain && act.app_name !== act.domain && (
                <div className="timeline-domain">{act.app_name}</div>
              )}
              <div className="timeline-duration">{formatDuration(act.duration_secs)}</div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
