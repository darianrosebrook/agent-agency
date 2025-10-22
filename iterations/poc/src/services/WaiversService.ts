/**
 * Central Waivers Service
 *
 * Extracted from CAWS waivers management to provide centralized
 * waiver and exception handling across the system.
 *
 * @author @darianrosebrook
 */

import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";
import { MultiTenantMemoryManager } from "../memory/MultiTenantMemoryManager.js";
import { Logger } from "../utils/Logger.js";
import type { WaiverRequest } from "../types/index.js";

export interface WaiverFilter {
  tenantId?: string;
  gate?: string;
  status?: WaiverRequest["status"];
  requester?: string;
  approver?: string;
  impactLevel?: WaiverRequest["impactLevel"];
  reason?: WaiverRequest["reason"];
}

export interface WaiverApprovalRequest {
  waiverId: string;
  approved: boolean;
  approver: string;
  notes?: string;
  approvalCriteria?: {
    riskAssessment: "low" | "medium" | "high";
    mitigationVerified: boolean;
    stakeholderNotified: boolean;
  };
}

export interface WaiverAnalytics {
  totalWaivers: number;
  activeWaivers: number;
  waiversByGate: Record<string, number>;
  waiversByStatus: Record<string, number>;
  waiversByImpact: Record<string, number>;
  recentApprovals: Array<{
    id: string;
    gate: string;
    approvedAt: string;
    approver: string;
  }>;
  averageApprovalTime: number; // in hours
}

export class WaiversService {
  private logger: Logger;
  private memoryManager?: MultiTenantMemoryManager;
  private waivers = new Map<string, WaiverRequest>();

  // Waiver approval workflow
  private approvalWorkflow = {
    low: { minApprovers: 1, maxApprovalTime: 24 }, // hours
    medium: { minApprovers: 2, maxApprovalTime: 72 },
    high: { minApprovers: 3, maxApprovalTime: 168 }, // 1 week
    critical: { minApprovers: 5, maxApprovalTime: 336 }, // 2 weeks
  };

  constructor(memoryManager?: MultiTenantMemoryManager) {
    this.logger = new Logger("WaiversService");
    this.memoryManager = memoryManager;
    this.loadWaiversFromMemory();
  }

  /**
   * Request a new waiver
   */
  async requestWaiver(
    request: Omit<WaiverRequest, "id" | "requestedAt" | "status">
  ): Promise<string> {
    const waiverId = this.generateWaiverId();

    const waiver: WaiverRequest = {
      ...request,
      id: waiverId,
      requestedAt: new Date(),
      status: "pending",
    };

    // Validate waiver request
    this.validateWaiverRequest(waiver);

    this.waivers.set(waiverId, waiver);

    // Store in memory for persistence
    await this.storeWaiverInMemory(waiver);

    // Initialize approval workflow
    await this.initializeApprovalWorkflow(waiver);

    this.logger.info(
      `Waiver requested: ${waiverId} (${request.title}) - Impact: ${request.impactLevel}`
    );

    return waiverId;
  }

  /**
   * Process waiver approval/rejection
   */
  async processWaiverApproval(
    request: WaiverApprovalRequest
  ): Promise<boolean> {
    const waiver = this.waivers.get(request.waiverId);
    if (!waiver) {
      throw new Error(`Waiver ${request.waiverId} not found`);
    }

    if (waiver.status !== "pending") {
      throw new Error(`Waiver ${request.waiverId} is already ${waiver.status}`);
    }

    // Validate approval criteria based on impact level
    if (request.approved) {
      await this.validateApprovalCriteria(waiver, request);
    }

    waiver.status = request.approved ? "approved" : "rejected";
    waiver.approvedBy = request.approver;

    // Update in memory
    await this.updateWaiverInMemory(waiver);

    // Log approval decision
    await this.logApprovalDecision(waiver, request);

    this.logger.info(
      `Waiver ${request.waiverId} ${waiver.status} by ${request.approver}`
    );

    return request.approved;
  }

  /**
   * Get waiver by ID
   */
  getWaiver(waiverId: string): WaiverRequest | null {
    return this.waivers.get(waiverId) || null;
  }

  /**
   * Find waivers matching filter criteria
   */
  findWaivers(filter: WaiverFilter = {}): WaiverRequest[] {
    let waivers = Array.from(this.waivers.values());

    if (filter.tenantId) {
      waivers = waivers.filter((w) => w.tenantId === filter.tenantId);
    }

    if (filter.gate) {
      waivers = waivers.filter((w) => w.gates.includes(filter.gate!));
    }

    if (filter.status) {
      waivers = waivers.filter((w) => w.status === filter.status);
    }

    if (filter.requester) {
      waivers = waivers.filter((w) => w.approvedBy === filter.requester);
    }

    if (filter.approver) {
      waivers = waivers.filter((w) => w.approvedBy === filter.approver);
    }

    if (filter.impactLevel) {
      waivers = waivers.filter((w) => w.impactLevel === filter.impactLevel);
    }

    if (filter.reason) {
      waivers = waivers.filter((w) => w.reason === filter.reason);
    }

    return waivers.sort(
      (a, b) => b.requestedAt.getTime() - a.requestedAt.getTime()
    );
  }

