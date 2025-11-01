/**
 * Dashboard Store - Zustand State Management
 * Centralized state for the entire dashboard application
 */

import { create } from 'zustand';
import { devtools, subscribeWithSelector } from 'zustand/middleware';

// Types
export interface ConnectionState {
  status: 'connected' | 'disconnected' | 'connecting';
  lastConnected?: string;
  retryCount: number;
}

export interface MetricsData {
  systemHealth: any;
  businessMetrics: any;
  agentPerformance: any[];
  coordinationMetrics: any;
}

export interface TasksData {
  tasks: any[];
  totalCount: number;
  filters: {
    status?: string;
    priority?: string;
    limit?: number;
    offset?: number;
  };
}

export interface AlertsData {
  alerts: any[];
  statistics: {
    total: number;
    acknowledged: number;
    resolved: number;
  };
}

export interface DashboardState {
  // Connection state
  connection: ConnectionState;
  
  // Data state
  metrics: MetricsData | null;
  tasks: TasksData;
  alerts: AlertsData;
  
  // UI state
  isLoading: boolean;
  error: string | null;
  lastUpdated: string | null;
  
  // Actions
  setConnection: (connection: Partial<ConnectionState>) => void;
  setMetrics: (metrics: MetricsData) => void;
  setTasks: (tasks: Partial<TasksData>) => void;
  setAlerts: (alerts: Partial<AlertsData>) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  updateLastUpdated: () => void;
  
  // Computed selectors
  isConnected: () => boolean;
  hasData: () => boolean;
  getActiveAlerts: () => any[];
}

export const useDashboardStore = create<DashboardState>()(
  devtools(
    subscribeWithSelector((set, get) => ({
      // Initial state
      connection: {
        status: 'disconnected',
        retryCount: 0,
      },
      
      metrics: null,
      tasks: {
        tasks: [],
        totalCount: 0,
        filters: {},
      },
      alerts: {
        alerts: [],
        statistics: {
          total: 0,
          acknowledged: 0,
          resolved: 0,
        },
      },
      
      isLoading: false,
      error: null,
      lastUpdated: null,
      
      // Actions
      setConnection: (connection) =>
        set((state) => ({
          connection: { ...state.connection, ...connection },
        })),
      
      setMetrics: (metrics) =>
        set({ metrics, error: null }),
      
      setTasks: (tasks) =>
        set((state) => ({
          tasks: { ...state.tasks, ...tasks },
        })),
      
      setAlerts: (alerts) =>
        set((state) => ({
          alerts: { ...state.alerts, ...alerts },
        })),
      
      setLoading: (isLoading) => set({ isLoading }),
      
      setError: (error) => set({ error }),
      
      updateLastUpdated: () =>
        set({ lastUpdated: new Date().toISOString() }),
      
      // Computed selectors
      isConnected: () => get().connection.status === 'connected',
      
      hasData: () => {
        const state = get();
        return !!(state.metrics || state.tasks.tasks.length > 0 || state.alerts.alerts.length > 0);
      },
      
      getActiveAlerts: () => {
        const state = get();
        return state.alerts.alerts.filter(alert => 
          alert.status === 'active' || alert.status === 'pending'
        );
      },
    })),
    {
      name: 'dashboard-store',
    }
  )
);

// Selector hooks for performance optimization
export const useConnection = () => useDashboardStore((state) => state.connection);
export const useMetrics = () => useDashboardStore((state) => state.metrics);
export const useTasks = () => useDashboardStore((state) => state.tasks);
export const useAlerts = () => useDashboardStore((state) => state.alerts);
export const useIsLoading = () => useDashboardStore((state) => state.isLoading);
export const useError = () => useDashboardStore((state) => state.error);

// Action hooks
export const useDashboardActions = () => useDashboardStore((state) => ({
  setConnection: state.setConnection,
  setMetrics: state.setMetrics,
  setTasks: state.setTasks,
  setAlerts: state.setAlerts,
  setLoading: state.setLoading,
  setError: state.setError,
  updateLastUpdated: state.updateLastUpdated,
}));

