import { useState, useEffect, useCallback } from "react";
import type { Activity, DomainSummary, FocusMetrics, Reference, SessionEvent } from "../types/bindings";
import { getActivityTimeline, getTopSites, getDailyFocusStats, getSessionEvents } from "../services/activity";
import { getReferences } from "../services/reference";
import { ActivityTimeline } from "../components/Dashboard/ActivityTimeline";
import { TopSites } from "../components/Dashboard/TopSites";
import { FocusScore } from "../components/Dashboard/FocusScore";
import { SavedReferences } from "../components/Dashboard/SavedReferences";
import { DatePicker } from "../components/Dashboard/DatePicker";
import { WeeklyReport } from "../components/Dashboard/WeeklyReport";
import { TrendChart } from "../components/Dashboard/TrendChart";
import { HabitInsights } from "../components/Dashboard/HabitInsights";

function todayDateStr(): string {
  return new Date().toISOString().split("T")[0];
}

function shiftDate(dateStr: string, days: number): string {
  const d = new Date(dateStr);
  d.setDate(d.getDate() + days);
  return d.toISOString().split("T")[0];
}

type Tab = "timeline" | "sites" | "score" | "refs" | "weekly" | "trend";

export function Dashboard() {
  const [date, setDate] = useState(todayDateStr);
  const [tab, setTab] = useState<Tab>("timeline");
  const [activities, setActivities] = useState<Activity[]>([]);
  const [sessionEvents, setSessionEvents] = useState<SessionEvent[]>([]);
  const [sites, setSites] = useState<DomainSummary[]>([]);
  const [metrics, setMetrics] = useState<FocusMetrics | null>(null);
  const [refs, setRefs] = useState<Reference[]>([]);
  const [loading, setLoading] = useState(false);

  const loadData = useCallback(async (d: string) => {
    setLoading(true);
    try {
      const [acts, events, topSites, focusStats, references] = await Promise.all([
        getActivityTimeline(d),
        getSessionEvents(d),
        getTopSites(d, 20),
        getDailyFocusStats(d),
        getReferences(),
      ]);
      setActivities(acts);
      setSessionEvents(events);
      setSites(topSites);
      setMetrics(focusStats);
      setRefs(references);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData(date);
  }, [date, loadData]);

  const TABS: { id: Tab; label: string }[] = [
    { id: "timeline", label: "타임라인" },
    { id: "sites", label: "Top Sites" },
    { id: "score", label: "Focus Score" },
    { id: "weekly", label: "주간 리포트" },
    { id: "trend", label: "트렌드" },
    { id: "refs", label: "References" },
  ];

  // Date nav only relevant for day-based tabs
  const isDayTab = tab === "timeline" || tab === "sites" || tab === "score";

  return (
    <div className="dashboard">
      <div className="dash-header">
        <h1 className="dash-title">Dashboard</h1>
        {isDayTab && (
          <div className="dash-date-nav">
            <button
              className="dash-date-btn"
              onClick={() => setDate((d) => shiftDate(d, -1))}
              aria-label="이전 날"
            >
              ‹
            </button>
            <DatePicker value={date} max={todayDateStr()} onChange={setDate} />
            <button
              className="dash-date-btn"
              onClick={() => setDate((d) => shiftDate(d, 1))}
              disabled={date >= todayDateStr()}
              aria-label="다음 날"
            >
              ›
            </button>
          </div>
        )}
      </div>

      <div className="dash-tabs">
        {TABS.map((t) => (
          <button
            key={t.id}
            className={`dash-tab${tab === t.id ? " dash-tab--active" : ""}`}
            onClick={() => setTab(t.id)}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div className="dash-content">
        {loading && isDayTab ? (
          <p className="dash-loading">로딩 중...</p>
        ) : (
          <>
            {tab === "timeline" && <ActivityTimeline activities={activities} sessionEvents={sessionEvents} />}
            {tab === "sites" && <TopSites sites={sites} />}
            {tab === "score" && <FocusScore metrics={metrics} />}
            {tab === "weekly" && (
              <div className="weekly-section">
                <div className="weekly-section__block">
                  <h3 className="weekly-section__subtitle">이번 주</h3>
                  <WeeklyReport weekOffset={0} />
                </div>
                <div className="weekly-section__block">
                  <h3 className="weekly-section__subtitle">지난 주</h3>
                  <WeeklyReport weekOffset={-1} />
                </div>
              </div>
            )}
            {tab === "trend" && (
              <div className="trend-section">
                <TrendChart />
                <HabitInsights />
              </div>
            )}
            {tab === "refs" && (
              <SavedReferences
                references={refs}
                onRefresh={() => getReferences().then(setRefs)}
              />
            )}
          </>
        )}
      </div>
    </div>
  );
}
