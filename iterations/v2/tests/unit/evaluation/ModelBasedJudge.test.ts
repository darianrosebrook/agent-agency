/**
 * @fileoverview
 * Unit tests for ModelBasedJudge
 * Tests LLM-based judgment system
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { ModelBasedJudge } from "../../../src/evaluation/ModelBasedJudge";
import type { JudgmentInput } from "../../../src/types/judge";

describe("ModelBasedJudge", () => {
  let judge: ModelBasedJudge;

  beforeEach(() => {
    judge = new ModelBasedJudge();
  });

  describe("Multi-Criteria Evaluation", () => {
    it("should evaluate all four criteria", async () => {
      const input: JudgmentInput = {
        task: "Write a greeting function",
        output: "function greet(name) { return `Hello, ${name}!`; }",
      };

      const result = await judge.evaluate(input);

      expect(result.assessments).toHaveLength(4);
      expect(result.assessments.map((a) => a.criterion)).toEqual([
        "faithfulness",
        "relevance",
        "minimality",
        "safety",
      ]);
    });

    it("should provide confidence scores for each criterion", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output with sufficient length to demonstrate relevance",
      };

      const result = await judge.evaluate(input);

      result.assessments.forEach((assessment) => {
        expect(assessment.confidence).toBeGreaterThanOrEqual(0);
        expect(assessment.confidence).toBeLessThanOrEqual(1);
      });
    });

    it("should calculate overall score as average", async () => {
      const input: JudgmentInput = {
        task: "Test",
        output: "Test output",
      };

      const result = await judge.evaluate(input);

      const avgScore =
        result.assessments.reduce((sum, a) => sum + a.score, 0) /
        result.assessments.length;

      expect(result.overallScore).toBeCloseTo(avgScore, 5);
    });
  });

  describe("Threshold Checking", () => {
    it("should check if criteria pass thresholds", async () => {
      const input: JudgmentInput = {
        task: "Test",
        output: "Test output",
      };

      const result = await judge.evaluate(input);

      result.assessments.forEach((assessment) => {
        expect(assessment.passes).toBeDefined();
        expect(typeof assessment.passes).toBe("boolean");
      });
    });

    it("should set allCriteriaPass correctly", async () => {
      const input: JudgmentInput = {
        task: "Write safe code",
        output: "function test() { return true; }",
        expectedOutput: "function test() { return true; }",
      };

      const result = await judge.evaluate(input);

      const allPass = result.assessments.every((a) => a.passes);
      expect(result.allCriteriaPass).toBe(allPass);
    });
  });

  describe("Input Validation", () => {
    it("should throw on empty task", async () => {
      const input: JudgmentInput = {
        task: "",
        output: "Test output",
      };

      await expect(judge.evaluate(input)).rejects.toThrow(/task is required/i);
    });

    it("should throw on empty output", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "",
      };

      await expect(judge.evaluate(input)).rejects.toThrow(
        /output is required/i
      );
    });

    it("should accept valid input", async () => {
      const input: JudgmentInput = {
        task: "Valid task",
        output: "Valid output",
      };

      await expect(judge.evaluate(input)).resolves.toBeDefined();
    });
  });

  describe("Performance", () => {
    it("should complete evaluation within 500ms", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      const result = await judge.evaluate(input);

      expect(result.evaluationTimeMs).toBeLessThan(500);
    });
  });

  describe("Metrics Tracking", () => {
    it("should initialize metrics correctly", () => {
      const metrics = judge.getMetrics();

      expect(metrics.totalJudgments).toBe(0);
      expect(metrics.averageEvaluationTimeMs).toBe(0);
    });

    it("should update metrics after evaluation", async () => {
      const input: JudgmentInput = {
        task: "Test",
        output: "Test output",
      };

      await judge.evaluate(input);
      const metrics = judge.getMetrics();

      expect(metrics.totalJudgments).toBe(1);
      expect(metrics.averageEvaluationTimeMs).toBeGreaterThanOrEqual(0);
    });

    it("should track judgments by criterion", async () => {
      const input: JudgmentInput = {
        task: "Test",
        output: "Test output",
      };

      await judge.evaluate(input);
      const metrics = judge.getMetrics();

      expect(metrics.judgmentsByCriterion.faithfulness).toBe(1);
      expect(metrics.judgmentsByCriterion.relevance).toBe(1);
      expect(metrics.judgmentsByCriterion.minimality).toBe(1);
      expect(metrics.judgmentsByCriterion.safety).toBe(1);
    });

    it("should reset metrics", async () => {
      const input: JudgmentInput = {
        task: "Test",
        output: "Test",
      };

      await judge.evaluate(input);
      judge.resetMetrics();

      const metrics = judge.getMetrics();
      expect(metrics.totalJudgments).toBe(0);
    });
  });

  describe("Configuration", () => {
    it("should use default configuration", () => {
      const config = judge.getConfig();

      expect(config.llm.provider).toBe("ollama");
      expect(config.enableFallback).toBe(true);
    });

    it("should allow config updates", () => {
      judge.updateConfig({
        enableFallback: false,
      });

      const config = judge.getConfig();
      expect(config.enableFallback).toBe(false);
    });
  });

  describe("Expected Output", () => {
    it("should use expected output when provided", async () => {
      const input: JudgmentInput = {
        task: "Test",
        output: "Test output",
        expectedOutput: "Test output",
      };

      const result = await judge.evaluate(input);

      expect(result.assessments).toBeDefined();
    });
  });

  describe("Context Handling", () => {
    it("should handle context when provided", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
        context: { key: "value" },
      };

      const result = await judge.evaluate(input);

      expect(result.assessments).toHaveLength(4);
    });
  });

  describe("Multi-Evaluation", () => {
    it("should maintain separate metrics for multiple evaluations", async () => {
      const input1: JudgmentInput = {
        task: "Test 1",
        output: "Output 1",
      };

      const input2: JudgmentInput = {
        task: "Test 2",
        output: "Output 2",
      };

      await judge.evaluate(input1);
      await judge.evaluate(input2);

      const metrics = judge.getMetrics();

      expect(metrics.totalJudgments).toBe(2);
      expect(metrics.judgmentsByCriterion.faithfulness).toBe(2);
    });
  });
});
