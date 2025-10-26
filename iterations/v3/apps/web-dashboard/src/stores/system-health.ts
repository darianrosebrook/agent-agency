/**
 * System Health Store
 * Zustand store for system health monitoring and Grafana integration state management
 *
 * @author @darianrosebrook
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import {
  SystemHealth,
  ComponentHealth,
  SystemAlert,
  GrafanaDashboard,
  GrafanaAlert,
  HealthTrend,
  DependencyMap,
  MetricsResponse,
  CustomDashboard,
  MetricsQuery
} from '@/lib/system-health-api';

interface SystemHealthState {
  // Core data
  systemHealth: SystemHealth | null;
  components: ComponentHealth[];
  alerts: SystemAlert[];
  grafanaDashboards: GrafanaDashboard[];
  grafanaAlerts: GrafanaAlert[];
  healthTrends: HealthTrend[];
  dependencyMap: DependencyMap | null;
  metrics: Record<string, MetricsResponse>;
  customDashboards: CustomDashboard[];

  // UI state
  selectedComponent: ComponentHealth | null;
  selectedAlert: SystemAlert | null;
  selectedGrafanaDashboard: GrafanaDashboard | null;
  embeddedPanels: Record<string, {
    url: string;
    html: string;
    lastUpdated: Date;
  }>;
  realTimeEnabled: boolean;
  lastUpdate: Date | null;

  // Loading states
  loading: {
    health: boolean;
    components: boolean;
    alerts: boolean;
    grafana: boolean;
    trends: boolean;
    dependencies: boolean;
    metrics: boolean;
    customDashboards: boolean;
  };

  // Error states
  errors: {
    health: string | null;
    components: string | null;
    alerts: string | null;
    grafana: string | null;
    trends: string | null;
    dependencies: string | null;
    metrics: string | null;
    customDashboards: string | null;
  };

  // Pagination and filtering
  pagination: {
    alertsPage: number;
    componentsPage: number;
    dashboardsPage: number;
    limit: number;
  };

  filters: {
    alertStatus: SystemAlert['status'][] | null;
    alertSeverity: SystemAlert['severity'][] | null;
    alertSource: SystemAlert['source'][] | null;
    componentType: ComponentHealth['type'][] | null;
    componentStatus: ComponentHealth['status'][] | null;
    dashboardTags: string[] | null;
  };

  // Time range for trends and metrics
  timeRange: {
    start: Date;
    end: Date;
    interval: '1m' | '5m' | '15m' | '1h' | '6h' | '24h';
  };
}

interface SystemHealthActions {
  // Core data actions
  setSystemHealth: (health: SystemHealth) => void;
  setComponents: (components: ComponentHealth[]) => void;
  updateComponent: (componentId: string, updates: Partial<ComponentHealth>) => void;
  setAlerts: (alerts: SystemAlert[]) => void;
  addAlert: (alert: SystemAlert) => void;
  updateAlert: (alertId: string, updates: Partial<SystemAlert>) => void;
  setGrafanaDashboards: (dashboards: GrafanaDashboard[]) => void;
  setGrafanaAlerts: (alerts: GrafanaAlert[]) => void;
  setHealthTrends: (trends: HealthTrend[]) => void;
  addHealthTrend: (trend: HealthTrend) => void;
  setDependencyMap: (map: DependencyMap) => void;
  setMetrics: (queryKey: string, metrics: MetricsResponse) => void;
  setCustomDashboards: (dashboards: CustomDashboard[]) => void;
  updateCustomDashboard: (dashboardId: string, updates: Partial<CustomDashboard>) => void;

  // UI state actions
  setSelectedComponent: (component: ComponentHealth | null) => void;
  setSelectedAlert: (alert: SystemAlert | null) => void;
  setSelectedGrafanaDashboard: (dashboard: GrafanaDashboard | null) => void;
  setEmbeddedPanel: (panelId: string, embedData: { url: string; html: string }) => void;
  setRealTimeEnabled: (enabled: boolean) => void;
  setLastUpdate: (timestamp: Date) => void;
  setTimeRange: (timeRange: Partial<SystemHealthState['timeRange']>) => void;

  // Loading actions
  setLoading: (key: keyof SystemHealthState['loading'], loading: boolean) => void;
  setError: (key: keyof SystemHealthState['errors'], error: string | null) => void;
  clearErrors: () => void;

  // Pagination actions
  setPagination: (pagination: Partial<SystemHealthState['pagination']>) => void;
  nextAlertsPage: () => void;
  nextComponentsPage: () => void;
  nextDashboardsPage: () => void;
  resetPagination: () => void;

  // Filter actions
  setFilters: (filters: Partial<SystemHealthState['filters']>) => void;
  clearFilters: () => void;

  // Utility actions
  reset: () => void;
}

const initialState: SystemHealthState = {
  systemHealth: null,
  components: [],
  alerts: [],
  grafanaDashboards: [],
  grafanaAlerts: [],
  healthTrends: [],
  dependencyMap: null,
  metrics: {},
  customDashboards: [],
  selectedComponent: null,
  selectedAlert: null,
  selectedGrafanaDashboard: null,
  embeddedPanels: {},
  realTimeEnabled: true,
  lastUpdate: null,
  loading: {
    health: false,
    components: false,
    alerts: false,
    grafana: false,
    trends: false,
    dependencies: false,
    metrics: false,
    customDashboards: false,
  },
  errors: {
    health: null,
    components: null,
    alerts: null,
    grafana: null,
    trends: null,
    dependencies: null,
    metrics: null,
    customDashboards: null,
  },
  pagination: {
    alertsPage: 1,
    componentsPage: 1,
    dashboardsPage: 1,
    limit: 50,
  },
  filters: {
    alertStatus: null,
    alertSeverity: null,
    alertSource: null,
    componentType: null,
    componentStatus: null,
    dashboardTags: null,
  },
  timeRange: {
    start: new Date(Date.now() - 24 * 60 * 60 * 1000), // 24 hours ago
    end: new Date(),
    interval: '15m',
  },
};

export const useSystemHealthStore = create<SystemHealthState & SystemHealthActions>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // Core data actions
      setSystemHealth: (health) => set({ systemHealth: health }),
      setComponents: (components) => set({ components }),
      updateComponent: (componentId, updates) => set((state) => ({
        components: state.components.map(component =>
          component.id === componentId ? { ...component, ...updates } : component
        ),
        selectedComponent: state.selectedComponent?.id === componentId
          ? { ...state.selectedComponent, ...updates }
          : state.selectedComponent
      })),
      setAlerts: (alerts) => set({ alerts }),
      addAlert: (alert) => set((state) => ({
        alerts: [alert, ...state.alerts]
      })),
      updateAlert: (alertId, updates) => set((state) => ({
        alerts: state.alerts.map(alert =>
          alert.id === alertId ? { ...alert, ...updates } : alert
        ),
        selectedAlert: state.selectedAlert?.id === alertId
          ? { ...state.selectedAlert, ...updates }
          : state.selectedAlert
      })),
      setGrafanaDashboards: (dashboards) => set({ grafanaDashboards: dashboards }),
      setGrafanaAlerts: (alerts) => set({ grafanaAlerts: alerts }),
      setHealthTrends: (trends) => set({ healthTrends: trends }),
      addHealthTrend: (trend) => set((state) => ({
        healthTrends: [...state.healthTrends.slice(-99), trend] // Keep last 100
      })),
      setDependencyMap: (map) => set({ dependencyMap: map }),
      setMetrics: (queryKey, metrics) => set((state) => ({
        metrics: { ...state.metrics, [queryKey]: metrics }
      })),
      setCustomDashboards: (dashboards) => set({ customDashboards: dashboards }),
      updateCustomDashboard: (dashboardId, updates) => set((state) => ({
        customDashboards: state.customDashboards.map(dashboard =>
          dashboard.id === dashboardId ? { ...dashboard, ...updates } : dashboard
        )
      })),

      // UI state actions
      setSelectedComponent: (component) => set({ selectedComponent: component }),
      setSelectedAlert: (alert) => set({ selectedAlert: alert }),
      setSelectedGrafanaDashboard: (dashboard) => set({ selectedGrafanaDashboard: dashboard }),
      setEmbeddedPanel: (panelId, embedData) => set((state) => ({
        embeddedPanels: {
          ...state.embeddedPanels,
          [panelId]: {
            ...embedData,
            lastUpdated: new Date()
          }
        }
      })),
      setRealTimeEnabled: (enabled) => set({ realTimeEnabled: enabled }),
      setLastUpdate: (timestamp) => set({ lastUpdate: timestamp }),
      setTimeRange: (timeRange) => set((state) => ({
        timeRange: { ...state.timeRange, ...timeRange }
      })),

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
      nextComponentsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          componentsPage: state.pagination.componentsPage + 1
        }
      })),
      nextDashboardsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          dashboardsPage: state.pagination.dashboardsPage + 1
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
      name: 'system-health-store',
    }
  )
);

// Selector hooks for better performance
export const useSystemHealth = () => useSystemHealthStore((state) => state.systemHealth);
export const useComponents = () => useSystemHealthStore((state) => state.components);
export const useSystemAlerts = () => useSystemHealthStore((state) => state.alerts);
export const useGrafanaDashboards = () => useSystemHealthStore((state) => state.grafanaDashboards);
export const useGrafanaAlerts = () => useSystemHealthStore((state) => state.grafanaAlerts);
export const useHealthTrends = () => useSystemHealthStore((state) => state.healthTrends);
export const useDependencyMap = () => useSystemHealthStore((state) => state.dependencyMap);
export const useSystemMetrics = () => useSystemHealthStore((state) => state.metrics);
export const useCustomDashboards = () => useSystemHealthStore((state) => state.customDashboards);
export const useSelectedSystemComponent = () => useSystemHealthStore((state) => state.selectedComponent);
export const useSelectedSystemAlert = () => useSystemHealthStore((state) => state.selectedAlert);
export const useSelectedGrafanaDashboard = () => useSystemHealthStore((state) => state.selectedGrafanaDashboard);
export const useEmbeddedPanels = () => useSystemHealthStore((state) => state.embeddedPanels);
export const useSystemHealthLoading = () => useSystemHealthStore((state) => state.loading);
export const useSystemHealthErrors = () => useSystemHealthStore((state) => state.errors);

// Computed selectors
export const useActiveAlerts = () => useSystemHealthStore((state) =>
  state.alerts.filter(alert => alert.status === 'active')
);

export const useCriticalAlerts = () => useSystemHealthStore((state) =>
  state.alerts.filter(alert => alert.severity === 'critical' && alert.status === 'active')
);

export const useHealthyComponents = () => useSystemHealthStore((state) =>
  state.components.filter(component => component.status === 'healthy')
);

export const useUnhealthyComponents = () => useSystemHealthStore((state) =>
  state.components.filter(component => component.status === 'warning' || component.status === 'critical')
);

export const useSystemHealthScore = () => useSystemHealthStore((state) => {
  if (!state.systemHealth) return 0;
  return state.systemHealth.overallScore;
});

export const useOverallSystemStatus = () => useSystemHealthStore((state) => {
  if (!state.systemHealth) return 'unknown';

  const { overallStatus } = state.systemHealth;
  const criticalAlerts = state.alerts.filter(alert =>
    alert.severity === 'critical' && alert.status === 'active'
  ).length;

  const unhealthyComponents = state.components.filter(component =>
    component.status === 'critical'
  ).length;

  if (criticalAlerts > 0 || unhealthyComponents > 0) {
    return 'critical';
  }

  return overallStatus;
});

export const useComponentHealthByType = () => useSystemHealthStore((state) => {
  const byType: Record<string, ComponentHealth[]> = {};
  state.components.forEach(component => {
    if (!byType[component.type]) {
      byType[component.type] = [];
    }
    byType[component.type].push(component);
  });
  return byType;
});

export const useAlertStats = () => useSystemHealthStore((state) => {
  return {
    total: state.alerts.length,
    active: state.alerts.filter(a => a.status === 'active').length,
    acknowledged: state.alerts.filter(a => a.status === 'acknowledged').length,
    resolved: state.alerts.filter(a => a.status === 'resolved').length,
    bySeverity: {
      critical: state.alerts.filter(a => a.severity === 'critical').length,
      high: state.alerts.filter(a => a.severity === 'high').length,
      medium: state.alerts.filter(a => a.severity === 'medium').length,
      low: state.alerts.filter(a => a.severity === 'low').length,
    },
    bySource: {
      grafana: state.alerts.filter(a => a.source === 'grafana').length,
      prometheus: state.alerts.filter(a => a.source === 'prometheus').length,
      application: state.alerts.filter(a => a.source === 'application').length,
      infrastructure: state.alerts.filter(a => a.source === 'infrastructure').length,
      custom: state.alerts.filter(a => a.source === 'custom').length,
    },
  };
});

export const useRecentHealthTrends = () => useSystemHealthStore((state) =>
  state.healthTrends.slice(-20) // Last 20 data points
);

export const useGrafanaDashboardStats = () => useSystemHealthStore((state) => {
  return {
    total: state.grafanaDashboards.length,
    byFolder: state.grafanaDashboards.reduce((acc, dashboard) => {
      const folder = dashboard.folderTitle || 'General';
      acc[folder] = (acc[folder] || 0) + 1;
      return acc;
    }, {} as Record<string, number>),
    withAlerts: state.grafanaDashboards.filter(dashboard =>
      dashboard.panels.some(panel =>
        state.grafanaAlerts.some(alert => alert.panelId === panel.id)
      )
    ).length,
  };
});

export const useSystemHealthActions = () => useSystemHealthStore((state) => ({
  setSystemHealth: state.setSystemHealth,
  setComponents: state.setComponents,
  updateComponent: state.updateComponent,
  setAlerts: state.setAlerts,
  addAlert: state.addAlert,
  updateAlert: state.updateAlert,
  setGrafanaDashboards: state.setGrafanaDashboards,
  setGrafanaAlerts: state.setGrafanaAlerts,
  setHealthTrends: state.setHealthTrends,
  addHealthTrend: state.addHealthTrend,
  setDependencyMap: state.setDependencyMap,
  setMetrics: state.setMetrics,
  setCustomDashboards: state.setCustomDashboards,
  updateCustomDashboard: state.updateCustomDashboard,
  setSelectedComponent: state.setSelectedComponent,
  setSelectedAlert: state.setSelectedAlert,
  setSelectedGrafanaDashboard: state.setSelectedGrafanaDashboard,
  setEmbeddedPanel: state.setEmbeddedPanel,
  setRealTimeEnabled: state.setRealTimeEnabled,
  setLastUpdate: state.setLastUpdate,
  setTimeRange: state.setTimeRange,
  setLoading: state.setLoading,
  setError: state.setError,
  clearErrors: state.clearErrors,
  setPagination: state.setPagination,
  nextAlertsPage: state.nextAlertsPage,
  nextComponentsPage: state.nextComponentsPage,
  nextDashboardsPage: state.nextDashboardsPage,
  resetPagination: state.resetPagination,
  setFilters: state.setFilters,
  clearFilters: state.clearFilters,
  reset: state.reset,
}));
