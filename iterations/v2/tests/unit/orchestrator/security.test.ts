/**
 * @fileoverview Tests for Security Manager (ARBITER-005)
 *
 * @author @darianrosebrook
 */

import {
  AuthCredentials,
  Permission,
  SecurityEventType,
  SecurityLevel,
  SecurityManager,
} from "../../../src/orchestrator/SecurityManager";

describe("SecurityManager", () => {
  let securityManager: SecurityManager;
  const testAgent = {
    id: "test-agent-1",
    name: "Test Agent",
    modelFamily: "gpt-4" as any,
    capabilities: {
      "code-editing": { supported: true, confidence: 0.9 },
      analysis: { supported: true, confidence: 0.8 },
    } as any,
    performanceHistory: [] as any,
    currentLoad: {
      activeTasks: 0,
      queueDepth: 0,
      memoryUsage: 0,
      cpuUsage: 0,
      lastUpdated: new Date(),
    } as any,
    registeredAt: new Date().toISOString(),
    lastActiveAt: new Date().toISOString(),
  };

  beforeEach(() => {
    securityManager = new SecurityManager({
      enabled: true,
      trustedAgents: ["trusted-agent"],
      adminAgents: ["admin-agent"],
    });
    securityManager.registerAgent(testAgent);
  });

  describe("Agent Registration", () => {
    it("should register agent successfully", () => {
      const newAgent = { ...testAgent, id: "new-agent" };
      securityManager.registerAgent(newAgent);

      const credentials: AuthCredentials = {
        agentId: "new-agent",
        token: "valid-token-12345",
      };

      const context = securityManager.authenticate(credentials);
      expect(context).toBeTruthy();
      expect(context!.agentId).toBe("new-agent");
    });
  });

  describe("Authentication", () => {
    it("should authenticate valid credentials", () => {
      const credentials: AuthCredentials = {
        agentId: "test-agent-1",
        token: "valid-token-12345",
      };

      const context = securityManager.authenticate(credentials);
      expect(context).toBeTruthy();
      expect(context!.agentId).toBe("test-agent-1");
      expect(context!.securityLevel).toBe(SecurityLevel.AGENT);
    });

    it("should reject invalid agent", () => {
      const credentials: AuthCredentials = {
        agentId: "unknown-agent",
        token: "token",
      };

      const context = securityManager.authenticate(credentials);
      expect(context).toBeNull();
    });

    it("should reject invalid token", () => {
      const credentials: AuthCredentials = {
        agentId: "test-agent-1",
        token: "short",
      };

      const context = securityManager.authenticate(credentials);
      expect(context).toBeNull();
    });
  });

  describe("Authorization", () => {
    let context;

    beforeEach(() => {
      const credentials: AuthCredentials = {
        agentId: "test-agent-1",
        token: "valid-token-12345",
      };
      context = securityManager.authenticate(credentials);
    });

    it("should authorize basic agent permissions", () => {
      expect(securityManager.authorize(context!, Permission.SUBMIT_TASK)).toBe(
        true
      );
      expect(
        securityManager.authorize(context!, Permission.QUERY_OWN_TASKS)
      ).toBe(true);
      expect(
        securityManager.authorize(context!, Permission.UPDATE_OWN_PROGRESS)
      ).toBe(true);
    });

    it("should reject admin permissions for basic agent", () => {
      expect(
        securityManager.authorize(context!, Permission.ADMIN_QUERY_ALL)
      ).toBe(false);
      expect(
        securityManager.authorize(context!, Permission.ADMIN_MANAGE_CONFIG)
      ).toBe(false);
    });
  });

  describe("Trusted Agent Permissions", () => {
    let context;

    beforeEach(() => {
      const trustedAgent = { ...testAgent, id: "trusted-agent" };
      securityManager.registerAgent(trustedAgent);

      const credentials: AuthCredentials = {
        agentId: "trusted-agent",
        token: "valid-token-12345",
      };
      context = securityManager.authenticate(credentials);
    });

    it("should have trusted agent security level", () => {
      expect(context!.securityLevel).toBe(SecurityLevel.TRUSTED_AGENT);
    });
  });

  describe("Admin Agent Permissions", () => {
    let context;

    beforeEach(() => {
      const adminAgent = { ...testAgent, id: "admin-agent" };
      securityManager.registerAgent(adminAgent);

      const credentials: AuthCredentials = {
        agentId: "admin-agent",
        token: "valid-token-12345",
      };
      context = securityManager.authenticate(credentials);
    });

    it("should have admin security level", () => {
      expect(context!.securityLevel).toBe(SecurityLevel.ADMIN);
    });

    it("should authorize all permissions", () => {
      expect(
        securityManager.authorize(context!, Permission.ADMIN_QUERY_ALL)
      ).toBe(true);
      expect(
        securityManager.authorize(context!, Permission.ADMIN_MANAGE_CONFIG)
      ).toBe(true);
      expect(
        securityManager.authorize(context!, Permission.ADMIN_SHUTDOWN)
      ).toBe(true);
    });
  });

  describe("Rate Limiting", () => {
    let context;

    beforeEach(() => {
      const credentials: AuthCredentials = {
        agentId: "test-agent-1",
        token: "valid-token-12345",
      };
      context = securityManager.authenticate(credentials);
    });

    it("should allow requests within limit", () => {
      // Rate limit is 10 requests per minute
      for (let i = 0; i < 10; i++) {
        expect(securityManager.checkRateLimit(context!, "submitTask")).toBe(
          true
        );
      }
    });

    it("should block requests over limit", () => {
      // Exceed the limit
      for (let i = 0; i < 15; i++) {
        securityManager.checkRateLimit(context!, "submitTask");
      }

      // Should be blocked now
      expect(securityManager.checkRateLimit(context!, "submitTask")).toBe(
        false
      );
    });
  });

  describe("Input Sanitization", () => {
    let context;

    beforeEach(() => {
      const credentials: AuthCredentials = {
        agentId: "test-agent-1",
        token: "valid-token-12345",
      };
      context = securityManager.authenticate(credentials);
    });

    it("should allow safe input", () => {
      const safeInput = { task: "safe task description" };
      expect(() =>
        securityManager.sanitizeInput(context!, "test", safeInput)
      ).not.toThrow();
    });

    it("should block suspicious input", () => {
      const suspiciousInput = { task: '<script>alert("xss")</script>' };
      expect(() =>
        securityManager.sanitizeInput(context!, "test", suspiciousInput)
      ).toThrow();
    });

    it("should block oversized input", () => {
      const largeInput = { data: "x".repeat(20000) }; // Over 10KB limit
      expect(() =>
        securityManager.sanitizeInput(context!, "test", largeInput)
      ).toThrow();
    });
  });

  describe("Security Events", () => {
    it("should log authentication events", () => {
      const credentials: AuthCredentials = {
        agentId: "test-agent-1",
        token: "valid-token-12345",
      };

      securityManager.authenticate(credentials);

      const events = securityManager.getSecurityEvents();
      expect(
        events.some((e) => e.type === SecurityEventType.AUTH_SUCCESS)
      ).toBe(true);
    });

    it("should log authorization failures", () => {
      const credentials: AuthCredentials = {
        agentId: "test-agent-1",
        token: "valid-token-12345",
      };
      const context = securityManager.authenticate(credentials);

      securityManager.authorize(context!, Permission.ADMIN_QUERY_ALL);

      const events = securityManager.getSecurityEvents();
      expect(
        events.some((e) => e.type === SecurityEventType.AUTHZ_FAILURE)
      ).toBe(true);
    });
  });

  describe("Resource Access Control", () => {
    let agent1Context;
    let agent2Context;

    beforeEach(() => {
      const agent2 = { ...testAgent, id: "agent-2" };
      securityManager.registerAgent(agent2);

      const cred1: AuthCredentials = {
        agentId: "test-agent-1",
        token: "valid-token-12345",
      };
      const cred2: AuthCredentials = {
        agentId: "agent-2",
        token: "valid-token-12345",
      };

      agent1Context = securityManager.authenticate(cred1);
      agent2Context = securityManager.authenticate(cred2);
    });

    it("should allow agent to access own resources", () => {
      expect(
        securityManager.canAccessResource(agent1Context!, "test-agent-1")
      ).toBe(true);
    });

    it("should deny agent access to other agent resources", () => {
      expect(securityManager.canAccessResource(agent1Context!, "agent-2")).toBe(
        false
      );
    });

    it("should allow admin access to all resources", () => {
      const adminAgent = { ...testAgent, id: "admin-agent" };
      securityManager.registerAgent(adminAgent);

      const adminCred: AuthCredentials = {
        agentId: "admin-agent",
        token: "valid-token-12345",
      };
      const adminContext = securityManager.authenticate(adminCred);

      expect(adminContext!.securityLevel).toBe(SecurityLevel.ADMIN);
      expect(
        securityManager.canAccessResource(adminContext!, "test-agent-1")
      ).toBe(true);
      expect(securityManager.canAccessResource(adminContext!, "agent-2")).toBe(
        true
      );
    });
  });
});
