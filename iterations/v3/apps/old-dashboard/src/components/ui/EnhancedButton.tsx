"use client";

import { forwardRef } from "react";
import styles from "./EnhancedButton.module.scss";

interface EnhancedButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "tertiary" | "ghost" | "danger";
  size?: "xs" | "sm" | "md" | "lg" | "xl";
  loading?: boolean;
  loadingText?: string;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  fullWidth?: boolean;
  children: React.ReactNode;
}

const EnhancedButton = forwardRef<HTMLButtonElement, EnhancedButtonProps>(
  (
    {
      variant = "primary",
      size = "md",
      loading = false,
      loadingText = "Loading...",
      leftIcon,
      rightIcon,
      fullWidth = false,
      children,
      className,
      disabled,
      ...props
    },
    ref
  ) => {
    const isDisabled = disabled || loading;

    return (
      <button
        ref={ref}
        className={`
          ${styles.button}
          ${styles[variant]}
          ${styles[size]}
          ${fullWidth ? styles.fullWidth : ""}
          ${loading ? styles.loading : ""}
          ${className || ""}
        `.trim()}
        disabled={isDisabled}
        aria-disabled={isDisabled}
        aria-busy={loading}
        {...props}
      >
        {loading && (
          <span className={styles.loadingSpinner} aria-hidden="true">
            <svg
              className={styles.spinner}
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <circle
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeDasharray="60 40"
                className={styles.spinnerPath}
              />
            </svg>
          </span>
        )}
        
        {!loading && leftIcon && (
          <span className={styles.leftIcon} aria-hidden="true">
            {leftIcon}
          </span>
        )}
        
        <span className={loading ? styles.srOnly : ""}>
          {loading ? loadingText : children}
        </span>
        
        {!loading && rightIcon && (
          <span className={styles.rightIcon} aria-hidden="true">
            {rightIcon}
          </span>
        )}
      </button>
    );
  }
);

EnhancedButton.displayName = "EnhancedButton";

export default EnhancedButton;




