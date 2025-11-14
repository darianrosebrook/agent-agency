/**
 * Chart Error Boundary
 * 
 * Specialized error boundary for chart components.
 * Provides graceful degradation for visualization components.
 * 
 * @author @darianrosebrook
 */

"use client";

import React, { type ReactNode } from "react";
import { ErrorIsolation } from "./ErrorIsolation";
import { BarChart3 } from "lucide-react";
import styles from "./ChartErrorBoundary.module.scss";

interface ChartErrorBoundaryProps {
  children: ReactNode;
  chartName: string;
  fallback?: ReactNode;
}

/**
 * Default fallback for chart errors
 */
function ChartFallback({ chartName }: { chartName: string }) {
  return (
    <div className={styles.chartFallback}>
      <div className={styles.chartFallbackIcon}>
        <BarChart3 className={styles.chartFallbackIconSvg} />
      </div>
      <div className={styles.chartFallbackText}>
        <p className={styles.chartFallbackMessage}>
          Unable to load {chartName}
        </p>
        <p className={styles.chartFallbackHint}>
          Data may be temporarily unavailable
        </p>
      </div>
    </div>
  );
}

/**
 * ChartErrorBoundary - Wraps chart components with error isolation
 */
export function ChartErrorBoundary({
  children,
  chartName,
  fallback,
}: ChartErrorBoundaryProps) {
  const scope = `chart-${chartName.toLowerCase().replace(/\s+/g, "-")}`;
  const fallbackUI = fallback || <ChartFallback chartName={chartName} />;

  return (
    <ErrorIsolation scope={scope} fallback={fallbackUI} isolate={true}>
      {children}
    </ErrorIsolation>
  );
}

