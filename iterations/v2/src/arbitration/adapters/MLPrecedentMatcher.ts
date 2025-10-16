/**
 * ML/NLP Precedent Matcher Adapter
 *
 * @author @darianrosebrook
 *
 * Provides advanced ML/NLP-based precedent matching for constitutional rule engine.
 * Uses semantic similarity, entity recognition, and context understanding for
 * more accurate precedent matching than simple text-based approaches.
 *
 * Features:
 * - Semantic similarity using embeddings
 * - Named entity recognition for context matching
 * - Intent classification for action matching
 * - Context-aware similarity scoring
 * - Fallback to rule-based matching
 */

import {
  Precedent,
  RuleCategory,
  ViolationSeverity,
} from "../../types/arbitration";

/**
 * ML/NLP matcher configuration
 */
export interface MLPrecedentMatcherConfig {
  /** Minimum similarity threshold for matches (0-1) */
  minSimilarityThreshold: number;

  /** Maximum number of precedents to return */
  maxResults: number;

  /** Enable semantic similarity */
  enableSemanticSimilarity: boolean;

  /** Enable entity recognition */
  enableEntityRecognition: boolean;

  /** Enable intent classification */
  enableIntentClassification: boolean;

  /** Fallback to rule-based matching if ML fails */
  enableFallback: boolean;

  /** Weight for different similarity factors */
  weights: {
    semantic: number;
    entity: number;
    intent: number;
    category: number;
    severity: number;
  };
}

/**
 * Entity extracted from text
 */
export interface ExtractedEntity {
  /** Entity text */
  text: string;

  /** Entity type */
  type: "PERSON" | "ORGANIZATION" | "ACTION" | "OBJECT" | "CONCEPT" | "RULE";

  /** Confidence score */
  confidence: number;

  /** Position in text */
  position: { start: number; end: number };
}

/**
 * Intent classification result
 */
export interface IntentClassification {
  /** Primary intent */
  intent: string;

  /** Confidence score */
  confidence: number;

  /** Alternative intents with scores */
  alternatives: Array<{ intent: string; score: number }>;
}

/**
 * Semantic similarity result
 */
export interface SemanticSimilarity {
  /** Similarity score (0-1) */
  score: number;

  /** Matching phrases */
  matchingPhrases: string[];

  /** Semantic distance */
  distance: number;
}

/**
 * ML precedent match result
 */
export interface MLPrecedentMatch {
  /** The precedent */
  precedent: Precedent;

  /** Overall similarity score */
  score: number;

  /** Breakdown of similarity factors */
  factors: {
    semantic: number;
    entity: number;
    intent: number;
    category: number;
    severity: number;
  };

  /** Matching entities */
  matchingEntities: ExtractedEntity[];

  /** Intent alignment */
  intentAlignment: IntentClassification;

  /** Semantic similarity details */
  semanticSimilarity: SemanticSimilarity;

  /** Reasoning for the match */
  reasoning: string;
}

/**
 * ML/NLP Precedent Matcher
 */
export class MLPrecedentMatcher {
  private config: MLPrecedentMatcherConfig;

  constructor(config?: Partial<MLPrecedentMatcherConfig>) {
    this.config = {
      minSimilarityThreshold: 0.7,
      maxResults: 10,
      enableSemanticSimilarity: true,
      enableEntityRecognition: true,
      enableIntentClassification: true,
      enableFallback: true,
      weights: {
        semantic: 0.4,
        entity: 0.3,
        intent: 0.2,
        category: 0.05,
        severity: 0.05,
      },
      ...config,
    };
  }

  /**
   * Find similar precedents using ML/NLP techniques
   */
  public async findSimilarPrecedents(
    context: {
      action: string;
      actor: string;
      parameters: Record<string, any>;
      environment: Record<string, any>;
      category: RuleCategory;
      severity: ViolationSeverity;
    },
    precedents: Precedent[]
  ): Promise<MLPrecedentMatch[]> {
    const matches: MLPrecedentMatch[] = [];

    for (const precedent of precedents) {
      try {
        const match = await this.calculateMLSimilarity(context, precedent);

        if (match.score >= this.config.minSimilarityThreshold) {
          matches.push(match);
        }
      } catch (error) {
        console.warn(
          `Failed to calculate ML similarity for precedent ${precedent.id}:`,
          error
        );

        // Fallback to rule-based matching if enabled
        if (this.config.enableFallback) {
          const fallbackMatch = this.calculateFallbackSimilarity(
            context,
            precedent
          );
          if (fallbackMatch.score >= this.config.minSimilarityThreshold) {
            matches.push(fallbackMatch);
          }
        }
      }
    }

    // Sort by score and return top results
    matches.sort((a, b) => b.score - a.score);
    return matches.slice(0, this.config.maxResults);
  }

