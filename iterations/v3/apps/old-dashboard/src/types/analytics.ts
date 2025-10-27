export interface TimeSeriesPoint {
  timestamp: string;
  value: number;
  metadata?: { [key: string]: any };
}

export interface TimeSeriesData {
  name: string;
  data: TimeSeriesPoint[];
  color?: string;
  unit?: string;
  description?: string;
}

export interface Anomaly {
  id: string;
  metric: string;
  timestamp: string;
  value: number;
  expected_value: number;
  deviation: number;
  severity: "low" | "medium" | "high" | "critical";
  confidence: number;
  description: string;
  context?: { [key: string]: any };
}

export interface Trend {
  metric: string;
  direction: "increasing" | "decreasing" | "stable" | "volatile";
  slope: number;
  r_squared: number;
  confidence: number;
  period_days: number;
  forecast?: {
    next_value: number;
    confidence_interval: [number, number];
    timestamp: string;
  };
  description: string;
}

export interface Correlation {
  metric_a: string;
  metric_b: string;
  correlation_coefficient: number;
  p_value: number;
  strength: "weak" | "moderate" | "strong" | "very_strong";
  direction: "positive" | "negative";
  lag_periods?: number;
  confidence: number;
}

export interface PerformancePrediction {
  metric: string;
  current_value: number;
  predicted_values: TimeSeriesPoint[];
  model_accuracy: number;
  confidence_intervals: Array<{
    timestamp: string;
    lower_bound: number;
    upper_bound: number;
  }>;
  factors: Array<{
    factor: string;
    impact: number;
    confidence: number;
  }>;
  recommendations: string[];
}

export interface AnalyticsSummary {
  time_range: {
    start: string;
    end: string;
    granularity: "1m" | "5m" | "15m" | "1h" | "6h" | "1d" | "7d" | "30d";
  };
  total_metrics: number;
  total_anomalies: number;
  anomalies_by_severity: {
    low: number;
    medium: number;
    high: number;
    critical: number;
  };
  key_insights: string[];
  recommendations: string[];
  system_health_score: number;
  prediction_accuracy: number;
}

export interface AnalyticsFilters {
  time_range: {
    start: string;
    end: string;
  };
  granularity: AnalyticsSummary["time_range"]["granularity"];
  metrics?: string[];
  anomaly_severity?: Anomaly["severity"][];
  confidence_threshold?: number;
}

export interface AnalyticsConfig {
  enabled_analyses: Array<
    | "anomaly_detection"
    | "trend_analysis"
    | "correlation_analysis"
    | "performance_prediction"
    | "seasonal_decomposition"
    | "forecasting"
  >;
  anomaly_detection: {
    enabled: boolean;
    sensitivity: "low" | "medium" | "high";
    algorithms: Array<
      "zscore" | "isolation_forest" | "prophet" | "autoencoder"
    >;
    min_confidence: number;
  };
  forecasting: {
    enabled: boolean;
    horizon_days: number;
    models: Array<"linear" | "exponential" | "arima" | "prophet">;
    seasonality_detection: boolean;
  };
  alerting: {
    enabled: boolean;
    thresholds: {
      anomaly_severity: Anomaly["severity"];
      prediction_accuracy: number;
      system_health_score: number;
    };
    channels: Array<"dashboard" | "email" | "slack" | "webhook">;
  };
}

// API Response Types
export interface GetAnalyticsSummaryResponse {
  summary: AnalyticsSummary;
  last_updated: string;
}

export interface GetAnomaliesResponse {
  anomalies: Anomaly[];
  total_count: number;
  filters_applied: AnalyticsFilters;
  detection_timestamp: string;
}

export interface GetTrendsResponse {
  trends: Trend[];
  total_count: number;
  analysis_period: {
    start: string;
    end: string;
  };
}

export interface GetCorrelationsResponse {
  correlations: Correlation[];
  total_count: number;
  analysis_timestamp: string;
  significance_threshold: number;
}

export interface GetPerformancePredictionsResponse {
  predictions: PerformancePrediction[];
  total_count: number;
  forecast_horizon_days: number;
  model_timestamp: string;
}

export interface GetTimeSeriesDataResponse {
  metrics: TimeSeriesData[];
  total_points: number;
  filters_applied: AnalyticsFilters;
}

// API Request Types
export interface AnalyticsQueryRequest {
  filters: AnalyticsFilters;
  include_anomalies?: boolean;
  include_trends?: boolean;
  include_predictions?: boolean;
  limit?: number;
  offset?: number;
}

