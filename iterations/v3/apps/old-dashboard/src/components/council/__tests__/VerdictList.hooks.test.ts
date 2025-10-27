/**
 * VerdictList Hooks Unit Tests
 * Tests the extracted logic from VerdictList component
 *
 * @author @darianrosebrook
 */

import { renderHook, act } from '@testing-library/react';
import { useVerdictList } from '../VerdictList.hooks';
import { councilApiClient } from '@/lib/council-api';

// Mock the API client
jest.mock('@/lib/council-api', () => ({
  councilApiClient: {
    getVerdicts: jest.fn(),
  },
}));

// Mock the store
jest.mock('@/stores/council', () => ({
  useCouncilStore: jest.fn(),
}));

// Mock the error handler
jest.mock('@/hooks/useErrorHandler', () => ({
  useErrorHandler: jest.fn(),
}));

const mockVerdicts = [
  {
    id: 'verdict-1',
    taskId: 'task-123',
    status: 'pending' as const,
    judges: [{ judgeId: 'judge-1', role: 'primary' as const, assignedAt: new Date(), status: 'completed' as const }],
    consensus: { algorithm: 'majority' as const, confidence: 0.85, participatingJudges: 1, agreementLevel: 1.0, finalDecision: 'approve' as const, rationale: 'Majority approval' },
    ethicalAssessment: { id: 'ethical-1', verdictId: 'verdict-1', overallRisk: 'low' as const, concerns: [], stakeholderImpact: { individuals: 0, organizations: 0, society: 0 }, recommendations: [], assessedAt: new Date() },
    evidence: [],
    createdAt: new Date(),
    updatedAt: new Date()
  }
];

describe('useVerdictList Hook', () => {
  const mockUseCouncilStore = require('@/stores/council').useCouncilStore;
  const mockHandleError = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();

    // Mock the store selectors
    mockUseCouncilStore.mockImplementation((selector: any) => {
      if (selector.name === 'selectVerdictListData') {
        return {
          verdicts: mockVerdicts,
          filters: { status: [], sortBy: 'createdAt', sortOrder: 'desc' },
          pagination: { currentPage: 1, pageSize: 20, total: 1, hasMore: false },
          loading: false,
          error: null,
          selectedVerdict: null
        };
      }
      if (selector.name === 'selectVerdictListActions') {
        return {
          setLoading: jest.fn(),
          setError: jest.fn(),
          setPagination: jest.fn(),
          setCurrentPage: jest.fn(),
        };
      }
      if (selector.name === 'selectPaginatedVerdicts') {
        return mockVerdicts;
      }
      if (selector.name === 'selectVerdictListUIState') {
        return {
          showFilters: false,
          sortBy: 'createdAt',
          sortOrder: 'desc',
          currentPage: 1,
          totalPages: 1,
          totalVerdicts: 1,
          filteredCount: 1,
        };
      }
      return {};
    });

    // Mock API response
    (councilApiClient.getVerdicts as jest.Mock).mockResolvedValue({
      verdicts: mockVerdicts,
      total: 1,
      hasMore: false
    });

    // Mock error handler
    const mockUseErrorHandler = require('@/hooks/useErrorHandler').useErrorHandler;
    mockUseErrorHandler.mockReturnValue({
      handleError: mockHandleError
    });
  });

  it('returns correct initial data structure', () => {
    const { result } = renderHook(() => useVerdictList());

    expect(result.current).toHaveProperty('paginatedVerdicts');
    expect(result.current).toHaveProperty('selectedVerdict');
    expect(result.current).toHaveProperty('loading');
    expect(result.current).toHaveProperty('error');
    expect(result.current).toHaveProperty('uiState');
    expect(result.current).toHaveProperty('handlePageChange');
    expect(result.current).toHaveProperty('handleSortChange');
    expect(result.current).toHaveProperty('handleVerdictSelect');
  });

  it('provides correct pagination info', () => {
    const { result } = renderHook(() => useVerdictList());

    expect(result.current.startIndex).toBe(0);
    expect(result.current.endIndex).toBe(1);
  });

  it('handles verdict selection', () => {
    const mockSetSelectedVerdict = jest.fn();
    mockUseCouncilStore.mockImplementation((selector: any) => {
      if (selector.name === 'selectVerdictListActions') {
        return {
          setLoading: jest.fn(),
          setError: jest.fn(),
          setPagination: jest.fn(),
          setCurrentPage: jest.fn(),
          setSelectedVerdict: mockSetSelectedVerdict,
        };
      }
      return {};
    });

    const { result } = renderHook(() => useVerdictList());

    act(() => {
      result.current.handleVerdictSelect(mockVerdicts[0]);
    });

    expect(mockSetSelectedVerdict).toHaveBeenCalledWith(mockVerdicts[0]);
  });

  it('handles page changes', () => {
    const mockSetCurrentPage = jest.fn();
    mockUseCouncilStore.mockImplementation((selector: any) => {
      if (selector.name === 'selectVerdictListActions') {
        return {
          setLoading: jest.fn(),
          setError: jest.fn(),
          setPagination: jest.fn(),
          setCurrentPage: mockSetCurrentPage,
        };
      }
      if (selector.name === 'selectVerdictListData') {
        return {
          verdicts: mockVerdicts,
          filters: { status: [], sortBy: 'createdAt', sortOrder: 'desc' },
          pagination: { currentPage: 1, pageSize: 20, total: 1, hasMore: false },
          loading: false,
          error: null,
          selectedVerdict: null
        };
      }
      return {};
    });

    const { result } = renderHook(() => useVerdictList());

    act(() => {
      result.current.handlePageChange(2);
    });

    expect(mockSetCurrentPage).toHaveBeenCalledWith(2);
  });

  it('handles sort changes', () => {
    const mockUpdateFilters = jest.fn();
    mockUseCouncilStore.mockImplementation((selector: any) => {
      if (selector.name === 'selectVerdictListActions') {
        return {
          setLoading: jest.fn(),
          setError: jest.fn(),
          setPagination: jest.fn(),
          setCurrentPage: jest.fn(),
          updateFilters: mockUpdateFilters,
        };
      }
      if (selector.name === 'selectVerdictListData') {
        return {
          verdicts: mockVerdicts,
          filters: { status: [], sortBy: 'createdAt', sortOrder: 'desc' },
          pagination: { currentPage: 1, pageSize: 20, total: 1, hasMore: false },
          loading: false,
          error: null,
          selectedVerdict: null
        };
      }
      return {};
    });

    const { result } = renderHook(() => useVerdictList());

    act(() => {
      result.current.handleSortChange('updatedAt');
    });

    expect(mockUpdateFilters).toHaveBeenCalledWith({
      sortBy: 'updatedAt',
      sortOrder: 'asc'
    });
  });

  it('handles API errors gracefully', async () => {
    (councilApiClient.getVerdicts as jest.Mock).mockRejectedValue(new Error('API Error'));

    const mockSetError = jest.fn();
    mockUseCouncilStore.mockImplementation((selector: any) => {
      if (selector.name === 'selectVerdictListActions') {
        return {
          setLoading: jest.fn(),
          setError: mockSetError,
        };
      }
      return {};
    });

    renderHook(() => useVerdictList());

    // Wait for useEffect to run
    await new Promise(resolve => setTimeout(resolve, 0));

    expect(mockHandleError).toHaveBeenCalled();
    expect(mockSetError).toHaveBeenCalled();
  });
});
