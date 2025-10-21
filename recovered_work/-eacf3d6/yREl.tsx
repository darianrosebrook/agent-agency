"use client";

import React, { useState, useEffect, useCallback } from "react";
import { CorrelationMatrixProps, Correlation } from "@/types/analytics";
import { analyticsApiClient, AnalyticsApiError } from "@/lib/analytics-api";
import styles from "./CorrelationMatrix.module.scss";

interface CorrelationMatrixState {
  correlations: Correlation[];
  isAnalyzing: boolean;
  error: string | null;
  minCorrelationStrength: Correlation["strength"];
}

export default function CorrelationMatrix({
  correlations: externalCorrelations,
  metrics,
  onCorrelationSelect,
  minCorrelationStrength = "moderate",
  isAnalyzing: externalAnalyzing,
  error: externalError,
}: CorrelationMatrixProps) {
  const [state, setState] = useState<CorrelationMatrixState>({
    correlations: externalCorrelations ?? [],
    isAnalyzing: externalAnalyzing ?? false,
    error: externalError ?? null,
    minCorrelationStrength,
  });

  // Load correlations if not provided externally
  const loadCorrelations = useCallback(async () => {
    if (externalCorrelations) return;

    try {
      setState((prev) => ({ ...prev, isAnalyzing: true, error: null }));

      // Get correlations for available metrics or all metrics
      const response = await analyticsApiClient.getCorrelations(
        undefined, // Use default filters
        0.1 // Significance threshold
      );

      setState((prev) => ({
        ...prev,
        correlations: response.correlations,
        isAnalyzing: false,
      }));
    } catch (error) {
      const errorMessage =
        error instanceof AnalyticsApiError
          ? error.message
          : "Failed to load correlation analysis";
      setState((prev) => ({
        ...prev,
        isAnalyzing: false,
        error: errorMessage,
      }));
      console.error("Failed to load correlations:", error);
    }
  }, [externalCorrelations]);

  // Handle correlation selection
  const handleCorrelationSelect = useCallback((correlation: Correlation) => {
    onCorrelationSelect?.(correlation);
  }, [onCorrelationSelect]);

  // Run correlation analysis
  const runCorrelationAnalysis = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, isAnalyzing: true, error: null }));

      // Re-fetch correlations with current settings
      const response = await analyticsApiClient.getCorrelations(
        undefined,
        0.1
      );

      setState((prev) => ({
        ...prev,
        correlations: response.correlations,
        isAnalyzing: false,
      }));
    } catch (error) {
      const errorMessage =
        error instanceof AnalyticsApiError
          ? error.message
          : "Failed to run correlation analysis";
      setState((prev) => ({
        ...prev,
        isAnalyzing: false,
        error: errorMessage,
      }));
      console.error("Failed to run correlation analysis:", error);
    }
  }, []);

  // Initialize data on mount
  useEffect(() => {
    if (!externalCorrelations) {
      loadCorrelations();
    }
  }, [externalCorrelations, loadCorrelations]);

  // Update state when external props change
  useEffect(() => {
    setState((prev) => ({
      ...prev,
      correlations: externalCorrelations ?? prev.correlations,
      isAnalyzing: externalAnalyzing ?? prev.isAnalyzing,
      error: externalError ?? prev.error,
      minCorrelationStrength,
    }));
  }, [externalCorrelations, externalAnalyzing, externalError, minCorrelationStrength]);

  // Filter correlations based on minimum strength
  const filteredCorrelations = state.correlations.filter(correlation => {
    const strengthLevels = {
      weak: 0,
      moderate: 1,
      strong: 2,
      very_strong: 3,
    };

    const currentLevel = strengthLevels[correlation.strength];
    const minLevel = strengthLevels[state.minCorrelationStrength];

    return currentLevel >= minLevel;
  });

  // Get unique metrics for matrix
  const allMetrics = Array.from(new Set([
    ...filteredCorrelations.flatMap(c => [c.metric_a, c.metric_b]),
    ...(metrics || []),
  ]));

  // Create correlation matrix
  const correlationMatrix = allMetrics.map(metricA =>
    allMetrics.map(metricB => {
      if (metricA === metricB) {
        return { correlation: 1, strength: "perfect" as const, direction: "positive" as const };
      }

      const correlation = filteredCorrelations.find(
        c => (c.metric_a === metricA && c.metric_b === metricB) ||
             (c.metric_a === metricB && c.metric_b === metricA)
      );

      return correlation ? {
        correlation: correlation.correlation_coefficient,
        strength: correlation.strength,
        direction: correlation.direction,
        fullCorrelation: correlation,
      } : null;
    })
  );

  const getCorrelationColor = (correlation: number | null) => {
    if (correlation === null) return styles.correlationEmpty;
    if (correlation > 0.8) return styles.correlationVeryStrong;
    if (correlation > 0.6) return styles.correlationStrong;
    if (correlation > 0.3) return styles.correlationModerate;
    if (correlation > 0.1) return styles.correlationWeak;
    if (correlation < -0.8) return styles.correlationVeryStrongNegative;
    if (correlation < -0.6) return styles.correlationStrongNegative;
    if (correlation < -0.3) return styles.correlationModerateNegative;
    if (correlation < -0.1) return styles.correlationWeakNegative;
    return styles.correlationNone;
  };

  const getCorrelationIcon = (direction: "positive" | "negative" | null) => {
    if (direction === "positive") return "↗️";
    if (direction === "negative") return "↘️";
    return "";
  };

  return (
    <div className={styles.correlationMatrix}>
      <div className={styles.matrixHeader}>
        <h2>Correlation Analysis</h2>
        <p className={styles.description}>
          Statistical analysis of relationships between metrics using correlation coefficients.
        </p>

        <div className={styles.matrixControls}>
          <div className={styles.filterControls}>
            <div className={styles.filterGroup}>
              <label>Min Strength:</label>
              <select
                value={state.minCorrelationStrength}
                onChange={(e) =>
                  setState((prev) => ({
                    ...prev,
                    minCorrelationStrength: e.target.value as Correlation["strength"],
                  }))
                }
              >
                <option value="weak">Weak</option>
                <option value="moderate">Moderate</option>
                <option value="strong">Strong</option>
                <option value="very_strong">Very Strong</option>
              </select>
            </div>
          </div>

          <button
            onClick={runCorrelationAnalysis}
            disabled={state.isAnalyzing}
            className={styles.analyzeButton}
          >
            {state.isAnalyzing ? (
              <>
                <div className={styles.spinner}></div>
                Analyzing...
              </>
            ) : (
              <>
                🔗 Analyze Correlations
              </>
            )}
          </button>
        </div>
      </div>

      {/* Error Display */}
      {state.error && (
        <div className={styles.errorBanner}>
          <span className={styles.errorIcon}>⚠️</span>
          <span>{state.error}</span>
        </div>
      )}

      {/* Correlation Matrix */}
      <div className={styles.matrixSection}>
        {state.isAnalyzing ? (
          <div className={styles.loading}>
            <div className={styles.spinner}></div>
            <p>Running correlation analysis...</p>
          </div>
        ) : filteredCorrelations.length > 0 ? (
          <>
            <div className={styles.matrixStats}>
              <span>
                Total Correlations: {filteredCorrelations.length}
              </span>
              <span>
                Metrics Analyzed: {allMetrics.length}
              </span>
              <span>
                Significance Threshold: 0.1
              </span>
            </div>

            <div className={styles.matrixContainer}>
              <div className={styles.matrixGrid}>
                {/* Header row with metric names */}
                <div className={styles.matrixRow}>
                  <div className={styles.matrixCell}></div>
                  {allMetrics.map((metric, index) => (
                    <div key={index} className={styles.matrixHeader}>
                      <span className={styles.metricLabel}>{metric}</span>
                    </div>
                  ))}
                </div>

                {/* Data rows */}
                {allMetrics.map((metricA, rowIndex) => (
                  <div key={rowIndex} className={styles.matrixRow}>
                    <div className={styles.matrixHeader}>
                      <span className={styles.metricLabel}>{metricA}</span>
                    </div>
                    {allMetrics.map((metricB, colIndex) => {
                      const cellData = correlationMatrix[rowIndex][colIndex];

                      if (rowIndex === colIndex) {
                        // Diagonal - perfect correlation
                        return (
                          <div key={colIndex} className={`${styles.matrixCell} ${styles.correlationPerfect}`}>
                            <span className={styles.correlationValue}>1.00</span>
                          </div>
                        );
                      }

                      if (!cellData) {
                        // No correlation data
                        return (
                          <div key={colIndex} className={`${styles.matrixCell} ${styles.correlationEmpty}`}>
                            <span className={styles.correlationValue}>-</span>
                          </div>
                        );
                      }

                      return (
                        <div
                          key={colIndex}
                          className={`${styles.matrixCell} ${getCorrelationColor(cellData.correlation)} ${
                            cellData.fullCorrelation ? styles.clickable : ""
                          }`}
                          onClick={() => cellData.fullCorrelation && handleCorrelationSelect(cellData.fullCorrelation)}
                          title={cellData.fullCorrelation ?
                            `${metricA} ↔ ${metricB}: ${cellData.correlation.toFixed(3)} (${cellData.strength})` :
                            undefined
                          }
                        >
                          <span className={styles.correlationValue}>
                            {cellData.correlation.toFixed(2)}
                          </span>
                          <span className={styles.correlationIcon}>
                            {getCorrelationIcon(cellData.direction)}
                          </span>
                        </div>
                      );
                    })}
                  </div>
                ))}
              </div>
            </div>

            {/* Correlation Legend */}
            <div className={styles.matrixLegend}>
              <h4>Correlation Strength</h4>
              <div className={styles.legendGrid}>
                <div className={styles.legendItem}>
                  <div className={`${styles.legendColor} ${styles.correlationVeryStrong}`}></div>
                  <span>Very Strong (±0.8+)</span>
                </div>
                <div className={styles.legendItem}>
                  <div className={`${styles.legendColor} ${styles.correlationStrong}`}></div>
                  <span>Strong (±0.6-0.8)</span>
                </div>
                <div className={styles.legendItem}>
                  <div className={`${styles.legendColor} ${styles.correlationModerate}`}></div>
                  <span>Moderate (±0.3-0.6)</span>
                </div>
                <div className={styles.legendItem}>
                  <div className={`${styles.legendColor} ${styles.correlationWeak}`}></div>
                  <span>Weak (±0.1-0.3)</span>
                </div>
                <div className={styles.legendItem}>
                  <div className={`${styles.legendColor} ${styles.correlationNone}`}></div>
                  <span>No Correlation</span>
                </div>
              </div>
            </div>

            {/* Top Correlations List */}
            <div className={styles.topCorrelations}>
              <h4>Top Correlations</h4>
              <div className={styles.correlationsList}>
                {filteredCorrelations
                  .sort((a, b) => Math.abs(b.correlation_coefficient) - Math.abs(a.correlation_coefficient))
                  .slice(0, 10)
                  .map((correlation, index) => (
                    <div
                      key={index}
                      className={styles.correlationItem}
                      onClick={() => handleCorrelationSelect(correlation)}
                    >
                      <div className={styles.correlationMetrics}>
                        <span className={styles.metricA}>{correlation.metric_a}</span>
                        <span className={styles.correlationSymbol}>
                          {correlation.direction === "positive" ? "↔" : "⮂"}
                        </span>
                        <span className={styles.metricB}>{correlation.metric_b}</span>
                      </div>
                      <div className={styles.correlationDetails}>
                        <span className={styles.correlationCoefficient}>
                          {correlation.correlation_coefficient.toFixed(3)}
                        </span>
                        <span className={`${styles.correlationStrength} ${styles[correlation.strength]}`}>
                          {correlation.strength.replace("_", " ").toUpperCase()}
                        </span>
                        <span className={styles.correlationPValue}>
                          p={correlation.p_value.toFixed(3)}
                        </span>
                      </div>
                    </div>
                  ))}
              </div>
            </div>
          </>
        ) : (
          <div className={styles.noCorrelations}>
            <div className={styles.emptyIcon}>🔗</div>
            <h3>No Correlation Data</h3>
            <p>
              Correlation analysis will be available once multiple metrics have been collected
              and analyzed for statistical relationships.
            </p>
            <div className={styles.analysisInfo}>
              <h4>About Correlation Analysis:</h4>
              <ul>
                <li>Measures statistical relationships between metric pairs</li>
                <li>Uses Pearson correlation coefficient by default</li>
                <li>Includes statistical significance testing (p-values)</li>
                <li>Supports lag analysis for time series relationships</li>
                <li>Helps identify predictive relationships between metrics</li>
              </ul>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
