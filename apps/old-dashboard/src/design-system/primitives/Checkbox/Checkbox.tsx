'use client';

// src/design-system/primitives/Checkbox/Checkbox.tsx
import React, { useState } from "react";

export interface CheckboxProps {
  /** Whether the checkbox is checked */
  checked?: boolean;
  /** Default checked state for uncontrolled usage */
  defaultChecked?: boolean;
  /** Whether the checkbox is in indeterminate state */
  indeterminate?: boolean;
  /** Whether the checkbox is disabled */
  disabled?: boolean;
  /** Size of the checkbox */
  size?: "sm" | "md" | "lg";
  /** Change handler */
  onChange?: (checked: boolean) => void;
  /** ARIA label for accessibility */
  "aria-label"?: string;
  /** ID for label association */
  id?: string;
  /** Additional CSS class */
  className?: string;
}

/**
 * Checkbox - Primitive checkbox component for FlowPress
 *
 * Provides accessible checkbox input with proper keyboard support.
 * Supports controlled and uncontrolled modes, indeterminate state.
 *
 * @param {CheckboxProps} props - Component properties
 * @returns {JSX.Element} Rendered checkbox
 */
export function Checkbox({
  checked,
  defaultChecked = false,
  indeterminate = false,
  disabled = false,
  size = "md",
  onChange,
  "aria-label": ariaLabel,
  id,
  className,
}: CheckboxProps) {
  const [internalChecked, setInternalChecked] = useState(defaultChecked);
  const isControlled = checked !== undefined;
  const isChecked = isControlled ? checked : internalChecked;

  const handleChange = () => {
    if (disabled) return;

    const newChecked = !isChecked;
    if (!isControlled) {
      setInternalChecked(newChecked);
    }
    onChange?.(newChecked);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      handleChange();
    }
  };

  const sizeStyles = {
    sm: { width: "16px", height: "16px" },
    md: { width: "20px", height: "20px" },
    lg: { width: "24px", height: "24px" },
  };

  return (
    <div
      role="checkbox"
      aria-checked={indeterminate ? "mixed" : isChecked}
      aria-label={ariaLabel}
      id={id}
      tabIndex={disabled ? -1 : 0}
      onClick={handleChange}
      onKeyDown={handleKeyDown}
      className={["checkbox", `checkbox-${size}`, className]
        .filter(Boolean)
        .join(" ")}
      style={{
        ...sizeStyles[size],
        backgroundColor: isChecked || indeterminate ? "var(--color-brand-primary)" : "var(--color-background-primary)",
        border: "0.5px solid var(--color-border-default)",
        borderRadius: "4px",
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.6 : 1,
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        outline: "none",
        transition: "all var(--transition-duration-fast) var(--transition-easing-out)",
      }}
      onFocus={(e) => {
        e.currentTarget.style.outline = "2px solid var(--color-border-focus)";
        e.currentTarget.style.outlineOffset = "2px";
      }}
      onBlur={(e) => {
        e.currentTarget.style.outline = "none";
      }}
    >
      {indeterminate ? (
        <div
          style={{
            width: "60%",
            height: "2px",
            backgroundColor: "var(--color-text-inverse)",
          }}
        />
      ) : isChecked ? (
        <svg
          width="12"
          height="12"
          viewBox="0 0 12 12"
          fill="none"
          style={{ color: "var(--color-text-inverse)" }}
        >
          <path
            d="M10 3L4.5 8.5L2 6"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      ) : null}
    </div>
  );
}

export default Checkbox;


