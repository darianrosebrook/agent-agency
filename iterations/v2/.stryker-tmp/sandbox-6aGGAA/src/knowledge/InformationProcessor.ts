/**
 * @fileoverview Information Processor for ARBITER-006
 *
 * Processes and filters search results, assesses relevance and credibility,
 * removes duplicates, and generates response summaries.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import {
  IInformationProcessor,
  InformationProcessorConfig,
  KnowledgeQuery,
  SearchResult,
  ResultQuality,
  SourceType,
} from "../types/knowledge";
import { events } from "../orchestrator/EventEmitter";
import { EventTypes } from "../orchestrator/OrchestratorEvents";

/**
 * Information Processor implementation
 */
export class InformationProcessor implements IInformationProcessor {
  private config: InformationProcessorConfig;

  constructor(config: InformationProcessorConfig) {
    this.config = config;
  }

  /**
   * Process search results through filtering, ranking, and deduplication
   */
  async processResults(query: KnowledgeQuery, results: SearchResult[]): Promise<SearchResult[]> {
    const startTime = Date.now();

    try {
      let processedResults = [...results];

      // Step 1: Remove duplicates
      if (this.config.quality.enableDuplicateDetection) {
        processedResults = this.detectDuplicates(processedResults);
      }

      // Step 2: Filter by relevance threshold
      if (this.config.quality.enableRelevanceFiltering) {
        processedResults = processedResults.filter(result =>
          result.relevanceScore >= this.config.minRelevanceScore
        );
      }

      // Step 3: Filter by credibility
      processedResults = processedResults.filter(result =>
        result.credibilityScore >= this.config.minCredibilityScore
      );

      // Step 4: Assess relevance for each result
      processedResults.forEach(result => {
        result.relevanceScore = this.scoreRelevance(query, result);
      });

      // Step 5: Re-assess credibility
      processedResults.forEach(result => {
        result.credibilityScore = this.assessCredibility(result);
        result.quality = this.determineQuality(result.relevanceScore, result.credibilityScore);
      });

      // Step 6: Apply diversity constraints
      processedResults = this.applyDiversityConstraints(processedResults);

      // Step 7: Sort by combined score
      processedResults.sort((a, b) => {
        const scoreA = this.calculateCombinedScore(a);
        const scoreB = this.calculateCombinedScore(b);
        return scoreB - scoreA; // Descending order
      });

      // Step 8: Limit results
      processedResults = processedResults.slice(0, this.config.maxResultsToProcess);

      // Emit processing completion event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
        timestamp: new Date(),
        severity: "info" as any,
        source: "InformationProcessor",
        taskId: query.id,
        metadata: {
          originalResults: results.length,
          processedResults: processedResults.length,
          processingTimeMs: Date.now() - startTime,
          relevanceThreshold: query.relevanceThreshold,
          duplicatesRemoved: results.length - processedResults.length,
        },
      });

