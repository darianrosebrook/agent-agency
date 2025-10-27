/**
 * Apple Silicon API Client
 * API client for Apple Silicon hardware monitoring and control
 *
 * @author @darianrosebrook
 */

import { ApiClient } from './api-client';

export interface ANEMetrics {
  utilization: number; // 0-100%
  activeModels: number;
  inferenceQueue: number;
  temperature: number; // Celsius
  throttling: boolean;
  powerConsumption: number; // Watts
  efficiency: number; // inferences/second/watt
}

export interface GPUMetrics {
  utilization: number; // 0-100%
  memoryUsage: number; // GB
  memoryBandwidth: number; // GB/s
  temperature: number; // Celsius
  activeComputeTasks: number;
  powerConsumption: number; // Watts
  frequency: number; // MHz
}

export interface CPUMetrics {
  utilization: number; // 0-100%
  coreCount: number;
  activeCores: number;
  temperature: number; // Celsius
  frequency: number; // MHz
  powerConsumption: number; // Watts
  thermalThrottling: boolean;
}

export interface MemoryMetrics {
  totalMemory: number; // GB
  usedMemory: number; // GB
  availableMemory: number; // GB
  bandwidth: number; // GB/s
  efficiency: number; // 0-100%
  fragmentation: number; // 0-100%
}

export interface PowerMetrics {
  totalConsumption: number; // Watts
  cpuConsumption: number; // Watts
  gpuConsumption: number; // Watts
  aneConsumption: number; // Watts
  batteryLevel?: number; // 0-100%
  charging: boolean;
  thermalDesignPower: number; // TDP in Watts
}

export interface ThermalMetrics {
  cpuTemperature: number; // Celsius
  gpuTemperature: number; // Celsius
  aneTemperature: number; // Celsius
  ambientTemperature: number; // Celsius
  fanSpeed?: number; // RPM
  coolingEfficiency: number; // 0-100%
  thermalThrottling: boolean;
  thermalMargin: number; // Degrees below max temp
}

export interface ModelMetrics {
  id: string;
  name: string;
  hardware: 'ane' | 'gpu' | 'cpu' | 'fallback';
  status: 'loading' | 'active' | 'inactive' | 'error';
  performance: {
    latency: number; // milliseconds
    throughput: number; // inferences/second
    accuracy: number; // 0-100%
    memoryUsage: number; // MB
  };
  utilization: number; // 0-100%
  lastInference: Date;
  loadTime: number; // milliseconds
}

export interface HardwareAlert {
  id: string;
  type: 'thermal' | 'performance' | 'power' | 'memory' | 'hardware_failure';
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
  component: 'ane' | 'gpu' | 'cpu' | 'memory' | 'power' | 'thermal';
  value: number;
  threshold: number;
  timestamp: Date;
  resolved: boolean;
}

export interface RoutingDecision {
  id: string;
  modelId: string;
  modelName: string;
  requestedHardware: 'ane' | 'gpu' | 'cpu' | 'auto';
  assignedHardware: 'ane' | 'gpu' | 'cpu';
  reason: string;
  performance: {
    expectedLatency: number;
    expectedThroughput: number;
    powerEfficiency: number;
  };
  timestamp: Date;
}

export interface OptimizationRecommendation {
  id: string;
  type: 'thermal' | 'performance' | 'power' | 'routing' | 'memory';
  priority: 'low' | 'medium' | 'high' | 'critical';
  title: string;
  description: string;
  impact: {
    performance: number; // percentage improvement
    power: number; // percentage reduction
    thermal: number; // temperature reduction in Celsius
  };
  implementation: {
    automatic: boolean;
    estimatedTime: number; // seconds
    risk: 'low' | 'medium' | 'high';
  };
  timestamp: Date;
}

export interface HardwareMetrics {
  ane: ANEMetrics;
  gpu: GPUMetrics;
  cpu: CPUMetrics;
  memory: MemoryMetrics;
  power: PowerMetrics;
  thermal: ThermalMetrics;
  timestamp: Date;
}

