/**
 * Information processing and ranking for search results
 * @author @darianrosebrook
 */

import {
  InformationProcessor as IInformationProcessor,
  KnowledgeQuery,
  RelevanceFactor,
  RelevanceScorer,
  SearchResult,
} from "../types/knowledge";

export class InformationProcessor implements IInformationProcessor {
  private readonly scorer: RelevanceScorer;

  constructor(scorer?: RelevanceScorer) {
    this.scorer = scorer || new DefaultRelevanceScorer();
  }

  async processResults(
    query: KnowledgeQuery,
    rawResults: SearchResult[]
  ): Promise<SearchResult[]> {
    // 1. Filter results based on query constraints
    let filtered = this.applyFilters(rawResults, query);

    // 2. Calculate relevance scores
    filtered = filtered.map((result) => ({
      ...result,
      relevanceScore: this.calculateRelevance(query.query, result),
    }));

    // 3. Deduplicate similar results
    filtered = this.deduplicateResults(filtered);

    // 4. Rank by relevance and credibility
    filtered = this.rankResults(filtered);

    // 5. Apply result limits
    const maxResults = query.maxResults || 50;
    filtered = filtered.slice(0, maxResults);

    return filtered;
  }

  calculateRelevance(query: string, result: SearchResult): number {
    return this.scorer.score(query, result);
  }

  deduplicateResults(results: SearchResult[]): SearchResult[] {
    const seen = new Set<string>();
    const deduplicated: SearchResult[] = [];

    for (const result of results) {
      const signature = this.createResultSignature(result);

      if (!seen.has(signature)) {
        seen.add(signature);
        deduplicated.push(result);
      }
    }

    return deduplicated;
  }

  rankResults(results: SearchResult[]): SearchResult[] {
    return results.sort((a, b) => {
      // Primary sort: relevance score
      const relevanceDiff = b.relevanceScore - a.relevanceScore;
      if (Math.abs(relevanceDiff) > 0.01) {
        return relevanceDiff;
      }

      // Secondary sort: credibility score
      const credibilityDiff = b.credibilityScore - a.credibilityScore;
      if (Math.abs(credibilityDiff) > 0.01) {
        return credibilityDiff;
      }

      // Tertiary sort: recency (if available)
      if (a.publishedDate && b.publishedDate) {
        return b.publishedDate.getTime() - a.publishedDate.getTime();
      }

      // Final sort: provider priority
      return this.getProviderPriority(b) - this.getProviderPriority(a);
    });
  }

  private applyFilters(
    results: SearchResult[],
    query: KnowledgeQuery
  ): SearchResult[] {
    if (!query.filters) return results;

    return results.filter((result) => {
      const filters = query.filters!;

      // Date range filter
      if (filters.dateRange && result.publishedDate) {
        if (
          result.publishedDate < filters.dateRange.start ||
          result.publishedDate > filters.dateRange.end
        ) {
          return false;
        }
      }

      // Language filter
      if (filters.language && result.metadata.language) {
        if (result.metadata.language !== filters.language) {
          return false;
        }
      }

      // Content type filter
      if (filters.contentType && filters.contentType.length > 0) {
        if (!filters.contentType.includes(result.contentType)) {
          return false;
        }
      }

      // Credibility filter
      if (filters.credibilityMinimum !== undefined) {
        if (result.credibilityScore < filters.credibilityMinimum) {
          return false;
        }
      }

      // Domain filters
      if (filters.excludeDomains && filters.excludeDomains.length > 0) {
        if (
          filters.excludeDomains.some((domain) => result.url.includes(domain))
        ) {
          return false;
        }
      }

      if (filters.includeDomains && filters.includeDomains.length > 0) {
        if (
          !filters.includeDomains.some((domain) => result.url.includes(domain))
        ) {
          return false;
        }
      }

      return true;
    });
  }

  private createResultSignature(result: SearchResult): string {
    // Create a signature based on title, URL domain, and first 100 chars of snippet
    const domain = this.extractDomain(result.url);
    const titleNormalized = result.title.toLowerCase().trim();
    const snippetPrefix = result.snippet.substring(0, 100).toLowerCase().trim();

    return `${domain}:${titleNormalized}:${snippetPrefix}`;
  }

  private extractDomain(url: string): string {
    try {
      return new URL(url).hostname;
    } catch {
      return url;
    }
  }

  private getProviderPriority(result: SearchResult): number {
    // Provider priority for tie-breaking (higher = better)
    const priorities: Record<string, number> = {
      wikipedia: 10,
      scholar: 9,
      github: 8,
      stackoverflow: 7,
      google: 5,
      bing: 4,
      duckduckgo: 3,
    };

    return priorities[result.provider.toString()] || 0;
  }
}

