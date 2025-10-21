"use client";

import React, { useState, useEffect, useCallback } from "react";
import {
  PerformancePredictorProps,
  PerformancePrediction,
  AnalyticsFilters,
} from "@/types/analytics";
import { analyticsApiClient, AnalyticsApiError } from "@/lib/analytics-api";
import ForecastingChart from "./ForecastingChart";
import styles from "./PerformancePredictor.module.scss";

interface PerformancePredictorState {
  predictions: PerformancePrediction[];
  selectedPrediction: PerformancePrediction | null;
  isPredicting: boolean;
  error: string | null;
  filters: AnalyticsFilters;
  showChart: boolean;
}

export default function PerformancePredictor({
  predictions: externalPredictions,
  timeSeriesData,
  onPredictionSelect,
  filters: externalFilters,
  isPredicting: externalPredicting,
  error: externalError,
}: PerformancePredictorProps) {
  const [state, setState] = useState<PerformancePredictorState>({
    predictions: externalPredictions ?? [],
    selectedPrediction: null,
    isPredicting: externalPredicting ?? false,
    error: externalError ?? null,
    filters: externalFilters ?? {
      time_range: {
        start: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(), // 30 days ago
        end: new Date().toISOString(),
      },
      granularity: "1d",
    },
    showChart: false,
  });

  // Load predictions if not provided externally
  const loadPredictions = useCallback(async () => {
    if (externalPredictions) return;

    try {
      setState((prev) => ({ ...prev, isPredicting: true, error: null }));
      const response = await analyticsApiClient.getPerformancePredictions(
        state.filters
      );
      setState((prev) => ({
        ...prev,
        predictions: response.predictions,
        isPredicting: false,
      }));
    } catch (error) {
      const errorMessage =
        error instanceof AnalyticsApiError
          ? error.message
          : "Failed to load performance predictions";
      setState((prev) => ({
        ...prev,
        isPredicting: false,
        error: errorMessage,
      }));
      console.error("Failed to load predictions:", error);
    }
  }, [externalPredictions, state.filters]);

  // Handle prediction selection
  const handlePredictionSelect = useCallback(
    (prediction: PerformancePrediction) => {
      setState((prev) => ({
        ...prev,
        selectedPrediction: prediction,
        showChart: true,
      }));
      onPredictionSelect?.(prediction);
    },
    [onPredictionSelect]
  );

  // Generate new predictions
  const generatePredictions = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, isPredicting: true, error: null }));

      const forecastingRequest = {
        metrics: [], // Will be populated from available metrics
        horizon_days: 7,
        time_range: state.filters.time_range,
        models: ["linear", "arima"],
      };

      const response = await analyticsApiClient.generateForecasting(
        forecastingRequest
      );
      setState((prev) => ({
        ...prev,
        predictions: response.predictions,
        isPredicting: false,
      }));
    } catch (error) {
      const errorMessage =
        error instanceof AnalyticsApiError
          ? error.message
          : "Failed to generate performance predictions";
      setState((prev) => ({
        ...prev,
        isPredicting: false,
        error: errorMessage,
      }));
      console.error("Failed to generate predictions:", error);
    }
  }, [state.filters.time_range]);

  // Initialize data on mount
  useEffect(() => {
    if (!externalPredictions) {
      loadPredictions();
    }
  }, [externalPredictions, loadPredictions]);

  // Update state when external props change
  useEffect(() => {
    setState((prev) => ({
      ...prev,
      predictions: externalPredictions ?? prev.predictions,
      isPredicting: externalPredicting ?? prev.isPredicting,
      error: externalError ?? prev.error,
      filters: externalFilters ?? prev.filters,
    }));
  }, [externalPredictions, externalPredicting, externalError, externalFilters]);

  const formatFactorImpact = (impact: number) => {
    const sign = impact > 0 ? "+" : "";
    return `${sign}${(impact * 100).toFixed(1)}%`;
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return styles.confidenceHigh;
    if (confidence >= 0.6) return styles.confidenceMedium;
    return styles.confidenceLow;
  };

  const getAccuracyColor = (accuracy: number) => {
    if (accuracy >= 0.8) return styles.accuracyHigh;
    if (accuracy >= 0.6) return styles.accuracyMedium;
    return styles.accuracyLow;
  };

  return (
    <div className={styles.performancePredictor}>
      <div className={styles.predictorHeader}>
        <h2>Performance Predictions</h2>
        <p className={styles.description}>
          AI-powered forecasting of system performance metrics using time series
          analysis and machine learning models.
        </p>

        <div className={styles.predictorControls}>
          <div className={styles.filterControls}>
            <div className={styles.filterGroup}>
              <label>Forecast Horizon:</label>
              <select
                value="7"
                onChange={(e) => {
                  // Update forecasting horizon if needed
                  console.log("Forecast horizon:", e.target.value);
                }}
              >
                <option value="1">1 day</option>
                <option value="3">3 days</option>
                <option value="7">7 days</option>
                <option value="14">14 days</option>
                <option value="30">30 days</option>
              </select>
            </div>
          </div>

          <button
            onClick={generatePredictions}
            disabled={state.isPredicting}
            className={styles.predictButton}
          >
            {state.isPredicting ? (
              <>
                <div className={styles.spinner}></div>
                Predicting...
              </>
            ) : (
              <>🔮 Generate Predictions</>
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

      {/* Forecasting Chart Modal */}
      {state.showChart && state.selectedPrediction && (
        <div className={styles.chartModal}>
          <div className={styles.modalHeader}>
            <h3>Forecast: {state.selectedPrediction.metric}</h3>
            <button
              onClick={() =>
                setState((prev) => ({ ...prev, showChart: false }))
              }
              className={styles.closeModal}
            >
              ✕
            </button>
          </div>
          <div className={styles.modalContent}>
            <ForecastingChart
              prediction={state.selectedPrediction}
              historicalData={timeSeriesData?.find(
                (d) => d.name === state.selectedPrediction.metric
              )}
            />
          </div>
        </div>
      )}

      {/* Predictions List */}
      <div className={styles.predictionsSection}>
        {state.isPredicting ? (
          <div className={styles.loading}>
            <div className={styles.spinner}></div>
            <p>Running forecasting models...</p>
          </div>
        ) : state.predictions.length > 0 ? (
          <>
            <div className={styles.predictionsHeader}>
              <h3>Performance Predictions ({state.predictions.length})</h3>
              <div className={styles.predictionsStats}>
                <span>
                  High Confidence:{" "}
                  {
                    state.predictions.filter((p) => p.model_accuracy >= 0.8)
                      .length
                  }
                </span>
                <span>
                  Medium Confidence:{" "}
                  {
                    state.predictions.filter(
                      (p) => p.model_accuracy >= 0.6 && p.model_accuracy < 0.8
                    ).length
                  }
                </span>
                <span>
                  Low Confidence:{" "}
                  {
                    state.predictions.filter((p) => p.model_accuracy < 0.6)
                      .length
                  }
                </span>
              </div>
            </div>

            <div className={styles.predictionsList}>
              {state.predictions.map((prediction) => (
                <div
                  key={prediction.metric}
                  className={`${styles.predictionCard} ${
                    state.selectedPrediction?.metric === prediction.metric
                      ? styles.selected
                      : ""
                  }`}
                  onClick={() => handlePredictionSelect(prediction)}
                >
                  <div className={styles.predictionHeader}>
                    <div className={styles.predictionTitle}>
                      <span className={styles.predictionMetric}>
                        {prediction.metric}
                      </span>
                      <span
                        className={`${styles.accuracyBadge} ${getAccuracyColor(
                          prediction.model_accuracy
                        )}`}
                      >
                        {(prediction.model_accuracy * 100).toFixed(0)}% Accuracy
                      </span>
                    </div>
                    <button
                      onClick={() => handlePredictionSelect(prediction)}
                      className={styles.viewChartButton}
                    >
                      📊
                    </button>
                  </div>

                  <div className={styles.predictionDetails}>
                    <div className={styles.predictionMetrics}>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>
                          Current Value:
                        </span>
                        <span className={styles.metricValue}>
                          {prediction.current_value.toFixed(2)}
                        </span>
                      </div>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>
                          Predicted Values:
                        </span>
                        <span className={styles.metricValue}>
                          {prediction.predicted_values.length} points
                        </span>
                      </div>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>
                          Forecast Horizon:
                        </span>
                        <span className={styles.metricValue}>
                          {prediction.predicted_values.length} periods
                        </span>
                      </div>
                    </div>

                    {prediction.factors && prediction.factors.length > 0 && (
                      <div className={styles.factorsSection}>
                        <h4>Key Factors</h4>
                        <div className={styles.factorsList}>
                          {prediction.factors
                            .slice(0, 3)
                            .map((factor, index) => (
                              <div key={index} className={styles.factorItem}>
                                <span className={styles.factorName}>
                                  {factor.factor}
                                </span>
                                <span className={styles.factorImpact}>
                                  {formatFactorImpact(factor.impact)}
                                </span>
                                <span
                                  className={`${
                                    styles.factorConfidence
                                  } ${getConfidenceColor(factor.confidence)}`}
                                >
                                  {(factor.confidence * 100).toFixed(0)}%
                                </span>
                              </div>
                            ))}
                        </div>
                      </div>
                    )}

                    {prediction.recommendations &&
                      prediction.recommendations.length > 0 && (
                        <div className={styles.recommendationsSection}>
                          <h4>Recommendations</h4>
                          <ul className={styles.recommendationsList}>
                            {prediction.recommendations
                              .slice(0, 2)
                              .map((rec, index) => (
                                <li
                                  key={index}
                                  className={styles.recommendationItem}
                                >
                                  {rec}
                                </li>
                              ))}
                          </ul>
                        </div>
                      )}

                    <div className={styles.confidenceIntervals}>
                      <h4>Confidence Intervals</h4>
                      <div className={styles.intervalsGrid}>
                        {prediction.confidence_intervals
                          .slice(0, 3)
                          .map((interval, index) => (
                            <div key={index} className={styles.intervalItem}>
                              <span className={styles.intervalDate}>
                                {new Date(
                                  interval.timestamp
                                ).toLocaleDateString()}
                              </span>
                              <span className={styles.intervalRange}>
                                [{interval.lower_bound.toFixed(2)},{" "}
                                {interval.upper_bound.toFixed(2)}]
                              </span>
                            </div>
                          ))}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </>
        ) : (
          <div className={styles.noPredictions}>
            <div className={styles.emptyIcon}>🔮</div>
            <h3>No Performance Predictions Available</h3>
            <p>
              Performance predictions will be available once forecasting models
              are configured and historical data is available for analysis.
            </p>
            <div className={styles.predictionInfo}>
              <h4>About Performance Prediction:</h4>
              <ul>
                <li>
                  Uses time series forecasting models (ARIMA, Prophet, Linear
                  Regression)
                </li>
                <li>
                  Provides confidence intervals for prediction uncertainty
                </li>
                <li>Analyzes factor impacts on performance metrics</li>
                <li>
                  Includes actionable recommendations based on predictions
                </li>
                <li>Supports multiple forecast horizons and granularities</li>
              </ul>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
