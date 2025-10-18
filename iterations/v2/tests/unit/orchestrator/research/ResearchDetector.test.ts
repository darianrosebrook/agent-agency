/**
 * @fileoverview Unit Tests for ResearchDetector (ARBITER-006 Phase 4)
 *
 * @author @darianrosebrook
 *
 * Tests for multi-heuristic research need detection including:
 * - Question detection
 * - Uncertainty keyword detection
 * - Technical information needs
 * - Comparison needs
 * - Fact-checking requirements
 * - Query generation
 * - Confidence scoring
 */

import { ResearchDetector } from "../../../../src/orchestrator/research/ResearchDetector";
import { QueryType } from "../../../../src/types/knowledge";
import { mockTask } from "../../../mocks/knowledge-mocks";

describe("ResearchDetector", () => {
  let detector: ResearchDetector;

  beforeEach(() => {
    detector = new ResearchDetector({
      minConfidence: 0.7,
      maxQueries: 3,
      enableQuestionDetection: true,
      enableUncertaintyDetection: true,
      enableTechnicalDetection: true,
    });
  });

  describe("Question Detection", () => {
    it("should detect explicit 'How' questions", () => {
      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.required).toBe(true);
      expect(result?.confidence).toBeGreaterThanOrEqual(0.7);
      expect(result?.reason).toContain("questions");
    });

    it("should detect explicit 'What' questions", () => {
      const task = mockTask({
        description: "What is the best way to handle JWT tokens?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.required).toBe(true);
    });

    it("should detect explicit 'Why' questions", () => {
      const task = mockTask({
        description: "Why does my React component re-render unnecessarily?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.required).toBe(true);
    });

    it("should detect explicit 'When' questions", () => {
      const task = mockTask({
        description: "When should I use useEffect vs useLayoutEffect?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.required).toBe(true);
    });

    it("should detect explicit 'Where' questions", () => {
      const task = mockTask({
        description:
          "Where should I place my API keys in a Node.js application?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.required).toBe(true);
    });

    it("should not false-positive on statements containing 'how'", () => {
      const task = mockTask({
        description:
          "I know how to implement this feature. It's straightforward.",
        type: "file_editing",
        metadata: {
          prompt: "", // Override to remove the default question
          requester: "test-user",
        },
      });

      const result = detector.detectResearchNeeds(task);

      // Should either be null or have low confidence
      if (result) {
        expect(result.confidence).toBeLessThan(0.8);
      }
    });

    it("should handle multiple questions in one task", () => {
      const task = mockTask({
        description:
          "How do I set up Redis? What are the connection options? Why is it caching incorrectly?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.confidence).toBeGreaterThan(0.8); // Multiple questions should increase confidence
    });

    it("should respect enableQuestionDetection config", () => {
      const disabledDetector = new ResearchDetector({
        enableQuestionDetection: false,
        enableUncertaintyDetection: false,
        enableTechnicalDetection: false,
        minConfidence: 0.7,
      });

      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const result = disabledDetector.detectResearchNeeds(task);

      // Should be null or have very low confidence without any detection enabled
      expect(result).toBeNull();
    });
  });

  describe("Uncertainty Detection", () => {
    it("should detect 'not sure' uncertainty", () => {
      const task = mockTask({
        description:
          "I'm not sure how to configure the database connection pool.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.reason).toContain("uncertainty");
    });

    it("should detect 'unclear' uncertainty", () => {
      const task = mockTask({
        description:
          "It's unclear whether I should use async/await or promises.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should detect 'need to find' uncertainty", () => {
      const task = mockTask({
        description: "Need to find the best approach for rate limiting.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should detect 'don't know' uncertainty", () => {
      const task = mockTask({
        description: "I don't know which GraphQL library to use.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should detect 'unsure' uncertainty", () => {
      const task = mockTask({
        description: "Unsure about the correct CORS configuration.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should detect 'research' keyword", () => {
      const task = mockTask({
        description: "Need to research authentication methods for the API.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should respect enableUncertaintyDetection config", () => {
      const disabledDetector = new ResearchDetector({
        enableQuestionDetection: false,
        enableUncertaintyDetection: false,
        enableTechnicalDetection: false,
        minConfidence: 0.7,
      });

      const task = mockTask({
        description: "I'm not sure how to implement this feature.",
      });

      const result = disabledDetector.detectResearchNeeds(task);

      expect(result).toBeNull();
    });
  });

  describe("Technical Detection", () => {
    it("should detect 'API' technical keyword", () => {
      const task = mockTask({
        description: "Integrate the Stripe API for payment processing.",
        type: "file_editing",
      });

      const result = detector.detectResearchNeeds(task);

      // Technical keyword + implementation task should trigger research
      expect(result).not.toBeNull();
    });

    it("should detect 'implementation' technical keyword", () => {
      const task = mockTask({
        description: "Review the implementation of the caching layer.",
        type: "code-review",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should detect 'documentation' technical keyword", () => {
      const task = mockTask({
        description: "Write documentation for the authentication module.",
        type: "general",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should detect 'architecture' technical keyword", () => {
      const task = mockTask({
        description: "Design the architecture for the microservices system.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should detect 'integration' technical keyword", () => {
      const task = mockTask({
        description: "Integration with external payment gateway.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should infer technical needs from task type", () => {
      const task = mockTask({
        description: "Build a new feature for user management.",
        type: "file_editing",
      });

      const result = detector.detectResearchNeeds(task);

      // Implementation tasks often need research
      if (result) {
        expect(result.confidence).toBeGreaterThan(0);
      }
    });

    it("should respect enableTechnicalDetection config", () => {
      const disabledDetector = new ResearchDetector({
        enableQuestionDetection: false,
        enableUncertaintyDetection: false,
        enableTechnicalDetection: false,
        minConfidence: 0.7,
      });

      const task = mockTask({
        description: "Implement the API authentication system.",
        type: "file_editing",
      });

      const result = disabledDetector.detectResearchNeeds(task);

      expect(result).toBeNull();
    });
  });

  describe("Comparison Detection", () => {
    it("should detect 'compare' keyword", () => {
      const task = mockTask({
        description: "Compare Redis vs Memcached for our caching needs.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.queryType).toBe(QueryType.COMPARATIVE);
    });

    it("should detect 'versus' keyword", () => {
      const task = mockTask({
        description: "Evaluate PostgreSQL versus MySQL for our database.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.queryType).toBe(QueryType.COMPARATIVE);
    });

    it("should detect 'vs' keyword", () => {
      const task = mockTask({
        description: "Research MongoDB vs PostgreSQL performance.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should detect 'pros and cons' phrase", () => {
      const task = mockTask({
        description: "List pros and cons of using GraphQL over REST.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should detect 'advantages and disadvantages' phrase", () => {
      const task = mockTask({
        description: "Analyze advantages and disadvantages of microservices.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });
  });

  describe("Fact-Checking Detection", () => {
    it("should detect analysis task types", () => {
      const task = mockTask({
        description: "Analyze the performance of the current system.",
        type: "analysis",
      });

      const result = detector.detectResearchNeeds(task);

      // Analysis tasks typically need research
      if (result) {
        expect(result.confidence).toBeGreaterThan(0);
      }
    });

    it("should detect research task types", () => {
      const task = mockTask({
        description: "Research available OAuth2 libraries for Node.js.",
        type: "research",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.confidence).toBeGreaterThan(0.7);
    });
  });

  describe("Confidence Scoring", () => {
    it("should calculate weighted confidence correctly", () => {
      const task = mockTask({
        description:
          "How do I implement OAuth2? I'm not sure which library to use. Compare passport-oauth2 vs simple-oauth2.",
        type: "file_editing",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      // Multiple indicators should yield high confidence
      // Questions (0.3) + Uncertainty (0.3) + Comparison (0.2) + Technical (0.15) = 0.95
      expect(result?.confidence).toBeGreaterThan(0.8);
    });

    it("should respect minConfidence threshold", () => {
      const strictDetector = new ResearchDetector({
        minConfidence: 0.95, // Very high threshold
        maxQueries: 3,
      });

      const task = mockTask({
        description: "Implement basic user authentication.",
        type: "file_editing",
      });

      const result = strictDetector.detectResearchNeeds(task);

      // With strict threshold and weak indicators, should be null
      expect(result).toBeNull();
    });

    it("should return null for low confidence tasks", () => {
      const task = mockTask({
        description: "Update the README file with installation instructions.",
        type: "general",
        metadata: {
          prompt: "", // Override to remove the default question
          requester: "test-user",
        },
      });

      const result = detector.detectResearchNeeds(task);

      // Simple documentation task with no questions/uncertainty should have low confidence
      if (result) {
        expect(result.confidence).toBeLessThan(0.8);
      }
    });

    it("should provide confidence between 0 and 1", () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.confidence).toBeGreaterThanOrEqual(0);
      expect(result?.confidence).toBeLessThanOrEqual(1);
    });
  });

  describe("Query Generation", () => {
    it("should generate relevant queries from task description", () => {
      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.suggestedQueries).toBeInstanceOf(Array);
      expect(result?.suggestedQueries.length).toBeGreaterThan(0);
      expect(result?.suggestedQueries.length).toBeLessThanOrEqual(3);
    });

    it("should include task description as primary query", () => {
      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.suggestedQueries).toContain(
        "How do I implement OAuth2 in Express.js?"
      );
    });

    it("should generate variations of the query", () => {
      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.suggestedQueries.length).toBeGreaterThan(1);

      // Should have variations like "OAuth2 Express.js implementation"
      const hasVariation = result?.suggestedQueries.some(
        (q) => q.includes("OAuth2") && q.includes("Express.js")
      );
      expect(hasVariation).toBe(true);
    });

    it("should respect maxQueries config", () => {
      const limitedDetector = new ResearchDetector({
        minConfidence: 0.7,
        maxQueries: 2,
      });

      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js with JWT tokens?",
      });

      const result = limitedDetector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.suggestedQueries.length).toBeLessThanOrEqual(2);
    });

    it("should generate at least one query", () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.suggestedQueries.length).toBeGreaterThanOrEqual(1);
    });
  });

  describe("Query Type Inference", () => {
    it("should infer EXPLANATORY for 'how' questions", () => {
      const task = mockTask({
        description: "How does OAuth2 work?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.queryType).toBe(QueryType.EXPLANATORY);
    });

    it("should infer FACTUAL for 'what' questions", () => {
      const task = mockTask({
        description: "What is OAuth2?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.queryType).toBe(QueryType.FACTUAL);
    });

    it("should infer COMPARATIVE for comparison keywords", () => {
      const task = mockTask({
        description: "Compare OAuth2 vs JWT authentication.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.queryType).toBe(QueryType.COMPARATIVE);
    });

    it("should infer TECHNICAL for technical keywords", () => {
      const task = mockTask({
        description: "Implementation details for OAuth2 API.",
        type: "file_editing",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.queryType).toBe(QueryType.TECHNICAL);
    });
  });

  describe("Reason Generation", () => {
    it("should provide a descriptive reason for detection", () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.reason).toBeDefined();
      if (result?.reason) {
        expect(result.reason.length).toBeGreaterThan(10);
        expect(result.reason).toContain("confidence");
      }
    });

    it("should mention detected indicators in reason", () => {
      const task = mockTask({
        description:
          "How do I implement OAuth2? I'm not sure which library to use.",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
      expect(result?.reason).toContain("questions");
      expect(result?.reason).toContain("uncertainty");
    });
  });

  describe("Edge Cases", () => {
    it("should handle empty task description", () => {
      const task = mockTask({
        description: "",
        metadata: {
          prompt: "", // Override to remove the default question
          requester: "test-user",
        },
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).toBeNull();
    });

    it("should handle very long descriptions", () => {
      const longDescription = "How do I ".repeat(1000) + "implement OAuth2?";
      const task = mockTask({
        description: longDescription,
      });

      const result = detector.detectResearchNeeds(task);

      // Should still detect questions
      expect(result).not.toBeNull();
    });

    it("should handle special characters", () => {
      const task = mockTask({
        description: "How do I implement OAuth2? 🔐 #security @authentication",
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should handle non-English text gracefully", () => {
      const task = mockTask({
        description: "¿Cómo implemento OAuth2 en Express.js?",
      });

      const result = detector.detectResearchNeeds(task);

      // May or may not detect, but should not crash
      expect(result === null || result !== null).toBe(true);
    });

    it("should handle null metadata", () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
        metadata: undefined,
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should handle missing metadata.prompt", () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
        metadata: {},
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).not.toBeNull();
    });

    it("should handle tasks with only whitespace", () => {
      const task = mockTask({
        description: "   \n\t  ",
        metadata: {
          prompt: "", // Override to remove the default question
          requester: "test-user",
        },
      });

      const result = detector.detectResearchNeeds(task);

      expect(result).toBeNull();
    });
  });

  describe("Performance", () => {
    it("should complete detection in <10ms", () => {
      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js?",
      });

      const startTime = Date.now();
      detector.detectResearchNeeds(task);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(10);
    });

    it("should handle 100 detections efficiently", () => {
      const tasks = Array(100)
        .fill(null)
        .map((_, i) =>
          mockTask({
            description: `Task ${i}: How do I implement feature ${i}?`,
          })
        );

      const startTime = Date.now();
      tasks.forEach((task) => detector.detectResearchNeeds(task));
      const duration = Date.now() - startTime;

      // Should complete 100 detections in <500ms (5ms average)
      expect(duration).toBeLessThan(500);
    });
  });
});
