// 자동 생성 바인딩 (tauri-specta) — 수동 편집 금지
// Rust 모델과 1:1 대응

export type SessionStatus = "Active" | "Completed" | "Incomplete";

export interface Session {
  id: string;
  started_at: string;
  ended_at: string | null;
  status: SessionStatus;
}

export type Classification = "Focus" | "Neutral" | "Distraction";

export interface Activity {
  id: string;
  session_id: string;
  app_name: string;
  url: string | null;
  domain: string | null;
  classification: Classification;
  started_at: string;
  duration_secs: number;
}

export interface FocusStats {
  total_secs: number;
  focus_secs: number;
  neutral_secs: number;
  distraction_secs: number;
}

export interface AppStat {
  app_name: string;
  duration_secs: number;
  classification: Classification;
  percentage: number;
}

export interface FocusMetrics {
  total_secs: number;
  focus_secs: number;
  neutral_secs: number;
  distraction_secs: number;
  focus_percentage: number;
  neutral_percentage: number;
  distraction_percentage: number;
}

export interface DomainSummary {
  domain: string;
  total_secs: number;
  classification: Classification;
}

export interface Reference {
  id: string;
  session_id: string;
  url: string;
  title: string;
  tags: string | null;
  created_at: string;
}

export interface SaveReferenceInput {
  session_id: string;
  url: string;
  title: string;
  tags: string | null;
}

export interface AppSettings {
  retention_days: number;
}
