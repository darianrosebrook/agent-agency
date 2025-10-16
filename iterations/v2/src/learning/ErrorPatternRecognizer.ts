/**
 * Error Pattern Recognizer
 *
 * Analyzes errors to identify patterns, categorize failure modes,
 * and generate targeted remediation strategies for iterative learning.
 *
 * Target: 90% accuracy in error categorization
 *
 * @author @darianrosebrook
 */

import { createHash } from "crypto";
import { EventEmitter } from "events";
import type { LearningDatabaseClient } from "../database/LearningDatabaseClient.js";
import { Logger } from "../observability/Logger.js";
import type {
  ErrorPattern,
  LearningIteration,
} from "../types/learning-coordination.js";
import {
  ErrorCategory,
  LearningCoordinatorEvent,
} from "../types/learning-coordination.js";

/**
 * Error analysis result
 */
export interface ErrorAnalysisResult {
  category: ErrorCategory;
  confidence: number;
  remediationStrategy: string;
  patternId?: string;
  isKnownPattern: boolean;
}

/**
 * Error Pattern Recognizer
 *
 * Identifies error patterns and generates remediation strategies
 * using pattern matching and clustering techniques.
 */
export class ErrorPatternRecognizer extends EventEmitter {
  private knownPatterns: Map<string, ErrorPattern>;
  private dbClient: LearningDatabaseClient;
  private logger: Logger;

  // Regex patterns for common error types
  private static readonly ERROR_PATTERNS = {
    [ErrorCategory.SYNTAX_ERROR]: [
      /SyntaxError/i,
      /Unexpected token/i,
      /Unexpected identifier/i,
      /Missing/i,
      /Expected/i,
    ],
    [ErrorCategory.TYPE_ERROR]: [
      /TypeError/i,
      /is not a function/i,
      /Cannot read property/i,
      /Cannot access/i,
      /undefined is not/i,
      /null is not/i,
    ],
    [ErrorCategory.RUNTIME_ERROR]: [
      /ReferenceError/i,
      /is not defined/i,
      /RangeError/i,
      /Maximum call stack/i,
      /out of range/i,
    ],
    [ErrorCategory.VALIDATION_ERROR]: [
      /ValidationError/i,
      /Invalid/i,
      /does not match/i,
      /Failed to validate/i,
      /Schema error/i,
    ],
    [ErrorCategory.TIMEOUT_ERROR]: [
      /timeout/i,
      /timed out/i,
      /ETIMEDOUT/i,
      /Request timeout/i,
    ],
    [ErrorCategory.RESOURCE_ERROR]: [
      /ENOMEM/i,
      /out of memory/i,
      /ENOSPC/i,
      /disk full/i,
      /Resource exhausted/i,
    ],
    [ErrorCategory.DEPENDENCY_ERROR]: [
      /Cannot find module/i,
      /Module not found/i,
      /ENOENT/i,
      /Package not found/i,
    ],
    [ErrorCategory.CONFIGURATION_ERROR]: [
      /Configuration/i,
      /Invalid config/i,
      /Missing required/i,
      /Environment variable/i,
    ],
  };

  // Remediation strategies by category
  private static readonly REMEDIATION_STRATEGIES = {
    [ErrorCategory.SYNTAX_ERROR]:
      "Review syntax near error location. Check for missing brackets, quotes, or semicolons. Validate against language specification.",
    [ErrorCategory.TYPE_ERROR]:
      "Verify variable types and function signatures. Add type guards or null checks. Ensure object properties exist before accessing.",
    [ErrorCategory.RUNTIME_ERROR]:
      "Check variable initialization. Verify function calls and recursion depth. Review scope and closure usage.",
    [ErrorCategory.LOGIC_ERROR]:
      "Review algorithm logic and control flow. Add assertions and boundary checks. Verify edge case handling.",
    [ErrorCategory.RESOURCE_ERROR]:
      "Optimize resource usage. Implement cleanup and disposal. Add resource pooling or caching strategies.",
    [ErrorCategory.TIMEOUT_ERROR]:
      "Increase timeout limits or optimize performance. Implement retry logic with exponential backoff. Check for blocking operations.",
    [ErrorCategory.VALIDATION_ERROR]:
      "Review validation rules and input data. Add proper error handling. Sanitize and normalize inputs before validation.",
    [ErrorCategory.DEPENDENCY_ERROR]:
      "Verify dependencies are installed. Check import paths and module resolution. Review package.json and lock files.",
    [ErrorCategory.CONFIGURATION_ERROR]:
      "Review configuration files and environment variables. Validate config schema. Provide default values for optional settings.",
    [ErrorCategory.UNKNOWN]:
      "Analyze error stack trace and context. Search documentation and known issues. Add detailed logging for diagnosis.",
  };