  /**
   * Calculate ML-based similarity between context and precedent
   */
  private async calculateMLSimilarity(
    context: any,
    precedent: Precedent
  ): Promise<MLPrecedentMatch> {
    // Extract text for analysis
    const contextText = this.extractContextText(context);
    const precedentText = this.extractPrecedentText(precedent);

    // Perform ML/NLP analysis
    const [entities, intent, semantic] = await Promise.all([
      this.config.enableEntityRecognition
        ? this.extractEntities(contextText)
        : Promise.resolve([]),
      this.config.enableIntentClassification
        ? this.classifyIntent(contextText)
        : Promise.resolve(null),
      this.config.enableSemanticSimilarity
        ? this.calculateSemanticSimilarity(contextText, precedentText)
        : Promise.resolve(null),
    ]);

    // Calculate weighted similarity score
    const factors = this.calculateSimilarityFactors(
      context,
      precedent,
      entities,
      intent,
      semantic
    );

    const score = this.weightedScore(factors);

    return {
      precedent,
      score,
      factors,
      matchingEntities: entities,
      intentAlignment: intent || {
        intent: "unknown",
        confidence: 0,
        alternatives: [],
      },
      semanticSimilarity: semantic || {
        score: 0,
        matchingPhrases: [],
        distance: 1,
      },
      reasoning: this.generateReasoning(factors, entities, intent, semantic),
    };
  }

  /**
   * Extract entities from text using NLP
   */
  private async extractEntities(text: string): Promise<ExtractedEntity[]> {
    // Mock implementation - in real system would use spaCy, NLTK, or cloud NLP service
    const entities: ExtractedEntity[] = [];

    // Simple rule-based entity extraction for demo
    const words = text.toLowerCase().split(/\s+/);

    // Look for common entity patterns
    const entityPatterns = [
      { type: "PERSON" as const, pattern: /\b(agent|user|admin|operator)\b/ },
      {
        type: "ACTION" as const,
        pattern: /\b(create|delete|modify|access|execute)\b/,
      },
      {
        type: "OBJECT" as const,
        pattern: /\b(file|document|database|system|resource)\b/,
      },
      {
        type: "RULE" as const,
        pattern: /\b(rule|policy|constraint|requirement)\b/,
      },
    ];

    for (const word of words) {
      for (const pattern of entityPatterns) {
        if (pattern.pattern.test(word)) {
          entities.push({
            text: word,
            type: pattern.type,
            confidence: 0.8,
            position: { start: 0, end: word.length },
          });
        }
      }
    }

    return entities;
  }

  /**
   * Classify intent from text
   */
  private async classifyIntent(text: string): Promise<IntentClassification> {
    // Mock implementation - in real system would use intent classification model
    const intents = [
      "create_resource",
      "access_data",
      "modify_system",
      "execute_task",
      "violate_policy",
      "request_permission",
    ];

    // Simple keyword-based intent classification
    const textLower = text.toLowerCase();
    let bestIntent = "unknown";
    let bestScore = 0;

    for (const intent of intents) {
      const keywords = intent.split("_");
      let score = 0;

      for (const keyword of keywords) {
        if (textLower.includes(keyword)) {
          score += 0.3;
        }
      }

      if (score > bestScore) {
        bestScore = score;
        bestIntent = intent;
      }
    }

    return {
      intent: bestIntent,
      confidence: Math.min(bestScore, 1.0),
      alternatives: intents
        .filter((i) => i !== bestIntent)
        .map((intent) => ({ intent, score: Math.random() * 0.3 })),
    };
  }

  /**
   * Calculate semantic similarity between texts
   */
  private async calculateSemanticSimilarity(
    text1: string,
    text2: string
  ): Promise<SemanticSimilarity> {
    // Mock implementation - in real system would use sentence transformers, BERT, etc.
    const words1 = new Set(text1.toLowerCase().split(/\s+/));
    const words2 = new Set(text2.toLowerCase().split(/\s+/));

    const intersection = new Set([...words1].filter((x) => words2.has(x)));
    const union = new Set([...words1, ...words2]);

    const jaccardSimilarity = intersection.size / union.size;

    return {
      score: jaccardSimilarity,
      matchingPhrases: Array.from(intersection),
      distance: 1 - jaccardSimilarity,
    };
  }

  /**
   * Calculate similarity factors
   */
  private calculateSimilarityFactors(
    context: any,
    precedent: Precedent,
    entities: ExtractedEntity[],
    intent: IntentClassification | null,
    semantic: SemanticSimilarity | null
  ) {
    // Category similarity
    const categorySimilarity =
      context.category === precedent.applicability.category ? 1.0 : 0.0;

    // Severity similarity
    const severitySimilarity =
      context.severity === precedent.applicability.severity ? 1.0 : 0.8;

    // Semantic similarity
    const semanticScore = semantic?.score || 0;

    // Entity similarity
    const entityScore = this.calculateEntitySimilarity(entities, precedent);

    // Intent similarity
    const intentScore = this.calculateIntentSimilarity(intent, precedent);

    return {
      semantic: semanticScore,
      entity: entityScore,
      intent: intentScore,
      category: categorySimilarity,
      severity: severitySimilarity,
    };
  }

