/**
 * AnalyticsGrid Component
 * Grid layout for analytics cards with responsive design
 *
 * @author @darianrosebrook
 */

'use client';

import { ReactNode } from 'react';
import { Text } from '@/design-system/primitives';
import styles from './AnalyticsGrid.module.scss';

export interface AnalyticsGridProps {
  title?: string;
  subtitle?: string;
  children: ReactNode;
  columns?: 1 | 2 | 3 | 4 | 5 | 6;
  gap?: 'sm' | 'md' | 'lg';
  className?: string;
}

export function AnalyticsGrid({
  title,
  subtitle,
  children,
  columns = 3,
  gap = 'md',
  className = '',
}: AnalyticsGridProps) {
  return (
    <div className={`${styles.analyticsGrid} ${styles[`columns-${columns}`]} ${styles[`gap-${gap}`]} ${className}`}>
      {(title || subtitle) && (
        <div className={styles.header}>
          {title && (
            <Text variant="h3" className={styles.title}>
              {title}
            </Text>
          )}
          {subtitle && (
            <Text variant="paragraph-medium" color="secondary" className={styles.subtitle}>
              {subtitle}
            </Text>
          )}
        </div>
      )}
      
      <div className={styles.grid}>
        {children}
      </div>
    </div>
  );
}
