/**
 * Fact checking provider for the verification engine
 * @author @darianrosebrook
 */

import {
  VerificationProvider,
  VerificationType,
  VerificationRequest,
  VerificationMethodResult,
  VerificationVerdict,
  FactCheckClaim,
  FactCheckResult,
  MethodStatus,
  VerificationError,
  VerificationErrorCode
} from '../types/verification';

export class FactChecker implements VerificationProvider {
  readonly type = VerificationType.FACT_CHECKING;

  private readonly apiKey?: string;
  private readonly baseUrl: string;
  private lastUsed?: Date;
  private successCount = 0;
  private totalCount = 0;

  constructor(config: { apiKey?: string; baseUrl?: string } = {}) {
    this.apiKey = config.apiKey;
    this.baseUrl = config.baseUrl || 'https://factchecktools.googleapis.com';
  }

  async verify(request: VerificationRequest): Promise<VerificationMethodResult> {
    const startTime = Date.now();
    this.lastUsed = new Date();

    try {
      const claim: FactCheckClaim = {
        text: request.content,
        context: request.context,
        language: 'en'
      };

      const result = await this.performFactCheck(claim);
      this.successCount++;
      this.totalCount++;

      return {
        method: this.type,
        verdict: this.mapVerdict(result.verdict),
        confidence: result.confidence,
        reasoning: [result.explanation],
        processingTimeMs: Date.now() - startTime,
        evidenceCount: result.sources.length
      };
    } catch (error) {
      this.totalCount++;

      throw new VerificationError(
        `Fact checking failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        VerificationErrorCode.EXTERNAL_SERVICE_ERROR,
        request.id,
        this.type
      );
    }
  }

  async isAvailable(): Promise<boolean> {
    try {
      // Basic connectivity check
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 5000);

      const response = await fetch(`${this.baseUrl}/v1alpha1/claims:search`, {
        method: 'GET',
        signal: controller.signal,
        headers: this.getHeaders()
      });

      clearTimeout(timeoutId);
      return response.ok;
    } catch {
      return false;
    }
  }

  async getHealth(): Promise<MethodStatus> {
    const available = await this.isAvailable();
    const successRate = this.totalCount > 0 ? this.successCount / this.totalCount : 1.0;

    return {
      type: this.type,
      enabled: true,
      healthy: available,
      lastUsed: this.lastUsed,
      successRate,
      averageProcessingTime: 3000 // Estimated
    };
  }

  private async performFactCheck(claim: FactCheckClaim): Promise<FactCheckResult> {
    // This would integrate with actual fact-checking services
    // For now, return a mock result

    const mockResult: FactCheckResult = {
      claim,
      verdict: VerificationVerdict.VERIFIED_TRUE,
      confidence: 0.85,
      explanation: 'Based on multiple reliable sources, this claim appears to be accurate.',
      sources: [
        {
          url: 'https://example.com/source1',
          title: 'Reliable Source 1',
          publisher: 'Trusted Publisher',
          credibilityScore: 0.9,
          publishDate: new Date(),
          excerpt: 'Supporting evidence from source 1'
        },
        {
          url: 'https://example.com/source2',
          title: 'Reliable Source 2',
          publisher: 'Another Trusted Publisher',
          credibilityScore: 0.85,
          publishDate: new Date(),
          excerpt: 'Additional supporting evidence'
        }
      ],
      relatedClaims: []
    };

    // Simulate API call delay
    await new Promise(resolve => setTimeout(resolve, 100));

    return mockResult;
  }

  private mapVerdict(apiVerdict: VerificationVerdict): VerificationVerdict {
    // Map external API verdicts to our internal enum
    return apiVerdict;
  }

  private getHeaders(): Record<string, string> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json'
    };

    if (this.apiKey) {
      headers['X-API-Key'] = this.apiKey;
    }

    return headers;
  }
}

export class CredibilityScorer implements VerificationProvider {
  readonly type = VerificationType.SOURCE_CREDIBILITY;

  private readonly credibilityDatabase: Map<string, number> = new Map();
  private lastUsed?: Date;
  private successCount = 0;
  private totalCount = 0;

  constructor() {
    // Initialize with some known credible domains
    this.initializeCredibilityDatabase();
  }

  async verify(request: VerificationRequest): Promise<VerificationMethodResult> {
    const startTime = Date.now();
    this.lastUsed = new Date();

    try {
      const sourceUrl = request.source || this.extractSourceFromContent(request.content);
      const credibilityScore = this.scoreSource(sourceUrl);

      const verdict = this.determineVerdict(credibilityScore);
      const confidence = Math.min(credibilityScore + 0.1, 1.0); // Add some confidence buffer

      this.successCount++;
      this.totalCount++;

      return {
        method: this.type,
        verdict,
        confidence,
        reasoning: [`Source credibility score: ${credibilityScore.toFixed(2)}`],
        processingTimeMs: Date.now() - startTime,
        evidenceCount: 1
      };
    } catch (error) {
      this.totalCount++;

      throw new VerificationError(
        `Source credibility scoring failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        VerificationErrorCode.EXTERNAL_SERVICE_ERROR,
        request.id,
        this.type
      );
    }
  }

