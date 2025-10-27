/**
 * Apple Silicon Store
 * Zustand store for Apple Silicon hardware monitoring state management
 *
 * @author @darianrosebrook
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import {
  HardwareMetrics,
  ModelMetrics,
  HardwareAlert,
  RoutingDecision,
  OptimizationRecommendation,
  AppleSiliconStatus
} from '@/lib/apple-silicon-api';

interface AppleSiliconState {
  // Core data
  currentMetrics: HardwareMetrics | null;
  historicalMetrics: HardwareMetrics[];
  deviceStatus: AppleSiliconStatus | null;
  activeModels: ModelMetrics[];
  alerts: HardwareAlert[];
  routingDecisions: RoutingDecision[];
  recommendations: OptimizationRecommendation[];

  // UI state
  selectedTimeRange: '1h' | '6h' | '24h' | '7d';
  selectedComponents: ('ane' | 'gpu' | 'cpu' | 'memory' | 'thermal' | 'power')[];
  realTimeEnabled: boolean;
  lastUpdate: Date | null;

  // Loading states
  loading: {
    metrics: boolean;
    history: boolean;
    models: boolean;
    alerts: boolean;
    routing: boolean;
    recommendations: boolean;
  };

  // Error states
  errors: {
    metrics: string | null;
    history: string | null;
    models: string | null;
    alerts: string | null;
    routing: string | null;
    recommendations: string | null;
  };

  // Pagination and filtering
  pagination: {
    alertsPage: number;
    routingPage: number;
    limit: number;
  };

  filters: {
    alertSeverity: ('low' | 'medium' | 'high' | 'critical')[] | null;
    recommendationType: ('thermal' | 'performance' | 'power' | 'routing' | 'memory')[] | null;
    modelHardware: ('ane' | 'gpu' | 'cpu')[] | null;
  };
}

interface AppleSiliconActions {
  // Core data actions
  setCurrentMetrics: (metrics: HardwareMetrics) => void;
  setHistoricalMetrics: (metrics: HardwareMetrics[]) => void;
  addHistoricalMetrics: (metrics: HardwareMetrics) => void;
  setDeviceStatus: (status: AppleSiliconStatus) => void;
  setActiveModels: (models: ModelMetrics[]) => void;
  updateModel: (modelId: string, updates: Partial<ModelMetrics>) => void;
  setAlerts: (alerts: HardwareAlert[]) => void;
  addAlert: (alert: HardwareAlert) => void;
  acknowledgeAlert: (alertId: string) => void;
  setRoutingDecisions: (decisions: RoutingDecision[]) => void;
  addRoutingDecision: (decision: RoutingDecision) => void;
  setRecommendations: (recommendations: OptimizationRecommendation[]) => void;
  addRecommendation: (recommendation: OptimizationRecommendation) => void;

  // UI state actions
  setTimeRange: (range: AppleSiliconState['selectedTimeRange']) => void;
  setSelectedComponents: (components: AppleSiliconState['selectedComponents']) => void;
  toggleComponent: (component: AppleSiliconState['selectedComponents'][0]) => void;
  setRealTimeEnabled: (enabled: boolean) => void;
  setLastUpdate: (timestamp: Date) => void;

  // Loading actions
  setLoading: (key: keyof AppleSiliconState['loading'], loading: boolean) => void;
  setError: (key: keyof AppleSiliconState['errors'], error: string | null) => void;
  clearErrors: () => void;

  // Pagination actions
  setPagination: (pagination: Partial<AppleSiliconState['pagination']>) => void;
  nextAlertsPage: () => void;
  nextRoutingPage: () => void;
  resetPagination: () => void;

  // Filter actions
  setFilters: (filters: Partial<AppleSiliconState['filters']>) => void;
  clearFilters: () => void;

  // Utility actions
  reset: () => void;
}

const initialState: AppleSiliconState = {
  currentMetrics: null,
  historicalMetrics: [],
  deviceStatus: null,
  activeModels: [],
  alerts: [],
  routingDecisions: [],
  recommendations: [],
  selectedTimeRange: '1h',
  selectedComponents: ['ane', 'gpu', 'cpu', 'memory', 'thermal', 'power'],
  realTimeEnabled: true,
  lastUpdate: null,
  loading: {
    metrics: false,
    history: false,
    models: false,
    alerts: false,
    routing: false,
    recommendations: false,
  },
  errors: {
    metrics: null,
    history: null,
    models: null,
    alerts: null,
    routing: null,
    recommendations: null,
  },
  pagination: {
    alertsPage: 1,
    routingPage: 1,
    limit: 50,
  },
  filters: {
    alertSeverity: null,
    recommendationType: null,
    modelHardware: null,
  },
};

export const useAppleSiliconStore = create<AppleSiliconState & AppleSiliconActions>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // Core data actions
      setCurrentMetrics: (metrics) => set({ currentMetrics: metrics }),
      setHistoricalMetrics: (metrics) => set({ historicalMetrics: metrics }),
      addHistoricalMetrics: (metrics) => set((state) => ({
        historicalMetrics: [...state.historicalMetrics.slice(-99), metrics] // Keep last 100 entries
      })),
      setDeviceStatus: (status) => set({ deviceStatus: status }),
      setActiveModels: (models) => set({ activeModels: models }),
      updateModel: (modelId, updates) => set((state) => ({
        activeModels: state.activeModels.map(model =>
          model.id === modelId ? { ...model, ...updates } : model
        )
      })),
      setAlerts: (alerts) => set({ alerts }),
      addAlert: (alert) => set((state) => ({
        alerts: [alert, ...state.alerts]
      })),
      acknowledgeAlert: (alertId) => set((state) => ({
        alerts: state.alerts.map(alert =>
          alert.id === alertId ? { ...alert, acknowledged: true } : alert
        )
      })),
      setRoutingDecisions: (decisions) => set({ routingDecisions: decisions }),
      addRoutingDecision: (decision) => set((state) => ({
        routingDecisions: [decision, ...state.routingDecisions.slice(0, 99)] // Keep last 100
      })),
      setRecommendations: (recommendations) => set({ recommendations }),
      addRecommendation: (recommendation) => set((state) => ({
        recommendations: [recommendation, ...state.recommendations]
      })),

      // UI state actions
      setTimeRange: (range) => set({ selectedTimeRange: range }),
      setSelectedComponents: (components) => set({ selectedComponents: components }),
      toggleComponent: (component) => set((state) => ({
        selectedComponents: state.selectedComponents.includes(component)
          ? state.selectedComponents.filter(c => c !== component)
          : [...state.selectedComponents, component]
      })),
      setRealTimeEnabled: (enabled) => set({ realTimeEnabled: enabled }),
      setLastUpdate: (timestamp) => set({ lastUpdate: timestamp }),

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
      nextRoutingPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          routingPage: state.pagination.routingPage + 1
        }
      })),
      resetPagination: () => set({ pagination: initialState.pagination }),

      // Filter actions
      setFilters: (filters) => set((state) => ({
        filters: { ...state.filters, ...filters }
      })),
      clearFilters: () => set({ filters: initialState.filters }),

      // Utility actions
      reset: () => set(initialState),
    }),
    {
      name: 'apple-silicon-store',
    }
  )
);

// Selector hooks for better performance
export const useCurrentMetrics = () => useAppleSiliconStore((state) => state.currentMetrics);
export const useHistoricalMetrics = () => useAppleSiliconStore((state) => state.historicalMetrics);
export const useDeviceStatus = () => useAppleSiliconStore((state) => state.deviceStatus);
export const useActiveModels = () => useAppleSiliconStore((state) => state.activeModels);
export const useHardwareAlerts = () => useAppleSiliconStore((state) => state.alerts);
export const useRoutingDecisions = () => useAppleSiliconStore((state) => state.routingDecisions);
export const useRecommendations = () => useAppleSiliconStore((state) => state.recommendations);
export const useAppleSiliconLoading = () => useAppleSiliconStore((state) => state.loading);
export const useAppleSiliconErrors = () => useAppleSiliconStore((state) => state.errors);

// Computed selectors
export const useUnacknowledgedAlerts = () => useAppleSiliconStore((state) =>
  state.alerts.filter(alert => !alert.acknowledged)
);

export const useCriticalAlerts = () => useAppleSiliconStore((state) =>
  state.alerts.filter(alert => alert.severity === 'critical' && !alert.acknowledged)
);

export const useHighPriorityRecommendations = () => useAppleSiliconStore((state) =>
  state.recommendations.filter(rec => rec.priority === 'high' || rec.priority === 'critical')
);

export const useThermalStatus = () => useAppleSiliconStore((state) => {
  if (!state.currentMetrics) return null;

  const { thermal } = state.currentMetrics;
  return {
    status: thermal.thermalThrottling ? 'critical' :
            thermal.cpuTemperature > 85 || thermal.gpuTemperature > 85 ? 'warning' : 'optimal',
    temperatures: {
      cpu: thermal.cpuTemperature,
      gpu: thermal.gpuTemperature,
      ane: thermal.aneTemperature,
      ambient: thermal.ambientTemperature,
    },
    cooling: {
      efficiency: thermal.coolingEfficiency,
      fanSpeed: thermal.fanSpeed,
      margin: thermal.thermalMargin,
    }
  };
});

export const useHardwareUtilization = () => useAppleSiliconStore((state) => {
  if (!state.currentMetrics) return null;

  const { ane, gpu, cpu, memory } = state.currentMetrics;
  return {
    ane: ane.utilization,
    gpu: gpu.utilization,
    cpu: cpu.utilization,
    memory: ((memory.totalMemory - memory.availableMemory) / memory.totalMemory) * 100,
    average: (ane.utilization + gpu.utilization + cpu.utilization +
             ((memory.totalMemory - memory.availableMemory) / memory.totalMemory) * 100) / 4,
  };
});

export const usePowerEfficiency = () => useAppleSiliconStore((state) => {
  if (!state.currentMetrics || !state.activeModels.length) return null;

  const { power } = state.currentMetrics;
  const totalThroughput = state.activeModels.reduce((sum, model) =>
    sum + model.performance.throughput, 0
  );

  return {
    currentConsumption: power.totalConsumption,
    efficiency: totalThroughput > 0 ? totalThroughput / power.totalConsumption : 0,
    breakdown: {
      cpu: power.cpuConsumption,
      gpu: power.gpuConsumption,
      ane: power.aneConsumption,
      other: power.totalConsumption - power.cpuConsumption - power.gpuConsumption - power.aneConsumption,
    },
    batteryLevel: power.batteryLevel,
    charging: power.charging,
  };
});

export const useAppleSiliconActions = () => useAppleSiliconStore((state) => ({
  setCurrentMetrics: state.setCurrentMetrics,
  setHistoricalMetrics: state.setHistoricalMetrics,
  addHistoricalMetrics: state.addHistoricalMetrics,
  setDeviceStatus: state.setDeviceStatus,
  setActiveModels: state.setActiveModels,
  updateModel: state.updateModel,
  setAlerts: state.setAlerts,
  addAlert: state.addAlert,
  acknowledgeAlert: state.acknowledgeAlert,
  setRoutingDecisions: state.setRoutingDecisions,
  addRoutingDecision: state.addRoutingDecision,
  setRecommendations: state.setRecommendations,
  addRecommendation: state.addRecommendation,
  setTimeRange: state.setTimeRange,
  setSelectedComponents: state.setSelectedComponents,
  toggleComponent: state.toggleComponent,
  setRealTimeEnabled: state.setRealTimeEnabled,
  setLastUpdate: state.setLastUpdate,
  setLoading: state.setLoading,
  setError: state.setError,
  clearErrors: state.clearErrors,
  setPagination: state.setPagination,
  nextAlertsPage: state.nextAlertsPage,
  nextRoutingPage: state.nextRoutingPage,
  resetPagination: state.resetPagination,
  setFilters: state.setFilters,
  clearFilters: state.clearFilters,
  reset: state.reset,
}));

// Add missing export
export const useFilteredModels = () => useAppleSiliconStore((state) => state.activeModels);