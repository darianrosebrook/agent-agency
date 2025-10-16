/**
 * @fileoverview Policy Audit Manager - ARBITER-029
 *
 * Comprehensive CAWS compliance auditing and waiver evaluation workflow
 * for post-task analysis and remediation tracking.
 *
 * @author @darianrosebrook
 */

import { CAWSPolicyViolation } from "./CAWSPolicyEnforcer";

export interface PolicyAuditResult {
  taskId: string;
  agentId: string;
  auditTimestamp: Date;
  overallCompliance: "compliant" | "non_compliant" | "requires_review";
  complianceScore: number; // 0-100
  violations: CAWSPolicyViolation[];
  recommendations: string[];
  waiverRequired: boolean;
  waiverReason?: string;
  remediationPlan?: RemediationPlan;
}

export interface RemediationPlan {
  priority: "high" | "medium" | "low";
  actions: Array<{
    action: string;
    description: string;
    dueDate: Date;
    assignee?: string;
    status: "pending" | "in_progress" | "completed" | "overdue";
  }>;
  estimatedCompletionDate: Date;
  successCriteria: string[];
}

export interface WaiverRequest {
  id: string;
  taskId: string;
  agentId: string;
  requestTimestamp: Date;
  reason: string;
  justification: string;
  requestedBy: string;
  status: "pending" | "approved" | "denied" | "expired";
  reviewedBy?: string;
  reviewTimestamp?: Date;
  reviewComments?: string;
  expirationDate: Date;
  conditions?: string[];
}

export interface ComplianceReport {
  reportId: string;
  generatedAt: Date;
  period: {
    startDate: Date;
    endDate: Date;
  };
  summary: {
    totalTasks: number;
    compliantTasks: number;
    nonCompliantTasks: number;
    waiverRequests: number;
    averageComplianceScore: number;
  };
  violations: {
    byType: Record<CAWSPolicyViolation["type"], number>;
    bySeverity: Record<CAWSPolicyViolation["severity"], number>;
    byAgent: Record<string, number>;
  };
  trends: {
    complianceTrend: "improving" | "stable" | "declining";
    violationTrend: "increasing" | "stable" | "decreasing";
    topViolatingAgents: Array<{ agentId: string; violationCount: number }>;
  };
  recommendations: string[];
}

export interface PolicyAuditManager {
  /**
   * Audit task for CAWS compliance
   */
  auditTask(
    taskId: string,
    agentId: string,
    taskData: any
  ): Promise<PolicyAuditResult>;

  /**
   * Request waiver for policy violations
   */
  requestWaiver(
    request: Omit<WaiverRequest, "id" | "requestTimestamp" | "status">
  ): Promise<WaiverRequest>;

  /**
   * Review and approve/deny waiver request
   */
  reviewWaiver(
    waiverId: string,
    decision: "approved" | "denied",
    reviewerId: string,
    comments?: string
  ): Promise<WaiverRequest>;

  /**
   * Get active waivers for agent
   */
  getActiveWaivers(agentId: string): Promise<WaiverRequest[]>;

  /**
   * Create remediation plan for violations
   */
  createRemediationPlan(
    violations: CAWSPolicyViolation[],
    agentId: string
  ): Promise<RemediationPlan>;

  /**
   * Update remediation plan status
   */
  updateRemediationStatus(
    planId: string,
    actionId: string,
    status: RemediationPlan["actions"][0]["status"]
  ): Promise<void>;

  /**
   * Generate compliance report
   */
  generateComplianceReport(
    startDate: Date,
    endDate: Date
  ): Promise<ComplianceReport>;

  /**
   * Get audit history for agent
   */
  getAuditHistory(
    agentId: string,
    limit?: number
  ): Promise<PolicyAuditResult[]>;

  /**
   * Get compliance statistics
   */
  getComplianceStatistics(): Promise<{
    totalAudits: number;
    complianceRate: number;
    averageComplianceScore: number;
    activeWaivers: number;
    pendingRemediations: number;
  }>;
}

/**
 * Implementation of Policy Audit Manager
 */
export class PolicyAuditManagerImpl implements PolicyAuditManager {
  private auditHistory: Map<string, PolicyAuditResult[]> = new Map();
  private waiverRequests: Map<string, WaiverRequest> = new Map();
  private remediationPlans: Map<string, RemediationPlan> = new Map();

  constructor(
    private config: {
      complianceThreshold: number; // minimum score for compliance
      waiverExpirationDays: number;
      remediationDueDays: number;
    } = {
      complianceThreshold: 80,
      waiverExpirationDays: 30,
      remediationDueDays: 14,
    }
  ) {}

