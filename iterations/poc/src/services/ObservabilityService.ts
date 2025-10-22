/**
 * Central Observability Service
 *
 * Extracted from CAWS provenance system to provide centralized
 * observability, audit trails, and change tracking across the system.
 *
 * @author @darianrosebrook
 */

import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";
import type {
  ProvenanceData,
  PerformanceMetrics,
  ProvenanceMetadata,
} from "../types/index.js";

export interface AuditEntry {
  id: string;
  timestamp: string;
  actor: string;
  action: string;
  resource: string;
  details: Record<string, any>;
  success: boolean;
  metadata?: ProvenanceMetadata;
}

export interface ChangeTrackingEntry {
  id: string;
  timestamp: string;
  component: string;
  changeType: "create" | "update" | "delete" | "deploy";
  description: string;
  artifacts: string[];
  verification: {
    testsPassed?: number;
    testsTotal?: number;
    coverage?: number;
    performance?: PerformanceMetrics;
  };
  approvals: Array<{
    approver: string;
    timestamp: string;
    type: "code-review" | "security-review" | "performance-review";
  }>;
  hash: string;
  metadata?: ProvenanceMetadata;
}

export class ObservabilityService {
  private auditLog: AuditEntry[] = [];
  private changeLog: ChangeTrackingEntry[] = [];
  private maxEntries: number;

  constructor(maxEntries: number = 10000) {
    this.maxEntries = maxEntries;
  }

  /**
   * Record an audit event
   */
  recordAudit(
    actor: string,
    action: string,
    resource: string,
    details: Record<string, any>,
    success: boolean = true,
    metadata?: ProvenanceMetadata
  ): string {
    const entry: AuditEntry = {
      id: this.generateId(),
      timestamp: new Date().toISOString(),
      actor,
      action,
      resource,
      details,
      success,
      metadata,
    };

    this.auditLog.push(entry);

    // Maintain size limit
    if (this.auditLog.length > this.maxEntries) {
      this.auditLog.shift();
    }

    return entry.id;
  }

  /**
   * Record a change event
   */
  recordChange(
    component: string,
    changeType: ChangeTrackingEntry["changeType"],
    description: string,
    artifacts: string[],
    verification: ChangeTrackingEntry["verification"],
    metadata?: ProvenanceMetadata
  ): string {
    const entry: ChangeTrackingEntry = {
      id: this.generateId(),
      timestamp: new Date().toISOString(),
      component,
      changeType,
      description,
      artifacts,
      verification,
      approvals: [],
      hash: this.calculateHash({
        component,
        changeType,
        description,
        artifacts,
      }),
      metadata,
    };

    this.changeLog.push(entry);

    // Maintain size limit
    if (this.changeLog.length > this.maxEntries) {
      this.changeLog.shift();
    }

    return entry.id;
  }

  /**
   * Add approval to a change
   */
  addApproval(
    changeId: string,
    approver: string,
    type: ChangeTrackingEntry["approvals"][0]["type"]
  ): boolean {
    const change = this.changeLog.find((c) => c.id === changeId);
    if (!change) {
      return false;
    }

    change.approvals.push({
      approver,
      timestamp: new Date().toISOString(),
      type,
    });

    return true;
  }

  /**
   * Generate provenance data for a change
   */
  generateProvenance(
    changeId: string,
    agent: string,
    model: string,
    results: Record<string, any>
  ): ProvenanceData {
    const change = this.changeLog.find((c) => c.id === changeId);
    if (!change) {
      throw new Error(`Change ${changeId} not found`);
    }

    const provenance: ProvenanceData = {
      agent,
      model,
      commit: change.hash,
      artifacts: change.artifacts,
      results,
      approvals: change.approvals.map((a) => `${a.approver} (${a.type})`),
      generatedAt: change.timestamp,
      metadata: change.metadata,
    };

    return provenance;
  }

  /**
   * Get audit trail for a resource
   */
  getAuditTrail(resource: string, limit: number = 50): AuditEntry[] {
    return this.auditLog
      .filter((entry) => entry.resource === resource)
      .slice(-limit);
  }

