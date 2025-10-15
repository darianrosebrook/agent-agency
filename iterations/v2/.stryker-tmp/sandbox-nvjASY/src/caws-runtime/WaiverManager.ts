/**
 * Waiver Manager
 *
 * Manages temporary exceptions to constitutional policies.
 * Handles waiver requests, approvals, and expiration.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { EventEmitter } from "events";
import {
  WaiverRequest,
  WaiverStatus,
  WaiverCheckResult,
  OperationContext,
  EvaluationContext,
} from "../types/caws-constitutional";

export class WaiverManager extends EventEmitter {
  private waivers: Map<string, WaiverRequest> = new Map();

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
      id: `waiver-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
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
  async revokeWaiver(waiverId: string, revokedBy: string, reason: string): Promise<void> {
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
    context: EvaluationContext
  ): Promise<WaiverCheckResult> {
    // Expire old waivers first
    this.expireWaivers();

    const activeWaivers = Array.from(this.waivers.values())
      .filter((waiver) =>
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
      if (
        waiver.status === WaiverStatus.APPROVED &&
        now > waiver.expiresAt
      ) {
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
  private matchesOperation(pattern: string, operation: OperationContext): boolean {
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
    // In a real implementation, this would send notifications
    // via email, Slack, or other communication channels
    console.log(`Waiver ${waiver.id} requires approval for policy ${waiver.policyId}`);
    console.log(`Reason: ${waiver.reason}`);
    console.log(`Requested by: ${waiver.requestedBy}`);
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
    // In a real implementation, this would write to audit logs
    const logEntry = {
      waiverId: waiver.id,
      policyId: waiver.policyId,
      action,
      actor,
      details,
      timestamp: new Date(),
    };

    console.log(`Waiver ${waiver.id} ${action} by ${actor}${details ? `: ${details}` : ""}`);
  }

  /**
   * Clear all waivers
   */
  clear(): void {
    this.waivers.clear();
    this.emit("waivers:cleared");
  }
}
