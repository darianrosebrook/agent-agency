"use client";

import { StatusIcon, type StatusIconType } from "./StatusIcon";
import styles from "./StatusBadge.module.scss";
import { cn } from "../primitives/utils";

export interface StatusConfig {
  label: string;
  color: string;
  icon: StatusIconType;
}

export interface StatusBadgeProps {
  status: string;
  config: StatusConfig;
  onClick?: () => void;
  className?: string;
  as?: "button" | "div" | "span";
}

export function StatusBadge({
  status,
  config,
  onClick,
  className = "",
  as = "button",
}: StatusBadgeProps) {
  // Map status to SCSS class name
  const statusClassMap: Record<string, string> = {
    planning: styles.statusPlanning,
    "in-progress": styles.statusInProgress,
    "on-hold": styles.statusOnHold,
    completed: styles.statusCompleted,
    backlog: styles.statusBacklog,
    todo: styles.statusTodo,
    done: styles.statusDone,
  };

  const statusClass = statusClassMap[status] || "";
  const content = (
    <>
      <StatusIcon type={config.icon} />
      {config.label}
    </>
  );

  // If used inside another button or as non-interactive, render as div/span
  if (as === "div") {
    return (
      <div className={cn(styles.statusBadge, statusClass, className)}>
        {content}
      </div>
    );
  }

  if (as === "span") {
    return (
      <span className={cn(styles.statusBadge, statusClass, className)}>
        {content}
      </span>
    );
  }

  // Default: render as button
  return (
    <button
      onClick={onClick}
      className={cn(styles.statusBadge, statusClass, className)}
      type={onClick ? "button" : undefined}
    >
      {content}
    </button>
  );
}
