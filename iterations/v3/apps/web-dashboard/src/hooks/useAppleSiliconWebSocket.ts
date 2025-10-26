/**
 * Apple Silicon WebSocket Hook
 * Real-time updates for Apple Silicon hardware monitoring
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState } from 'react';
import { useAppleSiliconStore, useAppleSiliconActions } from '@/stores/apple-silicon';
import { HardwareMetrics, HardwareAlert, RoutingDecision, ModelMetrics } from '@/lib/apple-silicon-api';

interface AppleSiliconWebSocketMessage {
  type: 'metrics_update' | 'alert_created' | 'routing_decision' | 'model_status_changed' | 'thermal_event' | 'recommendation_generated';
  data: any;
  timestamp: string;
}

export function useAppleSiliconWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 5;
  const reconnectDelay = 1000; // Start with 1 second

  const actions = useAppleSiliconActions();

  const connect = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setConnectionStatus('connecting');

    try {
      const ws = new WebSocket(`${process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080'}/apple-silicon`);

      ws.onopen = () => {
        console.log('Apple Silicon WebSocket connected');
        setIsConnected(true);
        setConnectionStatus('connected');
        reconnectAttempts.current = 0;

        // Send authentication if needed
        ws.send(JSON.stringify({
          type: 'auth',
          token: localStorage.getItem('auth_token')
        }));

        // Subscribe to real-time updates
        ws.send(JSON.stringify({
          type: 'subscribe',
          channels: ['metrics', 'alerts', 'routing', 'models', 'thermal', 'recommendations']
        }));
      };

      ws.onmessage = (event) => {
        try {
          const message: AppleSiliconWebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error('Failed to parse Apple Silicon WebSocket message:', error);
        }
      };

      ws.onclose = (event) => {
        console.log('Apple Silicon WebSocket disconnected:', event.code, event.reason);
        setIsConnected(false);
        setConnectionStatus('disconnected');

        // Attempt to reconnect if not a manual close
        if (event.code !== 1000 && reconnectAttempts.current < maxReconnectAttempts) {
          scheduleReconnect();
        }
      };

      ws.onerror = (error) => {
        console.error('Apple Silicon WebSocket error:', error);
        setConnectionStatus('error');
        setIsConnected(false);
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('Failed to create Apple Silicon WebSocket connection:', error);
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

    console.log(`Scheduling Apple Silicon WebSocket reconnect in ${delay}ms (attempt ${reconnectAttempts.current})`);

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

  const handleMessage = (message: AppleSiliconWebSocketMessage) => {
    const { type, data, timestamp } = message;

    // Update last update timestamp
    actions.setLastUpdate(new Date(timestamp));

    switch (type) {
      case 'metrics_update':
        actions.setCurrentMetrics(data as HardwareMetrics);
        actions.addHistoricalMetrics(data as HardwareMetrics);
        break;

      case 'alert_created':
        actions.addAlert(data as HardwareAlert);
        break;

      case 'routing_decision':
        actions.addRoutingDecision(data as RoutingDecision);
        break;

      case 'model_status_changed':
        actions.updateModel(data.id, data.updates);
        break;

      case 'thermal_event':
        // Thermal events are handled through metrics updates
        console.log('Thermal event:', data);
        break;

      case 'recommendation_generated':
        actions.addRecommendation(data);
        break;

      default:
        console.warn('Unknown Apple Silicon WebSocket message type:', type);
    }
  };

  const sendMessage = (message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('Apple Silicon WebSocket not connected, cannot send message');
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

  // Request current metrics
  const requestMetrics = () => {
    sendMessage({
      type: 'request_metrics'
    });
  };

  // Request active alerts
  const requestAlerts = () => {
    sendMessage({
      type: 'request_alerts'
    });
  };

  // Request routing updates
  const requestRouting = () => {
    sendMessage({
      type: 'request_routing'
    });
  };

  // Request model status
  const requestModels = () => {
    sendMessage({
      type: 'request_models'
    });
  };

  // Request recommendations
  const requestRecommendations = () => {
    sendMessage({
      type: 'request_recommendations'
    });
  };

  // Control thermal management
  const setThermalPolicy = (policy: any) => {
    sendMessage({
      type: 'set_thermal_policy',
      policy
    });
  };

  // Force routing decision
  const forceRouting = (modelId: string, hardware: string) => {
    sendMessage({
      type: 'force_routing',
      modelId,
      hardware
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
    requestMetrics,
    requestAlerts,
    requestRouting,
    requestModels,
    requestRecommendations,
    setThermalPolicy,
    forceRouting,
  };
}

// Hook for real-time hardware monitoring
export function useRealTimeHardwareMonitoring() {
  const currentMetrics = useAppleSiliconStore((state) => state.currentMetrics);
  const loading = useAppleSiliconStore((state) => state.loading.metrics);
  const realTimeEnabled = useAppleSiliconStore((state) => state.realTimeEnabled);

  return {
    currentMetrics,
    loading,
    realTimeEnabled,
    isHealthy: currentMetrics ? !currentMetrics.thermal.thermalThrottling &&
                                currentMetrics.cpu.temperature < 90 &&
                                currentMetrics.gpu.temperature < 90 : false,
    utilization: currentMetrics ? {
      ane: currentMetrics.ane.utilization,
      gpu: currentMetrics.gpu.utilization,
      cpu: currentMetrics.cpu.utilization,
      memory: ((currentMetrics.memory.totalMemory - currentMetrics.memory.availableMemory) /
               currentMetrics.memory.totalMemory) * 100,
    } : null,
    temperatures: currentMetrics ? {
      cpu: currentMetrics.cpu.temperature,
      gpu: currentMetrics.gpu.temperature,
      ane: currentMetrics.ane.temperature,
      ambient: currentMetrics.thermal.ambientTemperature,
    } : null,
  };
}

// Hook for real-time thermal monitoring
export function useRealTimeThermalMonitoring() {
  const currentMetrics = useAppleSiliconStore((state) => state.currentMetrics);
  const alerts = useAppleSiliconStore((state) => state.alerts);

  const thermalAlerts = alerts.filter(alert =>
    alert.type === 'thermal' && !alert.acknowledged
  );

  return {
    thermalMetrics: currentMetrics?.thermal || null,
    thermalAlerts,
    isThrottling: currentMetrics?.thermal.thermalThrottling || false,
    criticalTemperatures: currentMetrics ? {
      cpu: currentMetrics.cpu.temperature > 95,
      gpu: currentMetrics.gpu.temperature > 95,
      ane: currentMetrics.ane.temperature > 95,
    } : null,
    coolingEfficiency: currentMetrics?.thermal.coolingEfficiency || 0,
    thermalMargin: currentMetrics?.thermal.thermalMargin || 0,
  };
}

// Hook for real-time model monitoring
export function useRealTimeModelMonitoring() {
  const activeModels = useAppleSiliconStore((state) => state.activeModels);
  const routingDecisions = useAppleSiliconStore((state) => state.routingDecisions);

  return {
    activeModels,
    routingDecisions,
    modelCount: activeModels.length,
    modelsByHardware: {
      ane: activeModels.filter(m => m.hardware === 'ane').length,
      gpu: activeModels.filter(m => m.hardware === 'gpu').length,
      cpu: activeModels.filter(m => m.hardware === 'cpu').length,
    },
    averagePerformance: activeModels.length > 0 ? {
      latency: activeModels.reduce((sum, m) => sum + m.performance.latency, 0) / activeModels.length,
      throughput: activeModels.reduce((sum, m) => sum + m.performance.throughput, 0) / activeModels.length,
      accuracy: activeModels.reduce((sum, m) => sum + m.performance.accuracy, 0) / activeModels.length,
    } : null,
    recentRoutingDecisions: routingDecisions.slice(0, 10),
  };
}

// Hook for real-time power monitoring
export function useRealTimePowerMonitoring() {
  const currentMetrics = useAppleSiliconStore((state) => state.currentMetrics);

  return {
    powerMetrics: currentMetrics?.power || null,
    totalConsumption: currentMetrics?.power.totalConsumption || 0,
    breakdown: currentMetrics?.power ? {
      cpu: currentMetrics.power.cpuConsumption,
      gpu: currentMetrics.power.gpuConsumption,
      ane: currentMetrics.power.aneConsumption,
      other: currentMetrics.power.totalConsumption -
             currentMetrics.power.cpuConsumption -
             currentMetrics.power.gpuConsumption -
             currentMetrics.power.aneConsumption,
    } : null,
    efficiency: currentMetrics?.power.totalConsumption && currentMetrics.power.totalConsumption > 0
      ? (currentMetrics.power.thermalDesignPower / currentMetrics.power.totalConsumption) * 100
      : 0,
    batteryLevel: currentMetrics?.power.batteryLevel,
    charging: currentMetrics?.power.charging,
  };
}