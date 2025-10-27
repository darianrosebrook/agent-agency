/**
 * Council API Client
 * API client for council oversight and decision management
 *
 * @author @darianrosebrook
 */

import { ApiClient } from './api-client';

export interface Judge {
  id: string;
  name: string;
  role: 'primary' | 'secondary' | 'ethical' | 'domain_expert';
  model: string;
  status: 'active' | 'inactive' | 'error';
  performance: {
    accuracy: number;
    responseTime: number;
    consensusRate: number;
    biasScore: number;
  };
  lastActive: Date;
}

export interface JudgeAssignment {
  judgeId: string;
  role: Judge['role'];
  assignedAt: Date;
  status: 'pending' | 'completed' | 'failed';
  verdict?: JudgeVerdict;
}

export interface JudgeVerdict {
  judgeId: string;
  decision: 'approve' | 'reject' | 'escalate';
  confidence: number;
  rationale: string;
  timestamp: Date;
  ethicalConcerns?: EthicalConcern[];
}

export interface EthicalConcern {
  id: string;
  category: 'privacy' | 'bias' | 'safety' | 'fairness' | 'transparency' | 'accountability';
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  affectedParties: string[];
  mitigation?: string;
  resolved: boolean;
  createdAt: Date;
}

export interface EthicalAssessment {
  id: string;
  verdictId: string;
  overallRisk: 'low' | 'medium' | 'high' | 'critical';
  concerns: EthicalConcern[];
  stakeholderImpact: {
    individuals: number;
    organizations: number;
    society: number;
  };
  recommendations: string[];
  assessedAt: Date;
}

export interface Evidence {
  id: string;
  type: 'document' | 'data' | 'log' | 'metric' | 'model_output';
  title: string;
  content: string;
  source: string;
  confidence: number;
  timestamp: Date;
}

export interface ConsensusResult {
  algorithm: 'majority' | 'weighted' | 'supervisory' | 'ethical_override';
  confidence: number;
  participatingJudges: number;
  agreementLevel: number;
  finalDecision: 'approve' | 'reject' | 'escalate';
  rationale: string;
}

export interface Verdict {
  id: string;
  taskId: string;
  status: 'pending' | 'in_progress' | 'completed' | 'overridden' | 'escalated';
  judges: JudgeAssignment[];
  consensus: ConsensusResult;
  ethicalAssessment: EthicalAssessment;
  evidence: Evidence[];
  intervention?: {
    type: 'manual_override' | 'escalation' | 'pause';
    reason: string;
    operator: string;
    timestamp: Date;
  };
  createdAt: Date;
  completedAt?: Date;
  updatedAt: Date;
}

export interface CouncilMetrics {
  totalVerdicts: number;
  activeVerdicts: number;
  pendingInterventions: number;
  averageResponseTime: number;
  ethicalConcernRate: number;
  consensusAccuracy: number;
  judgePerformance: {
    [judgeId: string]: Judge['performance'];
  };
  recentActivity: {
    timestamp: Date;
    action: string;
    verdictId: string;
  }[];
}

export interface CouncilAlert {
  id: string;
  type: 'ethical_concern' | 'judge_failure' | 'consensus_failure' | 'performance_degradation';
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
  verdictId?: string;
  judgeId?: string;
  createdAt: Date;
  acknowledged: boolean;
}

export interface VerdictFilter {
  status?: Verdict['status'][];
  judgeId?: string;
  riskLevel?: EthicalAssessment['overallRisk'][];
  dateRange?: {
    start: Date;
    end: Date;
  };
  category?: string;
}

export interface JudgePerformanceReport {
  judgeId: string;
  period: {
    start: Date;
    end: Date;
  };
  metrics: {
    verdictsParticipated: number;
    accuracy: number;
    responseTime: {
      average: number;
      p95: number;
      p99: number;
    };
    consensusRate: number;
    biasScore: number;
    errorRate: number;
  };
  trends: {
    accuracy: number; // percentage change
    responseTime: number; // percentage change
    consensusRate: number; // percentage change
  };
}

export class CouncilApiClient {
  private apiClient: ApiClient;

