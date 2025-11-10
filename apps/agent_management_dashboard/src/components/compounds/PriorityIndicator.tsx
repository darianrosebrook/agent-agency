"use client";

import styles from "./PriorityIndicator.module.scss";
import { cn } from "../primitives/utils";

export interface PriorityConfig {
  label: string;
  color: string;
  icon: string;
}

export interface PriorityIndicatorProps {
  priority: string;
  config: PriorityConfig;
  onClick?: () => void;
  className?: string;
  as?: "button" | "div" | "span";
}

export function PriorityIndicator({
  priority,
  config,
  onClick,
  className = "",
  as,
}: PriorityIndicatorProps) {
  // Map priority to SCSS class name
  const priorityClassMap: Record<string, string> = {
    low: styles.priorityLow,
    medium: styles.priorityMedium,
    high: styles.priorityHigh,
  };

  const priorityClass = priorityClassMap[priority] || "";

  const content = (
    <>
      <span className={priorityClass}>{config.icon}</span>
      <span>{config.label}</span>
    </>
  );

  // If explicitly set to render as span/div (e.g., inside another button)
  if (as === "span") {
    return (
      <span className={cn(styles.priorityIndicator, className)}>{content}</span>
    );
  }

  if (as === "div") {
    return (
      <div className={cn(styles.priorityIndicator, className)}>{content}</div>
    );
  }

  // If onClick is provided and as is not explicitly set, render as button
  if (onClick && !as) {
    return (
      <button
        onClick={onClick}
        className={cn(styles.priorityIndicatorButton, className)}
        type="button"
      >
        {content}
      </button>
    );
  }

  // Default: render as div
  return (
    <div className={cn(styles.priorityIndicator, className)}>{content}</div>
  );
}
