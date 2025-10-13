/**
 * @fileoverview
 * Unit tests for ASTDiffAnalyzer
 * Tests AST parsing and diff analysis
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { ASTDiffAnalyzer } from "../../../src/evaluation/ASTDiffAnalyzer";
import type { CodeDiff } from "../../../src/types/evaluation";

describe("ASTDiffAnalyzer", () => {
  let analyzer: ASTDiffAnalyzer;

  beforeEach(() => {
    analyzer = new ASTDiffAnalyzer();
  });

  describe("TypeScript/JavaScript Analysis", () => {
    it("should detect minimal changes", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "const x = 2;",
        language: "typescript",
      };

      const result = analyzer.analyze(diff);

      expect(result.similarity).toBeGreaterThan(0.5);
      expect(result.similarity).toBeLessThanOrEqual(1.0);
      expect(result.totalNodesBefore).toBeGreaterThan(0);
      expect(result.totalNodesAfter).toBeGreaterThan(0);
    });

    it("should detect significant changes", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "function foo() { return 42; }",
        language: "typescript",
      };

      const result = analyzer.analyze(diff);

      expect(result.similarity).toBeLessThan(0.5);
      expect(result.changedNodeTypes.length).toBeGreaterThan(0);
    });

    it("should detect added nodes", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "const x = 1;\nconst y = 2;",
        language: "typescript",
      };

      const result = analyzer.analyze(diff);

      expect(result.nodesAdded).toBeGreaterThan(0);
      expect(result.totalNodesAfter).toBeGreaterThan(result.totalNodesBefore);
    });

    it("should detect removed nodes", () => {
      const diff: CodeDiff = {
        before: "const x = 1;\nconst y = 2;",
        after: "const x = 1;",
        language: "typescript",
      };

      const result = analyzer.analyze(diff);

      expect(result.nodesRemoved).toBeGreaterThan(0);
      expect(result.totalNodesAfter).toBeLessThan(result.totalNodesBefore);
    });

    it("should handle identical code", () => {
      const code = "function test() { return true; }";
      const diff: CodeDiff = {
        before: code,
        after: code,
        language: "typescript",
      };

      const result = analyzer.analyze(diff);

      expect(result.similarity).toBe(1.0);
      expect(result.nodesAdded).toBe(0);
      expect(result.nodesRemoved).toBe(0);
    });

    it("should handle complex TypeScript code", () => {
      const diff: CodeDiff = {
        before: `
          interface User {
            id: string;
            name: string;
          }
          function getUser(id: string): User {
            return { id, name: "Test" };
          }
        `,
        after: `
          interface User {
            id: string;
            name: string;
            email: string;
          }
          function getUser(id: string): User {
            return { id, name: "Test", email: "test@example.com" };
          }
        `,
        language: "typescript",
      };

      const result = analyzer.analyze(diff);

      expect(result.similarity).toBeGreaterThan(0.7);
      expect(result.nodesAdded).toBeGreaterThan(0);
    });
  });

  describe("Lines Changed Calculation", () => {
    it("should calculate lines added", () => {
      const diff: CodeDiff = {
        before: "line1\nline2",
        after: "line1\nline2\nline3",
        language: "typescript",
      };

      const linesChanged = analyzer.calculateLinesChanged(diff);

      expect(linesChanged).toBe(1);
    });

    it("should calculate lines removed", () => {
      const diff: CodeDiff = {
        before: "line1\nline2\nline3",
        after: "line1\nline2",
        language: "typescript",
      };

      const linesChanged = analyzer.calculateLinesChanged(diff);

      expect(linesChanged).toBe(1);
    });

    it("should handle no changes", () => {
      const diff: CodeDiff = {
        before: "line1\nline2",
        after: "line1\nline2",
        language: "typescript",
      };

      const linesChanged = analyzer.calculateLinesChanged(diff);

      expect(linesChanged).toBe(0);
    });

    it("should handle large diffs", () => {
      const beforeLines = Array.from(
        { length: 100 },
        (_, i) => `line${i}`
      ).join("\n");
      const afterLines = Array.from({ length: 200 }, (_, i) => `line${i}`).join(
        "\n"
      );

      const diff: CodeDiff = {
        before: beforeLines,
        after: afterLines,
        language: "typescript",
      };

      const linesChanged = analyzer.calculateLinesChanged(diff);

      expect(linesChanged).toBe(100);
    });
  });

  describe("Performance", () => {
    it("should analyze small diff within 200ms", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "const x = 2;",
        language: "typescript",
      };

      const startTime = Date.now();
      analyzer.analyze(diff);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(200);
    });

    it("should analyze moderate diff within 200ms", () => {
      const diff: CodeDiff = {
        before: Array.from(
          { length: 50 },
          (_, i) => `const x${i} = ${i};`
        ).join("\n"),
        after: Array.from(
          { length: 50 },
          (_, i) => `const x${i} = ${i + 1};`
        ).join("\n"),
        language: "typescript",
      };

      const startTime = Date.now();
      analyzer.analyze(diff);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(200);
    });
  });

  describe("Edge Cases", () => {
    it("should handle empty code", () => {
      const diff: CodeDiff = {
        before: "",
        after: "const x = 1;",
        language: "typescript",
      };

      const result = analyzer.analyze(diff);

      expect(result.nodesAdded).toBeGreaterThan(0);
      expect(result.totalNodesBefore).toBeGreaterThanOrEqual(0);
    });

    it("should handle code to empty", () => {
      const diff: CodeDiff = {
        before: "const x = 1;",
        after: "",
        language: "typescript",
      };

      const result = analyzer.analyze(diff);

      expect(result.nodesRemoved).toBeGreaterThan(0);
    });

    it("should handle malformed code gracefully", () => {
      const diff: CodeDiff = {
        before: "const x =",
        after: "const x = 1;",
        language: "typescript",
      };

      // Should not throw
      expect(() => analyzer.analyze(diff)).not.toThrow();
    });

    it("should handle unsupported languages with generic parser", () => {
      const diff: CodeDiff = {
        before: "line1\nline2",
        after: "line1\nline2\nline3",
        language: "python",
      };

      const result = analyzer.analyze(diff);

      expect(result.totalNodesBefore).toBeGreaterThan(0);
      expect(result.totalNodesAfter).toBeGreaterThan(0);
      expect(result.similarity).toBeDefined();
    });
  });
});
