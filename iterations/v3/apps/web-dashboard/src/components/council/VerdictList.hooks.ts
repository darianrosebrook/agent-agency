/**
 * VerdictList Hooks
 * Extracted logic from VerdictList component for better testability
 *
 * @author @darianrosebrook
 */

import { useCallback, useEffect } from 'react';
import { useCouncilStore } from '@/stores/council';
import { councilApiClient } from '@/lib/council-api';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import {
  selectVerdictListData,
  selectVerdictListActions,
  selectPaginatedVerdicts,
  selectVerdictListUIState,
} from '@/stores/council-selectors';
import { VerdictFilter } from '@/lib/council-api';

export function useVerdictList() {
  const { handleError } = useErrorHandler();

  // Get data from optimized selectors
  const {
    verdicts,
    filters,
    pagination,
    loading,
    error,
    selectedVerdict
  } = useCouncilStore(selectVerdictListData);

  const actions = useCouncilStore(selectVerdictListActions);
  const paginatedVerdicts = useCouncilStore(selectPaginatedVerdicts);
  const uiState = useCouncilStore(selectVerdictListUIState);

  // Fetch verdicts function
  const fetchVerdicts = useCallback(async (
    currentFilters: VerdictFilter = filters,
    page: number = pagination.currentPage
  ) => {
    try {
      actions.setLoading(true);
      actions.setError(null);

      const response = await councilApiClient.getVerdicts(currentFilters, page, pagination.pageSize);

      actions.setVerdicts(response.verdicts);
      actions.setPagination({
        currentPage: page,
        total: response.total,
        hasMore: response.hasMore
      });
    } catch (err) {
      const error = handleError(err, { context: 'verdict_list_fetch' });
      actions.setError(error.message);
    } finally {
      actions.setLoading(false);
    }
  }, [filters, pagination.currentPage, pagination.pageSize, actions, handleError]);

  // Handle page changes
  const handlePageChange = useCallback((page: number) => {
    actions.setCurrentPage(page);
    fetchVerdicts(filters, page);
  }, [actions, fetchVerdicts, filters]);

  // Handle sort changes
  const handleSortChange = useCallback((field: string) => {
    const newDirection = filters.sortBy === field && filters.sortOrder === 'desc' ? 'asc' : 'desc';
    actions.setVerdictFilters({ sortBy: field, sortOrder: newDirection });
  }, [filters.sortBy, filters.sortOrder, actions]);

  // Handle verdict selection
  const handleVerdictSelect = useCallback((verdict: any) => {
    actions.setSelectedVerdict(verdict);
  }, [actions]);

  // Handle verdict update (for real-time updates)
  const handleVerdictUpdate = useCallback((verdictId: string, updates: any) => {
    actions.updateVerdict(verdictId, updates);
  }, [actions]);

  // Initial data load
  useEffect(() => {
    fetchVerdicts();
  }, []); // Only run once on mount

  return {
    // Data
    verdicts,
    paginatedVerdicts,
    selectedVerdict,
    loading,
    error,
    filters,
    pagination,
    uiState,

    // Actions
    fetchVerdicts,
    handlePageChange,
    handleSortChange,
    handleVerdictSelect,
    handleVerdictUpdate,

    // Computed values
    startIndex: (pagination.currentPage - 1) * pagination.pageSize,
    endIndex: Math.min(
      (pagination.currentPage - 1) * pagination.pageSize + pagination.pageSize,
      uiState.filteredCount
    ),
  };
}

