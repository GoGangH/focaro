import { useEffect, useState } from "react";
import { getDailyGoal, getGoalProgress, setDailyGoal, type GoalProgress as GoalProgressData } from "../../services/goal";
import { getGoalHistory } from "../../services/activity";

const PRESET_HOURS = [1, 2, 3, 4, 6];

function formatSecs(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m > 0 ? `${m}m` : ""}`.trim();
  return `${m}m`;
}

export function GoalProgress() {
  const [progress, setProgress] = useState<GoalProgressData | null>(null);
  const [editing, setEditing] = useState(false);
  const [selectedHours, setSelectedHours] = useState(2);
  const [streak, setStreak] = useState(0);

  useEffect(() => {
    getGoalProgress().then(setProgress);
    getDailyGoal().then((g) => setSelectedHours(Math.round(g.target_secs / 3600)));
    getGoalHistory(30).then((hist) => {
      const sorted = [...hist].sort((a, b) => b.date.localeCompare(a.date));
      let s = 0;
      for (const e of sorted) {
        if (e.achieved) s++;
        else break;
      }
      setStreak(s);
    }).catch(() => {});
  }, []);

  const handleSave = async () => {
    await setDailyGoal(selectedHours * 3600);
    const updated = await getGoalProgress();
    setProgress(updated);
    setEditing(false);
  };

  if (!progress) return null;

  const pct = Math.min(Math.round(progress.progress_pct), 100);
  const isAchieved = pct >= 100;

  return (
    <div className="goal-progress">
      <div className="goal-progress__header">
        <span className="goal-progress__label">
          오늘 목표
          {streak > 0 && (
            <span className="goal-progress__streak" title={`${streak}일 연속 달성`}>
              {" "}🔥 {streak}
            </span>
          )}
        </span>
        <button
          className="goal-progress__edit-btn"
          onClick={() => setEditing((v) => !v)}
        >
          {editing ? "취소" : "변경"}
        </button>
      </div>

      {editing ? (
        <div className="goal-progress__edit">
          <div className="goal-progress__presets">
            {PRESET_HOURS.map((h) => (
              <button
                key={h}
                className={`goal-progress__preset${selectedHours === h ? " active" : ""}`}
                onClick={() => setSelectedHours(h)}
              >
                {h}h
              </button>
            ))}
          </div>
          <button className="goal-progress__save-btn" onClick={handleSave}>
            저장
          </button>
        </div>
      ) : (
        <>
          <div className="goal-progress__bar-wrap">
            <div
              className="goal-progress__bar"
              style={{
                width: `${pct}%`,
                background: isAchieved ? "#30d158" : "#0a84ff",
              }}
            />
          </div>
          <div className="goal-progress__info">
            <span style={{ color: isAchieved ? "#30d158" : "#fff" }}>
              {formatSecs(progress.actual_focus_secs)}
            </span>
            <span className="goal-progress__target">
              / {formatSecs(progress.target_secs)} ({pct}%)
            </span>
          </div>
        </>
      )}
    </div>
  );
}