  /**
   * Get change history for a component
   */
  getChangeHistory(
    component: string,
    limit: number = 50
  ): ChangeTrackingEntry[] {
    return this.changeLog
      .filter((entry) => entry.component === component)
      .slice(-limit);
  }

  /**
   * Get recent changes across all components
   */
  getRecentChanges(limit: number = 50): ChangeTrackingEntry[] {
    return this.changeLog.slice(-limit);
  }

  /**
   * Get audit statistics
   */
  getAuditStats(): {
    totalEvents: number;
    successRate: number;
    eventsByAction: Record<string, number>;
    eventsByResource: Record<string, number>;
    recentActivity: AuditEntry[];
  } {
    const totalEvents = this.auditLog.length;
    const successfulEvents = this.auditLog.filter((e) => e.success).length;
    const successRate =
      totalEvents > 0 ? (successfulEvents / totalEvents) * 100 : 0;

    const eventsByAction: Record<string, number> = {};
    const eventsByResource: Record<string, number> = {};

    for (const entry of this.auditLog) {
      eventsByAction[entry.action] = (eventsByAction[entry.action] || 0) + 1;
      eventsByResource[entry.resource] =
        (eventsByResource[entry.resource] || 0) + 1;
    }

    return {
      totalEvents,
      successRate: Math.round(successRate * 100) / 100,
      eventsByAction,
      eventsByResource,
      recentActivity: this.auditLog.slice(-10),
    };
  }

  /**
   * Get change tracking statistics
   */
  getChangeStats(): {
    totalChanges: number;
    changesByType: Record<string, number>;
    changesByComponent: Record<string, number>;
    approvalRate: number;
    recentChanges: ChangeTrackingEntry[];
  } {
    const totalChanges = this.changeLog.length;

    const changesByType: Record<string, number> = {};
    const changesByComponent: Record<string, number> = {};
    let totalApprovals = 0;

    for (const change of this.changeLog) {
      changesByType[change.changeType] =
        (changesByType[change.changeType] || 0) + 1;
      changesByComponent[change.component] =
        (changesByComponent[change.component] || 0) + 1;
      totalApprovals += change.approvals.length;
    }

    const approvalRate = totalChanges > 0 ? totalApprovals / totalChanges : 0;

    return {
      totalChanges,
      changesByType,
      changesByComponent,
      approvalRate: Math.round(approvalRate * 100) / 100,
      recentChanges: this.changeLog.slice(-10),
    };
  }

  /**
   * Export audit log to file
   */
  exportAuditLog(filePath: string): void {
    const dir = path.dirname(filePath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }

    fs.writeFileSync(filePath, JSON.stringify(this.auditLog, null, 2));
  }

  /**
   * Export change log to file
   */
  exportChangeLog(filePath: string): void {
    const dir = path.dirname(filePath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }

    fs.writeFileSync(filePath, JSON.stringify(this.changeLog, null, 2));
  }

  /**
   * Import logs from files
   */
  importAuditLog(filePath: string): void {
    if (fs.existsSync(filePath)) {
      const content = fs.readFileSync(filePath, "utf8");
      const imported = JSON.parse(content) as AuditEntry[];
      this.auditLog.push(...imported);
    }
  }

  importChangeLog(filePath: string): void {
    if (fs.existsSync(filePath)) {
      const content = fs.readFileSync(filePath, "utf8");
      const imported = JSON.parse(content) as ChangeTrackingEntry[];
      this.changeLog.push(...imported);
    }
  }

  /**
   * Clear all logs (for testing)
   */
  clearLogs(): void {
    this.auditLog = [];
    this.changeLog = [];
  }

  /**
   * Generate unique ID
   */
  private generateId(): string {
    return crypto.randomUUID();
  }

  /**
   * Calculate hash of data
   */
  private calculateHash(data: any): string {
    const content = JSON.stringify(data, Object.keys(data).sort());
    return crypto.createHash("sha256").update(content).digest("hex");
  }
}
