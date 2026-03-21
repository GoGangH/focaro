import { invoke } from "@tauri-apps/api/core";
import type { Reference, UpdateReferenceInput } from "../types/bindings";

export interface SaveReferenceInput {
  url: string;
  title: string;
  tags: string[] | null;
}

export async function saveReference(input: SaveReferenceInput): Promise<Reference> {
  return invoke<Reference>("save_reference", { input });
}

export async function getReferences(sessionId?: string): Promise<Reference[]> {
  return invoke<Reference[]>("get_references", { sessionId: sessionId ?? null });
}

export async function deleteReference(id: string): Promise<void> {
  return invoke<void>("delete_reference", { id });
}

export async function updateReference(input: UpdateReferenceInput): Promise<Reference> {
  return invoke<Reference>("update_reference", { input });
}

export async function getCurrentTitle(): Promise<string | null> {
  return invoke<string | null>("get_current_title");
}
