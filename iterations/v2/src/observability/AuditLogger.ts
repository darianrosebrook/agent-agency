/**
 * Audit Logger Implementation
 *
 * Structured audit logging for security, compliance, and operational tracking.
 * Supports multiple output sinks, log retention, and compliance features.
 *
 * @author @darianrosebrook
 */

import {
  BaseEvent,
  EventEmitter,
  EventSeverity,
} from "../orchestrator/EventEmitter";
import { LogLevel, Logger } from "./Logger";

export enum AuditEventType {
  // Security Events
  AUTHENTICATION = "authentication",
  AUTHORIZATION = "authorization",
  ACCESS_CONTROL = "access_control",
  DATA_ACCESS = "data_access",
  CONFIGURATION_CHANGE = "configuration_change",

  // Operational Events
  SYSTEM_STARTUP = "system_startup",
  SYSTEM_SHUTDOWN = "system_shutdown",
  COMPONENT_FAILURE = "component_failure",
  COMPONENT_RECOVERY = "component_recovery",

  // Task Events
  TASK_SUBMISSION = "task_submission",
  TASK_EXECUTION = "task_execution",
  TASK_COMPLETION = "task_completion",
  TASK_FAILURE = "task_failure",
  TASK_CANCELLATION = "task_cancellation",

  // Agent Events
  AGENT_REGISTRATION = "agent_registration",
  AGENT_DEREGISTRATION = "agent_deregistration",
  AGENT_HEALTH_CHECK = "agent_health_check",

  // Verification Events
  VERIFICATION_REQUEST = "verification_request",
  VERIFICATION_SUCCESS = "verification_success",
  VERIFICATION_FAILURE = "verification_failure",

  // Compliance Events
  POLICY_VIOLATION = "policy_violation",
  COMPLIANCE_CHECK = "compliance_check",
  AUDIT_LOG_ACCESS = "audit_log_access",
}

export enum AuditSeverity {
  LOW = "low",
  MEDIUM = "medium",
  HIGH = "high",
  CRITICAL = "critical",
}

/**
 * Convert AuditSeverity to EventSeverity
 */
function auditSeverityToEventSeverity(
  auditSeverity: AuditSeverity
): EventSeverity {
  switch (auditSeverity) {
    case AuditSeverity.LOW:
      return EventSeverity.INFO;
    case AuditSeverity.MEDIUM:
      return EventSeverity.WARN;
    case AuditSeverity.HIGH:
      return EventSeverity.ERROR;
    case AuditSeverity.CRITICAL:
      return EventSeverity.CRITICAL;
    default:
      return EventSeverity.INFO;
  }
}

/**
 * Convert EventSeverity back to AuditSeverity
 */
function eventSeverityToAuditSeverity(
  eventSeverity: EventSeverity
): AuditSeverity {
  switch (eventSeverity) {
    case EventSeverity.DEBUG:
    case EventSeverity.INFO:
      return AuditSeverity.LOW;
    case EventSeverity.WARN:
      return AuditSeverity.MEDIUM;
    case EventSeverity.ERROR:
      return AuditSeverity.HIGH;
    case EventSeverity.CRITICAL:
      return AuditSeverity.CRITICAL;
    default:
      return AuditSeverity.LOW;
  }
}

export interface AuditEvent extends BaseEvent {
  eventType: AuditEventType;
  actor: string; // Who performed the action
  resource: string; // What was acted upon
  action: string; // What action was taken
  outcome: "success" | "failure" | "partial";
  details: Record<string, any>;
  metadata: {
    sessionId?: string;
    requestId?: string;
    ipAddress?: string;
    userAgent?: string;
    riskScore?: number;
    complianceFlags?: string[];
  };
  retention: {
    category: "security" | "operational" | "compliance";
    retentionDays: number;
  };
}

export interface AuditLogQuery {
  startDate?: Date;
  endDate?: Date;
  eventTypes?: AuditEventType[];
  severity?: AuditSeverity[];
  actor?: string;
  resource?: string;
  outcome?: "success" | "failure" | "partial";
  limit?: number;
  offset?: number;
}

export interface AuditLogSink {
  write(event: AuditEvent): Promise<void>;
  query(query: AuditLogQuery): Promise<AuditEvent[]>;
  getStats(): Promise<{
    totalEvents: number;
    eventsByType: Record<AuditEventType, number>;
    eventsBySeverity: Record<AuditSeverity, number>;
    retentionStats: Record<string, number>;
  }>;
  cleanup(olderThanDays: number): Promise<number>; // Returns number of records cleaned
}

