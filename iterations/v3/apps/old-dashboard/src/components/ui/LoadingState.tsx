'use client';

import { cn } from '@/lib/utils';
import { SkeletonCard, SkeletonTable } from './Skeleton';
import { CircularProgress } from './Progress';
import styles from './LoadingState.module.scss';

interface LoadingStateProps {
  type?: 'skeleton' | 'spinner' | 'progress';
  message?: string;
  progress?: number;
  className?: string;
}

export function LoadingState({
  type = 'skeleton',
  message = 'Loading...',
  progress,
  className,
}: LoadingStateProps) {
  return (
    <div className={cn(styles.loadingState, className)}>
      {type === 'skeleton' && (
        <div className={styles.skeletonContainer}>
          <SkeletonCard />
        </div>
      )}
      
      {type === 'spinner' && (
        <div className={styles.spinnerContainer}>
          <div className={styles.spinner} />
          <p className={styles.message}>{message}</p>
        </div>
      )}
      
      {type === 'progress' && (
        <div className={styles.progressContainer}>
          <CircularProgress
            value={progress || 0}
            size={60}
            strokeWidth={6}
            showValue
          />
          <p className={styles.message}>{message}</p>
        </div>
      )}
    </div>
  );
}

interface PageLoadingProps {
  message?: string;
  progress?: number;
  className?: string;
}

export function PageLoading({
  message = 'Loading page...',
  progress,
  className,
}: PageLoadingProps) {
  return (
    <div className={cn(styles.pageLoading, className)}>
      <div className={styles.content}>
        <CircularProgress
          value={progress || 0}
          size={80}
          strokeWidth={8}
          showValue
        />
        <p className={styles.message}>{message}</p>
      </div>
    </div>
  );
}

interface TableLoadingProps {
  rows?: number;
  columns?: number;
  className?: string;
}

export function TableLoading({
  rows = 5,
  columns = 4,
  className,
}: TableLoadingProps) {
  return (
    <div className={cn(styles.tableLoading, className)}>
      <SkeletonTable rows={rows} columns={columns} />
    </div>
  );
}
