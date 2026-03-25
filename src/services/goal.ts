import { invoke } from "@tauri-apps/api/core";

export interface DailyGoal {
  date: string;
  target_secs: number;
}

export interface GoalProgress {
  date: string;
  target_secs: number;
  actual_focus_secs: number;
  progress_pct: number;
}

export async function getDailyGoal(date?: string): Promise<DailyGoal> {
  return invoke<DailyGoal>("get_daily_goal", { date });
}

export async function setDailyGoal(target_secs: number, date?: string): Promise<DailyGoal> {
  return invoke<DailyGoal>("set_daily_goal", { targetSecs: target_secs, date });
}

export async function getGoalProgress(date?: string): Promise<GoalProgress> {
  return invoke<GoalProgress>("get_goal_progress", { date });
}
