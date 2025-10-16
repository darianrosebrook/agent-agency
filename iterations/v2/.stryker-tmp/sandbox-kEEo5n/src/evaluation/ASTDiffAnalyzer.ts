/**
 * @fileoverview
 * AST-based diff analyzer for code changes.
 * Parses TypeScript/JavaScript code and computes similarity metrics.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import type { ASTDiffResult, ASTNode, CodeDiff } from "@/types/evaluation";
import * as ts from "typescript";

/**
 * Analyzes code diffs using AST parsing and comparison
 */
export class ASTDiffAnalyzer {
  /**
   * Analyzes a code diff and returns similarity metrics
   *
   * @param diff Code diff to analyze
   * @returns AST diff analysis result
   */
  analyze(diff: CodeDiff): ASTDiffResult {
    // Parse both versions into ASTs
    const beforeAST = this.parseToAST(diff.before, diff.language);
    const afterAST = this.parseToAST(diff.after, diff.language);

    // Compare ASTs and compute metrics
    return this.compareASTs(beforeAST, afterAST);
  }

  /**
   * Parses code string to AST representation
   *
   * @param code Source code to parse
   * @param language Programming language
   * @returns AST root node
   */
  private parseToAST(code: string, language: string): ASTNode {
    if (language === "typescript" || language === "javascript") {
      return this.parseTypeScript(code);
    }

    // For other languages, return a simple text-based representation
    return this.parseGeneric(code);
  }

  /**
   * Parses TypeScript/JavaScript code using TypeScript compiler
   *
   * @param code Source code
   * @returns AST root node
   */
  private parseTypeScript(code: string): ASTNode {
    const sourceFile = ts.createSourceFile(
      "temp.ts",
      code,
      ts.ScriptTarget.Latest,
      true
    );

    return this.convertTSNodeToAST(sourceFile);
  }

  /**
   * Converts TypeScript compiler node to our AST format
   *
   * @param node TypeScript node
   * @returns Our AST node format
   */
  private convertTSNodeToAST(node: ts.Node): ASTNode {
    const astNode: ASTNode = {
      type: ts.SyntaxKind[node.kind],
      start: node.getStart(),
      end: node.getEnd(),
      text: node.getText(),
      children: [],
    };

    node.forEachChild((child) => {
      astNode.children?.push(this.convertTSNodeToAST(child));
    });

    return astNode;
  }

  /**
   * Generic parser for unsupported languages (line-based)
   *
   * @param code Source code
   * @returns Generic AST representation
   */
  private parseGeneric(code: string): ASTNode {
    const lines = code.split("\n");

    return {
      type: "Program",
      start: 0,
      end: code.length,
      text: code,
      children: lines.map((line, index) => ({
        type: "Line",
        start: index,
        end: index + line.length,
        text: line,
      })),
    };
  }

  /**
   * Compares two ASTs and computes similarity metrics
   *
   * @param before Original AST
   * @param after Modified AST
   * @returns Diff analysis result
   */
  private compareASTs(before: ASTNode, after: ASTNode): ASTDiffResult {
    const beforeNodes = this.flattenAST(before);
    const afterNodes = this.flattenAST(after);

    // Calculate node changes
    const beforeNodeTypes = new Set(beforeNodes.map((n) => n.type));
    const afterNodeTypes = new Set(afterNodes.map((n) => n.type));

    const added = afterNodes.filter(
      (n) => !this.findMatchingNode(n, beforeNodes)
    );
    const removed = beforeNodes.filter(
      (n) => !this.findMatchingNode(n, afterNodes)
    );

    // Approximate modified nodes as nodes in both but with different text
    const common = afterNodes.filter((n) =>
      this.findMatchingNode(n, beforeNodes)
    );
    const modified = common.filter((n) => {
      const match = this.findMatchingNode(n, beforeNodes);
      return match && match.text !== n.text;
    });

    // Calculate similarity using Jaccard index
    const intersection = new Set(
      [...beforeNodeTypes].filter((x) => afterNodeTypes.has(x))
    ).size;
    const union = new Set([...beforeNodeTypes, ...afterNodeTypes]).size;
    const similarity = union > 0 ? intersection / union : 1.0;

    // Collect changed node types
    const changedNodeTypes = Array.from(
      new Set([
        ...added.map((n) => n.type),
        ...removed.map((n) => n.type),
        ...modified.map((n) => n.type),
      ])
    );

    return {
      similarity,
      nodesAdded: added.length,
      nodesRemoved: removed.length,
      nodesModified: modified.length,
      totalNodesBefore: beforeNodes.length,
      totalNodesAfter: afterNodes.length,
      changedNodeTypes,
    };
  }

  /**
   * Flattens AST tree into array of nodes
   *
   * @param node Root node
   * @returns Array of all nodes in tree
   */
  private flattenAST(node: ASTNode): ASTNode[] {
    const nodes: ASTNode[] = [node];

    if (node.children) {
      for (const child of node.children) {
        nodes.push(...this.flattenAST(child));
      }
    }

    return nodes;
  }

  /**
   * Finds a matching node in a node array
   *
   * @param needle Node to find
   * @param haystack Array to search
   * @returns Matching node or undefined
   */
  private findMatchingNode(
    needle: ASTNode,
    haystack: ASTNode[]
  ): ASTNode | undefined {
    return haystack.find(
      (n) => n.type === needle.type && n.text === needle.text
    );
  }

  /**
   * Calculates lines of code changed
   *
   * @param diff Code diff
   * @returns Number of lines changed
   */
  calculateLinesChanged(diff: CodeDiff): number {
    const beforeLines = diff.before.split("\n").length;
    const afterLines = diff.after.split("\n").length;

    return Math.abs(afterLines - beforeLines);
  }
}
