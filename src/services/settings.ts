import { invoke } from "@tauri-apps/api/core";

export interface AppSettings {
  retention_days: number;
  shortcut_save_ref: string;
}

export interface ClassificationRule {
  id: number;
  domain: string;
  category: string;
}

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function updateSettings(settings: AppSettings): Promise<void> {
  return invoke("update_settings", { settings });
}

export async function getClassificationRules(): Promise<ClassificationRule[]> {
  return invoke<ClassificationRule[]>("get_classification_rules");
}

export async function addClassificationRule(
  domain: string,
  category: string
): Promise<ClassificationRule> {
  return invoke<ClassificationRule>("add_classification_rule", { domain, category });
}

export async function deleteClassificationRule(id: number): Promise<void> {
  return invoke("delete_classification_rule", { id });
}

export async function openSettingsWindow(): Promise<void> {
  return invoke("open_settings_window");
}

export async function openSaveReferenceWindow(): Promise<void> {
  return invoke("open_save_reference_window");
}
