/**
 * @fileoverview Tests for Agent Registry Security Layer
 *
 * Tests authentication, authorization, input validation, and multi-tenant isolation.
 *
 * @author @darianrosebrook
 */

import {
  AgentRegistrySecurity,
  SecurityContext,
  SecurityError,
} from "../../../src/security/AgentRegistrySecurity";
import { AgentProfile, PerformanceMetrics } from "../../../src/types/agent-registry";

describe("AgentRegistrySecurity", () => {
  let security: AgentRegistrySecurity;

  beforeEach(() => {
    security = new AgentRegistrySecurity({
      authenticationEnabled: true,
      authorizationEnabled: true,
      multiTenantEnabled: true,
      auditLoggingEnabled: true,
      rateLimitingEnabled: true,
      rateLimitPerMinute: 10,
    });
  });

  describe("Authentication", () => {
    it("should authenticate valid token", () => {
      const token = Buffer.from("tenant1:user1:admin,orchestrator").toString("base64");

      const context = security.authenticateRequest(token, "req-001");

      expect(context.tenantId).toBe("tenant1");
      expect(context.userId).toBe("user1");
      expect(context.roles).toEqual(["admin", "orchestrator"]);
    });

    it("should reject invalid token format", () => {
      const invalidToken = Buffer.from("invalid").toString("base64");

      expect(() => security.authenticateRequest(invalidToken, "req-001")).toThrow(
        SecurityError
      );
    });
  });

  describe("Authorization", () => {
    it("should allow registration with admin role", () => {
      const context: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["admin"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      expect(() => security.authorizeRegistration(context)).not.toThrow();
    });

    it("should allow registration with agent-manager role", () => {
      const context: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["agent-manager"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      expect(() => security.authorizeRegistration(context)).not.toThrow();
    });

    it("should reject registration without proper role", () => {
      const context: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["public"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      expect(() => security.authorizeRegistration(context)).toThrow(SecurityError);
    });

    it("should allow deletion only for admin role", () => {
      const adminContext: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["admin"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      expect(() => security.authorizeDeletion(adminContext)).not.toThrow();

      const managerContext: SecurityContext = {
        ...adminContext,
        roles: ["agent-manager"],
      };

      expect(() => security.authorizeDeletion(managerContext)).toThrow(SecurityError);
    });
  });

  describe("Rate Limiting", () => {
    it("should allow requests under rate limit", () => {
      const context: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["admin"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      for (let i = 0; i < 10; i++) {
        expect(() => security.checkRateLimit(context)).not.toThrow();
      }
    });

    it("should reject requests over rate limit", () => {
      const context: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["admin"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      // Exhaust rate limit
      for (let i = 0; i < 10; i++) {
        security.checkRateLimit(context);
      }

      // Next request should fail
      expect(() => security.checkRateLimit(context)).toThrow(SecurityError);
    });

    it("should reset rate limit after time window", (done) => {
      const context: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["admin"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      // Exhaust rate limit
      for (let i = 0; i < 10; i++) {
        security.checkRateLimit(context);
      }

      // Wait for reset (simulate 1 minute passing)
      setTimeout(() => {
        expect(() => security.checkRateLimit(context)).not.toThrow();
        done();
      }, 100); // Short timeout for test
    }, 200);
  });

  describe("Input Validation", () => {
    it("should reject agent with invalid ID", () => {
      const invalidAgent: Partial<AgentProfile> = {
        id: "",
        name: "Test",
        modelFamily: "claude-3.5",
      };

      expect(() => security.validateAgentProfile(invalidAgent)).toThrow(SecurityError);
    });

    it("should reject agent with ID too long", () => {
      const invalidAgent: Partial<AgentProfile> = {
        id: "a".repeat(101),
        name: "Test",
        modelFamily: "claude-3.5",
      };

      expect(() => security.validateAgentProfile(invalidAgent)).toThrow(SecurityError);
    });

    it("should reject agent with invalid name", () => {
      const invalidAgent: Partial<AgentProfile> = {
        id: "agent-001",
        name: "",
        modelFamily: "claude-3.5",
      };

      expect(() => security.validateAgentProfile(invalidAgent)).toThrow(SecurityError);
    });

    it("should reject metrics with invalid quality score", () => {
      const invalidMetrics: PerformanceMetrics = {
        success: true,
        qualityScore: 1.5,
        latencyMs: 1000,
        tokensUsed: 100,
        taskType: "code-editing",
      };

      expect(() => security.validatePerformanceMetrics(invalidMetrics)).toThrow(
        SecurityError
      );
    });

    it("should reject metrics with excessive latency", () => {
      const invalidMetrics: PerformanceMetrics = {
        success: true,
        qualityScore: 0.9,
        latencyMs: 400000, // > 5 minutes
        tokensUsed: 100,
        taskType: "code-editing",
      };

      expect(() => security.validatePerformanceMetrics(invalidMetrics)).toThrow(
        SecurityError
      );
    });
  });

  describe("Multi-Tenant Isolation", () => {
    it("should scope agent ID to tenant", () => {
      const context: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["admin"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      const scopedId = security.scopeToTenant("agent-001", context);

      expect(scopedId).toBe("tenant1:agent-001");
    });

    it("should reject cross-tenant access", () => {
      const context: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["admin"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      const otherTenantAgentId = "tenant2:agent-001" as any;

      expect(() => security.verifyTenantOwnership(otherTenantAgentId, context)).toThrow(
        SecurityError
      );
    });

    it("should allow same-tenant access", () => {
      const context: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["admin"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      const sameTenantAgentId = "tenant1:agent-001" as any;

      expect(() =>
        security.verifyTenantOwnership(sameTenantAgentId, context)
      ).not.toThrow();
    });
  });

  describe("Audit Logging", () => {
    it("should log audit entries", () => {
      const context: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["admin"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      const entry = {
        id: "audit-001",
        tenantId: context.tenantId,
        userId: context.userId,
        operation: "register_agent",
        resource: "agent",
        resourceId: "agent-001",
        timestamp: new Date(),
        success: true,
        metadata: {},
      };

      security.logAuditEntry(entry);

      const log = security.getAuditLog(context.tenantId);

      expect(log.length).toBeGreaterThan(0);
      expect(log[log.length - 1].operation).toBe("register_agent");
    });

    it("should filter audit log by tenant", () => {
      const context1: SecurityContext = {
        tenantId: "tenant1",
        userId: "user1",
        roles: ["admin"],
        requestedAt: new Date(),
        requestId: "req-001",
      };

      const context2: SecurityContext = {
        tenantId: "tenant2",
        userId: "user2",
        roles: ["admin"],
        requestedAt: new Date(),
        requestId: "req-002",
      };

      security.logAuditEntry({
        id: "audit-001",
        tenantId: context1.tenantId,
        userId: context1.userId,
        operation: "test1",
        resource: "agent",
        resourceId: "agent-001",
        timestamp: new Date(),
        success: true,
        metadata: {},
      });

      security.logAuditEntry({
        id: "audit-002",
        tenantId: context2.tenantId,
        userId: context2.userId,
        operation: "test2",
        resource: "agent",
        resourceId: "agent-002",
        timestamp: new Date(),
        success: true,
        metadata: {},
      });

      const tenant1Log = security.getAuditLog("tenant1");
      const tenant2Log = security.getAuditLog("tenant2");

      expect(tenant1Log.every((e) => e.tenantId === "tenant1")).toBe(true);
      expect(tenant2Log.every((e) => e.tenantId === "tenant2")).toBe(true);
    });
  });
});

