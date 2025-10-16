/**
 * @fileoverview Research Detector for ARBITER-006 Phase 4
 *
 * Detects when tasks require research using multiple heuristics:
 * - Question detection
 * - Uncertainty keywords
 * - Fact-checking requirements
 * - Comparison needs
 * - Technical information needs
 *
 * @author @darianrosebrook
 */

import { Task, TaskType } from "../../types/arbiter-orchestration";
import { QueryType } from "../../types/knowledge";

/**
 * Research requirement detection result
 */
export interface ResearchRequirement {
  /** Whether research is required */
  required: boolean;

  /** Confidence score (0-1) */
  confidence: number;

  /** Inferred query type */
  queryType: QueryType;

  /** Suggested research queries */
  suggestedQueries: string[];

  /** Indicators that triggered detection */
  indicators: {
    hasQuestions: boolean;
    hasUncertainty: boolean;
    requiresFactChecking: boolean;
    needsComparison: boolean;
    requiresTechnicalInfo: boolean;
  };

  /** Reason for research requirement */
  reason?: string;
}

/**
 * Research Detector Configuration
 */
export interface ResearchDetectorConfig {
  /** Minimum confidence threshold to trigger research */
  minConfidence?: number;

  /** Maximum queries to generate */
  maxQueries?: number;

  /** Enable question detection */
  enableQuestionDetection?: boolean;

  /** Enable uncertainty detection */
  enableUncertaintyDetection?: boolean;

  /** Enable technical detection */
  enableTechnicalDetection?: boolean;
}

/**
 * Research Detector
 *
 * Uses multiple heuristics to detect when tasks require research.
 * Generates suggested queries and confidence scores.
 */
export class ResearchDetector {
  private config: Required<ResearchDetectorConfig>;

  constructor(config: ResearchDetectorConfig = {}) {
    this.config = {
      minConfidence: config.minConfidence ?? 0.7,
      maxQueries: config.maxQueries ?? 3,
      enableQuestionDetection: config.enableQuestionDetection ?? true,
      enableUncertaintyDetection: config.enableUncertaintyDetection ?? true,
      enableTechnicalDetection: config.enableTechnicalDetection ?? true,
    };
  }

  /**
   * Detect if a task requires research
   */
  detectResearchNeeds(task: Task): ResearchRequirement | null {
    const text = `${task.description} ${task.metadata?.prompt || ""}`.trim();

    // Early return for empty or whitespace-only text
    if (!text || text.length === 0) {
      return null;
    }

    // Calculate indicators
    const indicators = {
      hasQuestions: this.config.enableQuestionDetection
        ? this.containsQuestions(text)
        : false,
      hasUncertainty: this.config.enableUncertaintyDetection
        ? this.containsUncertaintyKeywords(text)
        : false,
      requiresFactChecking: this.requiresFactChecking(task.type),
      needsComparison: this.needsComparison(text),
      requiresTechnicalInfo: this.config.enableTechnicalDetection
        ? this.requiresTechnicalInfo(text)
        : false,
    };

    // Calculate confidence score
    const confidence = this.calculateResearchConfidence(indicators);

    // Check if research is required
    if (confidence < this.config.minConfidence) {
      return null;
    }

    // Infer query type
    const queryType = this.inferQueryType(indicators, text);

    // Generate suggested queries
    const suggestedQueries = this.generateQueries(task, queryType, indicators);

    // Generate reason with confidence
    const reason = this.generateReason(indicators, confidence);

    return {
      required: true,
      confidence,
      queryType,
      suggestedQueries: suggestedQueries.slice(0, this.config.maxQueries),
      indicators,
      reason,
    };
  }

  /**
   * Check if text contains questions
   */
  private containsQuestions(text: string): boolean {
    // Question patterns - question words must be followed by content ending with ?
    const questionPatterns = [
      /\b(what|how|why|when|where|who|which)\b[^?]*\?/gi,
      /\bcan\s+(you|we|i)\b[^?]*\?/gi,
      /\bshould\s+(i|we)\b[^?]*\?/gi,
      /\bis\s+there\b[^?]*\?/gi,
      /\bare\s+there\b[^?]*\?/gi,
    ];

    return questionPatterns.some((pattern) => pattern.test(text));
  }

