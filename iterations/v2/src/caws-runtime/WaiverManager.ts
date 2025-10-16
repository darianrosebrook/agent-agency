/**
 * Waiver Manager
 *
 * Manages temporary exceptions to constitutional policies.
 * Handles waiver requests, approvals, and expiration.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { AuditConfig, AuditLogger } from "../adapters/AuditLogger.js";
import {
  NotificationAdapter,
  NotificationConfig,
  NotificationMessage,
} from "../adapters/NotificationAdapter.js";
import { Logger } from "../observability/Logger.js";
import {
  EvaluationContext,
  OperationContext,
  WaiverCheckResult,
  WaiverRequest,
  WaiverStatus,
} from "../types/caws-constitutional";

export class WaiverManager extends EventEmitter {
  private waivers: Map<string, WaiverRequest> = new Map();
  private notificationAdapter?: NotificationAdapter;
  private auditLogger?: AuditLogger;
  private logger: Logger;

  constructor(
    notificationConfig?: NotificationConfig,
    auditConfig?: AuditConfig,
    logger?: Logger
  ) {
    super();
    this.logger = logger || new Logger("WaiverManager");

    // Initialize notification adapter
    if (notificationConfig) {
      this.notificationAdapter = new NotificationAdapter(
        notificationConfig,
        this.logger
      );
    }

    // Initialize audit logger
    if (auditConfig) {
      this.auditLogger = new AuditLogger(auditConfig, this.logger);
    }
  }

  /**
   * Request a waiver for policy violations
   */
  async requestWaiver(request: {
    policyId: string;
    operationPattern: string;
    reason: string;
    justification: string;
    requestedBy: string;
    expiresAt: Date;
    metadata?: Record<string, any>;
  }): Promise<string> {
    const waiver: WaiverRequest = {
      id: `waiver-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      policyId: request.policyId,
      operationPattern: request.operationPattern,
      reason: request.reason,
      justification: request.justification,
      requestedBy: request.requestedBy,
      expiresAt: request.expiresAt,
      status: WaiverStatus.PENDING,
      createdAt: new Date(),
      updatedAt: new Date(),
      metadata: request.metadata,
    };

    this.waivers.set(waiver.id, waiver);

    this.emit("waiver:created", {
      waiverId: waiver.id,
      policyId: waiver.policyId,
      requestedBy: waiver.requestedBy,
      timestamp: new Date(),
    });

    // In a real implementation, notify approvers
    await this.notifyApprovers(waiver);

    return waiver.id;
  }

  /**
   * Approve a waiver request
   */
  async approveWaiver(waiverId: string, approvedBy: string): Promise<void> {
    const waiver = this.waivers.get(waiverId);
    if (!waiver) {
      throw new Error(`Waiver ${waiverId} not found`);
    }

    if (waiver.status !== WaiverStatus.PENDING) {
      throw new Error(`Waiver ${waiverId} is not in pending status`);
    }

    waiver.status = WaiverStatus.APPROVED;
    waiver.approvedBy = approvedBy;
    waiver.updatedAt = new Date();

    this.emit("waiver:approved", {
      waiverId,
      approvedBy,
      policyId: waiver.policyId,
      timestamp: new Date(),
    });

    await this.logWaiverAction(waiver, "approved", approvedBy);
  }

  /**
   * Reject a waiver request
   */
  async rejectWaiver(
    waiverId: string,
    rejectedBy: string,
    reason: string
  ): Promise<void> {
    const waiver = this.waivers.get(waiverId);
    if (!waiver) {
      throw new Error(`Waiver ${waiverId} not found`);
    }

    if (waiver.status !== WaiverStatus.PENDING) {
      throw new Error(`Waiver ${waiverId} is not in pending status`);
    }

    waiver.status = WaiverStatus.REJECTED;
    waiver.updatedAt = new Date();

    this.emit("waiver:rejected", {
      waiverId,
      rejectedBy,
      reason,
      policyId: waiver.policyId,
      timestamp: new Date(),
    });

    await this.logWaiverAction(waiver, "rejected", rejectedBy, reason);
  }

  /**
   * Revoke an approved waiver
   */
  async revokeWaiver(
    waiverId: string,
    revokedBy: string,
    reason: string
  ): Promise<void> {
    const waiver = this.waivers.get(waiverId);
    if (!waiver) {
      throw new Error(`Waiver ${waiverId} not found`);
    }

    waiver.status = WaiverStatus.REVOKED;
    waiver.updatedAt = new Date();

    this.emit("waiver:revoked", {
      waiverId,
      revokedBy,
      reason,
      policyId: waiver.policyId,
      timestamp: new Date(),
    });

    await this.logWaiverAction(waiver, "revoked", revokedBy, reason);
  }

  /**
   * Check if operation has an active waiver
   */
  async checkWaiver(
    operation: OperationContext,
    _context: EvaluationContext
  ): Promise<WaiverCheckResult> {
    // Expire old waivers first
    this.expireWaivers();

    const activeWaivers = Array.from(this.waivers.values())
      .filter(
        (waiver) =>
          waiver.status === WaiverStatus.APPROVED &&
          new Date() < waiver.expiresAt &&
          this.matchesOperation(waiver.operationPattern, operation)
      )
      .sort((a, b) => a.createdAt.getTime() - b.createdAt.getTime()); // Oldest first

    if (activeWaivers.length > 0) {
      const waiver = activeWaivers[0]; // Use the first matching waiver
      const remainingTime = waiver.expiresAt.getTime() - Date.now();

      return {
        hasActiveWaiver: true,
        waiver,
        expiresAt: waiver.expiresAt,
        remainingTime: Math.max(0, remainingTime),
      };
    }

    return { hasActiveWaiver: false };
  }

  /**
   * Get all waivers with optional status filter
   */
  getWaivers(status?: WaiverStatus): WaiverRequest[] {
    const allWaivers = Array.from(this.waivers.values());

    if (status) {
      return allWaivers.filter((waiver) => waiver.status === status);
    }

    return allWaivers;
  }

  /**
   * Get waiver by ID
   */
  getWaiver(waiverId: string): WaiverRequest | undefined {
    return this.waivers.get(waiverId);
  }

  /**
   * Get waivers for a specific policy
   */
  getWaiversForPolicy(policyId: string): WaiverRequest[] {
    return Array.from(this.waivers.values()).filter(
      (waiver) => waiver.policyId === policyId
    );
  }

  /**
   * Expire waivers that have passed their expiration date
   */
  expireWaivers(): void {
    const now = new Date();
    let expiredCount = 0;

    for (const waiver of this.waivers.values()) {
      if (waiver.status === WaiverStatus.APPROVED && now > waiver.expiresAt) {
        waiver.status = WaiverStatus.EXPIRED;
        waiver.updatedAt = now;
        expiredCount++;

        this.emit("waiver:expired", {
          waiverId: waiver.id,
          policyId: waiver.policyId,
          timestamp: new Date(),
        });
      }
    }

    if (expiredCount > 0) {
      console.log(`Expired ${expiredCount} waivers`);
    }
  }

  /**
   * Clean up old waivers (older than specified days)
   */
  cleanupOldWaivers(maxAgeDays: number = 90): void {
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - maxAgeDays);

    const toDelete: string[] = [];

    for (const [id, waiver] of this.waivers.entries()) {
      if (waiver.createdAt < cutoffDate) {
        toDelete.push(id);
      }
    }

    for (const id of toDelete) {
      this.waivers.delete(id);
    }

    if (toDelete.length > 0) {
      console.log(`Cleaned up ${toDelete.length} old waivers`);
    }
  }

  /**
   * Get waiver statistics
   */
  getStats(): {
    total: number;
    pending: number;
    approved: number;
    rejected: number;
    expired: number;
    revoked: number;
  } {
    const waivers = Array.from(this.waivers.values());
    const stats = {
      total: waivers.length,
      pending: 0,
      approved: 0,
      rejected: 0,
      expired: 0,
      revoked: 0,
    };

    for (const waiver of waivers) {
      switch (waiver.status) {
        case WaiverStatus.PENDING:
          stats.pending++;
          break;
        case WaiverStatus.APPROVED:
          stats.approved++;
          break;
        case WaiverStatus.REJECTED:
          stats.rejected++;
          break;
        case WaiverStatus.EXPIRED:
          stats.expired++;
          break;
        case WaiverStatus.REVOKED:
          stats.revoked++;
          break;
      }
    }

    return stats;
  }

  /**
   * Check if operation matches waiver pattern
   */
  private matchesOperation(
    pattern: string,
    operation: OperationContext
  ): boolean {
    // Simple pattern matching - in production, use regex or more sophisticated matching
    const operationString = [
      operation.type,
      operation.id,
      operation.agentId,
      operation.userId,
      operation.sessionId,
      JSON.stringify(operation.payload),
    ]
      .filter(Boolean)
      .join(" ");

    return operationString.toLowerCase().includes(pattern.toLowerCase());
  }

  /**
   * Notify approvers of waiver request
   */
  private async notifyApprovers(waiver: WaiverRequest): Promise<void> {
    if (!this.notificationAdapter) {
      this.logger.warn(
        "No notification adapter configured, skipping approver notification"
      );
      return;
    }

    try {
      const message: NotificationMessage = {
        title: "Waiver Request Requires Approval",
        body: `A waiver request has been submitted for policy ${
          waiver.policyId
        }.

Request Details:
- Waiver ID: ${waiver.id}
- Policy: ${waiver.policyId}
- Operation Pattern: ${waiver.operationPattern}
- Reason: ${waiver.reason}
- Justification: ${waiver.justification}
- Requested by: ${waiver.requestedBy}
- Expires: ${waiver.expiresAt.toISOString()}

Please review and approve or reject this waiver request.`,
        priority: "high",
        metadata: {
          waiverId: waiver.id,
          policyId: waiver.policyId,
          requestedBy: waiver.requestedBy,
          expiresAt: waiver.expiresAt.toISOString(),
        },
      };

      const results = await this.notificationAdapter.sendToDefaultRecipients(
        message
      );

      const successCount = results.filter((r) => r.success).length;
      const failureCount = results.filter((r) => !r.success).length;

      this.logger.info("Waiver approval notifications sent", {
        waiverId: waiver.id,
        successCount,
        failureCount,
        totalRecipients: results.length,
      });

      if (failureCount > 0) {
        this.logger.warn("Some waiver notifications failed", {
          waiverId: waiver.id,
          failures: results
            .filter((r) => !r.success)
            .map((r) => ({
              recipientId: r.recipientId,
              channel: r.channel,
              error: r.error,
            })),
        });
      }
    } catch (error) {
      this.logger.error("Failed to send waiver approval notifications", {
        waiverId: waiver.id,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }

  /**
   * Log waiver action to audit trail
   */
  private async logWaiverAction(
    waiver: WaiverRequest,
    action: string,
    actor: string,
    details?: string
  ): Promise<void> {
    if (!this.auditLogger) {
      this.logger.warn("No audit logger configured, skipping audit log");
      return;
    }

    try {
      // Determine severity based on action
      let severity: "low" | "medium" | "high" | "critical" = "medium";
      if (action === "approved" || action === "rejected") {
        severity = "high";
      } else if (action === "revoked") {
        severity = "critical";
      }

      // Determine outcome
      let outcome: "success" | "failure" | "partial" = "success";
      if (action === "rejected") {
        outcome = "failure";
      }

      await this.auditLogger.logEvent({
        type: "waiver_action",
        actor: {
          id: actor,
          type: "user",
          name: actor,
        },
        resource: {
          type: "waiver",
          id: waiver.id,
          name: `Policy ${waiver.policyId} Waiver`,
        },
        action: action,
        outcome: outcome,
        metadata: {
          waiverId: waiver.id,
          policyId: waiver.policyId,
          operationPattern: waiver.operationPattern,
          reason: waiver.reason,
          justification: waiver.justification,
          requestedBy: waiver.requestedBy,
          status: waiver.status,
          expiresAt: waiver.expiresAt.toISOString(),
          details,
        },
        compliance: {
          severity,
          regulations: ["SOX", "GDPR", "HIPAA"],
          dataClassification: "confidential",
          retentionPeriod: 2555, // 7 years
        },
        timestamp: new Date(),
        sessionId: waiver.id,
      });

      this.logger.debug("Waiver action logged to audit trail", {
        waiverId: waiver.id,
        action,
        actor,
      });
    } catch (error) {
      this.logger.error("Failed to log waiver action to audit trail", {
        waiverId: waiver.id,
        action,
        actor,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }

  /**
   * Clear all waivers
   */
  clear(): void {
    this.waivers.clear();
    this.emit("waivers:cleared");
  }

  /**
   * Health check for notification and audit systems
   */
  async healthCheck(): Promise<{
    healthy: boolean;
    notifications: boolean;
    audit: boolean;
    activeWaivers: number;
  }> {
    const health = {
      healthy: true,
      notifications: true,
      audit: true,
      activeWaivers: this.waivers.size,
    };

    // Check notification adapter health
    if (this.notificationAdapter) {
      try {
        const notificationHealth = await this.notificationAdapter.healthCheck();
        health.notifications = notificationHealth.healthy;
        if (!notificationHealth.healthy) health.healthy = false;
      } catch (error) {
        health.notifications = false;
        health.healthy = false;
        this.logger.error("Notification adapter health check failed", {
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }

    // Check audit logger health
    if (this.auditLogger) {
      try {
        const auditHealth = await this.auditLogger.healthCheck();
        health.audit = auditHealth.healthy;
        if (!auditHealth.healthy) health.healthy = false;
      } catch (error) {
        health.audit = false;
        health.healthy = false;
        this.logger.error("Audit logger health check failed", {
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }

    return health;
  }
}
