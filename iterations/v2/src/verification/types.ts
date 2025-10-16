/**
 * @fileoverview TypeScript type definitions for the hardened verification pipeline
 *              implementing the four-stage Claimify methodology with CAWS compliance
 *              and research-based evaluation metrics.
 * @author @darianrosebrook
 */

import type { WorkingSpec } from "../types/caws-types.js";

// ============================================================================
// CORE VERIFICATION TYPES
// ============================================================================

/**
 * Represents a conversation context for claim extraction and verification
 */
export interface ConversationContext {
  /** The conversation ID for traceability */
  conversationId: string;
  /** The tenant or user ID */
  tenantId: string;
  /** Previous messages in the conversation */
  previousMessages: string[];
  /** Metadata about the conversation */
  metadata: Record<string, any>;
}

/**
 * Represents the result of verifiable content detection
 */
export interface VerifiableContentResult {
  /** Whether the content contains verifiable information */
  hasVerifiableContent: boolean;
  /** Rewritten sentence if unverifiable content was removed */
  rewrittenSentence?: string;
  /** Indicators that justified qualification */
  indicators: string[];
  /** Confidence score for the detection */
  confidence: number;
}

/**
 * Represents different types of ambiguities that can affect claim extraction
 */
export interface AmbiguityAnalysis {
  /** Referential ambiguities (e.g., "they", "the policy", "next year") */
  referentialAmbiguities: string[];
  /** Structural ambiguities (e.g., multiple grammatical interpretations) */
  structuralAmbiguities: string[];
  /** Temporal ambiguities (e.g., "next week", "recently") */
  temporalAmbiguities: string[];
  /** Whether context provides sufficient resolution */
  canResolve: boolean;
  /** Confidence in the resolution capability */
  resolutionConfidence: number;
}

/**
 * Result of disambiguation processing
 */
export interface DisambiguationResult {
  /** Whether disambiguation was successful */
  success: boolean;
  /** Disambiguated sentence if successful */
  disambiguatedSentence?: string;
  /** Reason for failure if unsuccessful */
  failureReason?: "no_ambiguity" | "cannot_resolve" | "insufficient_context";
  /** Audit trail of resolution attempts */
  auditTrail: ResolutionAttempt[];
}

/**
 * Represents an atomic, verifiable claim extracted from text
 */
export interface AtomicClaim {
  /** Unique identifier for the claim */
  id: string;
  /** The actual claim statement */
  statement: string;
  /** Contextual brackets for implied context */
  contextualBrackets: string[];
  /** The source sentence from which this claim was extracted */
  sourceSentence: string;
  /** Localized source context for traceability */
  sourceContext: string;
  /** Verification requirements inferred from the claim */
  verificationRequirements: VerificationCriteria[];
  /** Confidence score for the extraction */
  confidence: number;
}

/**
 * Represents the result of claim extraction validation
 */
export interface ValidationResult {
  /** Whether the claims are valid */
  isValid: boolean;
  /** Specific validation errors */
  errors: string[];
  /** Validation warnings */
  warnings: string[];
  /** Validation metadata */
  metadata: Record<string, any>;
}

/**
 * Represents an unresolvable ambiguity that prevents claim extraction
 */
export interface UnresolvableAmbiguity {
  /** Type of ambiguity */
  type: "referential" | "structural" | "temporal";
  /** The ambiguous phrase or structure */
  phrase: string;
  /** Why it cannot be resolved */
  reason: string;
  /** Confidence that this is unresolvable */
  confidence: number;
}

// ============================================================================
// VERIFICATION AND EVIDENCE TYPES
// ============================================================================

/**
 * Represents verification criteria for a claim
 */
export interface VerificationCriteria {
  /** Type of verification required */
  type:
    | "source_verification"
    | "cross_reference"
    | "temporal_consistency"
    | "caws_compliance";
  /** Specific requirements for this verification type */
  requirements: Record<string, any>;
  /** Priority level for this verification */
  priority: "high" | "medium" | "low";
}

/**
 * Represents an extracted claim with verification requirements
 */
export interface ExtractedClaim {
  /** Unique identifier for the claim */
  id: string;
  /** The claim statement */
  statement: string;
  /** Confidence in the extraction */
  confidence: number;
  /** Source context for the claim */
  sourceContext: string;
  /** Verification requirements for this claim */
  verificationRequirements: VerificationCriteria[];
}

/**
 * Represents evidence collected during verification
 */
export interface EvidenceManifest {
  /** Sources consulted for verification */
  sources: EvidenceSource[];
  /** Evidence items collected */
  evidence: EvidenceItem[];
  /** Overall evidence quality score */
  quality: number;
  /** Whether evidence meets CAWS requirements */
  cawsCompliant: boolean;
}

