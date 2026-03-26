import { invoke } from "@tauri-apps/api/core";

export interface WeeklyDayStat {
  date: string;
  focus_secs: number;
  total_secs: number;
}

export interface TrendPoint {
  date: string;
  focus_pct: number;
  focus_secs: number;
}

export async function getWeeklyReport(startDate: string): Promise<WeeklyDayStat[]> {
  return invoke<WeeklyDayStat[]>("get_weekly_report", { startDate });
}

export async function getTrend(days: number): Promise<TrendPoint[]> {
  return invoke<TrendPoint[]>("get_trend", { days });
}
