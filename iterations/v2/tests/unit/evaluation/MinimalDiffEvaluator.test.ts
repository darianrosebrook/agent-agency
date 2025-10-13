/**
 * @fileoverview
 * Unit tests for MinimalDiffEvaluator
 * Tests minimality factor calculation
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { MinimalDiffEvaluator } from "../../../src/evaluation/MinimalDiffEvaluator";
import type { CodeDiff } from "../../../src/types/evaluation";

describe("MinimalDiffEvaluator", () => {
  let evaluator: MinimalDiffEvaluator;

  beforeEach(() => {
    evaluator = new MinimalDiffEvaluator();
  });

  describe("Minimality Factor Calculation", () => {
    it("should give high score for minimal changes", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "const x = 2;",
        language: "typescript",
      };

      const result = evaluator.evaluate(diff);

      expect(result.minimalityFactor).toBeGreaterThan(0.8);
      expect(result.qualityAssessment).toBe("minimal");
    });

    it("should give moderate score for medium changes", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "function foo() { const x = 1; const y = 2; return x + y; }",
        language: "typescript",
      };

      const result = evaluator.evaluate(diff);

      expect(result.minimalityFactor).toBeLessThan(0.8);
      expect(result.minimalityFactor).toBeGreaterThan(0.3);
      expect(["minimal", "moderate"]).toContain(result.qualityAssessment);
    });

    it("should give low score for extensive changes", () => {
      const before = "const x = 1;";
      const after = Array.from(
        { length: 100 },
        (_, i) => `const x${i} = ${i};`
      ).join("\n");

      const diff: CodeDiff = {
        before,
        after,
        language: "typescript",
      };

      const result = evaluator.evaluate(diff);

      // Extensive changes should score lower than minimal
      expect(result.minimalityFactor).toBeLessThan(0.9);
      expect(result.linesChanged).toBeGreaterThan(50);
    });
  });

  describe("Scaffolding Penalty", () => {
    it("should evaluate scaffolding detection", () => {
      const withoutScaffolding: CodeDiff = {
        before: "const x = 1;",
        after: "const x = 2; const y = 3;",
        language: "typescript",
      };

      const withPotentialScaffolding: CodeDiff = {
        before: "const x = 1;",
        after: Array.from({ length: 100 }, (_, i) =>
          i % 2 === 0 ? `// Comment ${i}` : `const x${i} = ${i};`
        ).join("\n"),
        language: "typescript",
      };

      const resultWithout = evaluator.evaluate(withoutScaffolding);
      const resultWith = evaluator.evaluate(withPotentialScaffolding);

      // Larger changes should generally score lower
      expect(resultWith.linesChanged).toBeGreaterThan(
        resultWithout.linesChanged
      );
      expect(resultWith.scaffolding).toBeDefined();
      expect(resultWith.scaffolding.detected).toBeDefined();
    });

    it("should respect scaffolding detection disabled", () => {
      const noScaffoldingEval = new MinimalDiffEvaluator({
        enableScaffoldingDetection: false,
      });

      const diff: CodeDiff = {
        before: "",
        after: Array.from({ length: 100 }, (_, i) => `// Comment ${i}`).join(
          "\n"
        ),
        language: "typescript",
      };

      const result = noScaffoldingEval.evaluate(diff);

      expect(result.scaffolding.detected).toBe(false);
    });
  });

  describe("AST Similarity", () => {
    it("should track AST similarity", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "const x = 2;",
        language: "typescript",
      };

      const result = evaluator.evaluate(diff);

      expect(result.astSimilarity).toBeGreaterThan(0);
      expect(result.astSimilarity).toBeLessThanOrEqual(1);
    });
  });

  describe("Lines Changed", () => {
    it("should track lines changed", () => {
      const diff: CodeDiff = {
        before: "line1\nline2",
        after: "line1\nline2\nline3",
        language: "typescript",
      };

      const result = evaluator.evaluate(diff);

      expect(result.linesChanged).toBe(1);
    });
  });

  describe("Performance", () => {
    it("should evaluate within 200ms", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "const x = 2;",
        language: "typescript",
      };

      const result = evaluator.evaluate(diff);

      expect(result.evaluationTimeMs).toBeLessThan(200);
    });

    it("should handle large diffs within budget", () => {
      const before = Array.from(
        { length: 500 },
        (_, i) => `const x${i} = ${i};`
      ).join("\n");
      const after = Array.from(
        { length: 500 },
        (_, i) => `const x${i} = ${i + 1};`
      ).join("\n");

      const diff: CodeDiff = {
        before,
        after,
        language: "typescript",
      };

      const startTime = Date.now();
      evaluator.evaluate(diff);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(200);
    });
  });

  describe("Minimality Factor Bounds", () => {
    it("should never exceed 1.0", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "const x = 1;",
        language: "typescript",
      };

      const result = evaluator.evaluate(diff);

      expect(result.minimalityFactor).toBeLessThanOrEqual(1.0);
    });

    it("should never go below 0.1", () => {
      const before = "const x = 1;";
      const after = Array.from(
        { length: 1000 },
        (_, i) => `const x${i} = ${i};`
      ).join("\n");

      const diff: CodeDiff = {
        before,
        after,
        language: "typescript",
      };

      const result = evaluator.evaluate(diff);

      expect(result.minimalityFactor).toBeGreaterThanOrEqual(0.1);
    });
  });

  describe("Configuration", () => {
    it("should allow custom config", () => {
      const customEval = new MinimalDiffEvaluator({
        minMinimalityFactor: 0.2,
        maxMinimalityFactor: 0.9,
      });

      const config = customEval.getConfig();

      expect(config.minMinimalityFactor).toBe(0.2);
      expect(config.maxMinimalityFactor).toBe(0.9);
    });

    it("should allow config updates", () => {
      evaluator.updateConfig({
        enableScaffoldingDetection: false,
      });

      const config = evaluator.getConfig();

      expect(config.enableScaffoldingDetection).toBe(false);
    });
  });

  describe("Quality Assessment", () => {
    it("should assess quality as minimal for high scores", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "const x = 2;",
        language: "typescript",
      };

      const result = evaluator.evaluate(diff);

      if (result.minimalityFactor >= 0.8) {
        expect(result.qualityAssessment).toBe("minimal");
      }
    });

    it("should assess quality as moderate for medium scores", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "const x = 1;\nconst y = 2;\nconst z = 3;",
        language: "typescript",
      };

      const result = evaluator.evaluate(diff);

      if (result.minimalityFactor >= 0.5 && result.minimalityFactor < 0.8) {
        expect(result.qualityAssessment).toBe("moderate");
      }
    });
  });
});
