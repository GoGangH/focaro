import { invoke } from "@tauri-apps/api/core";
import type { Reference } from "../types/bindings";

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
