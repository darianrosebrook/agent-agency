/**
 * Council Store Selectors Unit Tests
 * Tests the performance-optimized selectors for the Council store
 *
 * @author @darianrosebrook
 */

import {
  selectVerdictListData,
  selectFilteredVerdicts,
  selectSortedVerdicts,
  selectPaginatedVerdicts,
  selectVerdictListUIState,
  selectActiveAlerts,
  selectAlertStats,
} from '../council-selectors';

const mockVerdicts = [
  {
    id: 'verdict-1',
    taskId: 'task-123',
    status: 'pending' as const,
    judges: [
      { judgeId: 'judge-1', role: 'primary' as const, assignedAt: new Date(), status: 'completed' as const }
    ],
    consensus: {
      algorithm: 'majority' as const,
      confidence: 0.85,
      participatingJudges: 1,
      agreementLevel: 1.0,
      finalDecision: 'approve' as const,
      rationale: 'Majority approval'
    },
    ethicalAssessment: {
      id: 'ethical-1',
      verdictId: 'verdict-1',
      overallRisk: 'low' as const,
      concerns: [],
      stakeholderImpact: { individuals: 0, organizations: 0, society: 0 },
      recommendations: [],
      assessedAt: new Date()
    },
    evidence: [],
    createdAt: new Date('2024-01-01'),
    updatedAt: new Date('2024-01-02')
  },
  {
    id: 'verdict-2',
    taskId: 'task-456',
    status: 'approved' as const,
    judges: [
      { judgeId: 'judge-2', role: 'primary' as const, assignedAt: new Date(), status: 'completed' as const }
    ],
    consensus: {
      algorithm: 'majority' as const,
      confidence: 0.95,
      participatingJudges: 1,
      agreementLevel: 1.0,
      finalDecision: 'approve' as const,
      rationale: 'Strong consensus'
    },
    ethicalAssessment: {
      id: 'ethical-2',
      verdictId: 'verdict-2',
      overallRisk: 'medium' as const,
      concerns: [{ type: 'bias', severity: 'medium', description: 'Potential bias detected' }],
      stakeholderImpact: { individuals: 1, organizations: 0, society: 0 },
      recommendations: ['Review bias mitigation'],
      assessedAt: new Date()
    },
    evidence: [],
    createdAt: new Date('2024-01-02'),
    updatedAt: new Date('2024-01-03')
  }
];

const mockAlerts = [
  { id: 'alert-1', type: 'ethical_concern', acknowledged: false, severity: 'high' as const },
  { id: 'alert-2', type: 'judge_failure', acknowledged: true, severity: 'medium' as const },
  { id: 'alert-3', type: 'system_issue', acknowledged: false, severity: 'low' as const },
];

const mockState = {
  verdicts: mockVerdicts,
  judges: [],
  metrics: null,
  alerts: mockAlerts,
  selectedVerdict: null,
  selectedJudge: null,
  verdictFilters: {
    status: [],
    sortBy: 'createdAt' as const,
    sortOrder: 'desc' as const,
    judgeCount: undefined,
    consensusScore: undefined,
    ethicalConcerns: undefined,
    search: undefined,
    dateRange: undefined,
  },
  pagination: {
    currentPage: 1,
    pageSize: 20,
    total: 2,
    hasMore: false,
  },
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
  realTimeEnabled: true,
  lastUpdate: new Date(),
};

