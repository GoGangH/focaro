import type { FocusMetrics } from "../../types/bindings";

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}초`;
  const m = Math.floor(secs / 60);
  if (m < 60) return `${m}분`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  return rm > 0 ? `${h}h ${String(rm).padStart(2, "0")}m` : `${h}h 00m`;
}

interface Props {
  metrics: FocusMetrics | null;
}

export function FocusScore({ metrics }: Props) {
  if (!metrics || metrics.total_secs === 0) {
    return <p className="dash-empty">데이터 없음</p>;
  }

  return (
    <div className="focus-score">
      <div className="focus-score__main">
        <span
          className="focus-score__pct"
          style={{ color: "#30d158" }}
        >
          {Math.round(metrics.focus_percentage)}%
        </span>
        <span className="focus-score__label">집중</span>
      </div>
      <div className="focus-score__bars">
        <div className="focus-score__bar-row">
          <span className="focus-score__bar-label">Focus</span>
          <div className="focus-score__bar-bg">
            <div
              className="focus-score__bar"
              style={{ width: `${metrics.focus_percentage}%`, background: "#30d158" }}
            />
          </div>
          <span className="focus-score__bar-time">{formatDuration(metrics.focus_secs)}</span>
        </div>
        <div className="focus-score__bar-row">
          <span className="focus-score__bar-label">Neutral</span>
          <div className="focus-score__bar-bg">
            <div
              className="focus-score__bar"
              style={{ width: `${metrics.neutral_percentage}%`, background: "#636366" }}
            />
          </div>
          <span className="focus-score__bar-time">{formatDuration(metrics.neutral_secs)}</span>
        </div>
        <div className="focus-score__bar-row">
          <span className="focus-score__bar-label">Distract</span>
          <div className="focus-score__bar-bg">
            <div
              className="focus-score__bar"
              style={{ width: `${metrics.distraction_percentage}%`, background: "#ff453a" }}
            />
          </div>
          <span className="focus-score__bar-time">{formatDuration(metrics.distraction_secs)}</span>
        </div>
      </div>
      <div className="focus-score__total">총 {formatDuration(metrics.total_secs)}</div>
    </div>
  );
}
