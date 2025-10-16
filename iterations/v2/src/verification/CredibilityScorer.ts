/**
 * @fileoverview Credibility Scorer Component (ARBITER-007)
 *
 * Assesses the credibility and reliability of information sources
 * using various credibility indicators and scoring algorithms.
 *
 * @author @darianrosebrook
 */

import {
  CredibilityFactor,
  SourceAnalysis,
  VerificationMethodConfig,
  VerificationMethodResult,
  VerificationRequest,
  VerificationType,
  VerificationVerdict,
} from "../types/verification";

/**
 * Credibility Scorer Implementation
 */
export class CredibilityScorer {
  private methodConfigs: VerificationMethodConfig[];
  private credibilityCache: Map<string, SourceAnalysis> = new Map();
  private healthMetrics: {
    totalRequests: number;
    successfulRequests: number;
    failedRequests: number;
    responseTimes: number[];
    lastHealthCheck: Date;
    consecutiveFailures: number;
    lastResponseTime: number;
    errorRate: number;
  } = {
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0,
    responseTimes: [],
    lastHealthCheck: new Date(),
    consecutiveFailures: 0,
    lastResponseTime: 0,
    errorRate: 0,
  };

  constructor(methodConfigs: VerificationMethodConfig[]) {
    this.methodConfigs = methodConfigs;
  }

  /**
   * Execute credibility scoring verification
   */
  async verify(
    request: VerificationRequest
  ): Promise<VerificationMethodResult> {
    const startTime = Date.now();

    try {
      // Extract sources from the request
      const sources = this.extractSources(request);

      if (sources.length === 0) {
        return {
          method: VerificationType.SOURCE_CREDIBILITY,
          verdict: VerificationVerdict.INSUFFICIENT_DATA,
          confidence: 0,
          reasoning: ["No sources found in content to evaluate"],
          processingTimeMs: Math.max(1, Date.now() - startTime),
          evidenceCount: 0,
        };
      }

      // Analyze source credibility
      const sourceAnalyses = await Promise.all(
        sources.map((source) => this.analyzeSource(source))
      );

      // Aggregate results
      const aggregatedResult = this.aggregateCredibilityResults(sourceAnalyses);
      const processingTime = Math.max(1, Date.now() - startTime);

      // Record successful verification
      this.recordSuccess(processingTime);

      return {
        method: VerificationType.SOURCE_CREDIBILITY,
        verdict: aggregatedResult.verdict,
        confidence: aggregatedResult.confidence,
        reasoning: aggregatedResult.explanations,
        processingTimeMs: processingTime,
        evidenceCount: sourceAnalyses.length,
      };
    } catch (error) {
      const processingTime = Math.max(1, Date.now() - startTime);

      // Record failed verification
      this.recordFailure();

      return {
        method: VerificationType.SOURCE_CREDIBILITY,
        verdict: VerificationVerdict.UNVERIFIED,
        confidence: 0,
        reasoning: [
          `Credibility scoring failed: ${
            error instanceof Error ? error.message : String(error)
          }`,
        ],
        processingTimeMs: processingTime,
        evidenceCount: 0,
      };
    }
  }

