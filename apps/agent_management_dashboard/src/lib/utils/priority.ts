/**
 * Priority Utilities
 * 
 * Converts between priority numbers (0-10) and human-readable labels.
 * 
 * @author @darianrosebrook
 */

/**
 * Priority label type
 */
export type PriorityLabel = "low" | "medium" | "high" | "critical";

/**
 * Priority number range: 0-10
 */
export type PriorityNumber = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10;

/**
 * Priority label mapping
 */
export const priorityLabels: Record<PriorityLabel, { min: number; max: number; color: string }> = {
  low: { min: 0, max: 3, color: "#10b981" }, // green
  medium: { min: 4, max: 6, color: "#f59e0b" }, // amber
  high: { min: 7, max: 9, color: "#ef4444" }, // red
  critical: { min: 10, max: 10, color: "#dc2626" }, // dark red
};

/**
 * Convert priority number to label
 * 
 * @param priority - Priority number (0-10)
 * @returns Priority label
 */
export function getPriorityLabel(priority: number | null | undefined): PriorityLabel {
  if (priority === null || priority === undefined) {
    return "low";
  }

  const clamped = Math.max(0, Math.min(10, priority));

  if (clamped <= 3) return "low";
  if (clamped <= 6) return "medium";
  if (clamped <= 9) return "high";
  return "critical";
}

/**
 * Convert priority label to number
 * 
 * @param label - Priority label
 * @returns Priority number (middle of range)
 */
export function getPriorityNumber(label: PriorityLabel): number {
  const range = priorityLabels[label];
  return Math.floor((range.min + range.max) / 2);
}

/**
 * Get priority color
 * 
 * @param priority - Priority number (0-10) or label
 * @returns Color hex code
 */
export function getPriorityColor(priority: number | PriorityLabel | null | undefined): string {
  if (priority === null || priority === undefined) {
    return priorityLabels.low.color;
  }

  if (typeof priority === "string") {
    return priorityLabels[priority]?.color || priorityLabels.low.color;
  }

  const label = getPriorityLabel(priority);
  return priorityLabels[label].color;
}

/**
 * Check if priority is valid
 * 
 * @param priority - Priority number
 * @returns True if valid (0-10)
 */
export function isValidPriority(priority: number | null | undefined): boolean {
  if (priority === null || priority === undefined) {
    return true; // null/undefined is valid (optional field)
  }
  return Number.isInteger(priority) && priority >= 0 && priority <= 10;
}

/**
 * Normalize priority value
 * 
 * Clamps priority to valid range (0-10)
 * 
 * @param priority - Priority number
 * @returns Normalized priority (0-10) or null
 */
export function normalizePriority(priority: number | null | undefined): number | null {
  if (priority === null || priority === undefined) {
    return null;
  }

  if (!isValidPriority(priority)) {
    return Math.max(0, Math.min(10, priority));
  }

  return priority;
}


