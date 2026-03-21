import type { Activity, Classification } from "../../types/bindings";

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

interface Props {
  activities: Activity[];
}

export function ActivityTimeline({ activities }: Props) {
  if (activities.length === 0) {
    return <p className="dash-empty">활동 없음</p>;
  }

  return (
    <div className="timeline">
      {activities.map((act) => (
        <div key={act.id} className="timeline-item">
          <span
            className="timeline-dot"
            style={{ background: classColor(act.classification) }}
          />
          <div className="timeline-content">
            <div className="timeline-header">
              <span className="timeline-app">{act.app_name}</span>
              <span className="timeline-time">{formatTime(act.started_at)}</span>
            </div>
            {act.domain && (
              <div className="timeline-domain">{act.domain}</div>
            )}
            <div className="timeline-duration">{formatDuration(act.duration_secs)}</div>
          </div>
        </div>
      ))}
    </div>
  );
}
