/**
 * Council State Management Store
 * Zustand store for managing council-related state across the application
 *
 * @author @darianrosebrook
 */

import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { Verdict, VerdictStatus, Judge, Evidence } from '@/components/council/VerdictList';
import { EthicalAssessment, JudgePerformance, CouncilStats, VerdictIntervention } from '@/lib/council-api';

// Types for internal state
interface CouncilFilters {
  status?: VerdictStatus[];
  judgeCount?: number;
  consensusScore?: { min: number; max: number };
  ethicalConcerns?: number;
  dateRange?: { start: Date; end: Date };
  search?: string;
  sortBy?: 'createdAt' | 'updatedAt' | 'consensusScore' | 'ethicalConcerns';
  sortOrder?: 'asc' | 'desc';
}

interface PaginationState {
  currentPage: number;
  pageSize: number;
  total: number;
  hasMore: boolean;
}

interface LoadingStates {
  verdicts: boolean;
  verdictDetails: boolean;
  judges: boolean;
  judgeMetrics: boolean;
  ethicalAssessments: boolean;
  interventions: boolean;
  stats: boolean;
}

interface ErrorStates {
  verdicts?: string;
  verdictDetails?: string;
  judges?: string;
  judgeMetrics?: string;
  ethicalAssessments?: string;
  interventions?: string;
  stats?: string;
}

// Main council state interface
interface CouncilState {
  // Data
  verdicts: Verdict[];
  selectedVerdict: Verdict | null;
  judges: Judge[];
  judgeMetrics: JudgePerformance[];
  ethicalAssessments: EthicalAssessment[];
  interventions: VerdictIntervention[];
  stats: CouncilStats | null;

  // UI State
  filters: CouncilFilters;
  pagination: PaginationState;
  loading: LoadingStates;
  errors: ErrorStates;
  lastUpdated: Date | null;

  // Actions
  setVerdicts: (verdicts: Verdict[], total?: number, hasMore?: boolean) => void;
  addVerdict: (verdict: Verdict) => void;
  updateVerdict: (id: string, updates: Partial<Verdict>) => void;
  removeVerdict: (id: string) => void;
  setSelectedVerdict: (verdict: Verdict | null) => void;

  setJudges: (judges: Judge[]) => void;
  updateJudge: (id: string, updates: Partial<Judge>) => void;

  setJudgeMetrics: (metrics: JudgePerformance[]) => void;
  updateJudgeMetrics: (judgeId: string, metrics: Partial<JudgePerformance>) => void;

  setEthicalAssessments: (assessments: EthicalAssessment[]) => void;
  addEthicalAssessment: (assessment: EthicalAssessment) => void;
  updateEthicalAssessment: (id: string, updates: Partial<EthicalAssessment>) => void;

  setInterventions: (interventions: VerdictIntervention[]) => void;
  addIntervention: (intervention: VerdictIntervention) => void;
  updateIntervention: (id: string, updates: Partial<VerdictIntervention>) => void;

  setStats: (stats: CouncilStats) => void;

  // Filter and pagination actions
  setFilters: (filters: CouncilFilters) => void;
  updateFilters: (updates: Partial<CouncilFilters>) => void;
  resetFilters: () => void;

  setPagination: (pagination: Partial<PaginationState>) => void;
  setCurrentPage: (page: number) => void;
  setPageSize: (size: number) => void;

  // Loading and error actions
  setLoading: (key: keyof LoadingStates, loading: boolean) => void;
  setError: (key: keyof ErrorStates, error: string | undefined) => void;
  clearErrors: () => void;

  // Utility actions
  refreshData: () => void;
  resetState: () => void;
}

// Default state values
const defaultFilters: CouncilFilters = {
  sortBy: 'createdAt',
  sortOrder: 'desc',
};

const defaultPagination: PaginationState = {
  currentPage: 1,
  pageSize: 20,
  total: 0,
  hasMore: false,
};

const defaultLoading: LoadingStates = {
  verdicts: false,
  verdictDetails: false,
  judges: false,
  judgeMetrics: false,
  ethicalAssessments: false,
  interventions: false,
  stats: false,
};

const defaultErrors: ErrorStates = {};