  constructor(dbClient: LearningDatabaseClient) {
    super();
    this.dbClient = dbClient;
    this.knownPatterns = new Map();
    this.logger = new Logger("ErrorPatternRecognizer");
  }

  /**
   * Initialize by loading known patterns from database
   */
  async initialize(): Promise<void> {
    try {
      const patterns = await this.dbClient.getErrorPatterns();

      for (const pattern of patterns) {
        this.knownPatterns.set(pattern.patternId, pattern);
      }
    } catch (error) {
      // If database table doesn't exist yet, start with empty patterns
      // This is expected during initial setup or migration
      this.logger.warn(
        "Failed to load error patterns from database, starting with empty set",
        {
          error: error instanceof Error ? error.message : String(error),
        }
      );
    }
  }

  /**
   * Analyze error from iteration
   *
   * @param iteration - Learning iteration with error
   * @param errorMessage - Error message to analyze
   * @param stackTrace - Optional stack trace
   * @returns Error analysis result
   */
  async analyzeError(
    iteration: LearningIteration,
    errorMessage: string,
    stackTrace?: string
  ): Promise<ErrorAnalysisResult> {
    const fullContext = `${errorMessage}\n${stackTrace || ""}`;

    // Try to match against known patterns first
    const knownMatch = this.matchKnownPattern(fullContext);
    if (knownMatch) {
      await this.updatePatternFrequency(knownMatch.patternId);

      this.emit(LearningCoordinatorEvent.PATTERN_RECOGNIZED, {
        sessionId: iteration.sessionId,
        timestamp: new Date(),
        eventType: LearningCoordinatorEvent.PATTERN_RECOGNIZED,
        data: {
          patternId: knownMatch.patternId,
          category: knownMatch.category,
          confidence: knownMatch.confidence,
        },
      });

      return {
        category: knownMatch.category,
        confidence: knownMatch.confidence,
        remediationStrategy: knownMatch.remediationStrategy,
        patternId: knownMatch.patternId,
        isKnownPattern: true,
      };
    }

    // Categorize using regex patterns
    const category = this.categorizeError(fullContext);
    const confidence = 0.7; // Lower confidence for new patterns
    const remediationStrategy =
      ErrorPatternRecognizer.REMEDIATION_STRATEGIES[category];

    // Create new pattern
    const patternId = this.generatePatternId(errorMessage);
    const newPattern: ErrorPattern = {
      patternId,
      category,
      pattern: this.extractPattern(errorMessage),
      frequency: 1,
      confidence,
      detectedAt: new Date(),
      remediationStrategy,
      successRate: 0,
      examples: [errorMessage.substring(0, 200)],
    };

    // Save to database
    await this.dbClient.upsertErrorPattern(newPattern);
    this.knownPatterns.set(patternId, newPattern);

    this.emit(LearningCoordinatorEvent.ERROR_DETECTED, {
      sessionId: iteration.sessionId,
      timestamp: new Date(),
      eventType: LearningCoordinatorEvent.ERROR_DETECTED,
      data: {
        category,
        patternId,
        isNewPattern: true,
      },
    });

    return {
      category,
      confidence,
      remediationStrategy,
      patternId,
      isKnownPattern: false,
    };
  }

  /**
   * Match error against known patterns
   *
   * @param errorContext - Full error context
   * @returns Matching pattern or null
   */
  private matchKnownPattern(errorContext: string): ErrorPattern | null {
    let bestMatch: ErrorPattern | null = null;
    let highestScore = 0;

    for (const pattern of this.knownPatterns.values()) {
      const score = this.calculateSimilarity(errorContext, pattern.pattern);

      if (score > highestScore && score > 0.7) {
        highestScore = score;
        bestMatch = pattern;
      }
    }

    return bestMatch;
  }

  /**
   * Categorize error using regex patterns
   *
   * @param errorContext - Full error context
   * @returns Error category
   */
  private categorizeError(errorContext: string): ErrorCategory {
    for (const [category, patterns] of Object.entries(
      ErrorPatternRecognizer.ERROR_PATTERNS
    )) {
      for (const pattern of patterns) {
        if (pattern.test(errorContext)) {
          return category as ErrorCategory;
        }
      }
    }

    return ErrorCategory.UNKNOWN;
  }

