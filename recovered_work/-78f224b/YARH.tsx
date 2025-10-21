"use client";

import React from "react";
import { MetricTileProps } from "@/types/metrics";
import styles from "./MetricTile.module.scss";

export default function MetricTile({
  title,
  value,
  change,
  changeLabel,
  status = "neutral",
  icon,
  trend,
  loading = false,
  format = "number",
}: MetricTileProps) {
  const formatValue = (val: string | number): string => {
    if (typeof val === "string") return val;

    switch (format) {
      case "percentage":
        return `${val.toFixed(1)}%`;
      case "currency":
        return `$${val.toLocaleString()}`;
      case "duration":
        if (val < 1000) return `${val}ms`;
        if (val < 60000) return `${(val / 1000).toFixed(1)}s`;
        if (val < 3600000) return `${(val / 60000).toFixed(1)}m`;
        return `${(val / 3600000).toFixed(1)}h`;
      case "bytes":
        if (val < 1024) return `${val}B`;
        if (val < 1024 * 1024) return `${(val / 1024).toFixed(1)}KB`;
        if (val < 1024 * 1024 * 1024)
          return `${(val / (1024 * 1024)).toFixed(1)}MB`;
        return `${(val / (1024 * 1024 * 1024)).toFixed(1)}GB`;
      default:
        return val.toLocaleString();
    }
  };

  const getTrendIcon = () => {
    if (!trend) return null;

    switch (trend) {
      case "up":
        return "📈";
      case "down":
        return "📉";
      case "stable":
        return "➡️";
      default:
        return null;
    }
  };

  const getStatusColor = () => {
    switch (status) {
      case "success":
        return styles.success;
      case "warning":
        return styles.warning;
      case "error":
        return styles.error;
      default:
        return styles.neutral;
    }
  };

  if (loading) {
    return (
      <div className={`${styles.metricTile} ${styles.loading}`}>
        <div className={styles.loadingSkeleton}>
          <div className={styles.skeletonHeader}></div>
          <div className={styles.skeletonValue}></div>
          <div className={styles.skeletonChange}></div>
        </div>
      </div>
    );
  }

  return (
    <div className={`${styles.metricTile} ${getStatusColor()}`}>
      <div className={styles.tileHeader}>
        <h3 className={styles.title}>{title}</h3>
        {icon && <span className={styles.icon}>{icon}</span>}
      </div>

      <div className={styles.tileBody}>
        <div className={styles.value}>{formatValue(value)}</div>

        {(change !== undefined || changeLabel) && (
          <div className={styles.change}>
            {change !== undefined && (
              <span
                className={`${styles.changeValue} ${
                  change > 0
                    ? styles.positive
                    : change < 0
                    ? styles.negative
                    : styles.neutral
                }`}
              >
                {change > 0 ? "+" : ""}
                {change.toFixed(1)}%
              </span>
            )}
            {changeLabel && (
              <span className={styles.changeLabel}>{changeLabel}</span>
            )}
            <span className={styles.trendIcon}>{getTrendIcon()}</span>
          </div>
        )}
      </div>
    </div>
  );
}
