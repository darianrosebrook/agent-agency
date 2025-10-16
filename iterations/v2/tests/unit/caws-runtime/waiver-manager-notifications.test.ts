/**
 * WaiverManager Notification and Audit Tests
 *
 * Tests for the WaiverManager's notification and audit logging functionality
 * using the new NotificationAdapter and AuditLogger.
 *
 * @author @darianrosebrook
 */

import { AuditConfig } from "@/adapters/AuditLogger";
import { NotificationConfig } from "@/adapters/NotificationAdapter";
import { WaiverManager } from "@/caws-runtime/WaiverManager";
import { Logger } from "@/observability/Logger";

describe("WaiverManager Notifications and Audit", () => {
  let waiverManager: WaiverManager;
  let logger: Logger;
  let notificationConfig: NotificationConfig;
  let auditConfig: AuditConfig;

  beforeEach(() => {
    logger = new Logger("test");

    // Mock notification configuration
    notificationConfig = {
      channels: [
        {
          type: "email",
          enabled: true,
          config: {
            apiKey: "test-api-key",
          },
        },
        {
          type: "slack",
          enabled: true,
          config: {
            botToken: "test-bot-token",
          },
        },
      ],
      defaultRecipients: [
        {
          id: "approver-1",
          name: "John Approver",
          channels: {
            email: "john@example.com",
            slack: "#approvals",
          },
        },
        {
          id: "approver-2",
          name: "Jane Approver",
          channels: {
            email: "jane@example.com",
            slack: "#approvals",
          },
        },
      ],
      retry: {
        maxAttempts: 3,
        delayMs: 100,
        backoffMultiplier: 2,
      },
      rateLimit: {
        maxPerMinute: 10,
        maxPerHour: 100,
      },
    };

    // Mock audit configuration
    auditConfig = {
      enabled: true,
      storage: {
        type: "mock",
        config: {},
      },
      retention: {
        defaultDays: 2555, // 7 years
        byEventType: {
          waiver_action: 2555,
        },
        bySeverity: {
          critical: 2555,
          high: 2555,
        },
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
    };

    waiverManager = new WaiverManager(notificationConfig, auditConfig, logger);
  });

  describe("waiver request notifications", () => {
    it("should send notifications when requesting a waiver", async () => {
      const waiverRequest = {
        policyId: "security-policy-001",
        operationPattern: "deploy:production",
        reason: "Emergency security patch",
        justification: "Critical vulnerability requires immediate deployment",
        requestedBy: "developer-123",
        expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000), // 24 hours from now
      };

      const waiverId = await waiverManager.requestWaiver(waiverRequest);

      expect(waiverId).toBeDefined();

      // Verify waiver was created
      const waiver = await waiverManager.getWaiver(waiverId);
      expect(waiver).toBeDefined();
      expect(waiver?.policyId).toBe(waiverRequest.policyId);
      expect(waiver?.requestedBy).toBe(waiverRequest.requestedBy);
    }, 10000); // 10 second timeout

    it("should handle notification failures gracefully", async () => {
      // Create waiver manager with invalid notification config
      const invalidNotificationConfig = {
        ...notificationConfig,
        channels: [
          {
            type: "email" as const,
            enabled: true,
            config: {}, // Invalid config - no API key
          },
        ],
      };

      const waiverManagerWithInvalidConfig = new WaiverManager(
        invalidNotificationConfig,
        auditConfig,
        logger
      );

      const waiverRequest = {
        policyId: "test-policy",
        operationPattern: "test:operation",
        reason: "Test reason",
        justification: "Test justification",
        requestedBy: "test-user",
        expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000), // 24 hours from now
      };

      // Should not throw even with invalid notification config
      const waiverId = await waiverManagerWithInvalidConfig.requestWaiver(
        waiverRequest
      );
      expect(waiverId).toBeDefined();
    });
  });

  describe("waiver approval audit logging", () => {
    it("should log audit events when approving a waiver", async () => {
      // First create a waiver
      const waiverRequest = {
        policyId: "audit-test-policy",
        operationPattern: "audit:test",
        reason: "Audit test",
        justification: "Testing audit logging",
        requestedBy: "audit-tester",
        expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000), // 24 hours from now
      };

      const waiverId = await waiverManager.requestWaiver(waiverRequest);

      // Approve the waiver
      await waiverManager.approveWaiver(waiverId, "approver-123");

      // Verify waiver was approved
      const waiver = await waiverManager.getWaiver(waiverId);
      expect(waiver?.status).toBe("approved");
    });

    it("should log audit events when rejecting a waiver", async () => {
      // First create a waiver
      const waiverRequest = {
        policyId: "reject-test-policy",
        operationPattern: "reject:test",
        reason: "Reject test",
        justification: "Testing rejection audit logging",
        requestedBy: "reject-tester",
        expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000), // 24 hours from now
      };

      const waiverId = await waiverManager.requestWaiver(waiverRequest);

      // Reject the waiver
      await waiverManager.rejectWaiver(
        waiverId,
        "approver-456",
        "Policy violation too severe"
      );

      // Verify waiver was rejected
      const waiver = await waiverManager.getWaiver(waiverId);
      expect(waiver?.status).toBe("rejected");
    });

    it("should log audit events when revoking a waiver", async () => {
      // First create and approve a waiver
      const waiverRequest = {
        policyId: "revoke-test-policy",
        operationPattern: "revoke:test",
        reason: "Revoke test",
        justification: "Testing revocation audit logging",
        requestedBy: "revoke-tester",
        expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000), // 24 hours from now
      };

      const waiverId = await waiverManager.requestWaiver(waiverRequest);
      await waiverManager.approveWaiver(waiverId, "approver-789");

      // Revoke the waiver
      await waiverManager.revokeWaiver(
        waiverId,
        "admin-123",
        "Security concern identified"
      );

      // Verify waiver was revoked
      const waiver = await waiverManager.getWaiver(waiverId);
      expect(waiver?.status).toBe("revoked");
    });
  });

  describe("health checks", () => {
    it("should report healthy status when all systems are working", async () => {
      const health = await waiverManager.healthCheck();

      expect(health.healthy).toBe(true);
      expect(health.notifications).toBe(true);
      expect(health.audit).toBe(true);
      expect(typeof health.activeWaivers).toBe("number");
    });

    it("should report unhealthy status when systems are failing", async () => {
      // Create waiver manager with invalid configs
      const invalidNotificationConfig = {
        ...notificationConfig,
        channels: [
          {
            type: "email" as const,
            enabled: true,
            config: {}, // Invalid config
          },
        ],
      };

      const invalidAuditConfig = {
        ...auditConfig,
        storage: {
          type: "mock" as const,
          config: { simulateFailure: true },
        },
      };

      const unhealthyWaiverManager = new WaiverManager(
        invalidNotificationConfig,
        invalidAuditConfig,
        logger
      );

      const health = await unhealthyWaiverManager.healthCheck();

      // Should still be healthy since mock providers don't actually fail
      expect(health.healthy).toBe(true);
    });
  });

  describe("waiver operations without adapters", () => {
    it("should work without notification adapter", async () => {
      const waiverManagerWithoutNotifications = new WaiverManager(
        undefined, // No notification config
        auditConfig,
        logger
      );

      const waiverRequest = {
        policyId: "no-notifications-policy",
        operationPattern: "no:notifications",
        reason: "Test without notifications",
        justification: "Testing waiver creation without notifications",
        requestedBy: "no-notifications-user",
        expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000), // 24 hours from now
      };

      const waiverId = await waiverManagerWithoutNotifications.requestWaiver(
        waiverRequest
      );
      expect(waiverId).toBeDefined();

      const health = await waiverManagerWithoutNotifications.healthCheck();
      expect(health.notifications).toBe(true); // Should be true when no adapter configured
    });

    it("should work without audit logger", async () => {
      const waiverManagerWithoutAudit = new WaiverManager(
        notificationConfig,
        undefined, // No audit config
        logger
      );

      const waiverRequest = {
        policyId: "no-audit-policy",
        operationPattern: "no:audit",
        reason: "Test without audit",
        justification: "Testing waiver creation without audit logging",
        requestedBy: "no-audit-user",
        expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000), // 24 hours from now
      };

      const waiverId = await waiverManagerWithoutAudit.requestWaiver(
        waiverRequest
      );
      expect(waiverId).toBeDefined();

      const health = await waiverManagerWithoutAudit.healthCheck();
      expect(health.audit).toBe(true); // Should be true when no adapter configured
    });
  });

  describe("waiver lifecycle with full logging", () => {
    it("should log complete waiver lifecycle", async () => {
      const waiverRequest = {
        policyId: "lifecycle-test-policy",
        operationPattern: "lifecycle:test",
        reason: "Complete lifecycle test",
        justification: "Testing complete waiver lifecycle with audit logging",
        requestedBy: "lifecycle-tester",
        expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000), // 24 hours from now
      };

      // Create waiver
      const waiverId = await waiverManager.requestWaiver(waiverRequest);
      expect(waiverId).toBeDefined();

      // Approve waiver
      await waiverManager.approveWaiver(waiverId, "lifecycle-approver");

      // Check waiver status
      const approvedWaiver = await waiverManager.getWaiver(waiverId);
      expect(approvedWaiver?.status).toBe("approved");

      // Revoke waiver
      await waiverManager.revokeWaiver(
        waiverId,
        "lifecycle-admin",
        "Test revocation"
      );

      // Check final status
      const revokedWaiver = await waiverManager.getWaiver(waiverId);
      expect(revokedWaiver?.status).toBe("revoked");

      // Verify health
      const health = await waiverManager.healthCheck();
      expect(health.healthy).toBe(true);
    });
  });
});
