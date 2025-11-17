/**
 * Task Status Utilities
 * 
 * Provides canonical task status definitions, mappings, and utilities
 * aligned with backend Rust models.
 * 
 * Backend status enum: ['pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed']
 * 
 * @author @darianrosebrook
 */

/**
 * Backend task status values (canonical)
 * 
 * These match the database CHECK constraint in migrations/014_create_agent_management_tables.sql
 */
export type BackendTaskStatus =
  | "pending"
  | "in_progress"
  | "paused"
  | "completed"
  | "cancelled"
  | "failed";

/**
 * All valid backend task status values
 */
export const BACKEND_TASK_STATUSES: readonly BackendTaskStatus[] = [
  "pending",
  "in_progress",
  "paused",
  "completed",
  "cancelled",
  "failed",
] as const;

/**
 * Human-readable labels for task statuses
 */
export const statusLabels: Record<BackendTaskStatus, string> = {
  pending: "To Do",
  in_progress: "In Progress",
  paused: "Paused",
  completed: "Done",
  cancelled: "Cancelled",
  failed: "Failed",
} as const;

/**
 * Get human-readable label for a status
 */
export function getStatusLabel(status: BackendTaskStatus): string {
  return statusLabels[status];
}

/**
 * Status color mapping for UI components
 */
export const statusColors: Record<
  BackendTaskStatus,
  { bg: string; text: string; border?: string }
> = {
  pending: { bg: "#2d3748", text: "#cbd5e0", border: "#4a5568" },
  in_progress: { bg: "#2c5282", text: "#90cdf4", border: "#3182ce" },
  paused: { bg: "#744210", text: "#fbd38d", border: "#d69e2e" },
  completed: { bg: "#22543d", text: "#9ae6b4", border: "#38a169" },
  cancelled: { bg: "#742a2a", text: "#fc8181", border: "#e53e3e" },
  failed: { bg: "#742a2a", text: "#fc8181", border: "#e53e3e" },
} as const;

/**
 * Get status color configuration
 */
export function getStatusColor(
  status: BackendTaskStatus
): { bg: string; text: string; border?: string } {
  return statusColors[status];
}

/**
 * Status transition validation
 * 
 * Defines which status transitions are valid
 */
export const validStatusTransitions: Record<
  BackendTaskStatus,
  readonly BackendTaskStatus[]
> = {
  pending: ["in_progress", "cancelled"] as const,
  in_progress: ["paused", "completed", "failed", "cancelled"] as const,
  paused: ["in_progress", "cancelled"] as const,
  completed: [] as const, // Terminal state
  cancelled: [] as const, // Terminal state
  failed: ["pending", "in_progress"] as const, // Can retry
} as const;

/**
 * Check if a status transition is valid
 */
export function isValidStatusTransition(
  from: BackendTaskStatus,
  to: BackendTaskStatus
): boolean {
  if (from === to) return true; // No-op transition
  return validStatusTransitions[from].includes(to);
}

/**
 * Legacy frontend status values (deprecated)
 * 
 * These are the old values used in the frontend before alignment.
 * Kept for migration purposes.
 */
export type LegacyTaskStatus = "backlog" | "todo" | "in-progress" | "done";

/**
 * Map legacy frontend status to backend status
 * 
 * @deprecated Use backend status values directly
 */
export function mapLegacyStatusToBackend(
  legacyStatus: LegacyTaskStatus | string
): BackendTaskStatus {
  const mapping: Record<string, BackendTaskStatus> = {
    backlog: "pending",
    todo: "pending",
    "in-progress": "in_progress",
    done: "completed",
  };

  // If already a backend status, return as-is
  if (BACKEND_TASK_STATUSES.includes(legacyStatus as BackendTaskStatus)) {
    return legacyStatus as BackendTaskStatus;
  }

  return mapping[legacyStatus] || "pending";
}

/**
 * Map backend status to legacy frontend status (for backward compatibility)
 * 
 * @deprecated Use backend status values directly
 */
export function mapBackendStatusToLegacy(
  backendStatus: BackendTaskStatus
): LegacyTaskStatus {
  const mapping: Record<BackendTaskStatus, LegacyTaskStatus> = {
    pending: "todo",
    in_progress: "in-progress",
    paused: "in-progress", // Map paused to in-progress for legacy UI
    completed: "done",
    cancelled: "done", // Map cancelled to done for legacy UI
    failed: "done", // Map failed to done for legacy UI
  };

  return mapping[backendStatus];
}

/**
 * Check if a status is a terminal state (cannot transition from)
 */
export function isTerminalStatus(status: BackendTaskStatus): boolean {
  return status === "completed" || status === "cancelled";
}

/**
 * Check if a status indicates active work
 */
export function isActiveStatus(status: BackendTaskStatus): boolean {
  return status === "in_progress" || status === "paused";
}

/**
 * Check if a status indicates completion
 */
export function isCompletedStatus(status: BackendTaskStatus): boolean {
  return status === "completed";
}

/**
 * Check if a status indicates failure
 */
export function isFailedStatus(status: BackendTaskStatus): boolean {
  return status === "failed" || status === "cancelled";
}