  /**
   * Extract sources from request content
   */
  private extractSources(request: VerificationRequest): string[] {
    const content = request.content;
    const sources: string[] = [];

    // Extract URLs
    const urlRegex = /(https?:\/\/[^\s]+)/g;
    let match;
    while ((match = urlRegex.exec(content)) !== null) {
      sources.push(match[1]);
    }

    // Extract domain references (simplified)
    const domainRegex = /\b([a-zA-Z0-9-]+\.[a-zA-Z]{2,})(?:\.[a-zA-Z]{2,})?\b/g;
    while ((match = domainRegex.exec(content)) !== null) {
      const domain = match[1];
      // Avoid common words that might match
      if (
        ![
          "the",
          "and",
          "for",
          "are",
          "but",
          "not",
          "you",
          "all",
          "can",
          "had",
          "her",
          "was",
          "one",
          "our",
          "out",
          "day",
          "get",
          "has",
          "him",
          "his",
          "how",
          "its",
          "may",
          "new",
          "now",
          "old",
          "see",
          "two",
          "who",
          "boy",
          "did",
          "has",
          "her",
          "him",
          "his",
          "how",
          "its",
          "may",
          "new",
          "now",
          "old",
          "see",
          "two",
          "who",
          "act",
          "add",
          "age",
          "ago",
          "aim",
          "air",
          "all",
          "and",
          "any",
          "are",
          "arm",
          "art",
          "ask",
          "bad",
          "bag",
          "ban",
          "bar",
          "bat",
          "bay",
          "bed",
          "bee",
          "beg",
          "bet",
          "bid",
          "big",
          "bin",
          "bit",
          "bob",
          "bog",
          "boo",
          "bow",
          "box",
          "boy",
          "bud",
          "bug",
          "bun",
          "bus",
          "but",
          "buy",
          "bye",
          "cab",
          "can",
          "cap",
          "car",
          "cat",
          "cow",
          "cry",
          "cup",
          "cut",
          "dad",
          "dam",
          "day",
          "did",
          "die",
          "dig",
          "dim",
          "dip",
          "dog",
          "dot",
          "dry",
          "dub",
          "dud",
          "due",
          "dug",
          "ear",
          "eat",
          "egg",
          "ego",
          "end",
          "era",
          "eve",
          "eye",
          "fan",
          "far",
          "fat",
          "fax",
          "fed",
          "fee",
          "few",
          "fig",
          "fin",
          "fir",
          "fit",
          "fix",
          "fly",
          "fog",
          "for",
          "fox",
          "fry",
          "fun",
          "fur",
          "gab",
          "gad",
          "gag",
          "gap",
          "gas",
          "gay",
          "gee",
          "gel",
          "gem",
          "get",
          "gig",
          "gin",
          "god",
          "got",
          "gum",
          "gun",
          "gut",
          "guy",
          "gym",
          "had",
          "hag",
          "ham",
          "has",
          "hat",
          "hay",
          "hem",
          "hen",
          "her",
          "hew",
          "hex",
          "hey",
          "hid",
          "him",
          "hip",
          "his",
          "hit",
          "hog",
          "hop",
          "hot",
          "how",
          "hub",
          "hue",
          "hug",
          "huh",
          "hum",
          "hut",
          "ice",
          "icy",
          "ill",
          "ink",
          "inn",
          "ion",
          "ire",
          "ivy",
          "jab",
          "jag",
          "jam",
          "jar",
          "jaw",
          "jay",
          "jet",
          "jew",
          "jig",
          "job",
          "jog",
          "joy",
          "jug",
          "jut",
          "kay",
          "ken",
          "key",
          "kid",
          "kin",
          "kit",
          "lab",
          "lad",
          "lag",
          "lap",
          "law",
          "lax",
          "lay",
          "led",
          "leg",
          "let",
          "lid",
          "lie",
          "lip",
          "lit",
          "lob",
          "log",
          "lop",
          "lot",
          "low",
          "loy",
          "lug",
          "lye",
          "mad",
          "mag",
          "man",
          "map",
          "mar",
          "mat",
          "may",
          "men",
          "met",
          "mew",
          "mid",
          "mix",
          "mob",
          "mod",
          "mom",
          "moo",
          "mop",
          "mow",
          "mud",
          "mug",
          "mum",
          "nab",
          "nag",
          "nap",
          "nay",
          "net",
          "new",
          "nil",
          "nip",
          "nit",
          "nob",
          "nod",
          "nog",
          "nor",
          "not",
          "now",
          "nun",
          "nut",
          "oaf",
          "oak",
          "oar",
          "oat",
          "odd",
          "ode",
          "off",
          "oft",
          "oil",
          "old",
          "ole",
          "one",
          "opt",
          "orb",
          "ore",
          "our",
          "out",
          "owe",
          "owl",
          "own",
          "pad",
          "pal",
          "pan",
          "pap",
          "par",
          "pat",
          "paw",
          "pay",
          "pea",
          "peg",
          "pen",
          "pep",
          "per",
          "pet",
          "pew",
          "pic",
          "pie",
          "pig",
          "pin",
          "pip",
          "pit",
          "ply",
          "pod",
          "pop",
          "pot",
          "pow",
          "pro",
          "pry",
          "pub",
          "pug",
          "pun",
          "pup",
          "pus",
          "put",
          "rag",
          "ram",
          "ran",
          "rap",
          "rat",
          "raw",
          "ray",
          "red",
          "rep",
          "rib",
          "rid",
          "rig",
          "rim",
          "rip",
          "rob",
          "rod",
          "roe",
          "rot",
          "row",
          "rub",
          "rug",
          "rum",
          "run",
          "rye",
          "sac",
          "sad",
          "sag",
          "sap",
          "sat",
          "saw",
          "say",
          "sea",
          "see",
          "sew",
          "sex",
          "she",
          "shy",
          "sin",
          "sip",
          "sir",
          "sit",
          "six",
          "ski",
          "sky",
          "sly",
          "sob",
          "sod",
          "son",
          "sop",
          "sow",
          "soy",
          "spa",
          "spy",
          "sub",
          "sue",
          "sum",
          "sun",
          "sup",
          "tab",
          "tag",
          "tan",
          "tap",
          "tar",
          "tax",
          "tea",
          "tee",
          "ten",
          "the",
          "thy",
          "tie",
          "tin",
          "tip",
          "toe",
          "tog",
          "ton",
          "too",
          "top",
          "tow",
          "toy",
          "try",
          "tub",
          "tug",
          "two",
          "use",
          "van",
          "vat",
          "vet",
          "vex",
          "via",
          "vie",
          "vow",
          "wag",
          "wan",
          "war",
          "was",
          "wax",
          "way",
          "web",
          "wed",
          "wee",
          "wet",
          "who",
          "why",
          "wig",
          "win",
          "wit",
          "woe",
          "won",
          "woo",
          "wow",
          "wry",
          "wye",
          "yak",
          "yam",
          "yap",
          "yaw",
          "yea",
          "yen",
          "yes",
          "yet",
          "yew",
          "yid",
          "yin",
          "yip",
          "yon",
          "you",
          "yow",
          "yup",
          "zag",
          "zap",
          "zed",
          "zee",
          "zen",
          "zig",
          "zip",
          "zoo",
        ].includes(domain.toLowerCase())
      ) {
        sources.push(`https://${domain}`);
      }
    }

    // Remove duplicates and limit
    const uniqueSources = Array.from(new Set(sources));
    return uniqueSources.slice(0, 10); // Limit to 10 sources
  }