  /**
   * Check if text contains uncertainty keywords
   */
  private containsUncertaintyKeywords(text: string): boolean {
    const uncertaintyWords = [
      "not sure",
      "unclear",
      "unknown",
      "uncertain",
      "unsure",
      "don't know",
      "need to find",
      "need to research",
      "research",
      "need to investigate",
      "need information",
      "looking for",
      "trying to understand",
      "help me understand",
      "explain",
    ];

    const textLower = text.toLowerCase();
    return uncertaintyWords.some((word) => textLower.includes(word));
  }

  /**
   * Check if task type requires fact-checking
   */
  private requiresFactChecking(taskType: TaskType): boolean {
    const factCheckingTypes: TaskType[] = [
      "analysis",
      "research",
      "validation",
    ];

    return factCheckingTypes.includes(taskType);
  }

  /**
   * Check if text needs comparison
   */
  private needsComparison(text: string): boolean {
    const comparisonKeywords = [
      "compare",
      "comparison",
      "versus",
      "vs",
      "difference between",
      "similarities",
      "pros and cons",
      "advantages",
      "disadvantages",
      "better than",
      "worse than",
      "alternative",
      "options",
      "choose between",
    ];

    const textLower = text.toLowerCase();
    return comparisonKeywords.some((keyword) => textLower.includes(keyword));
  }

  /**
   * Check if text requires technical information
   */
  private requiresTechnicalInfo(text: string): boolean {
    const technicalKeywords = [
      "api",
      "library",
      "framework",
      "implementation",
      "algorithm",
      "documentation",
      "architecture",
      "integration",
      "best practices",
      "code example",
      "tutorial",
      "guide",
      "reference",
      "specification",
      "how to implement",
      "how to use",
      "configuration",
      "setup",
    ];

    const textLower = text.toLowerCase();
    return technicalKeywords.some((keyword) => textLower.includes(keyword));
  }

  /**
   * Calculate research confidence score based on indicators
   */
  private calculateResearchConfidence(indicators: {
    hasQuestions: boolean;
    hasUncertainty: boolean;
    requiresFactChecking: boolean;
    needsComparison: boolean;
    requiresTechnicalInfo: boolean;
  }): number {
    let score = 0;

    // Strong indicators (individually sufficient)
    if (indicators.hasQuestions) {
      score = Math.max(score, 0.9); // Questions alone = 90% confidence
    }
    if (indicators.hasUncertainty) {
      score = Math.max(score, 0.85); // Uncertainty alone = 85% confidence
    }
    if (indicators.needsComparison) {
      score = Math.max(score, 0.8); // Comparison alone = 80% confidence
    }

    // Weaker indicators (need combination)
    if (indicators.requiresTechnicalInfo) {
      score = Math.max(score, 0.5); // Technical alone = 50% confidence
    }
    if (indicators.requiresFactChecking) {
      score = Math.max(score, 0.4); // Fact-checking alone = 40% confidence
    }

    // Boost for combinations
    const activeCount = [
      indicators.hasQuestions,
      indicators.hasUncertainty,
      indicators.needsComparison,
      indicators.requiresTechnicalInfo,
      indicators.requiresFactChecking,
    ].filter(Boolean).length;

    if (activeCount >= 2) {
      score = Math.min(1.0, score + 0.1 * (activeCount - 1)); // +10% per additional indicator
    }

    return score;
  }

