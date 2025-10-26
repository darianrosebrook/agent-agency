/**
 * Council API Integration Tests
 * Tests the integration between API client and backend routes
 *
 * @author @darianrosebrook
 */

import { councilApiClient } from '../council-api';
import { ApiClient } from '../api-client';

// Mock the ApiClient
jest.mock('../api-client', () => ({
  ApiClient: jest.fn().mockImplementation(() => ({
    request: jest.fn(),
  })),
}));

const mockApiClient = new ApiClient({ baseUrl: '/api/council' });

describe('Council API Client Integration', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    (ApiClient as jest.Mock).mockClear();
  });

  describe('getVerdicts', () => {
    it('calls API with correct parameters for basic request', async () => {
      const mockResponse = {
        verdicts: [],
        total: 0,
        page: 1,
        limit: 20,
        hasMore: false
      };

      (mockApiClient.request as jest.Mock).mockResolvedValue(mockResponse);

      const result = await councilApiClient.getVerdicts();

      expect(mockApiClient.request).toHaveBeenCalledWith('/verdicts?page=1&limit=20');
      expect(result).toEqual(mockResponse);
    });

    it('includes filters in API call', async () => {
      const filters = {
        status: ['pending' as const, 'approved' as const],
        judgeId: 'judge-123',
        riskLevel: ['high' as const],
        dateRange: {
          start: new Date('2024-01-01'),
          end: new Date('2024-01-31')
        },
        category: 'security'
      };

      const mockResponse = {
        verdicts: [],
        total: 0,
        page: 1,
        limit: 20,
        hasMore: false
      };

      (mockApiClient.request as jest.Mock).mockResolvedValue(mockResponse);

      await councilApiClient.getVerdicts(filters, 2, 10);

      const expectedUrl = '/verdicts?page=2&limit=10&status=pending,approved&judgeId=judge-123&riskLevel=high&startDate=2024-01-01T00:00:00.000Z&endDate=2024-01-31T00:00:00.000Z&category=security';
      expect(mockApiClient.request).toHaveBeenCalledWith(expectedUrl);
    });

    it('handles API errors gracefully', async () => {
      (mockApiClient.request as jest.Mock).mockRejectedValue(new Error('API Error'));

      await expect(councilApiClient.getVerdicts()).rejects.toThrow('API Error');
    });
  });

  describe('getVerdict', () => {
    it('calls API with correct verdict ID', async () => {
      const mockVerdict = { id: 'verdict-123', taskId: 'task-456' };
      (mockApiClient.request as jest.Mock).mockResolvedValue(mockVerdict);

      const result = await councilApiClient.getVerdict('verdict-123');

      expect(mockApiClient.request).toHaveBeenCalledWith('/verdicts/verdict-123');
      expect(result).toEqual(mockVerdict);
    });
  });

  describe('getVerdictEvidence', () => {
    it('calls API for verdict evidence', async () => {
      const mockEvidence = [{ id: 'evidence-1', type: 'document' }];
      (mockApiClient.request as jest.Mock).mockResolvedValue(mockEvidence);

      const result = await councilApiClient.getVerdictEvidence('verdict-123');

      expect(mockApiClient.request).toHaveBeenCalledWith('/verdicts/verdict-123/evidence');
      expect(result).toEqual(mockEvidence);
    });
  });

  describe('overrideVerdict', () => {
    it('calls API with correct override parameters', async () => {
      const overrideData = {
        decision: 'reject' as const,
        reason: 'Security concern',
        operator: 'admin-user'
      };

      const mockResponse = { id: 'verdict-123', status: 'intervened' };
      (mockApiClient.request as jest.Mock).mockResolvedValue(mockResponse);

      const result = await councilApiClient.overrideVerdict('verdict-123', overrideData);

      expect(mockApiClient.request).toHaveBeenCalledWith('/verdicts/verdict-123/override', {
        method: 'POST',
        body: JSON.stringify(overrideData)
      });
      expect(result).toEqual(mockResponse);
    });
  });

  describe('getJudges', () => {
    it('calls API for judge list', async () => {
      const mockJudges = [{ id: 'judge-1', name: 'Judge One' }];
      (mockApiClient.request as jest.Mock).mockResolvedValue(mockJudges);

      const result = await councilApiClient.getJudges();

      expect(mockApiClient.request).toHaveBeenCalledWith('/judges');
      expect(result).toEqual(mockJudges);
    });
  });

  describe('getMetrics', () => {
    it('calls API for council metrics', async () => {
      const mockMetrics = {
        totalVerdicts: 100,
        activeVerdicts: 5,
        averageResponseTime: 2.3
      };
      (mockApiClient.request as jest.Mock).mockResolvedValue(mockMetrics);

      const result = await councilApiClient.getMetrics();

      expect(mockApiClient.request).toHaveBeenCalledWith('/metrics');
      expect(result).toEqual(mockMetrics);
    });
  });

  describe('getAlerts', () => {
    it('calls API with default parameters', async () => {
      const mockAlerts = [{ id: 'alert-1', type: 'ethical_concern' }];
      (mockApiClient.request as jest.Mock).mockResolvedValue(mockAlerts);

      const result = await councilApiClient.getAlerts();

      expect(mockApiClient.request).toHaveBeenCalledWith('/alerts?acknowledged=false&limit=50');
      expect(result).toEqual(mockAlerts);
    });

    it('calls API with custom parameters', async () => {
      const mockAlerts = [{ id: 'alert-1', type: 'judge_failure' }];
      (mockApiClient.request as jest.Mock).mockResolvedValue(mockAlerts);

      const result = await councilApiClient.getAlerts(true, 25);

      expect(mockApiClient.request).toHaveBeenCalledWith('/alerts?acknowledged=true&limit=25');
      expect(result).toEqual(mockAlerts);
    });
  });

  describe('acknowledgeAlert', () => {
    it('calls API to acknowledge alert', async () => {
      (mockApiClient.request as jest.Mock).mockResolvedValue(undefined);

      await councilApiClient.acknowledgeAlert('alert-123');

      expect(mockApiClient.request).toHaveBeenCalledWith('/alerts/alert-123/acknowledge', {
        method: 'POST'
      });
    });
  });

  describe('getEthicalAssessments', () => {
    it('calls API with verdict ID filter', async () => {
      const mockAssessments = [{ id: 'assessment-1', verdictId: 'verdict-123' }];
      (mockApiClient.request as jest.Mock).mockResolvedValue(mockAssessments);

      const result = await councilApiClient.getEthicalAssessments('verdict-123');

      expect(mockApiClient.request).toHaveBeenCalledWith('/ethical-assessments?verdictId=verdict-123&limit=20');
      expect(result).toEqual(mockAssessments);
    });

    it('calls API with risk level filter', async () => {
      const mockAssessments = [{ id: 'assessment-1', overallRisk: 'high' }];
      (mockApiClient.request as jest.Mock).mockResolvedValue(mockAssessments);

      const result = await councilApiClient.getEthicalAssessments(undefined, 'high' as any);

      expect(mockApiClient.request).toHaveBeenCalledWith('/ethical-assessments?riskLevel=high&limit=20');
      expect(result).toEqual(mockAssessments);
    });
  });

  describe('getVerdictAnalytics', () => {
    it('calls API with correct analytics parameters', async () => {
      const period = {
        start: new Date('2024-01-01'),
        end: new Date('2024-01-31')
      };

      const mockAnalytics = {
        timeline: [],
        judgePerformance: [],
        ethicalTrends: []
      };

      (mockApiClient.request as jest.Mock).mockResolvedValue(mockAnalytics);

      const result = await councilApiClient.getVerdictAnalytics(period, 'day');

      const expectedUrl = `/analytics?startDate=${period.start.toISOString()}&endDate=${period.end.toISOString()}&granularity=day`;
      expect(mockApiClient.request).toHaveBeenCalledWith(expectedUrl);
      expect(result).toEqual(mockAnalytics);
    });
  });
});

