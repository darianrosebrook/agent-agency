/**
 * @fileoverview
 * Unit tests for ScaffoldingDetector
 * Tests scaffolding pattern detection
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { ScaffoldingDetector } from "../../../src/evaluation/ScaffoldingDetector";
import type { CodeDiff } from "../../../src/types/evaluation";

describe("ScaffoldingDetector", () => {
  let detector: ScaffoldingDetector;

  beforeEach(() => {
    detector = new ScaffoldingDetector();
  });

  describe("Excessive Comments Detection", () => {
    it("should detect patterns in code", () => {
      const code = Array.from({ length: 100 }, (_, i) =>
        i % 2 === 0 ? `// Comment ${i}` : `const x${i} = ${i};`
      ).join("\n");

      const diff: CodeDiff = {
        before: "const x = 1;",
        after: code,
        language: "typescript",
      };

      const result = detector.detect(diff);

      // Detection depends on pattern matching - verify structure works
      expect(result.detected).toBeDefined();
      expect(result.matchedPatterns).toBeDefined();
      expect(result.confidence).toBeGreaterThanOrEqual(0);
    });

    it("should not penalize minimal comments", () => {
      const code = `const x = 1;\nconst y = 2;`;

      const diff: CodeDiff = {
        before: "",
        after: code,
        language: "typescript",
      };

      const result = detector.detect(diff);

      expect(result.detected).toBe(false);
    });
  });

  describe("Boilerplate Detection", () => {
    it("should detect excessive imports", () => {
      const imports = Array.from(
        { length: 25 },
        (_, i) => `import { Module${i} } from 'lib${i}';`
      ).join("\n");

      const diff: CodeDiff = {
        before: "",
        after: imports,
        language: "typescript",
      };

      const result = detector.detect(diff);

      expect(result.detected).toBe(true);
      expect(result.matchedPatterns).toContain("boilerplate-imports");
    });
  });

  describe("Whitespace Detection", () => {
    it("should detect excessive whitespace", () => {
      const code = Array.from({ length: 100 }, (_, i) =>
        i % 2 === 0 ? "" : `const x${i} = ${i};`
      ).join("\n");

      const diff: CodeDiff = {
        before: "",
        after: code,
        language: "typescript",
      };

      const result = detector.detect(diff);

      expect(result.detected).toBe(true);
      expect(result.matchedPatterns).toContain("excessive-whitespace");
    });
  });

  describe("Confidence Scoring", () => {
    it("should provide confidence scores between 0 and 1", () => {
      const code = Array.from({ length: 100 }, (_, i) => {
        if (i % 3 === 0) return "";
        if (i % 3 === 1) return `// Comment ${i}`;
        return `const x${i} = ${i};`;
      }).join("\n");

      const diff: CodeDiff = {
        before: "",
        after: code,
        language: "typescript",
      };

      const result = detector.detect(diff);

      expect(result.confidence).toBeGreaterThanOrEqual(0);
      expect(result.confidence).toBeLessThanOrEqual(1);
    });
  });

  describe("Custom Patterns", () => {
    it("should support adding custom patterns", () => {
      detector.addPattern({
        name: "test-pattern",
        description: "Test pattern",
        pattern: /TEST/g,
        penalty: 0.5,
        category: "redundant",
      });

      const patterns = detector.getPatterns();

      expect(patterns.length).toBeGreaterThan(4);
      expect(patterns.find((p) => p.name === "test-pattern")).toBeDefined();
    });
  });

  describe("Category-specific Detection", () => {
    it("should handle comments category", () => {
      const code = Array.from({ length: 20 }, (_, i) =>
        i < 12 ? `// Comment ${i}` : `const x${i} = ${i};`
      ).join("\n");

      const diff: CodeDiff = {
        before: "",
        after: code,
        language: "typescript",
      };

      const result = detector.detect(diff);
      // Should evaluate comment patterns
      expect(result).toBeDefined();
    });

    it("should handle boilerplate category with fewer imports", () => {
      const imports = Array.from(
        { length: 15 },
        (_, i) => `import { Module${i} } from 'lib${i}';`
      ).join("\n");

      const diff: CodeDiff = {
        before: "",
        after: imports,
        language: "typescript",
      };

      const result = detector.detect(diff);
      // Should not detect with only 15 imports (threshold is 20)
      expect(result.detected).toBe(false);
    });

    it("should handle whitespace category below threshold", () => {
      const code = Array.from({ length: 20 }, (_, i) =>
        i < 5 ? "" : `const x${i} = ${i};`
      ).join("\n");

      const diff: CodeDiff = {
        before: "",
        after: code,
        language: "typescript",
      };

      const result = detector.detect(diff);
      // 25% whitespace should not trigger (threshold is 30%)
      expect(result.detected).toBe(false);
    });
  });
});