export interface AppleSiliconStatus {
  deviceInfo: {
    model: string;
    chip: string;
    memory: number; // GB
    macosVersion: string;
    supportedHardware: ('ane' | 'gpu' | 'cpu')[];
  };
  overallHealth: 'healthy' | 'warning' | 'critical';
  activeModels: number;
  totalUtilization: number;
  thermalStatus: 'optimal' | 'warning' | 'critical';
  powerEfficiency: number;
}

export class AppleSiliconApiClient {
  private apiClient: ApiClient;

  constructor(baseUrl: string = '/api/apple-silicon') {
    this.apiClient = new ApiClient({ baseUrl });
  }

  /**
   * Get current hardware metrics
   */
  async getCurrentMetrics(): Promise<HardwareMetrics> {
    const response = await this.apiClient.request<HardwareMetrics>('/metrics/current');
    return response;
  }

  /**
   * Get historical hardware metrics
   */
  async getHistoricalMetrics(
    period: '1h' | '6h' | '24h' | '7d' = '1h',
    interval: '1s' | '10s' | '1m' | '5m' = '10s'
  ): Promise<HardwareMetrics[]> {
    const params = new URLSearchParams({
      period,
      interval
    });

    const response = await this.apiClient.request<HardwareMetrics[]>(`/metrics/history?${params}`);
    return response;
  }

  /**
   * Get device status and capabilities
   */
  async getDeviceStatus(): Promise<AppleSiliconStatus> {
    const response = await this.apiClient.request<AppleSiliconStatus>('/status');
    return response;
  }

  /**
   * Get active models and their performance
   */
  async getActiveModels(): Promise<ModelMetrics[]> {
    const response = await this.apiClient.request<ModelMetrics[]>('/models/active');
    return response;
  }

  /**
   * Get model performance comparison
   */
  async getModelPerformance(
    modelId?: string,
    period: '1h' | '24h' | '7d' = '24h'
  ): Promise<ModelMetrics[]> {
    const params = new URLSearchParams({ period });
    if (modelId) params.append('modelId', modelId);

    const response = await this.apiClient.request<ModelMetrics[]>(`/models/performance?${params}`);
    return response;
  }

  /**
   * Get routing decisions and performance
   */
  async getRoutingDecisions(
    limit: number = 50,
    period: '1h' | '24h' | '7d' = '1h'
  ): Promise<RoutingDecision[]> {
    const params = new URLSearchParams({
      limit: limit.toString(),
      period
    });

    const response = await this.apiClient.request<RoutingDecision[]>(`/routing/decisions?${params}`);
    return response;
  }

  /**
   * Get active alerts
   */
  async getAlerts(
    acknowledged: boolean = false,
    severity?: 'low' | 'medium' | 'high' | 'critical',
    limit: number = 50
  ): Promise<HardwareAlert[]> {
    const params = new URLSearchParams({
      acknowledged: acknowledged.toString(),
      limit: limit.toString()
    });

    if (severity) params.append('severity', severity);

    const response = await this.apiClient.request<HardwareAlert[]>(`/alerts?${params}`);
    return response;
  }

  /**
   * Acknowledge hardware alert
   */
  async acknowledgeAlert(alertId: string): Promise<void> {
    await this.apiClient.request<void>(`/alerts/${alertId}/acknowledge`, {
      method: 'POST'
    });
  }

  /**
   * Get optimization recommendations
   */
  async getRecommendations(
    type?: 'thermal' | 'performance' | 'power' | 'routing' | 'memory',
    priority?: 'low' | 'medium' | 'high' | 'critical'
  ): Promise<OptimizationRecommendation[]> {
    const params = new URLSearchParams();
    if (type) params.append('type', type);
    if (priority) params.append('priority', priority);

    const response = await this.apiClient.request<OptimizationRecommendation[]>(
      `/recommendations?${params}`
    );
    return response;
  }

