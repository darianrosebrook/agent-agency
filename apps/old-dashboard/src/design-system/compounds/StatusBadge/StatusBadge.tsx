/**
 * StatusBadge - Compound badge for status indicators
 * 
 * @author @darianrosebrook
 * 
 * Specialized badge for task status, connection status, severity levels, etc.
 * Extends primitive Badge with dashboard-specific status styles.
 */

import React from "react";
import { Badge } from "../../primitives/Badge";
import { CheckCircle, XCircle, AlertCircle, Clock, Loader2 } from "lucide-react";

export type Status = 
  | "pending" 
  | "running" 
  | "completed" 
  | "failed" 
  | "online" 
  | "offline" 
  | "degraded" 
  | "checking"
  | "success"
  | "warning"
  | "error"
  | "info";

export interface StatusBadgeProps {
  /** Status type */
  status: Status;
  /** Size of the badge */
  size?: "sm" | "md" | "lg";
  /** Show icon */
  showIcon?: boolean;
  /** Custom label (overrides status name) */
  label?: string;
  /** Additional CSS class */
  className?: string;
}

const statusConfig: Record<Status, { 
  variant: "default" | "success" | "warning" | "error" | "info" | "neutral";
  icon: React.ReactNode;
  label: string;
}> = {
  pending: {
    variant: "warning",
    icon: <Clock size={12} />,
    label: "Pending",
  },
  running: {
    variant: "info",
    icon: <Loader2 size={12} className="spinning" />,
    label: "Running",
  },
  completed: {
    variant: "success",
    icon: <CheckCircle size={12} />,
    label: "Completed",
  },
  failed: {
    variant: "error",
    icon: <XCircle size={12} />,
    label: "Failed",
  },
  online: {
    variant: "success",
    icon: <CheckCircle size={12} />,
    label: "Online",
  },
  offline: {
    variant: "error",
    icon: <XCircle size={12} />,
    label: "Offline",
  },
  degraded: {
    variant: "warning",
    icon: <AlertCircle size={12} />,
    label: "Degraded",
  },
  checking: {
    variant: "info",
    icon: <Loader2 size={12} className="spinning" />,
    label: "Checking",
  },
  success: {
    variant: "success",
    icon: <CheckCircle size={12} />,
    label: "Success",
  },
  warning: {
    variant: "warning",
    icon: <AlertCircle size={12} />,
    label: "Warning",
  },
  error: {
    variant: "error",
    icon: <XCircle size={12} />,
    label: "Error",
  },
  info: {
    variant: "info",
    icon: <AlertCircle size={12} />,
    label: "Info",
  },
};

export function StatusBadge({
  status,
  size = "md",
  showIcon = true,
  label,
  className,
}: StatusBadgeProps) {
  const config = statusConfig[status];
  const displayLabel = label ?? config.label;

  return (
    <Badge
      variant={config.variant}
      size={size}
      icon={showIcon ? config.icon : undefined}
      className={className || ""}
    >
      {displayLabel}
    </Badge>
  );
}

export default StatusBadge;