  /**
   * Get active waivers for a tenant
   */
  getActiveWaivers(tenantId: string): WaiverRequest[] {
    const now = new Date();
    return Array.from(this.waivers.values()).filter(
      (w) =>
        w.tenantId === tenantId && w.status === "approved" && w.expiresAt > now
    );
  }

  /**
   * Check if waivers cover specific violations
   */
  waiversCoverViolations(
    waivers: WaiverRequest[],
    violations: string[]
  ): boolean {
    const coveredGates = new Set(waivers.flatMap((w) => w.gates));
    return violations.every((violation) =>
      Array.from(coveredGates).some((gate) =>
        violation.toLowerCase().includes(gate.toLowerCase())
      )
    );
  }

  /**
   * Get waivers that cover specific violations
   */
  getWaiversForViolations(
    tenantId: string,
    violations: string[]
  ): WaiverRequest[] {
    const activeWaivers = this.getActiveWaivers(tenantId);
    return activeWaivers.filter((waiver) =>
      violations.some((violation) =>
        waiver.gates.some((gate) =>
          violation.toLowerCase().includes(gate.toLowerCase())
        )
      )
    );
  }

  /**
   * Expire waivers that have passed their expiry date
   */
  async expireWaivers(): Promise<number> {
    const now = new Date();
    let expiredCount = 0;

    for (const [id, waiver] of this.waivers) {
      if (waiver.status === "approved" && waiver.expiresAt <= now) {
        waiver.status = "expired";
        await this.updateWaiverInMemory(waiver);
        expiredCount++;

        this.logger.info(`Waiver ${id} expired`);
      }
    }

    if (expiredCount > 0) {
      this.logger.info(`Expired ${expiredCount} waiver(s)`);
    }

    return expiredCount;
  }

  /**
   * Get waiver analytics
   */
  getAnalytics(): WaiverAnalytics {
    const allWaivers = Array.from(this.waivers.values());
    const now = new Date();

    // Count by various dimensions
    const waiversByGate: Record<string, number> = {};
    const waiversByStatus: Record<string, number> = {};
    const waiversByImpact: Record<string, number> = {};

    for (const waiver of allWaivers) {
      // By gate
      for (const gate of waiver.gates) {
        waiversByGate[gate] = (waiversByGate[gate] || 0) + 1;
      }

      // By status
      waiversByStatus[waiver.status] =
        (waiversByStatus[waiver.status] || 0) + 1;

      // By impact
      waiversByImpact[waiver.impactLevel] =
        (waiversByImpact[waiver.impactLevel] || 0) + 1;
    }

    // Recent approvals
    const recentApprovals = allWaivers
      .filter((w) => w.status === "approved")
      .sort((a, b) => b.requestedAt.getTime() - a.requestedAt.getTime())
      .slice(0, 10)
      .map((w) => ({
        id: w.id,
        gate: w.gates[0], // Primary gate
        approvedAt: w.requestedAt.toISOString(),
        approver: w.approvedBy,
      }));

    // Average approval time (for approved waivers)
    const approvedWaivers = allWaivers.filter((w) => w.status === "approved");
    const totalApprovalTime = approvedWaivers.reduce((sum, w) => {
      // Simplified: assume approval time is proportional to impact level
      const baseHours =
        { low: 4, medium: 12, high: 48, critical: 168 }[w.impactLevel] || 24;
      return sum + baseHours;
    }, 0);
    const averageApprovalTime =
      approvedWaivers.length > 0
        ? totalApprovalTime / approvedWaivers.length
        : 0;

    return {
      totalWaivers: allWaivers.length,
      activeWaivers: allWaivers.filter(
        (w) => w.status === "approved" && w.expiresAt > now
      ).length,
      waiversByGate,
      waiversByStatus,
      waiversByImpact,
      recentApprovals,
      averageApprovalTime,
    };
  }

  /**
   * Export waivers to file
   */
  exportWaivers(filePath: string): void {
    const waivers = Array.from(this.waivers.values());
    const exportData = {
      exportedAt: new Date().toISOString(),
      totalWaivers: waivers.length,
      waivers,
    };

    const dir = path.dirname(filePath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }

    fs.writeFileSync(filePath, JSON.stringify(exportData, null, 2));
    this.logger.info(`Exported ${waivers.length} waivers to ${filePath}`);
  }

  /**
   * Import waivers from file
   */
  importWaivers(filePath: string): void {
    if (!fs.existsSync(filePath)) {
      throw new Error(`Waivers file not found: ${filePath}`);
    }

    const content = fs.readFileSync(filePath, "utf8");
    const importData = JSON.parse(content);

    let importedCount = 0;
    for (const waiver of importData.waivers || []) {
      // Convert date strings back to Date objects
      waiver.requestedAt = new Date(waiver.requestedAt);
      waiver.expiresAt = new Date(waiver.expiresAt);

      this.waivers.set(waiver.id, waiver);
      importedCount++;
    }

    this.logger.info(`Imported ${importedCount} waivers from ${filePath}`);
  }