  async isAvailable(): Promise<boolean> {
    // Credibility scoring is always available (uses local database)
    return true;
  }

  async getHealth(): Promise<MethodStatus> {
    const successRate = this.totalCount > 0 ? this.successCount / this.totalCount : 1.0;

    return {
      type: this.type,
      enabled: true,
      healthy: true,
      lastUsed: this.lastUsed,
      successRate,
      averageProcessingTime: 50 // Very fast
    };
  }

  private initializeCredibilityDatabase(): void {
    // High credibility sources
    const highCredibility = [
      'wikipedia.org',
      'scholar.google.com',
      'nature.com',
      'science.org',
      'ieee.org',
      'acm.org',
      'reuters.com',
      'apnews.com',
      'bbc.com',
      'nytimes.com'
    ];

    highCredibility.forEach(domain => {
      this.credibilityDatabase.set(domain, 0.9);
    });

    // Medium credibility sources
    const mediumCredibility = [
      'github.com',
      'stackoverflow.com',
      'medium.com',
      'techcrunch.com',
      'wired.com',
      'arstechnica.com'
    ];

    mediumCredibility.forEach(domain => {
      this.credibilityDatabase.set(domain, 0.7);
    });

    // Low credibility sources (common fake news or biased sites)
    const lowCredibility = [
      'breitbart.com',
      'infowars.com',
      'naturalnews.com'
    ];

    lowCredibility.forEach(domain => {
      this.credibilityDatabase.set(domain, 0.2);
    });
  }

  private scoreSource(url: string): number {
    try {
      const domain = new URL(url).hostname.toLowerCase();

      // Exact domain match
      if (this.credibilityDatabase.has(domain)) {
        return this.credibilityDatabase.get(domain)!;
      }

      // Partial domain match (e.g., subdomains)
      for (const [knownDomain, score] of this.credibilityDatabase.entries()) {
        if (domain.includes(knownDomain) || knownDomain.includes(domain)) {
          return score * 0.8; // Slightly lower for partial matches
        }
      }

      // Default score for unknown domains
      return 0.5;
    } catch {
      // Invalid URL
      return 0.1;
    }
  }

  private determineVerdict(score: number): VerificationVerdict {
    if (score >= 0.8) return VerificationVerdict.VERIFIED_TRUE;
    if (score >= 0.6) return VerificationVerdict.PARTIALLY_TRUE;
    if (score >= 0.3) return VerificationVerdict.UNVERIFIED;

    return VerificationVerdict.VERIFIED_FALSE;
  }

  private extractSourceFromContent(content: string): string {
    // Try to extract source URL from content
    const urlRegex = /(https?:\/\/[^\s]+)/g;
    const match = content.match(urlRegex);

    if (match && match.length > 0) {
      return match[0];
    }

    // Fallback to a generic unknown source
    return 'unknown-source.com';
  }
}
