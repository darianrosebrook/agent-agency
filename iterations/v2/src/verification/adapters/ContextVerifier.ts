/**
 * @fileoverview Context Verification Adapter - ARBITER-020
 *
 * Verifies factual consistency by querying stored claims and citations,
 * checking temporal consistency, and cross-referencing conversation context.
 *
 * @author @darianrosebrook
 */

export interface ContextVerificationRequest {
  claim: string;
  context: {
    conversationHistory?: Array<{
      timestamp: Date;
      content: string;
      source: string;
    }>;
    storedClaims?: Array<{
      id: string;
      content: string;
      source: string;
      confidence: number;
      timestamp: Date;
    }>;
    citations?: Array<{
      url: string;
      title: string;
      timestamp?: Date;
      content?: string;
    }>;
  };
  verificationTypes?: Array<"temporal" | "factual" | "source">;
}

export interface ContextVerificationResult {
  success: boolean;
  confidence: number;
  inconsistencies: Array<{
    type: "temporal" | "factual" | "source";
    description: string;
    severity: "low" | "medium" | "high";
    conflictingClaims?: string[];
  }>;
  supportingEvidence: Array<{
    source: string;
    content: string;
    confidence: number;
    relevance: number;
  }>;
  temporalAnalysis?: {
    timelineConsistency: boolean;
    chronologicalIssues: string[];
  };
}

export class ContextVerifier {
  /**
   * Verify claim against conversation and stored context
   */
  async verify(
    request: ContextVerificationRequest
  ): Promise<ContextVerificationResult> {
    const {
      claim,
      context,
      verificationTypes = ["temporal", "factual", "source"],
    } = request;

    const inconsistencies: ContextVerificationResult["inconsistencies"] = [];
    const supportingEvidence: ContextVerificationResult["supportingEvidence"] =
      [];
    let temporalAnalysis: ContextVerificationResult["temporalAnalysis"];

    // Temporal verification
    if (verificationTypes.includes("temporal")) {
      temporalAnalysis = this.verifyTemporalConsistency(context);
      if (temporalAnalysis && !temporalAnalysis.timelineConsistency) {
        temporalAnalysis.chronologicalIssues.forEach((issue) => {
          inconsistencies.push({
            type: "temporal",
            description: issue,
            severity: "medium",
          });
        });
      }
    }

    // Factual verification against stored claims
    if (verificationTypes.includes("factual")) {
      const factualResult = this.verifyFactualConsistency(
        claim,
        context.storedClaims || []
      );
      inconsistencies.push(...factualResult.inconsistencies);
      supportingEvidence.push(...factualResult.supportingEvidence);
    }

    // Source verification
    if (verificationTypes.includes("source")) {
      const sourceResult = this.verifySourceCredibility(
        claim,
        context.citations || []
      );
      supportingEvidence.push(...sourceResult.supportingEvidence);

      if (sourceResult.lowCredibilitySources.length > 0) {
        inconsistencies.push({
          type: "source",
          description: `Claims based on low-credibility sources: ${sourceResult.lowCredibilitySources.join(
            ", "
          )}`,
          severity: "medium",
        });
      }
    }

    // Calculate overall confidence
    const confidence = this.calculateConfidence(
      inconsistencies,
      supportingEvidence
    );
    const success =
      confidence >= 0.6 &&
      inconsistencies.filter((i) => i.severity === "high").length === 0;

    return {
      success,
      confidence,
      inconsistencies,
      supportingEvidence,
      temporalAnalysis,
    };
  }

  /**
   * Verify timeline consistency across conversation and claims
   */
  private verifyTemporalConsistency(
    context: ContextVerificationRequest["context"]
  ): ContextVerificationResult["temporalAnalysis"] {
    const chronologicalIssues: string[] = [];

    // Check conversation history timeline
    if (context.conversationHistory) {
      const sortedHistory = [...context.conversationHistory].sort(
        (a, b) => a.timestamp.getTime() - b.timestamp.getTime()
      );

      for (let i = 1; i < sortedHistory.length; i++) {
        const prev = sortedHistory[i - 1];
        const curr = sortedHistory[i];

        // Check for impossible time jumps (e.g., future references in past messages)
        if (this.containsFutureReference(prev.content, curr.timestamp)) {
          chronologicalIssues.push(
            `Message at ${prev.timestamp.toISOString()} contains future reference`
          );
        }
      }
    }

    // Check stored claims timeline
    if (context.storedClaims) {
      const claimsWithDates = context.storedClaims.filter((claim) =>
        this.extractDateFromClaim(claim.content)
      );

      for (const claim of claimsWithDates) {
        const claimDate = this.extractDateFromClaim(claim.content);
        if (claimDate && claimDate > claim.timestamp) {
          chronologicalIssues.push(
            `Claim "${claim.content.substring(
              0,
              50
            )}..." references future date ${claimDate.toISOString()}`
          );
        }
      }
    }

    return {
      timelineConsistency: chronologicalIssues.length === 0,
      chronologicalIssues,
    };
  }

