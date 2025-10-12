/**
 * Constitutional Runtime
 *
 * Main entry point for CAWS constitutional compliance validation.
 * Orchestrates policy evaluation, violation handling, and waiver management.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import {
  OperationContext,
  EvaluationContext,
  ComplianceResult,
  ConstitutionalViolation,
  WaiverCheckResult,
  AuditResult,
  ConstitutionalConfig,
} from "../types/caws-constitutional";

import { ConstitutionalPolicyEngine } from "./ConstitutionalPolicyEngine";
import { ViolationHandler } from "./ViolationHandler";
import { WaiverManager } from "./WaiverManager";
import { TracingProvider } from "../observability/TracingProvider";

export class ConstitutionalRuntime extends EventEmitter {
  private config: ConstitutionalConfig;

  constructor(
    private policyEngine: ConstitutionalPolicyEngine,
    private violationHandler: ViolationHandler,
    private waiverManager: WaiverManager,
    private tracing: TracingProvider,
    config?: Partial<ConstitutionalConfig>
  ) {
    super();

    this.config = {
      enabled: true,
      strictMode: false,
      auditEnabled: true,
      violationResponseTimeout: 5000,
      waiverApprovalRequired: true,
      maxViolationsPerOperation: 10,
      cacheEnabled: true,
      cacheTTL: 300000, // 5 minutes
      ...config,
    };

    this.setupEventHandlers();
  }

  /**
   * Validate operation compliance before execution
   */
  async validateOperation(
    operation: OperationContext,
    context: EvaluationContext
  ): Promise<ComplianceResult> {
    if (!this.config.enabled) {
      return {
        operationId: operation.id,
        compliant: true,
        violations: [],
        evaluations: [],
        timestamp: new Date(),
        duration: 0,
      };
    }

    return this.tracing.traceOperation("constitutional:validateOperation", async () => {
      const startTime = Date.now();

      // Check for active waivers first
      const waiverCheck = await this.waiverManager.checkWaiver(operation, context);

      if (waiverCheck.hasActiveWaiver) {
        this.emit("operation:waiver-applied", {
          operationId: operation.id,
          waiverId: waiverCheck.waiver?.id,
          timestamp: new Date(),
        });

        return {
          operationId: operation.id,
          compliant: true,
          violations: [],
          evaluations: [],
          timestamp: new Date(),
          duration: Date.now() - startTime,
          waiverApplied: true,
          waiverId: waiverCheck.waiver?.id,
        };
      }

      // Evaluate against policies
      const result = await this.policyEngine.evaluateCompliance(operation, context);

      // Handle violations
      if (!result.compliant && result.violations.length > 0) {
        // Limit violations to prevent overload
        const violations = result.violations.slice(0, this.config.maxViolationsPerOperation);

        await this.violationHandler.handleViolations(
          violations,
          operation,
          context,
          this.config.violationResponseTimeout
        );

        this.emit("operation:violations-detected", {
          operationId: operation.id,
          violationCount: violations.length,
          violations: violations.map(v => ({
            id: v.id,
            principle: v.principle,
            severity: v.severity,
            message: v.message,
          })),
          timestamp: new Date(),
        });
      }

      result.duration = Date.now() - startTime;

      this.emit("operation:validated", {
        operationId: operation.id,
        compliant: result.compliant,
        violationCount: result.violations.length,
        duration: result.duration,
        timestamp: new Date(),
      });

      return result;
    });
  }

  /**
   * Monitor operation execution for ongoing compliance
   */
  async monitorOperation(
    operation: OperationContext,
    executionContext: any,
    context: EvaluationContext
  ): Promise<void> {
    if (!this.config.enabled) return;

    // In a full implementation, this would:
    // 1. Monitor execution in real-time
    // 2. Check for violations during execution
    // 3. Apply policies that require continuous monitoring

    this.emit("operation:monitoring-started", {
      operationId: operation.id,
      timestamp: new Date(),
    });
  }

  /**
   * Audit completed operation
   */
  async auditOperation(
    operation: OperationContext,
    result: any,
    context: EvaluationContext
  ): Promise<AuditResult> {
    if (!this.config.auditEnabled) {
      return {
        operationId: operation.id,
        compliant: true,
        violations: [],
        recommendations: [],
        score: 100,
        timestamp: new Date(),
        auditorVersion: "1.0.0",
      };
    }

    return this.tracing.traceOperation("constitutional:auditOperation", async () => {
      // Create audit operation context
      const auditOperation: OperationContext = {
        id: `audit-${operation.id}`,
        type: "operation_audit",
        timestamp: new Date(),
        agentId: operation.agentId,
        userId: operation.userId,
        sessionId: operation.sessionId,
        payload: {
          originalOperation: operation,
          executionResult: result,
        },
        metadata: {
          auditType: "post_execution",
          originalOperationId: operation.id,
        },
      };

      // Evaluate audit compliance
      const compliance = await this.policyEngine.evaluateCompliance(
        auditOperation,
        context
      );

      // Calculate compliance score (0-100)
      const score = this.calculateComplianceScore(compliance);

      // Generate recommendations
      const recommendations = this.generateRecommendations(compliance);

      const auditResult: AuditResult = {
        operationId: operation.id,
        compliant: compliance.compliant,
        violations: compliance.violations,
        recommendations,
        score,
        timestamp: new Date(),
        auditorVersion: "1.0.0",
      };

      this.emit("operation:audited", {
        operationId: operation.id,
        score,
        violationCount: compliance.violations.length,
        timestamp: new Date(),
      });

      return auditResult;
    });
  }

  /**
   * Request a waiver for policy violations
   */
  async requestWaiver(
    policyId: string,
    operationPattern: string,
    reason: string,
    justification: string,
    requestedBy: string,
    expiresAt: Date
  ): Promise<string> {
    return this.waiverManager.requestWaiver({
      policyId,
      operationPattern,
      reason,
      justification,
      requestedBy,
      expiresAt,
    });
  }

  /**
   * Approve a waiver request
   */
  async approveWaiver(waiverId: string, approvedBy: string): Promise<void> {
    await this.waiverManager.approveWaiver(waiverId, approvedBy);

    this.emit("waiver:approved", {
      waiverId,
      approvedBy,
      timestamp: new Date(),
    });
  }

  /**
   * Reject a waiver request
   */
  async rejectWaiver(
    waiverId: string,
    rejectedBy: string,
    reason: string
  ): Promise<void> {
    await this.waiverManager.rejectWaiver(waiverId, rejectedBy, reason);

    this.emit("waiver:rejected", {
      waiverId,
      rejectedBy,
      reason,
      timestamp: new Date(),
    });
  }

  /**
   * Check waiver status for operation
   */
  async checkWaiver(
    operation: OperationContext,
    context: EvaluationContext
  ): Promise<WaiverCheckResult> {
    return this.waiverManager.checkWaiver(operation, context);
  }

  /**
   * Get runtime statistics
   */
  getStats(): {
    policies: number;
    waivers: Record<string, number>;
    recentViolations: number;
  } {
    const waivers = this.waiverManager.getWaivers();
    const waiverStats = waivers.reduce((acc, waiver) => {
      acc[waiver.status] = (acc[waiver.status] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    // In a full implementation, track recent violations
    const recentViolations = 0;

    return {
      policies: this.policyEngine.getPolicies().length,
      waivers: waiverStats,
      recentViolations,
    };
  }

  /**
   * Update configuration
   */
  updateConfig(newConfig: Partial<ConstitutionalConfig>): void {
    this.config = { ...this.config, ...newConfig };
  }

  /**
   * Enable/disable constitutional validation
   */
  setEnabled(enabled: boolean): void {
    this.config.enabled = enabled;
  }

  /**
   * Check if constitutional validation is enabled
   */
  isEnabled(): boolean {
    return this.config.enabled;
  }

  /**
   * Calculate compliance score from evaluation results
   */
  private calculateComplianceScore(compliance: ComplianceResult): number {
    if (compliance.violations.length === 0) {
      return 100;
    }

    // Weight violations by severity
    const violationWeights = compliance.violations.reduce((total, violation) => {
      switch (violation.severity) {
        case "low": return total + 5;
        case "medium": return total + 15;
        case "high": return total + 30;
        case "critical": return total + 50;
        default: return total + 10;
      }
    }, 0);

    return Math.max(0, 100 - violationWeights);
  }

  /**
   * Generate recommendations based on violations
   */
  private generateRecommendations(compliance: ComplianceResult): string[] {
    const recommendations: string[] = [];

    if (compliance.violations.length === 0) {
      return ["Operation is fully compliant with CAWS principles."];
    }

    // Group violations by principle
    const violationsByPrinciple = compliance.violations.reduce((acc, violation) => {
      if (!acc[violation.principle]) {
        acc[violation.principle] = [];
      }
      acc[violation.principle].push(violation);
      return acc;
    }, {} as Record<string, ConstitutionalViolation[]>);

    // Generate recommendations for each principle
    for (const [principle, violations] of Object.entries(violationsByPrinciple)) {
      switch (principle) {
        case "transparency":
          recommendations.push("Consider adding more detailed audit logging to improve transparency.");
          break;
        case "accountability":
          recommendations.push("Ensure all operations include proper user and agent identification.");
          break;
        case "safety":
          recommendations.push("Review operation parameters to ensure system safety constraints.");
          break;
        case "fairness":
          recommendations.push("Audit decision-making processes for potential bias.");
          break;
        case "privacy":
          recommendations.push("Verify data handling complies with privacy regulations.");
          break;
        case "reliability":
          recommendations.push("Implement additional error handling and monitoring.");
          break;
      }
    }

    // Add waiver recommendation if violations are present
    if (compliance.violations.some(v => v.severity === "high" || v.severity === "critical")) {
      recommendations.push("Consider requesting a waiver for exceptional circumstances.");
    }

    return recommendations;
  }

  /**
   * Setup event handlers
   */
  private setupEventHandlers(): void {
    // Forward events from components
    this.violationHandler.on("violation:handled", (event) => {
      this.emit("violation:handled", event);
    });

    this.waiverManager.on("waiver:created", (event) => {
      this.emit("waiver:created", event);
    });
  }
}
