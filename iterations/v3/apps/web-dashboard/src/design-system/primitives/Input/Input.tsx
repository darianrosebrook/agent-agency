/**
 * Input - Primitive input component for Agent Agency Dashboard
 * 
 * @author @darianrosebrook
 * 
 * FlowPress-styled input field with focus states and validation.
 * Maintains consistent 48px minimum height for accessibility.
 */

import React from "react";

export interface InputProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "size"> {
  /** Visual size */
  visualSize?: "sm" | "md" | "lg";
  /** Error state */
  error?: boolean;
  /** Success state */
  success?: boolean;
  /** Additional CSS class */
  className?: string;
}

const sizeStyles = {
  sm: {
    padding: "0.5rem 0.75rem",
    fontSize: "0.875rem",
    minHeight: "40px",
  },
  md: {
    padding: "13px 16px",
    fontSize: "1rem",
    minHeight: "48px",
  },
  lg: {
    padding: "1rem 1.25rem",
    fontSize: "1.125rem",
    minHeight: "56px",
  },
};

export function Input({
  visualSize = "md",
  error = false,
  success = false,
  className,
  ...inputProps
}: InputProps) {
  const sizeStyle = sizeStyles[visualSize];

  const getBorderColor = () => {
    if (error) return "var(--color-error)";
    if (success) return "var(--color-success)";
    return "var(--color-border-default)";
  };

  return (
    <input
      {...inputProps}
      className={className}
      style={{
        ...sizeStyle,
        width: "100%",
        fontFamily: "var(--font-family-display)",
        backgroundColor: "var(--color-background-primary)",
        border: `0.5px solid ${getBorderColor()}`,
        borderRadius: "8px",
        color: "var(--color-text-primary)",
        outline: "none",
        transition: "border-color var(--transition-duration-fast) var(--transition-easing-out), background-color var(--transition-duration-fast) var(--transition-easing-out)",
        boxSizing: "border-box",
        ...(inputProps.disabled && {
          backgroundColor: "var(--color-background-secondary)",
          cursor: "not-allowed",
          opacity: 0.6,
        }),
      }}
      onFocus={(e) => {
        if (!error && !success) {
          e.currentTarget.style.borderColor = "var(--color-border-focus)";
        }
        e.currentTarget.style.backgroundColor = "var(--color-background-secondary)";
        inputProps.onFocus?.(e);
      }}
      onBlur={(e) => {
        e.currentTarget.style.borderColor = getBorderColor();
        e.currentTarget.style.backgroundColor = "var(--color-background-primary)";
        inputProps.onBlur?.(e);
      }}
    />
  );
}

export default Input;

