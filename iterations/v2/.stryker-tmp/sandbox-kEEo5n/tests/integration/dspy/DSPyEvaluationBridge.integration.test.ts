/**
 * Integration tests for DSPy Evaluation Bridge
 *
 * Tests integration between existing evaluation framework and DSPy service.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { DSPyEvaluationBridge } from "@/evaluation/DSPyEvaluationBridge";
import { ModelBasedJudge } from "@/evaluation/ModelBasedJudge";

describe("DSPyEvaluationBridge Integration Tests", () => {
  let bridge: DSPyEvaluationBridge;
  let mockJudge: ModelBasedJudge;

  beforeAll(() => {
    // Create mock existing judge with mock LLM provider
    const mockLLMProvider = {
      generate: jest.fn().mockResolvedValue({ text: "Mock response" }),
    } as any;

    mockJudge = new ModelBasedJudge({
      llmProvider: mockLLMProvider,
      judgeType: "relevance",
      confidenceThreshold: 0.7,
    } as any);

    // Create bridge
    bridge = new DSPyEvaluationBridge(
      {
        dspyServiceUrl: process.env.DSPY_SERVICE_URL ?? "http://localhost:8001",
        enabled: true,
        fallbackOnError: true,
      },
      mockJudge
    );
  });

  afterAll(async () => {
    // Clean up any resources
    if (bridge) {
      // Add cleanup logic if the bridge has cleanup methods
    }
    jest.clearAllMocks();
  });

  describe("rubric evaluation", () => {
    it("should evaluate rubric using DSPy enhancement", async () => {
      const result = await bridge.evaluateRubric({
        taskContext: "Generate user profile JSON",
        agentOutput:
          '{"name": "John Doe", "age": 30, "email": "john@example.com"}',
        evaluationCriteria: "Valid JSON with name, age, and email fields",
      });

      expect(result).toBeDefined();
      expect(result.score).toBeGreaterThanOrEqual(0);
      expect(result.score).toBeLessThanOrEqual(1);
      expect(result.reasoning).toBeDefined();
      expect(result.suggestions).toBeInstanceOf(Array);
      expect(result.metadata.dspyUsed).toBe(true);
      expect(result.metadata.enhanced).toBe(true);
    });

    it("should fallback to legacy evaluation on DSPy failure", async () => {
      // Create bridge with disabled DSPy
      const fallbackBridge = new DSPyEvaluationBridge(
        {
          dspyServiceUrl: "http://invalid-host:9999",
          enabled: false,
          fallbackOnError: true,
        },
        mockJudge
      );

      const result = await fallbackBridge.evaluateRubric({
        taskContext: "Test task",
        agentOutput: "Test output",
        evaluationCriteria: "Test criteria",
      });

      expect(result).toBeDefined();
      expect(result.metadata.dspyUsed).toBe(false);
      expect(result.metadata.enhanced).toBe(false);
    });

    it("should evaluate multiple rubrics concurrently", async () => {
      const requests = [
        {
          taskContext: "Task 1",
          agentOutput: "Output 1",
          evaluationCriteria: "Criteria 1",
        },
        {
          taskContext: "Task 2",
          agentOutput: "Output 2",
          evaluationCriteria: "Criteria 2",
        },
        {
          taskContext: "Task 3",
          agentOutput: "Output 3",
          evaluationCriteria: "Criteria 3",
        },
      ];

      const results = await Promise.all(
        requests.map((req) => bridge.evaluateRubric(req))
      );

      expect(results).toHaveLength(3);
      results.forEach((result) => {
        expect(result).toBeDefined();
        expect(result.score).toBeGreaterThanOrEqual(0);
        expect(result.score).toBeLessThanOrEqual(1);
      });
    });
  });

  describe("judge evaluation", () => {
    const judgeTypes: Array<
      "relevance" | "faithfulness" | "minimality" | "safety"
    > = ["relevance", "faithfulness", "minimality", "safety"];

    judgeTypes.forEach((judgeType) => {
      it(`should perform ${judgeType} judgment with DSPy`, async () => {
        const result = await bridge.evaluateWithJudge(
          judgeType,
          "User profile was successfully created with all required fields",
          "Create a user profile",
          "User registration workflow"
        );

        expect(result).toBeDefined();
        expect(result.judgment).toBeDefined();
        expect(result.confidence).toBeGreaterThanOrEqual(0);
        expect(result.confidence).toBeLessThanOrEqual(1);
        expect(result.reasoning).toBeDefined();
        expect(result.metadata.dspyEnhanced).toBe(true);
      });
    });

    it("should handle concurrent judge evaluations", async () => {
      const evaluations = [
        { type: "relevance" as const, artifact: "Artifact 1" },
        { type: "faithfulness" as const, artifact: "Artifact 2" },
        { type: "minimality" as const, artifact: "Artifact 3" },
      ];

      const results = await Promise.all(
        evaluations.map((evaluation) =>
          bridge.evaluateWithJudge(
            evaluation.type,
            evaluation.artifact,
            "Ground truth",
            "Context"
          )
        )
      );

      expect(results).toHaveLength(3);
      results.forEach((result) => {
        expect(result).toBeDefined();
        expect(result.judgment).toBeDefined();
      });
    });
  });

  describe("fallback behavior", () => {
    it("should use legacy evaluation when DSPy is disabled", async () => {
      const disabledBridge = new DSPyEvaluationBridge(
        {
          dspyServiceUrl: "http://localhost:8001",
          enabled: false,
          fallbackOnError: true,
        },
        mockJudge
      );

      const result = await disabledBridge.evaluateRubric({
        taskContext: "Test",
        agentOutput: "Test",
        evaluationCriteria: "Test",
      });

      expect(result.metadata.dspyUsed).toBe(false);
    });

    it("should fallback on DSPy service errors", async () => {
      const errorBridge = new DSPyEvaluationBridge(
        {
          dspyServiceUrl: "http://invalid-host:9999",
          enabled: true,
          fallbackOnError: true,
        },
        mockJudge
      );

      const result = await errorBridge.evaluateRubric({
        taskContext: "Test",
        agentOutput: "Test",
        evaluationCriteria: "Test",
      });

      expect(result).toBeDefined();
    });

    it("should throw error when fallback is disabled", async () => {
      const noFallbackBridge = new DSPyEvaluationBridge(
        {
          dspyServiceUrl: "http://invalid-host:9999",
          enabled: true,
          fallbackOnError: false,
        },
        mockJudge
      );

      await expect(
        noFallbackBridge.evaluateRubric({
          taskContext: "Test",
          agentOutput: "Test",
          evaluationCriteria: "Test",
        })
      ).rejects.toThrow();
    });
  });

  describe("performance", () => {
    it("should handle high-volume evaluations", async () => {
      const requests = Array.from({ length: 20 }, (_, i) => ({
        taskContext: `Task ${i}`,
        agentOutput: `Output ${i}`,
        evaluationCriteria: `Criteria ${i}`,
      }));

      const startTime = Date.now();
      const results = await Promise.all(
        requests.map((req) => bridge.evaluateRubric(req))
      );
      const duration = Date.now() - startTime;

      expect(results).toHaveLength(20);
      expect(duration).toBeLessThan(120000); // 2 minutes for 20 evaluations
    });

    it("should maintain response quality under load", async () => {
      const requests = Array.from({ length: 10 }, () => ({
        taskContext: "Generate user profile",
        agentOutput: '{"name": "John", "age": 30}',
        evaluationCriteria: "Valid JSON with required fields",
      }));

      const results = await Promise.all(
        requests.map((req) => bridge.evaluateRubric(req))
      );

      // All results should have similar scores (within 0.3)
      const scores = results.map((r) => r.score);
      const maxScore = Math.max(...scores);
      const minScore = Math.min(...scores);

      expect(maxScore - minScore).toBeLessThan(0.3);
    });
  });
});
