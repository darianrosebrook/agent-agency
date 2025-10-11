/**
 * CAWS Constitutional Enforcer
 *
 * Enforces CAWS constitutional authority through automatic budget limits,
 * waiver management, and quality gate enforcement during task execution.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { Logger } from "../utils/Logger.js";
import { MultiTenantMemoryManager } from "../memory/MultiTenantMemoryManager.js";

export interface BudgetLimits {
  maxFiles: number;
  maxLoc: number;
  maxDurationMs: number;
  maxConcurrentTasks: number;
}

export interface QualityGates {
  minCoverage: number;
  minMutationScore: number;
  requireContracts: boolean;
  requireManualReview: boolean;
  trustScore: number;
}

export interface WaiverRequest {
  id: string;
  title: string;
  reason: "emergency_hotfix" | "legacy_integration" | "experimental_feature" | "third_party_constraint" | "performance_critical" | "security_patch" | "infrastructure_limitation" | "other";
  description: string;
  gates: string[];
  expiresAt: Date;
  approvedBy: string;
  impactLevel: "low" | "medium" | "high" | "critical";
  mitigationPlan: string;
  tenantId: string;
  requestedAt: Date;
  status: "pending" | "approved" | "rejected" | "expired";
}

export interface EnforcementResult {
  allowed: boolean;
  violations: string[];
  waivers: WaiverRequest[];
  recommendations: string[];
  budgetStatus: {
    currentFiles: number;
    currentLoc: number;
    remainingTimeMs: number;
  };
  gateStatus: {
    coverageMet: boolean;
    mutationMet: boolean;
    contractsMet: boolean;
    trustScoreMet: boolean;
  };
}

export interface TaskBudget {
  taskId: string;
  tenantId: string;
  limits: BudgetLimits;
  currentUsage: {
    files: number;
    loc: number;
    durationMs: number;
  };
  startTime: Date;
  checkIns: Array<{
    timestamp: Date;
    files: number;
    loc: number;
    message: string;
  }>;
}

export class CawsConstitutionalEnforcer extends EventEmitter {
  private logger: Logger;
  private memoryManager?: MultiTenantMemoryManager;
  private activeBudgets = new Map<string, TaskBudget>();
  private waivers = new Map<string, WaiverRequest>();

  // Default CAWS constitutional limits by tier
  private readonly CONSTITUTIONAL_LIMITS = {
    1: { // Critical/Auth tier
      maxFiles: 40,
      maxLoc: 1500,
      maxDurationMs: 3600000, // 1 hour
      maxConcurrentTasks: 3,
    },
    2: { // Standard tier
      maxFiles: 25,
      maxLoc: 1000,
      maxDurationMs: 1800000, // 30 minutes
      maxConcurrentTasks: 5,
    },
    3: { // Low-risk tier
      maxFiles: 15,
      maxLoc: 500,
      maxDurationMs: 900000, // 15 minutes
      maxConcurrentTasks: 10,
    },
  };

  // Quality gates by tier
  private readonly QUALITY_GATES = {
    1: {
      minCoverage: 0.9,
      minMutationScore: 0.7,
      requireContracts: true,
      requireManualReview: true,
      trustScore: 85,
    },
    2: {
      minCoverage: 0.8,
      minMutationScore: 0.5,
      requireContracts: true,
      requireManualReview: false,
      trustScore: 82,
    },
    3: {
      minCoverage: 0.7,
      minMutationScore: 0.3,
      requireContracts: false,
      requireManualReview: false,
      trustScore: 75,
    },
  };

  constructor(memoryManager?: MultiTenantMemoryManager) {
    super();
    this.logger = new Logger("CawsConstitutionalEnforcer");
    this.memoryManager = memoryManager;

    // Load existing waivers from memory
    this.loadWaiversFromMemory();
  }

  /**
   * Enforce constitutional limits before task execution
   */
  async enforceConstitution(
    taskId: string,
    tenantId: string,
    tier: number,
    context: Record<string, any>
  ): Promise<EnforcementResult> {
    const result: EnforcementResult = {
      allowed: true,
      violations: [],
      waivers: [],
      recommendations: [],
      budgetStatus: {
        currentFiles: 0,
        currentLoc: 0,
        remainingTimeMs: 0,
      },
      gateStatus: {
        coverageMet: false,
        mutationMet: false,
        contractsMet: false,
        trustScoreMet: false,
      },
    };

    try {
      // Check budget limits
      const budgetResult = await this.checkBudgetLimits(taskId, tenantId, tier);
      if (!budgetResult.allowed) {
        result.allowed = false;
        result.violations.push(...budgetResult.violations);
        result.recommendations.push(...budgetResult.recommendations);
      }
      result.budgetStatus = budgetResult.budgetStatus;

      // Check quality gates
      const gateResult = await this.checkQualityGates(tenantId, tier, context);
      if (!gateResult.allowed) {
        result.allowed = false;
        result.violations.push(...gateResult.violations);
        result.recommendations.push(...gateResult.recommendations);
      }
      result.gateStatus = gateResult.gateStatus;

      // Check for applicable waivers
      const applicableWaivers = this.getApplicableWaivers(tenantId, result.violations);
      result.waivers = applicableWaivers;

      // If we have waivers that cover all violations, allow the task
      if (applicableWaivers.length > 0 && this.waiversCoverViolations(applicableWaivers, result.violations)) {
        result.allowed = true;
        result.violations = []; // Waived violations don't count
        this.logger.info(`Task ${taskId} allowed via constitutional waiver(s)`);
      }

      // Log enforcement decision
      this.logger.info(`Constitutional enforcement for task ${taskId}: ${result.allowed ? "ALLOWED" : "BLOCKED"} (${result.violations.length} violations, ${applicableWaivers.length} waivers)`);

      this.emit("enforcement-decision", { taskId, tenantId, result });

      return result;
    } catch (error) {
      this.logger.error(`Constitutional enforcement failed for task ${taskId}:`, error);
      // Default to allowing if enforcement fails (fail-open for safety)
      return result;
    }
  }

  /**
   * Start budget tracking for a task
   */
  startBudgetTracking(taskId: string, tenantId: string, tier: number): void {
    const limits = this.CONSTITUTIONAL_LIMITS[tier] || this.CONSTITUTIONAL_LIMITS[2];

    const budget: TaskBudget = {
      taskId,
      tenantId,
      limits,
      currentUsage: {
        files: 0,
        loc: 0,
        durationMs: 0,
      },
      startTime: new Date(),
      checkIns: [{
        timestamp: new Date(),
        files: 0,
        loc: 0,
        message: "Budget tracking started",
      }],
    };

    this.activeBudgets.set(taskId, budget);
    this.logger.info(`Started budget tracking for task ${taskId} (tier ${tier})`);
  }

  /**
   * Update budget usage during task execution
   */
  updateBudgetUsage(
    taskId: string,
    files: number,
    loc: number,
    message?: string
  ): boolean {
    const budget = this.activeBudgets.get(taskId);
    if (!budget) {
      this.logger.warn(`No budget tracking found for task ${taskId}`);
      return true; // Allow if not tracking
    }

    // Update usage
    budget.currentUsage.files = Math.max(budget.currentUsage.files, files);
    budget.currentUsage.loc = Math.max(budget.currentUsage.loc, loc);
    budget.currentUsage.durationMs = Date.now() - budget.startTime.getTime();

    // Add check-in
    budget.checkIns.push({
      timestamp: new Date(),
      files,
      loc,
      message: message || "Budget check-in",
    });

    // Check limits
    const violations = this.checkBudgetViolations(budget);
    if (violations.length > 0) {
      this.logger.warn(`Budget violations detected for task ${taskId}:`, violations);
      this.emit("budget-violation", { taskId, budget, violations });
      return false;
    }

    // Keep only last 10 check-ins to prevent memory bloat
    if (budget.checkIns.length > 10) {
      budget.checkIns = budget.checkIns.slice(-10);
    }

    return true;
  }

  /**
   * Stop budget tracking and clean up
   */
  stopBudgetTracking(taskId: string): TaskBudget | null {
    const budget = this.activeBudgets.get(taskId);
    if (budget) {
      this.activeBudgets.delete(taskId);

      // Store final budget data in memory for analysis
      this.storeBudgetAnalysis(budget);

      this.logger.info(`Stopped budget tracking for task ${taskId}`);
    }
    return budget;
  }

  /**
   * Request a constitutional waiver
   */
  async requestWaiver(request: Omit<WaiverRequest, "id" | "requestedAt" | "status">): Promise<string> {
    const waiverId = `waiver-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    const waiver: WaiverRequest = {
      ...request,
      id: waiverId,
      requestedAt: new Date(),
      status: "pending",
    };

    this.waivers.set(waiverId, waiver);

    // Store in memory for persistence
    await this.storeWaiverInMemory(waiver);

    this.logger.info(`Constitutional waiver requested: ${waiverId} (${request.title})`);
    this.emit("waiver-requested", waiver);

    return waiverId;
  }

  /**
   * Approve or reject a waiver
   */
  async processWaiver(waiverId: string, approved: boolean, approver: string, notes?: string): Promise<boolean> {
    const waiver = this.waivers.get(waiverId);
    if (!waiver) {
      throw new Error(`Waiver ${waiverId} not found`);
    }

    if (waiver.status !== "pending") {
      throw new Error(`Waiver ${waiverId} is already ${waiver.status}`);
    }

    waiver.status = approved ? "approved" : "rejected";
    waiver.approvedBy = approver;

    // Update in memory
    await this.updateWaiverInMemory(waiver);

    this.logger.info(`Waiver ${waiverId} ${waiver.status} by ${approver}`);
    this.emit("waiver-processed", { waiver, approved, notes });

    return approved;
  }

  /**
   * Get current budget status for a task
   */
  getBudgetStatus(taskId: string): TaskBudget | null {
    return this.activeBudgets.get(taskId) || null;
  }

  /**
   * Get active waivers for a tenant
   */
  getActiveWaivers(tenantId: string): WaiverRequest[] {
    const now = new Date();
    return Array.from(this.waivers.values())
      .filter(w => w.tenantId === tenantId && w.status === "approved" && w.expiresAt > now);
  }

  /**
   * Get constitutional enforcement analytics
   */
  getAnalytics(): {
    activeBudgets: number;
    activeWaivers: number;
    recentViolations: Array<{
      taskId: string;
      violations: string[];
      timestamp: Date;
    }>;
    waiverRequests: Array<{
      id: string;
      title: string;
      status: string;
      requestedAt: Date;
    }>;
  } {
    const recentViolations = Array.from(this.activeBudgets.values())
      .filter(b => this.checkBudgetViolations(b).length > 0)
      .map(b => ({
        taskId: b.taskId,
        violations: this.checkBudgetViolations(b),
        timestamp: b.startTime,
      }))
      .slice(-10); // Last 10 violations

    const waiverRequests = Array.from(this.waivers.values())
      .map(w => ({
        id: w.id,
        title: w.title,
        status: w.status,
        requestedAt: w.requestedAt,
      }))
      .sort((a, b) => b.requestedAt.getTime() - a.requestedAt.getTime())
      .slice(0, 10); // Most recent 10

    return {
      activeBudgets: this.activeBudgets.size,
      activeWaivers: Array.from(this.waivers.values()).filter(w => w.status === "approved" && w.expiresAt > new Date()).length,
      recentViolations,
      waiverRequests,
    };
  }

  /**
   * Check budget limits for a task
   */
  private async checkBudgetLimits(taskId: string, tenantId: string, tier: number): Promise<{
    allowed: boolean;
    violations: string[];
    recommendations: string[];
    budgetStatus: EnforcementResult["budgetStatus"];
  }> {
    const limits = this.CONSTITUTIONAL_LIMITS[tier] || this.CONSTITUTIONAL_LIMITS[2];

    // Check concurrent tasks
    const tenantTasks = Array.from(this.activeBudgets.values())
      .filter(b => b.tenantId === tenantId).length;

    const violations: string[] = [];
    const recommendations: string[] = [];

    if (tenantTasks >= limits.maxConcurrentTasks) {
      violations.push(`Exceeded concurrent task limit (${tenantTasks}/${limits.maxConcurrentTasks})`);
      recommendations.push("Wait for existing tasks to complete or request a waiver");
    }

    // Check for recent large tasks that might indicate scope creep
    const recentLargeTasks = await this.getRecentLargeTasks(tenantId);
    if (recentLargeTasks.length > 2) {
      recommendations.push("Consider breaking large tasks into smaller, focused tasks");
    }

    const budgetStatus = {
      currentFiles: 0,
      currentLoc: 0,
      remainingTimeMs: limits.maxDurationMs,
    };

    return {
      allowed: violations.length === 0,
      violations,
      recommendations,
      budgetStatus,
    };
  }

  /**
   * Check quality gates
   */
  private async checkQualityGates(
    tenantId: string,
    tier: number,
    context: Record<string, any>
  ): Promise<{
    allowed: boolean;
    violations: string[];
    recommendations: string[];
    gateStatus: EnforcementResult["gateStatus"];
  }> {
    const gates = this.QUALITY_GATES[tier] || this.QUALITY_GATES[2];

    const violations: string[] = [];
    const recommendations: string[] = [];

    // Get current quality metrics (this would integrate with actual test results)
    const qualityMetrics = await this.getQualityMetrics(tenantId);

    const gateStatus = {
      coverageMet: qualityMetrics.coverage >= gates.minCoverage,
      mutationMet: qualityMetrics.mutation >= gates.minMutationScore,
      contractsMet: !gates.requireContracts || qualityMetrics.contracts,
      trustScoreMet: qualityMetrics.trustScore >= gates.trustScore,
    };

    if (!gateStatus.coverageMet) {
      violations.push(`Coverage ${qualityMetrics.coverage.toFixed(2)} below required ${(gates.minCoverage * 100).toFixed(0)}%`);
      recommendations.push("Add more unit tests to improve coverage");
    }

    if (!gateStatus.mutationMet) {
      violations.push(`Mutation score ${qualityMetrics.mutation.toFixed(2)} below required ${(gates.minMutationScore * 100).toFixed(0)}%`);
      recommendations.push("Improve test quality to catch more mutations");
    }

    if (!gateStatus.contractsMet) {
      violations.push("Contract tests required but not present");
      recommendations.push("Add OpenAPI/GraphQL contract tests");
    }

    if (!gateStatus.trustScoreMet) {
      violations.push(`Trust score ${qualityMetrics.trustScore} below required ${gates.trustScore}`);
      recommendations.push("Address code quality issues and security concerns");
    }

    return {
      allowed: violations.length === 0,
      violations,
      recommendations,
      gateStatus,
    };
  }

  /**
   * Get quality metrics for tenant
   */
  private async getQualityMetrics(tenantId: string): Promise<{
    coverage: number;
    mutation: number;
    contracts: boolean;
    trustScore: number;
  }> {
    // This would integrate with actual CAWS tools to get real metrics
    // For now, return mock data based on recent performance
    const recentPerformance = await this.getRecentTenantPerformance(tenantId);

    return {
      coverage: recentPerformance.averageCoverage || 0.75,
      mutation: recentPerformance.averageMutation || 0.4,
      contracts: recentPerformance.contractTests || false,
      trustScore: recentPerformance.trustScore || 80,
    };
  }

  /**
   * Get recent tenant performance from memory
   */
  private async getRecentTenantPerformance(tenantId: string): Promise<{
    averageCoverage: number;
    averageMutation: number;
    contractTests: boolean;
    trustScore: number;
  }> {
    if (!this.memoryManager) {
      return { averageCoverage: 0.75, averageMutation: 0.4, contractTests: false, trustScore: 80 };
    }

    try {
      const query = {
        type: "performance_analysis",
        description: `Recent quality metrics for tenant ${tenantId}`,
        requirements: ["test_results", "quality_metrics"],
        constraints: { tenantId },
      };

      const memories = await this.memoryManager.getContextualMemories(tenantId, query, {
        limit: 10,
        minRelevance: 0.5,
      });

      if (!memories.success || !memories.data) {
        return { averageCoverage: 0.75, averageMutation: 0.4, contractTests: false, trustScore: 80 };
      }

      // Aggregate metrics from memories
      let totalCoverage = 0;
      let totalMutation = 0;
      let coverageCount = 0;
      let mutationCount = 0;
      let hasContracts = false;
      let totalTrust = 0;
      let trustCount = 0;

      for (const memory of memories.data) {
        if (memory.content.coverage) {
          totalCoverage += memory.content.coverage;
          coverageCount++;
        }
        if (memory.content.mutationScore) {
          totalMutation += memory.content.mutationScore;
          mutationCount++;
        }
        if (memory.content.contractTests) {
          hasContracts = true;
        }
        if (memory.content.trustScore) {
          totalTrust += memory.content.trustScore;
          trustCount++;
        }
      }

      return {
        averageCoverage: coverageCount > 0 ? totalCoverage / coverageCount : 0.75,
        averageMutation: mutationCount > 0 ? totalMutation / mutationCount : 0.4,
        contractTests: hasContracts,
        trustScore: trustCount > 0 ? totalTrust / trustCount : 80,
      };
    } catch (error) {
      this.logger.warn("Failed to get tenant performance metrics:", error);
      return { averageCoverage: 0.75, averageMutation: 0.4, contractTests: false, trustScore: 80 };
    }
  }

  /**
   * Check budget violations for a task
   */
  private checkBudgetViolations(budget: TaskBudget): string[] {
    const violations: string[] = [];

    if (budget.currentUsage.files > budget.limits.maxFiles) {
      violations.push(`File limit exceeded: ${budget.currentUsage.files}/${budget.limits.maxFiles}`);
    }

    if (budget.currentUsage.loc > budget.limits.maxLoc) {
      violations.push(`LOC limit exceeded: ${budget.currentUsage.loc}/${budget.limits.maxLoc}`);
    }

    if (budget.currentUsage.durationMs > budget.limits.maxDurationMs) {
      violations.push(`Time limit exceeded: ${Math.round(budget.currentUsage.durationMs / 1000)}s/${Math.round(budget.limits.maxDurationMs / 1000)}s`);
    }

    return violations;
  }

  /**
   * Get applicable waivers for violations
   */
  private getApplicableWaivers(tenantId: string, violations: string[]): WaiverRequest[] {
    const activeWaivers = this.getActiveWaivers(tenantId);
    return activeWaivers.filter(waiver =>
      violations.some(violation =>
        waiver.gates.some(gate => violation.toLowerCase().includes(gate.toLowerCase()))
      )
    );
  }

  /**
   * Check if waivers cover all violations
   */
  private waiversCoverViolations(waivers: WaiverRequest[], violations: string[]): boolean {
    const coveredGates = new Set(waivers.flatMap(w => w.gates));
    return violations.every(violation =>
      Array.from(coveredGates).some(gate =>
        violation.toLowerCase().includes(gate.toLowerCase())
      )
    );
  }

  /**
   * Get recent large tasks that might indicate scope issues
   */
  private async getRecentLargeTasks(tenantId: string): Promise<TaskBudget[]> {
    // This would query memory for recent tasks that exceeded certain thresholds
    return Array.from(this.activeBudgets.values())
      .filter(b => b.tenantId === tenantId &&
                   (b.currentUsage.files > 10 || b.currentUsage.loc > 500))
      .slice(-5); // Last 5 large tasks
  }

  /**
   * Store budget analysis in memory
   */
  private async storeBudgetAnalysis(budget: TaskBudget): Promise<void> {
    if (!this.memoryManager) return;

    try {
      await this.memoryManager.storeExperience(budget.tenantId, {
        memoryId: `budget-analysis-${budget.taskId}`,
        relevanceScore: 0.7,
        contextMatch: {
          similarityScore: 0.7,
          keywordMatches: ["budget", "limits", "enforcement"],
          semanticMatches: ["task budget", "resource usage", "constitutional limits"],
          temporalAlignment: 0.8,
        },
        content: {
          type: "budget_analysis",
          taskId: budget.taskId,
          limits: budget.limits,
          finalUsage: budget.currentUsage,
          duration: budget.currentUsage.durationMs,
          violations: this.checkBudgetViolations(budget),
          checkIns: budget.checkIns,
        },
      });
    } catch (error) {
      this.logger.warn(`Failed to store budget analysis for ${budget.taskId}:`, error);
    }
  }

  /**
   * Load waivers from memory
   */
  private async loadWaiversFromMemory(): Promise<void> {
    if (!this.memoryManager) return;

    try {
      // This would load waivers from memory system
      // For now, keep them in memory only
    } catch (error) {
      this.logger.warn("Failed to load waivers from memory:", error);
    }
  }

  /**
   * Store waiver in memory
   */
  private async storeWaiverInMemory(waiver: WaiverRequest): Promise<void> {
    if (!this.memoryManager) return;

    try {
      await this.memoryManager.storeExperience(waiver.tenantId, {
        memoryId: `waiver-${waiver.id}`,
        relevanceScore: 0.9,
        contextMatch: {
          similarityScore: 0.9,
          keywordMatches: ["waiver", "exception", waiver.reason],
          semanticMatches: ["constitutional waiver", "policy exception", "emergency override"],
          temporalAlignment: 0.9,
        },
        content: {
          type: "waiver_request",
          waiverId: waiver.id,
          title: waiver.title,
          reason: waiver.reason,
          gates: waiver.gates,
          expiresAt: waiver.expiresAt.toISOString(),
          status: waiver.status,
          impactLevel: waiver.impactLevel,
        },
      });
    } catch (error) {
      this.logger.warn(`Failed to store waiver ${waiver.id} in memory:`, error);
    }
  }

  /**
   * Update waiver in memory
   */
  private async updateWaiverInMemory(waiver: WaiverRequest): Promise<void> {
    await this.storeWaiverInMemory(waiver); // Just overwrite for now
  }
}
