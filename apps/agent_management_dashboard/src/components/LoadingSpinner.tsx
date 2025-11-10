/**
 * Loading Spinner Component
 * 
 * Reusable loading spinner with different sizes and variants.
 * 
 * @author @darianrosebrook
 */

"use client";

import { Loader2 } from "lucide-react";
import { cn } from "./primitives/utils";
import styles from "./LoadingSpinner.module.scss";

interface LoadingSpinnerProps {
  size?: "sm" | "md" | "lg";
  className?: string;
  text?: string;
}

const sizeClasses = {
  sm: styles.spinnerIconSmall,
  md: styles.spinnerIconMedium,
  lg: styles.spinnerIconLarge,
};

export function LoadingSpinner({
  size = "md",
  className,
  text,
}: LoadingSpinnerProps) {
  return (
    <div className={cn(styles.loadingSpinner, className)}>
      <Loader2 className={cn(styles.spinnerIcon, sizeClasses[size])} />
      {text && (
        <span className={styles.spinnerText}>{text}</span>
      )}
    </div>
  );
}

/**
 * Full page loading spinner
 */
export function PageLoading({ text = "Loading..." }: { text?: string }) {
  return (
    <div className={styles.pageLoading}>
      <div className={styles.pageLoadingContent}>
        <LoadingSpinner size="lg" />
        {text && (
          <p className={styles.pageLoadingText}>{text}</p>
        )}
      </div>
    </div>
  );
}

/**
 * Button loading spinner (for async actions)
 */
export function ButtonLoading() {
  return <LoadingSpinner size="sm" />;
}