export interface AnomalyDetectionRequest {
  metrics: string[];
  time_range: {
    start: string;
    end: string;
  };
  sensitivity?: AnalyticsConfig["anomaly_detection"]["sensitivity"];
  algorithms?: AnalyticsConfig["anomaly_detection"]["algorithms"];
}

export interface ForecastingRequest {
  metrics: string[];
  horizon_days: number;
  time_range: {
    start: string;
    end: string;
  };
  models?: AnalyticsConfig["forecasting"]["models"];
}

// Component Props Types
export interface AnalyticsDashboardProps {
  summary?: AnalyticsSummary;
  filters?: AnalyticsFilters;
  onFiltersChange?: (filters: AnalyticsFilters) => void;
  onRefresh?: () => void;
  isLoading?: boolean;
  error?: string | null;
}

export interface AnomalyDetectorProps {
  anomalies?: Anomaly[];
  timeSeriesData?: TimeSeriesData[];
  onAnomalySelect?: (anomaly: Anomaly) => void;
  onAnomalyDismiss?: (anomalyId: string) => void;
  filters?: AnalyticsFilters;
  isDetecting?: boolean;
  error?: string | null;
}

export interface TrendAnalyzerProps {
  trends?: Trend[];
  timeSeriesData?: TimeSeriesData[];
  onTrendSelect?: (trend: Trend) => void;
  filters?: AnalyticsFilters;
  isAnalyzing?: boolean;
  error?: string | null;
}

export interface PerformancePredictorProps {
  predictions?: PerformancePrediction[];
  timeSeriesData?: TimeSeriesData[];
  onPredictionSelect?: (prediction: PerformancePrediction) => void;
  filters?: AnalyticsFilters;
  isPredicting?: boolean;
  error?: string | null;
}

export interface CorrelationMatrixProps {
  correlations?: Correlation[];
  metrics?: string[];
  onCorrelationSelect?: (correlation: Correlation) => void;
  minCorrelationStrength?: Correlation["strength"];
  isAnalyzing?: boolean;
  error?: string | null;
}

export interface ForecastingChartProps {
  prediction?: PerformancePrediction;
  historicalData?: TimeSeriesData;
  showConfidenceIntervals?: boolean;
  onTimeRangeChange?: (start: string, end: string) => void;
  isLoading?: boolean;
  error?: string | null;
}

export interface AnalyticsCardProps {
  title: string;
  value: string | number;
  change?: {
    value: number;
    type: "increase" | "decrease" | "neutral";
    period: string;
  };
  icon?: string;
  status?: "success" | "warning" | "error" | "info";
  trend?: "up" | "down" | "stable";
  description?: string;
  onClick?: () => void;
}

// Utility Types
export type AnalyticsTimeRange = AnalyticsSummary["time_range"];
export type AnalyticsGranularity = AnalyticsTimeRange["granularity"];
export type AnomalySeverity = Anomaly["severity"];
export type TrendDirection = Trend["direction"];
export type CorrelationStrength = Correlation["strength"];

// Chart Data Types (for integration with charting libraries)
export interface ChartDataPoint {
  x: string | number | Date;
  y: number;
  label?: string;
  color?: string;
  metadata?: { [key: string]: any };
}

export interface ChartSeries {
  name: string;
  data: ChartDataPoint[];
  type?: "line" | "bar" | "area" | "scatter";
  color?: string;
  yAxis?: string;
}

export interface ChartConfig {
  title?: string;
  xAxis: {
    type: "datetime" | "category" | "linear";
    title?: string;
    format?: string;
  };
  yAxis: {
    title?: string;
    format?: string;
    min?: number;
    max?: number;
  };
  series: ChartSeries[];
  annotations?: Array<{
    type: "line" | "area" | "point";
    x?: string | number | Date;
    y?: number;
    label?: string;
    color?: string;
  }>;
}

// Alert Types
export interface AnalyticsAlert {
  id: string;
  type: "anomaly" | "trend" | "prediction" | "system_health";
  severity: "info" | "warning" | "error" | "critical";
  title: string;
  description: string;
  timestamp: string;
  metric?: string;
  value?: number;
  threshold?: number;
  acknowledged: boolean;
  acknowledged_by?: string;
  acknowledged_at?: string;
}

// Dashboard State Types
export interface AnalyticsDashboardState {
  summary: AnalyticsSummary | null;
  anomalies: Anomaly[];
  trends: Trend[];
  predictions: PerformancePrediction[];
  correlations: Correlation[];
  timeSeriesData: TimeSeriesData[];
  filters: AnalyticsFilters;
  isLoading: boolean;
  error: string | null;
  lastUpdated: Date | null;
  alerts: AnalyticsAlert[];
}