/**
 * In-Memory Audit Log Sink
 * Suitable for development and testing
 */
export class MemoryAuditLogSink implements AuditLogSink {
  private events: AuditEvent[] = [];
  private maxEvents: number;

  constructor(maxEvents: number = 10000) {
    this.maxEvents = maxEvents;
  }

  async write(event: AuditEvent): Promise<void> {
    this.events.push(event);

    // Maintain size limit
    if (this.events.length > this.maxEvents) {
      this.events.shift();
    }
  }

  async query(query: AuditLogQuery): Promise<AuditEvent[]> {
    let filtered = [...this.events];

    if (query.startDate) {
      filtered = filtered.filter((e) => e.timestamp >= query.startDate!);
    }
    if (query.endDate) {
      filtered = filtered.filter((e) => e.timestamp <= query.endDate!);
    }
    if (query.eventTypes?.length) {
      filtered = filtered.filter((e) =>
        query.eventTypes!.includes(e.eventType)
      );
    }
    if (query.severity?.length) {
      filtered = filtered.filter((e) => {
        const auditSeverity = eventSeverityToAuditSeverity(e.severity);
        return query.severity!.includes(auditSeverity);
      });
    }
    if (query.actor) {
      filtered = filtered.filter((e) => e.actor === query.actor);
    }
    if (query.resource) {
      filtered = filtered.filter((e) => e.resource === query.resource);
    }
    if (query.outcome) {
      filtered = filtered.filter((e) => e.outcome === query.outcome);
    }

    // Sort by timestamp descending (newest first)
    filtered.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());

