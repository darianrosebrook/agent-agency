/**
 * Progress Component
 * Visual progress indicator with customizable styling
 *
 * @author @darianrosebrook
 */

'use client';

import { forwardRef } from 'react';
import styles from './Progress.module.scss';

export interface ProgressProps {
  value: number;
  max?: number;
  size?: 'sm' | 'md' | 'lg';
  variant?: 'default' | 'success' | 'warning' | 'error';
  showValue?: boolean;
  label?: string;
  className?: string;
}

export const Progress = forwardRef<HTMLDivElement, ProgressProps>(
  (
    {
      value,
      max = 100,
      size = 'md',
      variant = 'default',
      showValue = false,
      label,
      className = '',
    },
    ref
  ) => {
    const percentage = Math.min(Math.max((value / max) * 100, 0), 100);

    return (
      <div
        ref={ref}
        className={`${styles.progress} ${styles[size]} ${styles[variant]} ${className}`}
      >
        {label && <div className={styles.label}>{label}</div>}
        <div className={styles.track}>
          <div
            className={styles.fill}
            style={{ width: `${percentage}%` }}
          />
        </div>
        {showValue && (
          <div className={styles.value}>
            {Math.round(percentage)}%
          </div>
        )}
      </div>
    );
  }
);

Progress.displayName = 'Progress';
