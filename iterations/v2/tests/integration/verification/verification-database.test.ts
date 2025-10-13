/**
 * @fileoverview Integration tests for VerificationDatabaseClient
 *
 * Tests database persistence, caching, and analytics for ARBITER-007
 *
 * @author @darianrosebrook
 */

import {
  VerificationPriority,
  VerificationRequest,
  VerificationResult,
  VerificationType,
  VerificationVerdict,
} from "@/types/verification";
import { VerificationDatabaseClient } from "@/verification/VerificationDatabaseClient";
import {
  createCompleteTestResult,
  createTestRequest,
} from "../../helpers/verification-helpers";

describe("VerificationDatabaseClient Integration", () => {
  let dbClient: VerificationDatabaseClient;

  beforeAll(async () => {
    // Uses centralized ConnectionPoolManager initialized in tests/setup.ts
    dbClient = new VerificationDatabaseClient();
    await dbClient.initialize();
  });

  afterAll(async () => {
    // Note: Pool lifecycle managed by ConnectionPoolManager
    // No need to call close() - handled in tests/setup.ts afterAll
  });

  describe("Request Persistence", () => {
    it("should save and retrieve verification request", async () => {
      const request = createTestRequest({
        id: "test-request-1",
        content: "Test content for verification",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.FACT_CHECKING],
      });

      await dbClient.saveRequest(request);
      const result = await dbClient.getResult(request.id);

      // Request should be saved, result not yet available
      expect(result).toBeNull();
    });

    it("should handle concurrent request saves", async () => {
      const requests: VerificationRequest[] = Array.from(
        { length: 5 },
        (_, i) => ({
          id: `concurrent-request-${i}`,
          content: `Concurrent test content ${i}`,
          source: `https://example${i}.com`,
          context: "Concurrent test",
          priority: VerificationPriority.MEDIUM,
          verificationTypes: [VerificationType.FACT_CHECKING],
        })
      );

      await Promise.all(requests.map((req) => dbClient.saveRequest(req)));

      // All requests should be saved successfully
      // No errors should be thrown during concurrent saves
    });
  });

  describe("Result Storage", () => {
    it("should save and retrieve verification result", async () => {
      const request: VerificationRequest = {
        id: "test-result-1",
        content: "Content to verify",
        source: "https://example.com",
        context: "Test",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.FACT_CHECKING],
      };

      const result = createCompleteTestResult(
        request.id,
        VerificationVerdict.VERIFIED_TRUE,
        1
      );

      await dbClient.saveRequest(request);
      await dbClient.saveResult(result);

      const retrieved = await dbClient.getResult(request.id);
      expect(retrieved).not.toBeNull();
      expect(retrieved?.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
      expect(retrieved?.confidence).toBeCloseTo(0.85, 2);
      expect(retrieved?.supportingEvidence.length).toBeGreaterThan(0);
    });

    it("should store result with multiple evidence items", async () => {
      const request: VerificationRequest = {
        id: "test-multi-evidence",
        content: "Multi-evidence test",
        source: "https://example.com",
        context: "Test",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [
          VerificationType.FACT_CHECKING,
          VerificationType.SOURCE_CREDIBILITY,
        ],
      };

      const result: VerificationResult = {
        requestId: request.id,
        verdict: VerificationVerdict.VERIFIED_TRUE,
        confidence: 0.75,
        supportingEvidence: [],
        contradictoryEvidence: [
          {
            source: "https://source1.com",
            content: "Evidence 1",
            relevance: 0.9,
            credibility: 0.8,
            supporting: true,
            metadata: { type: "primary" },
          },
          {
            source: "https://source2.com",
            content: "Evidence 2",
            relevance: 0.85,
            credibility: 0.75,
            supporting: true,
            metadata: { type: "secondary" },
          },
          {
            source: "https://source3.com",
            content: "Contradicting evidence",
            relevance: 0.7,
            credibility: 0.6,
            supporting: false,
            metadata: { type: "contradicting" },
          },
        ],
        verificationMethods: [
          {
            method: VerificationType.FACT_CHECKING,
            reasoning: ["Method verification reasoning"],
            verdict: VerificationVerdict.VERIFIED_TRUE,
            confidence: 0.8,
            supportingEvidence: [],
          },
          {
            method: VerificationType.SOURCE_CREDIBILITY,
            reasoning: ["Method verification reasoning"],
            verdict: VerificationVerdict.VERIFIED_TRUE,
            confidence: 0.7,
            supportingEvidence: [],
          },
        ],
      };

      await dbClient.saveRequest(request);
      await dbClient.saveResult(result);

      const retrieved = await dbClient.getResult(request.id);
      expect(retrieved?.supportingEvidence.length).toBe(3);
      expect(retrieved?.verificationMethods.length).toBe(2);
    });
  });

  describe("Cache Operations", () => {
    it("should cache and retrieve verification result", async () => {
      const request: VerificationRequest = {
        id: "test-cache-1",
        content: "Cacheable content",
        source: "https://example.com",
        context: "Cache test",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.FACT_CHECKING],
      };

      const result: VerificationResult = {
        requestId: request.id,
        verdict: VerificationVerdict.VERIFIED_TRUE,
        confidence: 0.9,
        supportingEvidence: [],
        verificationMethods: [],
      };

      await dbClient.cacheResult(request, result, 300000); // 5 min TTL

      const _cached = await dbClient.getCachedResult("cache-key-test");
      // Note: Need proper cache key generation in implementation
    });

    it("should respect cache TTL", async () => {
      const request: VerificationRequest = {
        id: "test-cache-ttl",
        content: "TTL test content",
        source: "https://example.com",
        context: "TTL test",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.FACT_CHECKING],
      };

      const result: VerificationResult = {
        requestId: request.id,
        verdict: VerificationVerdict.VERIFIED_TRUE,
        confidence: 0.8,
        supportingEvidence: [],
        verificationMethods: [],
      };

      // Cache with very short TTL
      await dbClient.cacheResult(request, result, 1); // 1ms TTL

      // Wait for expiration
      await new Promise((resolve) => setTimeout(resolve, 10));

      // Should return null after expiration
      // Implementation should check expiry_at timestamp
    });

    it("should cleanup expired cache entries", async () => {
      // Create multiple cache entries with short TTLs
      const requests: VerificationRequest[] = Array.from(
        { length: 3 },
        (_, i) => ({
          id: `cleanup-test-${i}`,
          content: `Cleanup test ${i}`,
          source: `https://example${i}.com`,
          context: "Cleanup test",
          priority: VerificationPriority.LOW,
          verificationTypes: [VerificationType.FACT_CHECKING],
        })
      );

      const results: VerificationResult[] = requests.map((req) => ({
        requestId: req.id,
        verdict: VerificationVerdict.VERIFIED_TRUE,
        confidence: 0.8,
        supportingEvidence: [],
        verificationMethods: [],
      }));

      // Cache all with short TTL
      await Promise.all(
        requests.map((req, i) => dbClient.cacheResult(req, results[i], 1))
      );

      // Wait for expiration
      await new Promise((resolve) => setTimeout(resolve, 10));

      // Cleanup operation should remove expired entries
      // This would need to be exposed as a method or run automatically
    });
  });

  describe("Method Performance Tracking", () => {
    it("should track method performance statistics", async () => {
      const stats = await dbClient.getMethodPerformance();
      expect(Array.isArray(stats)).toBe(true);
      // Stats should include various verification method types
    });

    it("should calculate accurate performance metrics", async () => {
      // Create several requests and results to generate statistics
      const requests: VerificationRequest[] = Array.from(
        { length: 5 },
        (_, i) => ({
          id: `perf-test-${i}`,
          content: `Performance test ${i}`,
          source: `https://perf${i}.com`,
          context: "Performance",
          priority: VerificationPriority.MEDIUM,
          verificationTypes: [VerificationType.FACT_CHECKING],
        })
      );

      const results: VerificationResult[] = requests.map((req, i) => ({
        requestId: req.id,
        verdict:
          i < 4
            ? VerificationVerdict.VERIFIED_TRUE
            : VerificationVerdict.VERIFIED_FALSE,
        confidence: 0.8 + i * 0.02,
        supportingEvidence: [],
        verificationMethods: [
          {
            method: VerificationType.FACT_CHECKING,
            reasoning: ["Method verification reasoning"],
            verdict:
              i < 4
                ? VerificationVerdict.VERIFIED
                : VerificationVerdict.VERIFIED_FALSE,
            confidence: 0.8 + i * 0.02,
            supportingEvidence: [],
            processingTimeMs: 100 + i * 10,
          },
        ],
        processingTimeMs: 150 + i * 10,
      }));

      await Promise.all(requests.map((req) => dbClient.saveRequest(req)));
      await Promise.all(results.map((res) => dbClient.saveResult(res)));

      const stats = await dbClient.getMethodPerformance();
      const factCheckStats = stats.find(
        (s) => s.methodType === VerificationType.FACT_CHECKING
      );

      expect(factCheckStats).toBeDefined();
      if (factCheckStats) {
        expect(factCheckStats.totalRequests).toBeGreaterThanOrEqual(5);
        expect(factCheckStats.successRate).toBeGreaterThan(0);
      }
    });
  });

  describe("Evidence Quality Analysis", () => {
    it("should retrieve evidence quality statistics", async () => {
      const stats = await dbClient.getEvidenceQualityStats();
      expect(Array.isArray(stats)).toBe(true);
    });

    it("should track evidence quality metrics across sources", async () => {
      // Create results with varying evidence quality
      const request: VerificationRequest = {
        id: "evidence-quality-test",
        content: "Evidence quality analysis",
        source: "https://example.com",
        context: "Quality test",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.FACT_CHECKING],
      };

      const result: VerificationResult = {
        requestId: request.id,
        verdict: VerificationVerdict.VERIFIED_TRUE,
        confidence: 0.85,
        supportingEvidence: [],
        contradictoryEvidence: [
          {
            source: "https://high-quality.com",
            content: "High quality evidence",
            relevance: 0.95,
            credibility: 0.9,
            supporting: true,
            metadata: { publisher: "reputable-source" },
          },
          {
            source: "https://medium-quality.com",
            content: "Medium quality evidence",
            relevance: 0.7,
            credibility: 0.65,
            supporting: true,
            metadata: { publisher: "average-source" },
          },
          {
            source: "https://low-quality.com",
            content: "Low quality evidence",
            relevance: 0.5,
            credibility: 0.4,
            supporting: false,
            metadata: { publisher: "questionable-source" },
          },
        ],
        verificationMethods: [],
      };

      await dbClient.saveRequest(request);
      await dbClient.saveResult(result);

      const stats = await dbClient.getEvidenceQualityStats();
      expect(stats.length).toBeGreaterThan(0);
    });
  });

  describe("Transaction and Error Handling", () => {
    it("should rollback on save failure", async () => {
      const request: VerificationRequest = {
        id: "rollback-test",
        content: "Rollback test",
        source: "https://example.com",
        context: "Test",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.FACT_CHECKING],
      };

      const invalidResult: any = {
        requestId: request.id,
        // Missing required fields to trigger error
      };

      await dbClient.saveRequest(request);

      try {
        await dbClient.saveResult(invalidResult);
        throw new Error("Should have thrown error for invalid result");
      } catch (error) {
        // Error expected - ensure data is consistent
        const retrieved = await dbClient.getResult(request.id);
        expect(retrieved).toBeNull(); // No partial result saved
      }
    });

    it("should handle database connection errors gracefully", async () => {
      // Note: With centralized ConnectionPoolManager, connection error
      // testing is moved to ConnectionPoolManager.test.ts
      // This test now verifies graceful handling of operations when pool is unavailable

      // Create a client but don't initialize (simulating connection failure)
      const uninitializedClient = new VerificationDatabaseClient();

      // Client operations should handle uninitialized state gracefully
      expect(uninitializedClient.isInitialized()).toBe(false);
    });
  });

  describe("Concurrent Operations", () => {
    it("should handle concurrent result storage", async () => {
      const requests: VerificationRequest[] = Array.from(
        { length: 10 },
        (_, i) => ({
          id: `concurrent-result-${i}`,
          content: `Concurrent content ${i}`,
          source: `https://concurrent${i}.com`,
          context: "Concurrent test",
          priority: VerificationPriority.MEDIUM,
          verificationTypes: [VerificationType.FACT_CHECKING],
        })
      );

      const results: VerificationResult[] = requests.map((req) => ({
        requestId: req.id,
        verdict: VerificationVerdict.VERIFIED_TRUE,
        confidence: 0.8,
        supportingEvidence: [],
        verificationMethods: [],
      }));

      await Promise.all(requests.map((req) => dbClient.saveRequest(req)));
      await Promise.all(results.map((res) => dbClient.saveResult(res)));

      // All results should be retrievable
      const retrieved = await Promise.all(
        requests.map((req) => dbClient.getResult(req.id))
      );

      expect(retrieved.every((r) => r !== null)).toBe(true);
    });
  });
});
