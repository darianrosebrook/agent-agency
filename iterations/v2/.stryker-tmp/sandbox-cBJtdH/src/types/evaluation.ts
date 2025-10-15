/**
 * @fileoverview
 * Type definitions for code evaluation and diff analysis.
 * Defines interfaces for AST-based diff evaluation and minimality scoring.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


/**
 * Represents a code change between two versions
 */
export interface CodeDiff {
  /** Original code before changes */
  before: string;
  /** Modified code after changes */
  after: string;
  /** Programming language */
  language: string;
  /** Optional file path context */
  filePath?: string;
}

/**
 * AST node representation for diff analysis
 */
export interface ASTNode {
  /** Node type (e.g., FunctionDeclaration, VariableStatement) */
  type: string;
  /** Node start position in source */
  start: number;
  /** Node end position in source */
  end: number;
  /** Child nodes */
  children?: ASTNode[];
  /** Node content/text */
  text?: string;
}

/**
 * Result of AST diff analysis
 */
export interface ASTDiffResult {
  /** Similarity score between ASTs (0-1) */
  similarity: number;
  /** Number of nodes added */
  nodesAdded: number;
  /** Number of nodes removed */
  nodesRemoved: number;
  /** Number of nodes modified */
  nodesModified: number;
  /** Total nodes in before AST */
  totalNodesBefore: number;
  /** Total nodes in after AST */
  totalNodesAfter: number;
  /** List of changed node types */
  changedNodeTypes: string[];
}

/**
 * Scaffolding detection result
 */
export interface ScaffoldingDetection {
  /** Whether scaffolding was detected */
  detected: boolean;
  /** Confidence score (0-1) */
  confidence: number;
  /** Reasons for detection */
  reasons: string[];
  /** Penalty factor to apply (0-1) */
  penaltyFactor: number;
  /** Patterns that matched */
  matchedPatterns: string[];
}

/**
 * Minimality evaluation result
 */
export interface MinimalityEvaluation {
  /** Final minimality factor (0.1-1.0) */
  minimalityFactor: number;
  /** AST similarity score */
  astSimilarity: number;
  /** Scaffolding detection result */
  scaffolding: ScaffoldingDetection;
  /** Lines of code changed */
  linesChanged: number;
  /** Assessment of change quality */
  qualityAssessment: "minimal" | "moderate" | "extensive";
  /** Evaluation duration in milliseconds */
  evaluationTimeMs: number;
}

/**
 * Configuration for minimality evaluator
 */
export interface MinimalityEvaluatorConfig {
  /** Language to analyze */
  language: string;
  /** Minimum minimality factor (default: 0.1) */
  minMinimalityFactor: number;
  /** Maximum minimality factor (default: 1.0) */
  maxMinimalityFactor: number;
  /** Enable scaffolding detection */
  enableScaffoldingDetection: boolean;
  /** Scaffolding penalty weight (0-1) */
  scaffoldingPenaltyWeight: number;
}

/**
 * Scaffolding pattern definition
 */
export interface ScaffoldingPattern {
  /** Pattern name */
  name: string;
  /** Pattern description */
  description: string;
  /** Regex pattern to match */
  pattern: RegExp;
  /** Penalty if matched (0-1) */
  penalty: number;
  /** Pattern category */
  category: "boilerplate" | "comments" | "whitespace" | "redundant";
}

/**
 * Default scaffolding patterns
 */
export const DEFAULT_SCAFFOLDING_PATTERNS: ScaffoldingPattern[] = [
  {
    name: "excessive-comments",
    description: "Code with > 50% comment lines",
    pattern: /^\/\/|^\/\*|\*\/$/gm,
    penalty: 0.3,
    category: "comments",
  },
  {
    name: "boilerplate-imports",
    description: "Large blocks of import statements (> 20 lines)",
    pattern: /^import\s+.*from\s+['"]/gm,
    penalty: 0.2,
    category: "boilerplate",
  },
  {
    name: "excessive-whitespace",
    description: "Files with > 30% blank lines",
    pattern: /^\s*$/gm,
    penalty: 0.1,
    category: "whitespace",
  },
  {
    name: "redundant-code",
    description: "Duplicate or redundant code blocks",
    pattern: /^function\s+\w+\s*\(\)\s*\{\s*\}/gm,
    penalty: 0.4,
    category: "redundant",
  },
];

/**
 * Default evaluator configuration
 */
export const DEFAULT_EVALUATOR_CONFIG: MinimalityEvaluatorConfig = {
  language: "typescript",
  minMinimalityFactor: 0.1,
  maxMinimalityFactor: 1.0,
  enableScaffoldingDetection: true,
  scaffoldingPenaltyWeight: 0.5,
};