  /**
   * Apply optimization recommendation
   */
  async applyRecommendation(recommendationId: string): Promise<{
    success: boolean;
    message: string;
    appliedChanges: Record<string, any>;
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      message: string;
      appliedChanges: Record<string, any>;
    }>(`/recommendations/${recommendationId}/apply`, {
      method: 'POST'
    });
    return response;
  }

  /**
   * Get thermal status and history
   */
  async getThermalStatus(): Promise<ThermalMetrics & { history: ThermalMetrics[] }> {
    const response = await this.apiClient.request<ThermalMetrics & { history: ThermalMetrics[] }>(
      '/thermal/status'
    );
    return response;
  }

  /**
   * Adjust thermal policy
   */
  async setThermalPolicy(policy: {
    targetTemperature?: number;
    maxUtilization?: number;
    coolingPriority?: 'performance' | 'efficiency' | 'quiet';
    throttlingThreshold?: number;
  }): Promise<{
    success: boolean;
    message: string;
    appliedPolicy: typeof policy;
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      message: string;
      appliedPolicy: typeof policy;
    }>('/thermal/policy', {
      method: 'POST',
      body: JSON.stringify(policy)
    });
    return response;
  }

  /**
   * Get power consumption analysis
   */
  async getPowerAnalysis(
    period: '1h' | '24h' | '7d' = '24h'
  ): Promise<{
    totalConsumption: number;
    efficiency: number;
    breakdown: {
      cpu: number;
      gpu: number;
      ane: number;
      other: number;
    };
    trends: {
      timestamp: Date;
      consumption: number;
      efficiency: number;
    }[];
  }> {
    const params = new URLSearchParams({ period });

    const response = await this.apiClient.request<{
      totalConsumption: number;
      efficiency: number;
      breakdown: {
        cpu: number;
        gpu: number;
        ane: number;
        other: number;
      };
      trends: {
        timestamp: Date;
        consumption: number;
        efficiency: number;
      }[];
    }>(`/power/analysis?${params}`);
    return response;
  }

  /**
   * Force model routing decision
   */
  async forceModelRouting(
    modelId: string,
    targetHardware: 'ane' | 'gpu' | 'cpu'
  ): Promise<{
    success: boolean;
    message: string;
    routingDecision: RoutingDecision;
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      message: string;
      routingDecision: RoutingDecision;
    }>('/routing/force', {
      method: 'POST',
      body: JSON.stringify({ modelId, targetHardware })
    });
    return response;
  }

  /**
   * Get performance benchmarks
   */
  async getBenchmarks(
    hardware?: 'ane' | 'gpu' | 'cpu',
    modelType?: string
  ): Promise<{
    benchmarks: {
      hardware: string;
      modelType: string;
      latency: number;
      throughput: number;
      accuracy: number;
      powerEfficiency: number;
    }[];
    recommendations: {
      bestHardware: string;
      expectedImprovement: number;
      tradeoffs: string[];
    };
  }> {
    const params = new URLSearchParams();
    if (hardware) params.append('hardware', hardware);
    if (modelType) params.append('modelType', modelType);

    const response = await this.apiClient.request<{
      benchmarks: {
        hardware: string;
        modelType: string;
        latency: number;
        throughput: number;
        accuracy: number;
        powerEfficiency: number;
      }[];
      recommendations: {
        bestHardware: string;
        expectedImprovement: number;
        tradeoffs: string[];
      };
    }>(`/benchmarks?${params}`);
    return response;
  }

  /**
   * Reset hardware to safe state
   */
  async emergencyReset(): Promise<{
    success: boolean;
    message: string;
    resetActions: string[];
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      message: string;
      resetActions: string[];
    }>('/emergency/reset', {
      method: 'POST'
    });
    return response;
  }

  /**
   * Export hardware metrics
   */
  async exportMetrics(
    format: 'json' | 'csv' | 'pdf' = 'json',
    period: '1h' | '24h' | '7d' = '24h',
    components?: ('ane' | 'gpu' | 'cpu' | 'memory' | 'thermal' | 'power')[]
  ): Promise<Blob> {
    const params = new URLSearchParams({
      format,
      period
    });

    if (components) params.append('components', components.join(','));

    const response = await fetch(`${this.apiClient['config'].baseUrl}/export?${params}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${this.apiClient['config'].authToken}`
      }
    });

    if (!response.ok) {
      throw new Error(`Export failed: ${response.statusText}`);
    }

    return response.blob();
  }
}

// Export singleton instance
export const appleSiliconApiClient = new AppleSiliconApiClient();