  /**
   * Verify factual consistency against stored claims
   */
  private verifyFactualConsistency(
    claim: string,
    storedClaims: ContextVerificationRequest["context"]["storedClaims"]
  ): {
    inconsistencies: ContextVerificationResult["inconsistencies"];
    supportingEvidence: ContextVerificationResult["supportingEvidence"];
  } {
    const inconsistencies: ContextVerificationResult["inconsistencies"] = [];
    const supportingEvidence: ContextVerificationResult["supportingEvidence"] =
      [];

    const claimEntities = this.extractEntities(claim);
    const claimKeywords = this.extractKeywords(claim);

    for (const storedClaim of storedClaims || []) {
      const storedEntities = this.extractEntities(storedClaim.content);
      const storedKeywords = this.extractKeywords(storedClaim.content);

      // Check for entity overlap
      const entityOverlap = this.calculateOverlap(
        claimEntities,
        storedEntities
      );
      const keywordOverlap = this.calculateOverlap(
        claimKeywords,
        storedKeywords
      );

      if (entityOverlap > 0.3 || keywordOverlap > 0.4) {
        // Check for contradictions
        const contradiction = this.detectContradiction(
          claim,
          storedClaim.content
        );

        if (contradiction.isContradiction) {
          inconsistencies.push({
            type: "factual",
            description: `Contradicts stored claim: ${contradiction.description}`,
            severity: contradiction.severity,
            conflictingClaims: [storedClaim.content],
          });
        } else if (contradiction.isSupporting) {
          supportingEvidence.push({
            source: storedClaim.source,
            content: storedClaim.content,
            confidence: storedClaim.confidence,
            relevance: Math.max(entityOverlap, keywordOverlap),
          });
        }
      }
    }

    return { inconsistencies, supportingEvidence };
  }

  /**
   * Verify source credibility
   */
  private verifySourceCredibility(
    claim: string,
    citations: ContextVerificationRequest["context"]["citations"]
  ): {
    supportingEvidence: ContextVerificationResult["supportingEvidence"];
    lowCredibilitySources: string[];
  } {
    const supportingEvidence: ContextVerificationResult["supportingEvidence"] =
      [];
    const lowCredibilitySources: string[] = [];

    for (const citation of citations || []) {
      const credibility = this.assessSourceCredibility(citation);

      if (credibility.score < 0.3) {
        lowCredibilitySources.push(citation.url);
      }

      // Check if citation supports the claim
      const relevance = this.calculateCitationRelevance(claim, citation);

      if (relevance > 0.3) {
        supportingEvidence.push({
          source: citation.url,
          content: citation.title,
          confidence: credibility.score,
          relevance,
        });
      }
    }

    return { supportingEvidence, lowCredibilitySources };
  }

  private calculateConfidence(
    inconsistencies: ContextVerificationResult["inconsistencies"],
    supportingEvidence: ContextVerificationResult["supportingEvidence"]
  ): number {
    let confidence = 1.0;

    // Reduce confidence based on inconsistencies
    for (const inconsistency of inconsistencies) {
      switch (inconsistency.severity) {
        case "high":
          confidence -= 0.3;
          break;
        case "medium":
          confidence -= 0.2;
          break;
        case "low":
          confidence -= 0.1;
          break;
      }
    }

    // Increase confidence based on supporting evidence
    const avgEvidenceConfidence =
      supportingEvidence.length > 0
        ? supportingEvidence.reduce(
            (sum, evidence) => sum + evidence.confidence * evidence.relevance,
            0
          ) / supportingEvidence.length
        : 0;

    confidence = Math.max(
      0,
      Math.min(1, confidence + avgEvidenceConfidence * 0.2)
    );

    return confidence;
  }

