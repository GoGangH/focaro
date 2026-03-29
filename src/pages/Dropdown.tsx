import { useEffect, useState, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Session, FocusStats, AppStat } from "../types/bindings";
import { useSession } from "../hooks/useSession";
import {
  getIncompleteSession,
  resumeSession,
  discardIncompleteSession,
  openDashboard,
  getFocusStats,
  getTopApps,
  getCurrentApp,
  getCurrentUrl,
  getCurrentTitle,
  checkAccessibilityPermission,
} from "../services/session";
import { DonutChart } from "../components/Dropdown/DonutChart";
import { GoalProgress } from "../components/Dropdown/GoalProgress";
import { QuickOverride } from "../components/Dropdown/QuickOverride";
import { openSaveReferenceWindow, openSettingsWindow } from "../services/settings";

function formatTimer(totalSecs: number): string {
  const h = Math.floor(totalSecs / 3600);
  const m = Math.floor((totalSecs % 3600) / 60);
  const s = totalSecs % 60;
  if (h > 0) return `${h}h ${String(m).padStart(2, "0")}m`;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function classColor(cls: string): string {
  if (cls === "Focus") return "#30d158";
  if (cls === "Distraction") return "#ff453a";
  return "#636366";
}

export function Dropdown() {
  const { session, setSession, isLoading, startSession, endSession } = useSession();
  const [incompleteSession, setIncompleteSession] = useState<Session | null>(null);
  const [recoveryChecked, setRecoveryChecked] = useState(false);
  const [stats, setStats] = useState<FocusStats | null>(null);
  const [topApps, setTopApps] = useState<AppStat[]>([]);
  const [currentApp, setCurrentApp] = useState<string | null>(null);
  const [elapsed, setElapsed] = useState(0);
  const [currentUrl, setCurrentUrl] = useState<string | null>(null);
  const [currentTitle, setCurrentTitle] = useState<string | null>(null);
  const [accessibilityGranted, setAccessibilityGranted] = useState(true);
  const [showQuickOverride, setShowQuickOverride] = useState(false);

  // Recovery + 권한 체크 on mount
  useEffect(() => {
    if (recoveryChecked) return;
    setRecoveryChecked(true);
    getIncompleteSession().then((s) => { if (s) setIncompleteSession(s); });
    checkAccessibilityPermission().then((granted) => setAccessibilityGranted(granted));
  }, [recoveryChecked]);

  // Poll stats every 5s when session active
  const refreshStats = useCallback(async (sid: string) => {
    const [s, apps, app, url, title] = await Promise.all([
      getFocusStats(sid),
      getTopApps(sid),
      getCurrentApp(),
      getCurrentUrl(),
      getCurrentTitle(),
    ]);
    setStats(s);
    setTopApps(apps);
    setCurrentApp(app);
    setCurrentUrl(url);
    setCurrentTitle(title);
    setElapsed(s.total_secs);
  }, []);

  useEffect(() => {
    if (!session) { setStats(null); setTopApps([]); setElapsed(0); return; }
    refreshStats(session.id);
    const interval = setInterval(() => refreshStats(session.id), 3000);
    return () => clearInterval(interval);
  }, [session, refreshStats]);

  // Local timer tick every second
  useEffect(() => {
    if (!session) return;
    const tick = setInterval(() => setElapsed((e) => e + 1), 1000);
    return () => clearInterval(tick);
  }, [session]);

  const handleResume = async () => {
    if (!incompleteSession) return;
    const resumed = await resumeSession(incompleteSession.id);
    setSession(resumed);
    setIncompleteSession(null);
    await getCurrentWindow().hide();
  };

  const handleDiscard = async () => {
    if (!incompleteSession) return;
    await discardIncompleteSession(incompleteSession.id);
    setIncompleteSession(null);
    await getCurrentWindow().hide();
  };

  // Recovery dialog
  if (incompleteSession) {
    return (
      <div className="dropdown dropdown--recovery">
        <p className="recovery__title">이전 세션이 종료되지 않았습니다</p>
        <p className="recovery__subtitle">이어서 진행할까요?</p>
        <div className="recovery__actions">
          <button onClick={handleResume} className="session-btn session-btn--start">이어가기</button>
          <button onClick={handleDiscard} className="session-btn session-btn--end">종료</button>
        </div>
      </div>
    );
  }

  const focusSecs = stats?.focus_secs ?? 0;
  const neutralSecs = stats?.neutral_secs ?? 0;
  const distractionSecs = stats?.distraction_secs ?? 0;
  const totalSecs = stats?.total_secs ?? 0;
  const focusPct = totalSecs > 0 ? Math.round((focusSecs / totalSecs) * 100) : 0;

  return (
    <div className="dropdown">
      {/* Accessibility 권한 없을 때 안내 배너 */}
      {!accessibilityGranted && (
        <div className="dd-permission-banner">
          <span>⚠️</span>
          <span>활동 추적을 위해 손쉬운 사용 권한이 필요합니다.</span>
        </div>
      )}

      {/* Header: timer + focus % */}
      <div className="dd-header">
        <div className="dd-timer">{session ? formatTimer(elapsed) : "--:--"}</div>
        {session && (
          <div className="dd-focus-pct" style={{ color: "#30d158" }}>
            Focus {focusPct}%
          </div>
        )}
      </div>

      {/* Session button */}
      <button
        className={`session-btn ${session ? "session-btn--end" : "session-btn--start"}`}
        disabled={isLoading}
        onClick={session ? endSession : startSession}
      >
        {session ? "세션 종료" : "세션 시작"}
      </button>

      {/* Donut chart + current app */}
      {session && (
        <div className="dd-middle">
          <DonutChart
            focus={focusSecs}
            neutral={neutralSecs}
            distraction={distractionSecs}
          />
          <div className="dd-current-app">
            <div className="dd-current-app__label">현재 앱</div>
            <div className="dd-current-app__name">{currentApp ?? "—"}</div>
            {currentUrl && (
              <button
                className="dd-current-app__override-btn"
                onClick={() => setShowQuickOverride((v) => !v)}
                title="분류 변경"
              >
                분류 변경
              </button>
            )}
          </div>
        </div>
      )}

      {/* Quick override panel */}
      {session && showQuickOverride && currentUrl && (
        <QuickOverride
          domain={currentUrl ? new URL(currentUrl).hostname.replace(/^www\./, "") : null}
          title={currentTitle}
          currentCategory={topApps[0]?.classification ?? "Neutral"}
          onClose={() => setShowQuickOverride(false)}
        />
      )}

      {/* 오늘 집중 목표 */}
      {session && <GoalProgress />}

      {/* Recent apps list */}
      {session && topApps.length > 0 && (
        <div className="dd-apps">
          <div className="dd-apps__title">최근 활동</div>
          {topApps.slice(0, 5).map((app) => (
            <div key={app.app_name} className="dd-app-row">
              <span className="dd-app-row__dot" style={{ background: classColor(app.classification) }} />
              <span className="dd-app-row__name">{app.app_name}</span>
              <span className="dd-app-row__pct">{Math.round(app.percentage)}%</span>
            </div>
          ))}
        </div>
      )}

      {/* Reference 저장 (세션 진행 중일 때만) */}
      {session && (
        <button
          className={`dashboard-btn${currentUrl ? " dashboard-btn--active" : ""}`}
          onClick={openSaveReferenceWindow}
          disabled={!currentUrl}
        >
          Reference 저장
        </button>
      )}

      {/* Footer */}
      <div className="dd-footer">
        <button onClick={openDashboard} className="dashboard-btn">Dashboard 열기</button>
        <button onClick={openSettingsWindow} className="dashboard-btn">설정</button>
      </div>
    </div>
  );
}
