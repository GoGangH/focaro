import { invoke } from "@tauri-apps/api/core";

export type ExportFormat = "csv" | "json";

export async function exportData(
  startDate: string,
  endDate: string,
  format: ExportFormat,
): Promise<string> {
  return invoke<string>("export_data", { startDate, endDate, format });
}