  private containsFutureReference(
    content: string,
    referenceTime: Date
  ): boolean {
    // Simple heuristic: check for future dates in content
    const futureDateRegex = /\b(20[3-9]\d|2[1-9]\d\d)\b/; // Years 2030+
    return futureDateRegex.test(content);
  }

  private extractDateFromClaim(content: string): Date | null {
    // Simple date extraction - in practice, use a more robust date parser
    const dateRegex = /\b(\d{4}-\d{2}-\d{2}|\d{1,2}\/\d{1,2}\/\d{4})\b/;
    const match = content.match(dateRegex);
    return match ? new Date(match[1]) : null;
  }

  private extractEntities(text: string): string[] {
    // Simple entity extraction - in practice, use NLP libraries
    const words = text.toLowerCase().split(/\s+/);
    return words.filter((word) => word.length > 3 && /^[a-z]+$/.test(word));
  }

  private extractKeywords(text: string): string[] {
    // Simple keyword extraction
    const words = text.toLowerCase().split(/\s+/);
    return words.filter((word) => word.length > 2);
  }

  private calculateOverlap(array1: string[], array2: string[]): number {
    const set1 = new Set(array1);
    const set2 = new Set(array2);
    const intersection = new Set([...set1].filter((x) => set2.has(x)));
    const union = new Set([...set1, ...set2]);
    return intersection.size / union.size;
  }

  private detectContradiction(
    claim1: string,
    claim2: string
  ): {
    isContradiction: boolean;
    isSupporting: boolean;
    description: string;
    severity: "low" | "medium" | "high";
  } {
    // Simple contradiction detection - in practice, use more sophisticated NLP
    const negationWords = [
      "not",
      "no",
      "never",
      "none",
      "nothing",
      "nobody",
      "nowhere",
    ];
    const claim1Words = claim1.toLowerCase().split(/\s+/);
    const claim2Words = claim2.toLowerCase().split(/\s+/);

    const claim1HasNegation = negationWords.some((neg) =>
      claim1Words.includes(neg)
    );
    const claim2HasNegation = negationWords.some((neg) =>
      claim2Words.includes(neg)
    );

    if (claim1HasNegation !== claim2HasNegation) {
      return {
        isContradiction: true,
        isSupporting: false,
        description: "One claim contains negation while the other does not",
        severity: "medium",
      };
    }

    // Check for supporting evidence
    const overlap = this.calculateOverlap(claim1Words, claim2Words);
    if (overlap > 0.5) {
      return {
        isContradiction: false,
        isSupporting: true,
        description: "Claims share significant content overlap",
        severity: "low",
      };
    }

    return {
      isContradiction: false,
      isSupporting: false,
      description: "No clear relationship detected",
      severity: "low",
    };
  }

  private assessSourceCredibility(citation: { url: string; title: string }): {
    score: number;
    reasons: string[];
  } {
    let score = 0.5; // Base score
    const reasons: string[] = [];

    const domain = new URL(citation.url).hostname.toLowerCase();

    // Check for known credible domains
    const credibleDomains = [
      "wikipedia.org",
      "scholar.google.com",
      "pubmed.ncbi.nlm.nih.gov",
      "arxiv.org",
    ];
    const questionableDomains = ["blogspot.com", "wordpress.com", "medium.com"];

    if (credibleDomains.some((credible) => domain.includes(credible))) {
      score += 0.3;
      reasons.push("Known credible domain");
    } else if (questionableDomains.some((q) => domain.includes(q))) {
      score -= 0.2;
      reasons.push("Questionable domain");
    }

    // Check for HTTPS
    if (citation.url.startsWith("https://")) {
      score += 0.1;
      reasons.push("HTTPS enabled");
    }

    // Check title quality
    if (citation.title.length > 10 && citation.title.length < 200) {
      score += 0.1;
      reasons.push("Reasonable title length");
    }

    return {
      score: Math.max(0, Math.min(1, score)),
      reasons,
    };
  }

  private calculateCitationRelevance(
    claim: string,
    citation: { title: string; content?: string }
  ): number {
    const claimWords = claim.toLowerCase().split(/\s+/);
    const citationWords = (citation.title + " " + (citation.content || ""))
      .toLowerCase()
      .split(/\s+/);

    return this.calculateOverlap(claimWords, citationWords);
  }
}
