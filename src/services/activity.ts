import { invoke } from "@tauri-apps/api/core";
import type { Activity, DomainSummary, FocusMetrics, SessionEvent } from "../types/bindings";

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