  /**
   * Validate waiver request
   */
  private validateWaiverRequest(waiver: WaiverRequest): void {
    if (!waiver.title || waiver.title.length < 10) {
      throw new Error("Waiver title must be at least 10 characters");
    }

    if (!waiver.description || waiver.description.length < 20) {
      throw new Error("Waiver description must be at least 20 characters");
    }

    if (!waiver.gates || waiver.gates.length === 0) {
      throw new Error("Waiver must specify at least one gate");
    }

    if (!waiver.reason) {
      throw new Error("Waiver must specify a reason");
    }

    if (!waiver.approvedBy) {
      throw new Error("Waiver must specify an approver");
    }

    if (!waiver.tenantId) {
      throw new Error("Waiver must specify a tenant ID");
    }

    if (!waiver.impactLevel) {
      throw new Error("Waiver must specify an impact level");
    }

    if (!waiver.mitigationPlan || waiver.mitigationPlan.length < 10) {
      throw new Error("Waiver must include a mitigation plan");
    }

    const now = new Date();
    if (waiver.expiresAt <= now) {
      throw new Error("Waiver expiry date must be in the future");
    }

    // Validate expiry is reasonable based on impact level
    const maxExpiryDays =
      { low: 30, medium: 90, high: 180, critical: 365 }[waiver.impactLevel] ||
      30;
    const maxExpiry = new Date(
      now.getTime() + maxExpiryDays * 24 * 60 * 60 * 1000
    );

    if (waiver.expiresAt > maxExpiry) {
      throw new Error(
        `Waiver expiry cannot exceed ${maxExpiryDays} days for ${waiver.impactLevel} impact level`
      );
    }
  }

  /**
   * Validate approval criteria
   */
  private async validateApprovalCriteria(
    waiver: WaiverRequest,
    approval: WaiverApprovalRequest
  ): Promise<void> {
    const _workflow = this.approvalWorkflow[waiver.impactLevel];

    if (!approval.approvalCriteria) {
      throw new Error("Approval criteria must be provided for waiver approval");
    }

    // Validate risk assessment
    const riskAssessment = approval.approvalCriteria.riskAssessment;
    if (riskAssessment !== waiver.impactLevel) {
      throw new Error(
        "Approval risk assessment must match waiver impact level"
      );
    }

    // Additional validation based on impact level
    if ((waiver.impactLevel as string) === "critical") {
      const criteria = approval.approvalCriteria;
      if (!criteria.stakeholderNotified) {
        throw new Error("Critical waivers require stakeholder notification");
      }
    }
  }

  /**
   * Initialize approval workflow
   */
  private async initializeApprovalWorkflow(
    waiver: WaiverRequest
  ): Promise<void> {
    const workflow = this.approvalWorkflow[waiver.impactLevel];

    // Store workflow metadata in memory
    if (this.memoryManager) {
      await this.memoryManager.storeExperience(waiver.tenantId, {
        memoryId: `waiver-workflow-${waiver.id}`,
        relevanceScore: 0.9,
        contextMatch: {
          similarityScore: 0.9,
          keywordMatches: ["waiver", "approval", "workflow"],
          semanticMatches: ["policy exception", "approval process"],
          temporalAlignment: 0.9,
        },
        content: {
          type: "waiver_workflow",
          waiverId: waiver.id,
          workflow,
          startedAt: waiver.requestedAt,
          status: "pending",
        },
      });
    }
  }

  /**
   * Log approval decision
   */
  private async logApprovalDecision(
    waiver: WaiverRequest,
    approval: WaiverApprovalRequest
  ): Promise<void> {
    const decision = {
      waiverId: waiver.id,
      decision: approval.approved ? "approved" : "rejected",
      approver: approval.approver,
      timestamp: new Date().toISOString(),
      notes: approval.notes,
      criteria: approval.approvalCriteria,
    };

    if (this.memoryManager) {
      await this.memoryManager.storeExperience(waiver.tenantId, {
        memoryId: `waiver-decision-${waiver.id}`,
        relevanceScore: 0.8,
        contextMatch: {
          similarityScore: 0.8,
          keywordMatches: ["waiver", "decision", "approval"],
          semanticMatches: ["policy decision", "exception handling"],
          temporalAlignment: 0.8,
        },
        content: {
          type: "waiver_decision",
          ...decision,
        },
      });
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
          semanticMatches: [
            "policy waiver",
            "policy exception",
            "emergency override",
          ],
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
    // TODO: Implement proper waiver update logic
    await this.storeWaiverInMemory(waiver);
  }

  /**
   * Generate unique waiver ID
   */
  private generateWaiverId(): string {
    const timestamp = Date.now().toString(36);
    const random = crypto.randomBytes(4).toString("hex");
    return `wv-${timestamp}-${random}`;
  }
}
