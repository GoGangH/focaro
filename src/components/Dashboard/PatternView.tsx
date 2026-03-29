import { useEffect, useState } from "react";
import type { HeatmapCell, WeekdayStat } from "../../types/bindings";
import { getHourlyHeatmap, getWeekdayStats } from "../../services/activity";

const WEEKDAYS = ["월", "화", "수", "목", "금", "토", "일"];
const HOURS = Array.from({ length: 24 }, (_, i) => i);

function focusColor(pct: number, hasData: boolean): string {
  if (!hasData) return "rgba(255,255,255,0.04)";
  if (pct >= 80) return "rgba(48,209,88,0.85)";
  if (pct >= 60) return "rgba(48,209,88,0.55)";
  if (pct >= 40) return "rgba(48,209,88,0.30)";
  if (pct >= 20) return "rgba(255,159,10,0.35)";
  return "rgba(255,69,58,0.30)";
}

export function PatternView() {
  const [heatmap, setHeatmap] = useState<HeatmapCell[]>([]);
  const [weekday, setWeekday] = useState<WeekdayStat[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([getHourlyHeatmap(30), getWeekdayStats(30)])
      .then(([hm, wd]) => {
        setHeatmap(hm);
        setWeekday(wd);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  // (weekday, hour) → focus_pct 조회용 맵
  const cellMap = new Map<string, number>();
  heatmap.forEach((c) => cellMap.set(`${c.weekday}-${c.hour}`, c.focus_pct));

  const maxFocusMins = Math.max(...weekday.map((s) => s.avg_focus_mins), 1);

  if (loading) {
    return <p className="dash-loading">로딩 중...</p>;
  }

  const hasAnyData = heatmap.length > 0;

  return (
    <div className="pattern-view">
      {/* 히트맵 */}
      <div className="pattern-section">
        <h3 className="pattern-title">시간대별 집중도 (최근 30일)</h3>
        {!hasAnyData ? (
          <p className="pattern-empty">데이터가 충분하지 않습니다. 세션을 진행하면 패턴이 표시됩니다.</p>
        ) : (
          <>
            <div className="heatmap-grid">
              {/* 시간 축 헤더 */}
              <div className="heatmap-row heatmap-row--header">
                <div className="heatmap-label" />
                {HOURS.map((h) => (
                  <div key={h} className="heatmap-hour-label">
                    {h % 3 === 0 ? `${h}시` : ""}
                  </div>
                ))}
              </div>
              {/* 요일별 행 */}
              {WEEKDAYS.map((wd, wdIdx) => (
                <div key={wdIdx} className="heatmap-row">
                  <div className="heatmap-label">{wd}</div>
                  {HOURS.map((h) => {
                    const pct = cellMap.get(`${wdIdx}-${h}`);
                    return (
                      <div
                        key={h}
                        className="heatmap-cell"
                        style={{ background: focusColor(pct ?? 0, pct !== undefined) }}
                        title={pct !== undefined ? `${wd} ${h}시 — Focus ${Math.round(pct)}%` : "데이터 없음"}
                      />
                    );
                  })}
                </div>
              ))}
            </div>
            {/* 범례 */}
            <div className="heatmap-legend">
              <span className="heatmap-legend__label">낮음</span>
              {[0, 20, 40, 60, 80].map((p) => (
                <div
                  key={p}
                  className="heatmap-legend__dot"
                  style={{ background: focusColor(p, true) }}
                />
              ))}
              <span className="heatmap-legend__label">높음</span>
            </div>
          </>
        )}
      </div>

      {/* 요일별 평균 집중 시간 */}
      <div className="pattern-section">
        <h3 className="pattern-title">요일별 평균 집중 시간 (최근 30일)</h3>
        {!hasAnyData ? (
          <p className="pattern-empty">데이터가 충분하지 않습니다.</p>
        ) : (
          <div className="weekday-chart">
            {weekday.map((s) => {
              const pct = (s.avg_focus_mins / maxFocusMins) * 100;
              const h = Math.floor(s.avg_focus_mins / 60);
              const m = Math.round(s.avg_focus_mins % 60);
              const label = h > 0 ? `${h}h ${m}m` : `${m}m`;
              return (
                <div key={s.weekday} className="weekday-bar-col">
                  <div className="weekday-bar-wrap">
                    <div
                      className="weekday-bar"
                      style={{ height: `${Math.max(pct, 2)}%` }}
                      title={label}
                    />
                  </div>
                  <span className="weekday-bar-label">{WEEKDAYS[s.weekday]}</span>
                  <span className="weekday-bar-value">{s.avg_focus_mins > 0 ? label : "—"}</span>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