/**
 * Represents a verification source
 */
export interface EvidenceSource {
  /** Source name/identifier */
  name: string;
  /** Source type (e.g., "api", "database", "website") */
  type: string;
  /** Source reliability score */
  reliability: number;
  /** Source recency (last updated) */
  lastUpdated?: string;
  /** Response time for this source */
  responseTime: number;
}

/**
 * Represents an individual piece of evidence
 */
export interface EvidenceItem {
  /** Evidence content or summary */
  content: string;
  /** Source of this evidence */
  source: string;
  /** Evidence strength/confidence */
  strength: number;
  /** Evidence timestamp */
  timestamp: string;
  /** Evidence metadata */
  metadata: Record<string, any>;
}

/**
 * Represents the result of claim verification
 */
export interface VerificationResult {
  /** Verification status */
  status: "VERIFIED" | "UNVERIFIED" | "INSUFFICIENT_EVIDENCE";
  /** Quality of evidence found */
  evidenceQuality: number;
  /** Whether verification meets CAWS standards */
  cawsCompliance: boolean;
  /** Detailed verification trail */
  verificationTrail: VerificationStep[];
}

/**
 * Represents a step in the verification process
 */
export interface VerificationStep {
  /** Step type */
  type:
    | "source_query"
    | "cross_reference"
    | "caws_check"
    | "ambiguity_resolution";
  /** Step description */
  description: string;
  /** Step outcome */
  outcome: "success" | "failure" | "partial";
  /** Step timestamp */
  timestamp: string;
  /** Step metadata */
  metadata: Record<string, any>;
}

/**
 * Represents CAWS compliance validation for a claim
 */
export interface ScopeValidation {
  /** Whether claim is within CAWS scope */
  withinScope: boolean;
  /** Scope violations if any */
  violations: string[];
  /** Waiver requirements if scope exceeded */
  waiverRequired: boolean;
  /** Waiver justification if required */
  waiverJustification?: string;
}

// ============================================================================
// EVALUATION AND METRICS TYPES
// ============================================================================

/**
 * Research-based claim evaluation metrics
 */
export interface ResearchBasedClaimEvaluation {
  /** Coverage metrics from Metropolitansky & Larson 2025 */
  coverageMetrics: {
    /** Percentage of factual content successfully extracted */
    factualCoverage: number;
    /** Percentage of semantic meaning preserved */
    semanticCoverage: number;
    /** Percentage of contextual information retained */
    contextualCoverage: number;
  };

  /** Decontextualization quality metrics */
  decontextualizationMetrics: {
    /** Claims can be understood without source context */
    selfContainment: number;
    /** Reduction in ambiguous interpretations */
    ambiguityReduction: number;
    /** Percentage of original precision maintained */
    precisionRetention: number;
  };

  /** Automated evaluation methods */
  automatedEvaluation: {
    /** Automated assessment of claim completeness */
    claimCompleteness: number;
    /** Automated fact-checking against knowledge bases */
    factualAccuracy: number;
    /** Internal consistency across extracted claims */
    consistencyScore: number;
  };
}

/**
 * Scalable claim evaluation configuration
 */
export interface ScalableClaimEvaluation {
  /** Replicable evaluation methods from research */
  evaluationPipeline: {
    /** Process multiple claims simultaneously */
    batchProcessing: boolean;
    /** Parallel fact-checking across claims */
    parallelVerification: boolean;
    /** Cache verification results strategy */
    cachingStrategy: "semantic" | "exact" | "hybrid";
  };

  /** Research-backed quality thresholds */
  qualityThresholds: {
    /** Minimum factual coverage required (85%) */
    minimumCoverage: number;
    /** Minimum self-containment required (80%) */
    minimumDecontextualization: number;
    /** Maximum ambiguous claims allowed (<15%) */
    maximumAmbiguity: number;
  };

  /** Automated quality gates */
  qualityGates: {
    /** Validate input before extraction */
    preExtractionValidation: boolean;
    /** Verify claims after extraction */
    postExtractionValidation: boolean;
    /** Real-time quality monitoring */
    continuousMonitoring: boolean;
  };
}

/**
 * Enhanced claim quality metrics dashboard
 */
export interface EnhancedClaimQualityMetrics {
  /** Research-based extraction quality metrics */
  extractionMetrics: {
    /** Percentage of correctly extracted claims */
    accuracy: number;
    /** Percentage of verifiable content captured */
    completeness: number;
    /** Percentage of extracted claims that are factual */
    precision: number;
    /** Percentage of factual content successfully extracted */
    recall: number;
    /** Research-based coverage metric */
    coverage: number;
    /** Research-based self-containment metric */
    decontextualization: number;
  };