  /**
   * Analyze credibility of a single source
   */
  private async analyzeSource(sourceUrl: string): Promise<SourceAnalysis> {
    // Check cache first
    const cached = this.credibilityCache.get(sourceUrl);
    if (cached && this.isCacheValid(cached)) {
      return cached;
    }

    const analysis: SourceAnalysis = {
      url: sourceUrl,
      domain: this.extractDomain(sourceUrl),
      credibilityScore: 0.5, // Default
      factors: [],
      analysisDate: new Date(),
    };

    // Analyze credibility factors
    const factors = await this.evaluateCredibilityFactors(
      sourceUrl,
      analysis.domain
    );
    analysis.factors = factors;

    // Calculate overall score
    analysis.credibilityScore = this.calculateCredibilityScore(factors);

    // Set cache expiry (24 hours)
    analysis.cacheExpiry = new Date(Date.now() + 24 * 60 * 60 * 1000);

    // Cache the result
    this.credibilityCache.set(sourceUrl, analysis);

    return analysis;
  }

  /**
   * Evaluate credibility factors for a source
   */
  private async evaluateCredibilityFactors(
    url: string,
    domain: string
  ): Promise<CredibilityFactor[]> {
    const factors: CredibilityFactor[] = [];

    // Domain reputation factor
    factors.push(await this.evaluateDomainReputation(domain));

    // Content type factor
    factors.push(this.evaluateContentType(url, domain));

    // Age and stability factor
    factors.push(await this.evaluateSourceAge(domain));

    // Traffic and authority factor
    factors.push(await this.evaluateAuthority(domain));

    // Bias and reliability factor
    factors.push(await this.evaluateBiasAndReliability(domain));

    // Technical factors
    factors.push(this.evaluateTechnicalFactors(url));

    return factors;
  }