// Create the Zustand store
export const useCouncilStore = create<CouncilState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    verdicts: [],
    selectedVerdict: null,
    judges: [],
    judgeMetrics: [],
    ethicalAssessments: [],
    interventions: [],
    stats: null,

    filters: defaultFilters,
    pagination: defaultPagination,
    loading: defaultLoading,
    errors: defaultErrors,
    lastUpdated: null,

    // Verdict actions
    setVerdicts: (verdicts, total, hasMore) => set((state) => ({
      verdicts,
      pagination: {
        ...state.pagination,
        total: total ?? verdicts.length,
        hasMore: hasMore ?? false,
      },
      lastUpdated: new Date(),
    })),

    addVerdict: (verdict) => set((state) => ({
      verdicts: [verdict, ...state.verdicts],
      lastUpdated: new Date(),
    })),

    updateVerdict: (id, updates) => set((state) => ({
      verdicts: state.verdicts.map(v => v.id === id ? { ...v, ...updates } : v),
      selectedVerdict: state.selectedVerdict?.id === id
        ? { ...state.selectedVerdict, ...updates }
        : state.selectedVerdict,
      lastUpdated: new Date(),
    })),

    removeVerdict: (id) => set((state) => ({
      verdicts: state.verdicts.filter(v => v.id !== id),
      selectedVerdict: state.selectedVerdict?.id === id ? null : state.selectedVerdict,
      lastUpdated: new Date(),
    })),

    setSelectedVerdict: (verdict) => set({ selectedVerdict: verdict }),

    // Judge actions
    setJudges: (judges) => set({ judges, lastUpdated: new Date() }),

    updateJudge: (id, updates) => set((state) => ({
      judges: state.judges.map(j => j.id === id ? { ...j, ...updates } : j),
      lastUpdated: new Date(),
    })),

    // Judge metrics actions
    setJudgeMetrics: (metrics) => set({ judgeMetrics: metrics, lastUpdated: new Date() }),

    updateJudgeMetrics: (judgeId, metrics) => set((state) => ({
      judgeMetrics: state.judgeMetrics.map(j =>
        j.id === judgeId ? { ...j, ...metrics } : j
      ),
      lastUpdated: new Date(),
    })),

    // Ethical assessment actions
    setEthicalAssessments: (assessments) => set({
      ethicalAssessments: assessments,
      lastUpdated: new Date()
    }),

    addEthicalAssessment: (assessment) => set((state) => ({
      ethicalAssessments: [assessment, ...state.ethicalAssessments],
      lastUpdated: new Date(),
    })),

    updateEthicalAssessment: (id, updates) => set((state) => ({
      ethicalAssessments: state.ethicalAssessments.map(a =>
        a.id === id ? { ...a, ...updates } : a
      ),
      lastUpdated: new Date(),
    })),

    // Intervention actions
    setInterventions: (interventions) => set({
      interventions,
      lastUpdated: new Date()
    }),

    addIntervention: (intervention) => set((state) => ({
      interventions: [intervention, ...state.interventions],
      lastUpdated: new Date(),
    })),

    updateIntervention: (id, updates) => set((state) => ({
      interventions: state.interventions.map(i =>
        i.id === id ? { ...i, ...updates } : i
      ),
      lastUpdated: new Date(),
    })),

    // Stats actions
    setStats: (stats) => set({ stats, lastUpdated: new Date() }),

    // Filter actions
    setFilters: (filters) => set({ filters }),

    updateFilters: (updates) => set((state) => ({
      filters: { ...state.filters, ...updates },
      pagination: { ...state.pagination, currentPage: 1 }, // Reset to first page
    })),

    resetFilters: () => set({ filters: defaultFilters }),

    // Pagination actions
    setPagination: (pagination) => set((state) => ({
      pagination: { ...state.pagination, ...pagination },
    })),

    setCurrentPage: (page) => set((state) => ({
      pagination: { ...state.pagination, currentPage: page },
    })),

    setPageSize: (size) => set((state) => ({
      pagination: { ...state.pagination, pageSize: size, currentPage: 1 },
    })),

    // Loading and error actions
    setLoading: (key, loading) => set((state) => ({
      loading: { ...state.loading, [key]: loading },
    })),

    setError: (key, error) => set((state) => ({
      errors: { ...state.errors, [key]: error },
    })),

    clearErrors: () => set({ errors: defaultErrors }),

    // Utility actions
    refreshData: () => {
      // This would trigger API calls to refresh all data
      console.log('Refreshing council data...');
    },

    resetState: () => set({
      verdicts: [],
      selectedVerdict: null,
      judges: [],
      judgeMetrics: [],
      ethicalAssessments: [],
      interventions: [],
      stats: null,
      filters: defaultFilters,
      pagination: defaultPagination,
      loading: defaultLoading,
      errors: defaultErrors,
      lastUpdated: null,
    }),
  }))
);

