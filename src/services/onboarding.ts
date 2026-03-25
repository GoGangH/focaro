import { invoke } from "@tauri-apps/api/core";

export type Profession = "developer" | "designer" | "marketer" | "student" | "other";

export interface TitleRule {
  id: number;
  domain: string;
  keyword: string;
  category: string;
}

export async function getOnboardingStatus(): Promise<boolean> {
  return invoke<boolean>("get_onboarding_status");
}

export async function completeOnboarding(): Promise<void> {
  return invoke<void>("complete_onboarding");
}

export async function applyProfessionRules(profession: Profession): Promise<void> {
  return invoke<void>("apply_profession_rules", { profession });
}

export async function getTitleRules(): Promise<TitleRule[]> {
  return invoke<TitleRule[]>("get_title_rules");
}

export async function addTitleRule(
  domain: string,
  keyword: string,
  category: string
): Promise<TitleRule> {
  return invoke<TitleRule>("add_title_rule", { domain, keyword, category });
}

export async function deleteTitleRule(id: number): Promise<void> {
  return invoke<void>("delete_title_rule", { id });
}

export async function overrideActivityClassification(
  activityId: string,
  category: string
): Promise<void> {
  return invoke<void>("override_activity_classification", { activityId, category });
}
