/**
 * ML Analytics Store
 * Zustand store for predictive analytics, anomaly detection, and business intelligence state management
 *
 * @author @darianrosebrook
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import {
  PredictiveModel,
  PredictionResult,
  AnomalyDetection,
  AnomalyAlert,
  TrendAnalysis,
  BusinessMetric,
  BusinessIntelligenceReport,
  CorrelationAnalysis,
  MLExperiment,
  ForecastingModel,
  DataQualityReport,
} from '@/lib/ml-analytics-api';

interface MLAnalyticsState {
  // Core data
  predictiveModels: PredictiveModel[];
  predictions: Record<string, PredictionResult[]>; // keyed by modelId
  anomalyDetectors: AnomalyDetection[];
  anomalyAlerts: AnomalyAlert[];
  trendAnalyses: Record<string, TrendAnalysis>; // keyed by metric
  businessMetrics: BusinessMetric[];
  businessReports: BusinessIntelligenceReport[];
  correlations: CorrelationAnalysis[];
  mlExperiments: MLExperiment[];
  forecastingModels: ForecastingModel[];
  dataQualityReports: DataQualityReport[];

  // UI state
  selectedModel: PredictiveModel | null;
  selectedAnomalyDetector: AnomalyDetection | null;
  selectedBusinessReport: BusinessIntelligenceReport | null;
  selectedMLExperiment: MLExperiment | null;
  realTimeMetrics: Record<string, Array<{ timestamp: Date; value: number }>>;
  activeSubscriptions: Set<string>;

  // Loading states
  loading: {
    models: boolean;
    predictions: boolean;
    anomalies: boolean;
    trends: boolean;
    business: boolean;
    correlations: boolean;
    experiments: boolean;
    forecasting: boolean;
    dataQuality: boolean;
  };

  // Error states
  errors: {
    models: string | null;
    predictions: string | null;
    anomalies: string | null;
    trends: string | null;
    business: string | null;
    correlations: string | null;
    experiments: string | null;
    forecasting: string | null;
    dataQuality: string | null;
  };

  // Pagination and filtering
  pagination: {
    alertsPage: number;
    reportsPage: number;
    experimentsPage: number;
    limit: number;
  };

  filters: {
    alertSeverity: AnomalyAlert['severity'][] | null;
    alertStatus: 'active' | 'acknowledged' | 'resolved' | null;
    modelType: PredictiveModel['type'][] | null;
    modelStatus: PredictiveModel['status'][] | null;
    experimentType: MLExperiment['type'][] | null;
    experimentStatus: MLExperiment['status'][] | null;
    businessCategory: BusinessMetric['category'][] | null;
    reportType: BusinessIntelligenceReport['type'][] | null;
  };

  // Settings
  settings: {
    autoRefresh: boolean;
    refreshInterval: number; // seconds
    realTimeEnabled: boolean;
    alertThreshold: number; // anomaly score threshold
    forecastHorizon: number; // periods
    correlationThreshold: number;
  };
}

interface MLAnalyticsActions {
  // Core data actions
  setPredictiveModels: (models: PredictiveModel[]) => void;
  addPredictiveModel: (model: PredictiveModel) => void;
  updatePredictiveModel: (modelId: string, updates: Partial<PredictiveModel>) => void;
  removePredictiveModel: (modelId: string) => void;
  setPredictions: (modelId: string, predictions: PredictionResult[]) => void;
  addPrediction: (modelId: string, prediction: PredictionResult) => void;
  setAnomalyDetectors: (detectors: AnomalyDetection[]) => void;
  addAnomalyDetector: (detector: AnomalyDetection) => void;
  updateAnomalyDetector: (detectorId: string, updates: Partial<AnomalyDetection>) => void;
  setAnomalyAlerts: (alerts: AnomalyAlert[]) => void;
  addAnomalyAlert: (alert: AnomalyAlert) => void;
  updateAnomalyAlert: (alertId: string, updates: Partial<AnomalyAlert>) => void;
  setTrendAnalyses: (trends: Record<string, TrendAnalysis>) => void;
  updateTrendAnalysis: (metric: string, trend: TrendAnalysis) => void;
  setBusinessMetrics: (metrics: BusinessMetric[]) => void;
  updateBusinessMetric: (metricId: string, updates: Partial<BusinessMetric>) => void;
  setBusinessReports: (reports: BusinessIntelligenceReport[]) => void;
  addBusinessReport: (report: BusinessIntelligenceReport) => void;
  setCorrelations: (correlations: CorrelationAnalysis[]) => void;
  addCorrelation: (correlation: CorrelationAnalysis) => void;
  setMLExperiments: (experiments: MLExperiment[]) => void;
  addMLExperiment: (experiment: MLExperiment) => void;
  updateMLExperiment: (experimentId: string, updates: Partial<MLExperiment>) => void;
  setForecastingModels: (models: ForecastingModel[]) => void;
  updateForecastingModel: (modelId: string, updates: Partial<ForecastingModel>) => void;
  setDataQualityReports: (reports: DataQualityReport[]) => void;
  addDataQualityReport: (report: DataQualityReport) => void;
  setRealTimeMetrics: (metrics: Record<string, Array<{ timestamp: Date; value: number }>>) => void;
  updateRealTimeMetric: (metric: string, dataPoint: { timestamp: Date; value: number }) => void;

  // UI state actions
  setSelectedModel: (model: PredictiveModel | null) => void;
  setSelectedAnomalyDetector: (detector: AnomalyDetection | null) => void;
  setSelectedBusinessReport: (report: BusinessIntelligenceReport | null) => void;
  setSelectedMLExperiment: (experiment: MLExperiment | null) => void;
  addActiveSubscription: (metric: string) => void;
  removeActiveSubscription: (metric: string) => void;
  clearActiveSubscriptions: () => void;

  // Loading actions
  setLoading: (key: keyof MLAnalyticsState['loading'], loading: boolean) => void;
  setError: (key: keyof MLAnalyticsState['errors'], error: string | null) => void;
  clearErrors: () => void;

  // Pagination actions
  setPagination: (pagination: Partial<MLAnalyticsState['pagination']>) => void;
  nextAlertsPage: () => void;
  nextReportsPage: () => void;
  nextExperimentsPage: () => void;
  resetPagination: () => void;

  // Filter actions
  setFilters: (filters: Partial<MLAnalyticsState['filters']>) => void;
  clearFilters: () => void;

  // Settings actions
  updateSettings: (settings: Partial<MLAnalyticsState['settings']>) => void;

  // Utility actions
  reset: () => void;
}

const initialState: MLAnalyticsState = {
  predictiveModels: [],
  predictions: {},
  anomalyDetectors: [],
  anomalyAlerts: [],
  trendAnalyses: {},
  businessMetrics: [],
  businessReports: [],
  correlations: [],
  mlExperiments: [],
  forecastingModels: [],
  dataQualityReports: [],
  selectedModel: null,
  selectedAnomalyDetector: null,
  selectedBusinessReport: null,
  selectedMLExperiment: null,
  realTimeMetrics: {},
  activeSubscriptions: new Set(),
  loading: {
    models: false,
    predictions: false,
    anomalies: false,
    trends: false,
    business: false,
    correlations: false,
    experiments: false,
    forecasting: false,
    dataQuality: false,
  },
  errors: {
    models: null,
    predictions: null,
    anomalies: null,
    trends: null,
    business: null,
    correlations: null,
    experiments: null,
    forecasting: null,
    dataQuality: null,
  },
  pagination: {
    alertsPage: 1,
    reportsPage: 1,
    experimentsPage: 1,
    limit: 50,
  },
  filters: {
    alertSeverity: null,
    alertStatus: null,
    modelType: null,
    modelStatus: null,
    experimentType: null,
    experimentStatus: null,
    businessCategory: null,
    reportType: null,
  },
  settings: {
    autoRefresh: true,
    refreshInterval: 30,
    realTimeEnabled: true,
    alertThreshold: 0.7,
    forecastHorizon: 24,
    correlationThreshold: 0.5,
  },
};

export const useMLAnalyticsStore = create<MLAnalyticsState & MLAnalyticsActions>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // Core data actions
      setPredictiveModels: (models) => set({ predictiveModels: models }),
      addPredictiveModel: (model) => set((state) => ({
        predictiveModels: [model, ...state.predictiveModels]
      })),
      updatePredictiveModel: (modelId, updates) => set((state) => ({
        predictiveModels: state.predictiveModels.map(model =>
          model.id === modelId ? { ...model, ...updates } : model
        ),
        selectedModel: state.selectedModel?.id === modelId
          ? { ...state.selectedModel, ...updates }
          : state.selectedModel
      })),
      removePredictiveModel: (modelId) => set((state) => ({
        predictiveModels: state.predictiveModels.filter(model => model.id !== modelId),
        selectedModel: state.selectedModel?.id === modelId ? null : state.selectedModel
      })),
      setPredictions: (modelId, predictions) => set((state) => ({
        predictions: { ...state.predictions, [modelId]: predictions }
      })),
      addPrediction: (modelId, prediction) => set((state) => ({
        predictions: {
          ...state.predictions,
          [modelId]: [prediction, ...(state.predictions[modelId] || []).slice(0, 99)] // Keep last 100
        }
      })),
      setAnomalyDetectors: (detectors) => set({ anomalyDetectors: detectors }),
      addAnomalyDetector: (detector) => set((state) => ({
        anomalyDetectors: [detector, ...state.anomalyDetectors]
      })),
      updateAnomalyDetector: (detectorId, updates) => set((state) => ({
        anomalyDetectors: state.anomalyDetectors.map(detector =>
          detector.id === detectorId ? { ...detector, ...updates } : detector
        ),
        selectedAnomalyDetector: state.selectedAnomalyDetector?.id === detectorId
          ? { ...state.selectedAnomalyDetector, ...updates }
          : state.selectedAnomalyDetector
      })),
      setAnomalyAlerts: (alerts) => set({ anomalyAlerts: alerts }),
      addAnomalyAlert: (alert) => set((state) => ({
        anomalyAlerts: [alert, ...state.anomalyAlerts]
      })),
      updateAnomalyAlert: (alertId, updates) => set((state) => ({
        anomalyAlerts: state.anomalyAlerts.map(alert =>
          alert.id === alertId ? { ...alert, ...updates } : alert
        )
      })),
      setTrendAnalyses: (trends) => set({ trendAnalyses: trends }),
      updateTrendAnalysis: (metric, trend) => set((state) => ({
        trendAnalyses: { ...state.trendAnalyses, [metric]: trend }
      })),
      setBusinessMetrics: (metrics) => set({ businessMetrics: metrics }),
      updateBusinessMetric: (metricId, updates) => set((state) => ({
        businessMetrics: state.businessMetrics.map(metric =>
          metric.id === metricId ? { ...metric, ...updates } : metric
        )
      })),
      setBusinessReports: (reports) => set({ businessReports: reports }),
      addBusinessReport: (report) => set((state) => ({
        businessReports: [report, ...state.businessReports.slice(0, 9)] // Keep last 10
      })),
      setCorrelations: (correlations) => set({ correlations }),
      addCorrelation: (correlation) => set((state) => ({
        correlations: [correlation, ...state.correlations.slice(0, 99)] // Keep last 100
      })),
      setMLExperiments: (experiments) => set({ mlExperiments: experiments }),
      addMLExperiment: (experiment) => set((state) => ({
        mlExperiments: [experiment, ...state.mlExperiments]
      })),
      updateMLExperiment: (experimentId, updates) => set((state) => ({
        mlExperiments: state.mlExperiments.map(experiment =>
          experiment.id === experimentId ? { ...experiment, ...updates } : experiment
        ),
        selectedMLExperiment: state.selectedMLExperiment?.id === experimentId
          ? { ...state.selectedMLExperiment, ...updates }
          : state.selectedMLExperiment
      })),
      setForecastingModels: (models) => set({ forecastingModels: models }),
      updateForecastingModel: (modelId, updates) => set((state) => ({
        forecastingModels: state.forecastingModels.map(model =>
          model.id === modelId ? { ...model, ...updates } : model
        )
      })),
      setDataQualityReports: (reports) => set({ dataQualityReports: reports }),
      addDataQualityReport: (report) => set((state) => ({
        dataQualityReports: [report, ...state.dataQualityReports.slice(0, 9)] // Keep last 10
      })),
      setRealTimeMetrics: (metrics) => set({ realTimeMetrics: metrics }),
      updateRealTimeMetric: (metric, dataPoint) => set((state) => ({
        realTimeMetrics: {
          ...state.realTimeMetrics,
          [metric]: [
            ...(state.realTimeMetrics[metric] || []).slice(-99), // Keep last 100 points
            dataPoint
          ]
        }
      })),

      // UI state actions
      setSelectedModel: (model) => set({ selectedModel: model }),
      setSelectedAnomalyDetector: (detector) => set({ selectedAnomalyDetector: detector }),
      setSelectedBusinessReport: (report) => set({ selectedBusinessReport: report }),
      setSelectedMLExperiment: (experiment) => set({ selectedMLExperiment: experiment }),
      addActiveSubscription: (metric) => set((state) => ({
        activeSubscriptions: new Set([...state.activeSubscriptions, metric])
      })),
      removeActiveSubscription: (metric) => set((state) => {
        const newSubscriptions = new Set(state.activeSubscriptions);
        newSubscriptions.delete(metric);
        return { activeSubscriptions: newSubscriptions };
      }),
      clearActiveSubscriptions: () => set({ activeSubscriptions: new Set() }),

      // Loading actions
      setLoading: (key, loading) => set((state) => ({
        loading: { ...state.loading, [key]: loading }
      })),
      setError: (key, error) => set((state) => ({
        errors: { ...state.errors, [key]: error }
      })),
      clearErrors: () => set({ errors: initialState.errors }),

      // Pagination actions
      setPagination: (pagination) => set((state) => ({
        pagination: { ...state.pagination, ...pagination }
      })),
      nextAlertsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          alertsPage: state.pagination.alertsPage + 1
        }
      })),
      nextReportsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          reportsPage: state.pagination.reportsPage + 1
        }
      })),
      nextExperimentsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          experimentsPage: state.pagination.experimentsPage + 1
        }
      })),
      resetPagination: () => set({ pagination: initialState.pagination }),

      // Filter actions
      setFilters: (filters) => set((state) => ({
        filters: { ...state.filters, ...filters }
      })),
      clearFilters: () => set({ filters: initialState.filters }),

      // Settings actions
      updateSettings: (settings) => set((state) => ({
        settings: { ...state.settings, ...settings }
      })),

      // Utility actions
      reset: () => set(initialState),
    }),
    {
      name: 'ml-analytics-store',
    }
  )
);

// Selector hooks for better performance
export const usePredictiveModels = () => useMLAnalyticsStore((state) => state.predictiveModels);
export const useAnomalyAlerts = () => useMLAnalyticsStore((state) => state.anomalyAlerts);
export const useBusinessMetrics = () => useMLAnalyticsStore((state) => state.businessMetrics);
export const useTrendAnalyses = () => useMLAnalyticsStore((state) => state.trendAnalyses);
export const useMLExperiments = () => useMLAnalyticsStore((state) => state.mlExperiments);
export const useRealTimeMetrics = () => useMLAnalyticsStore((state) => state.realTimeMetrics);
export const useSelectedPredictiveModel = () => useMLAnalyticsStore((state) => state.selectedModel);
export const useSelectedBusinessReport = () => useMLAnalyticsStore((state) => state.selectedBusinessReport);
export const useMLAnalyticsLoading = () => useMLAnalyticsStore((state) => state.loading);
export const useMLAnalyticsErrors = () => useMLAnalyticsStore((state) => state.errors);

// Computed selectors
export const useActiveAnomalyAlerts = () => useMLAnalyticsStore((state) =>
  state.anomalyAlerts.filter(alert => !alert.acknowledged && !alert.resolved)
);

export const useCriticalAnomalyAlerts = () => useMLAnalyticsStore((state) =>
  state.anomalyAlerts.filter(alert =>
    alert.severity === 'critical' && !alert.acknowledged && !alert.resolved
  )
);

export const useReadyModels = () => useMLAnalyticsStore((state) =>
  state.predictiveModels.filter(model => model.status === 'ready')
);

export const useTrainingModels = () => useMLAnalyticsStore((state) =>
  state.predictiveModels.filter(model => model.status === 'training')
);

export const useFailedModels = () => useMLAnalyticsStore((state) =>
  state.predictiveModels.filter(model => model.status === 'failed')
);

export const useAtRiskMetrics = () => useMLAnalyticsStore((state) =>
  state.businessMetrics.filter(metric => metric.status === 'at_risk')
);

export const useOffTrackMetrics = () => useMLAnalyticsStore((state) =>
  state.businessMetrics.filter(metric => metric.status === 'off_track')
);

export const useAchievedMetrics = () => useMLAnalyticsStore((state) =>
  state.businessMetrics.filter(metric => metric.status === 'achieved')
);

export const useRunningExperiments = () => useMLAnalyticsStore((state) =>
  state.mlExperiments.filter(experiment => experiment.status === 'running')
);

export const useFailedExperiments = () => useMLAnalyticsStore((state) =>
  state.mlExperiments.filter(experiment => experiment.status === 'failed')
);

export const useStrongCorrelations = () => useMLAnalyticsStore((state) =>
  state.correlations.filter(correlation => correlation.significance === 'strong')
);

export const useRecentDataQualityReports = () => useMLAnalyticsStore((state) =>
  state.dataQualityReports.slice(0, 5).sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
);

export const useAnomalyAlertStats = () => useMLAnalyticsStore((state) => {
  const alerts = state.anomalyAlerts;
  return {
    total: alerts.length,
    active: alerts.filter(a => !a.acknowledged && !a.resolved).length,
    acknowledged: alerts.filter(a => a.acknowledged && !a.resolved).length,
    resolved: alerts.filter(a => a.resolved).length,
    bySeverity: {
      low: alerts.filter(a => a.severity === 'low').length,
      medium: alerts.filter(a => a.severity === 'medium').length,
      high: alerts.filter(a => a.severity === 'high').length,
      critical: alerts.filter(a => a.severity === 'critical').length,
    },
    recentActivity: alerts.filter(a =>
      new Date(a.timestamp) > new Date(Date.now() - 24 * 60 * 60 * 1000)
    ).length,
  };
});

export const useBusinessMetricsStats = () => useMLAnalyticsStore((state) => {
  const metrics = state.businessMetrics;
  return {
    total: metrics.length,
    onTrack: metrics.filter(m => m.status === 'on_track').length,
    atRisk: metrics.filter(m => m.status === 'at_risk').length,
    offTrack: metrics.filter(m => m.status === 'off_track').length,
    achieved: metrics.filter(m => m.status === 'achieved').length,
    averageChange: metrics.length > 0
      ? metrics.reduce((sum, m) => sum + m.changePercent, 0) / metrics.length
      : 0,
    byCategory: metrics.reduce((acc, metric) => {
      acc[metric.category] = (acc[metric.category] || 0) + 1;
      return acc;
    }, {} as Record<string, number>),
  };
});

export const useModelPerformanceStats = () => useMLAnalyticsStore((state) => {
  const models = state.predictiveModels;
  return {
    total: models.length,
    ready: models.filter(m => m.status === 'ready').length,
    training: models.filter(m => m.status === 'training').length,
    failed: models.filter(m => m.status === 'failed').length,
    deprecated: models.filter(m => m.status === 'deprecated').length,
    averageAccuracy: models.length > 0
      ? models.reduce((sum, m) => sum + m.accuracy, 0) / models.length
      : 0,
    averageF1Score: models.length > 0
      ? models.reduce((sum, m) => sum + m.f1Score, 0) / models.length
      : 0,
    byType: models.reduce((acc, model) => {
      acc[model.type] = (acc[model.type] || 0) + 1;
      return acc;
    }, {} as Record<string, string>),
  };
});

export const useMLAnalyticsActions = () => useMLAnalyticsStore((state) => ({
  setPredictiveModels: state.setPredictiveModels,
  addPredictiveModel: state.addPredictiveModel,
  updatePredictiveModel: state.updatePredictiveModel,
  removePredictiveModel: state.removePredictiveModel,
  setPredictions: state.setPredictions,
  addPrediction: state.addPrediction,
  setAnomalyDetectors: state.setAnomalyDetectors,
  addAnomalyDetector: state.addAnomalyDetector,
  updateAnomalyDetector: state.updateAnomalyDetector,
  setAnomalyAlerts: state.setAnomalyAlerts,
  addAnomalyAlert: state.addAnomalyAlert,
  updateAnomalyAlert: state.updateAnomalyAlert,
  setTrendAnalyses: state.setTrendAnalyses,
  updateTrendAnalysis: state.updateTrendAnalysis,
  setBusinessMetrics: state.setBusinessMetrics,
  updateBusinessMetric: state.updateBusinessMetric,
  setBusinessReports: state.setBusinessReports,
  addBusinessReport: state.addBusinessReport,
  setCorrelations: state.setCorrelations,
  addCorrelation: state.addCorrelation,
  setMLExperiments: state.setMLExperiments,
  addMLExperiment: state.addMLExperiment,
  updateMLExperiment: state.updateMLExperiment,
  setForecastingModels: state.setForecastingModels,
  updateForecastingModel: state.updateForecastingModel,
  setDataQualityReports: state.setDataQualityReports,
  addDataQualityReport: state.addDataQualityReport,
  setRealTimeMetrics: state.setRealTimeMetrics,
  updateRealTimeMetric: state.updateRealTimeMetric,
  setSelectedModel: state.setSelectedModel,
  setSelectedAnomalyDetector: state.setSelectedAnomalyDetector,
  setSelectedBusinessReport: state.setSelectedBusinessReport,
  setSelectedMLExperiment: state.setSelectedMLExperiment,
  addActiveSubscription: state.addActiveSubscription,
  removeActiveSubscription: state.removeActiveSubscription,
  clearActiveSubscriptions: state.clearActiveSubscriptions,
  setLoading: state.setLoading,
  setError: state.setError,
  clearErrors: state.clearErrors,
  setPagination: state.setPagination,
  nextAlertsPage: state.nextAlertsPage,
  nextReportsPage: state.nextReportsPage,
  nextExperimentsPage: state.nextExperimentsPage,
  resetPagination: state.resetPagination,
  setFilters: state.setFilters,
  clearFilters: state.clearFilters,
  updateSettings: state.updateSettings,
  reset: state.reset,
}));
