/**
 * Integration tests for DSPy Client
 *
 * Tests TypeScript client communication with Python DSPy service.
 *
 * @author @darianrosebrook
 */

import type {
  JudgeEvaluationRequest,
  RubricOptimizationRequest,
} from "@/dspy-integration/DSPyClient.js";
import { DSPyClient } from "@/dspy-integration/DSPyClient.js";

describe("DSPyClient Integration Tests", () => {
  let client: DSPyClient;
  const dspyServiceUrl =
    process.env.DSPY_SERVICE_URL ?? "http://localhost:8001";

  beforeAll(() => {
    client = new DSPyClient({
      baseUrl: dspyServiceUrl,
      timeout: 30000,
      maxRetries: 3,
    });
  });

  describe("health check", () => {
    it("should successfully check DSPy service health", async () => {
      const health = await client.health();

      expect(health).toBeDefined();
      expect(health.status).toBe("healthy");
      expect(health.version).toBeDefined();
    });

    it("should throw error if service is unavailable", async () => {
      const badClient = new DSPyClient({
        baseUrl: "http://localhost:9999",
        timeout: 1000,
        maxRetries: 1,
      });

      await expect(badClient.health()).rejects.toThrow();
    });
  });

  describe("rubric optimization", () => {
    it("should optimize rubric computation", async () => {
      const request: RubricOptimizationRequest = {
        taskContext: "Generate JSON response for user profile",
        agentOutput: '{"name": "John", "age": 30}',
        evaluationCriteria: "Valid JSON structure with required fields",
      };

      const result = await client.optimizeRubric(request);

      expect(result).toBeDefined();
      expect(result.rewardScore).toBeGreaterThanOrEqual(0);
      expect(result.rewardScore).toBeLessThanOrEqual(1);
      expect(result.reasoning).toBeDefined();
      expect(result.improvementSuggestions).toBeDefined();
      expect(result.metadata).toBeDefined();
    });

    it("should handle invalid rubric requests", async () => {
      const request: RubricOptimizationRequest = {
        taskContext: "",
        agentOutput: "",
        evaluationCriteria: "",
      };

      await expect(client.optimizeRubric(request)).rejects.toThrow();
    });

    it("should retry on transient failures", async () => {
      const request: RubricOptimizationRequest = {
        taskContext: "Test retry logic",
        agentOutput: "test output",
        evaluationCriteria: "test criteria",
      };

      // Should succeed even with potential transient failures
      const result = await client.optimizeRubric(request);
      expect(result).toBeDefined();
    });
  });

  describe("judge evaluation", () => {
    const testCases: Array<{
      judgeType: "relevance" | "faithfulness" | "minimality" | "safety";
      description: string;
    }> = [
      { judgeType: "relevance", description: "relevance judgment" },
      { judgeType: "faithfulness", description: "faithfulness judgment" },
      { judgeType: "minimality", description: "minimality judgment" },
      { judgeType: "safety", description: "safety judgment" },
    ];

    testCases.forEach(({ judgeType, description }) => {
      it(`should perform ${description}`, async () => {
        const request: JudgeEvaluationRequest = {
          judgeType,
          artifact: "User profile was successfully created",
          groundTruth: "Create user profile",
          context: "User registration flow",
        };

        const result = await client.evaluateWithJudge(request);

        expect(result).toBeDefined();
        expect(result.judgment).toBeDefined();
        expect(result.confidence).toBeGreaterThanOrEqual(0);
        expect(result.confidence).toBeLessThanOrEqual(1);
        expect(result.reasoning).toBeDefined();
        expect(result.metadata).toBeDefined();
      });
    });

    it("should handle concurrent judge evaluations", async () => {
      const requests: JudgeEvaluationRequest[] = [
        {
          judgeType: "relevance",
          artifact: "Output 1",
          groundTruth: "Expected 1",
          context: "Context 1",
        },
        {
          judgeType: "faithfulness",
          artifact: "Output 2",
          groundTruth: "Expected 2",
          context: "Context 2",
        },
        {
          judgeType: "minimality",
          artifact: "Output 3",
          groundTruth: "Expected 3",
          context: "Context 3",
        },
      ];

      const results = await Promise.all(
        requests.map((req) => client.evaluateWithJudge(req))
      );

      expect(results).toHaveLength(3);
      results.forEach((result) => {
        expect(result).toBeDefined();
        expect(result.judgment).toBeDefined();
      });
    });
  });

  describe("signature optimization", () => {
    it("should optimize DSPy signature", async () => {
      const evalData = [
        {
          input: "test input 1",
          output: "test output 1",
          expected: "expected 1",
        },
        {
          input: "test input 2",
          output: "test output 2",
          expected: "expected 2",
        },
      ];

      const result = await client.optimizeSignature({
        signatureId: "test_signature_v1",
        evalData,
        optimizer: "MIPROv2",
      });

      expect(result).toBeDefined();
      expect(result.optimizedSignatureId).toBeDefined();
      expect(result.improvementMetrics).toBeDefined();
      expect(result.metadata).toBeDefined();
    });
  });

  describe("error handling", () => {
    it("should handle network errors gracefully", async () => {
      const badClient = new DSPyClient({
        baseUrl: "http://invalid-host:8001",
        timeout: 1000,
        maxRetries: 1,
      });

      await expect(
        badClient.optimizeRubric({
          taskContext: "test",
          agentOutput: "test",
          evaluationCriteria: "test",
        })
      ).rejects.toThrow();
    });

    it("should handle timeout errors", async () => {
      const slowClient = new DSPyClient({
        baseUrl: dspyServiceUrl,
        timeout: 1, // 1ms timeout
        maxRetries: 1,
      });

      await expect(
        slowClient.optimizeRubric({
          taskContext: "test",
          agentOutput: "test",
          evaluationCriteria: "test",
        })
      ).rejects.toThrow();
    });
  });

  describe("performance", () => {
    it("should handle high-volume rubric evaluations", async () => {
      const requests: RubricOptimizationRequest[] = Array.from(
        { length: 10 },
        (_, i) => ({
          taskContext: `Task ${i}`,
          agentOutput: `Output ${i}`,
          evaluationCriteria: `Criteria ${i}`,
        })
      );

      const startTime = Date.now();
      const results = await Promise.all(
        requests.map((req) => client.optimizeRubric(req))
      );
      const duration = Date.now() - startTime;

      expect(results).toHaveLength(10);
      expect(duration).toBeLessThan(60000); // Should complete within 60 seconds
    });
  });
});
