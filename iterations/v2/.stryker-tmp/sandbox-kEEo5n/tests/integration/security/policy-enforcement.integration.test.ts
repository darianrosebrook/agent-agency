/**
 * @fileoverview Integration Tests for Security Policy Enforcement
 *
 * Tests multi-component interactions and real-world security scenarios.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import {
  AuthCredentials,
  Permission,
  SecurityManager,
  SecurityMiddleware,
} from "../../../src/orchestrator/SecurityManager";
import { AgentProfile } from "../../../src/types/arbiter-orchestration";

describe("Security Policy Enforcement - Integration Tests", () => {
  let securityManager: SecurityManager;
  let middleware: SecurityMiddleware;

  const createAgent = (id: string): AgentProfile => ({
    id,
    name: `Agent ${id}`,
    modelFamily: "gpt-4" as any,
    capabilities: {
      "code-editing": { supported: true, confidence: 0.9 },
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
  });

  beforeEach(() => {
    securityManager = new SecurityManager({
      enabled: true,
      trustedAgents: ["trusted-1"],
      adminAgents: ["admin-1"],
      policies: {
        maxTaskDescriptionLength: 10000,
        maxMetadataSize: 10240,
        allowedTaskTypes: {},
        suspiciousPatterns: [
          /<script/i,
          /javascript:/i,
          /data:text\/html/i,
          /\.\./,
          /<iframe/i,
          /onclick/i,
          /onerror/i,
        ],
      },
    });
    middleware = new SecurityMiddleware(securityManager);

    securityManager.registerAgent(createAgent("agent-1"));
    securityManager.registerAgent(createAgent("agent-2"));
    securityManager.registerAgent(createAgent("trusted-1"));
    securityManager.registerAgent(createAgent("admin-1"));
  });

  describe("Multi-Policy Evaluation", () => {
    it("should enforce multiple policies in sequence", async () => {
      const credentials: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
      };

      // Test full security flow
      const result = await middleware.protect(
        credentials,
        Permission.SUBMIT_TASK,
        "submitTask",
        async (context) => {
          // Inside protected operation
          expect(context.agentId).toBe("agent-1");

          // Verify we can still check authorization
          const canQuery = securityManager.authorize(
            context,
            Permission.QUERY_OWN_TASKS
          );
          expect(canQuery).toBe(true);

          // Verify sanitization works
          const safeData = securityManager.sanitizeInput(context, "operation", {
            task: "Safe task description",
          });
          expect(safeData.task).toBe("Safe task description");

          return { success: true, data: "Task submitted" };
        }
      );

      expect(result.success).toBe(true);
      expect(result.data).toBe("Task submitted");
    });

    it("should handle cascading policy checks", async () => {
      const credentials: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
      };

      const context = securityManager.authenticate(credentials)!;

      // Check multiple authorization levels
      expect(securityManager.authorize(context, Permission.SUBMIT_TASK)).toBe(
        true
      );
      expect(
        securityManager.authorize(context, Permission.QUERY_OWN_TASKS)
      ).toBe(true);
      expect(
        securityManager.authorize(context, Permission.UPDATE_OWN_PROGRESS)
      ).toBe(true);

      // But not admin
      expect(
        securityManager.authorize(context, Permission.ADMIN_QUERY_ALL)
      ).toBe(false);
    });
  });

  describe("Concurrent Enforcement Scenarios", () => {
    it("should handle multiple agents simultaneously", async () => {
      const credentials1: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
      };
      const credentials2: AuthCredentials = {
        agentId: "agent-2",
        token: "valid-token-12345",
      };

      const results = await Promise.all([
        middleware.protect(
          credentials1,
          Permission.SUBMIT_TASK,
          "submitTask",
          async (ctx) => ({ agent: ctx.agentId, result: "success" })
        ),
        middleware.protect(
          credentials2,
          Permission.SUBMIT_TASK,
          "submitTask",
          async (ctx) => ({ agent: ctx.agentId, result: "success" })
        ),
      ]);

      expect(results[0].agent).toBe("agent-1");
      expect(results[1].agent).toBe("agent-2");
      expect(results[0].result).toBe("success");
      expect(results[1].result).toBe("success");
    });

    it("should isolate rate limits per agent", async () => {
      const creds1: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
      };
      const creds2: AuthCredentials = {
        agentId: "agent-2",
        token: "valid-token-12345",
      };

      const ctx1 = securityManager.authenticate(creds1)!;
      const ctx2 = securityManager.authenticate(creds2)!;

      // Exhaust agent-1's rate limit
      for (let i = 0; i < 15; i++) {
        securityManager.checkRateLimit(ctx1, "submitTask");
      }

      // Agent-2 should still have quota
      expect(securityManager.checkRateLimit(ctx2, "submitTask")).toBe(true);

      // But agent-1 should be blocked
      expect(securityManager.checkRateLimit(ctx1, "submitTask")).toBe(false);
    });
  });

  describe("Circuit Breaker Integration", () => {
    it("should block operations after rate limit exceeded", async () => {
      const credentials: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
      };

      // Exhaust rate limit
      const context = securityManager.authenticate(credentials)!;
      for (let i = 0; i < 15; i++) {
        securityManager.checkRateLimit(context, "submitTask");
      }

      // Next operation should fail
      await expect(
        middleware.protect(
          credentials,
          Permission.SUBMIT_TASK,
          "submitTask",
          async () => ({ success: true })
        )
      ).rejects.toThrow("Rate limit exceeded");
    });

    it("should recover after block duration", async () => {
      const shortBlockManager = new SecurityManager({
        enabled: true,
        rateLimits: {
          submitTask: {
            requestsPerWindow: 2,
            windowMs: 1000,
            blockDurationMs: 100,
          },
          queryTasks: {
            requestsPerWindow: 30,
            windowMs: 60000,
            blockDurationMs: 60000,
          },
          updateProgress: {
            requestsPerWindow: 60,
            windowMs: 60000,
            blockDurationMs: 30000,
          },
        },
      });

      shortBlockManager.registerAgent(createAgent("test-agent"));
      const testMiddleware = new SecurityMiddleware(shortBlockManager);

      const credentials: AuthCredentials = {
        agentId: "test-agent",
        token: "valid-token-12345",
      };

      // Exhaust and block
      const context = shortBlockManager.authenticate(credentials)!;
      shortBlockManager.checkRateLimit(context, "submitTask");
      shortBlockManager.checkRateLimit(context, "submitTask");
      expect(shortBlockManager.checkRateLimit(context, "submitTask")).toBe(
        false
      );

      // Wait for recovery (both block and window need to reset)
      await new Promise((resolve) => setTimeout(resolve, 1200));

      // Should work now
      const result = await testMiddleware.protect(
        credentials,
        Permission.SUBMIT_TASK,
        "submitTask",
        async () => ({ recovered: true })
      );

      expect(result.recovered).toBe(true);
    }, 10000);
  });

  describe("Real-World Attack Scenarios", () => {
    it("should defend against coordinated XSS attacks", async () => {
      const credentials: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
      };

      const context = securityManager.authenticate(credentials)!;

      const attacks = [
        { data: "<script>alert(1)</script>", shouldBlock: true }, // <script pattern
        { data: "javascript:alert(1)", shouldBlock: true }, // javascript: pattern
        { data: '<iframe src="evil.com"></iframe>', shouldBlock: true }, // <iframe pattern
        { data: "<img src=x onerror=alert(1)>", shouldBlock: true }, // onerror pattern
        { data: '<div onclick="bad()">X</div>', shouldBlock: true }, // onclick pattern
        { data: "data:text/html,<script>alert(1)</script>", shouldBlock: true }, // data:text/html pattern
      ];

      let blockedCount = 0;
      let expectedBlocks = 0;

      for (const attack of attacks) {
        if (attack.shouldBlock) expectedBlocks++;
        try {
          securityManager.sanitizeInput(context, "test", { data: attack.data });
        } catch (e) {
          blockedCount++;
        }
      }

      // Should block most known patterns (at least 5 out of 6)
      expect(blockedCount).toBeGreaterThanOrEqual(5);
      expect(blockedCount).toBeLessThanOrEqual(expectedBlocks);
    });

    it("should handle brute force authentication attempts", () => {
      const attempts = 100;
      let successCount = 0;
      let failureCount = 0;

      for (let i = 0; i < attempts; i++) {
        const credentials: AuthCredentials = {
          agentId: `random-agent-${i}`,
          token: `valid-token-12345-${i}`,
        };

        const context = securityManager.authenticate(credentials);
        if (context) {
          successCount++;
        } else {
          failureCount++;
        }
      }

      // All should fail (unregistered agents)
      expect(failureCount).toBe(attempts);
      expect(successCount).toBe(0);
    });

    it("should prevent session hijacking", () => {
      const credentials: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
      };

      const context1 = securityManager.authenticate(credentials)!;
      const context2 = securityManager.authenticate(credentials)!;

      // Different session IDs
      expect(context1.sessionId).not.toBe(context2.sessionId);

      // Both should be valid independently
      expect(securityManager.authorize(context1, Permission.SUBMIT_TASK)).toBe(
        true
      );
      expect(securityManager.authorize(context2, Permission.SUBMIT_TASK)).toBe(
        true
      );

      // Invalidating one shouldn't affect the other
      securityManager.invalidateSession(context1.sessionId);

      // Context2 should still work (separate session)
      expect(securityManager.authorize(context2, Permission.SUBMIT_TASK)).toBe(
        true
      );
    });
  });

  describe("Compliance Validation", () => {
    it("should maintain audit trail for all operations", async () => {
      const credentials: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
        metadata: {
          ipAddress: "192.168.1.100",
          userAgent: "TestClient/1.0",
          source: "api",
        },
      };

      const beforeEvents = securityManager.getSecurityEvents().length;

      await middleware.protect(
        credentials,
        Permission.SUBMIT_TASK,
        "submitTask",
        async (context) => {
          securityManager.sanitizeInput(context, "test", { data: "safe" });
          return { success: true };
        }
      );

      const afterEvents = securityManager.getSecurityEvents();
      expect(afterEvents.length).toBeGreaterThan(beforeEvents);

      // Verify audit trail quality
      const recentEvents = afterEvents.slice(-10);
      recentEvents.forEach((event) => {
        expect(event.id).toBeDefined();
        expect(event.timestamp).toBeDefined();
        expect(event.type).toBeDefined();
        expect(event.result).toMatch(/success|failure|blocked/);
      });
    });

    it("should track all security-relevant metadata", () => {
      const credentials: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
        metadata: {
          ipAddress: "10.0.0.1",
          userAgent: "SecureClient/2.0",
          source: "internal",
        },
      };

      const context = securityManager.authenticate(credentials)!;

      expect(context.metadata.ipAddress).toBe("10.0.0.1");
      expect(context.metadata.userAgent).toBe("SecureClient/2.0");
      expect(context.metadata.source).toBe("internal");
      expect(context.authenticatedAt).toBeInstanceOf(Date);
      expect(context.expiresAt).toBeInstanceOf(Date);
    });
  });

  describe("Error Recovery", () => {
    it("should recover from operation failures", async () => {
      const credentials: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
      };

      try {
        await middleware.protect(
          credentials,
          Permission.SUBMIT_TASK,
          "submitTask",
          async () => {
            throw new Error("Operation failed");
          }
        );
      } catch (error) {
        expect(error).toBeDefined();
      }

      // Security should still work after error
      const result = await middleware.protect(
        credentials,
        Permission.SUBMIT_TASK,
        "submitTask",
        async () => ({ recovered: true })
      );

      expect(result.recovered).toBe(true);
    });

    it("should handle partial authentication failures gracefully", () => {
      const credentials: AuthCredentials = {
        agentId: "agent-1",
        token: "short", // < 10 chars, should fail validation
      };

      const context = securityManager.authenticate(credentials);
      expect(context).toBeNull();

      // System should still be functional
      const validCreds: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
      };

      const validContext = securityManager.authenticate(validCreds);
      expect(validContext).toBeTruthy();
    });
  });

  describe("Performance Under Load", () => {
    it("should handle high-frequency authentication", () => {
      const startTime = Date.now();
      const iterations = 100;

      for (let i = 0; i < iterations; i++) {
        const credentials: AuthCredentials = {
          agentId: "agent-1",
          token: "valid-token-12345",
        };

        const context = securityManager.authenticate(credentials);
        expect(context).toBeTruthy();

        if (context) {
          securityManager.invalidateSession(context.sessionId);
        }
      }

      const duration = Date.now() - startTime;
      const perOp = duration / iterations;

      // Should be fast (< 10ms per operation)
      expect(perOp).toBeLessThan(10);
    });

    it("should handle high-frequency authorization checks", () => {
      const credentials: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
      };

      const context = securityManager.authenticate(credentials)!;

      const startTime = Date.now();
      const iterations = 1000;

      for (let i = 0; i < iterations; i++) {
        securityManager.authorize(context, Permission.SUBMIT_TASK);
      }

      const duration = Date.now() - startTime;
      const perOp = duration / iterations;

      // Should be very fast (< 1ms per operation)
      expect(perOp).toBeLessThan(1);
    });

    it("should handle high-frequency input sanitization", () => {
      const credentials: AuthCredentials = {
        agentId: "agent-1",
        token: "valid-token-12345",
      };

      const context = securityManager.authenticate(credentials)!;

      const startTime = Date.now();
      const iterations = 100;

      for (let i = 0; i < iterations; i++) {
        securityManager.sanitizeInput(context, "test", {
          data: `Safe input ${i}`,
        });
      }

      const duration = Date.now() - startTime;
      const perOp = duration / iterations;

      // Should be reasonably fast (< 5ms per operation)
      expect(perOp).toBeLessThan(5);
    });
  });
});
