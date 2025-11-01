import { apiClient } from "@/lib/api-client";
import {
  AnalyticsSummary,
  Anomaly,
  Trend,
  Correlation,
  PerformancePrediction,
  TimeSeriesData,
  AnalyticsFilters,
  AnalyticsQueryRequest,
  AnomalyDetectionRequest,
  ForecastingRequest,
  GetAnalyticsSummaryResponse,
  GetAnomaliesResponse,
  GetTrendsResponse,
  GetCorrelationsResponse,
  GetPerformancePredictionsResponse,
  GetTimeSeriesDataResponse,
} from "@/types/analytics";

// Analytics API Error Class
export class AnalyticsApiError extends Error {
  constructor(
    public code: string,
    message: string,
    public retryable: boolean = false,
    public details?: string
  ) {
    super(message);
    this.name = "AnalyticsApiError";
  }
}

class AnalyticsApiClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl ?? "/api/analytics";
  }

  /**
   * Fetches analytics summary for the specified time range.
   * @param filters - Analytics filters including time range and granularity.
   * @returns A promise that resolves to analytics summary data.
   */
  async getAnalyticsSummary(
    filters?: AnalyticsFilters
  ): Promise<GetAnalyticsSummaryResponse> {
    try {
      const params = new URLSearchParams();
      if (filters?.time_range) {
        params.append("start_time", filters.time_range.start);
        params.append("end_time", filters.time_range.end);
      }
      if (filters?.granularity) {
        params.append("granularity", filters.granularity);
      }

      const response = await apiClient.request<GetAnalyticsSummaryResponse>(
        `${this.baseUrl}?${params}`
      );

      return response;
    } catch (error) {
      console.error("Failed to get analytics summary:", error);
      throw new AnalyticsApiError(
        "summary_fetch_failed",
        "Failed to retrieve analytics summary",
        true
      );
    }
  }

  /**
   * Fetches detected anomalies for the specified filters.
   * @param filters - Analytics filters to apply to anomaly detection.
   * @returns A promise that resolves to anomalies data.
   */
  async getAnomalies(
    filters?: AnalyticsFilters
  ): Promise<GetAnomaliesResponse> {
    try {
      const params = new URLSearchParams();
      params.append("analytics_type", "anomalies");

      if (filters?.time_range) {
        params.append("start_time", filters.time_range.start);
        params.append("end_time", filters.time_range.end);
      }
      if (filters?.anomaly_severity) {
        params.append("severity", filters.anomaly_severity.join(","));
      }

      const response = await apiClient.request<GetAnomaliesResponse>(
        `${this.baseUrl}?${params}`
      );

      return response;
    } catch (error) {
      console.error("Failed to get anomalies:", error);
      throw new AnalyticsApiError(
        "anomalies_fetch_failed",
        "Failed to retrieve anomaly data",
        true,
        error instanceof Error ? error.message : "Unknown error"
      );
    }
  }

  /**
   * Runs anomaly detection on specified metrics.
   * @param request - Anomaly detection request parameters.
   * @returns A promise that resolves to newly detected anomalies.
   */
  async detectAnomalies(
    request: AnomalyDetectionRequest
  ): Promise<GetAnomaliesResponse> {
    try {
      const response = await apiClient.request<GetAnomaliesResponse>(
        "/analytics/anomalies/detect",
        {
          method: "POST",
          body: JSON.stringify(request),
        }
      );
      return response;
    } catch (error) {
      console.error("Failed to detect anomalies:", error);
      throw new AnalyticsApiError(
        "anomaly_detection_failed",
        "Failed to run anomaly detection",
        true
      );
    }
  }

  /**
   * Fetches trend analysis for metrics.
   * @param filters - Analytics filters to apply to trend analysis.
   * @returns A promise that resolves to trends data.
   */
  async getTrends(filters?: AnalyticsFilters): Promise<GetTrendsResponse> {
    try {
      const params = filters
        ? new URLSearchParams({
            start: filters.time_range.start,
            end: filters.time_range.end,
            granularity: filters.granularity,
          })
        : new URLSearchParams();
      const response = await apiClient.request<GetTrendsResponse>(
        `/analytics/trends?${params}`
      );
      return response;
    } catch (error) {
      console.error("Failed to get trends:", error);
      throw new AnalyticsApiError(
        "trends_fetch_failed",
        "Failed to retrieve trend analysis",
        true
      );
    }
  }

  /**
   * Fetches correlation analysis between metrics.
   * @param filters - Analytics filters to apply to correlation analysis.
   * @param significanceThreshold - Minimum significance threshold (default: 0.05).
   * @returns A promise that resolves to correlations data.
   */
  async getCorrelations(
    filters?: AnalyticsFilters,
    significanceThreshold: number = 0.05
  ): Promise<GetCorrelationsResponse> {
    try {
      const params = new URLSearchParams({
        significance_threshold: significanceThreshold.toString(),
        ...(filters && {
          start: filters.time_range.start,
          end: filters.time_range.end,
          granularity: filters.granularity,
        }),
      });
      const response = await apiClient.request<GetCorrelationsResponse>(
        `/analytics/correlations?${params}`
      );
      return response;
    } catch (error) {
      console.error("Failed to get correlations:", error);
      throw new AnalyticsApiError(
        "correlations_fetch_failed",
        "Failed to retrieve correlation analysis",
        true
      );
    }
  }

  /**
   * Fetches performance predictions for metrics.
   * @param filters - Analytics filters to apply to predictions.
   * @returns A promise that resolves to performance predictions.
   */
  async getPerformancePredictions(
    filters?: AnalyticsFilters
  ): Promise<GetPerformancePredictionsResponse> {
    try {
      const params = filters
        ? new URLSearchParams({
            start: filters.time_range.start,
            end: filters.time_range.end,
            granularity: filters.granularity,
          })
        : new URLSearchParams();
      const response =
        await apiClient.request<GetPerformancePredictionsResponse>(
          `/analytics/predictions?${params}`
        );
      return response;
    } catch (error) {
      console.error("Failed to get performance predictions:", error);
      throw new AnalyticsApiError(
        "predictions_fetch_failed",
        "Failed to retrieve performance predictions",
        true
      );
    }
  }

  /**
   * Generates performance predictions using specified models.
   * @param request - Forecasting request parameters.
   * @returns A promise that resolves to new performance predictions.
   */
  async generateForecasting(
    request: ForecastingRequest
  ): Promise<GetPerformancePredictionsResponse> {
    try {
      const response =
        await apiClient.request<GetPerformancePredictionsResponse>(
          "/analytics/forecasting",
          {
            method: "POST",
            body: JSON.stringify(request),
          }
        );
      return response;
    } catch (error) {
      console.error("Failed to generate forecasting:", error);
      throw new AnalyticsApiError(
        "forecasting_failed",
        "Failed to generate performance predictions",
        true
      );
    }
  }

  /**
   * Fetches time series data for metrics.
   * @param filters - Analytics filters to apply to time series data.
   * @returns A promise that resolves to time series data.
   */
  async getTimeSeriesData(
    filters?: AnalyticsFilters
  ): Promise<GetTimeSeriesDataResponse> {
    try {
      const params = filters
        ? new URLSearchParams({
            start: filters.time_range.start,
            end: filters.time_range.end,
            granularity: filters.granularity,
            ...(filters.metrics && { metrics: filters.metrics.join(",") }),
          })
        : new URLSearchParams();
      const response = await apiClient.request<GetTimeSeriesDataResponse>(
        `/analytics/timeseries?${params}`
      );
      return response;
    } catch (error) {
      console.error("Failed to get time series data:", error);
      throw new AnalyticsApiError(
        "timeseries_fetch_failed",
        "Failed to retrieve time series data",
        true
      );
    }
  }

  /**
   * Acknowledges an anomaly alert.
   * @param anomalyId - The ID of the anomaly to acknowledge.
   * @param userId - Optional user ID for audit trail.
   * @returns A promise that resolves when acknowledgment is complete.
   */
  async acknowledgeAnomaly(anomalyId: string, userId?: string): Promise<void> {
    try {
      const payload = userId ? { acknowledged_by: userId } : {};
      await apiClient.request(`/analytics/anomalies/${anomalyId}/acknowledge`, {
        method: "POST",
        body: JSON.stringify(payload),
      });
    } catch (error) {
      console.error("Failed to acknowledge anomaly:", error);
      throw new AnalyticsApiError(
        "acknowledgment_failed",
        "Failed to acknowledge anomaly",
        true
      );
    }
  }

  /**
   * Dismisses an anomaly alert.
   * @param anomalyId - The ID of the anomaly to dismiss.
   * @param reason - Optional reason for dismissal.
   * @param userId - Optional user ID for audit trail.
   * @returns A promise that resolves when dismissal is complete.
   */
  async dismissAnomaly(
    anomalyId: string,
    reason?: string,
    userId?: string
  ): Promise<void> {
    try {
      const payload = {
        ...(reason && { reason }),
        ...(userId && { dismissed_by: userId }),
      };
      await apiClient.request(`/analytics/anomalies/${anomalyId}/dismiss`, {
        method: "POST",
        body: JSON.stringify(payload),
      });
    } catch (error) {
      console.error("Failed to dismiss anomaly:", error);
      throw new AnalyticsApiError(
        "dismissal_failed",
        "Failed to dismiss anomaly",
        true
      );
    }
  }

  /**
   * Runs comprehensive analytics analysis.
   * @param request - Analytics query request with all desired analyses.
   * @returns A promise that resolves to comprehensive analytics results.
   */
  async runAnalyticsQuery(request: AnalyticsQueryRequest): Promise<{
    summary?: AnalyticsSummary;
    anomalies?: Anomaly[];
    trends?: Trend[];
    predictions?: PerformancePrediction[];
    correlations?: Correlation[];
    timeSeriesData?: TimeSeriesData[];
  }> {
    try {
      const response = await apiClient.request<{
        summary?: AnalyticsSummary;
        anomalies?: Anomaly[];
        trends?: Trend[];
        predictions?: PerformancePrediction[];
        correlations?: Correlation[];
        timeSeriesData?: TimeSeriesData[];
      }>("/analytics/query", {
        method: "POST",
        body: JSON.stringify(request),
      });
      return response;
    } catch (error) {
      console.error("Failed to run analytics query:", error);
      throw new AnalyticsApiError(
        "analytics_query_failed",
        "Failed to run comprehensive analytics query",
        true
      );
    }
  }
}

// Default analytics API client instance
export const analyticsApiClient = new AnalyticsApiClient();