  /**
   * Evaluate domain reputation
   */
  private async evaluateDomainReputation(
    domain: string
  ): Promise<CredibilityFactor> {
    // Known credible domains
    const highlyCredible = [
      "edu",
      "gov",
      "org",
      "ac.uk",
      "ac.jp",
      "ac.de",
      "ac.fr",
      "ac.au",
      "who.int",
      "un.org",
      "nasa.gov",
      "nih.gov",
      "cdc.gov",
    ];

    const somewhatCredible = ["com", "net", "io", "co", "news", "media"];

    const suspiciousTlds = [
      "xyz",
      "club",
      "online",
      "site",
      "website",
      "space",
    ];

    let score = 0.5;
    let explanation = "Average domain credibility";

    if (highlyCredible.some((credible) => domain.includes(credible))) {
      score = 0.9;
      explanation = "Highly credible institutional domain";
    } else if (somewhatCredible.some((credible) => domain.endsWith(credible))) {
      score = 0.7;
      explanation = "Commercial domain with potential credibility";
    } else if (
      suspiciousTlds.some((suspicious) => domain.endsWith(suspicious))
    ) {
      score = 0.2;
      explanation = "Suspicious top-level domain";
    }

    return {
      name: "domain_reputation",
      score,
      weight: 0.25,
      explanation,
      evidence: [`Domain: ${domain}`],
    };
  }

  /**
   * Evaluate content type
   */
  private evaluateContentType(url: string, domain: string): CredibilityFactor {
    const path = url.toLowerCase();
    let score = 0.5;
    let explanation = "General content type";

    // News and media domains
    if (
      domain.includes("news") ||
      domain.includes("cnn") ||
      domain.includes("bbc") ||
      domain.includes("reuters") ||
      domain.includes("apnews")
    ) {
      score = 0.8;
      explanation = "News and media content";
    }
    // Academic domains
    else if (
      domain.includes("edu") ||
      domain.includes("ac.") ||
      domain.includes("scholar")
    ) {
      score = 0.9;
      explanation = "Academic and scholarly content";
    }
    // Government domains
    else if (
      domain.includes("gov") ||
      domain.includes("gov.uk") ||
      domain.includes("gouv.fr")
    ) {
      score = 0.95;
      explanation = "Government and official content";
    }
    // Social media
    else if (
      domain.includes("twitter") ||
      domain.includes("facebook") ||
      domain.includes("reddit")
    ) {
      score = 0.3;
      explanation = "Social media content (user-generated)";
    }
    // Blog/personal sites
    else if (
      domain.includes("blogspot") ||
      domain.includes("wordpress") ||
      path.includes("/blog/")
    ) {
      score = 0.4;
      explanation = "Blog or personal content";
    }

    return {
      name: "content_type",
      score,
      weight: 0.2,
      explanation,
      evidence: [`URL: ${url}`],
    };
  }

  /**
   * Evaluate source age and stability
   */
  private async evaluateSourceAge(domain: string): Promise<CredibilityFactor> {
    // In production, this would check WHOIS data or domain registration date
    // For now, use heuristics
    let score = 0.5;
    let explanation = "Unknown domain age";

    // Well-established domains
    if (
      domain.includes("wikipedia.org") ||
      domain.includes("bbc") ||
      domain.includes("cnn")
    ) {
      score = 0.9;
      explanation = "Well-established domain with long history";
    } else if (/\b\d{4}\b/.test(domain)) {
      // Domains with years in them might be newer
      score = 0.4;
      explanation = "Domain appears relatively new";
    }

    return {
      name: "source_age",
      score,
      weight: 0.15,
      explanation,
      evidence: [`Domain registration analysis for ${domain}`],
    };
  }