  /** Enhanced verification performance metrics */
  verificationMetrics: {
    /** Percentage of claims successfully verified */
    verificationRate: number;
    /** Average evidence strength */
    evidenceQuality: number;
    /** Percentage of claims meeting CAWS standards */
    cawsCompliance: number;
    /** Percentage of ambiguities successfully resolved */
    ambiguityResolution: number;
    /** Percentage of context preserved in claims */
    contextualPreservation: number;
  };

  /** System health indicators with research backing */
  healthIndicators: {
    /** Percentage of outputs containing unverified claims */
    hallucinationRate: number;
    /** Consistency across multiple extractions */
    claimConsistency: number;
    /** Average claim extraction time */
    processingLatency: number;
    /** Reliability of automated evaluation */
    evaluationReliability: number;
    /** Performance under load */
    scalabilityMetrics: number;
  };
}

// ============================================================================
// VERIFICATION PIPELINE INTERFACES
// ============================================================================

/**
 * Main interface for the claim extraction and verification processor
 */
export interface ClaimExtractionAndVerificationProcessor {
  /** Stage 1: Contextual disambiguation */
  disambiguationStage: {
    /** Identify ambiguities in a sentence */
    identifyAmbiguities(
      _sentence: string,
      _context: ConversationContext
    ): Promise<AmbiguityAnalysis>;

    /** Resolve identified ambiguities */
    resolveAmbiguities(
      _sentence: string,
      _ambiguities: AmbiguityAnalysis,
      _context: ConversationContext
    ): Promise<DisambiguationResult>;

    /** Identify ambiguities that cannot be resolved */
    detectUnresolvableAmbiguities(
      _sentence: string,
      _context: ConversationContext
    ): Promise<UnresolvableAmbiguity[]>;
  };

  /** Stage 2: Verifiable content qualification */
  qualificationStage: {
    /** Detect verifiable content in a sentence */
    detectVerifiableContent(
      _sentence: string,
      _context: ConversationContext
    ): Promise<VerifiableContentResult>;

    /** Rewrite unverifiable content */
    rewriteUnverifiableContent(
      _sentence: string,
      _context: ConversationContext
    ): Promise<string | null>;
  };

  /** Stage 3: Atomic claim decomposition */
  decompositionStage: {
    /** Extract atomic claims from disambiguated text */
    extractAtomicClaims(
      _disambiguatedSentence: string,
      _context: ConversationContext
    ): Promise<AtomicClaim[]>;

    /** Add contextual brackets for implied context */
    addContextualBrackets(
      _claim: string,
      _impliedContext: string
    ): Promise<string>;
  };

  /** Stage 4: CAWS-compliant verification coordination */
  verificationStage: {
    /** Verify claims against CAWS evidence requirements */
    verifyClaimEvidence(
      _claim: ExtractedClaim,
      _evidence: EvidenceManifest
    ): Promise<VerificationResult>;

    /** Check claim compliance with CAWS budgets */
    validateClaimScope(
      _claim: ExtractedClaim,
      _workingSpec: WorkingSpec
    ): Promise<ScopeValidation>;
  };
}

/**
 * Interface for handling different types of ambiguities
 */
export interface AmbiguityHandler {
  /** Identify unresolvable ambiguities */
  detectUnresolvableAmbiguities(
    _sentence: string,
    _context: ConversationContext
  ): Promise<UnresolvableAmbiguity[]>;

  /** Handle referential ambiguity */
  handleReferentialAmbiguity(
    _ambiguousPhrase: string,
    _context: ConversationContext
  ): Promise<ResolutionAttempt>;

  /** Handle structural ambiguity */
  handleStructuralAmbiguity(
    _sentence: string,
    _possibleInterpretations: string[],
    _context: ConversationContext
  ): Promise<ResolutionAttempt>;
}

/**
 * Represents an attempt to resolve ambiguity
 */
export interface ResolutionAttempt {
  /** Whether resolution was successful */
  success: boolean;
  /** Resolved interpretation if successful */
  resolvedInterpretation?: string;
  /** Reason for failure if unsuccessful */
  reason?: string;
  /** Confidence in the resolution */
  confidence: number;
  /** Resolution strategy employed */
  strategy?:
    | "context_lookup"
    | "pattern_inference"
    | "surface_hint"
    | "fallback";
  /** Additional notes or metadata */
  metadata?: Record<string, any>;
}

