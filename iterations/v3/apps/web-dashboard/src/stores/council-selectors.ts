/**
 * Council Store Selectors
 * Performance-optimized selectors for the Council Zustand store
 *
 * @author @darianrosebrook
 */

import { CouncilState } from './council';

// Base selectors
export const selectVerdicts = (state: CouncilState) => state.verdicts;
export const selectJudges = (state: CouncilState) => state.judges;
export const selectMetrics = (state: CouncilState) => state.metrics;
export const selectAlerts = (state: CouncilState) => state.alerts;
export const selectSelectedVerdict = (state: CouncilState) => state.selectedVerdict;
export const selectSelectedJudge = (state: CouncilState) => state.selectedJudge;
export const selectVerdictFilters = (state: CouncilState) => state.verdictFilters;
export const selectPagination = (state: CouncilState) => state.pagination;
export const selectLoading = (state: CouncilState) => state.loading;
export const selectErrors = (state: CouncilState) => state.errors;
export const selectRealTimeEnabled = (state: CouncilState) => state.realTimeEnabled;
export const selectLastUpdate = (state: CouncilState) => state.lastUpdate;

// Composite selectors for VerdictList component
export const selectVerdictListData = (state: CouncilState) => ({
  verdicts: state.verdicts,
  filters: state.verdictFilters,
  pagination: state.pagination,
  loading: state.loading.verdicts,
  error: state.errors.verdicts,
  selectedVerdict: state.selectedVerdict,
});

export const selectVerdictListActions = (state: CouncilState) => ({
  setVerdicts: state.setVerdicts,
  updateVerdict: state.updateVerdict,
  setSelectedVerdict: state.setSelectedVerdict,
  setVerdictFilters: state.setVerdictFilters,
  setLoading: (loading: boolean) => state.setLoading('verdicts', loading),
  setError: (error: string | null) => state.setError('verdicts', error),
  setPagination: state.setPagination,
  setCurrentPage: state.setCurrentPage,
});

// Filtered and computed selectors
export const selectFilteredVerdicts = (state: CouncilState) => {
  const { verdicts, verdictFilters } = state;

  return verdicts.filter(verdict => {
    // Status filter
    if (verdictFilters.status && verdictFilters.status.length > 0 && !verdictFilters.status.includes(verdict.status)) {
      return false;
    }

    // Judge count filter
    if (verdictFilters.judgeCount && verdict.judges.length < verdictFilters.judgeCount) {
      return false;
    }

    // Consensus score filter
    if (verdictFilters.consensusScore) {
      const { min, max } = verdictFilters.consensusScore;
      if (verdict.consensus.confidence < min || verdict.consensus.confidence > max) {
        return false;
      }
    }

    // Ethical concerns filter
    if (verdictFilters.ethicalConcerns !== undefined && verdict.ethicalAssessment.concerns.length < verdictFilters.ethicalConcerns) {
      return false;
    }

    // Date range filter
    if (verdictFilters.dateRange) {
      const { start, end } = verdictFilters.dateRange;
      if (verdict.createdAt < start || verdict.createdAt > end) {
        return false;
      }
    }

    // Search filter
    if (verdictFilters.search) {
      const query = verdictFilters.search.toLowerCase();
      const searchableText = `${verdict.taskId} ${verdict.status}`.toLowerCase();
      if (!searchableText.includes(query)) {
        return false;
      }
    }

    return true;
  });
};

export const selectSortedVerdicts = (state: CouncilState) => {
  const filteredVerdicts = selectFilteredVerdicts(state);
  const { sortBy, sortOrder } = state.verdictFilters;

  return [...filteredVerdicts].sort((a, b) => {
    let aValue: any, bValue: any;

    switch (sortBy) {
      case 'createdAt':
        aValue = new Date(a.createdAt).getTime();
        bValue = new Date(b.createdAt).getTime();
        break;
      case 'updatedAt':
        aValue = new Date(a.updatedAt).getTime();
        bValue = new Date(b.updatedAt).getTime();
        break;
      case 'consensusScore':
        aValue = a.consensus.confidence;
        bValue = b.consensus.confidence;
        break;
      case 'ethicalConcerns':
        aValue = a.ethicalAssessment.concerns.length;
        bValue = b.ethicalAssessment.concerns.length;
        break;
      default:
        aValue = a.createdAt;
        bValue = b.createdAt;
    }

    if (aValue < bValue) return sortOrder === 'asc' ? -1 : 1;
    if (aValue > bValue) return sortOrder === 'asc' ? 1 : -1;
    return 0;
  });
};

export const selectPaginatedVerdicts = (state: CouncilState) => {
  const sortedVerdicts = selectSortedVerdicts(state);
  const { currentPage, pageSize } = state.pagination;

  const startIndex = (currentPage - 1) * pageSize;
  return sortedVerdicts.slice(startIndex, startIndex + pageSize);
};

export const selectVerdictListUIState = (state: CouncilState) => ({
  showFilters: state.verdictFilters.showFilters,
  sortBy: state.verdictFilters.sortBy,
  sortOrder: state.verdictFilters.sortOrder,
  currentPage: state.pagination.currentPage,
  totalPages: Math.ceil(selectFilteredVerdicts(state).length / state.pagination.pageSize),
  totalVerdicts: state.verdicts.length,
  filteredCount: selectFilteredVerdicts(state).length,
});

// Alert selectors
export const selectActiveAlerts = (state: CouncilState) =>
  state.alerts.filter(alert => !alert.acknowledged);

export const selectAlertStats = (state: CouncilState) => ({
  total: state.alerts.length,
  active: selectActiveAlerts(state).length,
  critical: state.alerts.filter(alert => alert.severity === 'critical').length,
  high: state.alerts.filter(alert => alert.severity === 'high').length,
  medium: state.alerts.filter(alert => alert.severity === 'medium').length,
  low: state.alerts.filter(alert => alert.severity === 'low').length,
});

// Judge performance selectors
export const selectTopPerformingJudges = (state: CouncilState) =>
  [...state.judges].sort((a, b) => (b.accuracy || 0) - (a.accuracy || 0)).slice(0, 5);

export const selectJudgeStats = (state: CouncilState) => ({
  total: state.judges.length,
  active: state.judges.filter(judge => judge.status === 'active').length,
  averageAccuracy: state.judges.reduce((sum, judge) => sum + (judge.accuracy || 0), 0) / state.judges.length,
  averageResponseTime: state.judges.reduce((sum, judge) => sum + (judge.averageResponseTime || 0), 0) / state.judges.length,
});

// Real-time selectors
export const selectRealTimeStatus = (state: CouncilState) => ({
  enabled: state.realTimeEnabled,
  lastUpdate: state.lastUpdate,
  isStale: state.lastUpdate ? Date.now() - state.lastUpdate.getTime() > 30000 : false, // 30 seconds
});