  /**
   * Infer query type from indicators and text
   */
  private inferQueryType(
    indicators: {
      hasQuestions: boolean;
      hasUncertainty: boolean;
      requiresFactChecking: boolean;
      needsComparison: boolean;
      requiresTechnicalInfo: boolean;
    },
    text: string
  ): QueryType {
    // Technical queries
    if (indicators.requiresTechnicalInfo) {
      return QueryType.TECHNICAL;
    }

    // Comparison queries
    if (indicators.needsComparison) {
      return QueryType.COMPARATIVE;
    }

    // Check for trending/time-sensitive keywords
    const trendKeywords = ["latest", "recent", "current", "new", "trending"];
    const textLower = text.toLowerCase();
    if (trendKeywords.some((keyword) => textLower.includes(keyword))) {
      return QueryType.TREND;
    }

    // Check for explanatory keywords
    const explanatoryKeywords = ["how", "why", "explain", "understand"];
    if (explanatoryKeywords.some((keyword) => textLower.includes(keyword))) {
      return QueryType.EXPLANATORY;
    }

    // Default to factual
    return QueryType.FACTUAL;
  }

  /**
   * Generate research queries from task
   */
  private generateQueries(
    task: Task,
    queryType: QueryType,
    indicators: {
      hasQuestions: boolean;
      hasUncertainty: boolean;
      requiresFactChecking: boolean;
      needsComparison: boolean;
      requiresTechnicalInfo: boolean;
    }
  ): string[] {
    const queries: string[] = [];
    const text = `${task.description} ${task.metadata?.prompt || ""}`;

    // Extract explicit questions from text
    const questionMatches = text.match(/([^.!?]*\?)/g);
    if (questionMatches && questionMatches.length > 0) {
      queries.push(...questionMatches.slice(0, 2).map((q) => q.trim()));
    }

    // Generate query from task description
    if (queries.length < this.config.maxQueries) {
      // Remove common filler words and create a concise query
      const mainQuery = this.extractMainQuery(task.description);
      if (mainQuery && !queries.includes(mainQuery)) {
        queries.push(mainQuery);
      }
    }

    // Add type-specific queries
    if (queries.length < this.config.maxQueries) {
      if (indicators.needsComparison) {
        queries.push(`Compare ${this.extractSubject(text)}`);
      } else if (indicators.requiresTechnicalInfo) {
        queries.push(`${this.extractSubject(text)} documentation`);
      }
    }

    return queries.filter((q) => q.length > 0);
  }

  /**
   * Extract main query from text
   */
  private extractMainQuery(text: string): string {
    // Remove common task prefixes
    let query = text
      .replace(/^(please|could you|can you|i need|we need|help me)/gi, "")
      .trim();

    // Take first sentence or up to 100 chars
    const firstSentence = query.split(/[.!?]/)[0];
    query =
      firstSentence.length > 100
        ? firstSentence.substring(0, 97) + "..."
        : firstSentence;

    return query.trim();
  }

  /**
   * Extract subject from text
   */
  private extractSubject(text: string): string {
    // Simple subject extraction - take first noun phrase
    const words = text.split(/\s+/);
    const subject = words.slice(0, Math.min(5, words.length)).join(" ");
    return subject;
  }

  /**
   * Generate reason for research requirement
   */
  private generateReason(
    indicators: {
      hasQuestions: boolean;
      hasUncertainty: boolean;
      requiresFactChecking: boolean;
      needsComparison: boolean;
      requiresTechnicalInfo: boolean;
    },
    confidence?: number
  ): string {
    const reasons: string[] = [];

    if (indicators.hasQuestions) {
      reasons.push("contains questions");
    }
    if (indicators.hasUncertainty) {
      reasons.push("expresses uncertainty");
    }
    if (indicators.needsComparison) {
      reasons.push("requires comparison");
    }
    if (indicators.requiresTechnicalInfo) {
      reasons.push("needs technical information");
    }
    if (indicators.requiresFactChecking) {
      reasons.push("requires fact-checking");
    }

    const baseReason =
      reasons.length > 0
        ? `Task ${reasons.join(", ")}`
        : "Task may benefit from research";

    // Include confidence if provided
    if (confidence !== undefined) {
      return `${baseReason} (confidence: ${(confidence * 100).toFixed(0)}%)`;
    }

    return baseReason;
  }
}
