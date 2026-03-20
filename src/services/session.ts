import { invoke } from "@tauri-apps/api/core";
import type { Session } from "../types/bindings";

export async function startSession(): Promise<Session> {
  return invoke<Session>("start_session");
}

export async function endSession(): Promise<Session> {
  return invoke<Session>("end_session");
}

export async function getCurrentSession(): Promise<Session | null> {
  return invoke<Session | null>("get_current_session");
}

export async function getIncompleteSession(): Promise<Session | null> {
  return invoke<Session | null>("get_incomplete_session");
}

export async function resumeSession(sessionId: string): Promise<Session> {
  return invoke<Session>("resume_session", { sessionId });
}

export async function discardIncompleteSession(sessionId: string): Promise<void> {
  return invoke<void>("discard_incomplete_session", { sessionId });
}

export async function openDashboard(): Promise<void> {
  return invoke<void>("open_dashboard");
}
