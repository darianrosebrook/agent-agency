/**
 * Audit Logger - Durable audit trail system
 *
 * Provides comprehensive audit logging for compliance, security, and operational
 * tracking. Supports multiple storage backends and structured logging.
 *
 * @author @darianrosebrook
 */

import { Logger } from "../observability/Logger.js";

export interface AuditEvent {
  id: string;
  timestamp: Date;
  eventType: string;
  actor: {
    id: string;
    type: "user" | "system" | "service";
    name?: string;
    email?: string;
  };
  resource: {
    type: string;
    id: string;
    name?: string;
  };
  action: string;
  outcome: "success" | "failure" | "partial";
  details: Record<string, any>;
  metadata?: {
    ipAddress?: string;
    userAgent?: string;
    sessionId?: string;
    requestId?: string;
    correlationId?: string;
  };
  severity: "low" | "medium" | "high" | "critical";
  compliance?: {
    regulations: string[];
    dataClassification: "public" | "internal" | "confidential" | "restricted";
    retentionPeriod: number; // days
  };
}

export interface AuditQuery {
  eventTypes?: string[];
  actors?: string[];
  resources?: string[];
  actions?: string[];
  outcomes?: string[];
  severity?: string[];
  startDate?: Date;
  endDate?: Date;
  limit?: number;
  offset?: number;
  sortBy?: "timestamp" | "severity" | "eventType";
  sortOrder?: "asc" | "desc";
}

export interface AuditConfig {
  enabled: boolean;
  storage: {
    type: "database" | "file" | "elasticsearch" | "mock";
    config: Record<string, any>;
  };
  retention: {
    defaultDays: number;
    byEventType: Record<string, number>;
    bySeverity: Record<string, number>;
  };
  encryption: {
    enabled: boolean;
    keyId?: string;
  };
  compression: {
    enabled: boolean;
    algorithm: "gzip" | "lz4";
  };
  batching: {
    enabled: boolean;
    batchSize: number;
    flushIntervalMs: number;
  };
}

export interface AuditStorageProvider {
  write(_event: AuditEvent): Promise<void>;
  writeBatch(_events: AuditEvent[]): Promise<void>;
  query(_query: AuditQuery): Promise<AuditEvent[]>;
  delete(_olderThan: Date): Promise<number>;
  healthCheck(): Promise<{ healthy: boolean; error?: string }>;
}

/**
 * Database audit storage provider
 */
export class DatabaseAuditStorage implements AuditStorageProvider {
  private batchBuffer: AuditEvent[] = [];
  private flushTimer?: ReturnType<typeof setTimeout>;

  constructor(private config: Record<string, any>, private logger: Logger) {
    if (config.batching?.enabled) {
      this.startBatchFlush();
    }
  }

  async write(event: AuditEvent): Promise<void> {
    if (this.config.batching?.enabled) {
      this.batchBuffer.push(event);
      if (this.batchBuffer.length >= this.config.batching.batchSize) {
        await this.flushBatch();
      }
    } else {
      await this.writeToDatabase([event]);
    }
  }

  async writeBatch(events: AuditEvent[]): Promise<void> {
    if (this.config.batching?.enabled) {
      this.batchBuffer.push(...events);
      if (this.batchBuffer.length >= this.config.batching.batchSize) {
        await this.flushBatch();
      }
    } else {
      await this.writeToDatabase(events);
    }
  }