describe('Council Store Selectors', () => {
  describe('selectVerdictListData', () => {
    it('returns correctly structured data for VerdictList component', () => {
      const result = selectVerdictListData(mockState);

      expect(result).toHaveProperty('verdicts');
      expect(result).toHaveProperty('filters');
      expect(result).toHaveProperty('pagination');
      expect(result).toHaveProperty('loading');
      expect(result).toHaveProperty('error');
      expect(result).toHaveProperty('selectedVerdict');

      expect(result.verdicts).toBe(mockState.verdicts);
      expect(result.filters).toBe(mockState.verdictFilters);
      expect(result.loading).toBe(false);
      expect(result.error).toBe(null);
    });
  });

  describe('selectFilteredVerdicts', () => {
    it('filters verdicts by status', () => {
      const stateWithFilter = {
        ...mockState,
        verdictFilters: {
          ...mockState.verdictFilters,
          status: ['approved']
        }
      };

      const result = selectFilteredVerdicts(stateWithFilter);

      expect(result).toHaveLength(1);
      expect(result[0].status).toBe('approved');
      expect(result[0].id).toBe('verdict-2');
    });

    it('filters verdicts by judge count', () => {
      const stateWithFilter = {
        ...mockState,
        verdictFilters: {
          ...mockState.verdictFilters,
          judgeCount: 2
        }
      };

      const result = selectFilteredVerdicts(stateWithFilter);

      expect(result).toHaveLength(0); // No verdicts have 2+ judges
    });

    it('filters verdicts by consensus score', () => {
      const stateWithFilter = {
        ...mockState,
        verdictFilters: {
          ...mockState.verdictFilters,
          consensusScore: { min: 0.9, max: 1.0 }
        }
      };

      const result = selectFilteredVerdicts(stateWithFilter);

      expect(result).toHaveLength(1);
      expect(result[0].consensus.confidence).toBeGreaterThanOrEqual(0.9);
    });

    it('filters verdicts by ethical concerns', () => {
      const stateWithFilter = {
        ...mockState,
        verdictFilters: {
          ...mockState.verdictFilters,
          ethicalConcerns: 1
        }
      };

      const result = selectFilteredVerdicts(stateWithFilter);

      expect(result).toHaveLength(1);
      expect(result[0].ethicalAssessment.concerns).toHaveLength(1);
    });

    it('filters verdicts by search term', () => {
      const stateWithFilter = {
        ...mockState,
        verdictFilters: {
          ...mockState.verdictFilters,
          search: 'task-123'
        }
      };

      const result = selectFilteredVerdicts(stateWithFilter);

      expect(result).toHaveLength(1);
      expect(result[0].taskId).toBe('task-123');
    });
  });

  describe('selectSortedVerdicts', () => {
    it('sorts verdicts by createdAt descending (default)', () => {
      const result = selectSortedVerdicts(mockState);

      expect(result[0].createdAt.getTime()).toBeGreaterThan(result[1].createdAt.getTime());
    });

    it('sorts verdicts by consensus score ascending', () => {
      const stateWithSort = {
        ...mockState,
        verdictFilters: {
          ...mockState.verdictFilters,
          sortBy: 'consensusScore' as const,
          sortOrder: 'asc' as const
        }
      };

      const result = selectSortedVerdicts(stateWithSort);

      expect(result[0].consensus.confidence).toBeLessThanOrEqual(result[1].consensus.confidence);
    });

    it('sorts verdicts by ethical concerns descending', () => {
      const stateWithSort = {
        ...mockState,
        verdictFilters: {
          ...mockState.verdictFilters,
          sortBy: 'ethicalConcerns' as const,
          sortOrder: 'desc' as const
        }
      };

      const result = selectSortedVerdicts(stateWithSort);

      expect(result[0].ethicalAssessment.concerns.length).toBeGreaterThanOrEqual(result[1].ethicalAssessment.concerns.length);
    });
  });

  describe('selectPaginatedVerdicts', () => {
    it('returns paginated results', () => {
      const result = selectPaginatedVerdicts(mockState);

      expect(result).toHaveLength(2); // Both verdicts fit on first page
    });

    it('respects pagination limits', () => {
      const stateWithPagination = {
        ...mockState,
        pagination: {
          ...mockState.pagination,
          pageSize: 1
        }
      };

      const result = selectPaginatedVerdicts(stateWithPagination);

      expect(result).toHaveLength(1); // Only one verdict per page
    });
  });

  describe('selectVerdictListUIState', () => {
    it('calculates UI state correctly', () => {
      const result = selectVerdictListUIState(mockState);

      expect(result).toHaveProperty('currentPage', 1);
      expect(result).toHaveProperty('totalPages', 1);
      expect(result).toHaveProperty('totalVerdicts', 2);
      expect(result).toHaveProperty('filteredCount', 2);
      expect(result).toHaveProperty('sortBy', 'createdAt');
      expect(result).toHaveProperty('sortOrder', 'desc');
    });
  });

  describe('selectActiveAlerts', () => {
    it('returns only unacknowledged alerts', () => {
      const result = selectActiveAlerts(mockState);

      expect(result).toHaveLength(2);
      expect(result.every(alert => !alert.acknowledged)).toBe(true);
    });
  });

  describe('selectAlertStats', () => {
    it('calculates alert statistics correctly', () => {
      const result = selectAlertStats(mockState);

      expect(result.total).toBe(3);
      expect(result.active).toBe(2);
      expect(result.critical).toBe(0); // No critical alerts in mock
      expect(result.high).toBe(1);
      expect(result.medium).toBe(1);
      expect(result.low).toBe(1);
    });
  });
});
