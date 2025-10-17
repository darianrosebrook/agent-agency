/**
 * Performance Tests: Arbitration System Load Testing
 *
 * Comprehensive performance testing for the arbitration system under various load conditions,
 * measuring throughput, latency, memory usage, and scalability characteristics.
 *
 * Test Coverage (15+ performance tests):
 * - Throughput testing under concurrent load
 * - Latency measurements for different operation types
 * - Memory usage patterns during sustained load
 * - Scalability testing with increasing concurrent sessions
 * - Resource utilization monitoring
 * - Performance degradation analysis
 * - Stress testing with system limits
 */

import { ArbitrationOrchestrator } from "@/arbitration/ArbitrationOrchestrator";
import {
  ArbitrationState,
  ConstitutionalRule,
  ConstitutionalViolation,
  RuleCategory,
  ViolationSeverity,
} from "@/types/arbitration";

describe("ARBITER-015 Performance: Arbitration System Load Testing", () => {
  let orchestrator: ArbitrationOrchestrator;

  beforeEach(() => {
    orchestrator = new ArbitrationOrchestrator({
      enableWaivers: true,
      enableAppeals: true,
      trackPerformance: true,
      maxConcurrentSessions: 50, // Higher limit for performance testing
      sessionTimeoutMs: 120000, // 2 minutes for performance tests
    });
  });

  afterEach(async () => {
    // Clean up all sessions
    const activeSessions = orchestrator.getActiveSessions();
    for (const session of activeSessions) {
      try {
        await orchestrator.completeSession(session.id);
      } catch (e) {
        // Ignore cleanup errors in performance tests
      }
    }
    orchestrator.clear();
  });

  // Helper: Create a lightweight rule for performance testing
  const createPerfRule = (index: number): ConstitutionalRule => ({
    id: `perf-rule-${index}`,
    version: "1.0.0",
    category: RuleCategory.CODE_QUALITY,
    title: `Performance Rule ${index}`,
    description: `Performance test rule ${index} for load testing`,
    condition: "true", // Always passes for performance testing
    severity: ViolationSeverity.MINOR,
    waivable: false,
    requiredEvidence: [],
    precedents: [],
    effectiveDate: new Date(),
    metadata: {},
  });

  // Helper: Create a lightweight violation for performance testing
  const createPerfViolation = (ruleId: string, index: number): ConstitutionalViolation => ({
    id: `perf-violation-${index}`,
    ruleId,
    severity: ViolationSeverity.MINOR,
    description: `Performance violation ${index}`,
    evidence: [`perf-evidence-${index}`],
    detectedAt: new Date(),
    violator: `perf-agent-${index}`,
    context: {},
  });

  // Helper: Measure execution time
  const measureTime = async (fn: () => Promise<void>): Promise<number> => {
    const start = process.hrtime.bigint();
    await fn();
    const end = process.hrtime.bigint();
    return Number(end - start) / 1_000_000; // Convert to milliseconds
  };

  // Helper: Get memory usage
  const getMemoryUsage = (): NodeJS.MemoryUsage => {
    return process.memoryUsage();
  };

  describe("Performance Test 1-5: Throughput and Latency", () => {
    it("should handle 10 concurrent arbitration sessions within 5 seconds", async () => {
      const sessionCount = 10;
      const sessions: string[] = [];

      const createSessionsTime = await measureTime(async () => {
        // Create sessions concurrently
        const promises = Array.from({ length: sessionCount }, async (_, i) => {
          const rule = createPerfRule(i);
          const violation = createPerfViolation(rule.id, i);
          const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
          sessions.push(session.id);
          return session;
        });
        await Promise.all(promises);
      });

      expect(createSessionsTime).toBeLessThan(5000); // 5 seconds for 10 sessions
      expect(orchestrator.getActiveSessions()).toHaveLength(sessionCount);

      // Clean up
      for (const sessionId of sessions) {
        await orchestrator.completeSession(sessionId);
      }
    });

    it("should process 20 sequential arbitration workflows within performance budget", async () => {
      const workflowCount = 20;
      const startTime = Date.now();

      for (let i = 0; i < workflowCount; i++) {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-perf");
        await orchestrator.completeSession(session.id);
      }

      const totalTime = Date.now() - startTime;
      const avgTimePerWorkflow = totalTime / workflowCount;

      expect(totalTime).toBeLessThan(30000); // 30 seconds total
      expect(avgTimePerWorkflow).toBeLessThan(1500); // 1.5 seconds per workflow

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(workflowCount);
      expect(stats.completedSessions).toBe(workflowCount);
    });

    it("should maintain sub-100ms latency for individual operations", async () => {
      const rule = createPerfRule(0);
      const violation = createPerfViolation(rule.id, 0);

      // Measure individual operation latencies
      const session = await orchestrator.startSession(violation, [rule], ["agent-perf"]);

      const evaluateTime = await measureTime(async () => {
        await orchestrator.evaluateRules(session.id);
      });

      const verdictTime = await measureTime(async () => {
        await orchestrator.generateVerdict(session.id, "arbiter-perf");
      });

      const completeTime = await measureTime(async () => {
        await orchestrator.completeSession(session.id);
      });

      expect(evaluateTime).toBeLessThan(100);
      expect(verdictTime).toBeLessThan(100);
      expect(completeTime).toBeLessThan(100);

      const metrics = orchestrator.getSessionMetrics(session.id);
      expect(metrics!.ruleEvaluationMs).toBeGreaterThanOrEqual(0);
      expect(metrics!.verdictGenerationMs).toBeGreaterThanOrEqual(0);
    });

    it("should handle burst load of 5 concurrent complex workflows", async () => {
      const burstSize = 5;
      const startTime = Date.now();

      // Create complex rules with multiple precedents
      const createComplexRule = (index: number): ConstitutionalRule => ({
        ...createPerfRule(index),
        description: "Complex rule with detailed description and multiple evidence requirements for performance testing under load conditions".repeat(5),
        requiredEvidence: Array.from({ length: 10 }, (_, i) => `evidence-${i}`),
        metadata: {
          complexData: Array.from({ length: 100 }, (_, i) => ({ key: `key-${i}`, value: `value-${i}` })),
        },
      });

      const promises = Array.from({ length: burstSize }, async (_, i) => {
        const rule = createComplexRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-perf");
        await orchestrator.completeSession(session.id);

        return session.id;
      });

      await Promise.all(promises);
      const burstTime = Date.now() - startTime;

      expect(burstTime).toBeLessThan(10000); // 10 seconds for burst of 5 complex workflows

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(burstSize);
      expect(stats.completedSessions).toBe(burstSize);
    });

    it("should demonstrate linear scalability up to 25 concurrent sessions", async () => {
      const sessionCounts = [5, 10, 15, 20, 25];
      const scalabilityResults: Array<{ count: number; time: number; avgTime: number }> = [];

      for (const count of sessionCounts) {
        const startTime = Date.now();

        const promises = Array.from({ length: count }, async (_, i) => {
          const rule = createPerfRule(i);
          const violation = createPerfViolation(rule.id, i);
          const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
          await orchestrator.evaluateRules(session.id);
          await orchestrator.generateVerdict(session.id, "arbiter-perf");
          await orchestrator.completeSession(session.id);
          return session.id;
        });

        await Promise.all(promises);
        const totalTime = Date.now() - startTime;
        const avgTime = totalTime / count;

        scalabilityResults.push({ count, time: totalTime, avgTime });

        // Verify all sessions completed
        const stats = orchestrator.getStatistics();
        expect(stats.totalSessions).toBe(count);
        expect(stats.completedSessions).toBe(count);

        // Reset for next iteration
        orchestrator.clear();
      }

      // Analyze scalability - should not degrade dramatically
      const baselineAvg = scalabilityResults[0].avgTime;
      for (const result of scalabilityResults) {
        const degradation = result.avgTime / baselineAvg;
        expect(degradation).toBeLessThan(3.0); // Allow up to 3x degradation for 5x load increase
      }
    });
  });

  describe("Performance Test 6-10: Memory Usage and Resource Management", () => {
    it("should maintain stable memory usage during sustained load", async () => {
      const initialMemory = getMemoryUsage();
      const sustainedSessions = 20;

      const memorySamples: NodeJS.MemoryUsage[] = [];

      for (let i = 0; i < sustainedSessions; i++) {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-perf");

        // Sample memory every 5 sessions
        if (i % 5 === 0) {
          memorySamples.push(getMemoryUsage());
        }

        await orchestrator.completeSession(session.id);
      }

      const finalMemory = getMemoryUsage();

      // Memory usage should not grow unbounded
      const memoryGrowth = finalMemory.heapUsed - initialMemory.heapUsed;
      expect(memoryGrowth).toBeLessThan(50 * 1024 * 1024); // Less than 50MB growth

      // Memory should be stable (not continuously increasing)
      if (memorySamples.length >= 3) {
        const firstSample = memorySamples[0].heapUsed;
        const lastSample = memorySamples[memorySamples.length - 1].heapUsed;
        const memoryTrend = lastSample - firstSample;
        expect(memoryTrend).toBeLessThan(10 * 1024 * 1024); // Less than 10MB trend increase
      }
    });

    it("should handle large evidence arrays without memory issues", async () => {
      const largeEvidenceCount = 1000;
      const largeEvidence = Array.from({ length: largeEvidenceCount }, (_, i) =>
        `Evidence item ${i} with substantial content that demonstrates memory handling capabilities. `.repeat(10)
      );

      const rule = createPerfRule(0);
      const violation = createPerfViolation(rule.id, 0);
      violation.evidence = largeEvidence;

      const initialMemory = getMemoryUsage();

      const session = await orchestrator.startSession(violation, [rule], ["agent-large-evidence"]);
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-perf");
      await orchestrator.completeSession(session.id);

      const finalMemory = getMemoryUsage();
      const memoryIncrease = finalMemory.heapUsed - initialMemory.heapUsed;

      // Should handle large evidence without excessive memory growth
      expect(memoryIncrease).toBeLessThan(100 * 1024 * 1024); // Less than 100MB for large evidence

      const finalSession = orchestrator.getSession(session.id);
      expect(finalSession.evidence).toHaveLength(largeEvidenceCount);
    });

    it("should cleanup resources properly after session completion", async () => {
      const sessionCount = 10;
      const initialActiveCount = orchestrator.getActiveSessions().length;

      // Create and complete sessions
      for (let i = 0; i < sessionCount; i++) {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-perf");
        await orchestrator.completeSession(session.id);
      }

      // All sessions should be cleaned up
      expect(orchestrator.getActiveSessions()).toHaveLength(initialActiveCount);

      const stats = orchestrator.getStatistics();
      expect(stats.completedSessions).toBe(sessionCount);
      expect(stats.activeSessions).toBe(initialActiveCount);
    });

    it("should handle memory pressure from concurrent complex operations", async () => {
      const concurrentComplex = 8;
      const initialMemory = getMemoryUsage();

      const promises = Array.from({ length: concurrentComplex }, async (_, i) => {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        // Add complex metadata to increase memory pressure
        rule.metadata = {
          complexData: Array.from({ length: 1000 }, (_, j) => ({
            nested: {
              data: `Complex nested data ${j} for session ${i}`,
              array: Array.from({ length: 50 }, (_, k) => `Item ${k}`),
            },
          })),
        };

        violation.context = {
          largeContext: Array.from({ length: 500 }, (_, j) => ({
            key: `context-${j}`,
            value: `Large context value ${j} for performance testing`,
          })),
        };

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-perf");
        await orchestrator.completeSession(session.id);

        return session.id;
      });

      await Promise.all(promises);

      const finalMemory = getMemoryUsage();
      const memoryIncrease = finalMemory.heapUsed - initialMemory.heapUsed;

      // Should handle complex concurrent operations without excessive memory usage
      expect(memoryIncrease).toBeLessThan(200 * 1024 * 1024); // Less than 200MB for complex concurrent ops

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(concurrentComplex);
      expect(stats.completedSessions).toBe(concurrentComplex);
    });

    it("should demonstrate efficient garbage collection under load", async () => {
      // Force garbage collection if available (only in certain Node.js versions)
      if (global.gc) {
        global.gc();
        await new Promise(resolve => setImmediate(resolve));
      }

      const initialMemory = getMemoryUsage();
      const loadTestSessions = 15;

      // Run load test
      const promises = Array.from({ length: loadTestSessions }, async (_, i) => {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-perf");
        await orchestrator.completeSession(session.id);

        return session.id;
      });

      await Promise.all(promises);

      // Force GC again if available
      if (global.gc) {
        global.gc();
        await new Promise(resolve => setImmediate(resolve));
      }

      const finalMemory = getMemoryUsage();
      const memoryIncrease = finalMemory.heapUsed - initialMemory.heapUsed;

      // After GC, memory usage should be reasonable
      expect(memoryIncrease).toBeLessThan(50 * 1024 * 1024); // Less than 50MB after GC

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(loadTestSessions);
      expect(stats.completedSessions).toBe(loadTestSessions);
    });
  });

  describe("Performance Test 11-15: Stress Testing and Limits", () => {
    it("should handle maximum concurrent sessions without failure", async () => {
      const maxConcurrent = 50; // Match orchestrator config
      const sessions: string[] = [];

      const startTime = Date.now();

      // Create maximum concurrent sessions
      const promises = Array.from({ length: maxConcurrent }, async (_, i) => {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);
        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        sessions.push(session.id);
        return session;
      });

      await Promise.all(promises);
      const creationTime = Date.now() - startTime;

      expect(orchestrator.getActiveSessions()).toHaveLength(maxConcurrent);
      expect(creationTime).toBeLessThan(10000); // 10 seconds for max concurrent

      // Try to create one more - should fail
      const extraRule = createPerfRule(maxConcurrent);
      const extraViolation = createPerfViolation(extraRule.id, maxConcurrent);

      await expect(
        orchestrator.startSession(extraViolation, [extraRule], ["agent-extra"])
      ).rejects.toThrow("Maximum concurrent sessions reached");

      // Clean up all sessions
      const cleanupPromises = sessions.map(sessionId =>
        orchestrator.completeSession(sessionId)
      );
      await Promise.all(cleanupPromises);
    });

    it("should maintain performance under sustained high load", async () => {
      const sustainedLoad = 30;
      const timeWindows: number[] = [];

      for (let i = 0; i < sustainedLoad; i++) {
        const windowStart = Date.now();

        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-perf");
        await orchestrator.completeSession(session.id);

        const windowTime = Date.now() - windowStart;
        timeWindows.push(windowTime);

        // Check performance degradation every 10 sessions
        if (i > 0 && i % 10 === 0) {
          const recentWindows = timeWindows.slice(-10);
          const avgRecent = recentWindows.reduce((a, b) => a + b, 0) / recentWindows.length;
          const initialWindows = timeWindows.slice(0, 10);
          const avgInitial = initialWindows.reduce((a, b) => a + b, 0) / initialWindows.length;

          const degradation = avgRecent / avgInitial;
          expect(degradation).toBeLessThan(2.0); // Allow up to 2x degradation under sustained load
        }
      }

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(sustainedLoad);
      expect(stats.completedSessions).toBe(sustainedLoad);
    });

    it("should handle rapid session failure and recovery", async () => {
      const failureTestSessions = 20;
      const failedSessions: string[] = [];

      // Create sessions and fail half of them
      for (let i = 0; i < failureTestSessions; i++) {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);

        if (i % 2 === 0) {
          // Fail even-numbered sessions
          await orchestrator.failSession(session.id, new Error("Simulated failure"));
          failedSessions.push(session.id);
        } else {
          // Complete odd-numbered sessions
          await orchestrator.evaluateRules(session.id);
          await orchestrator.generateVerdict(session.id, "arbiter-perf");
          await orchestrator.completeSession(session.id);
        }
      }

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(failureTestSessions);
      expect(stats.completedSessions).toBe(failureTestSessions / 2);
      expect(stats.failedSessions).toBe(failureTestSessions / 2);

      // System should still be functional
      const newRule = createPerfRule(failureTestSessions);
      const newViolation = createPerfViolation(newRule.id, failureTestSessions);
      const newSession = await orchestrator.startSession(newViolation, [newRule], ["agent-recovery"]);

      expect(newSession.state).toBe(ArbitrationState.RULE_EVALUATION);

      await orchestrator.completeSession(newSession.id);
    });

    it("should demonstrate graceful degradation under extreme load", async () => {
      const extremeLoad = 100;
      const batchSize = 10;
      const batchTimes: number[] = [];

      // Process in batches to measure degradation
      for (let batch = 0; batch < extremeLoad / batchSize; batch++) {
        const batchStart = Date.now();

        const promises = Array.from({ length: batchSize }, async (_, i) => {
          const index = batch * batchSize + i;
          const rule = createPerfRule(index);
          const violation = createPerfViolation(rule.id, index);

          const session = await orchestrator.startSession(violation, [rule], [`agent-${index}`]);
          await orchestrator.evaluateRules(session.id);
          await orchestrator.generateVerdict(session.id, "arbiter-perf");
          await orchestrator.completeSession(session.id);

          return session.id;
        });

        await Promise.all(promises);
        const batchTime = Date.now() - batchStart;
        batchTimes.push(batchTime);

        // Check for graceful degradation (should not fail completely)
        const stats = orchestrator.getStatistics();
        expect(stats.totalSessions).toBe((batch + 1) * batchSize);
        expect(stats.completedSessions).toBe((batch + 1) * batchSize);
      }

      // Analyze performance degradation pattern
      const firstBatch = batchTimes[0];
      const lastBatch = batchTimes[batchTimes.length - 1];
      const degradationRatio = lastBatch / firstBatch;

      // Allow significant degradation under extreme load but not complete failure
      expect(degradationRatio).toBeLessThan(5.0);
      expect(orchestrator.getStatistics().totalSessions).toBe(extremeLoad);
    });

    it("should maintain data integrity under concurrent load", async () => {
      const integrityTestSessions = 15;
      const sessionData: Array<{ id: string; expectedState: ArbitrationState }> = [];

      // Create sessions with expected final states
      const promises = Array.from({ length: integrityTestSessions }, async (_, i) => {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);

        // Some sessions get full workflow, others partial
        if (i % 3 === 0) {
          await orchestrator.evaluateRules(session.id);
          await orchestrator.generateVerdict(session.id, "arbiter-perf");
          await orchestrator.completeSession(session.id);
          sessionData.push({ id: session.id, expectedState: ArbitrationState.COMPLETED });
        } else if (i % 3 === 1) {
          await orchestrator.evaluateRules(session.id);
          await orchestrator.generateVerdict(session.id, "arbiter-perf");
          sessionData.push({ id: session.id, expectedState: ArbitrationState.VERDICT_GENERATION });
        } else {
          await orchestrator.evaluateRules(session.id);
          sessionData.push({ id: session.id, expectedState: ArbitrationState.VERDICT_GENERATION });
        }

        return session.id;
      });

      await Promise.all(promises);

      // Verify data integrity - all sessions should have correct state and data
      for (const { id, expectedState } of sessionData) {
        const session = orchestrator.getSession(id);
        expect(session.state).toBe(expectedState);
        expect(session.id).toBe(id);
        expect(session.violation).toBeDefined();
        expect(session.rulesEvaluated).toHaveLength(1);

        // Verify session metrics exist and are valid
        const metrics = orchestrator.getSessionMetrics(id);
        expect(metrics).toBeDefined();
        expect(metrics!.sessionId).toBe(id);
        expect(metrics!.totalDurationMs).toBeGreaterThanOrEqual(0);
      }

      // Complete remaining sessions
      for (const { id } of sessionData.filter(s => s.expectedState !== ArbitrationState.COMPLETED)) {
        await orchestrator.completeSession(id);
      }

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(integrityTestSessions);
      expect(stats.completedSessions).toBe(integrityTestSessions);
    });
  });

  describe("Performance Test 16-20: Benchmarking and Metrics", () => {
    it("should provide detailed performance metrics for analysis", async () => {
      const benchmarkSessions = 12;

      for (let i = 0; i < benchmarkSessions; i++) {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-perf");
        await orchestrator.completeSession(session.id);
      }

      // Collect comprehensive metrics
      const allMetrics = orchestrator.getAllMetrics();
      const stats = orchestrator.getStatistics();

      // Verify metrics completeness
      expect(allMetrics).toHaveLength(benchmarkSessions);
      expect(stats.totalSessions).toBe(benchmarkSessions);
      expect(stats.completedSessions).toBe(benchmarkSessions);

      // Analyze metrics quality
      for (const metrics of allMetrics) {
        expect(metrics.sessionId).toMatch(/^ARB-/);
        expect(metrics.ruleEvaluationMs).toBeGreaterThanOrEqual(0);
        expect(metrics.verdictGenerationMs).toBeGreaterThanOrEqual(0);
        expect(metrics.totalDurationMs).toBeGreaterThanOrEqual(0);
        expect(metrics.rulesEvaluated).toBeGreaterThan(0);
        expect(metrics.finalState).toBe(ArbitrationState.COMPLETED);

        // Performance assertions
        expect(metrics.ruleEvaluationMs).toBeLessThan(500); // Rule evaluation should be fast
        expect(metrics.verdictGenerationMs).toBeLessThan(500); // Verdict generation should be fast
        expect(metrics.totalDurationMs).toBeLessThan(2000); // Total workflow should be reasonable
      }

      // Calculate aggregate statistics
      const avgRuleEvalTime = allMetrics.reduce((sum, m) => sum + m.ruleEvaluationMs, 0) / allMetrics.length;
      const avgVerdictTime = allMetrics.reduce((sum, m) => sum + m.verdictGenerationMs, 0) / allMetrics.length;
      const avgTotalTime = allMetrics.reduce((sum, m) => sum + m.totalDurationMs, 0) / allMetrics.length;

      expect(avgRuleEvalTime).toBeLessThan(100);
      expect(avgVerdictTime).toBeLessThan(100);
      expect(avgTotalTime).toBeLessThan(500);
    });

    it("should benchmark waiver evaluation performance", async () => {
      const waiverBenchmarkSessions = 8;

      for (let i = 0; i < waiverBenchmarkSessions; i++) {
        const rule = createPerfRule(i);
        rule.waivable = true; // Make waivable for waiver testing

        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-perf");

        // Submit waiver
        const waiverRequest = {
          id: `waiver-perf-${i}`,
          ruleId: rule.id,
          requestedBy: `agent-${i}`,
          justification: `Performance waiver test ${i}`,
          evidence: [`evidence-${i}`],
          requestedDuration: 86400000,
          requestedAt: new Date(),
          context: {},
        };

        await orchestrator.evaluateWaiver(session.id, waiverRequest, "arbiter-perf");
      }

      const allMetrics = orchestrator.getAllMetrics();
      const waiverMetrics = allMetrics.filter(m => m.waiverEvaluationMs !== undefined);

      expect(waiverMetrics).toHaveLength(waiverBenchmarkSessions);

      // Analyze waiver performance
      const avgWaiverTime = waiverMetrics.reduce((sum, m) => sum + (m.waiverEvaluationMs || 0), 0) / waiverMetrics.length;
      expect(avgWaiverTime).toBeLessThan(200); // Waiver evaluation should be reasonably fast

      // Verify all waivers were processed
      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(waiverBenchmarkSessions);
      expect(stats.completedSessions).toBe(waiverBenchmarkSessions);
    });

    it("should benchmark appeal processing performance", async () => {
      const appealBenchmarkSessions = 6;

      for (let i = 0; i < appealBenchmarkSessions; i++) {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-perf");
        await orchestrator.completeSession(session.id);

        // Submit appeal
        const appeal = await orchestrator.submitAppeal(
          session.id,
          `agent-${i}`,
          `Performance appeal test ${i}`,
          [`appeal-evidence-${i}`]
        );

        // Review appeal
        await orchestrator.reviewAppeal(session.id, appeal.id, [`reviewer-${i}`]);
      }

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(appealBenchmarkSessions);
      expect(stats.totalAppeals).toBe(appealBenchmarkSessions);

      // All sessions should complete appeal process
      expect(stats.completedSessions).toBe(appealBenchmarkSessions);
    });

    it("should measure and report memory efficiency metrics", async () => {
      const memoryTestSessions = 10;
      const memorySamples: Array<{ sessionCount: number; memoryUsage: NodeJS.MemoryUsage }> = [];

      memorySamples.push({ sessionCount: 0, memoryUsage: getMemoryUsage() });

      for (let i = 0; i < memoryTestSessions; i++) {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);
        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-perf");
        await orchestrator.completeSession(session.id);

        memorySamples.push({
          sessionCount: i + 1,
          memoryUsage: getMemoryUsage(),
        });
      }

      // Analyze memory efficiency
      const initialMemory = memorySamples[0].memoryUsage.heapUsed;
      const finalMemory = memorySamples[memorySamples.length - 1].memoryUsage.heapUsed;
      const totalMemoryIncrease = finalMemory - initialMemory;
      const avgMemoryPerSession = totalMemoryIncrease / memoryTestSessions;

      // Memory per session should be reasonable
      expect(avgMemoryPerSession).toBeLessThan(5 * 1024 * 1024); // Less than 5MB per session

      // Total memory increase should be bounded
      expect(totalMemoryIncrease).toBeLessThan(50 * 1024 * 1024); // Less than 50MB total

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(memoryTestSessions);
    });

    it("should benchmark component interaction performance", async () => {
      const componentTestSessions = 5;

      // Test each component interaction
      for (let i = 0; i < componentTestSessions; i++) {
        const rule = createPerfRule(i);
        const violation = createPerfViolation(rule.id, i);

        const session = await orchestrator.startSession(violation, [rule], [`agent-${i}`]);

        // Measure component interactions
        const evaluateStart = process.hrtime.bigint();
        await orchestrator.evaluateRules(session.id);
        const evaluateTime = Number(process.hrtime.bigint() - evaluateStart) / 1_000_000;

        const verdictStart = process.hrtime.bigint();
        await orchestrator.generateVerdict(session.id, "arbiter-perf");
        const verdictTime = Number(process.hrtime.bigint() - verdictStart) / 1_000_000;

        await orchestrator.completeSession(session.id);

        // Component interactions should be efficient
        expect(evaluateTime).toBeLessThan(200);
        expect(verdictTime).toBeLessThan(200);
      }

      // Test component access performance
      const components = orchestrator.getComponents();
      expect(components.ruleEngine).toBeDefined();
      expect(components.verdictGenerator).toBeDefined();
      expect(components.waiverInterpreter).toBeDefined();
      expect(components.precedentManager).toBeDefined();
      expect(components.appealArbitrator).toBeDefined();

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(componentTestSessions);
    });
  });
});
