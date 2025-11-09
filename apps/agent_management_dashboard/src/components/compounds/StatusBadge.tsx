"use client";

import { StatusIcon, type StatusIconType } from "./StatusIcon";

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
      className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-full ${config.color} font-medium hover:opacity-80 transition-opacity ${className}`}
      type={onClick ? "button" : undefined}
    >
      <StatusIcon type={config.icon} />
      {config.label}
    </button>
  );
}
