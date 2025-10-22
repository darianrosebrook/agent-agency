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
    identifyAmbiguities: (sentence: string, context: ConversationContext) =>
      this.identifyAmbiguities(sentence, context),
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
    detectVerifiableContent: (sentence: string, context: ConversationContext) =>
      this.detectVerifiableContent(sentence, context),
    rewriteUnverifiableContent: (
      sentence: string,
      context: ConversationContext
    ) => this.rewriteUnverifiableContent(sentence, context),
  };

  readonly decompositionStage = {
    extractAtomicClaims: (sentence: string, context: ConversationContext) =>
      this.extractAtomicClaims(sentence, context),
    addContextualBrackets: (claim: string, impliedContext: string) =>
      this.addContextualBrackets(claim, impliedContext),
  };

  readonly verificationStage = {
    verifyClaimEvidence: (claim: ExtractedClaim, evidence: EvidenceManifest) =>
      this.verifyClaim(claim, { evidenceManifest: evidence }),
    validateClaimScope: (claim: ExtractedClaim, workingSpec: WorkingSpec) =>
      Promise.resolve(this.validateClaimScopeAgainstSpec(claim, workingSpec)),
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
    // Enhanced pronoun and reference detection
    const pronounPatterns = [
      /\b(he|she|it|they|we|us|them|him|her)\b/gi,
      /\b(this|that|these|those)\b/gi,
    ];

    // Extract pronouns from the sentence, filtering out conjunctions
    const referentialAmbiguities: string[] = [];
    for (const pattern of pronounPatterns) {
      const matches = sentence.match(pattern);
      if (matches) {
        for (const match of matches) {
          // Filter out "that" when it's used as a conjunction (followed by a verb)
          if (match.toLowerCase() === "that") {
            const index = sentence.toLowerCase().indexOf("that");
            const afterThat = sentence.slice(index + 4).trim();
            // If followed by a verb or another pronoun, it's likely a conjunction
            if (
              /\b(is|are|was|were|has|have|will|shall|did|does|can|could|should|would|may|might|it|they|he|she|we)\b/i.test(
                afterThat
              )
            ) {
              continue; // Skip this "that" as it's a conjunction
            }
          }
          referentialAmbiguities.push(match);
        }
      }
    }

    // Remove duplicates
    const uniqueReferential = [...new Set(referentialAmbiguities)];

    // Basic structural ambiguities (unchanged)
    const structuralPatterns = [
      /\b[A-Z][a-z]+ (is|are|was|were) [a-z]+ (and|or) [a-z]+\b/gi,
      /\b[A-Z][a-z]+ (called|named|known as) [A-Z][a-z]+\b/gi,
      /\b(before|after|during|while) [a-z]+ (and|or) [a-z]+\b/gi,
    ];
    const structuralAmbiguities = this.matchAllUnique(
      structuralPatterns,
      sentence
    );

    // Temporal patterns (unchanged)
    const temporalPatterns = [
      /\b(next|last|previous|upcoming|recent|soon|recently)\b/gi,
      /\b(tomorrow|yesterday|today|now|then)\b/gi,
    ];
    const temporalAmbiguities = this.matchAllUnique(temporalPatterns, sentence);

    // Enhanced resolution checking - can resolve pronouns if we have context
    const contextEntities = this.extractContextEntities(context);
    const conversationEntities = this.extractConversationEntities(context);

    const referentialResolvable =
      uniqueReferential.length === 0 ||
      (contextEntities.length > 0 && conversationEntities.length > 0);

    const temporalResolvable =
      temporalAmbiguities.length === 0 || this.hasTimelineContext(context);
    const structuralResolvable = structuralAmbiguities.length <= 2;

    const canResolve =
      referentialResolvable && temporalResolvable && structuralResolvable;
    const resolutionConfidence = this.computeResolutionConfidence({
      referentialAmbiguities: uniqueReferential,
      structuralAmbiguities,
      temporalAmbiguities,
      referentialResolvable,
      temporalResolvable,
      structuralResolvable,
    });

    return {
      referentialAmbiguities: uniqueReferential,
      structuralAmbiguities,
      temporalAmbiguities,
      canResolve,
      resolutionConfidence,
    };
  }

  /**
   * Resolve referential ambiguities (pronouns) using conversation context
   */
  private async resolveReferentialAmbiguities(
    sentence: string,
    pronouns: string[],
    context: ConversationContext,
    auditTrail: ResolutionAttempt[]
  ): Promise<string> {
    let resolvedSentence = sentence;

    // Build a context map of potential referents
    const contextMap = this.buildReferentMap(context);

    for (const pronoun of pronouns) {
      const referent = this.findReferentForPronoun(
        pronoun.toLowerCase(),
        contextMap
      );

      if (referent) {
        // Replace pronoun with referent in the sentence
        const pronounRegex = new RegExp(`\\b${pronoun}\\b`, "gi");
        resolvedSentence = resolvedSentence.replace(
          pronounRegex,
          referent.entity
        );

        auditTrail.push({
          success: true,
          reason: `Resolved "${pronoun}" to "${referent.entity}"`,
          confidence: referent.confidence,
          strategy: "context_lookup",
          metadata: {
            pronoun,
            referent: referent.entity,
            source: referent.source,
            type: "referential",
          },
        });
      } else {
        auditTrail.push({
          success: false,
          reason: `Could not resolve pronoun "${pronoun}"`,
          confidence: 0.1,
          strategy: "context_lookup",
          metadata: { pronoun, type: "referential" },
        });
      }
    }

    return resolvedSentence;
  }

  /**
   * Build a map of potential referents from conversation context
   */
  private buildReferentMap(
    context: ConversationContext
  ): Map<string, { entity: string; confidence: number; source: string }> {
    const referentMap = new Map<
      string,
      { entity: string; confidence: number; source: string }
    >();

    // Extract from metadata participants first (highest priority)
    // Prioritize human names over system names for "he"/"she"
    if (context.metadata?.participants) {
      const humanParticipants = context.metadata.participants.filter(
        (p: string) =>
          !p.toLowerCase().includes("system") &&
          !p.toLowerCase().includes("database")
      );
      const systemParticipants = context.metadata.participants.filter(
        (p: string) =>
          p.toLowerCase().includes("system") ||
          p.toLowerCase().includes("database")
      );

      // Set human participants first for he/she
      for (const participant of humanParticipants) {
        referentMap.set("he", {
          entity: participant,
          confidence: 0.95,
          source: "metadata",
        });
        referentMap.set("she", {
          entity: participant,
          confidence: 0.95,
          source: "metadata",
        });
      }

      // Set system participants for "it" with lower priority for he/she
      for (const participant of systemParticipants) {
        referentMap.set("it", {
          entity: participant,
          confidence: 0.9,
          source: "metadata",
        });
        // Only set for he/she if no human participant was found
        if (!referentMap.has("he")) {
          referentMap.set("he", {
            entity: participant,
            confidence: 0.7,
            source: "metadata",
          });
        }
      }

      // Set "they" for all participants
      for (const participant of context.metadata.participants) {
        referentMap.set("they", {
          entity: participant,
          confidence: 0.8,
          source: "metadata",
        });
      }
    }

    // Extract entities from previous messages (most recent first)
    if (context.previousMessages) {
      for (let i = context.previousMessages.length - 1; i >= 0; i--) {
        const message = context.previousMessages[i];
        const distance = context.previousMessages.length - i; // Recency factor

        // Extract proper nouns (likely people) - only if not already set with higher confidence
        const properNouns = message.match(/\b[A-Z][a-z]+\b/g) || [];
        for (const noun of properNouns) {
          if (
            noun.match(/^(He|She|It|They|We|This|That|These|Those|The|A|An)$/i)
          )
            continue; // Skip pronouns and articles

          const confidence = Math.max(0.5, 1.0 / distance); // More recent = higher confidence
          const existingHe = referentMap.get("he");
          if (!existingHe || existingHe.confidence < confidence) {
            referentMap.set("he", {
              entity: noun,
              confidence,
              source: `message_${i}`,
            });
            referentMap.set("she", {
              entity: noun,
              confidence,
              source: `message_${i}`,
            });
          }
        }

        // Extract noun phrases that could be "it" (e.g., "database query", "the system")
        // Look for specific patterns like "the X", "this X", etc.
        const nounPhrasePatterns = [
          /\bthe\s+([a-z]+(?:\s+[a-z]+)+)\b/gi, // "the database query"
          /\bthis\s+([a-z]+(?:\s+[a-z]+)*)\b/gi, // "this system"
          /\bthat\s+([a-z]+(?:\s+[a-z]+)*)\b/gi, // "that problem"
        ];

        for (const pattern of nounPhrasePatterns) {
          const matches = message.match(pattern);
          if (matches) {
            for (const match of matches) {
              const cleanPhrase = match.trim();
              if (cleanPhrase.length > 5) {
                // Skip very short phrases
                const confidence = Math.max(0.7, 1.0 / distance); // Higher confidence for specific patterns
                const existingIt = referentMap.get("it");
                if (!existingIt || existingIt.confidence < confidence) {
                  referentMap.set("it", {
                    entity: cleanPhrase,
                    confidence,
                    source: `message_${i}`,
                  });
                }
              }
            }
          }
        }

        // Extract specific technical terms that could be "it"
        const technicalTerms =
          message.match(
            /\b(system|database|query|performance|optimization|problem|issue)\b/gi
          ) || [];
        for (const term of technicalTerms) {
          const confidence = Math.max(0.6, 1.0 / distance);
          const existingIt = referentMap.get("it");
          if (!existingIt || existingIt.confidence < confidence) {
            referentMap.set("it", {
              entity: term,
              confidence,
              source: `message_${i}`,
            });
          }
        }
      }
    }

    return referentMap;
  }

  /**
   * Find the best referent for a given pronoun
   */
  private findReferentForPronoun(
    pronoun: string,
    referentMap: Map<
      string,
      { entity: string; confidence: number; source: string }
    >
  ): { entity: string; confidence: number; source: string } | null {
    const candidates = referentMap.get(pronoun);
    return candidates || null;
  }

  /**
   * Extract entities mentioned in conversation history
   */
  private extractConversationEntities(context: ConversationContext): string[] {
    const entities = new Set<string>();

    if (context.previousMessages) {
      for (const message of context.previousMessages) {
        // Extract proper nouns (capitalized words)
        const properNouns = message.match(/\b[A-Z][a-z]+\b/g) || [];
        for (const noun of properNouns) {
          entities.add(noun.toLowerCase());
        }

        // Extract common technical entities
        const technicalTerms =
          message.match(
            /\b(system|database|query|performance|optimization)\b/gi
          ) || [];
        for (const term of technicalTerms) {
          entities.add(term.toLowerCase());
        }
      }
    }

    return Array.from(entities);
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
    let resolvedSentence = sentence;

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

    // Resolve referential ambiguities (pronouns)
    if (ambiguities.referentialAmbiguities.length > 0) {
      resolvedSentence = await this.resolveReferentialAmbiguities(
        resolvedSentence,
        ambiguities.referentialAmbiguities,
        context,
        auditTrail
      );
    }

    // Resolve temporal ambiguities (basic handling)
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

    // Check for specific unresolvable pronouns
    const pronounMatches = sentence.match(referentialPatterns);
    if (pronounMatches) {
      for (const pronoun of pronounMatches) {
        // Try to find if there's a reasonable referent for this pronoun
        const hasReferent = this.hasReferentForPronoun(
          pronoun.toLowerCase(),
          context,
          contextEntities
        );
        if (!hasReferent) {
          ambiguities.push({
            type: "referential",
            phrase: pronoun.toLowerCase(),
            reason: "insufficient_context",
            confidence: 0.8,
          });
        }
      }
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
        reason:
          "Multiple grammatical interpretations detected without disambiguating cues",
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
    const allIndicators = this.collectQualificationIndicators(normalized);

    // Separate objective and subjective indicators
    const objectiveIndicators = allIndicators.filter(
      (ind) => !ind.startsWith("subjective:")
    );
    const subjectiveIndicators = allIndicators.filter((ind) =>
      ind.startsWith("subjective:")
    );

    // Extract just the subjective words (remove the "subjective:" prefix)
    const subjectiveWords = subjectiveIndicators.map((ind) =>
      ind.replace("subjective:", "")
    );

    // For mixed content, we want to return both objective indicators and subjective words
    const mixedIndicators = [
      ...objectiveIndicators.map((ind) => {
        // Extract readable versions of prefixed indicators
        if (ind.startsWith("quantity:")) {
          return ind.replace("quantity:", "");
        }
        return ind;
      }),
      ...subjectiveWords,
    ];

    let rewrittenSentence = normalized;
    let finalIndicators = allIndicators;

    // If we have subjective content, try to rewrite it to remove subjective elements
    if (subjectiveIndicators.length > 0) {
      const rewritten = await this.rewriteUnverifiableContent(
        normalized,
        context
      );
      if (rewritten) {
        rewrittenSentence = rewritten;
        const rewrittenIndicators =
          this.collectQualificationIndicators(rewrittenSentence);
        finalIndicators = rewrittenIndicators;

        // Check remaining indicators after rewriting
        const remainingSubjective = rewrittenIndicators.filter((ind) =>
          ind.startsWith("subjective:")
        );
        const remainingObjective = rewrittenIndicators.filter(
          (ind) => !ind.startsWith("subjective:")
        );

        // If this was purely subjective content and rewriting didn't help, reject it
        if (
          objectiveIndicators.length === 0 &&
          remainingSubjective.length > 0
        ) {
          return {
            hasVerifiableContent: false,
            rewrittenSentence: undefined,
            indicators: subjectiveWords,
            confidence: 0.1,
          };
        }

        // For mixed content, we keep the rewritten version even if some subjective elements remain
        // as long as there's still objective content
        if (remainingObjective.length > 0) {
          finalIndicators = rewrittenIndicators;
        }
      }
      // If rewriting fails for mixed content, we still consider it verifiable if it has objective indicators
    }

    // Content is verifiable if it has objective indicators and no remaining subjective content
    const hasVerifiableContent = objectiveIndicators.length > 0;
    const confidence = hasVerifiableContent
      ? Math.min(0.6 + objectiveIndicators.length * 0.1, 1.0)
      : 0.1;

    // If content is unverifiable due to subjective language (no objective indicators),
    // return the subjective words as indicators
    if (!hasVerifiableContent && subjectiveIndicators.length > 0) {
      return {
        hasVerifiableContent: false,
        rewrittenSentence: undefined,
        indicators: subjectiveWords,
        confidence: 0.1,
      };
    }

    // For mixed content, return the readable mixed indicators
    const resultIndicators =
      hasVerifiableContent && subjectiveIndicators.length > 0
        ? mixedIndicators
        : finalIndicators;

    return {
      hasVerifiableContent,
      rewrittenSentence: hasVerifiableContent ? rewrittenSentence : undefined,
      indicators: resultIndicators,
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
    let rewritten = sentence;

    // Remove parenthetical opinions and asides (e.g., "which is great")
    rewritten = rewritten.replace(
      /\s*,?\s*which is (great|terrible|awesome|awful|good|bad|excellent|poor|amazing|horrible|wonderful|awful)\b/gi,
      ""
    );

    // Remove speculative adverbs
    const speculativeAdverbs =
      /\b(probably|possibly|maybe|perhaps|likely|unlikely|certainly|definitely|absolutely)\b/gi;
    rewritten = rewritten.replace(speculativeAdverbs, "");

    // Remove subjective phrases
    const subjectivePhrases = [
      /\b(I think|I believe|I feel|in my opinion|personally)\b/gi,
      /\b(the best|the worst|better|worse|great|terrible|awesome|awful)\b/gi,
      /\b(supposedly|allegedly|reportedly|appears to|seems to)\b/gi,
      /\b(might|may|could|would|should|ought to)\b/gi,
    ];

    for (const pattern of subjectivePhrases) {
      rewritten = rewritten.replace(pattern, "");
    }

    // Handle question marks if needed
    if (context.metadata?.forceDeclarative === true) {
      rewritten = rewritten.replace(/\?+/g, ".").trim();
    }

    // Clean up extra spaces, commas, and punctuation
    rewritten = rewritten
      .replace(/\s+/g, " ")
      .replace(/ ,/g, ",")
      .replace(/ ,/g, ",")
      .trim();

    // Remove multiple consecutive commas
    rewritten = rewritten.replace(/,+/g, ",").replace(/, ,/g, ",");

    // Remove leading/trailing commas and spaces
    rewritten = rewritten.replace(/^,?\s*/, "").replace(/\s*,?$/, "");

    // If the result is too short or identical, return null
    if (
      rewritten.length < 8 ||
      rewritten.split(" ").length < 3 ||
      rewritten === sentence
    ) {
      return null;
    }

    // Ensure proper punctuation
    if (!/[.!?]$/.test(rewritten)) {
      rewritten += ".";
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

    for (
      let sentenceIndex = 0;
      sentenceIndex < sentences.length;
      sentenceIndex += 1
    ) {
      const sentence = sentences[sentenceIndex];

      // First, decompose compound sentences
      const compoundClaims = this.decomposeCompoundSentence(sentence);
      let lastSubject =
        this.extractFallbackSubject(context) ??
        this.extractContextEntities(context)[0] ??
        null;

      for (
        let compoundIndex = 0;
        compoundIndex < compoundClaims.length;
        compoundIndex += 1
      ) {
        const compoundClaim = compoundClaims[compoundIndex];
        const clauses = this.splitIntoClauses(compoundClaim);
        let clauseOffset = 0;

        for (
          let clauseIndex = 0;
          clauseIndex < clauses.length;
          clauseIndex += 1
        ) {
          const clause = clauses[clauseIndex];
          let normalizedClause = this.normalizeClause(clause);

          // Extract or propagate subject
          const subjectCandidate = normalizedClause.match(
            /^[A-Z][a-z]+(?: [A-Z][a-z]+)*/
          )?.[0];
          const verbPattern =
            /\b(is|are|was|were|has|have|will|shall|did|does|announced|promised|reported|expects|pledged|committed|approved)\b/i;

          if (subjectCandidate && !verbPattern.test(subjectCandidate)) {
            lastSubject = subjectCandidate;
          } else if (
            lastSubject &&
            !normalizedClause.includes(lastSubject.split(" ")[0])
          ) {
            // Only prepend subject if it's not already present
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
            compoundIndex * 100 + clauseOffset // Ensure unique IDs
          );

          const contextualBrackets = await this.extractContextualBrackets(
            normalizedClause,
            context
          );

          // Apply contextual brackets to the statement
          let bracketedStatement = normalizedClause;
          for (const bracket of contextualBrackets) {
            // Extract the term and context from the bracket string format "term [context]"
            const match = bracket.match(/^(.+?)\s+\[(.+)\]$/);
            if (match) {
              const [, term, _context] = match;
              const regex = new RegExp(`\\b${term}\\b`, "gi");
              bracketedStatement = bracketedStatement.replace(regex, bracket);
            }
          }
          const verificationRequirements = this.deriveVerificationRequirements(
            normalizedClause,
            contextualBrackets
          );
          const confidence = this.calculateClaimConfidence(normalizedClause);

          claims.push({
            id: claimId,
            statement: bracketedStatement,
            contextualBrackets,
            sourceSentence: sentence,
            sourceContext: this.buildSourceContext(
              sentence,
              bracketedStatement
            ),
            verificationRequirements,
            confidence,
          });

          clauseOffset++;
        }
      }
    }

    return claims;
  }

  /**
   * Decompose compound sentences into separate atomic claims
   */
  private decomposeCompoundSentence(sentence: string): string[] {
    // Handle compound sentences connected by coordinating conjunctions
    const conjunctions = /\s+(and|but|or|yet|so|nor|for)\s+/i;
    const verbPattern =
      /\b(is|are|was|were|has|have|will|shall|did|does|announced|promised|reported|expects|pledged|committed|approved|supports|uses|provides|contains|includes|requires|needs|allows|enables)\b/i;

    // Split on conjunctions, but only if both parts can stand as independent claims
    if (conjunctions.test(sentence)) {
      const parts = sentence
        .split(conjunctions)
        .filter((part) => part.trim().length > 0);

      // Remove the conjunctions themselves (they appear at odd indices after split)
      const cleanParts: string[] = [];
      for (let i = 0; i < parts.length; i += 2) {
        cleanParts.push(parts[i].trim());
      }

      // Check if all parts have verbs and can be independent claims
      const allHaveVerbs = cleanParts.every((part) => verbPattern.test(part));
      const allLongEnough = cleanParts.every((part) => part.length > 10);
      const reasonableSplit = cleanParts.length >= 2 && cleanParts.length <= 4;

      if (allHaveVerbs && allLongEnough && reasonableSplit) {
        // Additional check: each part should have a clear subject-predicate structure
        const validParts = cleanParts.filter((part, index) => {
          const hasVerb = verbPattern.test(part);
          const words = part.split(/\s+/);

          // Must have at least one verb
          if (!hasVerb) return false;

          // First part should be a complete clause, subsequent parts can be shorter
          // if they inherit subject from previous parts
          if (index === 0 && words.length < 3) return false;

          return true;
        });

        if (validParts.length >= 2) {
          return validParts;
        }
      }
    }

    // If no valid decomposition, return the original sentence
    return [sentence];
  }

  /**
   * Extract contextual brackets for claims that need additional context
   */
  private async extractContextualBrackets(
    claim: string,
    context: ConversationContext
  ): Promise<string[]> {
    const brackets: string[] = [];

    // Extract context from conversation history
    const conversationText = context.previousMessages?.join(" ") || "";
    const combinedContext = conversationText.toLowerCase();

    // Common contextual patterns that need explicit brackets
    const patterns = [
      {
        // Authentication-related terms
        pattern:
          /\b(uses|requires|supports|provides)\s+(JWT|tokens?|authentication|auth|login|credentials?|passwords?|sessions?)\b/i,
        context: "authentication system",
        triggerWords: [
          "authentication",
          "security",
          "login",
          "user",
          "access",
          "auth",
        ],
      },
      {
        // Database-related terms
        pattern:
          /\b(uses|supports|provides|requires)\s+(PostgreSQL|database|DB|SQL|queries?|tables?|schemas?)\b/i,
        context: "database system",
        triggerWords: [
          "database",
          "data",
          "storage",
          "query",
          "table",
          "schema",
        ],
      },
      {
        // API-related terms
        pattern:
          /\b(uses|provides|supports|requires)\s+(API|REST|GraphQL|endpoints?|routes?|requests?|responses?)\b/i,
        context: "API system",
        triggerWords: [
          "api",
          "endpoint",
          "request",
          "response",
          "rest",
          "graphql",
        ],
      },
      {
        // Testing-related terms
        pattern:
          /\b(uses|provides|supports|requires)\s+(tests?|testing|spec|specs|assertions?|validations?)\b/i,
        context: "testing framework",
        triggerWords: ["test", "testing", "spec", "validation", "assert"],
      },
      {
        // Configuration-related terms
        pattern:
          /\b(uses|provides|supports|requires)\s+(config|configuration|settings?|options?|parameters?)\b/i,
        context: "configuration system",
        triggerWords: ["config", "configuration", "settings", "options"],
      },
    ];

    for (const { pattern, context: bracketContext, triggerWords } of patterns) {
      if (pattern.test(claim)) {
        // Check if context supports this interpretation
        const hasContextSupport = triggerWords.some((word) =>
          combinedContext.includes(word)
        );

        if (hasContextSupport) {
          // Extract the key term that needs context
          const match = claim.match(pattern);
          if (match && match[2]) {
            brackets.push(`${match[2]} [${bracketContext}]`);
          }
        }
      }
    }

    return brackets;
  }

  /**
   * Add contextual brackets for implied context in claims (legacy method)
   */
  async addContextualBrackets(
    claim: string,
    impliedContext: string
  ): Promise<string> {
    // Create a context with the implied context as previous messages
    const context: ConversationContext = {
      conversationId: "legacy",
      tenantId: "system",
      previousMessages: [impliedContext],
      metadata: {},
    };

    const brackets = await this.extractContextualBrackets(claim, context);

    // Apply the brackets to the claim
    let contextualizedClaim = claim;
    for (const bracket of brackets) {
      // Extract the term and context from the bracket string format "term [context]"
      const match = bracket.match(/^(.+?)\s+\[(.+)\]$/);
      if (match) {
        const [, term, context] = match;
        // Add the bracket at the end if it contains the term
        if (contextualizedClaim.toLowerCase().includes(term.toLowerCase())) {
          contextualizedClaim = `${contextualizedClaim} [${context}]`;
          // TODO: Implement sophisticated contextual annotation system
          // - Support multiple contextual brackets and nested annotations
          // - Implement context priority and conflict resolution
          // - Add context relevance scoring and filtering
          // - Support contextual metadata enrichment and tagging
          // - Implement context hierarchy and inheritance
          // - Add context validation and consistency checking
          // - Support context-based claim classification and routing
          // - Implement context evolution and update mechanisms
        }
      }
    }

    return contextualizedClaim;
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
      case "file_editing":
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

    if (
      /\b(might|may|could|would|should|possibly|probably)\b/i.test(sentence)
    ) {
      confidence -= 0.25;
    }

    if (sentence.length > 160) {
      confidence -= 0.08;
    }

    return Math.max(0.1, Math.min(1.0, confidence));
  }

  /**
   * Check if there's a reasonable referent for a pronoun in the conversation context
   */
  private hasReferentForPronoun(
    pronoun: string,
    context: ConversationContext,
    contextEntities: string[]
  ): boolean {
    // For "it", look for technical/system entities
    if (pronoun === "it") {
      // Check if any context entities could be technical/system references
      return contextEntities.some((entity) =>
        /\b(system|database|api|server|service|application|component|module|process|query|request|response|data|file)\b/i.test(
          entity
        )
      );
    }

    // For "they"/"them", look for plural entities or groups
    if (pronoun === "they" || pronoun === "them") {
      return contextEntities.some(
        (entity) =>
          /\b(team|users|developers|people|systems|apis|services|components|modules)\b/i.test(
            entity
          ) || contextEntities.length > 1 // Multiple entities could be "they"
      );
    }

    // For "he"/"she", look for person names
    if (pronoun === "he" || pronoun === "she") {
      return contextEntities.some(
        (entity) =>
          // Person-like entities (not technical terms)
          /^[A-Z][a-z]+(?: [A-Z][a-z]+)?$/.test(entity) &&
          !/\b(system|database|api|server|service)\b/i.test(entity)
      );
    }

    // For "this"/"that", we need some noun to refer to
    if (pronoun === "this" || pronoun === "that") {
      return contextEntities.length > 0;
    }

    return false;
  }

  private collectQualificationIndicators(sentence: string): string[] {
    const indicators: string[] = [];

    // Detect objective/factual indicators
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

    // Detect numeric quantities (e.g., "1000 requests per second")
    const quantityMatches = sentence.match(
      /\b\d+(?:\.\d+)?\s+[a-z]+(?:\s+[a-z]+)*(?:\s+per\s+[a-z]+)?\b/gi
    );
    if (quantityMatches) {
      for (const qty of quantityMatches) {
        // Only include quantities that look like measurements
        if (
          qty.match(
            /\b\d+(?:\.\d+)?\s+(?:requests?|responses?|calls?|operations?|per\s+second|per\s+minute|per\s+hour|per\s+day)/i
          )
        ) {
          indicators.push(`quantity:${qty.toLowerCase()}`);
        }
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

    // Detect HTTP status codes
    const statusCodeMatches = sentence.match(/\b\d{3}\b/g);
    if (statusCodeMatches) {
      for (const code of statusCodeMatches) {
        if (["200", "201", "400", "404", "500"].includes(code)) {
          indicators.push(`http_status:${code}`);
        }
      }
    }

    // Detect subjective/unverifiable indicators
    const subjectivePatterns = [
      /\b(probably|possibly|might|may|could|would|should|ought to)\b/gi,
      /\b(I think|I believe|I feel|in my opinion|personally)\b/gi,
      /\b(best|worst|better|worse|great|terrible|awesome|awful|impressive|amazing|excellent|poor|good|bad)\b/gi,
      /\b(supposedly|allegedly|reportedly|appears to|seems to)\b/gi,
      /\b(likely|unlikely|certainly|definitely|absolutely)\b/gi,
    ];

    for (const pattern of subjectivePatterns) {
      const matches = sentence.match(pattern);
      if (matches) {
        for (const match of matches) {
          indicators.push(`subjective:${match.toLowerCase()}`);
        }
      }
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
      const hasVerb =
        /\b(is|are|was|were|has|have|will|shall|did|does|announced|promised|reported|expects|pledged|committed|approved)\b/i.test(
          clause
        );
      if (!hasVerb && normalized.length > 0) {
        normalized[normalized.length - 1] = `${
          normalized[normalized.length - 1]
        } ${clause}`;
      } else {
        normalized.push(clause);
      }
    }

    const finalClauses: string[] = [];
    const conjunctionRegex = /\s+(?:and|but|or)\s+/i;
    const verbPattern =
      /\b(is|are|was|were|has|have|will|shall|did|does|announced|promised|reported|expects|pledged|committed|approved)\b/i;

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

    normalized = normalized.charAt(0).toUpperCase() + normalized.slice(1);

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
      return workerOutput.filter((item) => typeof item === "string").join(" ");
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

    // Extract from previous messages - both full names and single capitalized words
    for (const message of context.previousMessages) {
      // Full proper names (e.g., "John Doe")
      const fullNameMatches =
        message.match(/\b[A-Z][a-z]+(?: [A-Z][a-z]+)+\b/g) ?? [];
      for (const match of fullNameMatches) {
        entities.add(this.normalizeEntityName(match));
      }

      // Single proper nouns (e.g., "John")
      const singleNameMatches = message.match(/\b[A-Z][a-z]+\b/g) ?? [];
      for (const match of singleNameMatches) {
        entities.add(this.normalizeEntityName(match));
      }
    }

    // Extract from metadata entities
    const metadataEntities = context.metadata?.entities;
    if (Array.isArray(metadataEntities)) {
      for (const entity of metadataEntities) {
        if (typeof entity === "string") {
          entities.add(this.normalizeEntityName(entity));
        }
      }
    }

    // Extract from metadata participants (common in test contexts)
    const metadataParticipants = context.metadata?.participants;
    if (Array.isArray(metadataParticipants)) {
      for (const participant of metadataParticipants) {
        if (typeof participant === "string") {
          entities.add(this.normalizeEntityName(participant));
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

  private extractFallbackSubject(context: ConversationContext): string | null {
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
