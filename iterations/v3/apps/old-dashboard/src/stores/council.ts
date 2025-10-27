/**
 * Council Store
 * Zustand store for council oversight state management
 *
 * @author @darianrosebrook
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import {
  Verdict,
  Judge,
  CouncilMetrics,
  CouncilAlert,
  EthicalAssessment,
  VerdictFilter,
  JudgePerformanceReport
} from '@/lib/council-api';

interface CouncilState {
  // Core data
  verdicts: Verdict[];
  judges: Judge[];
  metrics: CouncilMetrics | null;
  alerts: CouncilAlert[];

  // UI state
  selectedVerdict: Verdict | null;
  selectedJudge: Judge | null;
  verdictFilters: VerdictFilter;

  // Loading states
  loading: {
    verdicts: boolean;
    judges: boolean;
    metrics: boolean;
    alerts: boolean;
    verdict: boolean;
    judge: boolean;
  };

  // Error states
  errors: {
    verdicts: string | null;
    judges: string | null;
    metrics: string | null;
    alerts: string | null;
    verdict: string | null;
    judge: string | null;
  };

  // Pagination
  pagination: {
    page: number;
    limit: number;
    total: number;
    hasMore: boolean;
  };

  // Real-time updates
  realTimeEnabled: boolean;
  lastUpdate: Date | null;
}

interface CouncilActions {
  // Verdict actions
  setVerdicts: (verdicts: Verdict[]) => void;
  addVerdict: (verdict: Verdict) => void;
  updateVerdict: (id: string, updates: Partial<Verdict>) => void;
  removeVerdict: (id: string) => void;
  setSelectedVerdict: (verdict: Verdict | null) => void;

  // Judge actions
  setJudges: (judges: Judge[]) => void;
  updateJudge: (id: string, updates: Partial<Judge>) => void;
  setSelectedJudge: (judge: Judge | null) => void;

  // Metrics and alerts
  setMetrics: (metrics: CouncilMetrics) => void;
  setAlerts: (alerts: CouncilAlert[]) => void;
  addAlert: (alert: CouncilAlert) => void;
  acknowledgeAlert: (alertId: string) => void;

  // Filter actions
  setVerdictFilters: (filters: VerdictFilter) => void;
  clearVerdictFilters: () => void;

  // Loading actions
  setLoading: (key: keyof CouncilState['loading'], loading: boolean) => void;
  setError: (key: keyof CouncilState['errors'], error: string | null) => void;
  clearErrors: () => void;

  // Pagination actions
  setPagination: (pagination: Partial<CouncilState['pagination']>) => void;
  nextPage: () => void;
  prevPage: () => void;
  resetPagination: () => void;

  // Real-time actions
  setRealTimeEnabled: (enabled: boolean) => void;
  setLastUpdate: (timestamp: Date) => void;

  // Utility actions
  reset: () => void;
}

const initialState: CouncilState = {
  verdicts: [],
  judges: [],
  metrics: null,
  alerts: [],
  selectedVerdict: null,
  selectedJudge: null,
  verdictFilters: {},
  loading: {
    verdicts: false,
    judges: false,
    metrics: false,
    alerts: false,
    verdict: false,
    judge: false,
  },
  errors: {
    verdicts: null,
    judges: null,
    metrics: null,
    alerts: null,
    verdict: null,
    judge: null,
  },
  pagination: {
    page: 1,
    limit: 20,
    total: 0,
    hasMore: false,
  },
  realTimeEnabled: true,
  lastUpdate: null,
};

export const useCouncilStore = create<CouncilState & CouncilActions>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // Verdict actions
      setVerdicts: (verdicts) => set({ verdicts }),
      addVerdict: (verdict) => set((state) => ({
        verdicts: [verdict, ...state.verdicts]
      })),
      updateVerdict: (id, updates) => set((state) => ({
        verdicts: state.verdicts.map(v => v.id === id ? { ...v, ...updates } : v),
        selectedVerdict: state.selectedVerdict?.id === id
          ? { ...state.selectedVerdict, ...updates }
          : state.selectedVerdict
      })),
      removeVerdict: (id) => set((state) => ({
        verdicts: state.verdicts.filter(v => v.id !== id),
        selectedVerdict: state.selectedVerdict?.id === id ? null : state.selectedVerdict
      })),
      setSelectedVerdict: (verdict) => set({ selectedVerdict: verdict }),

      // Judge actions
      setJudges: (judges) => set({ judges }),
      updateJudge: (id, updates) => set((state) => ({
        judges: state.judges.map(j => j.id === id ? { ...j, ...updates } : j),
        selectedJudge: state.selectedJudge?.id === id
          ? { ...state.selectedJudge, ...updates }
          : state.selectedJudge
      })),
      setSelectedJudge: (judge) => set({ selectedJudge: judge }),

      // Metrics and alerts
      setMetrics: (metrics) => set({ metrics }),
      setAlerts: (alerts) => set({ alerts }),
      addAlert: (alert) => set((state) => ({
        alerts: [alert, ...state.alerts]
      })),
      acknowledgeAlert: (alertId) => set((state) => ({
        alerts: state.alerts.map(a =>
          a.id === alertId ? { ...a, acknowledged: true } : a
        )
      })),

      // Filter actions
      setVerdictFilters: (filters) => set({ verdictFilters: filters }),
      clearVerdictFilters: () => set({ verdictFilters: {} }),

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
      nextPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          page: state.pagination.page + 1,
          hasMore: (state.pagination.page + 1) * state.pagination.limit < state.pagination.total
        }
      })),
      prevPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          page: Math.max(1, state.pagination.page - 1)
        }
      })),
      resetPagination: () => set({ pagination: initialState.pagination }),

      // Real-time actions
      setRealTimeEnabled: (enabled) => set({ realTimeEnabled: enabled }),
      setLastUpdate: (timestamp) => set({ lastUpdate: timestamp }),

      // Utility actions
      reset: () => set(initialState),
    }),
    {
      name: 'council-store',
    }
  )
);

// Selector hooks for better performance
export const useVerdicts = () => useCouncilStore((state) => state.verdicts);
export const useSelectedVerdict = () => useCouncilStore((state) => state.selectedVerdict);
export const useJudges = () => useCouncilStore((state) => state.judges);
export const useSelectedJudge = () => useCouncilStore((state) => state.selectedJudge);
export const useCouncilMetrics = () => useCouncilStore((state) => state.metrics);
export const useCouncilAlerts = () => useCouncilStore((state) => state.alerts);
export const useVerdictFilters = () => useCouncilStore((state) => state.verdictFilters);
export const useCouncilLoading = () => useCouncilStore((state) => state.loading);
export const useCouncilErrors = () => useCouncilStore((state) => state.errors);
export const useCouncilPagination = () => useCouncilStore((state) => state.pagination);

// Add missing exports
export const useCouncilFilters = () => useCouncilStore((state) => state.verdictFilters);
export const useCouncilStats = () => useCouncilStore((state) => state.metrics);
export const useSortedVerdicts = () => useCouncilStore((state) => state.verdicts);

export const useCouncilActions = () => useCouncilStore((state) => ({
  setVerdicts: state.setVerdicts,
  addVerdict: state.addVerdict,
  updateVerdict: state.updateVerdict,
  removeVerdict: state.removeVerdict,
  setSelectedVerdict: state.setSelectedVerdict,
  setJudges: state.setJudges,
  updateJudge: state.updateJudge,
  setSelectedJudge: state.setSelectedJudge,
  setMetrics: state.setMetrics,
  setAlerts: state.setAlerts,
  addAlert: state.addAlert,
  acknowledgeAlert: state.acknowledgeAlert,
  setVerdictFilters: state.setVerdictFilters,
  clearVerdictFilters: state.clearVerdictFilters,
  setLoading: state.setLoading,
  setError: state.setError,
  clearErrors: state.clearErrors,
  setPagination: state.setPagination,
  nextPage: state.nextPage,
  prevPage: state.prevPage,
  resetPagination: state.resetPagination,
  setRealTimeEnabled: state.setRealTimeEnabled,
  setLastUpdate: state.setLastUpdate,
  reset: state.reset,
}));

// Computed selectors
export const useActiveVerdicts = () => useCouncilStore((state) =>
  state.verdicts.filter(v => v.status === 'in_progress' || v.status === 'pending')
);

export const usePendingInterventions = () => useCouncilStore((state) =>
  state.verdicts.filter(v => v.status === 'escalated' || v.intervention)
);

export const useUnacknowledgedAlerts = () => useCouncilStore((state) =>
  state.alerts.filter(a => !a.acknowledged)
);

export const useHighRiskVerdicts = () => useCouncilStore((state) =>
  state.verdicts.filter(v =>
    v.ethicalAssessment.overallRisk === 'high' ||
    v.ethicalAssessment.overallRisk === 'critical'
  )
);