  async auditTask(
    taskId: string,
    agentId: string,
    taskData: any
  ): Promise<PolicyAuditResult> {
    const auditTimestamp = new Date();

    // Extract violations from task data (this would come from CAWSPolicyEnforcer)
    const violations = taskData.violations || [];

    // Calculate compliance score
    const complianceScore = this.calculateComplianceScore(violations);

    // Determine overall compliance
    const overallCompliance = this.determineCompliance(
      complianceScore,
      violations
    );

    // Generate recommendations
    const recommendations = this.generateRecommendations(violations);

    // Check if waiver is required
    const waiverRequired = this.isWaiverRequired(violations);
    const waiverReason = waiverRequired
      ? this.getWaiverReason(violations)
      : undefined;

    // Create remediation plan if needed
    const remediationPlan =
      violations.length > 0
        ? await this.createRemediationPlan(violations, agentId)
        : undefined;

    const auditResult: PolicyAuditResult = {
      taskId,
      agentId,
      auditTimestamp,
      overallCompliance,
      complianceScore,
      violations,
      recommendations,
      waiverRequired,
      waiverReason,
      remediationPlan,
    };

    // Store audit result
    this.storeAuditResult(agentId, auditResult);

    return auditResult;
  }

  async requestWaiver(
    request: Omit<WaiverRequest, "id" | "requestTimestamp" | "status">
  ): Promise<WaiverRequest> {
    const waiverRequest: WaiverRequest = {
      ...request,
      id: `waiver-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      requestTimestamp: new Date(),
      status: "pending",
    };

    this.waiverRequests.set(waiverRequest.id, waiverRequest);

    return waiverRequest;
  }

  async reviewWaiver(
    waiverId: string,
    decision: "approved" | "denied",
    reviewerId: string,
    comments?: string
  ): Promise<WaiverRequest> {
    const waiver = this.waiverRequests.get(waiverId);
    if (!waiver) {
      throw new Error(`Waiver request not found: ${waiverId}`);
    }

    if (waiver.status !== "pending") {
      throw new Error(`Waiver request already reviewed: ${waiverId}`);
    }

    waiver.status = decision;
    waiver.reviewedBy = reviewerId;
    waiver.reviewTimestamp = new Date();
    waiver.reviewComments = comments;

    this.waiverRequests.set(waiverId, waiver);

    return waiver;
  }

  async getActiveWaivers(agentId: string): Promise<WaiverRequest[]> {
    const now = new Date();
    return Array.from(this.waiverRequests.values()).filter(
      (waiver) =>
        waiver.agentId === agentId &&
        waiver.status === "approved" &&
        waiver.expirationDate > now
    );
  }

  async createRemediationPlan(
    violations: CAWSPolicyViolation[],
    agentId: string
  ): Promise<RemediationPlan> {
    const actions: RemediationPlan["actions"] = [];
    const now = new Date();

    // Analyze violations and create specific actions
    for (const violation of violations) {
      switch (violation.type) {
        case "budget_exceeded":
          actions.push({
            action: "budget_training",
            description: "Complete budget management training module",
            dueDate: new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000), // 7 days
            status: "pending",
          });
          break;
        case "tool_usage_violation":
          actions.push({
            action: "tool_usage_review",
            description: "Review approved tools and usage guidelines",
            dueDate: new Date(now.getTime() + 3 * 24 * 60 * 60 * 1000), // 3 days
            status: "pending",
          });
          break;
        case "reasoning_depth_exceeded":
          actions.push({
            action: "reasoning_optimization",
            description: "Complete reasoning efficiency training",
            dueDate: new Date(now.getTime() + 10 * 24 * 60 * 60 * 1000), // 10 days
            status: "pending",
          });
          break;
        case "forbidden_operation":
          actions.push({
            action: "security_training",
            description: "Mandatory security awareness training",
            dueDate: new Date(now.getTime() + 1 * 24 * 60 * 60 * 1000), // 1 day
            status: "pending",
          });
          break;
      }
    }

    // Determine priority based on violation severity
    const hasCriticalViolations = violations.some(
      (v) => v.severity === "critical"
    );
    const hasHighViolations = violations.some((v) => v.severity === "high");

    const priority: RemediationPlan["priority"] = hasCriticalViolations
      ? "high"
      : hasHighViolations
      ? "medium"
      : "low";

    const remediationPlan: RemediationPlan = {
      priority,
      actions,
      estimatedCompletionDate: new Date(
        now.getTime() + this.config.remediationDueDays * 24 * 60 * 60 * 1000
      ),
      successCriteria: [
        "Complete all assigned training modules",
        "Pass compliance assessment with 90% score",
        "Demonstrate improved task execution within budget",
        "No new violations for 30 days",
      ],
    };

    const planId = `remediation-${Date.now()}-${agentId}`;
    this.remediationPlans.set(planId, remediationPlan);

    return remediationPlan;
  }

  async updateRemediationStatus(
    planId: string,
    actionId: string,
    status: RemediationPlan["actions"][0]["status"]
  ): Promise<void> {
    const plan = this.remediationPlans.get(planId);
    if (!plan) {
      throw new Error(`Remediation plan not found: ${planId}`);
    }

    const action = plan.actions.find((a) => a.action === actionId);
    if (!action) {
      throw new Error(`Action not found in plan: ${actionId}`);
    }

    action.status = status;
    this.remediationPlans.set(planId, plan);
  }

  async generateComplianceReport(
    startDate: Date,
    endDate: Date
  ): Promise<ComplianceReport> {
    const allAudits = Array.from(this.auditHistory.values()).flat();
    const periodAudits = allAudits.filter(
      (audit) =>
        audit.auditTimestamp >= startDate && audit.auditTimestamp <= endDate
    );

    const totalTasks = periodAudits.length;
    const compliantTasks = periodAudits.filter(
      (a) => a.overallCompliance === "compliant"
    ).length;
    const nonCompliantTasks = periodAudits.filter(
      (a) => a.overallCompliance === "non_compliant"
    ).length;
    const averageComplianceScore =
      totalTasks > 0
        ? periodAudits.reduce((sum, a) => sum + a.complianceScore, 0) /
          totalTasks
        : 0;

    // Analyze violations
    const violations = {
      byType: {} as Record<CAWSPolicyViolation["type"], number>,
      bySeverity: {} as Record<CAWSPolicyViolation["severity"], number>,
      byAgent: {} as Record<string, number>,
    };

    for (const audit of periodAudits) {
      for (const violation of audit.violations) {
        violations.byType[violation.type] =
          (violations.byType[violation.type] || 0) + 1;
        violations.bySeverity[violation.severity] =
          (violations.bySeverity[violation.severity] || 0) + 1;
        violations.byAgent[audit.agentId] =
          (violations.byAgent[audit.agentId] || 0) + 1;
      }
    }

    // Calculate trends
    const midPoint = new Date(
      startDate.getTime() + (endDate.getTime() - startDate.getTime()) / 2
    );
    const firstHalfAudits = periodAudits.filter(
      (a) => a.auditTimestamp <= midPoint
    );
    const secondHalfAudits = periodAudits.filter(
      (a) => a.auditTimestamp > midPoint
    );

    const firstHalfScore =
      firstHalfAudits.length > 0
        ? firstHalfAudits.reduce((sum, a) => sum + a.complianceScore, 0) /
          firstHalfAudits.length
        : 0;
    const secondHalfScore =
      secondHalfAudits.length > 0
        ? secondHalfAudits.reduce((sum, a) => sum + a.complianceScore, 0) /
          secondHalfAudits.length
        : 0;

    const complianceTrend =
      secondHalfScore > firstHalfScore + 5
        ? "improving"
        : secondHalfScore < firstHalfScore - 5
        ? "declining"
        : "stable";

    const firstHalfViolations = firstHalfAudits.reduce(
      (sum, a) => sum + a.violations.length,
      0
    );
    const secondHalfViolations = secondHalfAudits.reduce(
      (sum, a) => sum + a.violations.length,
      0
    );

    const violationTrend =
      secondHalfViolations > firstHalfViolations * 1.1
        ? "increasing"
        : secondHalfViolations < firstHalfViolations * 0.9
        ? "decreasing"
        : "stable";

    const topViolatingAgents = Object.entries(violations.byAgent)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 5)
      .map(([agentId, violationCount]) => ({ agentId, violationCount }));

    // Generate recommendations
    const recommendations = this.generateReportRecommendations(
      violations,
      complianceTrend,
      violationTrend
    );

    const report: ComplianceReport = {
      reportId: `report-${Date.now()}`,
      generatedAt: new Date(),
      period: { startDate, endDate },
      summary: {
        totalTasks,
        compliantTasks,
        nonCompliantTasks,
        waiverRequests: Array.from(this.waiverRequests.values()).filter(
          (w) =>
            w.requestTimestamp >= startDate && w.requestTimestamp <= endDate
        ).length,
        averageComplianceScore,
      },
      violations,
      trends: {
        complianceTrend,
        violationTrend,
        topViolatingAgents,
      },
      recommendations,
    };

    return report;
  }

  async getAuditHistory(
    agentId: string,
    limit: number = 50
  ): Promise<PolicyAuditResult[]> {
    const audits = this.auditHistory.get(agentId) || [];
    return audits
      .sort((a, b) => b.auditTimestamp.getTime() - a.auditTimestamp.getTime())
      .slice(0, limit);
  }

  async getComplianceStatistics(): Promise<{
    totalAudits: number;
    complianceRate: number;
    averageComplianceScore: number;
    activeWaivers: number;
    pendingRemediations: number;
  }> {
    const allAudits = Array.from(this.auditHistory.values()).flat();
    const totalAudits = allAudits.length;

    const compliantAudits = allAudits.filter(
      (a) => a.overallCompliance === "compliant"
    ).length;
    const complianceRate =
      totalAudits > 0 ? (compliantAudits / totalAudits) * 100 : 0;

    const averageComplianceScore =
      totalAudits > 0
        ? allAudits.reduce((sum, a) => sum + a.complianceScore, 0) / totalAudits
        : 0;

    const now = new Date();
    const activeWaivers = Array.from(this.waiverRequests.values()).filter(
      (w) => w.status === "approved" && w.expirationDate > now
    ).length;

    const pendingRemediations = Array.from(
      this.remediationPlans.values()
    ).filter((p) => p.actions.some((a) => a.status === "pending")).length;

    return {
      totalAudits,
      complianceRate,
      averageComplianceScore,
      activeWaivers,
      pendingRemediations,
    };
  }

  private calculateComplianceScore(violations: CAWSPolicyViolation[]): number {
    if (violations.length === 0) {
      return 100;
    }

    let penalty = 0;
    for (const violation of violations) {
      switch (violation.severity) {
        case "critical":
          penalty += 25;
          break;
        case "high":
          penalty += 15;
          break;
        case "medium":
          penalty += 10;
          break;
        case "low":
          penalty += 5;
          break;
      }
    }

    return Math.max(0, 100 - penalty);
  }

  private determineCompliance(
    score: number,
    violations: CAWSPolicyViolation[]
  ): PolicyAuditResult["overallCompliance"] {
    if (score >= this.config.complianceThreshold) {
      return "compliant";
    } else if (violations.some((v) => v.severity === "critical")) {
      return "non_compliant";
    } else {
      return "requires_review";
    }
  }

  private generateRecommendations(violations: CAWSPolicyViolation[]): string[] {
    const recommendations: string[] = [];

    if (violations.some((v) => v.type === "budget_exceeded")) {
      recommendations.push(
        "Review task planning and file management practices"
      );
    }

    if (violations.some((v) => v.type === "tool_usage_violation")) {
      recommendations.push(
        "Familiarize with approved tools and usage guidelines"
      );
    }

    if (violations.some((v) => v.type === "reasoning_depth_exceeded")) {
      recommendations.push("Optimize reasoning approach to reduce depth");
    }

    if (violations.some((v) => v.type === "forbidden_operation")) {
      recommendations.push("Complete security awareness training immediately");
    }

    if (violations.length > 3) {
      recommendations.push("Consider comprehensive compliance training");
    }

    return recommendations;
  }

  private isWaiverRequired(violations: CAWSPolicyViolation[]): boolean {
    return violations.some(
      (v) => v.severity === "critical" || v.severity === "high"
    );
  }

  private getWaiverReason(violations: CAWSPolicyViolation[]): string {
    const criticalViolations = violations.filter(
      (v) => v.severity === "critical"
    );
    const highViolations = violations.filter((v) => v.severity === "high");

    if (criticalViolations.length > 0) {
      return `Critical violations detected: ${criticalViolations
        .map((v) => v.type)
        .join(", ")}`;
    } else if (highViolations.length > 0) {
      return `High severity violations detected: ${highViolations
        .map((v) => v.type)
        .join(", ")}`;
    }

    return "Multiple policy violations requiring review";
  }

  private generateReportRecommendations(
    violations: any,
    complianceTrend: string,
    violationTrend: string
  ): string[] {
    const recommendations: string[] = [];

    if (complianceTrend === "declining") {
      recommendations.push(
        "Compliance trend is declining - investigate root causes"
      );
    }

    if (violationTrend === "increasing") {
      recommendations.push(
        "Violation trend is increasing - strengthen enforcement"
      );
    }

    const topViolationType = Object.entries(violations.byType).sort(
      ([, a], [, b]) => (b as number) - (a as number)
    )[0];

    if (topViolationType) {
      recommendations.push(
        `Focus training on most common violation: ${topViolationType[0]}`
      );
    }

    return recommendations;
  }

  private storeAuditResult(
    agentId: string,
    auditResult: PolicyAuditResult
  ): void {
    if (!this.auditHistory.has(agentId)) {
      this.auditHistory.set(agentId, []);
    }

    this.auditHistory.get(agentId)!.push(auditResult);
  }
}

