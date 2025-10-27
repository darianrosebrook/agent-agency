/**
 * ML Analytics API Client
 * API client for predictive analytics, anomaly detection, and business intelligence
 *
 * @author @darianrosebrook
 */

import { ApiClient } from './api-client';

export interface PredictiveModel {
  id: string;
  name: string;
  description: string;
  type: 'regression' | 'classification' | 'clustering' | 'forecasting' | 'anomaly_detection';
  targetVariable: string;
  features: string[];
  accuracy: number;
  precision: number;
  recall: number;
  f1Score: number;
  trainingDataSize: number;
  lastTrained: Date;
  status: 'training' | 'ready' | 'failed' | 'deprecated';
  performance: {
    mse?: number;
    rmse?: number;
    mae?: number;
    r2?: number;
    auc?: number;
  };
}

export interface PredictionResult {
  id: string;
  modelId: string;
  input: Record<string, any>;
  prediction: any;
  confidence: number;
  probability?: number;
  timestamp: Date;
  executionTime: number;
  featureImportance?: Record<string, number>;
}

export interface AnomalyDetection {
  id: string;
  name: string;
  description: string;
  algorithm: 'isolation_forest' | 'one_class_svm' | 'local_outlier_factor' | 'autoencoder' | 'prophet' | 'arima';
  targetMetric: string;
  sensitivity: number; // 0-1
  status: 'active' | 'inactive' | 'training';
  lastUpdated: Date;
  performance: {
    truePositives: number;
    falsePositives: number;
    trueNegatives: number;
    falseNegatives: number;
    precision: number;
    recall: number;
    f1Score: number;
  };
}

export interface AnomalyAlert {
  id: string;
  detectionId: string;
  timestamp: Date;
  severity: 'low' | 'medium' | 'high' | 'critical';
  score: number; // anomaly score 0-1
  metric: string;
  value: number;
  expectedValue?: number;
  deviation: number;
  description: string;
  acknowledged: boolean;
  acknowledgedBy?: string;
  acknowledgedAt?: Date;
  resolved: boolean;
  resolvedAt?: Date;
  context: Record<string, any>;
}

export interface TrendAnalysis {
  id: string;
  metric: string;
  period: '1h' | '24h' | '7d' | '30d' | '90d';
  trend: 'increasing' | 'decreasing' | 'stable' | 'volatile';
  slope: number;
  r2: number;
  seasonality: boolean;
  seasonalityPeriod?: number;
  forecast: Array<{
    timestamp: Date;
    value: number;
    lowerBound: number;
    upperBound: number;
  }>;
  confidence: number;
  lastUpdated: Date;
}

export interface BusinessMetric {
  id: string;
  name: string;
  description: string;
  category: 'revenue' | 'user_engagement' | 'performance' | 'quality' | 'efficiency' | 'security';
  type: 'count' | 'rate' | 'ratio' | 'duration' | 'percentage';
  unit: string;
  value: number;
  previousValue?: number;
  change: number;
  changePercent: number;
  target?: number;
  status: 'on_track' | 'at_risk' | 'off_track' | 'achieved';
  trend: 'up' | 'down' | 'stable';
  lastUpdated: Date;
  dataPoints: Array<{
    timestamp: Date;
    value: number;
  }>;
}

export interface BusinessIntelligenceReport {
  id: string;
  name: string;
  description: string;
  type: 'daily' | 'weekly' | 'monthly' | 'quarterly';
  period: {
    start: Date;
    end: Date;
  };
  metrics: BusinessMetric[];
  insights: Array<{
    type: 'trend' | 'anomaly' | 'correlation' | 'prediction';
    title: string;
    description: string;
    severity: 'low' | 'medium' | 'high' | 'critical';
    data: any;
  }>;
  recommendations: Array<{
    priority: 'low' | 'medium' | 'high';
    title: string;
    description: string;
    impact: 'low' | 'medium' | 'high';
    effort: 'low' | 'medium' | 'high';
  }>;
  generatedAt: Date;
  generatedBy: string;
}

export interface CorrelationAnalysis {
  id: string;
  metricA: string;
  metricB: string;
  correlation: number; // -1 to 1
  pValue: number;
  significance: 'none' | 'weak' | 'moderate' | 'strong';
  lag?: number; // for time series correlation
  direction: 'positive' | 'negative' | 'none';
  confidence: number;
  sampleSize: number;
  period: {
    start: Date;
    end: Date;
  };
}

export interface MLExperiment {
  id: string;
  name: string;
  description: string;
  type: 'model_training' | 'hyperparameter_tuning' | 'feature_selection' | 'data_analysis';
  status: 'pending' | 'running' | 'completed' | 'failed';
  startedAt?: Date;
  completedAt?: Date;
  progress?: {
    current: number;
    total: number;
    message: string;
  };
  parameters: Record<string, any>;
  results?: {
    metrics: Record<string, number>;
    artifacts: string[];
    modelPath?: string;
    bestParameters?: Record<string, any>;
  };
  error?: string;
}

