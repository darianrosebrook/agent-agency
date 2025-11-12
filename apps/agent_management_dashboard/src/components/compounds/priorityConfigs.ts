import type { PriorityConfig } from "./PriorityIndicator";

export type Priority = "low" | "medium" | "high";

export const priorityConfig: Record<Priority, PriorityConfig> = {
  low: { label: "Low", color: "", icon: "▼" }, // Now handled by SCSS module classes
  medium: { label: "Medium", color: "", icon: "▲" }, // Now handled by SCSS module classes
  high: { label: "High", color: "", icon: "▲▲" }, // Now handled by SCSS module classes
};









