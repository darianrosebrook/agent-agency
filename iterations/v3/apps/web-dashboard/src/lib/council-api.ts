/**
 * Council API Client
 * Handles all API interactions for council verdict management, judge monitoring, and ethical assessments
 *
 * @author @darianrosebrook
 */

import { ApiClient } from './api-client';

// Re-export types from components for convenience
export type { Verdict, VerdictStatus, Judge, Evidence } from '@/components/council/VerdictList';
export type { InterventionRequest } from '@/components/council/InterventionForm';

// Council-specific types
export interface VerdictListResponse {
  verdicts: Verdict[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

export interface VerdictFilters {
  status?: VerdictStatus[];
  judgeCount?: number;
  consensusScore?: { min: number; max: number };
  ethicalConcerns?: number;
  dateRange?: { start: Date; end: Date };
  search?: string;
  sortBy?: 'createdAt' | 'updatedAt' | 'consensusScore' | 'ethicalConcerns';
  sortOrder?: 'asc' | 'desc';
}

export interface JudgePerformance {
  id: string;
  name: string;
  totalVerdicts: number;
  accuracy: number;
  averageResponseTime: number;
  consensusRate: number;
  ethicalConcernsFlagged: number;
  lastActive: Date;
  status: 'active' | 'idle' | 'error';
  performanceHistory: PerformancePoint[];
}

export interface PerformancePoint {
  timestamp: Date;
  accuracy: number;
  responseTime: number;
  consensusRate: number;
}

export interface EthicalAssessment {
  id: string;
  verdictId: string;
  concerns: EthicalConcern[];
  overallRisk: 'low' | 'medium' | 'high' | 'critical';
  assessmentDate: Date;
  reviewedBy?: string;
  reviewDate?: Date;
  mitigationStrategies?: string[];
}

export interface EthicalConcern {
  id: string;
  category: 'privacy' | 'bias' | 'safety' | 'transparency' | 'fairness' | 'accountability';
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  evidence: string[];
  mitigation: string;
  status: 'identified' | 'mitigated' | 'accepted';
}

export interface CouncilStats {
  totalVerdicts: number;
  pendingVerdicts: number;
  completedVerdicts: number;
  intervenedVerdicts: number;
  activeJudges: number;
  totalJudges: number;
  averageConsensus: number;
  ethicalConcernsCount: number;
  averageResolutionTime: number;
}

export interface VerdictIntervention {
  id: string;
  verdictId: string;
  reason: string;
  priority: 'low' | 'medium' | 'high' | 'critical';
  requestedBy: string;
  requestedAt: Date;
  reviewDeadline: Date;
  status: 'pending' | 'approved' | 'rejected' | 'escalated';
  reviewedBy?: string;
  reviewedAt?: Date;
  decision?: 'approve' | 'reject';
  justification?: string;
  notes?: string;
}

// Import types from components
type Verdict = import('@/components/council/VerdictList').Verdict;
type VerdictStatus = import('@/components/council/VerdictList').VerdictStatus;
type Judge = import('@/components/council/VerdictList').Judge;
type Evidence = import('@/components/council/VerdictList').Evidence;
type InterventionRequest = import('@/components/council/InterventionForm').InterventionRequest;

/**
 * Council API Client Class
 * Provides methods for all council-related API operations
 */
export class CouncilApiClient {
  private apiClient: ApiClient;
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl ?? '/api/council';
    this.apiClient = new ApiClient({
      baseUrl: this.baseUrl,
      timeout: 30000, // 30 second timeout for council operations
    });
  }

  // ===== VERDICT MANAGEMENT =====

  /**
   * Get verdicts with filtering, sorting, and pagination
   */
  async getVerdicts(
    filters: VerdictFilters = {},
    page = 1,
    pageSize = 20
  ): Promise<VerdictListResponse> {
    const params = new URLSearchParams({
      page: page.toString(),
      pageSize: pageSize.toString(),
    });

    // Add filters
    if (filters.status?.length) {
      filters.status.forEach(status => params.append('status', status));
    }
    if (filters.judgeCount) params.append('judgeCount', filters.judgeCount.toString());
    if (filters.consensusScore) {
      params.append('consensusMin', filters.consensusScore.min.toString());
      params.append('consensusMax', filters.consensusScore.max.toString());
    }
    if (filters.ethicalConcerns !== undefined) {
      params.append('ethicalConcerns', filters.ethicalConcerns.toString());
    }
    if (filters.dateRange) {
      params.append('startDate', filters.dateRange.start.toISOString());
      params.append('endDate', filters.dateRange.end.toISOString());
    }
    if (filters.search) params.append('search', filters.search);
    if (filters.sortBy) params.append('sortBy', filters.sortBy);
    if (filters.sortOrder) params.append('sortOrder', filters.sortOrder);

    const response = await this.apiClient.request<VerdictListResponse>(
      `/verdicts?${params.toString()}`
    );

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to fetch verdicts');
  }

  /**
   * Get a specific verdict by ID
   */
  async getVerdict(id: string): Promise<Verdict> {
    const response = await this.apiClient.request<Verdict>(`/verdicts/${id}`);

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to fetch verdict');
  }

  /**
   * Get evidence for a specific verdict
   */
  async getVerdictEvidence(id: string): Promise<Evidence[]> {
    const response = await this.apiClient.request<Evidence[]>(`/verdicts/${id}/evidence`);

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to fetch verdict evidence');
  }

  /**
   * Override a verdict decision
   */
  async overrideVerdict(
    id: string,
    decision: 'approve' | 'reject',
    justification: string,
    notes?: string
  ): Promise<Verdict> {
    const response = await this.apiClient.request<Verdict>(`/verdicts/${id}/override`, {
      method: 'POST',
      body: JSON.stringify({
        decision,
        justification,
        notes,
        overrideTimestamp: new Date().toISOString(),
      }),
    });

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to override verdict');
  }

  // ===== JUDGE MANAGEMENT =====

  /**
   * Get all judges with their current status
   */
  async getJudges(): Promise<Judge[]> {
    const response = await this.apiClient.request<Judge[]>('/judges');

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to fetch judges');
  }

  /**
   * Get detailed performance metrics for a specific judge
   */
  async getJudgePerformance(id: string): Promise<JudgePerformance> {
    const response = await this.apiClient.request<JudgePerformance>(`/judges/${id}/performance`);

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to fetch judge performance');
  }

  /**
   * Get aggregated judge metrics for the dashboard
   */
  async getJudgeMetrics(): Promise<{
    judges: JudgePerformance[];
    systemMetrics: {
      totalJudges: number;
      activeJudges: number;
      averageAccuracy: number;
      averageResponseTime: number;
      totalVerdictsToday: number;
      consensusRate: number;
    };
  }> {
    const response = await this.apiClient.request<{
      judges: JudgePerformance[];
      systemMetrics: any;
    }>('/judges/metrics');

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to fetch judge metrics');
  }

  // ===== ETHICAL ASSESSMENTS =====

  /**
   * Get all ethical assessments
   */
  async getEthicalAssessments(
    status?: 'pending' | 'reviewed',
    severity?: 'low' | 'medium' | 'high' | 'critical',
    limit = 50
  ): Promise<EthicalAssessment[]> {
    const params = new URLSearchParams();
    if (status) params.append('status', status);
    if (severity) params.append('severity', severity);
    params.append('limit', limit.toString());

    const response = await this.apiClient.request<EthicalAssessment[]>(
      `/ethical/assessments?${params.toString()}`
    );

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to fetch ethical assessments');
  }

  /**
   * Review an ethical assessment
   */
  async reviewEthicalAssessment(
    id: string,
    review: {
      status: 'approved' | 'requires_action' | 'escalated';
      reviewerNotes: string;
      mitigationStrategies?: string[];
      followUpRequired?: boolean;
    }
  ): Promise<EthicalAssessment> {
    const response = await this.apiClient.request<EthicalAssessment>(
      `/ethical/assessments/${id}/review`,
      {
        method: 'POST',
        body: JSON.stringify({
          ...review,
          reviewDate: new Date().toISOString(),
        }),
      }
    );

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to review ethical assessment');
  }

  // ===== INTERVENTIONS =====

  /**
   * Request manual intervention for a verdict
   */
  async requestIntervention(
    verdictId: string,
    intervention: InterventionRequest
  ): Promise<VerdictIntervention> {
    const response = await this.apiClient.request<VerdictIntervention>(
      `/verdicts/${verdictId}/intervention`,
      {
        method: 'POST',
        body: JSON.stringify({
          ...intervention,
          verdictId,
          requestedAt: new Date().toISOString(),
        }),
      }
    );

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to request intervention');
  }

  /**
   * Get intervention requests
   */
  async getInterventions(
    status?: 'pending' | 'approved' | 'rejected' | 'escalated',
    priority?: 'low' | 'medium' | 'high' | 'critical'
  ): Promise<VerdictIntervention[]> {
    const params = new URLSearchParams();
    if (status) params.append('status', status);
    if (priority) params.append('priority', priority);

    const response = await this.apiClient.request<VerdictIntervention[]>(
      `/interventions?${params.toString()}`
    );

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to fetch interventions');
  }

  // ===== DASHBOARD STATS =====

  /**
   * Get council dashboard statistics
   */
  async getCouncilStats(): Promise<CouncilStats> {
    const response = await this.apiClient.request<CouncilStats>('/stats');

    if (response.success) {
      return response.data;
    }
    throw new Error(response.error?.message || 'Failed to fetch council stats');
  }

  // ===== STREAMING/REAL-TIME =====

  /**
   * Get Server-Sent Events stream for real-time verdict updates
   */
  getVerdictStream(): EventSource {
    const eventSource = new EventSource(`${this.baseUrl}/verdicts/stream`);

    eventSource.onerror = (error) => {
      console.error('Verdict stream error:', error);
      // Auto-reconnect logic could be added here
    };

    return eventSource;
  }

  /**
   * Get Server-Sent Events stream for real-time judge metrics
   */
  getJudgeMetricsStream(): EventSource {
    return new EventSource(`${this.baseUrl}/judges/metrics/stream`);
  }

  /**
   * Get Server-Sent Events stream for ethical assessment updates
   */
  getEthicalAssessmentStream(): EventSource {
    return new EventSource(`${this.baseUrl}/ethical/stream`);
  }
}

// Export singleton instance
export const councilApiClient = new CouncilApiClient();
