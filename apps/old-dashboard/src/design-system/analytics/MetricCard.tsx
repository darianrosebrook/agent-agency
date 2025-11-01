/**
 * MetricCard Component
 * Professional metric display with enhanced typography and visual hierarchy
 *
 * @author @darianrosebrook
 */

'use client';

import { ReactNode } from 'react';
import { Text } from '@/design-system/primitives';
import { Badge } from '@/design-system/primitives';
import { TrendingUp, TrendingDown, Minus } from 'lucide-react';
import styles from './MetricCard.module.scss';

export interface MetricCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  change?: {
    value: number;
    type: 'increase' | 'decrease' | 'neutral';
    period?: string;
  };
  status?: 'good' | 'warning' | 'critical' | 'neutral';
  icon?: ReactNode;
  trend?: 'up' | 'down' | 'stable';
  size?: 'small' | 'medium' | 'large';
  className?: string;
}

export function MetricCard({
  title,
  value,
  subtitle,
  change,
  status = 'neutral',
  icon,
  trend,
  size = 'medium',
  className = '',
}: MetricCardProps) {
  const formatValue = (val: string | number): string => {
    if (typeof val === 'number') {
      if (val >= 1000000) return `${(val / 1000000).toFixed(1)}M`;
      if (val >= 1000) return `${(val / 1000).toFixed(1)}k`;
      return val.toString();
    }
    return val;
  };

  const getTrendIcon = () => {
    if (trend === 'up') return <TrendingUp size={16} />;
    if (trend === 'down') return <TrendingDown size={16} />;
    return <Minus size={16} />;
  };

  const getStatusColor = () => {
    switch (status) {
      case 'good': return 'success';
      case 'warning': return 'warning';
      case 'critical': return 'error';
      default: return 'secondary';
    }
  };

  const getChangeColor = () => {
    if (!change) return 'secondary';
    switch (change.type) {
      case 'increase': return 'success';
      case 'decrease': return 'error';
      default: return 'secondary';
    }
  };

  return (
    <div className={`${styles.metricCard} ${styles[size]} ${className}`}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.titleSection}>
          {icon && <div className={styles.icon}>{icon}</div>}
          <Text variant="paragraph-medium" className={styles.title}>
            {title}
          </Text>
        </div>
        
        {status !== 'neutral' && (
          <Badge variant={getStatusColor()} size="sm">
            {status}
          </Badge>
        )}
      </div>

      {/* Main Value */}
      <div className={styles.valueSection}>
        <Text variant="display-1" className={styles.value}>
          {formatValue(value)}
        </Text>
        
        {subtitle && (
          <Text variant="paragraph-small" color="secondary" className={styles.subtitle}>
            {subtitle}
          </Text>
        )}
      </div>

      {/* Change Indicator */}
      {change && (
        <div className={styles.changeSection}>
          <div className={`${styles.changeIndicator} ${styles[change.type]}`}>
            {getTrendIcon()}
            <Text variant="paragraph-small" color={getChangeColor()}>
              {change.value > 0 ? '+' : ''}{change.value}%
            </Text>
          </div>
          
          {change.period && (
            <Text variant="paragraph-small" color="secondary">
              {change.period}
            </Text>
          )}
        </div>
      )}

      {/* Trend Visualization */}
      {trend && (
        <div className={styles.trendSection}>
          <div className={`${styles.trendIndicator} ${styles[trend]}`}>
            {getTrendIcon()}
          </div>
        </div>
      )}
    </div>
  );
}
