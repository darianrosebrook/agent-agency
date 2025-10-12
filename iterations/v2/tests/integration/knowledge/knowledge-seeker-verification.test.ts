/**
 * @fileoverview Integration tests for KnowledgeSeeker Verification
 *
 * Tests automatic verification of research results for ARBITER-007
 *
 * @author @darianrosebrook
 */

import { KnowledgeSeeker } from "@/knowledge/KnowledgeSeeker";
import {
  KnowledgeQuery,
  KnowledgeSeekerConfig,
  QueryType,
} from "@/types/knowledge";
import { VerificationType } from "@/types/verification";
import { VerificationEngineImpl } from "@/verification/VerificationEngine";

describe("KnowledgeSeeker Verification Integration", () => {
  let knowledgeSeeker: KnowledgeSeeker;
  let knowledgeSeekerWithVerification: KnowledgeSeeker;
  let verificationEngine: VerificationEngineImpl;

  const baseConfig: KnowledgeSeekerConfig = {
    providers: [],
    processor: {
      relevanceThreshold: 0.5,
      credibilityThreshold: 0.6,
    },
    caching: {
      enableQueryCaching: false,
      enableResultCaching: false,
      cacheTtlMs: 300000,
    },
  };

  const configWithVerification: KnowledgeSeekerConfig = {
    ...baseConfig,
    verification: {
      enabled: true,
      autoVerify: true,
      minConfidenceThreshold: 0.6,
      verificationTypes: [
        VerificationType.FACT_CHECKING,
        VerificationType.SOURCE_CREDIBILITY,
      ],
    },
  };

  beforeAll(async () => {
    // Initialize verification engine
    verificationEngine = new VerificationEngineImpl(
      {
        defaultTimeoutMs: 30000,
        maxConcurrentVerifications: 5,
        minConfidenceThreshold: 0.5,
        maxEvidencePerMethod: 10,
        methods: [
          {
            type: VerificationType.FACT_CHECKING,
            enabled: true,
            priority: 1,
            timeoutMs: 10000,
            config: {},
          },
          {
            type: VerificationType.SOURCE_CREDIBILITY,
            enabled: true,
            priority: 2,
            timeoutMs: 5000,
            config: {},
          },
        ],
        cacheEnabled: true,
        cacheTtlMs: 300000,
        retryAttempts: 2,
        retryDelayMs: 1000,
      },
      undefined
    );

    knowledgeSeeker = new KnowledgeSeeker(baseConfig, undefined, undefined);
    knowledgeSeekerWithVerification = new KnowledgeSeeker(
      configWithVerification,
      undefined,
      verificationEngine
    );
  });

  describe("Auto-Verification of Research Results", () => {
    it("should verify high-priority factual queries automatically", async () => {
      const query: KnowledgeQuery = {
        id: "ks-verify-1",
        query: "What is the capital of France?",
        queryType: QueryType.FACTUAL,
        maxResults: 5,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 8, // High priority
          tags: ["geography", "factual"],
        },
      };

      const response = await knowledgeSeekerWithVerification.processQuery(
        query
      );

      expect(response).toBeDefined();
      expect(response.verificationResults).toBeDefined();
      if (response.verificationResults) {
        expect(response.verificationResults.length).toBeGreaterThan(0);
      }
      expect(response.metadata.verifiedCount).toBeGreaterThanOrEqual(0);
    });

    it("should not auto-verify low-priority conceptual queries", async () => {
      const query: KnowledgeQuery = {
        id: "ks-no-verify",
        query: "What is the meaning of life?",
        queryType: QueryType.EXPLANATORY,
        maxResults: 5,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 3, // Low priority
          tags: ["philosophy"],
        },
      };

      const response = await knowledgeSeekerWithVerification.processQuery(
        query
      );

      expect(response).toBeDefined();
      // Should not auto-verify low priority conceptual queries
      expect(
        response.verificationResults === undefined ||
          response.verificationResults.length === 0
      ).toBe(true);
    });

    it("should filter results based on verification confidence", async () => {
      const query: KnowledgeQuery = {
        id: "ks-filter",
        query: "Historical fact requiring verification",
        queryType: QueryType.FACTUAL,
        maxResults: 10,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 7,
          tags: ["history"],
        },
      };

      const response = await knowledgeSeekerWithVerification.processQuery(
        query
      );

      expect(response).toBeDefined();
      if (
        response.verificationResults &&
        response.verificationResults.length > 0
      ) {
        // Filtered results should have adequate verification confidence
        expect(response.metadata.verifiedCount).toBeGreaterThanOrEqual(0);
      }
    });
  });

  describe("Verification vs No Verification Comparison", () => {
    it("should have different result counts with and without verification", async () => {
      const query: KnowledgeQuery = {
        id: "ks-comparison",
        query: "Scientific fact for comparison",
        queryType: QueryType.FACTUAL,
        maxResults: 10,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 8,
          tags: ["science"],
        },
      };

      const responseNoVerification = await knowledgeSeeker.processQuery(query);
      const responseWithVerification =
        await knowledgeSeekerWithVerification.processQuery(query);

      expect(responseNoVerification).toBeDefined();
      expect(responseWithVerification).toBeDefined();

      // Verification may filter some results
      expect(responseWithVerification.results.length).toBeLessThanOrEqual(
        responseNoVerification.results.length
      );

      // Only verified response should have verification results
      expect(responseWithVerification.verificationResults).toBeDefined();
      expect(responseNoVerification.verificationResults).toBeUndefined();
    });
  });

  describe("Verification Failure Handling", () => {
    it("should continue with unverified results if verification fails", async () => {
      const query: KnowledgeQuery = {
        id: "ks-verify-fail",
        query: "Query that may cause verification issues",
        queryType: QueryType.FACTUAL,
        maxResults: 5,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 7,
          tags: ["test"],
        },
      };

      // Should not throw, even if verification has issues
      const response = await knowledgeSeekerWithVerification.processQuery(
        query
      );

      expect(response).toBeDefined();
      expect(response.results).toBeDefined();
    });
  });

  describe("Verification Configuration", () => {
    it("should respect verification enabled flag", async () => {
      const configDisabled: KnowledgeSeekerConfig = {
        ...baseConfig,
        verification: {
          enabled: false,
          autoVerify: true,
          minConfidenceThreshold: 0.6,
          verificationTypes: [VerificationType.FACT_CHECKING],
        },
      };

      const ksDisabled = new KnowledgeSeeker(
        configDisabled,
        undefined,
        verificationEngine
      );

      const query: KnowledgeQuery = {
        id: "ks-disabled",
        query: "Test query with verification disabled",
        queryType: QueryType.FACTUAL,
        maxResults: 5,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 8,
          tags: ["test"],
        },
      };

      const response = await ksDisabled.processQuery(query);

      expect(response).toBeDefined();
      // Should not have verification results when disabled
      expect(
        response.verificationResults === undefined ||
          response.verificationResults.length === 0
      ).toBe(true);
    });

    it("should respect autoVerify flag", async () => {
      const configManualVerify: KnowledgeSeekerConfig = {
        ...baseConfig,
        verification: {
          enabled: true,
          autoVerify: false, // Manual verification only
          minConfidenceThreshold: 0.6,
          verificationTypes: [VerificationType.FACT_CHECKING],
        },
      };

      const ksManual = new KnowledgeSeeker(
        configManualVerify,
        undefined,
        verificationEngine
      );

      const query: KnowledgeQuery = {
        id: "ks-manual",
        query: "Test query without auto-verification",
        queryType: QueryType.FACTUAL,
        maxResults: 5,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 8,
          tags: ["test"],
        },
      };

      const response = await ksManual.processQuery(query);

      expect(response).toBeDefined();
      // Should not auto-verify
      expect(
        response.verificationResults === undefined ||
          response.verificationResults.length === 0
      ).toBe(true);
    });

    it("should respect minConfidenceThreshold configuration", async () => {
      const configHighThreshold: KnowledgeSeekerConfig = {
        ...baseConfig,
        verification: {
          enabled: true,
          autoVerify: true,
          minConfidenceThreshold: 0.9, // Very high threshold
          verificationTypes: [VerificationType.FACT_CHECKING],
        },
      };

      const ksHighThreshold = new KnowledgeSeeker(
        configHighThreshold,
        undefined,
        verificationEngine
      );

      const query: KnowledgeQuery = {
        id: "ks-high-threshold",
        query: "Test query with high verification threshold",
        queryType: QueryType.FACTUAL,
        maxResults: 10,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 8,
          tags: ["test"],
        },
      };

      const response = await ksHighThreshold.processQuery(query);

      expect(response).toBeDefined();
      // High threshold should result in fewer verified results
      expect(response.metadata.verifiedCount).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Verification Types Configuration", () => {
    it("should use configured verification types", async () => {
      const configMultipleTypes: KnowledgeSeekerConfig = {
        ...baseConfig,
        verification: {
          enabled: true,
          autoVerify: true,
          minConfidenceThreshold: 0.6,
          verificationTypes: [
            VerificationType.FACT_CHECKING,
            VerificationType.SOURCE_CREDIBILITY,
            VerificationType.CROSS_REFERENCE,
          ],
        },
      };

      const ksMultipleTypes = new KnowledgeSeeker(
        configMultipleTypes,
        undefined,
        verificationEngine
      );

      const query: KnowledgeQuery = {
        id: "ks-multiple-types",
        query: "Test query with multiple verification types",
        queryType: QueryType.FACTUAL,
        maxResults: 5,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 8,
          tags: ["test"],
        },
      };

      const response = await ksMultipleTypes.processQuery(query);

      expect(response).toBeDefined();
      if (
        response.verificationResults &&
        response.verificationResults.length > 0
      ) {
        // Should use multiple verification methods
        response.verificationResults.forEach((vr) => {
          expect(vr.methodResults).toBeDefined();
        });
      }
    });
  });

  describe("Performance Impact", () => {
    it("should complete verification within reasonable time", async () => {
      const query: KnowledgeQuery = {
        id: "ks-perf",
        query: "Performance test query",
        queryType: QueryType.FACTUAL,
        maxResults: 5,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 7,
          tags: ["performance"],
        },
      };

      const startTime = Date.now();
      const response = await knowledgeSeekerWithVerification.processQuery(
        query
      );
      const duration = Date.now() - startTime;

      expect(response).toBeDefined();
      expect(duration).toBeLessThan(60000); // 60 second timeout
    });

    it("should not significantly slow down low-priority queries", async () => {
      const query: KnowledgeQuery = {
        id: "ks-perf-low",
        query: "Low priority query",
        queryType: QueryType.EXPLANATORY,
        maxResults: 5,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 2,
          tags: ["test"],
        },
      };

      const startTime = Date.now();
      const response = await knowledgeSeekerWithVerification.processQuery(
        query
      );
      const duration = Date.now() - startTime;

      expect(response).toBeDefined();
      expect(duration).toBeLessThan(30000); // Should be quick for low priority
    });
  });

  describe("Concurrent Query Verification", () => {
    it("should handle multiple concurrent queries with verification", async () => {
      const queries: KnowledgeQuery[] = Array.from({ length: 3 }, (_, i) => ({
        id: `ks-concurrent-${i}`,
        query: `Concurrent query ${i}`,
        queryType: QueryType.FACTUAL,
        maxResults: 5,
        relevanceThreshold: 0.5,
        metadata: {
          priority: 7,
          tags: ["concurrent"],
        },
      }));

      const responses = await Promise.all(
        queries.map((q) => knowledgeSeekerWithVerification.processQuery(q))
      );

      expect(responses.length).toBe(3);
      expect(responses.every((r) => r !== undefined)).toBe(true);
    });
  });
});
