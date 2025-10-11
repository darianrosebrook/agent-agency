/**
 * Error Pattern Analyzer
 *
 * Analyzes task failures to identify patterns and generate adaptive improvements
 * for prompt engineering and error prevention.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { Logger } from "../utils/Logger.js";
import { MultiTenantMemoryManager } from "../memory/MultiTenantMemoryManager.js";

export interface ErrorPattern {
  id: string;
  pattern: string;
  category: ErrorCategory;
  frequency: number;
  confidence: number;
  commonCauses: string[];
  preventionStrategies: string[];
  adaptivePrompts: string[];
  affectedTaskTypes: string[];
  firstSeen: Date;
  lastSeen: Date;
  impact: {
    failureRate: number;
    averageSeverity: number;
    affectedTasks: number;
  };
}

export interface FailureAnalysis {
  taskId: string;
  taskType: string;
  error: string;
  context: Record<string, any>;
  severity: "low" | "medium" | "high" | "critical";
  patterns: ErrorPattern[];
  recommendations: string[];
  adaptivePrompt: string;
  confidence: number;
}

export interface AdaptivePrompt {
  originalPrompt: string;
  enhancedPrompt: string;
  errorPatterns: string[];
  improvements: string[];
  confidence: number;
  generated: Date;
  taskType: string;
}

export enum ErrorCategory {
  SYNTAX = "syntax",
  LOGIC = "logic",
  CONTEXT = "context",
  RESOURCE = "resource",
  TIMEOUT = "timeout",
  VALIDATION = "validation",
  DEPENDENCY = "dependency",
  CONFIGURATION = "configuration",
  NETWORK = "network",
  UNKNOWN = "unknown",
}

export class ErrorPatternAnalyzer extends EventEmitter {
  private logger: Logger;
  private memoryManager?: MultiTenantMemoryManager;
  private patterns = new Map<string, ErrorPattern>();
  private recentFailures: FailureAnalysis[] = [];
  private adaptivePrompts = new Map<string, AdaptivePrompt[]>();

  // Pattern recognition thresholds
  private readonly MIN_PATTERN_OCCURRENCES = 3;
  private readonly PATTERN_SIMILARITY_THRESHOLD = 0.7;
  private readonly MAX_RECENT_FAILURES = 100;

  constructor(memoryManager?: MultiTenantMemoryManager) {
    super();
    this.logger = new Logger("ErrorPatternAnalyzer");
    this.memoryManager = memoryManager;
  }

  /**
   * Analyze a task failure and identify patterns
   */
  async analyzeFailure(
    taskId: string,
    taskType: string,
    error: string,
    context: Record<string, any>,
    tenantId: string
  ): Promise<FailureAnalysis> {
    const analysis: FailureAnalysis = {
      taskId,
      taskType,
      error,
      context,
      severity: this.determineSeverity(error, context),
      patterns: [],
      recommendations: [],
      adaptivePrompt: "",
      confidence: 0,
    };

    try {
      // Extract error patterns
      analysis.patterns = await this.identifyPatterns(error, taskType, tenantId);

      // Generate recommendations
      analysis.recommendations = this.generateRecommendations(analysis.patterns, taskType);

      // Create adaptive prompt
      analysis.adaptivePrompt = await this.generateAdaptivePrompt(taskType, analysis.patterns, tenantId);

      // Update confidence based on pattern matches
      analysis.confidence = analysis.patterns.length > 0 ?
        Math.min(1.0, analysis.patterns.reduce((sum, p) => sum + p.confidence, 0) / analysis.patterns.length) :
        0.3;

      // Store analysis
      this.storeFailureAnalysis(analysis, tenantId);

      this.logger.info(`Failure analyzed for task ${taskId}: ${analysis.patterns.length} patterns identified`);

      this.emit("failure-analyzed", analysis);

      return analysis;
    } catch (analysisError) {
      this.logger.error(`Error pattern analysis failed for ${taskId}:`, analysisError);
      return analysis; // Return basic analysis even if advanced analysis fails
    }
  }

  /**
   * Identify error patterns from historical data
   */
  private async identifyPatterns(
    error: string,
    taskType: string,
    tenantId: string
  ): Promise<ErrorPattern[]> {
    const matches: ErrorPattern[] = [];

    // Check existing patterns
    for (const pattern of this.patterns.values()) {
      if (pattern.affectedTaskTypes.includes(taskType) &&
          this.patternMatchesError(pattern, error)) {
        matches.push(pattern);
      }
    }

    // Query memory for similar failures if memory is available
    if (this.memoryManager) {
      const similarFailures = await this.querySimilarFailures(error, taskType, tenantId);
      for (const failure of similarFailures) {
        const pattern = this.extractPatternFromFailure(failure, taskType);
        if (pattern) {
          matches.push(pattern);
        }
      }
    }

    // Update pattern frequencies
    matches.forEach(pattern => {
      pattern.frequency++;
      pattern.lastSeen = new Date();
      if (pattern.affectedTaskTypes.indexOf(taskType) === -1) {
        pattern.affectedTaskTypes.push(taskType);
      }
    });

    return matches.slice(0, 5); // Return top 5 matches
  }

  /**
   * Check if a pattern matches the current error
   */
  private patternMatchesError(pattern: ErrorPattern, error: string): boolean {
    const errorLower = error.toLowerCase();
    const patternLower = pattern.pattern.toLowerCase();

    // Exact pattern match
    if (errorLower.includes(patternLower)) {
      return true;
    }

    // Fuzzy matching for similar errors
    return this.calculateSimilarity(errorLower, patternLower) >= this.PATTERN_SIMILARITY_THRESHOLD;
  }

  /**
   * Calculate string similarity (simple implementation)
   */
  private calculateSimilarity(str1: string, str2: string): number {
    const longer = str1.length > str2.length ? str1 : str2;
    const shorter = str1.length > str2.length ? str2 : str1;

    if (longer.length === 0) return 1.0;

    const editDistance = this.levenshteinDistance(longer, shorter);
    return (longer.length - editDistance) / longer.length;
  }

  /**
   * Levenshtein distance for fuzzy matching
   */
  private levenshteinDistance(str1: string, str2: string): number {
    const matrix = Array(str2.length + 1).fill(null).map(() => Array(str1.length + 1).fill(null));

    for (let i = 0; i <= str1.length; i++) matrix[0][i] = i;
    for (let j = 0; j <= str2.length; j++) matrix[j][0] = j;

    for (let j = 1; j <= str2.length; j++) {
      for (let i = 1; i <= str1.length; i++) {
        const indicator = str1[i - 1] === str2[j - 1] ? 0 : 1;
        matrix[j][i] = Math.min(
          matrix[j][i - 1] + 1,     // deletion
          matrix[j - 1][i] + 1,     // insertion
          matrix[j - 1][i - 1] + indicator, // substitution
        );
      }
    }

    return matrix[str2.length][str1.length];
  }

  /**
   * Query memory for similar failures
   */
  private async querySimilarFailures(
    error: string,
    taskType: string,
    tenantId: string
  ): Promise<any[]> {
    if (!this.memoryManager) return [];

    try {
      const query = {
        type: "error_analysis",
        description: `Similar failures for ${taskType} tasks`,
        requirements: ["failure_analysis", "error_patterns"],
        constraints: {
          taskType,
          error: error.substring(0, 100), // First 100 chars for similarity
        },
      };

      const memories = await this.memoryManager.getContextualMemories(tenantId, query, {
        limit: 10,
        minRelevance: 0.6,
      });

      return memories.success ? memories.data || [] : [];
    } catch (error) {
      this.logger.warn("Failed to query similar failures:", error);
      return [];
    }
  }

  /**
   * Extract pattern from failure data
   */
  private extractPatternFromFailure(failure: any, taskType: string): ErrorPattern | null {
    if (!failure.content || !failure.content.error) return null;

    const error = failure.content.error;
    const category = this.categorizeError(error);

    const patternId = `pattern-${category}-${Date.now()}`;

    return {
      id: patternId,
      pattern: this.extractErrorPattern(error),
      category,
      frequency: 1,
      confidence: 0.6,
      commonCauses: this.identifyCommonCauses(error, category),
      preventionStrategies: this.generatePreventionStrategies(category),
      adaptivePrompts: [],
      affectedTaskTypes: [taskType],
      firstSeen: new Date(),
      lastSeen: new Date(),
      impact: {
        failureRate: 0.1,
        averageSeverity: this.getSeverityScore(failure.content.severity || "medium"),
        affectedTasks: 1,
      },
    };
  }

  /**
   * Categorize error type
   */
  private categorizeError(error: string): ErrorCategory {
    const errorLower = error.toLowerCase();

    if (errorLower.includes("syntax") || errorLower.includes("parse") || errorLower.includes("unexpected token")) {
      return ErrorCategory.SYNTAX;
    }
    if (errorLower.includes("logic") || errorLower.includes("algorithm") || errorLower.includes("incorrect")) {
      return ErrorCategory.LOGIC;
    }
    if (errorLower.includes("context") || errorLower.includes("memory") || errorLower.includes("undefined")) {
      return ErrorCategory.CONTEXT;
    }
    if (errorLower.includes("resource") || errorLower.includes("memory") || errorLower.includes("disk")) {
      return ErrorCategory.RESOURCE;
    }
    if (errorLower.includes("timeout") || errorLower.includes("timed out")) {
      return ErrorCategory.TIMEOUT;
    }
    if (errorLower.includes("validation") || errorLower.includes("invalid") || errorLower.includes("required")) {
      return ErrorCategory.VALIDATION;
    }
    if (errorLower.includes("dependency") || errorLower.includes("import") || errorLower.includes("module")) {
      return ErrorCategory.DEPENDENCY;
    }
    if (errorLower.includes("config") || errorLower.includes("setting") || errorLower.includes("parameter")) {
      return ErrorCategory.CONFIGURATION;
    }
    if (errorLower.includes("network") || errorLower.includes("connection") || errorLower.includes("http")) {
      return ErrorCategory.NETWORK;
    }

    return ErrorCategory.UNKNOWN;
  }

  /**
   * Extract core error pattern
   */
  private extractErrorPattern(error: string): string {
    // Simple pattern extraction - in practice, this would use NLP
    const words = error.split(/\s+/).filter(word => word.length > 3);
    return words.slice(0, 5).join(" ");
  }

  /**
   * Identify common causes for error category
   */
  private identifyCommonCauses(error: string, category: ErrorCategory): string[] {
    const causes: string[] = [];

    switch (category) {
      case ErrorCategory.SYNTAX:
        causes.push("Incorrect language syntax", "Missing semicolons/brackets", "Invalid variable names");
        break;
      case ErrorCategory.LOGIC:
        causes.push("Incorrect algorithm implementation", "Wrong conditional logic", "Off-by-one errors");
        break;
      case ErrorCategory.CONTEXT:
        causes.push("Undefined variables", "Incorrect context usage", "Memory leaks");
        break;
      case ErrorCategory.TIMEOUT:
        causes.push("Long-running operations", "Infinite loops", "Network delays");
        break;
      case ErrorCategory.VALIDATION:
        causes.push("Missing required fields", "Invalid data formats", "Constraint violations");
        break;
      default:
        causes.push("Unknown cause - requires investigation");
    }

    return causes;
  }

  /**
   * Generate prevention strategies
   */
  private generatePreventionStrategies(category: ErrorCategory): string[] {
    const strategies: string[] = [];

    switch (category) {
      case ErrorCategory.SYNTAX:
        strategies.push("Use syntax highlighting and linters", "Follow language style guides", "Test compilation frequently");
        break;
      case ErrorCategory.LOGIC:
        strategies.push("Write comprehensive unit tests", "Use code reviews", "Implement logging for debugging");
        break;
      case ErrorCategory.CONTEXT:
        strategies.push("Initialize variables properly", "Use strict mode", "Implement context validation");
        break;
      case ErrorCategory.TIMEOUT:
        strategies.push("Set reasonable timeouts", "Implement progress monitoring", "Use asynchronous processing");
        break;
      case ErrorCategory.VALIDATION:
        strategies.push("Implement input validation", "Use schema validation", "Add comprehensive error handling");
        break;
      default:
        strategies.push("Implement comprehensive error handling", "Add logging and monitoring", "Conduct thorough testing");
    }

    return strategies;
  }

  /**
   * Generate recommendations based on identified patterns
   */
  private generateRecommendations(patterns: ErrorPattern[], taskType: string): string[] {
    const recommendations: string[] = [];

    // Aggregate recommendations from all patterns
    const allStrategies = patterns.flatMap(p => p.preventionStrategies);
    const uniqueStrategies = [...new Set(allStrategies)];

    recommendations.push(...uniqueStrategies.slice(0, 3));

    // Add task-specific recommendations
    if (taskType === "code-generation") {
      recommendations.push("Include syntax validation in prompts");
      recommendations.push("Specify language-specific requirements clearly");
    } else if (taskType === "text-transformation") {
      recommendations.push("Define clear transformation rules");
      recommendations.push("Provide examples of desired output format");
    }

    return recommendations;
  }

  /**
   * Generate adaptive prompt based on error patterns
   */
  private async generateAdaptivePrompt(
    taskType: string,
    patterns: ErrorPattern[],
    tenantId: string
  ): Promise<string> {
    let prompt = "";

    // Add prevention instructions based on patterns
    if (patterns.length > 0) {
      prompt += "IMPORTANT: Avoid these common errors:\n";
      patterns.slice(0, 3).forEach(pattern => {
        prompt += `- ${pattern.commonCauses[0]}\n`;
      });
      prompt += "\n";
    }

    // Add task-specific adaptive instructions
    if (taskType === "code-generation") {
      prompt += "When generating code:\n";
      prompt += "- Ensure proper syntax and formatting\n";
      prompt += "- Include error handling where appropriate\n";
      prompt += "- Follow language-specific conventions\n";
      prompt += "- Test the code for common edge cases\n";
    } else if (taskType === "text-transformation") {
      prompt += "When transforming text:\n";
      prompt += "- Maintain the original meaning and intent\n";
      prompt += "- Preserve important formatting and structure\n";
      prompt += "- Avoid introducing banned phrases or terms\n";
      prompt += "- Ensure the output meets all specified requirements\n";
    }

    // Query memory for successful prompts if available
    if (this.memoryManager) {
      const successfulPrompts = await this.querySuccessfulPrompts(taskType, tenantId);
      if (successfulPrompts.length > 0) {
        prompt += "\nReference successful approaches:\n";
        successfulPrompts.slice(0, 2).forEach(success => {
          prompt += `- ${success.content.prompt?.substring(0, 100)}...\n`;
        });
      }
    }

    return prompt;
  }

  /**
   * Query successful prompts from memory
   */
  private async querySuccessfulPrompts(taskType: string, tenantId: string): Promise<any[]> {
    if (!this.memoryManager) return [];

    try {
      const query = {
        type: "successful_prompts",
        description: `Successful prompts for ${taskType} tasks`,
        requirements: ["successful_execution", "prompt_analysis"],
        constraints: { taskType, outcome: "success" },
      };

      const memories = await this.memoryManager.getContextualMemories(tenantId, query, {
        limit: 5,
        minRelevance: 0.7,
      });

      return memories.success ? memories.data || [] : [];
    } catch (error) {
      this.logger.warn("Failed to query successful prompts:", error);
      return [];
    }
  }

  /**
   * Determine error severity
   */
  private determineSeverity(error: string, context: Record<string, any>): "low" | "medium" | "high" | "critical" {
    const errorLower = error.toLowerCase();

    // Critical errors
    if (errorLower.includes("security") || errorLower.includes("breach") ||
        errorLower.includes("critical") || errorLower.includes("system failure")) {
      return "critical";
    }

    // High severity
    if (errorLower.includes("timeout") || errorLower.includes("crash") ||
        errorLower.includes("exception") || errorLower.includes("fatal")) {
      return "high";
    }

    // Medium severity
    if (errorLower.includes("error") || errorLower.includes("warning") ||
        errorLower.includes("invalid") || errorLower.includes("failed")) {
      return "medium";
    }

    return "low";
  }

  /**
   * Get severity score (1-10)
   */
  private getSeverityScore(severity: string): number {
    switch (severity) {
      case "critical": return 10;
      case "high": return 7;
      case "medium": return 4;
      case "low": return 1;
      default: return 4;
    }
  }

  /**
   * Store failure analysis in memory
   */
  private async storeFailureAnalysis(analysis: FailureAnalysis, tenantId: string): Promise<void> {
    // Store in recent failures (limited size)
    this.recentFailures.unshift(analysis);
    if (this.recentFailures.length > this.MAX_RECENT_FAILURES) {
      this.recentFailures = this.recentFailures.slice(0, this.MAX_RECENT_FAILURES);
    }

    // Store in memory if available
    if (this.memoryManager) {
      try {
        await this.memoryManager.storeExperience(tenantId, {
          memoryId: `failure-analysis-${analysis.taskId}`,
          relevanceScore: 0.8,
          contextMatch: {
            similarityScore: 0.8,
            keywordMatches: ["failure", "error", analysis.taskType],
            semanticMatches: ["error analysis", "failure pattern", "task failure"],
            temporalAlignment: 0.9,
          },
          content: {
            type: "failure_analysis",
            taskId: analysis.taskId,
            taskType: analysis.taskType,
            error: analysis.error,
            patterns: analysis.patterns.map(p => p.id),
            recommendations: analysis.recommendations,
            severity: analysis.severity,
            confidence: analysis.confidence,
          },
        });
      } catch (error) {
        this.logger.warn(`Failed to store failure analysis for ${analysis.taskId}:`, error);
      }
    }
  }

  /**
   * Get error analytics
   */
  getAnalytics(): {
    totalPatterns: number;
    recentFailures: number;
    topPatterns: Array<{ pattern: string; frequency: number; category: ErrorCategory }>;
    categoryBreakdown: Record<ErrorCategory, number>;
    severityDistribution: Record<string, number>;
    averageConfidence: number;
  } {
    const topPatterns = Array.from(this.patterns.values())
      .sort((a, b) => b.frequency - a.frequency)
      .slice(0, 5)
      .map(p => ({
        pattern: p.pattern,
        frequency: p.frequency,
        category: p.category,
      }));

    const categoryBreakdown = Object.values(ErrorCategory).reduce((acc, category) => {
      acc[category] = Array.from(this.patterns.values())
        .filter(p => p.category === category).length;
      return acc;
    }, {} as Record<ErrorCategory, number>);

    const severityDistribution = this.recentFailures.reduce((acc, failure) => {
      acc[failure.severity] = (acc[failure.severity] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    const averageConfidence = this.recentFailures.length > 0 ?
      this.recentFailures.reduce((sum, f) => sum + f.confidence, 0) / this.recentFailures.length : 0;

    return {
      totalPatterns: this.patterns.size,
      recentFailures: this.recentFailures.length,
      topPatterns,
      categoryBreakdown,
      severityDistribution,
      averageConfidence,
    };
  }

  /**
   * Get adaptive prompt for task type
   */
  getAdaptivePrompt(taskType: string): AdaptivePrompt | null {
    const prompts = this.adaptivePrompts.get(taskType);
    return prompts && prompts.length > 0 ? prompts[0] : null;
  }

  /**
   * Update pattern from learning
   */
  updatePattern(patternId: string, updates: Partial<ErrorPattern>): void {
    const pattern = this.patterns.get(patternId);
    if (pattern) {
      Object.assign(pattern, updates);
      this.patterns.set(patternId, pattern);
    }
  }
}