      return processedResults;
    } catch (error) {
      console.error(`Information processing failed for query ${query.id}:`, error);

      // Emit error event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
        timestamp: new Date(),
        severity: "error" as any,
        source: "InformationProcessor",
        taskId: query.id,
        metadata: {
          error: error instanceof Error ? error.message : String(error),
          processingTimeMs: Date.now() - startTime,
        },
      });

      // Return empty results on error
      return [];
    }
  }

  /**
   * Score relevance of a result to the query
   */
  scoreRelevance(query: KnowledgeQuery, result: SearchResult): number {
    let score = 0.5; // Base score

    const queryText = query.query.toLowerCase();
    const title = result.title.toLowerCase();
    const content = result.content.toLowerCase();

    // Title match bonus
    if (title.includes(queryText)) {
      score += 0.3;
    }

    // Content match bonus
    if (content.includes(queryText)) {
      score += 0.2;
    }

    // Exact phrase match bonus
    const queryWords = queryText.split(/\s+/);
    const exactPhraseMatches = queryWords.filter(word =>
      title.includes(word) || content.includes(word)
    ).length;
    score += (exactPhraseMatches / queryWords.length) * 0.1;

    // Query type adjustments
    switch (query.queryType) {
      case "factual":
        // Prefer recent results for factual queries
        if (result.publishedAt) {
          const daysOld = (Date.now() - result.publishedAt.getTime()) / (1000 * 60 * 60 * 24);
          if (daysOld < 365) score += 0.1;
        }
        break;

      case "technical":
        // Prefer documentation and code sources
        if (result.sourceType === "documentation") score += 0.1;
        break;
    }

    // Provider reputation bonus
    const reputableProviders = ["google", "bing", "arxiv", "pubmed"];
    if (reputableProviders.includes(result.provider.toLowerCase())) {
      score += 0.05;
    }

    return Math.min(1.0, Math.max(0.0, score));
  }

  /**
   * Assess credibility of a result
   */
  assessCredibility(result: SearchResult): number {
    let score = 0.5; // Base score

    // Source type credibility
    switch (result.sourceType) {
      case "academic":
        score += 0.3;
        break;
      case "documentation":
        score += 0.2;
        break;
      case "news":
        score += 0.1;
        break;
      case "web":
        score += 0.0;
        break;
      case "social":
        score -= 0.1; // Social media typically less credible
        break;
    }

    // Domain reputation (simplified)
    const trustedDomains = [
      "edu", "gov", "org", "ac.uk", "ac.jp", "ac.de", "ac.fr", "ac.au"
    ];
    const suspiciousDomains = [
      "blogspot", "wordpress", "medium", "reddit"
    ];

    if (trustedDomains.some(domain => result.domain.includes(domain))) {
      score += 0.1;
    }
    if (suspiciousDomains.some(domain => result.domain.includes(domain))) {
      score -= 0.1;
    }

    // Content quality indicators
    if (result.content.length > 100) score += 0.05; // Substantial content
    if (result.title.length > 10) score += 0.05; // Descriptive title

    // Recency bonus (newer content tends to be more reliable)
    if (result.publishedAt) {
      const daysOld = (Date.now() - result.publishedAt.getTime()) / (1000 * 60 * 60 * 24);
      if (daysOld < 30) score += 0.05;
      else if (daysOld > 365 * 2) score -= 0.05; // Very old content
    }

    return Math.min(1.0, Math.max(0.0, score));
  }

  /**
   * Detect and remove duplicate results
   */
  detectDuplicates(results: SearchResult[]): SearchResult[] {
    const seen = new Set<string>();
    const uniqueResults: SearchResult[] = [];

    for (const result of results) {
      // Create content hash for deduplication
      const contentHash = this.createContentHash(result);

      if (!seen.has(contentHash)) {
        seen.add(contentHash);
        uniqueResults.push(result);
      }
    }

    return uniqueResults;
  }

  /**
   * Generate summary from processed results
   */
  generateSummary(query: KnowledgeQuery, results: SearchResult[]): string {
    if (results.length === 0) {
      return `No relevant information found for query: "${query.query}"`;
    }

    const highQualityResults = results.filter(r => r.quality === ResultQuality.HIGH);
    const topResults = results.slice(0, Math.min(3, results.length));

    let summary = `Found ${results.length} relevant results for "${query.query}". `;

    if (highQualityResults.length > 0) {
      summary += `${highQualityResults.length} high-quality sources identified. `;
    }

    // Add key insights from top results
    const keyPoints: string[] = [];
    topResults.forEach((result, index) => {
      const truncatedContent = result.content.substring(0, 150);
      keyPoints.push(`${index + 1}. ${truncatedContent}...`);
    });

    if (keyPoints.length > 0) {
      summary += `Key findings: ${keyPoints.join(' ')}`;
    }

    // Add source diversity info
    const uniqueSources = new Set(results.map(r => r.domain)).size;
    summary += ` Information sourced from ${uniqueSources} different domains.`;

    return summary;
  }

  /**
   * Apply diversity constraints to results
   */
  private applyDiversityConstraints(results: SearchResult[]): SearchResult[] {
    const diversity = this.config.diversity;
    let filteredResults = [...results];

    // Limit results per domain
    const domainCounts = new Map<string, number>();
    filteredResults = filteredResults.filter(result => {
      const count = domainCounts.get(result.domain) || 0;
      if (count >= diversity.maxResultsPerDomain) {
        return false;
      }
      domainCounts.set(result.domain, count + 1);
      return true;
    });

    // Ensure minimum source types
    const sourceTypes = new Set(filteredResults.map(r => r.sourceType));
    if (sourceTypes.size < diversity.minSourceTypes && filteredResults.length > diversity.minSourceTypes) {
      // If we don't have enough diversity, prioritize keeping diverse sources
      const diverseResults: SearchResult[] = [];
      const typesAdded = new Set<SourceType>();

      for (const result of filteredResults) {
        if (!typesAdded.has(result.sourceType) && typesAdded.size < diversity.minSourceTypes) {
          diverseResults.push(result);
          typesAdded.add(result.sourceType);
        }
      }

      // Fill remaining slots with highest-scoring results
      const remaining = filteredResults.filter(r => !diverseResults.includes(r));
      remaining.sort((a, b) => this.calculateCombinedScore(b) - this.calculateCombinedScore(a));
      diverseResults.push(...remaining.slice(0, diversity.minSources - diverseResults.length));

      filteredResults = diverseResults;
    }

    return filteredResults;
  }

  /**
   * Determine result quality based on scores
   */
  private determineQuality(relevanceScore: number, credibilityScore: number): ResultQuality {
    const combinedScore = (relevanceScore + credibilityScore) / 2;

    if (combinedScore >= 0.8) return ResultQuality.HIGH;
    if (combinedScore >= 0.6) return ResultQuality.MEDIUM;
    if (combinedScore >= 0.3) return ResultQuality.LOW;
    return ResultQuality.UNRELIABLE;
  }

  /**
   * Calculate combined score for ranking
   */
  private calculateCombinedScore(result: SearchResult): number {
    // Weight relevance and credibility equally, with small bonus for quality
    let score = (result.relevanceScore + result.credibilityScore) / 2;

    // Quality bonus
    switch (result.quality) {
      case ResultQuality.HIGH:
        score += 0.1;
        break;
      case ResultQuality.MEDIUM:
        score += 0.05;
        break;
      case ResultQuality.LOW:
        score -= 0.05;
        break;
      case ResultQuality.UNRELIABLE:
        score -= 0.1;
        break;
    }

    // Recency bonus
    if (result.publishedAt) {
      const daysOld = (Date.now() - result.publishedAt.getTime()) / (1000 * 60 * 60 * 24);
      if (daysOld < 7) score += 0.05;
      else if (daysOld < 30) score += 0.02;
    }

    return Math.min(1.0, Math.max(0.0, score));
  }

  /**
   * Create content hash for duplicate detection
   */
  private createContentHash(result: SearchResult): string {
    // Simple hash of title and first 500 chars of content
    const content = `${result.title}${result.content.substring(0, 500)}`.toLowerCase();
    let hash = 0;
    for (let i = 0; i < content.length; i++) {
      const char = content.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return hash.toString();
  }
}


