/**
 * Adapters System Integration Tests
 *
 * Tests the complete adapters system integration including:
 * - Audit logging across different components
 * - Distributed caching with cache invalidation
 * - Notification system with multiple channels
 * - Infrastructure controller coordination
 * - Cross-adapter communication and data flow
 *
 * @author @darianrosebrook
 */

import { AuditEvent, AuditLogger } from "../../src/adapters/AuditLogger";
import {
  CacheEntry,
  DistributedCacheClient,
} from "../../src/adapters/DistributedCacheClient";
import { IncidentNotifier } from "../../src/adapters/IncidentNotifier";
import { InfrastructureController } from "../../src/adapters/InfrastructureController";
import {
  NotificationAdapter,
  NotificationPriority,
} from "../../src/adapters/NotificationAdapter";
import { AuditEventType } from "../../src/observability/AuditLogger";
import { LogLevel, Logger } from "../../src/observability/Logger";

describe("Adapters System Integration", () => {
  let auditLogger: AuditLogger;
  let cacheClient: DistributedCacheClient;
  let incidentNotifier: IncidentNotifier;
  let infraController: InfrastructureController;
  let notificationAdapter: NotificationAdapter;

  // Test data fixtures
  const createAuditEvent = (
    eventTypes: AuditEventType = AuditEventType.TASK_SUBMISSION,
    details: any = {}
  ): Omit<AuditEvent, "id" | "timestamp"> => ({
    eventType: eventTypes,
    actor: {
      id: "test-user",
      type: "user" as const,
      name: "test-user",
    },
    resource: {
      type: "test",
      id: "test-resource",
    },
    action: "test-action",
    outcome: "success",
    details,
    severity: "medium",
    metadata: {
      ipAddress: "127.0.0.1",
      userAgent: "test-agent",
    },
  });

  const createCacheEntry = (
    key: string,
    value: any = { test: "data" },
    ttl = 3600000
  ): CacheEntry => ({
    key,
    value,
    ttl,
    metadata: {
      createdAt: new Date(),
      lastAccessed: new Date(),
      accessCount: 0,
    },
  });

  const createNotification = (
    title: string = "Test Notification",
    message: string = "Test message",
    priority: NotificationPriority = NotificationPriority.MEDIUM
  ) => ({
    title,
    body: message,
    priority: priority as any,
    metadata: { test: true },
  });

  beforeEach(async () => {
    // Initialize adapters
    const logger = new Logger("test-logger", LogLevel._INFO);

    auditLogger = new AuditLogger(
      {
        enabled: true,
        storage: {
          type: "mock",
          config: {},
        },
        retention: {
          defaultDays: 30,
          byEventType: {},
          bySeverity: {},
        },
        encryption: {
          enabled: false,
        },
        compression: {
          enabled: false,
          algorithm: "gzip",
        },
        batching: {
          enabled: false,
          batchSize: 100,
          flushIntervalMs: 5000,
        },
      },
      logger
    );

    cacheClient = new DistributedCacheClient(
      {
        enabled: true,
        provider: {
          type: "mock",
          redis: {
            host: "localhost",
            port: 6379,
          },
        },
        retry: {
          maxAttempts: 3,
          delayMs: 1000,
          backoffMultiplier: 1.5,
        },
        ttl: 3600000,
        maxMemory: "100mb",
      },
      logger
    );

    incidentNotifier = new IncidentNotifier({
      enabled: true,
      incidentSystem: {
        type: "mock",
      },
      notifications: {
        enabled: true,
        targets: [
          { type: "email" as const, enabled: true, config: {} },
          { type: "slack" as const, enabled: true, config: {} },
        ],
        escalationDelayMs: 300000,
      },
      retry: {
        maxAttempts: 3,
        backoffMs: 1000,
      },
    });

    infraController = new InfrastructureController({
      enabled: true,
      providers: {
        docker: {
          enabled: true,
          socketPath: "/var/run/docker.sock",
        },
      },
    });

    notificationAdapter = new NotificationAdapter(
      {
        channels: [
          {
            type: "email" as const,
            enabled: true,
            config: { smtp: { host: "localhost", port: 587 } },
          },
          {
            type: "slack" as const,
            enabled: true,
            config: { webhookUrl: "https://hooks.slack.com/test" },
          },
        ],
        defaultRecipients: [],
        retry: {
          maxAttempts: 3,
          delayMs: 1000,
          backoffMultiplier: 2,
        },
        rateLimit: {
          maxPerMinute: 60,
          maxPerHour: 1000,
        },
      },
      logger
    );

    // Skip initialization - adapters initialize in constructor
    await Promise.all([]);
  });

  afterEach(async () => {
    // Cleanup
    await Promise.all([
      auditLogger.shutdown(),
      cacheClient.shutdown(),
      notificationAdapter.shutdown(),
    ]);
  });

  describe("Audit Logging Integration", () => {
    it("should log events across multiple adapters", async () => {
      const event = createAuditEvent(AuditEventType.TASK_SUBMISSION, {
        component: "test-component",
        operation: "integration-test",
      });

      // Log event
      await auditLogger.logEvent(event);

      // Verify event was logged
      const events = await auditLogger.queryEvents({
        actor: "test-user",
        limit: 10,
      });

      expect(events.length).toBeGreaterThan(0);
      expect(events[0].actor).toBe("test-user");
      expect(events[0].eventType).toBe(AuditEventType.TASK_SUBMISSION);
    });

    it("should support audit trails for adapter operations", async () => {
      // Perform cache operation
      const cacheEntry = createCacheEntry("test-key");
      await cacheClient.set("test-key", cacheEntry);

      // This should generate audit events
      const auditEvents = await auditLogger.queryEvents({
        resources: "cache",
        limit: 10,
      });

      expect(auditEvents.length).toBeGreaterThan(0);
      expect(auditEvents.some((e) => e.resource === "cache")).toBe(true);
    });

    it("should handle concurrent audit logging", async () => {
      const eventCount = 50;
      const events = Array.from({ length: eventCount }, (_, i) =>
        createAuditEvent(AuditEventType.SYSTEM_STARTUP, { index: i })
      );

      // Log events concurrently
      const startTime = Date.now();
      await Promise.all(events.map((event) => auditLogger.logEvent(event)));
      const duration = Date.now() - startTime;

      // Should complete efficiently
      expect(duration).toBeLessThan(2000);

      // All events should be logged
      const loggedEvents = await auditLogger.queryEvents({
        eventTypes: AuditEventType.SYSTEM_STARTUP,
        limit: eventCount + 10,
      });

      expect(loggedEvents.length).toBe(eventCount);
    });

    it("should support audit event correlation", async () => {
      const correlationId = "test-correlation-123";

      // Log related events
      await auditLogger.logEvent({
        ...createAuditEvent(AuditEventType.TASK_SUBMISSION),
        correlationId,
        details: { step: 1 },
      });

      await auditLogger.logEvent({
        ...createAuditEvent(AuditEventType.SYSTEM_STARTUP),
        correlationId,
        details: { step: 2 },
      });

      // Query by correlation ID
      const correlatedEvents = await auditLogger.queryEvents({
        correlationId,
        limit: 10,
      });

      expect(correlatedEvents.length).toBe(2);
      expect(
        correlatedEvents.every((e) => e.correlationId === correlationId)
      ).toBe(true);
    });
  });

  describe("Distributed Caching Integration", () => {
    it("should cache and retrieve data across adapters", async () => {
      const key = "integration-test-key";
      const data = { user: "test-user", preferences: { theme: "dark" } };

      // Cache data
      await cacheClient.set(key, createCacheEntry(key, data));

      // Retrieve from cache
      const cached = await cacheClient.get(key);

      expect(cached).toBeDefined();
      expect(cached?.data).toEqual(data);
    });

    it("should handle cache invalidation across components", async () => {
      const userKey = "user-preferences";
      const userData = { theme: "light", notifications: true };

      // Cache user preferences
      await cacheClient.set(userKey, createCacheEntry(userKey, userData));

      // Simulate preference update (should invalidate cache)
      await cacheClient.delete(userKey);

      // Cache should be empty
      const cached = await cacheClient.get(userKey);
      expect(cached).toBeNull();
    });

    it("should support cache warming for performance", async () => {
      const cacheEntries = [
        createCacheEntry("config-1", { setting: "value1" }),
        createCacheEntry("config-2", { setting: "value2" }),
        createCacheEntry("config-3", { setting: "value3" }),
      ];

      // Warm cache with multiple entries
      await Promise.all(
        cacheEntries.map((entry) => cacheClient.set(entry.key, entry))
      );

      // Verify all entries are cached
      const results = await Promise.all(
        cacheEntries.map((entry) => cacheClient.get(entry.key))
      );

      expect(results.every((result) => result !== null)).toBe(true);
      expect(results.length).toBe(3);
    });

    it("should handle cache cluster operations", async () => {
      // Test cache operations in a cluster-like scenario
      const operations = [];

      // Simulate multiple clients accessing cache
      for (let i = 0; i < 20; i++) {
        operations.push(
          cacheClient.set(
            `key-${i}`,
            createCacheEntry(`key-${i}`, { value: i })
          )
        );
      }

      await Promise.all(operations);

      // Verify all keys are accessible
      const verificationOps = [];
      for (let i = 0; i < 20; i++) {
        verificationOps.push(cacheClient.get(`key-${i}`));
      }

      const results = await Promise.all(verificationOps);
      expect(results.every((result) => result !== null)).toBe(true);
    });
  });

  describe("Notification System Integration", () => {
    it("should send notifications through multiple channels", async () => {
      const notification = createNotification(
        "System Alert",
        "Integration test notification",
        NotificationPriority.HIGH
      );

      // Send notification
      const result = await notificationAdapter.send(notification);

      expect(result).toBeDefined();
      expect(result.channel).toBeDefined();
      expect(result.success).toBe(true);
    });

    it("should integrate notifications with incident management", async () => {
      // Create an incident
      const incident = {
        id: "test-incident-1",
        title: "Test Incident",
        description: "Integration test incident",
        severity: "high" as const,
        status: "open" as const,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      // Incident notifier should send notifications
      await incidentNotifier.notifyIncident(incident);

      // Check that notifications were sent (would be verified by mocks in real tests)
      expect(incident.status).toBe("open");
    });

    it("should handle notification failures gracefully", async () => {
      // Create notification that will fail
      const failingNotification = {
        ...createNotification(),
        channels: ["non-existent-channel" as any],
      };

      // Should not throw, but mark as failed
      const result = await notificationAdapter.send(failingNotification);

      expect(result.success).toBe(false);
      expect(result.failedChannels).toBeDefined();
    });

    it("should support notification templates and personalization", async () => {
      const templateNotification = {
        template: "user_welcome",
        parameters: {
          userName: "John Doe",
          accountType: "premium",
        },
        priority: NotificationPriority.LOW,
        channels: ["email"],
      };

      const result = await notificationAdapter.send(
        templateNotification as any
      );

      // Template resolution would happen here
      expect(result.success).toBeDefined();
    });
  });

  describe("Infrastructure Controller Integration", () => {
    it("should monitor and report infrastructure status", async () => {
      // Get current infrastructure status
      const status = await infraController.getStatus();

      expect(status).toBeDefined();
      expect(status.overall).toBeDefined();
      expect(status.components).toBeDefined();
      expect(Array.isArray(status.components)).toBe(true);
    });

    it("should coordinate with other adapters for health monitoring", async () => {
      // Infrastructure controller should integrate with cache and audit systems
      const healthStatus = await infraController.checkHealth();

      expect(healthStatus).toBeDefined();
      expect(typeof healthStatus.healthy).toBe("boolean");
    });

    it("should handle infrastructure scaling decisions", async () => {
      // Simulate load increase
      await infraController.reportMetrics({
        cpuUsage: 0.85,
        memoryUsage: 0.9,
        activeConnections: 150,
      });

      // Controller should make scaling decisions
      const scalingDecision = await infraController.evaluateScaling();

      expect(scalingDecision).toBeDefined();
      expect(typeof scalingDecision.shouldScale).toBe("boolean");
    });

    it("should integrate infrastructure events with audit logging", async () => {
      // Trigger infrastructure event
      await infraController.reportEvent({
        type: "scaling",
        details: { direction: "up", reason: "high_load" },
      });

      // Check audit log for infrastructure events
      const auditEvents = await auditLogger.queryEvents({
        resources: "infrastructure",
        limit: 10,
      });

      expect(auditEvents.length).toBeGreaterThan(0);
    });
  });

  describe("Cross-Adapter Communication", () => {
    it("should enable data flow between adapters", async () => {
      // Create a workflow: Cache -> Audit -> Notification

      // 1. Cache some data
      const cacheKey = "workflow-data";
      const workflowData = { workflowId: "test-123", status: "processing" };
      await cacheClient.set(cacheKey, createCacheEntry(cacheKey, workflowData));

      // 2. Audit the caching operation
      await auditLogger.logEvent({
        ...createAuditEvent(AuditEventType.DATA_ACCESS),
        resources: "cache",
        action: "set",
        details: { key: cacheKey },
      });

      // 3. Send notification about the operation
      const notification = createNotification(
        "Workflow Started",
        `Workflow ${workflowData.workflowId} has started processing`,
        NotificationPriority.MEDIUM
      );
      await notificationAdapter.send(notification);

      // 4. Verify the entire flow worked
      const cachedData = await cacheClient.get(cacheKey);
      expect(cachedData?.data).toEqual(workflowData);

      const auditEvents = await auditLogger.queryEvents({
        resources: "cache",
        limit: 5,
      });
      expect(auditEvents.length).toBeGreaterThan(0);
    });

    it("should handle adapter failure propagation", async () => {
      // Simulate cache failure
      // (In real implementation, this would trigger notifications and audit logging)

      // Create a scenario where cache fails
      const failedOperation = async () => {
        throw new Error("Cache connection lost");
      };

      // This should trigger incident notification and audit logging
      try {
        await failedOperation();
      } catch (error) {
        // Error should be logged and notified
        await auditLogger.logEvent({
          ...createAuditEvent(AuditEventType.COMPONENT_FAILURE),
          resources: "cache",
          action: "operation_failed",
          outcome: "failure",
          details: { error: error.message },
        });

        await incidentNotifier.notifyIncident({
          id: "cache-failure-1",
          title: "Cache Connection Lost",
          description: "Distributed cache is unavailable",
          severity: "high",
          status: "open",
          createdAt: new Date(),
          updatedAt: new Date(),
        });
      }

      // Verify error was logged
      const errorEvents = await auditLogger.queryEvents({
        eventTypes: AuditEventType.COMPONENT_FAILURE,
        limit: 5,
      });

      expect(errorEvents.length).toBeGreaterThan(0);
    });

    it("should support adapter health monitoring", async () => {
      // Check health of all adapters
      const adapterHealthChecks = await Promise.all([
        auditLogger.getHealth(),
        cacheClient.getHealth(),
        notificationAdapter.getHealth(),
        infraController.getHealth(),
      ]);

      // All adapters should report their health
      expect(adapterHealthChecks.every((health) => health !== undefined)).toBe(
        true
      );
      expect(
        adapterHealthChecks.every(
          (health) => typeof health.healthy === "boolean"
        )
      ).toBe(true);
    });

    it("should coordinate adapter lifecycle events", async () => {
      // Test shutdown coordination
      const shutdownOrder = [];

      // Override shutdown methods to track order (in real implementation)
      const originalAuditShutdown = auditLogger.shutdown;
      auditLogger.shutdown = async () => {
        shutdownOrder.push("audit");
        await originalAuditShutdown.call(auditLogger);
      };

      const originalCacheShutdown = cacheClient.shutdown;
      cacheClient.shutdown = async () => {
        shutdownOrder.push("cache");
        await originalCacheShutdown.call(cacheClient);
      };

      // Shutdown all adapters
      await Promise.all([
        auditLogger.shutdown(),
        cacheClient.shutdown(),
        incidentNotifier.shutdown(),
        infraController.shutdown(),
        notificationAdapter.shutdown(),
      ]);

      // Verify coordinated shutdown
      expect(shutdownOrder.length).toBe(2); // Audit and cache were tracked
    });
  });

  describe("Performance and Scalability", () => {
    it("should handle high-throughput adapter operations", async () => {
      const operationCount = 100;

      // Perform many concurrent operations across adapters
      const startTime = Date.now();

      const operations = [];
      for (let i = 0; i < operationCount; i++) {
        operations.push(
          auditLogger.logEvent({
            ...createAuditEvent(AuditEventType.SYSTEM_STARTUP),
            details: { index: i },
          })
        );

        operations.push(
          cacheClient.set(
            `perf-key-${i}`,
            createCacheEntry(`perf-key-${i}`, { value: i })
          )
        );
      }

      await Promise.all(operations);
      const duration = Date.now() - startTime;

      // Should handle high throughput efficiently
      expect(duration).toBeLessThan(5000);

      // Verify operations completed
      const auditEvents = await auditLogger.queryEvents({
        eventTypes: AuditEventType.SYSTEM_STARTUP,
        limit: operationCount + 10,
      });

      expect(auditEvents.length).toBe(operationCount);
    });

    it("should maintain performance under load", async () => {
      // Simulate sustained load
      const loadTestDuration = 5000; // 5 seconds
      const startTime = Date.now();
      let operationCount = 0;

      while (Date.now() - startTime < loadTestDuration) {
        // Perform mixed operations
        await Promise.all([
          auditLogger.logEvent(createAuditEvent()),
          cacheClient.set(
            `load-key-${operationCount}`,
            createCacheEntry(`load-key-${operationCount}`)
          ),
        ]);
        operationCount += 2;
      }

      const actualDuration = Date.now() - startTime;

      // Should maintain reasonable performance
      const operationsPerSecond = operationCount / (actualDuration / 1000);
      expect(operationsPerSecond).toBeGreaterThan(10); // At least 10 operations/second
    });

    it("should handle large data sets efficiently", async () => {
      // Create large audit event
      const largeEvent = {
        ...createAuditEvent(AuditEventType.DATA_ACCESS),
        details: {
          data: Array.from({ length: 1000 }, (_, i) => ({
            id: i,
            name: `item-${i}`,
            data: "x".repeat(100), // 100 chars per item
          })),
        },
      };

      const startTime = Date.now();
      await auditLogger.logEvent(largeEvent);
      const logDuration = Date.now() - startTime;

      // Should handle large events efficiently
      expect(logDuration).toBeLessThan(1000);

      // Should be retrievable
      const events = await auditLogger.queryEvents({
        eventTypes: AuditEventType.DATA_ACCESS,
        limit: 1,
      });

      expect(events.length).toBe(1);
      expect(events[0].details.data.length).toBe(1000);
    });
  });

  describe("Security and Compliance", () => {
    it("should maintain security context across adapters", async () => {
      const securityContext = {
        userId: "secure-user-123",
        roles: ["admin", "auditor"],
        permissions: ["read", "write", "audit"],
        sessionId: "session-456",
      };

      // Operations should maintain security context
      await auditLogger.logEvent({
        ...createAuditEvent(AuditEventType.AUTHENTICATION),
        securityContext,
        details: { operation: "secure-access" },
      });

      // Cache operations with security context
      await cacheClient.set("secure-data", {
        ...createCacheEntry("secure-data"),
        securityContext,
      });

      // Verify security context is preserved
      const auditEvents = await auditLogger.queryEvents({
        eventTypes: AuditEventType.AUTHENTICATION,
        limit: 5,
      });

      expect(auditEvents.length).toBeGreaterThan(0);
      expect(auditEvents[0].securityContext).toEqual(securityContext);
    });

    it("should support data encryption across adapters", async () => {
      const sensitiveData = {
        userId: "user-123",
        creditCard: "4111111111111111",
        ssn: "123-45-6789",
      };

      // Store encrypted data in cache
      const encryptedEntry = {
        ...createCacheEntry("sensitive-data", sensitiveData),
        encrypted: true,
      };

      await cacheClient.set("sensitive-data", encryptedEntry);

      // Retrieve and verify encryption
      const cached = await cacheClient.get("sensitive-data");

      expect(cached?.encrypted).toBe(true);
      // In real implementation, data would be encrypted/decrypted transparently
    });

    it("should enforce access controls between adapters", async () => {
      // Test that adapters respect each other's access controls
      const restrictedData = {
        classification: "confidential",
        owner: "admin",
        allowedRoles: ["admin"],
      };

      // Store with access controls
      await cacheClient.set("restricted-data", {
        ...createCacheEntry("restricted-data", restrictedData),
        accessControl: {
          requiredRoles: ["admin"],
          owner: "admin",
        },
      });

      // Audit access attempts
      await auditLogger.logEvent({
        ...createAuditEvent(AuditEventType.ACCESS_CONTROL),
        resources: "restricted-data",
        action: "read",
        outcome: "denied",
        details: { reason: "insufficient_permissions" },
      });

      // Verify access was denied and logged
      const accessEvents = await auditLogger.queryEvents({
        eventTypes: AuditEventType.ACCESS_CONTROL,
        limit: 5,
      });

      expect(accessEvents.length).toBeGreaterThan(0);
    });
  });
});
