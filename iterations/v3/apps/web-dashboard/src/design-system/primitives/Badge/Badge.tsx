/**
 * Badge - Primitive badge component for Agent Agency Dashboard
 * 
 * @author @darianrosebrook
 * 
 * Status badges and labels following FlowPress design system.
 * Used for task status, severity levels, and categorical labels.
 */

import React from "react";

export interface BadgeProps {
  /** Visual variant */
  variant?: "default" | "success" | "warning" | "error" | "info" | "neutral";
  /** Size of the badge */
  size?: "sm" | "md" | "lg";
  /** Badge content */
  children: React.ReactNode;
  /** Additional CSS class */
  className?: string;
  /** Icon to display */
  icon?: React.ReactNode;
}

const variantStyles = {
  default: {
    background: "var(--color-background-secondary)",
    color: "var(--color-text-primary)",
    border: "0.5px solid var(--color-border-default)",
  },
  success: {
    background: "var(--color-success-light)",
    color: "var(--color-success-dark)",
    border: "0.5px solid var(--color-success)",
  },
  warning: {
    background: "var(--color-warning-light)",
    color: "var(--color-warning-dark)",
    border: "0.5px solid var(--color-warning)",
  },
  error: {
    background: "var(--color-error-light)",
    color: "var(--color-error-dark)",
    border: "0.5px solid var(--color-error)",
  },
  info: {
    background: "var(--color-info-light)",
    color: "var(--color-info-dark)",
    border: "0.5px solid var(--color-info)",
  },
  neutral: {
    background: "var(--color-background-muted)",
    color: "var(--color-text-secondary)",
    border: "0.5px solid var(--color-border-default)",
  },
};

const sizeStyles = {
  sm: {
    padding: "0.125rem 0.5rem",
    fontSize: "0.75rem",
    borderRadius: "6px",
  },
  md: {
    padding: "0.25rem 0.75rem",
    fontSize: "0.875rem",
    borderRadius: "8px",
  },
  lg: {
    padding: "0.5rem 1rem",
    fontSize: "1rem",
    borderRadius: "8px",
  },
};

export function Badge({
  variant = "default",
  size = "md",
  children,
  className,
  icon,
}: BadgeProps) {
  const variantStyle = variantStyles[variant];
  const sizeStyle = sizeStyles[size];

  return (
    <span
      className={className}
      style={{
        ...sizeStyle,
        ...variantStyle,
        display: "inline-flex",
        alignItems: "center",
        gap: "0.25rem",
        fontWeight: "var(--font-weight-medium)",
        fontFamily: "var(--font-family-display)",
        whiteSpace: "nowrap",
        textTransform: "uppercase",
        letterSpacing: "0.025em",
        lineHeight: "1",
        contain: "layout style paint",
      }}
    >
      {icon && <span style={{ display: "flex", alignItems: "center" }}>{icon}</span>}
      {children}
    </span>
  );
}

export default Badge;