export class DefaultRelevanceScorer implements RelevanceScorer {
  score(query: string, result: SearchResult): number {
    const factors = this.getFactors(query, result);
    const totalWeight = factors.reduce((sum, factor) => sum + factor.weight, 0);

    if (totalWeight === 0) return 0;

    const weightedSum = factors.reduce(
      (sum, factor) => sum + factor.score * factor.weight,
      0
    );

    return weightedSum / totalWeight;
  }

  getFactors(query: string, result: SearchResult): RelevanceFactor[] {
    const queryLower = query.toLowerCase();
    const titleLower = result.title.toLowerCase();
    const snippetLower = result.snippet.toLowerCase();

    const factors: RelevanceFactor[] = [];

    // Title match factor
    const titleMatchScore = this.calculateTextMatch(queryLower, titleLower);
    factors.push({
      name: "title_match",
      weight: 0.4,
      score: titleMatchScore,
      explanation: `Title contains ${Math.round(
        titleMatchScore * 100
      )}% of query terms`,
    });

    // Snippet match factor
    const snippetMatchScore = this.calculateTextMatch(queryLower, snippetLower);
    factors.push({
      name: "snippet_match",
      weight: 0.3,
      score: snippetMatchScore,
      explanation: `Snippet contains ${Math.round(
        snippetMatchScore * 100
      )}% of query terms`,
    });

    // Credibility factor
    factors.push({
      name: "credibility",
      weight: 0.2,
      score: result.credibilityScore,
      explanation: `Source credibility score: ${result.credibilityScore}`,
    });

    // Recency factor (if date available)
    let recencyScore = 0.5; // Neutral score if no date
    if (result.publishedDate) {
      const daysOld =
        (Date.now() - result.publishedDate.getTime()) / (1000 * 60 * 60 * 24);
      if (daysOld < 7) recencyScore = 1.0;
      else if (daysOld < 30) recencyScore = 0.8;
      else if (daysOld < 365) recencyScore = 0.6;
      else recencyScore = 0.3;
    }
    factors.push({
      name: "recency",
      weight: 0.1,
      score: recencyScore,
      explanation: `Content recency score: ${recencyScore}`,
    });

    return factors;
  }

  private calculateTextMatch(query: string, text: string): number {
    const queryTerms = query.split(/\s+/).filter((term) => term.length > 2);
    if (queryTerms.length === 0) return 0;

    const textLower = text.toLowerCase();
    let matchedTerms = 0;

    for (const term of queryTerms) {
      if (textLower.includes(term)) {
        matchedTerms++;
      }
    }

    return matchedTerms / queryTerms.length;
  }
}

export class AdvancedRelevanceScorer extends DefaultRelevanceScorer {
  score(query: string, result: SearchResult): number {
    let score = super.score(query, result);

    // Additional advanced scoring factors

    // Domain authority boost
    if (this.isHighAuthorityDomain(result.url)) {
      score = Math.min(score * 1.2, 1.0);
    }

    // Content length preference (prefer substantial content)
    if (result.metadata.wordCount && result.metadata.wordCount > 500) {
      score = Math.min(score * 1.1, 1.0);
    }

    // Query-result semantic similarity (placeholder for ML-based scoring)
    // In a real implementation, this would use embeddings or ML models
    const semanticBoost = this.calculateSemanticSimilarity(query, result);
    score = Math.min(score * (1 + semanticBoost * 0.1), 1.0);

    return score;
  }

  private isHighAuthorityDomain(url: string): boolean {
    const highAuthorityDomains = [
      "wikipedia.org",
      "github.com",
      "stackoverflow.com",
      "scholar.google.com",
      "ieee.org",
      "acm.org",
      "nature.com",
      "science.org",
    ];

    try {
      const domain = new URL(url).hostname;
      return highAuthorityDomains.some((authDomain) =>
        domain.includes(authDomain)
      );
    } catch {
      return false;
    }
  }

  private calculateSemanticSimilarity(
    query: string,
    result: SearchResult
  ): number {
    // Placeholder for semantic similarity calculation
    // In production, this would use:
    // - BERT embeddings
    // - TF-IDF with cosine similarity
    // - Pre-trained language models

    // Simple fallback based on term overlap
    const queryTerms = new Set(query.toLowerCase().split(/\s+/));
    const titleTerms = new Set(result.title.toLowerCase().split(/\s+/));
    const snippetTerms = new Set(result.snippet.toLowerCase().split(/\s+/));

    const titleOverlap = this.calculateJaccardSimilarity(
      queryTerms,
      titleTerms
    );
    const snippetOverlap = this.calculateJaccardSimilarity(
      queryTerms,
      snippetTerms
    );

    return Math.max(titleOverlap, snippetOverlap);
  }

  private calculateJaccardSimilarity(
    set1: Set<string>,
    set2: Set<string>
  ): number {
    const intersection = new Set([...set1].filter((x) => set2.has(x)));
    const union = new Set([...set1, ...set2]);

    return intersection.size / union.size;
  }
}