  /**
   * Calculate similarity between error and pattern
   *
   * Uses simple Jaccard similarity on words
   *
   * @param error - Error message
   * @param pattern - Pattern string
   * @returns Similarity score 0-1
   */
  private calculateSimilarity(error: string, pattern: string): number {
    const errorWords = new Set(this.tokenize(error));
    const patternWords = new Set(this.tokenize(pattern));

    const intersection = new Set(
      [...errorWords].filter((x) => patternWords.has(x))
    );
    const union = new Set([...errorWords, ...patternWords]);

    return intersection.size / union.size;
  }

  /**
   * Tokenize string into words
   *
   * @param text - Text to tokenize
   * @returns Array of tokens
   */
  private tokenize(text: string): string[] {
    return text
      .toLowerCase()
      .replace(/[^a-z0-9\s]/g, " ")
      .split(/\s+/)
      .filter((word) => word.length > 2);
  }

  /**
   * Extract pattern from error message
   *
   * Removes specific values and paths to create generalized pattern
   *
   * @param errorMessage - Error message
   * @returns Generalized pattern
   */
  private extractPattern(errorMessage: string): string {
    return errorMessage
      .replace(/\d+/g, "NUM")
      .replace(/["'].*?["']/g, "STR")
      .replace(/\/[\w\/\-_.]+/g, "PATH")
      .replace(/0x[0-9a-f]+/gi, "HEX")
      .substring(0, 200);
  }

  /**
   * Generate unique pattern ID
   *
   * @param errorMessage - Error message
   * @returns Pattern ID
   */
  private generatePatternId(errorMessage: string): string {
    const pattern = this.extractPattern(errorMessage);
    const hash = createHash("md5")
      .update(pattern)
      .digest("hex")
      .substring(0, 8);
    return `pattern_${hash}`;
  }

  /**
   * Update pattern frequency when matched
   *
   * @param patternId - Pattern ID to update
   */
  private async updatePatternFrequency(patternId: string): Promise<void> {
    const pattern = this.knownPatterns.get(patternId);

    if (pattern) {
      pattern.frequency++;
      pattern.confidence = Math.min(0.95, pattern.confidence + 0.01);
      await this.dbClient.upsertErrorPattern(pattern);
    }
  }

  /**
   * Update pattern success rate after remediation
   *
   * @param patternId - Pattern ID
   * @param wasSuccessful - Whether remediation was successful
   */
  async updatePatternSuccess(
    patternId: string,
    wasSuccessful: boolean
  ): Promise<void> {
    const pattern = this.knownPatterns.get(patternId);

    if (pattern) {
      const totalAttempts = pattern.frequency;
      const successfulAttempts = Math.floor(
        pattern.successRate * totalAttempts
      );
      const newSuccessfulAttempts =
        successfulAttempts + (wasSuccessful ? 1 : 0);

      pattern.successRate = newSuccessfulAttempts / totalAttempts;
      await this.dbClient.upsertErrorPattern(pattern);
    }
  }

  /**
   * Get patterns by category
   *
   * @param category - Error category
   * @returns Array of patterns
   */
  async getPatternsByCategory(
    category: ErrorCategory
  ): Promise<ErrorPattern[]> {
    return this.dbClient.getErrorPatterns(category);
  }

  /**
   * Get most common patterns
   *
   * @param limit - Number of patterns to return
   * @returns Top patterns by frequency
   */
  getMostCommonPatterns(limit: number = 10): ErrorPattern[] {
    return Array.from(this.knownPatterns.values())
      .sort((a, b) => b.frequency - a.frequency)
      .slice(0, limit);
  }

  /**
   * Get pattern statistics
   *
   * @returns Pattern statistics
   */
  getStatistics(): {
    totalPatterns: number;
    byCategory: Record<string, number>;
    averageConfidence: number;
    averageSuccessRate: number;
  } {
    const byCategory: Record<string, number> = {};
    let totalConfidence = 0;
    let totalSuccessRate = 0;

    for (const pattern of this.knownPatterns.values()) {
      byCategory[pattern.category] = (byCategory[pattern.category] || 0) + 1;
      totalConfidence += pattern.confidence;
      totalSuccessRate += pattern.successRate;
    }

    const count = this.knownPatterns.size;

    return {
      totalPatterns: count,
      byCategory,
      averageConfidence: count > 0 ? totalConfidence / count : 0,
      averageSuccessRate: count > 0 ? totalSuccessRate / count : 0,
    };
  }
}