  constructor(baseUrl: string = '/api/council') {
    this.apiClient = new ApiClient({ baseUrl });
  }

  /**
   * Get verdicts with filtering and pagination
   */
  async getVerdicts(
    filters?: VerdictFilter,
    page: number = 1,
    limit: number = 20
  ): Promise<{
    verdicts: Verdict[];
    total: number;
    page: number;
    limit: number;
  }> {
    const params = new URLSearchParams({
      page: page.toString(),
      limit: limit.toString(),
    });

    if (filters) {
      if (filters.status) params.append('status', filters.status.join(','));
      if (filters.judgeId) params.append('judgeId', filters.judgeId);
      if (filters.riskLevel) params.append('riskLevel', filters.riskLevel.join(','));
      if (filters.category) params.append('category', filters.category);
      if (filters.dateRange) {
        params.append('startDate', filters.dateRange.start.toISOString());
        params.append('endDate', filters.dateRange.end.toISOString());
      }
    }

    const response = await this.apiClient.request<{
      verdicts: Verdict[];
      total: number;
      page: number;
      limit: number;
    }>(`/verdicts?${params}`);

    return response;
  }

  /**
   * Get specific verdict details
   */
  async getVerdict(id: string): Promise<Verdict> {
    const response = await this.apiClient.request<Verdict>(`/verdicts/${id}`);
    return response;
  }

  /**
   * Get verdict evidence
   */
  async getVerdictEvidence(verdictId: string): Promise<Evidence[]> {
    const response = await this.apiClient.request<Evidence[]>(`/verdicts/${verdictId}/evidence`);
    return response;
  }

  /**
   * Override verdict decision
   */
  async overrideVerdict(
    verdictId: string,
    override: {
      decision: 'approve' | 'reject' | 'escalate';
      reason: string;
      operator: string;
    }
  ): Promise<Verdict> {
    const response = await this.apiClient.request<Verdict>(`/verdicts/${verdictId}/override`, {
      method: 'POST',
      body: JSON.stringify(override)
    });
    return response;
  }

  /**
   * Escalate verdict for review
   */
  async escalateVerdict(
    verdictId: string,
    escalation: {
      reason: string;
      priority: 'low' | 'medium' | 'high' | 'critical';
      operator: string;
    }
  ): Promise<Verdict> {
    const response = await this.apiClient.request<Verdict>(`/verdicts/${verdictId}/escalate`, {
      method: 'POST',
      body: JSON.stringify(escalation)
    });
    return response;
  }

  /**
   * Get all judges
   */
  async getJudges(): Promise<Judge[]> {
    const response = await this.apiClient.request<Judge[]>('/judges');
    return response;
  }

  /**
   * Get judge details
   */
  async getJudge(id: string): Promise<Judge> {
    const response = await this.apiClient.request<Judge>(`/judges/${id}`);
    return response;
  }

  /**
   * Get judge performance report
   */
  async getJudgePerformance(
    judgeId: string,
    period: { start: Date; end: Date }
  ): Promise<JudgePerformanceReport> {
    const params = new URLSearchParams({
      startDate: period.start.toISOString(),
      endDate: period.end.toISOString(),
    });

    const response = await this.apiClient.request<JudgePerformanceReport>(
      `/judges/${judgeId}/performance?${params}`
    );
    return response;
  }

  /**
   * Get council metrics
   */
  async getMetrics(): Promise<CouncilMetrics> {
    const response = await this.apiClient.request<CouncilMetrics>('/metrics');
    return response;
  }

  /**
   * Get active alerts
   */
  async getAlerts(
    acknowledged: boolean = false,
    limit: number = 50
  ): Promise<CouncilAlert[]> {
    const params = new URLSearchParams({
      acknowledged: acknowledged.toString(),
      limit: limit.toString(),
    });

    const response = await this.apiClient.request<CouncilAlert[]>(`/alerts?${params}`);
    return response;
  }

  /**
   * Acknowledge alert
   */
  async acknowledgeAlert(alertId: string): Promise<void> {
    await this.apiClient.request<void>(`/alerts/${alertId}/acknowledge`, {
      method: 'POST'
    });
  }

