'use client';

import { cn } from '@/lib/utils';
import styles from './Progress.module.scss';

interface ProgressProps {
  value: number;
  max?: number;
  size?: 'sm' | 'md' | 'lg';
  variant?: 'default' | 'success' | 'warning' | 'error';
  showValue?: boolean;
  className?: string;
}

export function Progress({
  value,
  max = 100,
  size = 'md',
  variant = 'default',
  showValue = false,
  className,
}: ProgressProps) {
  const percentage = Math.min(Math.max((value / max) * 100, 0), 100);

  return (
    <div className={cn(styles.progress, styles[size], className)}>
      <div className={styles.progressTrack}>
        <div
          className={cn(styles.progressBar, styles[variant])}
          style={{ width: `${percentage}%` }}
        />
      </div>
      {showValue && (
        <span className={styles.progressValue}>
          {Math.round(percentage)}%
        </span>
      )}
    </div>
  );
}

interface CircularProgressProps {
  value: number;
  max?: number;
  size?: number;
  strokeWidth?: number;
  variant?: 'default' | 'success' | 'warning' | 'error';
  showValue?: boolean;
  className?: string;
}

export function CircularProgress({
  value,
  max = 100,
  size = 40,
  strokeWidth = 4,
  variant = 'default',
  showValue = false,
  className,
}: CircularProgressProps) {
  const percentage = Math.min(Math.max((value / max) * 100, 0), 100);
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const strokeDasharray = circumference;
  const strokeDashoffset = circumference - (percentage / 100) * circumference;

  return (
    <div className={cn(styles.circularProgress, className)} style={{ width: size, height: size }}>
      <svg width={size} height={size} className={styles.circularSvg}>
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          stroke="#e0e0e0"
          strokeWidth={strokeWidth}
          fill="none"
        />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          stroke="currentColor"
          strokeWidth={strokeWidth}
          fill="none"
          strokeDasharray={strokeDasharray}
          strokeDashoffset={strokeDashoffset}
          className={cn(styles.circularBar, styles[variant])}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
        />
      </svg>
      {showValue && (
        <span className={styles.circularValue}>
          {Math.round(percentage)}%
        </span>
      )}
    </div>
  );
}
