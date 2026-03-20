import { create } from "zustand";

export interface Session {
  id: string;
  started_at: string;
  ended_at: string | null;
  status: "Active" | "Completed" | "Incomplete";
}

export interface Activity {
  id: string;
  session_id: string;
  app_name: string;
  url: string | null;
  domain: string | null;
  timestamp: string;
  duration_secs: number | null;
  classification: "Focus" | "Neutral" | "Distraction";
}

export interface FocusMetrics {
  session_id: string;
  total_secs: number;
  focus_secs: number;
  neutral_secs: number;
  distraction_secs: number;
  focus_percentage: number;
  neutral_percentage: number;
  distraction_percentage: number;
  top_domains: { domain: string; duration_secs: number; classification: string }[];
}

interface AppStore {
  currentSession: Session | null;
  currentActivity: Activity | null;
  metrics: FocusMetrics | null;
  setCurrentSession: (session: Session | null) => void;
  setCurrentActivity: (activity: Activity | null) => void;
  setMetrics: (metrics: FocusMetrics | null) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  currentSession: null,
  currentActivity: null,
  metrics: null,
  setCurrentSession: (session) => set({ currentSession: session }),
  setCurrentActivity: (activity) => set({ currentActivity: activity }),
  setMetrics: (metrics) => set({ metrics }),
}));
