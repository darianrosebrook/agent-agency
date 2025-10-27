/**
 * Hardware Metric Card
 * Individual hardware component metric display
 *
 * @author @darianrosebrook
 */

'use client';

import { useState } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { ChevronDown, ChevronUp, AlertTriangle } from 'lucide-react';
import styles from './HardwareMetricCard.module.scss';

interface HardwareMetricCardProps {
  title: string;
  value: string;
  subtitle: string;
  icon: React.ReactNode;
  status: 'good' | 'warning' | 'error';
  details: Record<string, any>;
}

export function HardwareMetricCard({ title, value, subtitle, icon, status, details }: HardwareMetricCardProps) {
  const [expanded, setExpanded] = useState(false);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'good':
        return 'var(--color-success)';
      case 'warning':
        return 'var(--color-warning)';
      case 'error':
        return 'var(--color-error)';
      default:
        return 'var(--color-neutral)';
    }
  };

  const formatDetailValue = (key: string, value: any): string => {
    if (typeof value === 'boolean') {
      return value ? 'Yes' : 'No';
    }
    if (typeof value === 'number') {
      if (key.toLowerCase().includes('temperature') || key.toLowerCase().includes('temp')) {
        return `${value}°C`;
      }
      if (key.toLowerCase().includes('power') || key.toLowerCase().includes('consumption')) {
        return `${value}W`;
      }
      if (key.toLowerCase().includes('efficiency') || key.toLowerCase().includes('percentage')) {
        return `${value}%`;
      }
      if (key.toLowerCase().includes('memory') && !key.toLowerCase().includes('bandwidth')) {
        return typeof value === 'number' && value > 100 ? `${(value / 1024).toFixed(1)}GB` : `${value}MB`;
      }
      if (key.toLowerCase().includes('bandwidth')) {
        return `${(value / 1024).toFixed(1)}GB/s`;
      }
      if (key.toLowerCase().includes('frequency')) {
        return `${value}MHz`;
      }
      return value.toString();
    }
    return value?.toString() || 'N/A';
  };

  const formatDetailKey = (key: string): string => {
    return key.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase());
  };

  return (
    <div className={`${styles.metricCard} ${styles[status]}`}>
      {/* Header */}
      <div className={styles.cardHeader}>
        <div className={styles.headerLeft}>
          <div className={styles.iconWrapper} style={{ color: getStatusColor(status) }}>
            {icon}
          </div>
          <div className={styles.headerContent}>
            <Text variant="paragraph-medium" className={styles.cardTitle}>
              {title}
            </Text>
            <Text variant="paragraph-small" color="secondary" className={styles.cardValue}>
              {value}
            </Text>
          </div>
        </div>

        <Button
          variant="secondary"
          size="sm"
          onClick={() => setExpanded(!expanded)}
          className={styles.expandButton || ''}
        >
          {expanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
        </Button>
      </div>

      {/* Main Content */}
      <div className={styles.cardContent}>
        <Text variant="paragraph-small" color="secondary">
          {subtitle}
        </Text>

        {status !== 'good' && (
          <div className={styles.statusIndicator}>
            <AlertTriangle size={14} />
            <Text variant="paragraph-small" className={styles.statusText}>
              {status === 'warning' ? 'High utilization' : 'Critical condition'}
            </Text>
          </div>
        )}
      </div>

      {/* Expanded Details */}
      {expanded && (
        <div className={styles.detailsSection}>
          <div className={styles.detailsGrid}>
            {Object.entries(details).map(([key, value]) => (
              <div key={key} className={styles.detailItem}>
                <Text variant="paragraph-small" className={styles.detailKey}>
                  {formatDetailKey(key)}:
                </Text>
                <Text variant="paragraph-small" className={styles.detailValue}>
                  {formatDetailValue(key, value)}
                </Text>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
