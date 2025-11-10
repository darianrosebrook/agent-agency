import type { StatusConfig } from "./StatusBadge";

// Project status configuration
export type ProjectStatus =
  | "planning"
  | "in-progress"
  | "on-hold"
  | "completed";

export const projectStatusConfig: Record<ProjectStatus, StatusConfig> = {
  planning: {
    label: "Planning",
    color: "", // Now handled by SCSS module classes
    icon: "dashed-circle",
  },
  "in-progress": {
    label: "In-progress",
    color: "", // Now handled by SCSS module classes
    icon: "half-circle",
  },
  "on-hold": {
    label: "On-hold",
    color: "", // Now handled by SCSS module classes
    icon: "circle-arrow",
  },
  completed: {
    label: "Completed",
    color: "", // Now handled by SCSS module classes
    icon: "check",
  },
};

// Task status configuration
export type TaskStatus = "backlog" | "todo" | "in-progress" | "done";

export const taskStatusConfig: Record<TaskStatus, StatusConfig> = {
  backlog: {
    label: "Backlog",
    color: "", // Now handled by SCSS module classes
    icon: "dashed-circle",
  },
  todo: {
    label: "Todo",
    color: "", // Now handled by SCSS module classes
    icon: "circle",
  },
  "in-progress": {
    label: "In-progress",
    color: "", // Now handled by SCSS module classes
    icon: "half-circle",
  },
  done: {
    label: "Done",
    color: "", // Now handled by SCSS module classes
    icon: "check",
  },
};