// Selectors for commonly used state slices
export const useCouncilVerdicts = () => useCouncilStore((state) => state.verdicts);
export const useCouncilFilters = () => useCouncilStore((state) => state.filters);
export const useCouncilPagination = () => useCouncilStore((state) => state.pagination);
export const useCouncilLoading = () => useCouncilStore((state) => state.loading);
export const useCouncilErrors = () => useCouncilStore((state) => state.errors);
export const useSelectedVerdict = () => useCouncilStore((state) => state.selectedVerdict);
export const useCouncilStats = () => useCouncilStore((state) => state.stats);

// Computed selectors
export const useFilteredVerdicts = () => {
  const { verdicts, filters } = useCouncilStore();

  return verdicts.filter(verdict => {
    // Status filter
    if (filters.status && filters.status.length > 0 && !filters.status.includes(verdict.status)) {
      return false;
    }

    // Judge count filter
    if (filters.judgeCount && verdict.judgeCount < filters.judgeCount) {
      return false;
    }

    // Consensus score filter
    if (filters.consensusScore) {
      const { min, max } = filters.consensusScore;
      if (verdict.consensusScore < min || verdict.consensusScore > max) {
        return false;
      }
    }

    // Ethical concerns filter
    if (filters.ethicalConcerns !== undefined && verdict.ethicalConcerns < filters.ethicalConcerns) {
      return false;
    }

    // Date range filter
    if (filters.dateRange) {
      const verdictDate = new Date(verdict.createdAt);
      if (verdictDate < filters.dateRange.start || verdictDate > filters.dateRange.end) {
        return false;
      }
    }

    // Search filter
    if (filters.search) {
      const query = filters.search.toLowerCase();
      const searchableText = `${verdict.title} ${verdict.summary} ${verdict.taskId}`.toLowerCase();
      if (!searchableText.includes(query)) {
        return false;
      }
    }

    return true;
  });
};

export const useSortedVerdicts = () => {
  const filteredVerdicts = useFilteredVerdicts();
  const { sortBy, sortOrder } = useCouncilStore((state) => state.filters);

  return [...filteredVerdicts].sort((a, b) => {
    let aValue: any = a[sortBy || 'createdAt'];
    let bValue: any = b[sortBy || 'createdAt'];

    if (sortBy === 'createdAt' || sortBy === 'updatedAt') {
      aValue = new Date(aValue).getTime();
      bValue = new Date(bValue).getTime();
    }

    if (aValue < bValue) return sortOrder === 'asc' ? -1 : 1;
    if (aValue > bValue) return sortOrder === 'asc' ? 1 : -1;
    return 0;
  });
};

// Actions selectors
export const useCouncilActions = () => useCouncilStore((state) => ({
  setVerdicts: state.setVerdicts,
  addVerdict: state.addVerdict,
  updateVerdict: state.updateVerdict,
  removeVerdict: state.removeVerdict,
  setSelectedVerdict: state.setSelectedVerdict,
  setJudges: state.setJudges,
  updateJudge: state.updateJudge,
  setJudgeMetrics: state.setJudgeMetrics,
  updateJudgeMetrics: state.updateJudgeMetrics,
  setEthicalAssessments: state.setEthicalAssessments,
  addEthicalAssessment: state.addEthicalAssessment,
  updateEthicalAssessment: state.updateEthicalAssessment,
  setInterventions: state.setInterventions,
  addIntervention: state.addIntervention,
  updateIntervention: state.updateIntervention,
  setStats: state.setStats,
  setFilters: state.setFilters,
  updateFilters: state.updateFilters,
  resetFilters: state.resetFilters,
  setPagination: state.setPagination,
  setCurrentPage: state.setCurrentPage,
  setPageSize: state.setPageSize,
  setLoading: state.setLoading,
  setError: state.setError,
  clearErrors: state.clearErrors,
  refreshData: state.refreshData,
  resetState: state.resetState,
}));
