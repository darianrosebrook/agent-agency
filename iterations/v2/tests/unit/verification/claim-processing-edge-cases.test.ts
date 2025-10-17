/**
 * @fileoverview Comprehensive edge case tests for claim processing components.
 *
 * These tests target the 80% → 95% coverage goal for Claim Processing by covering:
 * - Complex ambiguity resolution scenarios
 * - Edge cases in claim extraction
 * - Error handling and recovery
 * - Performance under load
 * - CAWS compliance validation
 */

import type { VerificationRequest } from "../../../src/types/verification";
import {
  VerificationPriority,
  VerificationType,
  VerificationVerdict,
} from "../../../src/types/verification";
import { createClaimExtractor } from "../../../src/verification/ClaimExtractor";
import { FactChecker } from "../../../src/verification/FactChecker";
import { VerificationEngineImpl } from "../../../src/verification/VerificationEngine";
import type {
  ConversationContext,
  EvidenceManifest,
  ExtractedClaim,
} from "../../../src/verification/types";

describe("Claim Processing - Edge Cases and Advanced Scenarios", () => {
  let extractor: ReturnType<typeof createClaimExtractor>;
  let verificationEngine: VerificationEngineImpl;
  let factChecker: FactChecker;

  beforeEach(() => {
    extractor = createClaimExtractor();
    verificationEngine = new VerificationEngineImpl({
      methods: [
        {
          type: VerificationType.FACT_CHECKING,
          enabled: true,
          priority: 1,
          timeoutMs: 5000,
          config: {},
        },
        {
          type: VerificationType.CODE_VERIFICATION,
          enabled: true,
          priority: 1,
          timeoutMs: 5000,
          config: {},
        },
        {
          type: VerificationType.CONTEXT_VERIFICATION,
          enabled: true,
          priority: 1,
          timeoutMs: 5000,
          config: {},
        },
      ],
      defaultTimeoutMs: 5000,
      maxConcurrentVerifications: 10,
      minConfidenceThreshold: 0.5,
      maxEvidencePerMethod: 10,
      cacheEnabled: true,
      cacheTtlMs: 300000,
      retryAttempts: 3,
      retryDelayMs: 1000,
    });
    factChecker = new FactChecker([
      {
        type: VerificationType.FACT_CHECKING,
        enabled: true,
        priority: 1,
        timeoutMs: 5000,
        config: {},
      },
    ]);
  });

  describe("Complex Ambiguity Resolution", () => {
    it("should handle nested pronoun references", async () => {
      const context: ConversationContext = {
        conversationId: "conv-001",
        tenantId: "tenant-123",
        previousMessages: [
          "John mentioned that the system needs optimization",
          "He said it would improve performance significantly",
          "The database query was taking too long",
        ],
        metadata: { participants: ["John", "system"] },
      };

      const sentence =
        "He believes it can be fixed by indexing the primary key";

      const ambiguityAnalysis =
        await extractor.disambiguationStage.identifyAmbiguities(
          sentence,
          context
        );

      expect(ambiguityAnalysis.referentialAmbiguities).toHaveLength(2); // "He" and "it"
      expect(ambiguityAnalysis.canResolve).toBe(true);

      const resolution = await extractor.disambiguationStage.resolveAmbiguities(
        sentence,
        ambiguityAnalysis,
        context
      );

      expect(resolution.success).toBe(true);
      expect(resolution.disambiguatedSentence).toContain("John");
      expect(resolution.disambiguatedSentence).toContain("database query");
    });

    it("should detect and handle unresolvable ambiguities", async () => {
      const context: ConversationContext = {
        conversationId: "conv-002",
        tenantId: "tenant-456",
        previousMessages: ["The system is running"],
        metadata: {},
      };

      const sentence = "It needs to be fixed immediately";

      const unresolvable = await extractor.detectUnresolvableAmbiguities(
        sentence,
        context
      );

      expect(unresolvable).toHaveLength(1);
      expect(unresolvable[0].phrase).toBe("it");
      expect(unresolvable[0].reason).toBe("insufficient_context");
    });

    it("should handle multiple ambiguous entities in complex sentences", async () => {
      const context: ConversationContext = {
        conversationId: "conv-003",
        tenantId: "tenant-789",
        previousMessages: [
          "Alice and Bob discussed the API",
          "They agreed on the new authentication method",
          "The old system had security issues",
        ],
        metadata: { participants: ["Alice", "Bob"] },
      };

      const sentence = "They decided that it should use OAuth 2.0 instead";

      const analysis = await extractor.disambiguationStage.identifyAmbiguities(
        sentence,
        context
      );

      expect(analysis.referentialAmbiguities).toHaveLength(2); // "They" and "it"
      expect(analysis.canResolve).toBe(true);
    });
  });

  describe("Verifiable Content Qualification", () => {
    it("should identify and rewrite subjective statements", async () => {
      const context: ConversationContext = {
        conversationId: "conv-004",
        tenantId: "tenant-101",
        previousMessages: [],
        metadata: {},
      };

      const sentence = "This is probably the best solution we have";

      const result = await extractor.qualificationStage.detectVerifiableContent(
        sentence,
        context
      );

      expect(result.hasVerifiableContent).toBe(false);
      expect(result.indicators).toContain("probably");
    });

    it("should preserve factual claims while removing opinions", async () => {
      const context: ConversationContext = {
        conversationId: "conv-005",
        tenantId: "tenant-102",
        previousMessages: [],
        metadata: {},
      };

      const sentence = "The API returns a 200 status code, which is great";

      const result = await extractor.qualificationStage.detectVerifiableContent(
        sentence,
        context
      );

      expect(result.hasVerifiableContent).toBe(true);
      expect(result.rewrittenSentence).toContain("200 status code");
      expect(result.rewrittenSentence).not.toContain("which is great");
    });

    it("should handle mixed verifiable and unverifiable content", async () => {
      const context: ConversationContext = {
        conversationId: "conv-006",
        tenantId: "tenant-103",
        previousMessages: [],
        metadata: {},
      };

      const sentence =
        "The system processes 1000 requests per second, which is impressive";

      const result = await extractor.qualificationStage.detectVerifiableContent(
        sentence,
        context
      );

      expect(result.hasVerifiableContent).toBe(true);
      expect(result.indicators).toContain("1000 requests per second");
      expect(result.indicators).toContain("impressive");
    });
  });

  describe("Atomic Claim Decomposition", () => {
    it("should decompose complex compound sentences", async () => {
      const context: ConversationContext = {
        conversationId: "conv-007",
        tenantId: "tenant-104",
        previousMessages: [],
        metadata: {},
      };

      const sentence =
        "The database uses PostgreSQL version 14.5 and supports transactions";

      const claims = await extractor.decompositionStage.extractAtomicClaims(
        sentence,
        context
      );

      expect(claims).toHaveLength(2);
      expect(claims[0].statement).toContain("PostgreSQL version 14.5");
      expect(claims[1].statement).toContain("supports transactions");
    });

    it("should add contextual brackets for implied context", async () => {
      const context: ConversationContext = {
        conversationId: "conv-008",
        tenantId: "tenant-105",
        previousMessages: ["We're discussing the authentication system"],
        metadata: {},
      };

      const claim = "uses JWT tokens";

      const contextualized =
        await extractor.decompositionStage.addContextualBrackets(
          claim,
          "authentication system"
        );

      expect(contextualized).toContain("[authentication system]");
      expect(contextualized).toContain("JWT tokens");
    });

    it("should handle claims with temporal references", async () => {
      const context: ConversationContext = {
        conversationId: "conv-009",
        tenantId: "tenant-106",
        previousMessages: [],
        metadata: {},
      };

      const sentence =
        "The system was updated yesterday and now supports OAuth 2.0";

      const claims = await extractor.decompositionStage.extractAtomicClaims(
        sentence,
        context
      );

      expect(claims).toHaveLength(2);
      expect(claims[0].statement).toContain("updated yesterday");
      expect(claims[1].statement).toContain("OAuth 2.0");
    });
  });

  describe("Error Handling and Recovery", () => {
    it("should handle malformed input gracefully", async () => {
      const context: ConversationContext = {
        conversationId: "conv-010",
        tenantId: "tenant-107",
        previousMessages: [],
        metadata: {},
      };

      const malformedInput = "   \n\t   \n   ";

      const result = await extractor.qualificationStage.detectVerifiableContent(
        malformedInput,
        context
      );

      expect(result.hasVerifiableContent).toBe(false);
      expect(result.rewrittenSentence).toBeNull();
    });

    it("should handle extremely long sentences", async () => {
      const context: ConversationContext = {
        conversationId: "conv-011",
        tenantId: "tenant-108",
        previousMessages: [],
        metadata: {},
      };

      const longSentence =
        "The system that we have been developing for the past six months using React, TypeScript, Node.js, PostgreSQL, Redis, Docker, Kubernetes, and various other technologies has finally reached a stable state where it can handle concurrent users, process real-time data, integrate with third-party APIs, and provide comprehensive analytics while maintaining security, performance, and scalability requirements";

      const claims = await extractor.decompositionStage.extractAtomicClaims(
        longSentence,
        context
      );

      expect(claims.length).toBeGreaterThan(0);
      expect(claims[0].statement).toContain("React");
      expect(claims[0].statement).toContain("TypeScript");
    });

    it("should handle Unicode and special characters", async () => {
      const context: ConversationContext = {
        conversationId: "conv-012",
        tenantId: "tenant-109",
        previousMessages: [],
        metadata: {},
      };

      const unicodeSentence =
        "The API supports UTF-8 encoding and handles émojis 🚀 correctly";

      const claims = await extractor.decompositionStage.extractAtomicClaims(
        unicodeSentence,
        context
      );

      expect(claims).toHaveLength(2);
      expect(claims[0].statement).toContain("UTF-8");
      expect(claims[1].statement).toContain("émojis");
    });
  });

  describe("Performance Under Load", () => {
    it("should process multiple claims concurrently", async () => {
      const context: ConversationContext = {
        conversationId: "conv-013",
        tenantId: "tenant-110",
        previousMessages: [],
        metadata: {},
      };

      const sentences = [
        "The API uses REST protocol",
        "Authentication is handled by JWT tokens",
        "The database supports ACID transactions",
        "Caching is implemented with Redis",
        "The system scales horizontally",
      ];

      const startTime = Date.now();
      const promises = sentences.map((sentence) =>
        extractor.decompositionStage.extractAtomicClaims(sentence, context)
      );

      const results = await Promise.all(promises);
      const duration = Date.now() - startTime;

      expect(results).toHaveLength(5);
      expect(duration).toBeLessThan(2000); // Should complete within 2 seconds
      results.forEach((claims) => {
        expect(claims.length).toBeGreaterThan(0);
      });
    });

    it("should handle large batch processing", async () => {
      const context: ConversationContext = {
        conversationId: "conv-014",
        tenantId: "tenant-111",
        previousMessages: [],
        metadata: {},
      };

      const largeBatch = Array.from(
        { length: 100 },
        (_, i) =>
          `Claim ${i}: The system processes ${i * 10} requests per second`
      );

      const startTime = Date.now();
      const results = await Promise.all(
        largeBatch.map((sentence) =>
          extractor.qualificationStage.detectVerifiableContent(
            sentence,
            context
          )
        )
      );
      const duration = Date.now() - startTime;

      expect(results).toHaveLength(100);
      expect(duration).toBeLessThan(5000); // Should complete within 5 seconds
      expect(results.every((r) => r.hasVerifiableContent)).toBe(true);
    });
  });

  describe("CAWS Compliance Validation", () => {
    it("should validate claim scope against working spec", async () => {
      const workingSpec = {
        id: "FEAT-001",
        title: "API Authentication Implementation",
        scope: {
          in: ["src/auth/", "src/api/"],
          out: ["src/ui/", "tests/"],
        },
        changeBudget: {
          maxFiles: 10,
          maxLoc: 500,
        },
      };

      const claim: ExtractedClaim = {
        id: "claim-001",
        statement: "The authentication module validates JWT tokens",
        confidence: 0.9,
        sourceContext: "src/auth/jwt-validator.ts",
        verificationRequirements: [],
      };

      const evidence: EvidenceManifest = {
        sources: [
          {
            name: "jwt-validator.ts",
            type: "source_code",
            reliability: 0.9,
            responseTime: 0,
          },
        ],
        evidence: [
          {
            content: "JWT validation logic",
            source: "jwt-validator.ts",
            strength: 0.9,
            timestamp: new Date().toISOString(),
            metadata: {},
          },
        ],
        quality: 0.8,
        cawsCompliant: true,
      };

      const validation = await extractor.verificationStage.validateClaimScope(
        claim,
        workingSpec as any
      );

      expect(validation.withinScope).toBe(true);
      expect(validation.violations).toHaveLength(0);
    });

    it("should detect scope violations", async () => {
      const workingSpec = {
        id: "FEAT-002",
        title: "Database Schema Update",
        scope: {
          in: ["migrations/", "src/db/"],
          out: ["src/ui/", "src/api/"],
        },
        changeBudget: {
          maxFiles: 5,
          maxLoc: 200,
        },
      };

      const claim: ExtractedClaim = {
        id: "claim-002",
        statement: "The UI component displays user data",
        confidence: 0.9,
        sourceContext: "src/ui/user-display.tsx",
        verificationRequirements: [],
      };

      const validation = await extractor.verificationStage.validateClaimScope(
        claim,
        workingSpec as any
      );

      expect(validation.withinScope).toBe(false);
      expect(validation.violations).toContain("scope_violation");
    });

    it("should validate evidence quality requirements", async () => {
      const claim: ExtractedClaim = {
        id: "claim-003",
        statement: "The API returns JSON responses",
        confidence: 0.9,
        sourceContext: "src/api/response-handler.ts",
        verificationRequirements: [],
      };

      const evidence: EvidenceManifest = {
        sources: [
          {
            name: "response-handler.ts",
            type: "source_code",
            reliability: 0.9,
            responseTime: 0,
          },
        ],
        evidence: [
          {
            content: "JSON.stringify() usage",
            source: "response-handler.ts",
            strength: 0.8,
            timestamp: new Date().toISOString(),
            metadata: {},
          },
        ],
        quality: 0.7,
        cawsCompliant: true,
      };

      const result = await extractor.verificationStage.verifyClaimEvidence(
        claim,
        evidence
      );

      expect(result.status).toBe("VERIFIED");
      expect(result.evidenceQuality).toBeGreaterThan(0.8);
    });
  });

  describe("Integration with Verification Engine", () => {
    it("should process verification requests end-to-end", async () => {
      const request: VerificationRequest = {
        id: "req-001",
        content: "The system uses PostgreSQL database with ACID compliance",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [
          VerificationType.FACT_CHECKING,
          VerificationType.CONTEXT_VERIFICATION,
        ],
        claims: [],
        metadata: {},
      };

      const result = await verificationEngine.verify(request);

      expect(result.requestId).toBe("req-001");
      expect(result.verdict).toBeDefined();
      expect(result.confidence).toBeGreaterThan(0);
      expect(result.methodResults).toHaveLength(2);
    });

    it("should handle verification timeouts gracefully", async () => {
      const request: VerificationRequest = {
        id: "req-002",
        content: "This is a very complex claim that might timeout",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.FACT_CHECKING],
        claims: [],
        metadata: {},
      };

      // Mock a slow verification
      const slowVerificationEngine = new VerificationEngineImpl({
        methods: [
          {
            type: VerificationType.FACT_CHECKING,
            enabled: true,
            priority: 1,
            timeoutMs: 100,
            config: {},
          },
        ],
        defaultTimeoutMs: 100, // Very short timeout
        maxConcurrentVerifications: 1,
        minConfidenceThreshold: 0.5,
        maxEvidencePerMethod: 10,
        cacheEnabled: false,
        cacheTtlMs: 0,
        retryAttempts: 1,
        retryDelayMs: 100,
      });

      const result = await slowVerificationEngine.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.ERROR);
      expect(result.error).toBeDefined();
    });

    it("should cache verification results", async () => {
      const request: VerificationRequest = {
        id: "req-003",
        content: "The API supports HTTP/2 protocol",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.FACT_CHECKING],
        claims: [],
        metadata: {},
      };

      // First verification
      const result1 = await verificationEngine.verify(request);

      // Second verification should use cache
      const result2 = await verificationEngine.verify(request);

      expect(result1.requestId).toBe(result2.requestId);
      expect(result1.verdict).toBe(result2.verdict);
      // Cache hit should be faster
      expect(result2.processingTimeMs).toBeLessThan(result1.processingTimeMs);
    });
  });

  describe("Fact Checker Edge Cases", () => {
    it("should handle claims with no available sources", async () => {
      // Test the fact checker through the verification engine
      const request: VerificationRequest = {
        id: "fact-001",
        content:
          "This is a very specific technical claim with no public sources",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.FACT_CHECKING],
        claims: [],
        metadata: {},
      };

      const result = await verificationEngine.verify(request);

      expect(result.requestId).toBe("fact-001");
      expect(result.verdict).toBe(VerificationVerdict.UNVERIFIED);
      expect(result.confidence).toBeLessThan(0.5);
    });

    it("should aggregate results from multiple sources", async () => {
      const request: VerificationRequest = {
        id: "fact-002",
        content: "JavaScript is a programming language",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.FACT_CHECKING],
        claims: [],
        metadata: {},
      };

      const result = await verificationEngine.verify(request);

      expect(result.requestId).toBe("fact-002");
      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
      expect(result.confidence).toBeGreaterThan(0.8);
      expect(result.supportingEvidence.length).toBeGreaterThan(0);
    });

    it("should handle conflicting source results", async () => {
      const request: VerificationRequest = {
        id: "fact-003",
        content: "This claim has conflicting information across sources",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.FACT_CHECKING],
        claims: [],
        metadata: {},
      };

      const result = await verificationEngine.verify(request);

      expect(result.requestId).toBe("fact-003");
      expect(result.verdict).toBe(VerificationVerdict.CONTRADICTORY);
      expect(result.confidence).toBeLessThan(0.7);
    });
  });

  describe("Learning System Integration", () => {
    it("should learn from verification patterns", async () => {
      // Test pattern learning through verification requests
      const requests = [
        {
          id: "pattern-001",
          content: "The system uses PostgreSQL",
          priority: VerificationPriority.MEDIUM,
          verificationTypes: [VerificationType.FACT_CHECKING],
          claims: [],
          metadata: {},
        },
        {
          id: "pattern-002",
          content: "The system uses Redis",
          priority: VerificationPriority.MEDIUM,
          verificationTypes: [VerificationType.FACT_CHECKING],
          claims: [],
          metadata: {},
        },
        {
          id: "pattern-003",
          content: "The system uses Docker",
          priority: VerificationPriority.MEDIUM,
          verificationTypes: [VerificationType.FACT_CHECKING],
          claims: [],
          metadata: {},
        },
      ];

      const results = await Promise.all(
        requests.map((request) => verificationEngine.verify(request))
      );

      // All should be verified successfully
      expect(
        results.every((r) => r.verdict === VerificationVerdict.VERIFIED_TRUE)
      ).toBe(true);
      expect(results.length).toBe(3);
    });

    it("should adapt to new claim types", async () => {
      const request: VerificationRequest = {
        id: "new-claim-type",
        content:
          "The microservice architecture uses event-driven communication",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.FACT_CHECKING],
        claims: [],
        metadata: {},
      };

      const result = await verificationEngine.verify(request);

      expect(result.requestId).toBe("new-claim-type");
      expect(result.verdict).toBeDefined();
      expect(result.confidence).toBeGreaterThan(0);
    });
  });
});