    const offset = query.offset || 0;
    const limit = query.limit || filtered.length;
    return filtered.slice(offset, offset + limit);
  }

  async getStats() {
    const eventsByType = this.events.reduce((acc, event) => {
      acc[event.eventType] = (acc[event.eventType] || 0) + 1;
      return acc;
    }, {} as Record<AuditEventType, number>);

    const eventsBySeverity = this.events.reduce((acc, event) => {
      const auditSeverity = eventSeverityToAuditSeverity(event.severity);
      acc[auditSeverity] = (acc[auditSeverity] || 0) + 1;
      return acc;
    }, {} as Record<AuditSeverity, number>);

    const retentionStats = this.events.reduce((acc, event) => {
      const category = event.retention.category;
      acc[category] = (acc[category] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    return {
      totalEvents: this.events.length,
      eventsByType,
      eventsBySeverity,
      retentionStats,
    };
  }

  async cleanup(olderThanDays: number): Promise<number> {
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - olderThanDays);

    const beforeCount = this.events.length;
    this.events = this.events.filter((event) => event.timestamp >= cutoffDate);
    return beforeCount - this.events.length;
  }
}

/**
 * Main Audit Logger Class
 */
export class AuditLogger extends Logger {
  private sinks: AuditLogSink[] = [];
  private eventEmitter: EventEmitter;
  private defaultRetention = {
    security: 2555, // ~7 years
    operational: 365, // 1 year
    compliance: 2555, // ~7 years
  };

  constructor(
    name: string,
    level: LogLevel = LogLevel.INFO,
    sinks: AuditLogSink[] = []
  ) {
    super(name, level);
    this.eventEmitter = new EventEmitter();
    this.sinks = sinks.length > 0 ? sinks : [new MemoryAuditLogSink()];
  }

  /**
   * Log an audit event
   */
  async logAuditEvent(
    eventType: AuditEventType,
    severity: AuditSeverity,
    actor: string,
    resource: string,
    action: string,
    outcome: "success" | "failure" | "partial",
    details: Record<string, any> = {},
    metadata: Partial<AuditEvent["metadata"]> = {}
  ): Promise<void> {
    const event: AuditEvent = {
      id: `audit-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      type: "audit.event",
      timestamp: new Date(),
      severity: auditSeverityToEventSeverity(severity),
      source: "audit-logger",
      eventType,
      actor,
      resource,
      action,
      outcome,
      details: this.sanitizeDetails(details),
      metadata: {
        sessionId: metadata.sessionId,
        requestId: metadata.requestId,
        ipAddress: metadata.ipAddress,
        userAgent: metadata.userAgent,
        riskScore: metadata.riskScore || 0,
        complianceFlags: metadata.complianceFlags || [],
      },
      retention: {
        category: this.getRetentionCategory(eventType),
        retentionDays:
          this.defaultRetention[this.getRetentionCategory(eventType)],
      },
    };

    // Write to all sinks
    await Promise.all(this.sinks.map((sink) => sink.write(event)));

    // Emit event for real-time processing
    this.eventEmitter.emit(event);

    // Also log to regular logger with appropriate level
    const logMessage = `[AUDIT] ${eventType}: ${action} on ${resource} by ${actor} (${outcome})`;
    const logData = {
      eventId: event.id,
      severity: event.severity,
      riskScore: event.metadata.riskScore,
    };

    switch (severity) {
      case AuditSeverity.CRITICAL:
        super.error(logMessage, logData);
        break;
      case AuditSeverity.HIGH:
        super.error(logMessage, logData);
        break;
      case AuditSeverity.MEDIUM:
        super.warn(logMessage, logData);
        break;
      case AuditSeverity.LOW:
        super.info(logMessage, logData);
        break;
    }
  }

  /**
   * Query audit events
   */
  async queryAuditEvents(query: AuditLogQuery): Promise<AuditEvent[]> {
    // Query all sinks and merge results
    const allResults = await Promise.all(
      this.sinks.map((sink) => sink.query(query))
    );

    // Merge and deduplicate by event ID
    const merged = allResults.flat();
    const unique = merged.filter(
      (event, index, self) => self.findIndex((e) => e.id === event.id) === index
    );

    // Sort by timestamp descending
    unique.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());

    return unique;
  }

  /**
   * Get audit statistics
   */
  async getAuditStats() {
    const allStats = await Promise.all(
      this.sinks.map((sink) => sink.getStats())
    );

    return allStats.reduce(
      (combined, stats) => ({
        totalEvents: combined.totalEvents + stats.totalEvents,
        eventsByType: this.mergeCounts(
          combined.eventsByType,
          stats.eventsByType
        ),
        eventsBySeverity: this.mergeCounts(
          combined.eventsBySeverity,
          stats.eventsBySeverity
        ),
        retentionStats: this.mergeCounts(
          combined.retentionStats,
          stats.retentionStats
        ),
      }),
      {
        totalEvents: 0,
        eventsByType: {} as Record<AuditEventType, number>,
        eventsBySeverity: {} as Record<AuditSeverity, number>,
        retentionStats: {} as Record<string, number>,
      }
    );
  }

  /**
   * Clean up old audit events
   */
  async cleanupAuditLogs(olderThanDays: number = 90): Promise<number> {
    const results = await Promise.all(
      this.sinks.map((sink) => sink.cleanup(olderThanDays))
    );
    return results.reduce((sum, count) => sum + count, 0);
  }

  /**
   * Add a new audit log sink
   */
  addSink(sink: AuditLogSink): void {
    this.sinks.push(sink);
  }

  /**
   * Remove a sink
   */
  removeSink(sink: AuditLogSink): void {
    this.sinks = this.sinks.filter((s) => s !== sink);
  }

  /**
   * Get event emitter for real-time audit event handling
   */
  getEventEmitter(): EventEmitter {
    return this.eventEmitter;
  }

  private sanitizeDetails(details: Record<string, any>): Record<string, any> {
    const sanitized = { ...details };

    // Remove sensitive fields
    const sensitiveFields = [
      "password",
      "token",
      "secret",
      "key",
      "api_key",
      "apikey",
      "session_token",
      "auth_token",
      "bearer_token",
    ];

    for (const field of sensitiveFields) {
      if (sanitized[field]) {
        sanitized[field] = "[REDACTED]";
      }
    }

    return sanitized;
  }

  private getRetentionCategory(
    eventType: AuditEventType
  ): "security" | "operational" | "compliance" {
    const securityEvents = [
      AuditEventType.AUTHENTICATION,
      AuditEventType.AUTHORIZATION,
      AuditEventType.ACCESS_CONTROL,
      AuditEventType.DATA_ACCESS,
      AuditEventType.POLICY_VIOLATION,
    ];

    const complianceEvents = [
      AuditEventType.CONFIGURATION_CHANGE,
      AuditEventType.COMPLIANCE_CHECK,
      AuditEventType.AUDIT_LOG_ACCESS,
    ];

    if (securityEvents.includes(eventType)) {
      return "security";
    }
    if (complianceEvents.includes(eventType)) {
      return "compliance";
    }
    return "operational";
  }

  private mergeCounts(
    a: Record<string, number>,
    b: Record<string, number>
  ): Record<string, number> {
    const result = { ...a };
    for (const [key, value] of Object.entries(b)) {
      result[key] = (result[key] || 0) + value;
    }
    return result;
  }
}
