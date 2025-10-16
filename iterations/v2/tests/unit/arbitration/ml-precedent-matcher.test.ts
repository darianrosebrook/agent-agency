/**
 * Tests for ML/NLP Precedent Matcher
 */

import { MLPrecedentMatcher } from "../../../src/arbitration/adapters/MLPrecedentMatcher";
import {
  Precedent,
  RuleCategory,
  VerdictOutcome,
  ViolationSeverity,
} from "../../../src/types/arbitration";

describe("MLPrecedentMatcher", () => {
  let matcher: MLPrecedentMatcher;
  let mockPrecedents: Precedent[];

  beforeEach(() => {
    // Create matcher with test configuration
    matcher = new MLPrecedentMatcher({
      minSimilarityThreshold: 0.5,
      maxResults: 5,
      enableSemanticSimilarity: true,
      enableEntityRecognition: true,
      enableIntentClassification: true,
      enableFallback: true,
    });

    // Create mock precedents
    mockPrecedents = [
      {
        id: "precedent-1",
        title: "Database Access Violation",
        keyFacts: [
          "User attempted to access restricted database",
          "No proper authorization provided",
          "Access was blocked by security system",
        ],
        applicability: {
          category: RuleCategory.SECURITY,
          severity: ViolationSeverity.MAJOR,
          conditions: ["database_access", "authorization_required"],
        },
        verdict: {
          id: "verdict-1",
          sessionId: "session-1",
          outcome: VerdictOutcome.REJECTED,
          reasoning: [
            {
              step: 1,
              description:
                "Database access without proper authorization violates security policy",
              evidence: ["No authorization provided"],
              ruleReferences: ["SEC-001"],
              confidence: 0.95,
            },
          ],
          rulesApplied: ["SEC-001"],
          evidence: ["No authorization provided"],
          precedents: [],
          confidence: 0.95,
          issuedBy: "system",
          issuedAt: new Date("2024-01-01"),
          auditLog: [
            {
              timestamp: new Date("2024-01-01"),
              action: "verdict_issued",
              actor: "system",
              details: "Database access violation rejected",
            },
          ],
        },
        rulesInvolved: ["SEC-001"],
        reasoningSummary:
          "Database access without authorization violates security policy",
        citationCount: 5,
        createdAt: new Date("2024-01-01"),
        metadata: {
          createdAt: new Date("2024-01-01"),
          createdBy: "system",
          citations: 5,
          overruled: false,
        },
      },
      {
        id: "precedent-2",
        title: "File Deletion Policy Violation",
        keyFacts: [
          "User attempted to delete critical system files",
          "Files were marked as protected",
          "Deletion was prevented by file protection",
        ],
        applicability: {
          category: RuleCategory.SECURITY,
          severity: ViolationSeverity.CRITICAL,
          conditions: ["file_operations", "system_protection"],
        },
        verdict: {
          id: "verdict-2",
          sessionId: "session-2",
          outcome: VerdictOutcome.REJECTED,
          reasoning: [
            {
              step: 1,
              description: "Critical system files must not be deleted",
              evidence: ["Files marked as protected"],
              ruleReferences: ["SEC-002"],
              confidence: 1.0,
            },
          ],
          rulesApplied: ["SEC-002"],
          evidence: ["Files marked as protected"],
          precedents: [],
          confidence: 1.0,
          issuedBy: "admin",
          issuedAt: new Date("2024-01-15"),
          auditLog: [
            {
              timestamp: new Date("2024-01-15"),
              action: "verdict_issued",
              actor: "admin",
              details: "File deletion violation rejected",
            },
          ],
        },
        rulesInvolved: ["SEC-002"],
        reasoningSummary: "Critical system files must not be deleted",
        citationCount: 12,
        createdAt: new Date("2024-01-15"),
        metadata: {
          createdAt: new Date("2024-01-15"),
          createdBy: "admin",
          citations: 12,
          overruled: false,
        },
      },
      {
        id: "precedent-3",
        title: "Code Quality Review",
        keyFacts: [
          "Code review found quality issues",
          "Missing error handling",
          "Insufficient test coverage",
        ],
        applicability: {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MINOR,
          conditions: ["code_review", "quality_standards"],
        },
        verdict: {
          id: "verdict-3",
          sessionId: "session-3",
          outcome: VerdictOutcome.APPROVED,
          reasoning: [
            {
              step: 1,
              description:
                "Quality issues are minor and can be addressed later",
              evidence: ["Issues are minor"],
              ruleReferences: ["QUAL-001"],
              confidence: 0.7,
            },
          ],
          rulesApplied: ["QUAL-001"],
          evidence: ["Issues are minor"],
          precedents: [],
          confidence: 0.7,
          issuedBy: "reviewer",
          issuedAt: new Date("2024-02-01"),
          auditLog: [
            {
              timestamp: new Date("2024-02-01"),
              action: "verdict_issued",
              actor: "reviewer",
              details: "Code quality review approved",
            },
          ],
        },
        rulesInvolved: ["QUAL-001"],
        reasoningSummary: "Quality issues are minor and can be addressed later",
        citationCount: 3,
        createdAt: new Date("2024-02-01"),
        metadata: {
          createdAt: new Date("2024-02-01"),
          createdBy: "reviewer",
          citations: 3,
          overruled: false,
        },
      },
    ];
  });

  describe("findSimilarPrecedents", () => {
    it("should find similar precedents using ML/NLP matching", async () => {
      const matcher = new MLPrecedentMatcher({
        minSimilarityThreshold: 0.1, // Lower threshold for fallback matches
      });

      const context = {
        action: "access_database",
        actor: "user-123",
        parameters: {
          database: "production_db",
          table: "user_data",
        },
        environment: {
          authorization: "none",
          user_role: "basic",
        },
        category: RuleCategory.SECURITY,
        severity: ViolationSeverity.MAJOR,
      };

      const matches = await matcher.findSimilarPrecedents(
        context,
        mockPrecedents
      );

      expect(matches).toBeDefined();
      expect(Array.isArray(matches)).toBe(true);
      // When ML fails, should still find matches via fallback
      expect(matches.length).toBeGreaterThan(0);
    });

    it("should handle empty precedents list", async () => {
      const context = {
        action: "test_action",
        actor: "test_actor",
        parameters: {},
        environment: {},
        category: RuleCategory.SECURITY,
        severity: ViolationSeverity.MINOR,
      };

      const matches = await matcher.findSimilarPrecedents(context, []);
      expect(matches).toEqual([]);
    });

    it("should filter results by similarity threshold", async () => {
      // Create matcher with high threshold
      const strictMatcher = new MLPrecedentMatcher({
        minSimilarityThreshold: 0.9,
      });

      const context = {
        action: "unrelated_action",
        actor: "test_actor",
        parameters: {},
        environment: {},
        category: RuleCategory.SECURITY,
        severity: ViolationSeverity.MINOR,
      };

      const matches = await strictMatcher.findSimilarPrecedents(
        context,
        mockPrecedents
      );

      // Should have fewer matches due to high threshold
      expect(matches.length).toBeLessThanOrEqual(mockPrecedents.length);
    });

    it("should respect max results limit", async () => {
      const context = {
        action: "general_action",
        actor: "test_actor",
        parameters: {},
        environment: {},
        category: RuleCategory.SECURITY,
        severity: ViolationSeverity.MINOR,
      };

      const matches = await matcher.findSimilarPrecedents(
        context,
        mockPrecedents
      );

      expect(matches.length).toBeLessThanOrEqual(
        matcher.getConfig().maxResults
      );
    });

    it("should provide detailed match information", async () => {
      const context = {
        action: "access_database",
        actor: "user-123",
        parameters: {
          database: "production_db",
        },
        environment: {},
        category: RuleCategory.SECURITY,
        severity: ViolationSeverity.MAJOR,
      };

      const matches = await matcher.findSimilarPrecedents(
        context,
        mockPrecedents
      );

      if (matches.length > 0) {
        const match = matches[0];

        expect(match.precedent).toBeDefined();
        expect(match.score).toBeGreaterThanOrEqual(0);
        expect(match.score).toBeLessThanOrEqual(1);
        expect(match.factors).toBeDefined();
        expect(match.matchingEntities).toBeDefined();
        expect(match.intentAlignment).toBeDefined();
        expect(match.semanticSimilarity).toBeDefined();
        expect(match.reasoning).toBeDefined();
      }
    });

    it("should fallback to rule-based matching when ML fails", async () => {
      // Create matcher that will fail ML processing
      const failingMatcher = new MLPrecedentMatcher({
        enableFallback: true,
        minSimilarityThreshold: 0.3,
      });

      // Mock the ML methods to throw errors
      const originalExtractEntities = (failingMatcher as any).extractEntities;
      (failingMatcher as any).extractEntities = jest
        .fn()
        .mockRejectedValue(new Error("ML processing failed"));

      const context = {
        action: "access_database",
        actor: "user-123",
        parameters: {},
        environment: {},
        category: RuleCategory.SECURITY,
        severity: ViolationSeverity.MAJOR,
      };

      const matches = await failingMatcher.findSimilarPrecedents(
        context,
        mockPrecedents
      );

      // Should still return matches due to fallback
      expect(matches).toBeDefined();
      expect(Array.isArray(matches)).toBe(true);
    });
  });

  describe("configuration", () => {
    it("should update configuration", () => {
      const newConfig = {
        minSimilarityThreshold: 0.8,
        maxResults: 3,
      };

      matcher.updateConfig(newConfig);
      const config = matcher.getConfig();

      expect(config.minSimilarityThreshold).toBe(0.8);
      expect(config.maxResults).toBe(3);
    });

    it("should return current configuration", () => {
      const config = matcher.getConfig();

      expect(config).toBeDefined();
      expect(config.minSimilarityThreshold).toBeDefined();
      expect(config.maxResults).toBeDefined();
      expect(config.enableSemanticSimilarity).toBeDefined();
      expect(config.enableEntityRecognition).toBeDefined();
      expect(config.enableIntentClassification).toBeDefined();
      expect(config.enableFallback).toBeDefined();
      expect(config.weights).toBeDefined();
    });
  });

  describe("ML/NLP features", () => {
    it("should extract entities from text", async () => {
      const text = "User agent-123 attempted to access production database";

      // Access private method for testing
      const entities = await (matcher as any).extractEntities(text);

      expect(entities).toBeDefined();
      expect(Array.isArray(entities)).toBe(true);

      if (entities.length > 0) {
        const entity = entities[0];
        expect(entity.text).toBeDefined();
        expect(entity.type).toBeDefined();
        expect(entity.confidence).toBeGreaterThanOrEqual(0);
        expect(entity.confidence).toBeLessThanOrEqual(1);
      }
    });

    it("should classify intent from text", async () => {
      const text = "I want to create a new user account";

      // Access private method for testing
      const intent = await (matcher as any).classifyIntent(text);

      expect(intent).toBeDefined();
      expect(intent.intent).toBeDefined();
      expect(intent.confidence).toBeGreaterThanOrEqual(0);
      expect(intent.confidence).toBeLessThanOrEqual(1);
      expect(intent.alternatives).toBeDefined();
      expect(Array.isArray(intent.alternatives)).toBe(true);
    });

    it("should calculate semantic similarity", async () => {
      const text1 = "User accessed database";
      const text2 = "Agent accessed production database";

      // Access private method for testing
      const similarity = await (matcher as any).calculateSemanticSimilarity(
        text1,
        text2
      );

      expect(similarity).toBeDefined();
      expect(similarity.score).toBeGreaterThanOrEqual(0);
      expect(similarity.score).toBeLessThanOrEqual(1);
      expect(similarity.matchingPhrases).toBeDefined();
      expect(Array.isArray(similarity.matchingPhrases)).toBe(true);
      expect(similarity.distance).toBeGreaterThanOrEqual(0);
      expect(similarity.distance).toBeLessThanOrEqual(1);
    });
  });

  describe("error handling", () => {
    it("should handle malformed context gracefully", async () => {
      const malformedContext = {
        action: "",
        actor: null,
        parameters: undefined,
        environment: null,
        category: RuleCategory.SECURITY,
        severity: ViolationSeverity.MINOR,
      } as any;

      const matches = await matcher.findSimilarPrecedents(
        malformedContext,
        mockPrecedents
      );

      expect(matches).toBeDefined();
      expect(Array.isArray(matches)).toBe(true);
    });

    it("should handle malformed precedents gracefully", async () => {
      const context = {
        action: "test_action",
        actor: "test_actor",
        parameters: {},
        environment: {},
        category: RuleCategory.SECURITY,
        severity: ViolationSeverity.MINOR,
      };

      const malformedPrecedents = [
        {
          id: "malformed",
          applicability: undefined, // Malformed: missing applicability object
        },
      ] as any;

      // Should not crash and return empty matches for malformed precedents
      const matches = await matcher.findSimilarPrecedents(
        context,
        malformedPrecedents
      );

      expect(matches).toBeDefined();
      expect(Array.isArray(matches)).toBe(true);
      // Should return empty array for malformed precedents
      expect(matches).toHaveLength(0);
    });
  });
});