  /**
   * Evaluate authority and traffic
   */
  private async evaluateAuthority(domain: string): Promise<CredibilityFactor> {
    // In production, this would check Alexa rank, backlinks, etc.
    // For now, use domain-based heuristics

    let score = 0.5;
    let explanation = "Average authority and traffic";

    const highAuthority = [
      "wikipedia.org",
      "github.com",
      "stackoverflow.com",
      "nytimes.com",
      "washingtonpost.com",
      "bbc.com",
      "reuters.com",
      "apnews.com",
    ];

    if (highAuthority.includes(domain)) {
      score = 0.9;
      explanation = "High-authority domain with significant traffic";
    }

    return {
      name: "authority_traffic",
      score,
      weight: 0.15,
      explanation,
      evidence: [`Authority analysis for ${domain}`],
    };
  }

  /**
   * Evaluate bias and reliability
   */
  private async evaluateBiasAndReliability(
    domain: string
  ): Promise<CredibilityFactor> {
    // In production, this would check Media Bias/Fact Check ratings
    // For now, use known biases

    let score = 0.5;
    let explanation = "Moderate bias and reliability";

    // Known reliable sources
    const highlyReliable = [
      "bbc.com",
      "reuters.com",
      "apnews.com",
      "npr.org",
      "pbs.org",
      "who.int",
      "un.org",
      "nasa.gov",
      "nih.gov",
      "cdc.gov",
    ];

    // Known biased sources (examples)
    const biased = [
      "breitbart.com",
      "dailymail.co.uk",
      "foxnews.com",
      "msnbc.com",
    ];

    if (highlyReliable.includes(domain)) {
      score = 0.9;
      explanation = "Known reliable source with minimal bias";
    } else if (biased.includes(domain)) {
      score = 0.3;
      explanation = "Source with known political bias";
    }

    return {
      name: "bias_reliability",
      score,
      weight: 0.15,
      explanation,
      evidence: [`Bias analysis for ${domain}`],
    };
  }

  /**
   * Evaluate technical factors
   */
  private evaluateTechnicalFactors(url: string): CredibilityFactor {
    let score = 0.5;
    let explanation = "Standard technical implementation";
    const evidence: string[] = [];

    // HTTPS check
    if (url.startsWith("https://")) {
      score += 0.2;
      evidence.push("Uses HTTPS encryption");
    } else {
      score -= 0.3;
      evidence.push("Uses HTTP (not encrypted)");
    }

    // URL structure
    if (url.includes("://") && !url.includes(" ")) {
      score += 0.1;
      evidence.push("Valid URL structure");
    }

    // Subdomain analysis
    const subdomain = url.split("://")[1]?.split(".")[0];
    if (
      subdomain &&
      subdomain !== "www" &&
      subdomain !== url.split("://")[1]?.split(".")[1]
    ) {
      // Has meaningful subdomain
      evidence.push(`Subdomain: ${subdomain}`);
    }

    if (score > 0.7) {
      explanation = "Strong technical implementation";
    } else if (score < 0.4) {
      explanation = "Weak technical implementation";
    }

    return {
      name: "technical_factors",
      score: Math.max(0, Math.min(1, score)),
      weight: 0.1,
      explanation,
      evidence,
    };
  }

  /**
   * Calculate overall credibility score from factors
   */
  private calculateCredibilityScore(factors: CredibilityFactor[]): number {
    if (factors.length === 0) return 0.5;

    let weightedSum = 0;
    let totalWeight = 0;

    for (const factor of factors) {
      weightedSum += factor.score * factor.weight;
      totalWeight += factor.weight;
    }

    return totalWeight > 0 ? weightedSum / totalWeight : 0.5;
  }

