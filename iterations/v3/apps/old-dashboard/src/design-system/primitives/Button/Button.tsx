/**
 * Button - Primitive button component for Agent Agency Dashboard
 * 
 * @author @darianrosebrook
 * 
 * FlowPress-styled button with hover states and loading support.
 * Follows design system tokens for consistency.
 */

import React, { ReactNode } from "react";

export interface ButtonProps {
  /** Visual weight of the button */
  variant?: "primary" | "secondary" | "tertiary" | "ghost" | "danger";
  /** Size of the button */
  size?: "xs" | "sm" | "md" | "lg" | "xl";
  /** Disabled state */
  disabled?: boolean;
  /** Optional loading spinner */
  isLoading?: boolean;
  /** Full width button */
  fullWidth?: boolean;
  /** Icon on the left */
  leftIcon?: ReactNode;
  /** Icon on the right */
  rightIcon?: ReactNode;
  /** Button content */
  children: React.ReactNode;
  /** Click handler */
  onClick?: (e: React.MouseEvent<HTMLButtonElement>) => void;
  /** Button type for forms */
  type?: "button" | "submit" | "reset";
  /** Additional CSS class */
  className?: string;
  /** ARIA label for accessibility */
  "aria-label"?: string;
}

const sizeStyles = {
  xs: {
    padding: "0.25rem 0.5rem",
    fontSize: "0.75rem",
    minHeight: "32px",
    minWidth: "32px",
  },
  sm: {
    padding: "0.5rem 0.75rem",
    fontSize: "0.875rem",
    minHeight: "36px",
    minWidth: "36px",
  },
  md: {
    padding: "13px 16px",
    fontSize: "1rem",
    minHeight: "48px",
    minWidth: "48px",
  },
  lg: {
    padding: "1rem 1.5rem",
    fontSize: "1.125rem",
    minHeight: "52px",
    minWidth: "52px",
  },
  xl: {
    padding: "1.25rem 2rem",
    fontSize: "1.25rem",
    minHeight: "60px",
    minWidth: "60px",
  },
};

const variantStyles = {
  primary: {
    background: "var(--color-brand-primary)",
    color: "var(--color-text-inverse)",
    border: "0.5px solid var(--color-brand-primary)",
  },
  secondary: {
    background: "var(--color-background-secondary)",
    color: "var(--color-text-primary)",
    border: "0.5px solid var(--color-border-default)",
  },
  tertiary: {
    background: "transparent",
    color: "var(--color-text-primary)",
    border: "0.5px solid transparent",
  },
  ghost: {
    background: "transparent",
    color: "var(--color-text-secondary)",
    border: "0.5px solid transparent",
  },
  danger: {
    background: "var(--color-error)",
    color: "var(--color-text-inverse)",
    border: "0.5px solid var(--color-error)",
  },
};

export function Button({
  variant = "primary",
  size = "md",
  disabled = false,
  isLoading = false,
  fullWidth = false,
  leftIcon,
  rightIcon,
  children,
  onClick,
  type = "button",
  className,
  "aria-label": ariaLabel,
}: ButtonProps) {
  const isDisabled = disabled || isLoading;
  const sizeStyle = sizeStyles[size];
  const variantStyle = variantStyles[variant];

  return (
    <button
      type={type}
      disabled={isDisabled}
      onClick={onClick}
      aria-label={ariaLabel}
      className={className}
      style={{
        ...sizeStyle,
        ...variantStyle,
        borderRadius: "8px",
        fontWeight: "var(--font-weight-medium)",
        fontFamily: "var(--font-family-display)",
        cursor: isDisabled ? "not-allowed" : "pointer",
        opacity: isDisabled ? 0.5 : 1,
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        gap: "var(--spacing-2)",
        transition: "all var(--transition-duration-base) var(--transition-easing-out)",
        position: "relative",
        overflow: "hidden",
        width: fullWidth ? "100%" : "auto",
        willChange: "transform",
      }}
      onMouseEnter={(e) => {
        if (!isDisabled) {
          e.currentTarget.style.transform = "translateY(-2px) translateZ(0)";
          e.currentTarget.style.boxShadow = "var(--box-shadow-md)";
        }
      }}
      onMouseLeave={(e) => {
        if (!isDisabled) {
          e.currentTarget.style.transform = "translateY(0)";
          e.currentTarget.style.boxShadow = "none";
        }
      }}
    >
      {isLoading && (
        <span
          style={{
            width: "14px",
            height: "14px",
            border: "2px solid currentColor",
            borderTop: "2px solid transparent",
            borderRadius: "50%",
            animation: "spin 1s linear infinite",
            willChange: "transform",
            transform: "translateZ(0)",
          }}
          aria-hidden="true"
        />
      )}
      {!isLoading && leftIcon && <span style={{ display: "flex" }}>{leftIcon}</span>}
      <span>{children}</span>
      {!isLoading && rightIcon && <span style={{ display: "flex" }}>{rightIcon}</span>}
    </button>
  );
}

export default Button;