/**
 * Interface for CAWS-compliant claim verification
 */
export interface CAWSClaimVerification {
  /** Verify claims against CAWS evidence requirements */
  verifyClaimEvidence(
    _claim: ExtractedClaim,
    _evidence: EvidenceManifest
  ): Promise<VerificationResult>;

  /** Check claim compliance with CAWS budgets */
  validateClaimScope(
    _claim: ExtractedClaim,
    _workingSpec: WorkingSpec
  ): Promise<ScopeValidation>;
}

/**
 * Interface for claim-based arbitration
 */
export interface ClaimBasedArbiter {
  /** Evaluate worker outputs using claim extraction */
  evaluateWithClaims(
    _workerOutput: any,
    _taskContext: any
  ): Promise<ClaimBasedEvaluation>;

  /** Compare competing outputs using claim verification */
  compareOutputs(
    _outputs: any[],
    _verificationCriteria: VerificationCriteria
  ): Promise<ArbitrationDecision>;
}

/**
 * Represents claim-based evaluation results
 */
export interface ClaimBasedEvaluation {
  /** Extracted claims from the output */
  extractedClaims: ExtractedClaim[];
  /** Verification results for each claim */
  verificationResults: VerificationResult[];
  /** Overall factual accuracy score */
  factualAccuracyScore: number;
  /** CAWS compliance score */
  cawsComplianceScore: number;
  /** Overall quality score */
  overallQuality: number;
}

/**
 * Represents an arbitration decision
 */
export interface ArbitrationDecision {
  /** The selected output */
  selectedOutput: any;
  /** Reason for selection */
  selectionReason: string;
  /** Confidence in the decision */
  confidence: number;
  /** Alternative outputs considered */
  alternatives: Array<{
    output: any;
    reason: string;
    score: number;
  }>;
}

// ============================================================================
// MULTI-MODAL CLAIM PROCESSING
// ============================================================================

/**
 * Interface for multi-modal claim processing
 */
export interface MultiModalClaimProcessor {
  /** Extract claims from code outputs */
  extractCodeClaims(
    _codeOutput: any,
    _specification: any
  ): Promise<CodeClaim[]>;

  /** Extract claims from documentation */
  extractDocumentationClaims(
    _docOutput: any,
    _styleGuide: any
  ): Promise<DocumentationClaim[]>;

  /** Extract claims from data analysis outputs */
  extractDataClaims(
    _analysisOutput: any,
    _dataSchema: any
  ): Promise<DataClaim[]>;
}

/**
 * Represents a claim extracted from code
 */
export interface CodeClaim {
  /** The code statement or behavior */
  statement: string;
  /** Code location or context */
  codeContext: string;
  /** Specification requirement it fulfills */
  requirement: string;
  /** Test evidence for the claim */
  testEvidence: string[];
}

/**
 * Represents a claim extracted from documentation
 */
export interface DocumentationClaim {
  /** The documentation statement */
  statement: string;
  /** Documentation section */
  section: string;
  /** Style guide compliance */
  styleCompliance: boolean;
  /** Accuracy against implementation */
  implementationAccuracy: number;
}

/**
 * Represents a claim extracted from data analysis
 */
export interface DataClaim {
  /** The analytical statement */
  statement: string;
  /** Data source and methodology */
  dataContext: string;
  /** Statistical confidence */
  statisticalConfidence: number;
  /** Schema compliance */
  schemaCompliance: boolean;
}

// ============================================================================
// LEARNING AND ADAPTATION
// ============================================================================

/**
 * Interface for claim learning system
 */
export interface ClaimLearningSystem {
  /** Learn from verification feedback */
  learnFromVerification(
    _claims: ExtractedClaim[],
    _verificationResults: VerificationResult[],
    _humanFeedback?: any
  ): Promise<LearningUpdate>;

  /** Adapt extraction patterns based on task surface */
  adaptExtractionPatterns(
    _taskSurface: string,
    _historicalPerformance: any
  ): Promise<PatternUpdate>;
}

/**
 * Represents a learning update from verification feedback
 */
export interface LearningUpdate {
  /** Updated extraction patterns */
  patterns: Record<string, any>;
  /** Performance improvements */
  improvements: Record<string, number>;
  /** Areas needing attention */
  concerns: string[];
  /** Next learning cycle recommendations */
  recommendations: string[];
}

/**
 * Represents pattern updates for extraction
 */
export interface PatternUpdate {
  /** Updated patterns */
  patterns: Record<string, any>;
  /** Performance impact */
  impact: Record<string, number>;
  /** Rollback information */
  rollbackInfo: string;
}
