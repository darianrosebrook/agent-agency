/**
 * ML Analytics WebSocket Hook
 * Real-time updates for predictive analytics, anomaly detection, and business intelligence
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState } from 'react';
import { useMLAnalyticsStore, useMLAnalyticsActions } from '@/stores/ml-analytics';
import { AnomalyAlert, PredictionResult, BusinessMetric } from '@/lib/ml-analytics-api';

interface MLAnalyticsWebSocketMessage {
  type: 'anomaly_alert' | 'prediction_update' | 'business_metric_update' | 'experiment_update' | 'realtime_metric_update' | 'model_status_update';
  data: any;
  timestamp: string;
}

export function useMLAnalyticsWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 5;
  const reconnectDelay = 1000; // Start with 1 second

  const actions = useMLAnalyticsActions();

  const connect = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setConnectionStatus('connecting');

    try {
      const ws = new WebSocket(`${process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080'}/ml-analytics`);

      ws.onopen = () => {
        console.log('ML Analytics WebSocket connected');
        setIsConnected(true);
        setConnectionStatus('connected');
        reconnectAttempts.current = 0;

        // Send authentication if needed
        ws.send(JSON.stringify({
          type: 'auth',
          token: localStorage.getItem('auth_token')
        }));

        // Subscribe to real-time ML analytics updates
        ws.send(JSON.stringify({
          type: 'subscribe',
          channels: ['anomalies', 'predictions', 'business', 'experiments', 'realtime']
        }));
      };

      ws.onmessage = (event) => {
        try {
          const message: MLAnalyticsWebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error('Failed to parse ML Analytics WebSocket message:', error);
        }
      };

      ws.onclose = (event) => {
        console.log('ML Analytics WebSocket disconnected:', event.code, event.reason);
        setIsConnected(false);
        setConnectionStatus('disconnected');

        // Attempt to reconnect if not a manual close
        if (event.code !== 1000 && reconnectAttempts.current < maxReconnectAttempts) {
          scheduleReconnect();
        }
      };

      ws.onerror = (error) => {
        console.error('ML Analytics WebSocket error:', error);
        setConnectionStatus('error');
        setIsConnected(false);
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('Failed to create ML Analytics WebSocket connection:', error);
      setConnectionStatus('error');
      scheduleReconnect();
    }
  };

  const scheduleReconnect = () => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }

    const delay = reconnectDelay * Math.pow(2, reconnectAttempts.current);
    reconnectAttempts.current++;

    console.log(`Scheduling ML Analytics WebSocket reconnect in ${delay}ms (attempt ${reconnectAttempts.current})`);

    reconnectTimeoutRef.current = setTimeout(() => {
      connect();
    }, delay);
  };

  const disconnect = () => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close(1000, 'Manual disconnect');
      wsRef.current = null;
    }

    setIsConnected(false);
    setConnectionStatus('disconnected');
  };

  const handleMessage = (message: MLAnalyticsWebSocketMessage) => {
    const { type, data, timestamp } = message;

    switch (type) {
      case 'anomaly_alert':
        actions.addAnomalyAlert(data as AnomalyAlert);
        break;

      case 'prediction_update':
        const prediction = data as PredictionResult;
        actions.addPrediction(prediction.modelId, prediction);
        break;

      case 'business_metric_update':
        actions.updateBusinessMetric(data.id, data.updates);
        break;

      case 'experiment_update':
        actions.updateMLExperiment(data.id, data.updates);
        break;

      case 'realtime_metric_update':
        // Handle real-time metric updates
        Object.entries(data).forEach(([metric, dataPoint]) => {
          actions.updateRealTimeMetric(metric, dataPoint as { timestamp: Date; value: number });
        });
        break;

      case 'model_status_update':
        actions.updatePredictiveModel(data.id, data.updates);
        break;

      default:
        console.warn('Unknown ML Analytics WebSocket message type:', type);
    }
  };

  const sendMessage = (message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('ML Analytics WebSocket not connected, cannot send message');
    }
  };

  // Subscribe to specific channels
  const subscribe = (channels: string[]) => {
    sendMessage({
      type: 'subscribe',
      channels
    });
  };

  // Unsubscribe from channels
  const unsubscribe = (channels: string[]) => {
    sendMessage({
      type: 'unsubscribe',
      channels
    });
  };

  // Subscribe to real-time metrics
  const subscribeToMetrics = (metrics: string[]) => {
    sendMessage({
      type: 'subscribe_metrics',
      metrics
    });

    metrics.forEach(metric => {
      actions.addActiveSubscription(metric);
    });
  };

  // Unsubscribe from real-time metrics
  const unsubscribeFromMetrics = (metrics: string[]) => {
    sendMessage({
      type: 'unsubscribe_metrics',
      metrics
    });

    metrics.forEach(metric => {
      actions.removeActiveSubscription(metric);
    });
  };

  // Request current anomaly alerts
  const requestAnomalyAlerts = () => {
    sendMessage({
      type: 'request_anomaly_alerts'
    });
  };

  // Request current business metrics
  const requestBusinessMetrics = () => {
    sendMessage({
      type: 'request_business_metrics'
    });
  };

  // Request current experiments status
  const requestExperiments = () => {
    sendMessage({
      type: 'request_experiments'
    });
  };

  // Request model status updates
  const requestModelStatus = () => {
    sendMessage({
      type: 'request_model_status'
    });
  };

  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, []);

  return {
    isConnected,
    connectionStatus,
    connect,
    disconnect,
    sendMessage,
    subscribe,
    unsubscribe,
    subscribeToMetrics,
    unsubscribeFromMetrics,
    requestAnomalyAlerts,
    requestBusinessMetrics,
    requestExperiments,
    requestModelStatus,
  };
}

// Hook for real-time anomaly monitoring
export function useRealTimeAnomalyMonitoring() {
  const anomalyAlerts = useMLAnalyticsStore((state) => state.anomalyAlerts);
  const activeSubscriptions = useMLAnalyticsStore((state) => state.activeSubscriptions);
  const loading = useMLAnalyticsStore((state) => state.loading.anomalies);

  return {
    anomalyAlerts,
    activeSubscriptions,
    loading,
    activeAlerts: anomalyAlerts.filter(alert => !alert.acknowledged && !alert.resolved),
    criticalAlerts: anomalyAlerts.filter(alert =>
      alert.severity === 'critical' && !alert.acknowledged && !alert.resolved
    ),
    recentAlerts: anomalyAlerts.slice(0, 10),
    alertCountBySeverity: {
      low: anomalyAlerts.filter(a => a.severity === 'low').length,
      medium: anomalyAlerts.filter(a => a.severity === 'medium').length,
      high: anomalyAlerts.filter(a => a.severity === 'high').length,
      critical: anomalyAlerts.filter(a => a.severity === 'critical').length,
    },
    alertCountByStatus: {
      active: anomalyAlerts.filter(a => !a.acknowledged && !a.resolved).length,
      acknowledged: anomalyAlerts.filter(a => a.acknowledged && !a.resolved).length,
      resolved: anomalyAlerts.filter(a => a.resolved).length,
    },
    alertTrends: {
      lastHour: anomalyAlerts.filter(a => new Date(a.timestamp) > new Date(Date.now() - 60 * 60 * 1000)).length,
      last24Hours: anomalyAlerts.filter(a => new Date(a.timestamp) > new Date(Date.now() - 24 * 60 * 60 * 1000)).length,
      last7Days: anomalyAlerts.filter(a => new Date(a.timestamp) > new Date(Date.now() - 7 * 24 * 60 * 60 * 1000)).length,
    },
  };
}

// Hook for real-time business metrics monitoring
export function useRealTimeBusinessMonitoring() {
  const businessMetrics = useMLAnalyticsStore((state) => state.businessMetrics);
  const loading = useMLAnalyticsStore((state) => state.loading.business);

  return {
    businessMetrics,
    loading,
    atRiskMetrics: businessMetrics.filter(metric => metric.status === 'at_risk'),
    offTrackMetrics: businessMetrics.filter(metric => metric.status === 'off_track'),
    achievedMetrics: businessMetrics.filter(metric => metric.status === 'achieved'),
    onTrackMetrics: businessMetrics.filter(metric => metric.status === 'on_track'),
    metricsByCategory: businessMetrics.reduce((acc, metric) => {
      acc[metric.category] = (acc[metric.category] || []).concat(metric);
      return acc;
    }, {} as Record<string, typeof businessMetrics>),
    averagePerformance: businessMetrics.length > 0
      ? businessMetrics.reduce((sum, metric) => {
          const score = metric.target ? (metric.value / metric.target) : 1;
          return sum + score;
        }, 0) / businessMetrics.length
      : 0,
    trendingUp: businessMetrics.filter(metric => metric.trend === 'up'),
    trendingDown: businessMetrics.filter(metric => metric.trend === 'down'),
    overallChange: businessMetrics.length > 0
      ? businessMetrics.reduce((sum, metric) => sum + metric.changePercent, 0) / businessMetrics.length
      : 0,
  };
}

// Hook for real-time model performance monitoring
export function useRealTimeModelMonitoring() {
  const predictiveModels = useMLAnalyticsStore((state) => state.predictiveModels);
  const mlExperiments = useMLAnalyticsStore((state) => state.mlExperiments);
  const loading = useMLAnalyticsStore((state) => state.loading.models);

  return {
    predictiveModels,
    mlExperiments,
    loading,
    readyModels: predictiveModels.filter(model => model.status === 'ready'),
    trainingModels: predictiveModels.filter(model => model.status === 'training'),
    failedModels: predictiveModels.filter(model => model.status === 'failed'),
    runningExperiments: mlExperiments.filter(experiment => experiment.status === 'running'),
    failedExperiments: mlExperiments.filter(experiment => experiment.status === 'failed'),
    completedExperiments: mlExperiments.filter(experiment => experiment.status === 'completed'),
    modelStats: {
      total: predictiveModels.length,
      ready: predictiveModels.filter(m => m.status === 'ready').length,
      training: predictiveModels.filter(m => m.status === 'training').length,
      failed: predictiveModels.filter(m => m.status === 'failed').length,
      deprecated: predictiveModels.filter(m => m.status === 'deprecated').length,
      averageAccuracy: predictiveModels.length > 0
        ? predictiveModels.reduce((sum, m) => sum + m.accuracy, 0) / predictiveModels.length
        : 0,
    },
    experimentStats: {
      total: mlExperiments.length,
      running: mlExperiments.filter(e => e.status === 'running').length,
      completed: mlExperiments.filter(e => e.status === 'completed').length,
      failed: mlExperiments.filter(e => e.status === 'failed').length,
      pending: mlExperiments.filter(e => e.status === 'pending').length,
    },
    recentPredictions: predictiveModels.reduce((acc, model) => {
      const predictions = useMLAnalyticsStore.getState().predictions[model.id] || [];
      return acc.concat(predictions.slice(0, 5));
    }, [] as PredictionResult[]).sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime()).slice(0, 10),
  };
}

// Hook for real-time trend analysis monitoring
export function useRealTimeTrendMonitoring() {
  const trendAnalyses = useMLAnalyticsStore((state) => state.trendAnalyses);
  const correlations = useMLAnalyticsStore((state) => state.correlations);
  const loading = useMLAnalyticsStore((state) => state.loading.trends);

  return {
    trendAnalyses,
    correlations,
    loading,
    increasingTrends: Object.values(trendAnalyses).filter(trend => trend.trend === 'increasing'),
    decreasingTrends: Object.values(trendAnalyses).filter(trend => trend.trend === 'decreasing'),
    volatileTrends: Object.values(trendAnalyses).filter(trend => trend.trend === 'volatile'),
    strongCorrelations: correlations.filter(correlation => correlation.significance === 'strong'),
    positiveCorrelations: correlations.filter(correlation => correlation.direction === 'positive'),
    negativeCorrelations: correlations.filter(correlation => correlation.direction === 'negative'),
    trendStats: {
      total: Object.keys(trendAnalyses).length,
      increasing: Object.values(trendAnalyses).filter(t => t.trend === 'increasing').length,
      decreasing: Object.values(trendAnalyses).filter(t => t.trend === 'decreasing').length,
      stable: Object.values(trendAnalyses).filter(t => t.trend === 'stable').length,
      volatile: Object.values(trendAnalyses).filter(t => t.trend === 'volatile').length,
      withSeasonality: Object.values(trendAnalyses).filter(t => t.seasonality).length,
    },
    correlationStats: {
      total: correlations.length,
      strong: correlations.filter(c => c.significance === 'strong').length,
      moderate: correlations.filter(c => c.significance === 'moderate').length,
      weak: correlations.filter(c => c.significance === 'weak').length,
      positive: correlations.filter(c => c.direction === 'positive').length,
      negative: correlations.filter(c => c.direction === 'negative').length,
    },
  };
}

// Hook for real-time forecasting monitoring
export function useRealTimeForecastingMonitoring() {
  const forecastingModels = useMLAnalyticsStore((state) => state.forecastingModels);
  const loading = useMLAnalyticsStore((state) => state.loading.forecasting);

  return {
    forecastingModels,
    loading,
    activeForecasts: forecastingModels.filter(model => model.forecast.length > 0),
    highAccuracyForecasts: forecastingModels.filter(model => model.accuracy.mape < 10),
    lowAccuracyForecasts: forecastingModels.filter(model => model.accuracy.mape > 20),
    forecastStats: {
      total: forecastingModels.length,
      active: forecastingModels.filter(m => m.forecast.length > 0).length,
      averageMAPE: forecastingModels.length > 0
        ? forecastingModels.reduce((sum, m) => sum + m.accuracy.mape, 0) / forecastingModels.length
        : 0,
      averageRMSE: forecastingModels.length > 0
        ? forecastingModels.reduce((sum, m) => sum + m.accuracy.rmse, 0) / forecastingModels.length
        : 0,
      byAlgorithm: forecastingModels.reduce((acc, model) => {
        acc[model.algorithm] = (acc[model.algorithm] || 0) + 1;
        return acc;
      }, {} as Record<string, number>),
      byFrequency: forecastingModels.reduce((acc, model) => {
        acc[model.frequency] = (acc[model.frequency] || 0) + 1;
        return acc;
      }, {} as Record<string, number>),
    },
    recentForecasts: forecastingModels
      .filter(model => model.forecast.length > 0)
      .sort((a, b) => b.lastUpdated.getTime() - a.lastUpdated.getTime())
      .slice(0, 5),
  };
}

// Hook for real-time data quality monitoring
export function useRealTimeDataQualityMonitoring() {
  const dataQualityReports = useMLAnalyticsStore((state) => state.dataQualityReports);
  const loading = useMLAnalyticsStore((state) => state.loading.dataQuality);

  return {
    dataQualityReports,
    loading,
    recentReports: dataQualityReports.slice(0, 5).sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime()),
    averageQualityScores: {
      completeness: dataQualityReports.length > 0
        ? dataQualityReports.reduce((sum, r) => sum + r.metrics.completeness, 0) / dataQualityReports.length
        : 0,
      accuracy: dataQualityReports.length > 0
        ? dataQualityReports.reduce((sum, r) => sum + r.metrics.accuracy, 0) / dataQualityReports.length
        : 0,
      consistency: dataQualityReports.length > 0
        ? dataQualityReports.reduce((sum, r) => sum + r.metrics.consistency, 0) / dataQualityReports.length
        : 0,
      timeliness: dataQualityReports.length > 0
        ? dataQualityReports.reduce((sum, r) => sum + r.metrics.timeliness, 0) / dataQualityReports.length
        : 0,
      validity: dataQualityReports.length > 0
        ? dataQualityReports.reduce((sum, r) => sum + r.metrics.validity, 0) / dataQualityReports.length
        : 0,
      uniqueness: dataQualityReports.length > 0
        ? dataQualityReports.reduce((sum, r) => sum + r.metrics.uniqueness, 0) / dataQualityReports.length
        : 0,
    },
    datasetsWithIssues: dataQualityReports.filter(report =>
      report.issues.some(issue => issue.severity !== 'low')
    ),
    criticalIssues: dataQualityReports.flatMap(report =>
      report.issues.filter(issue => issue.severity === 'high' || issue.severity === 'critical')
    ),
    issueStats: {
      totalIssues: dataQualityReports.reduce((sum, report) => sum + report.issues.length, 0),
      byType: dataQualityReports.reduce((acc, report) => {
        report.issues.forEach(issue => {
          acc[issue.type] = (acc[issue.type] || 0) + 1;
        });
        return acc;
      }, {} as Record<string, number>),
      bySeverity: dataQualityReports.reduce((acc, report) => {
        report.issues.forEach(issue => {
          acc[issue.severity] = (acc[issue.severity] || 0) + 1;
        });
        return acc;
      }, {} as Record<string, number>),
    },
  };
}
