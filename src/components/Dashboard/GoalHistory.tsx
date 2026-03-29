import { useEffect, useState } from "react";
import type { GoalHistoryEntry } from "../../types/bindings";
import { getGoalHistory } from "../../services/activity";

function formatMins(secs: number): string {
  const m = Math.floor(secs / 60);
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  return rm > 0 ? `${h}h ${rm}m` : `${h}h`;
}

export function GoalHistory() {
  const [history, setHistory] = useState<GoalHistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getGoalHistory(30)
      .then(setHistory)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <p className="dash-loading">로딩 중...</p>;
  if (history.length === 0) {
    return (
      <p className="pattern-empty">
        아직 목표 달성 기록이 없습니다. 세션을 완료하면 여기에 표시됩니다.
      </p>
    );
  }

  // 연속 달성 일수 계산 (최근부터 역순으로 연속된 달성 수)
  const sorted = [...history].sort((a, b) => b.date.localeCompare(a.date));
  let streak = 0;
  for (const entry of sorted) {
    if (entry.achieved) streak++;
    else break;
  }

  const total = history.length;
  const achieved = history.filter((e) => e.achieved).length;

  return (
    <div className="goal-history">
      <div className="goal-history__stats">
        <div className="goal-history__stat">
          <span className="goal-history__stat-value" style={{ color: "#ffd60a" }}>{streak}일</span>
          <span className="goal-history__stat-label">연속 달성</span>
        </div>
        <div className="goal-history__stat">
          <span className="goal-history__stat-value">{achieved}/{total}</span>
          <span className="goal-history__stat-label">달성 / 전체</span>
        </div>
        <div className="goal-history__stat">
          <span className="goal-history__stat-value">
            {total > 0 ? Math.round((achieved / total) * 100) : 0}%
          </span>
          <span className="goal-history__stat-label">달성률</span>
        </div>
      </div>

      <div className="goal-history__calendar">
        {history.map((entry) => (
          <div
            key={entry.date}
            className={`goal-history__day${entry.achieved ? " goal-history__day--achieved" : ""}`}
            title={`${entry.date} — ${formatMins(entry.actual_secs)} / ${formatMins(entry.target_secs)}${entry.achieved ? " ✓" : ""}`}
          >
            <span className="goal-history__day-label">{entry.date.slice(5)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