  /**
   * Get ethical assessments
   */
  async getEthicalAssessments(
    verdictId?: string,
    riskLevel?: EthicalAssessment['overallRisk'],
    limit: number = 20
  ): Promise<EthicalAssessment[]> {
    const params = new URLSearchParams({
      limit: limit.toString(),
    });

    if (verdictId) params.append('verdictId', verdictId);
    if (riskLevel) params.append('riskLevel', riskLevel);

    const response = await this.apiClient.request<EthicalAssessment[]>(
      `/ethical-assessments?${params}`
    );
    return response;
  }

  /**
   * Get ethical assessment details
   */
  async getEthicalAssessment(id: string): Promise<EthicalAssessment> {
    const response = await this.apiClient.request<EthicalAssessment>(`/ethical-assessments/${id}`);
    return response;
  }

  /**
   * Update ethical assessment
   */
  async updateEthicalAssessment(
    id: string,
    updates: Partial<EthicalAssessment>
  ): Promise<EthicalAssessment> {
    const response = await this.apiClient.request<EthicalAssessment>(
      `/ethical-assessments/${id}`,
      {
        method: 'PATCH',
        body: JSON.stringify(updates)
      }
    );
    return response;
  }

  /**
   * Get decision flow data
   */
  async getDecisionFlow(verdictId: string): Promise<{
    stages: {
      id: string;
      name: string;
      status: 'pending' | 'in_progress' | 'completed';
      startTime: Date;
      endTime?: Date;
      judges: JudgeAssignment[];
    }[];
    currentStage: string;
    progress: number;
  }> {
    const response = await this.apiClient.request<{
      stages: {
        id: string;
        name: string;
        status: 'pending' | 'in_progress' | 'completed';
        startTime: Date;
        endTime?: Date;
        judges: JudgeAssignment[];
      }[];
      currentStage: string;
      progress: number;
    }>(`/verdicts/${verdictId}/flow`);
    return response;
  }

  /**
   * Get historical verdict data for analytics
   */
  async getVerdictAnalytics(
    period: { start: Date; end: Date },
    granularity: 'hour' | 'day' | 'week' | 'month' = 'day'
  ): Promise<{
    timeline: {
      timestamp: Date;
      verdicts: number;
      approvals: number;
      rejections: number;
      escalations: number;
    }[];
    judgePerformance: {
      judgeId: string;
      accuracy: number;
      responseTime: number;
      consensusRate: number;
    }[];
    ethicalTrends: {
      timestamp: Date;
      highRiskVerdicts: number;
      ethicalConcerns: number;
    }[];
  }> {
    const params = new URLSearchParams({
      startDate: period.start.toISOString(),
      endDate: period.end.toISOString(),
      granularity,
    });

    const response = await this.apiClient.request<{
      timeline: {
        timestamp: Date;
        verdicts: number;
        approvals: number;
        rejections: number;
        escalations: number;
      }[];
      judgePerformance: {
        judgeId: string;
        accuracy: number;
        responseTime: number;
        consensusRate: number;
      }[];
      ethicalTrends: {
        timestamp: Date;
        highRiskVerdicts: number;
        ethicalConcerns: number;
      }[];
    }>(`/analytics?${params}`);
    return response;
  }

  /**
   * Export verdict data
   */
  async exportVerdicts(
    format: 'json' | 'csv' | 'pdf' = 'json',
    filters?: VerdictFilter
  ): Promise<Blob> {
    const params = new URLSearchParams({ format });

    if (filters) {
      if (filters.status) params.append('status', filters.status.join(','));
      if (filters.judgeId) params.append('judgeId', filters.judgeId);
      if (filters.riskLevel) params.append('riskLevel', filters.riskLevel.join(','));
      if (filters.dateRange) {
        params.append('startDate', filters.dateRange.start.toISOString());
        params.append('endDate', filters.dateRange.end.toISOString());
      }
    }

    const response = await fetch(`${this.apiClient['config'].baseUrl}/export?${params}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${this.apiClient['config'].authToken}`
      }
    });

    if (!response.ok) {
      throw new Error(`Export failed: ${response.statusText}`);
    }

    return response.blob();
  }
}

// Export singleton instance
export const councilApiClient = new CouncilApiClient();