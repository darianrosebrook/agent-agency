/**
 * @fileoverview
 * Unit tests for TaskComplexityAnalyzer
 * Tests complexity assessment logic and edge cases
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { TaskComplexityAnalyzer } from "../../../src/thinking/TaskComplexityAnalyzer";
import {
  ComplexityLevel,
  TaskCharacteristics,
} from "../../../src/types/thinking-budget";

describe("TaskComplexityAnalyzer", () => {
  let analyzer: TaskComplexityAnalyzer;

  beforeEach(() => {
    analyzer = new TaskComplexityAnalyzer();
  });

  describe("Trivial Complexity Assessment", () => {
    it("should assess simple query as trivial", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 200,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.TRIVIAL);
      expect(assessment.confidence).toBeGreaterThan(0.9);
      expect(assessment.reasoning).toContain("trivial");
    });

    it("should assess small context task as trivial", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 800,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.TRIVIAL);
      expect(assessment.confidence).toBeGreaterThan(0.7);
    });

    it("should return assessment within performance budget", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 100,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      // Assessment should complete within 20ms (per working spec)
      expect(assessment.assessmentTimeMs).toBeLessThan(20);
    });
  });

  describe("Standard Complexity Assessment", () => {
    it("should assess single tool task as standard", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1500,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.STANDARD);
      expect(assessment.confidence).toBeGreaterThan(0.7);
      expect(assessment.reasoning).toContain("standard");
    });

    it("should assess moderate context as standard", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 2,
        contextSize: 3000,
        stepCount: 3,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.STANDARD);
      expect(assessment.confidence).toBeGreaterThan(0.8);
    });

    it("should assess external API calls as standard", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 500,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: true,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.STANDARD);
    });
  });

  describe("Complex Complexity Assessment", () => {
    it("should assess high tool usage as complex", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 3,
        contextSize: 2000,
        stepCount: 4,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.COMPLEX);
      expect(assessment.confidence).toBeGreaterThan(0.9);
      expect(assessment.reasoning).toContain("complex");
    });

    it("should assess large context as complex", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 6000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.COMPLEX);
      expect(assessment.confidence).toBeGreaterThan(0.9);
    });

    it("should assess many steps as complex", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 5,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.COMPLEX);
    });

    it("should assess multi-turn with external calls as complex", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 2000,
        stepCount: 2,
        multiTurn: true,
        hasExternalCalls: true,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.COMPLEX);
    });
  });

  describe("Confidence Scoring", () => {
    it("should return high confidence for clear trivial case", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 100,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.confidence).toBeGreaterThan(0.9);
    });

    it("should return moderate confidence for edge case", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 950,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      // Edge case near trivial/standard boundary
      expect(assessment.confidence).toBeLessThan(0.9);
      expect(assessment.confidence).toBeGreaterThan(0.5);
    });

    it("should never return confidence outside [0, 1]", () => {
      const testCases: TaskCharacteristics[] = [
        {
          toolCount: 0,
          contextSize: 0,
          stepCount: 0,
          multiTurn: false,
          hasExternalCalls: false,
        },
        {
          toolCount: 10,
          contextSize: 100000,
          stepCount: 100,
          multiTurn: true,
          hasExternalCalls: true,
        },
      ];

      testCases.forEach((characteristics) => {
        const assessment = analyzer.analyze(characteristics);
        expect(assessment.confidence).toBeGreaterThanOrEqual(0);
        expect(assessment.confidence).toBeLessThanOrEqual(1);
      });
    });
  });

  describe("Reasoning Generation", () => {
    it("should include tool count in reasoning", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 2,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.reasoning).toMatch(/tools/i);
      expect(assessment.reasoning).toContain("2");
    });

    it("should include context size in reasoning", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 3500,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.reasoning).toMatch(/context/i);
      expect(assessment.reasoning).toContain("3500");
    });

    it("should mention multi-turn in reasoning", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: true,
        hasExternalCalls: false,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.reasoning).toMatch(/multi-turn/i);
    });

    it("should mention external calls in reasoning", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: true,
      };

      const assessment = analyzer.analyze(characteristics);

      expect(assessment.reasoning).toMatch(/external/i);
    });
  });

  describe("Determinism", () => {
    it("should return identical results for identical inputs", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 2,
        contextSize: 2500,
        stepCount: 3,
        multiTurn: false,
        hasExternalCalls: true,
      };

      const assessment1 = analyzer.analyze(characteristics);
      const assessment2 = analyzer.analyze(characteristics);

      expect(assessment1.level).toBe(assessment2.level);
      expect(assessment1.confidence).toBe(assessment2.confidence);
      expect(assessment1.reasoning).toBe(assessment2.reasoning);
    });
  });
});