export interface FeatureImportance {
  feature: string;
  importance: number;
  rank: number;
  category?: string;
  description?: string;
}

export interface ModelPerformanceComparison {
  models: Array<{
    modelId: string;
    name: string;
    metrics: Record<string, number>;
    rank: number;
  }>;
  baseline?: {
    name: string;
    metrics: Record<string, number>;
  };
  comparison: {
    bestModel: string;
    improvements: Record<string, number>;
    tradeoffs: Record<string, string>;
  };
  timestamp: Date;
}

export interface ForecastingModel {
  id: string;
  name: string;
  description: string;
  algorithm: 'arima' | 'prophet' | 'lstm' | 'xgboost' | 'linear_regression';
  target: string;
  horizon: number; // periods to forecast
  frequency: 'hourly' | 'daily' | 'weekly' | 'monthly';
  accuracy: {
    mape: number;
    rmse: number;
    mae: number;
  };
  forecast: Array<{
    timestamp: Date;
    value: number;
    lowerBound: number;
    upperBound: number;
    confidence: number;
  }>;
  lastUpdated: Date;
}

export interface DataQualityReport {
  id: string;
  name: string;
  description: string;
  dataset: string;
  timestamp: Date;
  metrics: {
    completeness: number;
    accuracy: number;
    consistency: number;
    timeliness: number;
    validity: number;
    uniqueness: number;
  };
  issues: Array<{
    type: 'missing_values' | 'duplicates' | 'outliers' | 'inconsistencies' | 'invalid_formats';
    severity: 'low' | 'medium' | 'high';
    count: number;
    percentage: number;
    description: string;
    affectedColumns?: string[];
  }>;
  recommendations: Array<{
    priority: 'low' | 'medium' | 'high';
    action: string;
    impact: string;
  }>;
}

export class MLAnalyticsApiClient {
  private apiClient: ApiClient;

  constructor(baseUrl: string = '/api/ml-analytics') {
    this.apiClient = new ApiClient({ baseUrl });
  }

  /**
   * Predictive Modeling endpoints
   */
  async getPredictiveModels(): Promise<PredictiveModel[]> {
    const response = await this.apiClient.request<PredictiveModel[]>('/models');
    return response;
  }

  async getPredictiveModel(modelId: string): Promise<PredictiveModel> {
    const response = await this.apiClient.request<PredictiveModel>(`/models/${modelId}`);
    return response;
  }

  async createPrediction(modelId: string, input: Record<string, any>): Promise<PredictionResult> {
    const response = await this.apiClient.request<PredictionResult>(`/models/${modelId}/predict`, {
      method: 'POST',
      body: JSON.stringify({ input })
    });
    return response;
  }

  async getModelPredictions(modelId: string, limit: number = 50): Promise<PredictionResult[]> {
    const response = await this.apiClient.request<PredictionResult[]>(
      `/models/${modelId}/predictions?limit=${limit}`
    );
    return response;
  }

  async getFeatureImportance(modelId: string): Promise<FeatureImportance[]> {
    const response = await this.apiClient.request<FeatureImportance[]>(
      `/models/${modelId}/feature-importance`
    );
    return response;
  }

  async retrainModel(modelId: string, parameters?: Record<string, any>): Promise<{
    experimentId: string;
    status: 'started';
  }> {
    const response = await this.apiClient.request<{
      experimentId: string;
      status: 'started';
    }>(`/models/${modelId}/retrain`, {
      method: 'POST',
      body: JSON.stringify({ parameters })
    });
    return response;
  }

  /**
   * Anomaly Detection endpoints
   */
  async getAnomalyDetectors(): Promise<AnomalyDetection[]> {
    const response = await this.apiClient.request<AnomalyDetection[]>('/anomaly-detectors');
    return response;
  }

  async getAnomalyDetector(detectorId: string): Promise<AnomalyDetection> {
    const response = await this.apiClient.request<AnomalyDetection>(`/anomaly-detectors/${detectorId}`);
    return response;
  }

  async getAnomalyAlerts(
    detectorId?: string,
    status?: 'active' | 'acknowledged' | 'resolved',
    severity?: AnomalyAlert['severity'][],
    limit: number = 50
  ): Promise<AnomalyAlert[]> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (detectorId) params.append('detectorId', detectorId);
    if (status) params.append('status', status);
    if (severity) params.append('severity', severity.join(','));