  /**
   * Aggregate credibility analysis results
   */
  private aggregateCredibilityResults(analyses: SourceAnalysis[]): {
    verdict: VerificationVerdict;
    confidence: number;
    explanations: string[];
    evidenceCount: number;
  } {
    if (analyses.length === 0) {
      return {
        verdict: VerificationVerdict.INSUFFICIENT_DATA,
        confidence: 0,
        explanations: ["No sources analyzed"],
        evidenceCount: 0,
      };
    }

    const avgCredibility =
      analyses.reduce((sum, a) => sum + a.credibilityScore, 0) /
      analyses.length;

    // Classify overall credibility
    let verdict = VerificationVerdict.UNVERIFIED;
    const confidence = avgCredibility;

    if (avgCredibility >= 0.8) {
      verdict = VerificationVerdict.VERIFIED_TRUE; // High credibility sources
    } else if (avgCredibility >= 0.6) {
      verdict = VerificationVerdict.PARTIALLY_TRUE; // Moderate credibility
    } else if (avgCredibility < 0.3) {
      verdict = VerificationVerdict.VERIFIED_FALSE; // Low credibility sources
    }

    const explanations = analyses.map(
      (analysis) =>
        `${analysis.domain}: ${analysis.credibilityScore.toFixed(
          2
        )} credibility`
    );

    return {
      verdict,
      confidence,
      explanations,
      evidenceCount: analyses.length,
    };
  }

  /**
   * Extract domain from URL
   */
  private extractDomain(url: string): string {
    try {
      return new URL(url).hostname;
    } catch {
      return "unknown";
    }
  }

  /**
   * Check if cached analysis is still valid
   */
  private isCacheValid(analysis: SourceAnalysis): boolean {
    if (!analysis.cacheExpiry) return false;
    return analysis.cacheExpiry.getTime() > Date.now();
  }

  /**
   * Get method configuration
   */
  private getMethodConfig(): VerificationMethodConfig | undefined {
    return this.methodConfigs.find(
      (config) => config.type === VerificationType.SOURCE_CREDIBILITY
    );
  }

  /**
   * Check if method is available
   */
  async isAvailable(): Promise<boolean> {
    const config = this.getMethodConfig();
    return config?.enabled ?? false;
  }

  /**
   * Get method health status
   */
  getHealth(): { available: boolean; responseTime: number; errorRate: number } {
    // Update error rate based on recent metrics
    this.updateErrorRate();

    // Check availability based on consecutive failures and cache health
    const now = new Date();
    const timeSinceLastCheck =
      now.getTime() - this.healthMetrics.lastHealthCheck.getTime();
    const available =
      this.healthMetrics.consecutiveFailures < 3 && timeSinceLastCheck < 300000; // 5 minutes

    // Calculate average response time
    const avgResponseTime =
      this.healthMetrics.responseTimes.length > 0
        ? this.healthMetrics.responseTimes.reduce(
            (sum, time) => sum + time,
            0
          ) / this.healthMetrics.responseTimes.length
        : this.healthMetrics.lastResponseTime || 0;

    return {
      available,
      responseTime: Math.round(avgResponseTime),
      errorRate: Math.round(this.healthMetrics.errorRate * 100) / 100,
    };
  }

  /**
   * Record a successful verification request
   */
  private recordSuccess(responseTime: number): void {
    this.healthMetrics.totalRequests++;
    this.healthMetrics.successfulRequests++;
    this.healthMetrics.responseTimes.push(responseTime);
    this.healthMetrics.lastResponseTime = responseTime;
    this.healthMetrics.consecutiveFailures = 0;

    // Keep only last 100 response times
    if (this.healthMetrics.responseTimes.length > 100) {
      this.healthMetrics.responseTimes =
        this.healthMetrics.responseTimes.slice(-100);
    }
  }

  /**
   * Record a failed verification request
   */
  private recordFailure(): void {
    this.healthMetrics.totalRequests++;
    this.healthMetrics.failedRequests++;
    this.healthMetrics.consecutiveFailures++;
  }

  /**
   * Update error rate calculation
   */
  private updateErrorRate(): void {
    if (this.healthMetrics.totalRequests === 0) {
      this.healthMetrics.errorRate = 0;
    } else {
      // Use exponential moving average for error rate
      const currentErrorRate =
        this.healthMetrics.failedRequests / this.healthMetrics.totalRequests;
      const alpha = 0.1; // Smoothing factor
      this.healthMetrics.errorRate =
        this.healthMetrics.errorRate * (1 - alpha) + currentErrorRate * alpha;
    }
  }
}
