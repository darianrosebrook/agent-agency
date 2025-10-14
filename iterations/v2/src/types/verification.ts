/**
 * Verification Engine types for information validation and fact-checking
 * @author @darianrosebrook
 */

export interface VerificationRequest {
  id: string;
  content: string;
  source?: string;
  context?: string;
  priority: VerificationPriority;
  timeoutMs?: number;
  verificationTypes?: VerificationType[];
  metadata?: Record<string, any>;
}

export enum VerificationPriority {
  LOW = "low",
  MEDIUM = "medium",
  HIGH = "high",
  CRITICAL = "critical",
}

export enum VerificationType {
  FACT_CHECKING = "fact_checking",
  SOURCE_CREDIBILITY = "source_credibility",
  CROSS_REFERENCE = "cross_reference",
  CONSISTENCY_CHECK = "consistency_check",
  LOGICAL_VALIDATION = "logical_validation",
  STATISTICAL_VALIDATION = "statistical_validation",
}

export interface VerificationResult {
  requestId: string;
  verdict: VerificationVerdict;
  confidence: number;
  reasoning: string[];
  supportingEvidence: Evidence[];
  contradictoryEvidence: Evidence[];
  verificationMethods: VerificationMethodResult[];
  processingTimeMs: number;
  error?: string;
}

export enum VerificationVerdict {
  VERIFIED_TRUE = "verified_true",
  VERIFIED_FALSE = "verified_false",
  PARTIALLY_TRUE = "partially_true",
  UNVERIFIED = "unverified",
  CONTRADICTORY = "contradictory",
  INSUFFICIENT_DATA = "insufficient_data",
  MIXED = "mixed",
  ERROR = "error",
}

export interface Evidence {
  source: string;
  content: string;
  relevance: number;
  credibility: number;
  supporting: boolean;
  verificationDate: Date;
  metadata?: Record<string, any>;
}

export interface VerificationMethodResult {
  method: VerificationType;
  verdict: VerificationVerdict;
  confidence: number;
  reasoning: string | string[];
  processingTimeMs: number;
  evidenceCount: number;
  metadata?: Record<string, any>;
}

export interface VerificationEngineConfig {
  defaultTimeoutMs: number;
  maxConcurrentVerifications: number;
  minConfidenceThreshold: number;
  maxEvidencePerMethod: number;
  methods: VerificationMethodConfig[];
  cacheEnabled: boolean;
  cacheTtlMs: number;
  retryAttempts: number;
  retryDelayMs: number;
}

export interface VerificationMethodConfig {
  type: VerificationType;
  enabled: boolean;
  priority: number;
  timeoutMs: number;
  config: Record<string, any>;
}

// Fact checking interfaces
export interface FactCheckClaim {
  id?: string; // Optional identifier for the claim
  text: string;
  context?: string;
  language?: string;
  category?: string;
}

export interface FactCheckResult {
  claim: FactCheckClaim;
  verdict: VerificationVerdict;
  confidence: number;
  explanation: string;
  sources: FactCheckSource[];
  relatedClaims: RelatedClaim[];
  processingTimeMs?: number;
}

export interface FactCheckSource {
  url: string;
  title: string;
  publisher: string;
  credibilityScore: number;
  publishDate?: Date;
  excerpt?: string;
}

export interface RelatedClaim {
  text: string;
  similarity: number;
  verdict?: VerificationVerdict;
}

// Source credibility interfaces
export interface SourceAnalysis {
  url: string;
  domain: string;
  credibilityScore: number;
  factors: CredibilityFactor[];
  analysisDate: Date;
  cacheExpiry?: Date;
}

export interface CredibilityFactor {
  name: string;
  score: number;
  weight: number;
  explanation: string;
  evidence: string[];
}

// Cross-reference interfaces
export interface CrossReferenceRequest {
  claim: string;
  sources: string[];
  maxReferences: number;
}

export interface CrossReferenceResult {
  claim: string;
  consensus: VerificationVerdict;
  confidence: number;
  agreements: number;
  disagreements: number;
  totalSources: number;
  sourceResults: SourceVerificationResult[];
}

export interface SourceVerificationResult {
  source: string;
  verdict: VerificationVerdict;
  confidence: number;
  reasoning: string;
}

// Consistency checking interfaces
export interface ConsistencyCheck {
  statements: string[];
  context?: string;
}

export interface ConsistencyResult {
  isConsistent: boolean;
  confidence: number;
  contradictions: Contradiction[];
  explanations: string[];
}

export interface Contradiction {
  statement1: string;
  statement2: string;
  explanation: string;
  severity: number;
}

// Logical validation interfaces
export interface LogicalValidationRequest {
  premise: string[];
  conclusion: string;
  logicType?: LogicType;
}

export enum LogicType {
  DEDUCTIVE = "deductive",
  INDUCTIVE = "inductive",
  ABDUCTIVE = "abductive",
}

export interface LogicalValidationResult {
  isValid: boolean;
  confidence: number;
  logicType: LogicType;
  reasoning: string[];
  counterExamples?: string[];
}

// Statistical validation interfaces
export interface StatisticalClaim {
  statement: string;
  claimedValue: number;
  unit?: string;
  timeFrame?: string;
}

export interface StatisticalValidationResult {
  claim: StatisticalClaim;
  verdict: VerificationVerdict;
  confidence: number;
  actualValue?: number;
  source?: string;
  marginOfError?: number;
  sampleSize?: number;
  methodology?: string;
}

// Verification engine interfaces
export interface VerificationEngine {
  verify(request: VerificationRequest): Promise<VerificationResult>;
  verifyBatch(requests: VerificationRequest[]): Promise<VerificationResult[]>;
  getSupportedMethods(): VerificationType[];
  getMethodStatus(method: VerificationType): MethodStatus;
  healthCheck(): Promise<EngineHealth>;
}

export interface MethodStatus {
  type: VerificationType;
  enabled: boolean;
  healthy: boolean;
  lastUsed?: Date;
  successRate?: number;
  averageProcessingTime?: number;
}

export interface EngineHealth {
  healthy: boolean;
  totalMethods: number;
  enabledMethods: number;
  healthyMethods: number;
  cacheSize: number;
  activeVerifications: number;
}

// Error handling
export class VerificationError extends Error {
  constructor(
    message: string,
    public code: VerificationErrorCode,
    public requestId?: string,
    public method?: VerificationType
  ) {
    super(message);
    this.name = "VerificationError";
  }
}

export enum VerificationErrorCode {
  INVALID_REQUEST = "invalid_request",
  METHOD_UNAVAILABLE = "method_unavailable",
  TIMEOUT = "timeout",
  INSUFFICIENT_DATA = "insufficient_data",
  EXTERNAL_SERVICE_ERROR = "external_service_error",
  CONFIGURATION_ERROR = "configuration_error",
  RATE_LIMIT_EXCEEDED = "rate_limit_exceeded",
}

// Caching interfaces
export interface VerificationCacheEntry {
  key: string;
  result: VerificationResult;
  timestamp: Date;
  ttlMs: number;
  accessCount: number;
  lastAccessed: Date;
}

// Configuration interfaces
export interface VerificationProvider {
  readonly type: VerificationType;
  verify(request: VerificationRequest): Promise<VerificationMethodResult>;
  isAvailable(): Promise<boolean>;
  getHealth(): Promise<MethodStatus>;
}

export interface VerificationProviderFactory {
  createProvider(type: VerificationType, config: any): VerificationProvider;
  getSupportedTypes(): VerificationType[];
}