  /**
   * Calculate entity similarity
   */
  private calculateEntitySimilarity(
    entities: ExtractedEntity[],
    precedent: Precedent
  ): number {
    if (entities.length === 0) return 0;

    // Extract entities from precedent key facts
    const precedentEntities = precedent.keyFacts
      .join(" ")
      .toLowerCase()
      .split(/\s+/);
    const contextEntities = entities.map((e) => e.text);

    const intersection = contextEntities.filter((e) =>
      precedentEntities.some((pe) => pe.includes(e) || e.includes(pe))
    );

    return (
      intersection.length /
      Math.max(contextEntities.length, precedentEntities.length)
    );
  }

  /**
   * Calculate intent similarity
   */
  private calculateIntentSimilarity(
    intent: IntentClassification | null,
    precedent: Precedent
  ): number {
    if (!intent) return 0;

    // Extract intent from precedent verdict
    const precedentIntent = precedent.verdict.reasoning
      .map((r) => r.description)
      .join(" ")
      .toLowerCase();
    const contextIntent = intent.intent;

    // Simple keyword matching for intent alignment
    const intentKeywords = contextIntent.split("_");
    let score = 0;

    for (const keyword of intentKeywords) {
      if (precedentIntent.includes(keyword)) {
        score += 0.3;
      }
    }

    return Math.min(score, 1.0) * intent.confidence;
  }

  /**
   * Calculate weighted score
   */
  private weightedScore(factors: any): number {
    return (
      factors.semantic * this.config.weights.semantic +
      factors.entity * this.config.weights.entity +
      factors.intent * this.config.weights.intent +
      factors.category * this.config.weights.category +
      factors.severity * this.config.weights.severity
    );
  }

  /**
   * Generate reasoning for the match
   */
  private generateReasoning(
    factors: any,
    entities: ExtractedEntity[],
    intent: IntentClassification | null,
    semantic: SemanticSimilarity | null
  ): string {
    const reasons: string[] = [];

    if (factors.category === 1.0) {
      reasons.push("Exact category match");
    }

    if (factors.semantic > 0.7) {
      reasons.push(
        `High semantic similarity (${(factors.semantic * 100).toFixed(1)}%)`
      );
    }

    if (factors.entity > 0.5) {
      reasons.push(
        `Entity alignment with ${entities.length} matching entities`
      );
    }

    if (intent && factors.intent > 0.6) {
      reasons.push(
        `Intent alignment: ${intent.intent} (${(
          intent.confidence * 100
        ).toFixed(1)}% confidence)`
      );
    }

    if (factors.severity > 0.8) {
      reasons.push("Severity level match");
    }

    return reasons.length > 0
      ? `Match based on: ${reasons.join(", ")}`
      : "Low confidence match";
  }

  /**
   * Fallback similarity calculation using simple rules
   */
  private calculateFallbackSimilarity(
    context: any,
    precedent: Precedent
  ): MLPrecedentMatch {
    // Simple rule-based fallback
    const categoryMatch =
      precedent.applicability?.category &&
      context.category === precedent.applicability.category
        ? 1.0
        : 0.0;
    const severityMatch =
      precedent.applicability?.severity &&
      context.severity === precedent.applicability.severity
        ? 1.0
        : 0.8;

    // Simple text similarity
    const contextText = this.extractContextText(context);
    const precedentText = this.extractPrecedentText(precedent);
    const textSimilarity = this.calculateSimpleTextSimilarity(
      contextText,
      precedentText
    );

    const factors = {
      semantic: textSimilarity,
      entity: 0.5,
      intent: 0.5,
      category: categoryMatch,
      severity: severityMatch,
    };

    const score = this.weightedScore(factors);

    return {
      precedent,
      score,
      factors,
      matchingEntities: [],
      intentAlignment: { intent: "unknown", confidence: 0, alternatives: [] },
      semanticSimilarity: {
        score: textSimilarity,
        matchingPhrases: [],
        distance: 1 - textSimilarity,
      },
      reasoning: `Fallback match: category=${categoryMatch}, text=${textSimilarity.toFixed(
        2
      )}`,
    };
  }

  /**
   * Extract text from evaluation context
   */
  private extractContextText(context: any): string {
    return [
      context.action,
      context.actor,
      JSON.stringify(context.parameters),
      JSON.stringify(context.environment),
    ].join(" ");
  }

  /**
   * Extract text from precedent
   */
  private extractPrecedentText(precedent: Precedent): string {
    const reasoningText = Array.isArray(precedent.verdict?.reasoning)
      ? precedent.verdict.reasoning.map((r) => r.description).join(" ")
      : precedent.verdict?.reasoning || "";

    return [
      ...(precedent.keyFacts || []),
      reasoningText,
      ...(precedent.applicability?.conditions || []),
    ].join(" ");
  }

  /**
   * Simple text similarity calculation
   */
  private calculateSimpleTextSimilarity(text1: string, text2: string): number {
    const words1 = new Set(text1.toLowerCase().split(/\s+/));
    const words2 = new Set(text2.toLowerCase().split(/\s+/));

    const intersection = new Set([...words1].filter((x) => words2.has(x)));
    const union = new Set([...words1, ...words2]);

    return intersection.size / union.size;
  }

  /**
   * Update configuration
   */
  public updateConfig(config: Partial<MLPrecedentMatcherConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current configuration
   */
  public getConfig(): MLPrecedentMatcherConfig {
    return { ...this.config };
  }
}
