import { invoke } from "@tauri-apps/api/core";
import type { Activity, DomainSummary, FocusMetrics, SessionEvent, HeatmapCell, WeekdayStat } from "../types/bindings";

export async function getActivityTimeline(date: string): Promise<Activity[]> {
  return invoke<Activity[]>("get_activity_timeline", { date });
}

export async function getTopSites(date: string, limit: number = 10): Promise<DomainSummary[]> {
  return invoke<DomainSummary[]>("get_top_sites", { date, limit });
}

export async function getDailyFocusStats(date: string): Promise<FocusMetrics> {
  return invoke<FocusMetrics>("get_daily_focus_stats", { date });
}

export async function getSessionEvents(date: string): Promise<SessionEvent[]> {
  return invoke<SessionEvent[]>("get_session_events", { date });
}

export async function getTrackedApps(): Promise<string[]> {
  return invoke<string[]>("get_tracked_apps");
}

export async function getHourlyHeatmap(days: number = 30): Promise<HeatmapCell[]> {
  return invoke<HeatmapCell[]>("get_hourly_heatmap", { days });
}

export async function getWeekdayStats(days: number = 30): Promise<WeekdayStat[]> {
  return invoke<WeekdayStat[]>("get_weekday_stats", { days });
}
