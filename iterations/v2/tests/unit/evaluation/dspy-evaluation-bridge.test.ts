/**
 * @fileoverview
 * Unit tests for DSPyEvaluationBridge
 *
 * @author @darianrosebrook
 */

import type { RubricEvaluationRequest } from "@/evaluation/DSPyEvaluationBridge";
import { DSPyEvaluationBridge } from "@/evaluation/DSPyEvaluationBridge";
import { MockLLMProvider } from "@/evaluation/LLMProvider";
import { ModelBasedJudge } from "@/evaluation/ModelBasedJudge";

describe("DSPyEvaluationBridge", () => {
  let bridge: DSPyEvaluationBridge;
  let mockJudge: ModelBasedJudge;
  let mockProvider: MockLLMProvider;

  beforeEach(() => {
    mockProvider = new MockLLMProvider({
      provider: "mock",
      model: "test-model",
      temperature: 0.7,
      maxTokens: 1000,
    });

    mockJudge = new ModelBasedJudge(
      {
        enableFallback: true,
        thresholds: {
          faithfulness: 0.7,
          relevance: 0.7,
          minimality: 0.7,
          safety: 0.8,
        },
      },
      mockProvider
    );

    bridge = new DSPyEvaluationBridge(
      {
        dspyServiceUrl: "http://localhost:8000",
        enabled: false, // Start with DSPy disabled to test fallback
        fallbackOnError: true,
      },
      mockJudge
    );
  });

  describe("evaluateRubric", () => {
    it("should use legacy evaluation when DSPy is disabled", async () => {
      const request: RubricEvaluationRequest = {
        taskContext: "Test task context",
        agentOutput: "Test agent output",
        evaluationCriteria: "Test evaluation criteria",
      };

      const result = await bridge.evaluateRubric(request);

      expect(result).toBeDefined();
      expect(result.score).toBeGreaterThanOrEqual(0);
      expect(result.score).toBeLessThanOrEqual(1);
      expect(result.reasoning).toBeDefined();
      expect(result.suggestions).toBeInstanceOf(Array);
      expect(result.metadata.enhanced).toBe(false);
      expect(result.metadata.dspyUsed).toBe(false);
    });

    it("should handle evaluation with comprehensive criteria", async () => {
      const request: RubricEvaluationRequest = {
        taskContext: "Write a function to calculate fibonacci numbers",
        agentOutput:
          "def fibonacci(n):\n    if n <= 1:\n        return n\n    return fibonacci(n-1) + fibonacci(n-2)",
        evaluationCriteria: "Code correctness, efficiency, and readability",
      };

      const result = await bridge.evaluateRubric(request);

      expect(result).toBeDefined();
      expect(result.score).toBeGreaterThanOrEqual(0);
      expect(result.score).toBeLessThanOrEqual(1);
      expect(result.reasoning).toContain("model-based judge");
      expect(result.suggestions).toBeInstanceOf(Array);
      expect(result.metadata.enhanced).toBe(false);
    });

    it("should provide meaningful suggestions", async () => {
      const request: RubricEvaluationRequest = {
        taskContext: "Create a simple calculator",
        agentOutput: "2 + 2 = 4",
        evaluationCriteria: "Completeness and functionality",
      };

      const result = await bridge.evaluateRubric(request);

      expect(result.suggestions).toBeInstanceOf(Array);
      expect(result.suggestions.length).toBeGreaterThan(0);
      expect(result.suggestions[0]).toBeDefined();
      expect(typeof result.suggestions[0]).toBe("string");
    });

    it("should handle empty or minimal outputs", async () => {
      const request: RubricEvaluationRequest = {
        taskContext: "Explain quantum computing",
        agentOutput: "",
        evaluationCriteria: "Completeness and accuracy",
      };

      const result = await bridge.evaluateRubric(request);

      expect(result).toBeDefined();
      expect(result.score).toBeGreaterThanOrEqual(0);
      expect(result.score).toBeLessThanOrEqual(1);
      expect(result.reasoning).toBeDefined();
    });

    it("should maintain consistent scoring across multiple evaluations", async () => {
      const request: RubricEvaluationRequest = {
        taskContext: "Test task",
        agentOutput: "Test output",
        evaluationCriteria: "Test criteria",
      };

      const result1 = await bridge.evaluateRubric(request);
      const result2 = await bridge.evaluateRubric(request);

      // Results should be consistent (within reasonable tolerance)
      expect(Math.abs(result1.score - result2.score)).toBeLessThan(0.1);
    });
  });

  describe("configuration", () => {
    it("should respect enabled configuration", async () => {
      const request: RubricEvaluationRequest = {
        taskContext: "Test task",
        agentOutput: "Test output",
        evaluationCriteria: "Test criteria",
      };

      // With DSPy disabled, should use legacy evaluation
      const result = await bridge.evaluateRubric(request);
      expect(result.metadata.dspyUsed).toBe(false);
    });

    it("should handle fallback configuration", async () => {
      const bridgeWithFallback = new DSPyEvaluationBridge(
        {
          dspyServiceUrl: "http://localhost:8000",
          enabled: true,
          fallbackOnError: true,
        },
        mockJudge
      );

      const request: RubricEvaluationRequest = {
        taskContext: "Test task",
        agentOutput: "Test output",
        evaluationCriteria: "Test criteria",
      };

      // Should fallback to legacy evaluation when DSPy fails
      const result = await bridgeWithFallback.evaluateRubric(request);
      expect(result).toBeDefined();
      expect(result.metadata.enhanced).toBe(false);
    });
  });

  describe("error handling", () => {
    it("should handle malformed requests gracefully", async () => {
      const malformedRequest = {
        taskContext: "",
        agentOutput: "",
        evaluationCriteria: "",
      } as RubricEvaluationRequest;

      const result = await bridge.evaluateRubric(malformedRequest);

      expect(result).toBeDefined();
      expect(result.score).toBeGreaterThanOrEqual(0);
      expect(result.score).toBeLessThanOrEqual(1);
    });

    it("should handle very long inputs", async () => {
      const longString = "a".repeat(10000);
      const request: RubricEvaluationRequest = {
        taskContext: longString,
        agentOutput: longString,
        evaluationCriteria: longString,
      };

      const result = await bridge.evaluateRubric(request);

      expect(result).toBeDefined();
      expect(result.score).toBeGreaterThanOrEqual(0);
      expect(result.score).toBeLessThanOrEqual(1);
    });
  });

  describe("integration with ModelBasedJudge", () => {
    it("should properly integrate with existing evaluation framework", async () => {
      const request: RubricEvaluationRequest = {
        taskContext: "Complex task requiring multiple criteria evaluation",
        agentOutput: "Comprehensive output with multiple aspects",
        evaluationCriteria: "faithfulness, relevance, minimality, safety",
      };

      const result = await bridge.evaluateRubric(request);

      expect(result).toBeDefined();
      expect(result.score).toBeGreaterThanOrEqual(0);
      expect(result.score).toBeLessThanOrEqual(1);
      expect(result.reasoning).toContain("model-based judge");
      expect(result.metadata.enhanced).toBe(false);
    });

    it("should provide detailed evaluation metadata", async () => {
      const request: RubricEvaluationRequest = {
        taskContext: "Test task",
        agentOutput: "Test output",
        evaluationCriteria: "Test criteria",
      };

      const result = await bridge.evaluateRubric(request);

      expect(result.metadata).toBeDefined();
      expect(result.metadata.enhanced).toBe(false);
      expect(result.metadata.dspyUsed).toBe(false);
      expect(typeof result.metadata).toBe("object");
    });
  });
});
