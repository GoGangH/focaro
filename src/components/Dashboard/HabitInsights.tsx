import { useEffect, useState } from "react";
import { getTrend, type TrendPoint } from "../../services/stats";

const DAY_KR = ["일", "월", "화", "수", "목", "금", "토"];

function formatHours(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h === 0) return `${m}분`;
  return m > 0 ? `${h}시간 ${m}분` : `${h}시간`;
}

function analyzeInsights(points: TrendPoint[]) {
  if (points.length === 0) return null;

  // Average focus %
  const avgPct = points.reduce((s, p) => s + p.focus_pct, 0) / points.length;

  // Best & worst day
  const best = points.reduce((a, b) => (a.focus_pct > b.focus_pct ? a : b));
  const worst = points.reduce((a, b) => (a.focus_pct < b.focus_pct ? a : b));

  // Day-of-week aggregation
  const dayBuckets: { sum: number; count: number }[] = Array.from(
    { length: 7 },
    () => ({ sum: 0, count: 0 })
  );
  for (const p of points) {
    const dow = new Date(p.date).getDay();
    dayBuckets[dow].sum += p.focus_pct;
    dayBuckets[dow].count += 1;
  }
  const dayAvg = dayBuckets.map((b) => (b.count > 0 ? b.sum / b.count : -1));
  const validDays = dayAvg.map((a, i) => ({ avg: a, i })).filter((d) => d.avg >= 0);
  const bestDow = validDays.length > 0
    ? validDays.reduce((a, b) => (a.avg > b.avg ? a : b))
    : null;

  // Current streak (consecutive days with focus_pct >= 50)
  let streak = 0;
  for (let i = points.length - 1; i >= 0; i--) {
    if (points[i].focus_pct >= 50) streak++;
    else break;
  }

  // Total focus time
  const totalFocusSecs = points.reduce((s, p) => s + p.focus_secs, 0);

  return { avgPct, best, worst, bestDow, streak, totalFocusSecs };
}

export function HabitInsights() {
  const [points, setPoints] = useState<TrendPoint[]>([]);

  useEffect(() => {
    getTrend(30).then(setPoints);
  }, []);

  const insights = analyzeInsights(points);

  if (!insights) {
    return <p className="dash-empty">최근 30일 데이터가 없습니다</p>;
  }

  const { avgPct, best, bestDow, streak, totalFocusSecs } = insights;

  return (
    <div className="habit-insights">
      <h3 className="habit-insights__title">30일 패턴 분석</h3>

      <div className="habit-insights__grid">
        <div className="habit-card">
          <div className="habit-card__value">{avgPct.toFixed(0)}%</div>
          <div className="habit-card__label">평균 Focus율</div>
        </div>

        <div className="habit-card">
          <div className="habit-card__value">{formatHours(totalFocusSecs)}</div>
          <div className="habit-card__label">총 집중 시간</div>
        </div>

        <div className="habit-card">
          <div className="habit-card__value">{streak}일</div>
          <div className="habit-card__label">Focus 연속 달성</div>
          <div className="habit-card__sub">(50% 이상)</div>
        </div>

        {bestDow && (
          <div className="habit-card">
            <div className="habit-card__value">{DAY_KR[bestDow.i]}요일</div>
            <div className="habit-card__label">가장 집중한 요일</div>
            <div className="habit-card__sub">{bestDow.avg.toFixed(0)}% 평균</div>
          </div>
        )}
      </div>

      {best.focus_pct > 0 && (
        <div className="habit-insights__best">
          <span className="habit-insights__best-label">최고 기록</span>
          <span className="habit-insights__best-date">
            {new Date(best.date).toLocaleDateString("ko-KR")}
          </span>
          <span className="habit-insights__best-pct">
            {best.focus_pct.toFixed(0)}%
          </span>
        </div>
      )}
    </div>
  );
}
