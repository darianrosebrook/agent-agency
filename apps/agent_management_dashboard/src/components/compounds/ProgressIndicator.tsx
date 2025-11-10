/**
 * Progress Indicator Component
 *
 * Displays progress for long-running operations with percentage and optional message.
 *
 * @author @darianrosebrook
 */

"use client";

import React from "react";
import { cn } from "../primitives/utils";
import { Loader2 } from "lucide-react";
import styles from "./ProgressIndicator.module.scss";

interface ProgressIndicatorProps {
  progress?: number; // 0-100
  message?: string;
  showPercentage?: boolean;
  className?: string;
}

export function ProgressIndicator({
  progress,
  message,
  showPercentage = true,
  className,
}: ProgressIndicatorProps) {
  const displayProgress = Math.min(100, Math.max(0, progress ?? 0));

  return (
    <div className={cn(styles.progressIndicator, className)}>
      {progress !== undefined ? (
        <>
          {/* Progress Bar */}
          <div className={styles.progressBarContainer}>
            <div
              className={styles.progressBarFill}
              style={{ width: `${displayProgress}%` }}
            />
          </div>

          {/* Percentage and Message */}
          <div className={styles.progressInfo}>
            {showPercentage && (
              <div className={styles.progressPercentage}>
                {Math.round(displayProgress)}%
              </div>
            )}
            {message && <div className={styles.progressMessage}>{message}</div>}
          </div>
        </>
      ) : (
        <>
          {/* Indeterminate Progress */}
          <Loader2 className={styles.loaderIcon} />
          {message && <div className={styles.loaderMessage}>{message}</div>}
        </>
      )}
    </div>
  );
}
