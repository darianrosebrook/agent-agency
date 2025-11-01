// src/design-system/primitives/Icon/Icon.tsx
import React from "react";

export interface IconProps {
  /** Icon name or SVG element */
  children: React.ReactNode;
  /** Visual size */
  size?: "xs" | "sm" | "md" | "lg" | "xl";
  /** Color variant */
  color?: "primary" | "secondary" | "muted" | "inverse" | "success" | "warning" | "error";
  /** Additional CSS class */
  className?: string;
  /** Accessibility label */
  "aria-label"?: string;
  /** Hidden from screen readers */
  "aria-hidden"?: boolean;
}

/**
 * Icon - Primitive icon wrapper component for FlowPress
 *
 * Provides consistent sizing and coloring for icon elements.
 * Works with any icon library or SVG elements.
 *
 * @param {IconProps} props - Component properties
 * @returns {JSX.Element} Rendered icon wrapper
 */
export function Icon({
  children,
  size = "md",
  color = "primary",
  className,
  "aria-label": ariaLabel,
  "aria-hidden": ariaHidden = false,
}: IconProps) {
  const sizeMap = {
    xs: "12px",
    sm: "16px",
    md: "20px",
    lg: "24px",
    xl: "32px",
  };

  const colorMap = {
    primary: "var(--color-brand-primary)",
    secondary: "var(--color-text-secondary)",
    muted: "var(--color-text-muted)",
    inverse: "var(--color-text-inverse)",
    success: "var(--color-success)",
    warning: "var(--color-warning)",
    error: "var(--color-error)",
  };

  return (
    <span
      className={["icon", `icon-${size}`, `icon-${color}`, className]
        .filter(Boolean)
        .join(" ")}
      aria-label={ariaLabel}
      aria-hidden={ariaHidden}
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        width: sizeMap[size],
        height: sizeMap[size],
        color: colorMap[color],
        flexShrink: 0,
      }}
    >
      {children}
    </span>
  );
}

export default Icon;


