import { useEffect, useState } from "react";
import { getWeeklyReport, type WeeklyDayStat } from "../../services/stats";

const DAY_LABELS = ["월", "화", "수", "목", "금", "토", "일"];

function getMonday(date: Date): string {
  const d = new Date(date);
  const day = d.getDay();
  const diff = day === 0 ? -6 : 1 - day;
  d.setDate(d.getDate() + diff);
  return d.toISOString().slice(0, 10);
}

function formatHours(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h === 0) return `${m}m`;
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

interface Props {
  weekOffset?: number; // 0 = 이번주, -1 = 지난주
}

export function WeeklyReport({ weekOffset = 0 }: Props) {
  const [stats, setStats] = useState<WeeklyDayStat[]>([]);
  const [startDate, setStartDate] = useState("");

  useEffect(() => {
    const base = new Date();
    base.setDate(base.getDate() + weekOffset * 7);
    const monday = getMonday(base);
    setStartDate(monday);
    getWeeklyReport(monday).then(setStats);
  }, [weekOffset]);

  // 7일 슬롯 생성 (데이터 없는 날은 0으로)
  const slots = Array.from({ length: 7 }, (_, i) => {
    const d = new Date(startDate);
    d.setDate(d.getDate() + i);
    const dateStr = d.toISOString().slice(0, 10);
    const found = stats.find((s) => s.date === dateStr);
    return { label: DAY_LABELS[i], date: dateStr, focus_secs: found?.focus_secs ?? 0 };
  });

  const maxSecs = Math.max(...slots.map((s) => s.focus_secs), 1);
  const today = new Date().toISOString().slice(0, 10);

  return (
    <div className="weekly-report">
      <div className="weekly-report__bars">
        {slots.map((slot) => {
          const pct = (slot.focus_secs / maxSecs) * 100;
          const isToday = slot.date === today;
          return (
            <div key={slot.date} className="weekly-report__col">
              <div className="weekly-report__bar-wrap">
                <div
                  className="weekly-report__bar"
                  style={{
                    height: `${pct}%`,
                    background: isToday ? "#30d158" : "#0a84ff",
                    opacity: slot.focus_secs === 0 ? 0.2 : 1,
                  }}
                  title={slot.focus_secs > 0 ? formatHours(slot.focus_secs) : "기록 없음"}
                />
              </div>
              <span className={`weekly-report__day${isToday ? " today" : ""}`}>
                {slot.label}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
