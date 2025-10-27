// src/design-system/compounds/MetricCard/MetricCard.tsx
import React from "react";
import { Text } from "../../primitives/Text";
import styles from "./MetricCard.module.scss";

export interface MetricCardProps {
  /** Metric title */
  title: string;
  /** Metric value */
  value: string | number;
  /** Optional description or unit */
  description?: string;
  /** Icon for the metric */
  icon?: React.ReactNode;
  /** Optional trend indicator */
  trend?: "positive" | "negative" | "neutral";
  /** Trend value (e.g., "+12%") */
  trendValue?: string;
  /** Additional CSS class */
  className?: string;
}

/**
 * MetricCard - Compound component for displaying individual metrics
 *
 * Uses container queries to adapt layout based on available space.
 * Displays a metric with a title, value, optional description, icon, and trend.
 *
 * @param {MetricCardProps} props - Component properties
 * @returns {JSX.Element} Rendered metric card
 */
export function MetricCard({
  title,
  value,
  description,
  icon,
  trend,
  trendValue,
  className,
}: MetricCardProps) {
  return (
    <div className={[styles.metricCard, "metric-container", className].filter(Boolean).join(" ")}>
      <div className={styles.metricHeader}>
        <Text variant="paragraph-small" color="secondary" weight="medium" className={styles.metricLabel}>
          {title}
        </Text>
        {icon && <span className={styles.metricIcon}>{icon}</span>}
      </div>
      
      <div className={styles.metricContent}>
        <Text variant="h4" color="primary" className={styles.metricValue}>
          {value}
        </Text>
        {description && (
          <Text variant="paragraph-small" color="muted" className={styles.metricDescription}>
            {description}
          </Text>
        )}
        {trend && trendValue && (
          <div className={`${styles.metricTrend} ${styles[trend]}`}>
            <span>{trend === "positive" ? "↗" : trend === "negative" ? "↘" : "→"}</span>
            <Text variant="paragraph-small" weight="medium">
              {trendValue}
            </Text>
          </div>
        )}
      </div>
    </div>
  );
}

export default MetricCard;
