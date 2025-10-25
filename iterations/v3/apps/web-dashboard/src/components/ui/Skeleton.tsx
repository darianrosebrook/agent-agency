'use client';

import { cn } from '@/lib/utils';
import styles from './Skeleton.module.scss';

interface SkeletonProps {
  className?: string;
  width?: string | number;
  height?: string | number;
  variant?: 'text' | 'rectangular' | 'circular';
  animation?: 'pulse' | 'wave' | 'none';
}

export function Skeleton({
  className,
  width,
  height,
  variant = 'rectangular',
  animation = 'pulse',
}: SkeletonProps) {
  return (
    <div
      className={cn(
        styles.skeleton,
        styles[variant],
        styles[animation],
        className
      )}
      style={{
        width: typeof width === 'number' ? `${width}px` : width,
        height: typeof height === 'number' ? `${height}px` : height,
      }}
    />
  );
}

// Predefined skeleton components for common use cases
export function SkeletonText({ lines = 1, className }: { lines?: number; className?: string }) {
  return (
    <div className={cn(styles.textSkeleton, className)}>
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton
          key={i}
          variant="text"
          height={16}
          className={styles.textLine || ''}
        />
      ))}
    </div>
  );
}

export function SkeletonCard({ className }: { className?: string }) {
  return (
    <div className={cn(styles.cardSkeleton, className)}>
      <Skeleton variant="rectangular" height={200} />
      <div className={styles.cardContent}>
        <SkeletonText lines={2} />
        <Skeleton variant="rectangular" height={32} width="60%" />
      </div>
    </div>
  );
}

export function SkeletonTable({ rows = 5, columns = 4 }: { rows?: number; columns?: number }) {
  return (
    <div className={styles.tableSkeleton}>
      <div className={styles.tableHeader}>
        {Array.from({ length: columns }).map((_, i) => (
          <Skeleton key={i} variant="text" height={20} />
        ))}
      </div>
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className={styles.tableRow}>
          {Array.from({ length: columns }).map((_, j) => (
            <Skeleton key={j} variant="text" height={16} />
          ))}
        </div>
      ))}
    </div>
  );
}