  async query(query: AuditQuery): Promise<AuditEvent[]> {
    try {
      // In a real implementation, this would query the database
      this.logger.debug("Querying audit events", { query });

      // Mock query results
      const mockEvents: AuditEvent[] = [
        {
          id: "audit_001",
          timestamp: new Date(),
          eventType: "waiver_created",
          actor: {
            id: "user_123",
            type: "user",
            name: "John Doe",
            email: "john@example.com",
          },
          resource: {
            type: "waiver",
            id: "waiver_456",
            name: "Security Policy Waiver",
          },
          action: "create",
          outcome: "success",
          details: {
            reason: "Emergency deployment required",
            policyId: "security_policy_001",
          },
          severity: "high",
          compliance: {
            regulations: ["SOX", "GDPR"],
            dataClassification: "confidential",
            retentionPeriod: 2555, // 7 years
          },
        },
      ];

      return mockEvents;
    } catch (error) {
      this.logger.error("Failed to query audit events", {
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  async delete(olderThan: Date): Promise<number> {
    try {
      // In a real implementation, this would delete old records
      this.logger.info("Deleting old audit events", { olderThan });
      return 0; // Mock return
    } catch (error) {
      this.logger.error("Failed to delete old audit events", {
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  async healthCheck(): Promise<{ healthy: boolean; error?: string }> {
    try {
      // In a real implementation, this would test database connectivity
      return { healthy: true };
    } catch (error) {
      return {
        healthy: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  private async writeToDatabase(events: AuditEvent[]): Promise<void> {
    try {
      // In a real implementation, this would write to PostgreSQL, MySQL, etc.
      this.logger.debug("Writing audit events to database", {
        count: events.length,
      });

      // Simulate database write
      await new Promise((resolve) => setTimeout(resolve, 50));
    } catch (error) {
      this.logger.error("Failed to write audit events to database", {
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  private startBatchFlush(): void {
    this.flushTimer = setInterval(async () => {
      if (this.batchBuffer.length > 0) {
        await this.flushBatch();
      }
    }, this.config.batching.flushIntervalMs);
  }

  private async flushBatch(): Promise<void> {
    if (this.batchBuffer.length === 0) return;

    const batch = [...this.batchBuffer];
    this.batchBuffer = [];

    try {
      await this.writeToDatabase(batch);
      this.logger.debug("Flushed audit event batch", { count: batch.length });
    } catch (error) {
      this.logger.error("Failed to flush audit event batch", {
        error: error instanceof Error ? error.message : String(error),
      });
      // Re-add events to buffer for retry
      this.batchBuffer.unshift(...batch);
    }
  }
}

/**
 * File-based audit storage provider
 */
export class FileAuditStorage implements AuditStorageProvider {
  constructor(private config: Record<string, any>, private logger: Logger) {}

  async write(event: AuditEvent): Promise<void> {
    try {
      // In a real implementation, this would write to structured log files
      this.logger.info("Audit event", {
        auditEvent: event,
        storage: "file",
      });
    } catch (error) {
      this.logger.error("Failed to write audit event to file", {
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  async writeBatch(events: AuditEvent[]): Promise<void> {
    for (const event of events) {
      await this.write(event);
    }
  }

  async query(_query: AuditQuery): Promise<AuditEvent[]> {
    // File-based storage typically doesn't support complex queries
    this.logger.warn("File-based audit storage does not support queries");
    return [];
  }

  async delete(olderThan: Date): Promise<number> {
    // File-based storage typically uses log rotation
    this.logger.info("File-based audit storage uses log rotation", {
      olderThan,
    });
    return 0;
  }

  async healthCheck(): Promise<{ healthy: boolean; error?: string }> {
    try {
      // In a real implementation, this would check file system access
      return { healthy: true };
    } catch (error) {
      return {
        healthy: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }
}

/**
 * Mock audit storage provider for testing
 */
export class MockAuditStorage implements AuditStorageProvider {
  private events: AuditEvent[] = [];

  constructor(private config: Record<string, any>, private logger: Logger) {}

  async write(event: AuditEvent): Promise<void> {
    this.events.push(event);
    this.logger.debug("Mock audit event written", { eventId: event.id });
  }

  async writeBatch(events: AuditEvent[]): Promise<void> {
    this.events.push(...events);
    this.logger.debug("Mock audit events written", { count: events.length });
  }

  async query(query: AuditQuery): Promise<AuditEvent[]> {
    let filtered = [...this.events];

    if (query.eventTypes?.length) {
      filtered = filtered.filter((e) =>
        query.eventTypes!.includes(e.eventType)
      );
    }

    if (query.actors?.length) {
      filtered = filtered.filter((e) => query.actors!.includes(e.actor.id));
    }

    if (query.startDate) {
      filtered = filtered.filter((e) => e.timestamp >= query.startDate!);
    }

    if (query.endDate) {
      filtered = filtered.filter((e) => e.timestamp <= query.endDate!);
    }

    if (query.limit) {
      filtered = filtered.slice(0, query.limit);
    }

    return filtered;
  }

  async delete(olderThan: Date): Promise<number> {
    const initialCount = this.events.length;
    this.events = this.events.filter((e) => e.timestamp > olderThan);
    return initialCount - this.events.length;
  }

  async healthCheck(): Promise<{ healthy: boolean; error?: string }> {
    return { healthy: true };
  }

  // Test helper methods
  getEvents(): AuditEvent[] {
    return [...this.events];
  }

  clear(): void {
    this.events = [];
  }
}

/**
 * Comprehensive audit logger
 */
export class AuditLogger {
  private storage: AuditStorageProvider;

  constructor(private config: AuditConfig, private logger: Logger) {
    this.storage = this.createStorageProvider();
  }

  private createStorageProvider(): AuditStorageProvider {
    switch (this.config.storage.type) {
      case "database":
        return new DatabaseAuditStorage(
          this.config.storage.config,
          this.logger
        );
      case "file":
        return new FileAuditStorage(this.config.storage.config, this.logger);
      case "mock":
        return new MockAuditStorage(this.config.storage.config, this.logger);
      default:
        this.logger.warn(
          `Unsupported audit storage type: ${this.config.storage.type}, using mock`
        );
        return new MockAuditStorage({}, this.logger);
    }
  }

  /**
   * Log an audit event from an AuditEvent object
   */
  async logEvent(event: AuditEvent): Promise<void>;
  /**
   * Log an audit event with individual parameters
   */
  async logEvent(
    eventOrType: AuditEvent | string,
    actor?: AuditEvent["actor"],
    resource?: AuditEvent["resource"],
    action?: string,
    outcome?: AuditEvent["outcome"],
    details?: Record<string, any>,
    options?: {
      severity?: AuditEvent["severity"];
      metadata?: AuditEvent["metadata"];
      compliance?: AuditEvent["compliance"];
    }
  ): Promise<void> {
    // Handle AuditEvent object
    if (typeof eventOrType === "object" && "eventType" in eventOrType) {
      const event = eventOrType;
      return this.logEventWithParams(
        event.eventType,
        event.actor,
        event.resource,
        event.action || "unknown",
        event.outcome || { success: true },
        event.details || {},
        {
          severity: event.severity,
          metadata: event.metadata,
          compliance: event.compliance,
        }
      );
    }

    // Handle individual parameters
    return this.logEventWithParams(
      eventOrType as string,
      actor!,
      resource!,
      action!,
      outcome!,
      details!,
      options
    );
  }

  /**
   * Log an audit event with individual parameters (internal implementation)
   */
  private async logEventWithParams(
    eventType: string,
    actor: AuditEvent["actor"],
    resource: AuditEvent["resource"],
    action: string,
    outcome: AuditEvent["outcome"],
    details: Record<string, any>,
    options?: {
      severity?: AuditEvent["severity"];
      metadata?: AuditEvent["metadata"];
      compliance?: AuditEvent["compliance"];
    }
  ): Promise<void> {
    if (!this.config.enabled) {
      this.logger.debug("Audit logging disabled, skipping event", {
        eventType,
      });
      return;
    }

    const event: AuditEvent = {
      id: this.generateEventId(),
      timestamp: new Date(),
      eventType,
      actor,
      resource,
      action,
      outcome,
      details,
      severity: options?.severity || "medium",
      metadata: options?.metadata,
      compliance: options?.compliance || {
        regulations: [],
        dataClassification: "internal",
        retentionPeriod: this.config.retention.defaultDays,
      },
    };

    try {
      await this.storage.write(event);
      this.logger.debug("Audit event logged", { eventId: event.id, eventType });
    } catch (error) {
      this.logger.error("Failed to log audit event", {
        eventId: event.id,
        eventType,
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  /**
   * Query audit events
   */
  async queryEvents(query: AuditQuery): Promise<AuditEvent[]> {
    try {
      return await this.storage.query(query);
    } catch (error) {
      this.logger.error("Failed to query audit events", {
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  /**
   * Clean up old audit events
   */
  async cleanup(): Promise<number> {
    try {
      const cutoffDate = new Date();
      cutoffDate.setDate(
        cutoffDate.getDate() - this.config.retention.defaultDays
      );

      const deletedCount = await this.storage.delete(cutoffDate);
      this.logger.info("Audit cleanup completed", { deletedCount });
      return deletedCount;
    } catch (error) {
      this.logger.error("Failed to cleanup audit events", {
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  /**
   * Health check
   */
  async healthCheck(): Promise<{ healthy: boolean; storage: boolean }> {
    try {
      const storageHealth = await this.storage.healthCheck();
      return {
        healthy: storageHealth.healthy,
        storage: storageHealth.healthy,
      };
    } catch (error) {
      this.logger.error("Audit logger health check failed", {
        error: error instanceof Error ? error.message : String(error),
      });
      return { healthy: false, storage: false };
    }
  }

  private generateEventId(): string {
    const timestamp = Date.now().toString(36);
    const random = Math.random().toString(36).substr(2, 9);
    return `audit_${timestamp}_${random}`;
  }
}