    const response = await this.apiClient.request<AnomalyAlert[]>(
      `/anomaly-alerts?${params.toString()}`
    );
    return response;
  }

  async acknowledgeAnomalyAlert(alertId: string): Promise<void> {
    await this.apiClient.request<void>(`/anomaly-alerts/${alertId}/acknowledge`, {
      method: 'POST'
    });
  }

  async resolveAnomalyAlert(alertId: string, resolution?: string): Promise<void> {
    await this.apiClient.request<void>(`/anomaly-alerts/${alertId}/resolve`, {
      method: 'POST',
      body: JSON.stringify({ resolution })
    });
  }

  async createAnomalyDetector(config: {
    name: string;
    description: string;
    algorithm: AnomalyDetection['algorithm'];
    targetMetric: string;
    sensitivity: number;
    parameters?: Record<string, any>;
  }): Promise<AnomalyDetection> {
    const response = await this.apiClient.request<AnomalyDetection>('/anomaly-detectors', {
      method: 'POST',
      body: JSON.stringify(config)
    });
    return response;
  }

  async updateAnomalyDetector(
    detectorId: string,
    updates: Partial<AnomalyDetection>
  ): Promise<AnomalyDetection> {
    const response = await this.apiClient.request<AnomalyDetection>(
      `/anomaly-detectors/${detectorId}`,
      {
        method: 'PATCH',
        body: JSON.stringify(updates)
      }
    );
    return response;
  }

  /**
   * Trend Analysis endpoints
   */
  async getTrendAnalysis(
    metric: string,
    period: TrendAnalysis['period'] = '24h'
  ): Promise<TrendAnalysis> {
    const response = await this.apiClient.request<TrendAnalysis>(
      `/trends?metric=${encodeURIComponent(metric)}&period=${period}`
    );
    return response;
  }

  async getMultipleTrendAnalysis(
    metrics: string[],
    period: TrendAnalysis['period'] = '24h'
  ): Promise<TrendAnalysis[]> {
    const params = new URLSearchParams({
      metrics: metrics.join(','),
      period
    });

    const response = await this.apiClient.request<TrendAnalysis[]>(
      `/trends/batch?${params.toString()}`
    );
    return response;
  }

  async getForecastingModels(): Promise<ForecastingModel[]> {
    const response = await this.apiClient.request<ForecastingModel[]>('/forecasting/models');
    return response;
  }

  async createForecast(modelId: string, horizon: number): Promise<{
    forecast: ForecastingModel['forecast'];
    accuracy: ForecastingModel['accuracy'];
  }> {
    const response = await this.apiClient.request<{
      forecast: ForecastingModel['forecast'];
      accuracy: ForecastingModel['accuracy'];
    }>(`/forecasting/models/${modelId}/forecast`, {
      method: 'POST',
      body: JSON.stringify({ horizon })
    });
    return response;
  }

  /**
   * Business Intelligence endpoints
   */
  async getBusinessMetrics(
    category?: BusinessMetric['category'],
    limit: number = 50
  ): Promise<BusinessMetric[]> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (category) params.append('category', category);

    const response = await this.apiClient.request<BusinessMetric[]>(
      `/business/metrics?${params.toString()}`
    );
    return response;
  }

  async getBusinessMetric(metricId: string): Promise<BusinessMetric> {
    const response = await this.apiClient.request<BusinessMetric>(`/business/metrics/${metricId}`);
    return response;
  }

  async generateBusinessReport(
    type: BusinessIntelligenceReport['type'],
    period?: { start: Date; end: Date }
  ): Promise<BusinessIntelligenceReport> {
    const body: any = { type };
    if (period) {
      body.period = {
        start: period.start.toISOString(),
        end: period.end.toISOString()
      };
    }

    const response = await this.apiClient.request<BusinessIntelligenceReport>('/business/reports', {
      method: 'POST',
      body: JSON.stringify(body)
    });
    return response;
  }

  async getBusinessReports(
    type?: BusinessIntelligenceReport['type'],
    limit: number = 10
  ): Promise<BusinessIntelligenceReport[]> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (type) params.append('type', type);

    const response = await this.apiClient.request<BusinessIntelligenceReport[]>(
      `/business/reports?${params.toString()}`
    );
    return response;
  }

  /**
   * Correlation Analysis endpoints
   */
  async analyzeCorrelation(
    metricA: string,
    metricB: string,
    period: { start: Date; end: Date }
  ): Promise<CorrelationAnalysis> {
    const response = await this.apiClient.request<CorrelationAnalysis>('/correlation/analyze', {
      method: 'POST',
      body: JSON.stringify({
        metricA,
        metricB,
        period: {
          start: period.start.toISOString(),
          end: period.end.toISOString()
        }
      })
    });
    return response;
  }

  async getCorrelations(
    threshold: number = 0.5,
    limit: number = 50
  ): Promise<CorrelationAnalysis[]> {
    const response = await this.apiClient.request<CorrelationAnalysis[]>(
      `/correlation?threshold=${threshold}&limit=${limit}`
    );
    return response;
  }

  /**
   * ML Experiments endpoints
   */
  async getMLExperiments(
    type?: MLExperiment['type'],
    status?: MLExperiment['status']
  ): Promise<MLExperiment[]> {
    const params = new URLSearchParams();
    if (type) params.append('type', type);
    if (status) params.append('status', status);

    const query = params.toString() ? `?${params.toString()}` : '';
    const response = await this.apiClient.request<MLExperiment[]>(`/experiments${query}`);
    return response;
  }

  async getMLExperiment(experimentId: string): Promise<MLExperiment> {
    const response = await this.apiClient.request<MLExperiment>(`/experiments/${experimentId}`);
    return response;
  }

  async createMLExperiment(experiment: Omit<MLExperiment, 'id' | 'status' | 'startedAt'>): Promise<MLExperiment> {
    const response = await this.apiClient.request<MLExperiment>('/experiments', {
      method: 'POST',
      body: JSON.stringify(experiment)
    });
    return response;
  }

  async cancelMLExperiment(experimentId: string): Promise<void> {
    await this.apiClient.request<void>(`/experiments/${experimentId}/cancel`, {
      method: 'POST'
    });
  }

  /**
   * Model Comparison endpoints
   */
  async compareModels(
    modelIds: string[],
    baseline?: string
  ): Promise<ModelPerformanceComparison> {
    const response = await this.apiClient.request<ModelPerformanceComparison>('/models/compare', {
      method: 'POST',
      body: JSON.stringify({ modelIds, baseline })
    });
    return response;
  }

  /**
   * Data Quality endpoints
   */
  async getDataQualityReports(
    dataset?: string,
    limit: number = 10
  ): Promise<DataQualityReport[]> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (dataset) params.append('dataset', dataset);

    const response = await this.apiClient.request<DataQualityReport[]>(
      `/data-quality/reports?${params.toString()}`
    );
    return response;
  }

  async generateDataQualityReport(
    dataset: string,
    options?: {
      includeMetrics?: string[];
      sampleSize?: number;
    }
  ): Promise<DataQualityReport> {
    const response = await this.apiClient.request<DataQualityReport>('/data-quality/reports', {
      method: 'POST',
      body: JSON.stringify({ dataset, options })
    });
    return response;
  }

  /**
   * Real-time Analytics endpoints
   */
  async getRealTimeMetrics(
    metrics: string[],
    window: '1m' | '5m' | '15m' | '1h' = '5m'
  ): Promise<Record<string, Array<{ timestamp: Date; value: number }>>> {
    const response = await this.apiClient.request<Record<string, Array<{ timestamp: Date; value: number }>>>(
      `/realtime/metrics?metrics=${metrics.join(',')}&window=${window}`
    );
    return response;
  }

  async subscribeToRealTimeUpdates(
    metrics: string[],
    callback: (data: Record<string, { timestamp: Date; value: number }>) => void
  ): Promise<() => void> {
    // This would typically use WebSocket or Server-Sent Events
    // For now, we'll implement polling
    const interval = setInterval(async () => {
      try {
        const data = await this.getRealTimeMetrics(metrics);
        const latestData: Record<string, { timestamp: Date; value: number }> = {};

        Object.entries(data).forEach(([metric, points]) => {
          if (points.length > 0) {
            latestData[metric] = points[points.length - 1];
          }
        });

        callback(latestData);
      } catch (error) {
        console.error('Failed to fetch real-time metrics:', error);
      }
    }, 30000); // Update every 30 seconds

    return () => clearInterval(interval);
  }

  /**
   * Analytics Configuration endpoints
   */
  async updateAnomalyDetectorSensitivity(
    detectorId: string,
    sensitivity: number
  ): Promise<AnomalyDetection> {
    const response = await this.apiClient.request<AnomalyDetection>(
      `/anomaly-detectors/${detectorId}/sensitivity`,
      {
        method: 'PATCH',
        body: JSON.stringify({ sensitivity })
      }
    );
    return response;
  }

  async setBusinessMetricTarget(
    metricId: string,
    target: number
  ): Promise<BusinessMetric> {
    const response = await this.apiClient.request<BusinessMetric>(
      `/business/metrics/${metricId}/target`,
      {
        method: 'PATCH',
        body: JSON.stringify({ target })
      }
    );
    return response;
  }

  async createCustomMetric(
    metric: Omit<BusinessMetric, 'id' | 'value' | 'previousValue' | 'change' | 'changePercent' | 'lastUpdated' | 'dataPoints'>
  ): Promise<BusinessMetric> {
    const response = await this.apiClient.request<BusinessMetric>('/business/metrics', {
      method: 'POST',
      body: JSON.stringify(metric)
    });
    return response;
  }
}

// Export singleton instance
export const mlAnalyticsApiClient = new MLAnalyticsApiClient();
