"use client";

import { StatusIcon, type StatusIconType } from "./StatusIcon";
import styles from "./StatusBadge.module.scss";
import { cn } from "../ui/utils";

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
}

export function StatusBadge({
  status: _status, // eslint-disable-line @typescript-eslint/no-unused-vars, no-unused-vars
  config,
  onClick,
  className = "",
}: StatusBadgeProps) {
  return (
    <button
      onClick={onClick}
      className={cn(styles.statusBadge, config.color, className)}
      type={onClick ? "button" : undefined}
    >
      <StatusIcon type={config.icon} />
      {config.label}
    </button>
  );
}
