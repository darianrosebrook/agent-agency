// Formatting utilities
import { format, formatDistanceToNow } from "date-fns";

export const formatDate = (date: string | Date): string => {
  try {
    const dateObj = typeof date === "string" ? new Date(date) : date;
    return format(dateObj, "MMM d, yyyy");
  } catch {
    return "Invalid date";
  }
};

export const formatDateTime = (date: string | Date): string => {
  try {
    const dateObj = typeof date === "string" ? new Date(date) : date;
    return format(dateObj, "MMM d, yyyy HH:mm:ss");
  } catch {
    return "Invalid date";
  }
};

export const formatRelativeTime = (date: string | Date): string => {
  try {
    const dateObj = typeof date === "string" ? new Date(date) : date;
    return formatDistanceToNow(dateObj, { addSuffix: true });
  } catch {
    return "Invalid date";
  }
};

export const formatDuration = (ms: number | undefined | null): string => {
  if (ms === undefined || ms === null || isNaN(Number(ms))) return "N/A";
  const numMs = Number(ms);
  if (numMs < 1000) return `${numMs}ms`;
  if (numMs < 60000) return `${(numMs / 1000).toFixed(1)}s`;
  if (numMs < 3600000) return `${(numMs / 60000).toFixed(1)}m`;
  return `${(numMs / 3600000).toFixed(1)}h`;
};

export const formatBytes = (bytes: number): string => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
};

export const formatNumber = (num: number): string => {
  return new Intl.NumberFormat().format(num);
};

export const formatPercentage = (
  value: number | string | undefined | null,
  decimals = 1
): string => {
  // Handle undefined/null
  if (value === undefined || value === null) return "N/A";

  // Handle string percentages (e.g., "0.00%")
  if (typeof value === "string") {
    // If already formatted, return as-is
    if (value.includes("%")) {
      return value;
    }
    // Otherwise parse as number
    const num = parseFloat(value);
    if (isNaN(num)) return "0%";
    return `${num.toFixed(decimals)}%`;
  }
  // Handle number values
  if (typeof value === "number" && !isNaN(value)) {
    return `${value.toFixed(decimals)}%`;
  }
  return "N/A";
};
