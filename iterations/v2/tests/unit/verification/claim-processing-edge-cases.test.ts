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

import { createClaimExtractor } from "../../../src/verification/ClaimExtractor";
import { VerificationEngineImpl } from "../../../src/verification/VerificationEngine";
import { FactChecker } from "../../../src/verification/FactChecker";
import type {
  ConversationContext,
  EvidenceManifest,
  AmbiguityAnalysis,
  DisambiguationResult,
  VerifiableContentResult,
  AtomicClaim,
  ExtractedClaim,
} from "../../../src/verification/types";
import type {
  VerificationRequest,
  VerificationPriority,
  VerificationType,
  VerificationResult,
  VerificationVerdict,
} from "../../../src/types/verification";

describe("Claim Processing - Edge Cases and Advanced Scenarios", () => {
  let extractor: ReturnType<typeof createClaimExtractor>;
  let verificationEngine: VerificationEngineImpl;
  let factChecker: FactChecker;

  beforeEach(() => {
    extractor = createClaimExtractor();
    verificationEngine = new VerificationEngineImpl({
      methods: [
        { type: VerificationType.FACT_CHECKING, enabled: true },
        { type: VerificationType.CODE_VERIFICATION, enabled: true },
        { type: VerificationType.CONTEXT_VERIFICATION, enabled: true },
      ],
      timeoutMs: 5000,
      maxConcurrentRequests: 10,
      cacheEnabled: true,
      cacheTtlMs: 300000,
    });
    factChecker = new FactChecker([
      { type: VerificationType.FACT_CHECKING, enabled: true },
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

      const sentence = "He believes it can be fixed by indexing the primary key";
      
      const ambiguityAnalysis = await extractor.disambiguationStage.identifyAmbiguities(
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
      expect(unresolvable[0].token).toBe("it");
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

      const sentence = "The system processes 1000 requests per second, which is impressive";
      
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

      const sentence = "The database uses PostgreSQL version 14.5 and supports transactions";
      
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
      
      const contextualized = await extractor.decompositionStage.addContextualBrackets(
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

      const sentence = "The system was updated yesterday and now supports OAuth 2.0";
      
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

      const longSentence = "The system that we have been developing for the past six months using React, TypeScript, Node.js, PostgreSQL, Redis, Docker, Kubernetes, and various other technologies has finally reached a stable state where it can handle concurrent users, process real-time data, integrate with third-party APIs, and provide comprehensive analytics while maintaining security, performance, and scalability requirements";
      
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

      const unicodeSentence = "The API supports UTF-8 encoding and handles émojis 🚀 correctly";
      
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
      const promises = sentences.map(sentence => 
        extractor.decompositionStage.extractAtomicClaims(sentence, context)
      );
      
      const results = await Promise.all(promises);
      const duration = Date.now() - startTime;

      expect(results).toHaveLength(5);
      expect(duration).toBeLessThan(2000); // Should complete within 2 seconds
      results.forEach(claims => {
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

      const largeBatch = Array.from({ length: 100 }, (_, i) => 
        `Claim ${i}: The system processes ${i * 10} requests per second`
      );

      const startTime = Date.now();
      const results = await Promise.all(
        largeBatch.map(sentence =>
          extractor.qualificationStage.detectVerifiableContent(sentence, context)
        )
      );
      const duration = Date.now() - startTime;

      expect(results).toHaveLength(100);
      expect(duration).toBeLessThan(5000); // Should complete within 5 seconds
      expect(results.every(r => r.hasVerifiableContent)).toBe(true);
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
        sources: [{
          name: "jwt-validator.ts",
          type: "source_code",
          reliability: 0.9,
          responseTime: 0,
        }],
        evidence: [{
          content: "JWT validation logic",
          source: "jwt-validator.ts",
          relevance: 0.9,
          credibility: 0.8,
          supporting: true,
          verificationDate: new Date(),
        }],
        quality: 0.8,
        cawsCompliant: true,
      };

      const validation = await extractor.verificationStage.validateClaimScope(
        claim,
        workingSpec as any
      );

      expect(validation.isValid).toBe(true);
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
        text: "The UI component displays user data",
        source: "src/ui/user-display.tsx",
        confidence: 0.9,
        type: "implementation",
        evidence: [],
        metadata: {},
      };

      const validation = await extractor.verificationStage.validateClaimScope(
        claim,
        workingSpec as any
      );

      expect(validation.isValid).toBe(false);
      expect(validation.violations).toContain("scope_violation");
    });

    it("should validate evidence quality requirements", async () => {
      const claim: ExtractedClaim = {
        id: "claim-003",
        text: "The API returns JSON responses",
        source: "src/api/response-handler.ts",
        confidence: 0.9,
        type: "implementation",
        evidence: [],
        metadata: {},
      };

      const evidence: EvidenceManifest = {
        sources: ["src/api/response-handler.ts"],
        evidence: ["JSON.stringify() usage"],
        quality: 0.7,
      };

      const result = await extractor.verificationStage.verifyClaimEvidence(
        claim,
        evidence
      );

      expect(result.verdict).toBe("verified");
      expect(result.confidence).toBeGreaterThan(0.8);
    });
  });

  describe("Integration with Verification Engine", () => {
    it("should process verification requests end-to-end", async () => {
      const request: VerificationRequest = {
        id: "req-001",
        content: "The system uses PostgreSQL database with ACID compliance",
        verificationTypes: ["fact_check", "context_verification"],
        claims: [],
        metadata: {},
      };

      const result = await verificationEngine.verify(request);

      expect(result.id).toBe("req-001");
      expect(result.verdict).toBeDefined();
      expect(result.confidence).toBeGreaterThan(0);
      expect(result.methodResults).toHaveLength(2);
    });

    it("should handle verification timeouts gracefully", async () => {
      const request: VerificationRequest = {
        id: "req-002",
        content: "This is a very complex claim that might timeout",
        verificationTypes: ["fact_check"],
        claims: [],
        metadata: {},
      };

      // Mock a slow verification
      const slowVerificationEngine = new VerificationEngineImpl({
        methods: ["fact_check"],
        timeoutMs: 100, // Very short timeout
        maxConcurrentRequests: 1,
        cacheEnabled: false,
      });

      const result = await slowVerificationEngine.verify(request);

      expect(result.verdict).toBe("timeout");
      expect(result.error).toBeDefined();
    });

    it("should cache verification results", async () => {
      const request: VerificationRequest = {
        id: "req-003",
        content: "The API supports HTTP/2 protocol",
        verificationTypes: ["fact_check"],
        claims: [],
        metadata: {},
      };

      // First verification
      const result1 = await verificationEngine.verify(request);
      
      // Second verification should use cache
      const result2 = await verificationEngine.verify(request);

      expect(result1.id).toBe(result2.id);
      expect(result1.verdict).toBe(result2.verdict);
      // Cache hit should be faster
      expect(result2.metadata?.cacheHit).toBe(true);
    });
  });

  describe("Fact Checker Edge Cases", () => {
    it("should handle claims with no available sources", async () => {
      const claim = {
        id: "fact-001",
        text: "This is a very specific technical claim with no public sources",
        category: "technical",
        confidence: 0.5,
        metadata: {},
      };

      const result = await factChecker.checkClaim(claim);

      expect(result.claimId).toBe("fact-001");
      expect(result.verdict).toBe("unverified");
      expect(result.confidence).toBeLessThan(0.5);
    });

    it("should aggregate results from multiple sources", async () => {
      const claim = {
        id: "fact-002",
        text: "JavaScript is a programming language",
        category: "general",
        confidence: 0.9,
        metadata: {},
      };

      const result = await factChecker.checkClaim(claim);

      expect(result.claimId).toBe("fact-002");
      expect(result.verdict).toBe("verified");
      expect(result.confidence).toBeGreaterThan(0.8);
      expect(result.sources).toHaveLength(2); // Google + Snopes
    });

    it("should handle conflicting source results", async () => {
      const claim = {
        id: "fact-003",
        text: "This claim has conflicting information across sources",
        category: "general",
        confidence: 0.7,
        metadata: {},
      };

      const result = await factChecker.checkClaim(claim);

      expect(result.claimId).toBe("fact-003");
      expect(result.verdict).toBe("disputed");
      expect(result.confidence).toBeLessThan(0.7);
    });
  });

  describe("Learning System Integration", () => {
    it("should learn from verification patterns", async () => {
      const patterns = [
        {
          pattern: "The system uses {technology}",
          examples: [
            "The system uses PostgreSQL",
            "The system uses Redis",
            "The system uses Docker",
          ],
          verificationResults: ["verified", "verified", "verified"],
        },
      ];

      const update = await extractor.updatePatterns(patterns);

      expect(update.success).toBe(true);
      expect(update.updatedPatterns).toBeGreaterThan(0);
    });

    it("should adapt to new claim types", async () => {
      const learningUpdate: any = {
        claimType: "new_technical_claim",
        examples: [
          "The microservice architecture uses event-driven communication",
        ],
        verificationResults: ["verified"],
        metadata: {},
      };

      const result = await extractor.updateLearning(learningUpdate);

      expect(result.success).toBe(true);
      expect(result.adaptationScore).toBeGreaterThan(0);
    });
  });
});
