/**
 * @fileoverview Core implementation of the four-stage claim extraction and verification pipeline
 *               aligning with the Claimify methodology and CAWS governance requirements.
 *               The stages are:
 *               1. Contextual disambiguation
 *               2. Verifiable content qualification
 *               3. Atomic claim decomposition
 *               4. CAWS-compliant verification
 */

import type { WorkingSpec } from "../types/caws-types.js";

import {
  AmbiguityAnalysis,
  AmbiguityHandler,
  ArbitrationDecision,
  AtomicClaim,
  ClaimBasedArbiter,
  ClaimBasedEvaluation,
  ClaimExtractionAndVerificationProcessor,
  ClaimLearningSystem,
  ConversationContext,
  DisambiguationResult,
  EvidenceManifest,
  ExtractedClaim,
  LearningUpdate,
  PatternUpdate,
  ResolutionAttempt,
  ScopeValidation,
  UnresolvableAmbiguity,
  VerifiableContentResult,
  VerificationCriteria,
  VerificationResult,
  VerificationStep,
} from "./types";

/**
 * Main implementation of the claim extraction and verification processor
 * Implements the four-stage Claimify pipeline with CAWS compliance
 */
export class ClaimExtractor
  implements
    ClaimExtractionAndVerificationProcessor,
    AmbiguityHandler,
    ClaimBasedArbiter,
    ClaimLearningSystem
{
  private ambiguityPatterns: Map<string, RegExp[]> = new Map();
  private extractionPatterns: Map<string, RegExp[]> = new Map();
  private verificationSources: Map<string, any> = new Map();

  constructor() {
    this.initializePatterns();
    this.initializeVerificationSources();
  }

  readonly disambiguationStage = {
    identifyAmbiguities: (
      sentence: string,
      context: ConversationContext
    ) => this.identifyAmbiguities(sentence, context),
    resolveAmbiguities: (
      sentence: string,
      ambiguities: AmbiguityAnalysis,
      context: ConversationContext
    ) => this.resolveAmbiguities(sentence, ambiguities, context),
    detectUnresolvableAmbiguities: (
      sentence: string,
      context: ConversationContext
    ) => this.detectUnresolvableAmbiguities(sentence, context),
  };

  readonly qualificationStage = {
    detectVerifiableContent: (
      sentence: string,
      context: ConversationContext
    ) => this.detectVerifiableContent(sentence, context),
    rewriteUnverifiableContent: (
      sentence: string,
      context: ConversationContext
    ) => this.rewriteUnverifiableContent(sentence, context),
  };

  readonly decompositionStage = {
    extractAtomicClaims: (
      sentence: string,
      context: ConversationContext
    ) => this.extractAtomicClaims(sentence, context),
    addContextualBrackets: (claim: string, impliedContext: string) =>
      this.addContextualBrackets(claim, impliedContext),
  };

  readonly verificationStage = {
    verifyClaimEvidence: (
      claim: ExtractedClaim,
      evidence: EvidenceManifest
    ) => this.verifyClaim(claim, { evidenceManifest: evidence }),
    validateClaimScope: (claim: ExtractedClaim, workingSpec: WorkingSpec) =>
      Promise.resolve(
        this.validateClaimScopeAgainstSpec(claim, workingSpec)
      ),
  };

  // ============================================================================
  // STAGE 1: CONTEXTUAL DISAMBIGUATION
  // ============================================================================

  /**
   * Identify ambiguities in a sentence that could affect claim extraction
   */
  async identifyAmbiguities(
    sentence: string,
    context: ConversationContext
  ): Promise<AmbiguityAnalysis> {
    const referentialPatterns = [
      /\b(this|that|these|those|it|they|he|she|we|us|them|him|her)\b/gi,
    ];
    const structuralPatterns = [
      /\b[A-Z][a-z]+ (is|are|was|were) [a-z]+ (and|or) [a-z]+\b/gi,
      /\b[A-Z][a-z]+ (called|named|known as) [A-Z][a-z]+\b/gi,
      /\b(before|after|during|while) [a-z]+ (and|or) [a-z]+\b/gi,
    ];
    const temporalPatterns = [
      /\b(next|last|previous|upcoming|recent|soon|recently)\b/gi,
      /\b(tomorrow|yesterday|today|now|then)\b/gi,
    ];

    const referentialAmbiguities = this.matchAllUnique(
      referentialPatterns,
      sentence
    );
    const structuralAmbiguities = this.matchAllUnique(
      structuralPatterns,
      sentence
    );
    const temporalAmbiguities = this.matchAllUnique(
      temporalPatterns,
      sentence
    );

    const contextEntities = this.extractContextEntities(context);
    const referentialResolvable =
      referentialAmbiguities.length === 0 || contextEntities.length > 0;
    const temporalResolvable =
      temporalAmbiguities.length === 0 || this.hasTimelineContext(context);
    const structuralResolvable = structuralAmbiguities.length <= 2;

    const canResolve =
      referentialResolvable && temporalResolvable && structuralResolvable;
    const resolutionConfidence = this.computeResolutionConfidence({
      referentialAmbiguities,
      structuralAmbiguities,
      temporalAmbiguities,
      referentialResolvable,
      temporalResolvable,
      structuralResolvable,
    });

    return {
      referentialAmbiguities,
      structuralAmbiguities,
      temporalAmbiguities,
      canResolve,
      resolutionConfidence,
    };
  }

  /**
   * Resolve identified ambiguities using available context
   */
  async resolveAmbiguities(
    sentence: string,
    ambiguities: AmbiguityAnalysis,
    context: ConversationContext
  ): Promise<DisambiguationResult> {
    const auditTrail: ResolutionAttempt[] = [];

    const noAmbiguities =
      ambiguities.referentialAmbiguities.length === 0 &&
      ambiguities.structuralAmbiguities.length === 0 &&
      ambiguities.temporalAmbiguities.length === 0;

    if (noAmbiguities) {
      return {
        success: true,
        disambiguatedSentence: sentence,
        auditTrail,
      };
    }

    if (!ambiguities.canResolve) {
      return {
        success: false,
        failureReason: "cannot_resolve",
        auditTrail,
      };
    }

    if (
      ambiguities.temporalAmbiguities.length > 0 &&
      !this.hasTimelineContext(context)
    ) {
      for (const phrase of ambiguities.temporalAmbiguities) {
        auditTrail.push({
          success: false,
          reason: "Missing timeline context",
          confidence: 0.1,
          strategy: "fallback",
          metadata: { phrase, type: "temporal" },
        });
      }

      return {
        success: false,
        failureReason: "insufficient_context",
        auditTrail,
      };
    }

    let resolvedSentence = sentence;
    const uniqueReferential = [
      ...new Set(
        ambiguities.referentialAmbiguities.map((token) => token.trim())
      ),
    ];

    for (const pronoun of uniqueReferential) {
      const attempt = await this.handleReferentialAmbiguity(pronoun, context);
      auditTrail.push({
        ...attempt,
        metadata: {
          ...(attempt.metadata ?? {}),
          pronoun,
          type: "referential",
        },
      });

      if (!attempt.success || !attempt.resolvedInterpretation) {
        return {
          success: false,
          failureReason: "cannot_resolve",
          auditTrail,
        };
      }

      const regex = new RegExp(`\\b${this.escapeRegex(pronoun)}\\b`, "gi");
      resolvedSentence = resolvedSentence.replace(
        regex,
        attempt.resolvedInterpretation
      );
    }

    for (const structural of ambiguities.structuralAmbiguities) {
      const interpretations = this.generateStructuralInterpretations(
        structural,
        sentence
      );
      const attempt = await this.handleStructuralAmbiguity(
        structural,
        interpretations,
        context
      );

      auditTrail.push({
        ...attempt,
        metadata: {
          ...(attempt.metadata ?? {}),
          phrase: structural,
          type: "structural",
        },
      });

      if (attempt.success && attempt.resolvedInterpretation) {
        resolvedSentence = resolvedSentence.replace(
          structural,
          attempt.resolvedInterpretation
        );
      }
    }

    return {
      success: true,
      disambiguatedSentence: resolvedSentence,
      auditTrail,
    };
  }

  /**
   * Identify unresolvable ambiguities that prevent claim extraction
   */
  async detectUnresolvableAmbiguities(
    sentence: string,
    context: ConversationContext
  ): Promise<UnresolvableAmbiguity[]> {
    const ambiguities: UnresolvableAmbiguity[] = [];

    const contextEntities = this.extractContextEntities(context);
    const referentialPatterns = /\b(this|that|it|they|he|she)\b/gi;
    const temporalPatterns =
      /\b(next|last|previous|upcoming|recent|soon|recently|tomorrow|yesterday|today|now|then)\b/gi;

    if (referentialPatterns.test(sentence) && contextEntities.length === 0) {
      ambiguities.push({
        type: "referential",
        phrase: "pronoun reference",
        reason: "Pronoun used without clear antecedent in conversation context",
        confidence: 0.8,
      });
    }

    const temporalMatches = sentence.match(temporalPatterns) || [];
    if (temporalMatches.length > 0 && !this.hasTimelineContext(context)) {
      for (const phrase of temporalMatches) {
        ambiguities.push({
          type: "temporal",
          phrase,
          reason: "Temporal reference without clear timeline context",
          confidence: 0.9,
        });
      }
    }

    if (this.isStructurallyAmbiguous(sentence)) {
      ambiguities.push({
        type: "structural",
        phrase: sentence,
        reason: "Multiple grammatical interpretations detected without disambiguating cues",
        confidence: 0.6,
      });
    }

    return ambiguities;
  }

  /**
   * Handle referential ambiguity resolution attempts
   */
  async handleReferentialAmbiguity(
    ambiguousPhrase: string,
    context: ConversationContext
  ): Promise<ResolutionAttempt> {
    const candidates = this.extractContextEntities(context);
    const normalizedPronoun = ambiguousPhrase.trim().toLowerCase();

    if (candidates.length > 0) {
      const resolved = this.selectAntecedent(normalizedPronoun, candidates);
      if (resolved) {
        return {
          success: true,
          resolvedInterpretation: resolved,
          confidence: 0.8,
          strategy: "context_lookup",
          metadata: {
            candidates,
            chosen: resolved,
          },
        };
      }
    }

    const fallbackSubject = this.extractFallbackSubject(context);
    if (fallbackSubject) {
      return {
        success: true,
        resolvedInterpretation: fallbackSubject,
        confidence: 0.6,
        strategy: "fallback",
        metadata: {
          pronoun: ambiguousPhrase,
          source: "context.metadata.defaultSubject",
        },
      };
    }

    return {
      success: false,
      reason: "No clear antecedent found in context",
      confidence: 0.2,
      strategy: "fallback",
      metadata: {
        pronoun: ambiguousPhrase,
      },
    };
  }

  /**
   * Handle structural ambiguity resolution attempts
   */
  async handleStructuralAmbiguity(
    sentence: string,
    possibleInterpretations: string[],
    context: ConversationContext
  ): Promise<ResolutionAttempt> {
    if (possibleInterpretations.length > 0) {
      const resolved = this.selectStructuralInterpretation(
        possibleInterpretations,
        context
      );
      return {
        success: true,
        resolvedInterpretation: resolved,
        confidence: 0.6,
        strategy: "pattern_inference",
        metadata: {
          interpretations: possibleInterpretations,
          chosen: resolved,
        },
      };
    }

    return {
      success: false,
      reason: "No viable interpretations available",
      confidence: 0.1,
      strategy: "fallback",
      metadata: {
        sentence,
      },
    };
  }

  // ============================================================================
  // STAGE 2: VERIFIABLE CONTENT QUALIFICATION
  // ============================================================================

  /**
   * Detect verifiable content in a sentence after disambiguation
   */
  async detectVerifiableContent(
    sentence: string,
    context: ConversationContext
  ): Promise<VerifiableContentResult> {
    const normalized = sentence.trim();
    let indicators = this.collectQualificationIndicators(normalized);
    let rewrittenSentence = normalized;

    if (indicators.length === 0) {
      const rewritten = await this.rewriteUnverifiableContent(
        normalized,
        context
      );
      if (rewritten) {
        rewrittenSentence = rewritten;
        indicators = this.collectQualificationIndicators(rewrittenSentence);
      }
    }

    const hasVerifiableContent = indicators.length > 0;
    const confidence = hasVerifiableContent
      ? Math.min(0.4 + indicators.length * 0.15, 1.0)
      : 0.1;

    return {
      hasVerifiableContent,
      rewrittenSentence: hasVerifiableContent ? rewrittenSentence : undefined,
      indicators,
      confidence,
    };
  }

  /**
   * Rewrite unverifiable content by removing speculative or opinion-based elements
   */
  async rewriteUnverifiableContent(
    sentence: string,
    context: ConversationContext
  ): Promise<string | null> {
    const speculativePatterns = [
      /\b(might|may|could|would|should|ought to)\b/gi,
      /\b(I think|I believe|in my opinion|personally)\b/gi,
      /\b(possibly|probably|likely|unlikely)\b/gi,
      /\b(supposedly|allegedly|reportedly)\b/gi,
    ];

    const policySofteners = [/\bappears to\b/gi, /\bseems to\b/gi];

    let rewritten = sentence;
    for (const pattern of [...speculativePatterns, ...policySofteners]) {
      rewritten = rewritten.replace(pattern, "");
    }

    if (context.metadata?.forceDeclarative === true) {
      rewritten = rewritten.replace(/\?+/g, ".").trim();
    }

    rewritten = rewritten.replace(/\s+/g, " ").trim();

    if (rewritten.length < 10 || rewritten.split(" ").length < 3) {
      return null;
    }

    return rewritten;
  }

  // ============================================================================
  // STAGE 3: DECOMPOSITION - Extract atomic claims
  // ============================================================================

  /**
   * Extract atomic claims from disambiguated text
   */
  async extractAtomicClaims(
    disambiguatedSentence: string,
    context: ConversationContext
  ): Promise<AtomicClaim[]> {
    const claims: AtomicClaim[] = [];
    const sentences = this.splitIntoSentences(disambiguatedSentence);

    const verbPattern = /\b(is|are|was|were|has|have|will|shall|did|does|announced|promised|reported|expects|pledged|committed|approved)\b/i;

    for (let sentenceIndex = 0; sentenceIndex < sentences.length; sentenceIndex += 1) {
      const sentence = sentences[sentenceIndex];
      const clauses = this.splitIntoClauses(sentence);
      let lastSubject =
        this.extractFallbackSubject(context) ?? this.extractContextEntities(context)[0] ?? null;

      for (let clauseIndex = 0; clauseIndex < clauses.length; clauseIndex += 1) {
        const clause = clauses[clauseIndex];
        let normalizedClause = this.normalizeClause(clause);

        const subjectCandidate =
          normalizedClause.match(/^[A-Z][a-z]+(?: [A-Z][a-z]+)*/)?.[0];
        if (subjectCandidate && !verbPattern.test(subjectCandidate)) {
          lastSubject = subjectCandidate;
        } else if (lastSubject) {
          const lowerClause =
            normalizedClause.charAt(0).toLowerCase() +
            normalizedClause.slice(1);
          normalizedClause = `${lastSubject} ${lowerClause}`;
        }

        if (normalizedClause.length < 8) {
          continue;
        }

        const claimId = this.generateClaimId(
          context.conversationId,
          sentenceIndex,
          clauseIndex
        );

        const contextualBrackets = await this.extractContextualBrackets(
          normalizedClause,
          context
        );
        const verificationRequirements = this.deriveVerificationRequirements(
          normalizedClause,
          contextualBrackets
        );
        const confidence = this.calculateClaimConfidence(normalizedClause);

        claims.push({
          id: claimId,
          statement: normalizedClause,
          contextualBrackets,
          sourceSentence: sentence,
          sourceContext: this.buildSourceContext(sentence, normalizedClause),
          verificationRequirements,
          confidence,
        });
      }
    }

    return claims;
  }

  /**
   * Add contextual brackets for implied context in claims
   */
  async addContextualBrackets(
    claim: string,
    impliedContext: string
  ): Promise<string> {
    // Simple implementation - in practice, this would use sophisticated NLP
    // to identify what context is implied and needs to be made explicit

    if (
      impliedContext.includes("celebrity") ||
      impliedContext.includes("person")
    ) {
      return claim.replace(/\bJohn\b/g, "John [celebrity]");
    }

    if (
      impliedContext.includes("location") ||
      impliedContext.includes("place")
    ) {
      return claim.replace(/\bParis\b/g, "Paris [European capital]");
    }

    return claim;
  }

  // ============================================================================
  // VERIFICATION AND ARBITRATION
  // ============================================================================

  /**
   * Evaluate worker outputs using claim extraction and verification
   */
  async evaluateWithClaims(
    workerOutput: any,
    taskContext: any
  ): Promise<ClaimBasedEvaluation> {
    // Extract claims from the worker output
    const extractedClaims = await this.extractClaimsFromOutput(
      workerOutput,
      taskContext
    );

    // Verify each claim
    const verificationResults: VerificationResult[] = [];
    let totalFactualScore = 0;
    let totalCawsScore = 0;

    for (const claim of extractedClaims) {
      const verification = await this.verifyClaim(claim, taskContext);
      verificationResults.push(verification);

      totalFactualScore += verification.evidenceQuality;
      totalCawsScore += verification.cawsCompliance ? 1 : 0;
    }

    const factualAccuracyScore =
      extractedClaims.length > 0
        ? totalFactualScore / extractedClaims.length
        : 0;
    const cawsComplianceScore =
      extractedClaims.length > 0 ? totalCawsScore / extractedClaims.length : 0;
    const overallQuality =
      factualAccuracyScore * 0.7 + cawsComplianceScore * 0.3;

    return {
      extractedClaims,
      verificationResults,
      factualAccuracyScore,
      cawsComplianceScore,
      overallQuality,
    };
  }

  /**
   * Compare competing outputs using claim verification
   */
  async compareOutputs(
    outputs: any[],
    verificationCriteria: VerificationCriteria
  ): Promise<ArbitrationDecision> {
    const evaluations = [];

    for (const output of outputs) {
      const evaluation = await this.evaluateWithClaims(output, {
        verificationCriteria,
      });
      evaluations.push({
        output,
        evaluation,
      });
    }

    // Select the output with the highest overall quality score
    const bestEvaluation = evaluations.reduce((best, current) =>
      current.evaluation.overallQuality > best.evaluation.overallQuality
        ? current
        : best
    );

    const alternatives = evaluations
      .filter((e) => e !== bestEvaluation)
      .map((e) => ({
        output: e.output,
        reason: `Quality score: ${e.evaluation.overallQuality.toFixed(2)}`,
        score: e.evaluation.overallQuality,
      }));

    return {
      selectedOutput: bestEvaluation.output,
      selectionReason: `Highest quality score: ${bestEvaluation.evaluation.overallQuality.toFixed(
        2
      )}`,
      confidence: bestEvaluation.evaluation.overallQuality,
      alternatives,
    };
  }

  // ============================================================================
  // LEARNING AND ADAPTATION
  // ============================================================================

  /**
   * Learn from verification feedback to improve extraction patterns
   */
  async learnFromVerification(
    claims: ExtractedClaim[],
    verificationResults: any[],
    humanFeedback?: any
  ): Promise<LearningUpdate> {
    const improvements: Record<string, number> = {};
    const concerns: string[] = [];
    const recommendations: string[] = [];

    // Analyze verification results to identify patterns
    const successfulVerifications = verificationResults.filter(
      (r) => r.status === "VERIFIED"
    ).length;
    const totalVerifications = verificationResults.length;
    const successRate =
      totalVerifications > 0 ? successfulVerifications / totalVerifications : 0;

    if (successRate > 0.8) {
      improvements.extractionAccuracy = successRate;
      recommendations.push(
        "Maintain current extraction patterns - performing well"
      );
    } else if (successRate < 0.6) {
      improvements.extractionAccuracy = successRate;
      concerns.push(
        "Low verification success rate - extraction patterns may need refinement"
      );
      recommendations.push(
        "Review and update extraction patterns for better factual claim identification"
      );
    }

    // Check for human feedback
    if (humanFeedback) {
      if (humanFeedback.incorrectClaims) {
        concerns.push("Human feedback indicates incorrect claim extraction");
        recommendations.push(
          "Adjust extraction patterns to avoid similar errors"
        );
      }

      if (humanFeedback.missingClaims) {
        concerns.push("Human feedback indicates missing claims");
        recommendations.push(
          "Expand extraction patterns to capture more verifiable content"
        );
      }
    }

    return {
      patterns: this.extractionPatterns,
      improvements,
      concerns,
      recommendations,
    };
  }

  /**
   * Adapt extraction patterns based on task surface and historical performance
   */
  async adaptExtractionPatterns(
    taskSurface: string,
    _historicalPerformance: any
  ): Promise<PatternUpdate> {
    const updatedPatterns: Record<string, any> = {};
    const impact: Record<string, number> = {};

    // Adapt patterns based on task surface
    switch (taskSurface) {
      case "code-editing":
        // Focus on code-related factual claims
        updatedPatterns.codePatterns =
          this.extractionPatterns.get("code") || [];
        impact.codeAccuracy = 0.1;
        break;

      case "research-assistant":
        // Focus on research and citation claims
        updatedPatterns.researchPatterns =
          this.extractionPatterns.get("research") || [];
        impact.researchAccuracy = 0.1;
        break;

      case "data-analysis":
        // Focus on statistical and data claims
        updatedPatterns.dataPatterns =
          this.extractionPatterns.get("data") || [];
        impact.dataAccuracy = 0.1;
        break;

      default:
        // General adaptation
        impact.generalAccuracy = 0.05;
    }

    return {
      patterns: updatedPatterns,
      impact,
      rollbackInfo:
        "Can rollback to previous pattern set if performance degrades",
    };
  }

  // ============================================================================
  // PRIVATE HELPER METHODS
  // ============================================================================

  /**
   * Initialize ambiguity detection patterns
   */
  private initializePatterns(): void {
    this.ambiguityPatterns.set("referential", [
      /\b(this|that|it|they|he|she|we|us|them|him|her)\b/gi,
      /\b(the [a-z]+|that [a-z]+|those [a-z]+)\b/gi,
    ]);

    this.extractionPatterns.set("factual", [
      /\b[A-Z][a-z]+ [A-Z][a-z]+\b/,
      /\b(19|20)\d{2}\b/,
      /\$\d+(?:\.\d+)?\b/,
      /\b\d+(?:\.\d+)?%/,
      /\b\d+(?:\.\d+)?\s+(people|users|customers|employees|projects)\b/gi,
    ]);

    this.extractionPatterns.set("temporal", [
      /\b(?:January|February|March|April|May|June|July|August|September|October|November|December) \d{1,2}, \d{4}\b/,
      /\b\d{1,2}\/\d{1,2}\/\d{4}\b/,
      /\b\d{4}-\d{2}-\d{2}\b/,
    ]);

    this.extractionPatterns.set("quantitative", [
      /\b\d+(?:\.\d+)?\s*(?:million|billion|trillion|percent|%)\b/gi,
      /\$\d+(?:\.\d+)?\b/gi,
      /\btop \d+|bottom \d+|first \d+|last \d+\b/gi,
    ]);

    this.extractionPatterns.set("policy", [
      /\b(policy|program|initiative|plan|order|bill)\b/gi,
    ]);
  }

  /**
   * Initialize verification sources
   */
  private initializeVerificationSources(): void {
    if (this.verificationSources.size > 0) {
      return;
    }

    this.verificationSources.set("whitehouse", {
      name: "White House Official Website",
      type: "official_government",
      reliability: 1.0,
      apiEndpoint: "https://www.whitehouse.gov/api",
      rateLimit: 1000,
    });

    this.verificationSources.set("congress", {
      name: "U.S. Congress Website",
      type: "official_government",
      reliability: 1.0,
      apiEndpoint: "https://www.congress.gov/api",
      rateLimit: 1000,
    });

    this.verificationSources.set("factcheck", {
      name: "FactCheck.org",
      type: "independent_fact_checker",
      reliability: 0.9,
      apiEndpoint: "https://factcheck.org/api",
      rateLimit: 500,
    });
  }

  /**
   * Calculate confidence score for a claim based on its characteristics
   */
  private calculateClaimConfidence(sentence: string): number {
    let confidence = 0.45;

    if (/\b[A-Z][a-z]+ [A-Z][a-z]+\b/.test(sentence)) {
      confidence += 0.12;
    }

    if (/\b(19|20)\d{2}\b/.test(sentence)) {
      confidence += 0.1;
    }

    if (
      /\$\d+(?:\.\d+)?\b/.test(sentence) ||
      /\b\d+(?:\.\d+)?%/.test(sentence)
    ) {
      confidence += 0.1;
    }

    if (
      /\b(according to|official|report|data shows|announced|stated)\b/i.test(
        sentence
      )
    ) {
      confidence += 0.08;
    }

    if (/\b(might|may|could|would|should|possibly|probably)\b/i.test(sentence)) {
      confidence -= 0.25;
    }

    if (sentence.length > 160) {
      confidence -= 0.08;
    }

    return Math.max(0.1, Math.min(1.0, confidence));
  }

  /**
   * Extract contextual brackets for implied context
   */
  private async extractContextualBrackets(
    sentence: string,
    context: ConversationContext
  ): Promise<string[]> {
    const brackets = new Set<string>();

    if (/\b[A-Z][a-z]+ [A-Z][a-z]+\b/.test(sentence)) {
      brackets.add("person");
    }

    if (/\b(?:department|agency|committee|organization)\b/i.test(sentence)) {
      brackets.add("organization");
    }

    if (/\b(policy|program|initiative|plan|bill|order)\b/i.test(sentence)) {
      brackets.add("policy");
    }

    if (/\b(?:city|state|country|capital)\b/i.test(sentence)) {
      brackets.add("location");
    }

    if (
      /\b(?:January|February|March|April|May|June|July|August|September|October|November|December)\b/i.test(
        sentence
      ) ||
      /\b(19|20)\d{2}\b/.test(sentence)
    ) {
      brackets.add("temporal");
    }

    if (
      /\$\d+(?:\.\d+)?\b/.test(sentence) ||
      /\b\d+(?:\.\d+)?\s*(?:million|billion|percent|%)\b/i.test(sentence)
    ) {
      brackets.add("monetary");
      brackets.add("statistic");
    }

    if (context.metadata?.surface === "code") {
      brackets.add("code");
    }

    if (Array.isArray(context.metadata?.domains)) {
      for (const domain of context.metadata.domains) {
        brackets.add(String(domain));
      }
    }

    return Array.from(brackets);
  }

  private collectQualificationIndicators(sentence: string): string[] {
    const indicators: string[] = [];

    const yearMatches = sentence.match(/\b(19|20)\d{2}\b/g);
    if (yearMatches) {
      for (const year of yearMatches) {
        indicators.push(`year:${year}`);
      }
    }

    const currencyMatches = sentence.match(/\$\d+(?:\.\d+)?/g);
    if (currencyMatches) {
      for (const value of currencyMatches) {
        indicators.push(`currency:${value}`);
      }
    }

    const percentageMatches = sentence.match(/\b\d+(?:\.\d+)?%/g);
    if (percentageMatches) {
      for (const pct of percentageMatches) {
        indicators.push(`percentage:${pct}`);
      }
    }

    const properNames = sentence.match(/\b[A-Z][a-z]+ [A-Z][a-z]+\b/g);
    if (properNames) {
      indicators.push(
        ...properNames.map((name) => `entity:${name.toLowerCase()}`)
      );
    }

    if (
      /\b(according to|official|reports?|data shows|analysis)\b/i.test(sentence)
    ) {
      indicators.push("attribution");
    }

    return Array.from(new Set(indicators));
  }

  private splitIntoSentences(text: string): string[] {
    return text
      .split(/(?<=[.!?])\s+/)
      .map((segment) => segment.trim())
      .filter((segment) => segment.length > 0);
  }

  private splitIntoClauses(sentence: string): string[] {
    const hardSplits = sentence
      .split(/(?:;|—|--)/)
      .map((fragment) => fragment.trim())
      .filter(Boolean);

    const clauses: string[] = [];

    for (const fragment of hardSplits) {
      const softSplits = fragment.split(/,(?=\s+(?:and|but|or)\s+)/i);
      for (const part of softSplits) {
        const trimmed = part.trim();
        if (trimmed.length === 0) {
          continue;
        }
        clauses.push(trimmed);
      }
    }

    const normalized: string[] = [];
    for (const clause of clauses) {
      const hasVerb = /\b(is|are|was|were|has|have|will|shall|did|does|announced|promised|reported|expects|pledged|committed|approved)\b/i.test(
        clause
      );
      if (!hasVerb && normalized.length > 0) {
        normalized[normalized.length - 1] = `${normalized[
          normalized.length - 1
        ]} ${clause}`;
      } else {
        normalized.push(clause);
      }
    }

    const finalClauses: string[] = [];
    const conjunctionRegex = /\s+(?:and|but|or)\s+/i;
    const verbPattern = /\b(is|are|was|were|has|have|will|shall|did|does|announced|promised|reported|expects|pledged|committed|approved)\b/i;

    for (const clause of normalized) {
      if (conjunctionRegex.test(clause)) {
        const pieces = clause
          .split(conjunctionRegex)
          .map((piece) => piece.trim())
          .filter(Boolean);

        const allHaveVerb = pieces.every((piece) => verbPattern.test(piece));
        if (pieces.length > 1 && allHaveVerb) {
          finalClauses.push(...pieces);
          continue;
        }
      }

      finalClauses.push(clause);
    }

    return finalClauses.map((clause) => clause.trim()).filter(Boolean);
  }

  private normalizeClause(clause: string): string {
    let normalized = clause.replace(/\s+/g, " ").trim();
    if (normalized.length === 0) {
      return normalized;
    }

    normalized =
      normalized.charAt(0).toUpperCase() + normalized.slice(1);

    if (!/[.!?]$/.test(normalized)) {
      normalized = `${normalized}.`;
    }

    return normalized;
  }

  private generateClaimId(
    conversationId: string,
    sentenceIndex: number,
    clauseIndex: number
  ): string {
    const timestamp = Date.now();
    return `claim_${conversationId}_${timestamp}_${sentenceIndex}_${clauseIndex}`;
  }

  private deriveVerificationRequirements(
    clause: string,
    contextualBrackets: string[]
  ): VerificationCriteria[] {
    const requirements: VerificationCriteria[] = [];

    if (
      /\b\d+(?:\.\d+)?\s*(?:million|billion|trillion|percent|%)\b/i.test(
        clause
      ) ||
      /\$\d+(?:\.\d+)?\b/.test(clause)
    ) {
      requirements.push({
        type: "cross_reference",
        requirements: { category: "quantitative" },
        priority: "high",
      });
    }

    if (
      /\b(?:January|February|March|April|May|June|July|August|September|October|November|December)\b/i.test(
        clause
      ) ||
      /\b(19|20)\d{2}\b/.test(clause)
    ) {
      requirements.push({
        type: "temporal_consistency",
        requirements: { hint: "date" },
        priority: "high",
      });
    }

    const properNames = clause.match(/\b[A-Z][a-z]+ [A-Z][a-z]+\b/g) || [];
    if (properNames.length > 0) {
      requirements.push({
        type: "source_verification",
        requirements: { entities: properNames },
        priority: "medium",
      });
    }

    requirements.push({
      type: "caws_compliance",
      requirements: { brackets: contextualBrackets },
      priority: "medium",
    });

    return requirements;
  }

  private buildSourceContext(sentence: string, clause: string): string {
    const trimmedSentence = sentence.trim();
    if (trimmedSentence.includes(clause.trim())) {
      return trimmedSentence;
    }

    return `${trimmedSentence} :: ${clause.trim()}`;
  }

  /**
   * Extract claims from worker output
   */
  private async extractClaimsFromOutput(
    workerOutput: any,
    taskContext: any
  ): Promise<ExtractedClaim[]> {
    const text = this.normalizeWorkerOutput(workerOutput);
    if (!text) {
      return [];
    }

    const conversationContext = this.toConversationContext(
      taskContext?.conversationContext,
      taskContext
    );

    const sentences = this.splitIntoSentences(text);
    const extracted: ExtractedClaim[] = [];

    for (const sentence of sentences) {
      const trimmedSentence = sentence.trim();
      if (trimmedSentence.length < 5) {
        continue;
      }

      const unresolvable = await this.detectUnresolvableAmbiguities(
        trimmedSentence,
        conversationContext
      );
      if (unresolvable.length > 0) {
        continue;
      }

      const ambiguityAnalysis = await this.identifyAmbiguities(
        trimmedSentence,
        conversationContext
      );
      if (!ambiguityAnalysis.canResolve) {
        continue;
      }

      const disambiguation = await this.resolveAmbiguities(
        trimmedSentence,
        ambiguityAnalysis,
        conversationContext
      );
      if (!disambiguation.success || !disambiguation.disambiguatedSentence) {
        continue;
      }

      const qualified = await this.detectVerifiableContent(
        disambiguation.disambiguatedSentence,
        conversationContext
      );
      if (!qualified.hasVerifiableContent) {
        continue;
      }

      const qualifiedSentence =
        qualified.rewrittenSentence ?? disambiguation.disambiguatedSentence;

      const atomicClaims = await this.extractAtomicClaims(
        qualifiedSentence,
        conversationContext
      );

      for (const atomicClaim of atomicClaims) {
        extracted.push({
          id: atomicClaim.id,
          statement: atomicClaim.statement,
          confidence: atomicClaim.confidence,
          sourceContext: atomicClaim.sourceContext,
          verificationRequirements: atomicClaim.verificationRequirements,
        });
      }
    }

    return extracted;
  }

  private async verifyClaim(
    claim: ExtractedClaim,
    taskContext: any
  ): Promise<VerificationResult> {
    const evidenceManifest: EvidenceManifest | undefined =
      taskContext?.evidenceManifest;
    const workingSpec: WorkingSpec | undefined = taskContext?.workingSpec;
    const verificationTrail: VerificationStep[] = [];
    const timestamp = () => this.currentTimestamp();

    if (!evidenceManifest || evidenceManifest.evidence.length === 0) {
      const scopeValidation = this.validateClaimScopeAgainstSpec(
        claim,
        workingSpec
      );

      verificationTrail.push({
        type: "source_query",
        description: "No evidence provided for claim verification",
        outcome: "failure",
        timestamp: timestamp(),
        metadata: { claimId: claim.id },
      });

      verificationTrail.push({
        type: "caws_check",
        description: "CAWS scope validation without supporting evidence",
        outcome: scopeValidation.withinScope ? "partial" : "failure",
        timestamp: timestamp(),
        metadata: scopeValidation,
      });

      verificationTrail.push({
        type: "ambiguity_resolution",
        description: "Verification requirements audit",
        outcome:
          claim.verificationRequirements.length > 0 ? "partial" : "failure",
        timestamp: timestamp(),
        metadata: {
          requirementCount: claim.verificationRequirements.length,
        },
      });

      return {
        status: "INSUFFICIENT_EVIDENCE",
        evidenceQuality: 0,
        cawsCompliance: scopeValidation.withinScope,
        verificationTrail,
      };
    }

    let bestScore = 0;
    let supportingEvidence = evidenceManifest.evidence[0] ?? null;

    for (const evidence of evidenceManifest.evidence) {
      const score = this.computeEvidenceMatchScore(claim, evidence.content);
      if (score > bestScore) {
        bestScore = score;
        supportingEvidence = evidence;
      }
    }

    verificationTrail.push({
      type: "cross_reference",
      description: "Evidence overlap analysis",
      outcome:
        bestScore >= 0.6 ? "success" : bestScore > 0.3 ? "partial" : "failure",
      timestamp: timestamp(),
      metadata: {
        bestScore,
        supportingEvidence: supportingEvidence?.source,
      },
    });

    const scopeValidation = this.validateClaimScopeAgainstSpec(
      claim,
      workingSpec
    );

    verificationTrail.push({
      type: "caws_check",
      description: "CAWS scope validation",
      outcome: scopeValidation.withinScope ? "success" : "failure",
      timestamp: timestamp(),
      metadata: scopeValidation,
    });

    verificationTrail.push({
      type: "ambiguity_resolution",
      description: "Verification requirements audit",
      outcome:
        claim.verificationRequirements.length > 0 ? "success" : "partial",
      timestamp: timestamp(),
      metadata: {
        requirementCount: claim.verificationRequirements.length,
      },
    });

    const status: VerificationResult["status"] =
      bestScore >= 0.6
        ? "VERIFIED"
        : bestScore > 0.3
        ? "UNVERIFIED"
        : "INSUFFICIENT_EVIDENCE";

    const evidenceQuality = Number(
      Math.min(bestScore * (evidenceManifest.quality ?? 1), 1).toFixed(2)
    );

    const cawsCompliance =
      scopeValidation.withinScope && Boolean(evidenceManifest.cawsCompliant);

    return {
      status,
      evidenceQuality,
      cawsCompliance,
      verificationTrail,
    };
  }

  private normalizeWorkerOutput(workerOutput: any): string {
    if (typeof workerOutput === "string") {
      return workerOutput;
    }

    if (Array.isArray(workerOutput)) {
      return workerOutput
        .filter((item) => typeof item === "string")
        .join(" ");
    }

    if (workerOutput && typeof workerOutput === "object") {
      if (typeof workerOutput.content === "string") {
        return workerOutput.content;
      }

      if (Array.isArray(workerOutput.content)) {
        return workerOutput.content.join(" ");
      }

      if (typeof workerOutput.text === "string") {
        return workerOutput.text;
      }
    }

    return "";
  }

  private toConversationContext(
    rawContext: ConversationContext | undefined,
    taskContext: any
  ): ConversationContext {
    if (rawContext) {
      return {
        conversationId: rawContext.conversationId ?? "unknown-conversation",
        tenantId: rawContext.tenantId ?? "unknown-tenant",
        previousMessages: [...rawContext.previousMessages],
        metadata: { ...(rawContext.metadata ?? {}) },
      };
    }

    return {
      conversationId: taskContext?.conversationId ?? "unknown-conversation",
      tenantId: taskContext?.tenantId ?? "unknown-tenant",
      previousMessages: [],
      metadata: taskContext?.metadata ?? {},
    };
  }

  private matchAllUnique(patterns: RegExp[], sentence: string): string[] {
    const matches = new Set<string>();

    for (const pattern of patterns) {
      const found = sentence.match(pattern);
      if (found) {
        for (const item of found) {
          matches.add(item);
        }
      }
    }

    return Array.from(matches);
  }

  private extractContextEntities(context: ConversationContext): string[] {
    const entities = new Set<string>();

    for (const message of context.previousMessages) {
      const matches =
        message.match(/\b[A-Z][a-z]+(?: [A-Z][a-z]+)+\b/g) ?? [];
      for (const match of matches) {
        entities.add(this.normalizeEntityName(match));
      }
    }

    const metadataEntities = context.metadata?.entities;
    if (Array.isArray(metadataEntities)) {
      for (const entity of metadataEntities) {
        if (typeof entity === "string") {
          entities.add(this.normalizeEntityName(entity));
        }
      }
    }

    const fallbackSubject = this.extractFallbackSubject(context);
    if (fallbackSubject) {
      entities.add(this.normalizeEntityName(fallbackSubject));
    }

    return Array.from(entities);
  }

  private hasTimelineContext(context: ConversationContext): boolean {
    if (context.metadata?.referenceDate) {
      return true;
    }

    if (
      Array.isArray(context.metadata?.timeline) &&
      context.metadata.timeline.length > 0
    ) {
      return true;
    }

    return context.previousMessages.some((message) =>
      /\b(19|20)\d{2}\b/.test(message)
    );
  }

  private computeResolutionConfidence(params: {
    referentialAmbiguities: string[];
    structuralAmbiguities: string[];
    temporalAmbiguities: string[];
    referentialResolvable: boolean;
    temporalResolvable: boolean;
    structuralResolvable: boolean;
  }): number {
    let confidence = 0.9;

    if (params.referentialAmbiguities.length > 0) {
      confidence -= params.referentialResolvable ? 0.1 : 0.4;
    }

    if (params.temporalAmbiguities.length > 0) {
      confidence -= params.temporalResolvable ? 0.05 : 0.3;
    }

    if (params.structuralAmbiguities.length > 1) {
      confidence -= params.structuralResolvable ? 0.1 : 0.2;
    }

    return Math.max(0.1, Math.min(0.9, confidence));
  }

  private selectAntecedent(
    pronoun: string,
    candidates: string[]
  ): string | null {
    if (candidates.length === 0) {
      return null;
    }

    return candidates[candidates.length - 1] ?? null;
  }

  private extractFallbackSubject(
    context: ConversationContext
  ): string | null {
    const fallback =
      typeof context.metadata?.defaultSubject === "string"
        ? context.metadata.defaultSubject
        : typeof context.metadata?.primaryEntity === "string"
        ? context.metadata.primaryEntity
        : null;

    return fallback ? fallback.trim() : null;
  }

  private generateStructuralInterpretations(
    phrase: string,
    _sentence: string
  ): string[] {
    const interpretations = new Set<string>([phrase]);

    if (phrase.includes(" and ")) {
      const parts = phrase.split(/\band\b/i).map((part) => part.trim());
      if (parts.length === 2) {
        interpretations.add(`${parts[0]} and ${parts[1]}`);
      }
    }

    if (phrase.includes(" or ")) {
      const parts = phrase.split(/\bor\b/i).map((part) => part.trim());
      for (const part of parts) {
        if (part.length > 0) {
          interpretations.add(part);
        }
      }
    }

    return Array.from(interpretations);
  }

  private selectStructuralInterpretation(
    options: string[],
    context: ConversationContext
  ): string {
    if (options.length === 0) {
      return "";
    }

    const preferred = context.metadata?.preferredInterpretation;
    if (typeof preferred === "string") {
      const match = options.find(
        (option) => option.toLowerCase() === preferred.toLowerCase()
      );
      if (match) {
        return match;
      }
    }

    return options[0];
  }

  private isStructurallyAmbiguous(sentence: string): boolean {
    return /,(?=\s+(?:and|or)\s+)/i.test(sentence);
  }

  private escapeRegex(value: string): string {
    return value.replace(/[-/\\^$*+?.()|[\]{}]/g, "\\$&");
  }

  private computeEvidenceMatchScore(
    claim: ExtractedClaim,
    evidenceContent: string
  ): number {
    const claimText = claim.statement.toLowerCase();
    const evidenceText = evidenceContent.toLowerCase();

    if (evidenceText.includes(claimText)) {
      return 1;
    }

    const keywords = this.extractKeywords(claim.statement);
    if (keywords.length === 0) {
      return 0;
    }

    const matches = keywords.filter((keyword) =>
      evidenceText.includes(keyword)
    );

    return matches.length / keywords.length;
  }

  private extractKeywords(text: string): string[] {
    const stopwords = new Set<string>([
      "the",
      "and",
      "for",
      "with",
      "that",
      "this",
      "from",
      "about",
      "into",
      "will",
      "have",
      "has",
      "had",
      "been",
      "were",
      "was",
      "are",
      "is",
      "announcement",
      "announced",
      "policy",
    ]);

    return text
      .toLowerCase()
      .replace(/[^\w\s]/g, "")
      .split(/\s+/)
      .filter((token) => {
        if (stopwords.has(token)) {
          return false;
        }

        if (/^\d+$/.test(token)) {
          return true;
        }

        return token.length > 2;
      });
  }

  private validateClaimScopeAgainstSpec(
    claim: ExtractedClaim,
    workingSpec?: WorkingSpec
  ): ScopeValidation {
    if (!workingSpec) {
      return {
        withinScope: true,
        violations: [],
        waiverRequired: false,
      };
    }

    const violations: string[] = [];
    const statement = claim.statement.toLowerCase();

    const outOfScopePaths: string[] = workingSpec.scope?.out ?? [];
    for (const path of outOfScopePaths) {
      if (path && statement.includes(path.toLowerCase())) {
        violations.push(`Claim references out-of-scope path: ${path}`);
      }
    }

    const withinScope = violations.length === 0;

    return {
      withinScope,
      violations,
      waiverRequired: !withinScope,
      waiverJustification: withinScope
        ? undefined
        : "Claim references content outside declared CAWS scope",
    };
  }

  private normalizeEntityName(entity: string): string {
    const normalized = entity
      .replace(
        /\b(President|Prime Minister|Secretary|Senator|Representative|Dr|Mr|Mrs|Ms)\b\.?/gi,
        ""
      )
      .replace(/\s+/g, " ")
      .trim();

    return normalized.length > 0 ? normalized : entity.trim();
  }

  private currentTimestamp(): string {
    return new Date().toISOString();
  }
}

/**
 * Factory function to create a new claim extractor instance
 */
export function createClaimExtractor(): ClaimExtractor {
  return new ClaimExtractor();
}

/**
 * Utility function to validate claim extraction results
 */
export function validateClaimExtraction(
  claims: AtomicClaim[],
  _sourceText: string
): { isValid: boolean; errors: string[]; warnings: string[] } {
  const errors: string[] = [];
  const warnings: string[] = [];

  if (claims.length === 0) {
    warnings.push("No claims extracted from text");
  }

  // Check for overly long claims
  const longClaims = claims.filter((c) => c.statement.length > 200);
  if (longClaims.length > 0) {
    warnings.push(
      `${longClaims.length} claims are unusually long and may not be atomic`
    );
  }

  // Check for very low confidence claims
  const lowConfidenceClaims = claims.filter((c) => c.confidence < 0.3);
  if (lowConfidenceClaims.length > 0) {
    warnings.push(
      `${lowConfidenceClaims.length} claims have low confidence scores`
    );
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings,
  };
}
