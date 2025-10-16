/**
 * Arbitration Protocol Types
 *
 * @author @darianrosebrook
 *
 * Core type definitions for the CAWS Arbitration Protocol Engine.
 * Provides constitutional rule enforcement, debate coordination, verdict generation,
 * and waiver management.
 */

/**
 * Severity of constitutional violation
 */
export enum ViolationSeverity {
  MINOR = "minor",
  MODERATE = "moderate",
  MAJOR = "major",
  CRITICAL = "critical",
}

/**
 * Constitutional rule category
 */
export enum RuleCategory {
  CODE_QUALITY = "code_quality",
  TESTING = "testing",
  SECURITY = "security",
  PERFORMANCE = "performance",
  DOCUMENTATION = "documentation",
  DEPLOYMENT = "deployment",
  BUDGET = "budget",
  PROCESS = "process",
  RESOURCE_MANAGEMENT = "resource_management",
}

/**
 * Verdict outcome
 */
export enum VerdictOutcome {
  APPROVED = "approved",
  REJECTED = "rejected",
  CONDITIONAL = "conditional",
  WAIVED = "waived",
  APPEALED = "appealed",
}

/**
 * Waiver status
 */
export enum WaiverStatus {
  PENDING = "pending",
  APPROVED = "approved",
  REJECTED = "rejected",
  EXPIRED = "expired",
  REVOKED = "revoked",
}

/**
 * Arbitration session state
 */
export enum ArbitrationState {
  INITIALIZED = "initialized",
  RULE_EVALUATION = "rule_evaluation",
  DEBATE_IN_PROGRESS = "debate_in_progress",
  EVIDENCE_COLLECTION = "evidence_collection",
  VERDICT_GENERATION = "verdict_generation",
  WAIVER_EVALUATION = "waiver_evaluation",
  APPEAL_REVIEW = "appeal_review",
  COMPLETED = "completed",
  FAILED = "failed",
}

/**
 * Appeal status
 */
export enum AppealStatus {
  SUBMITTED = "submitted",
  UNDER_REVIEW = "under_review",
  UPHELD = "upheld",
  OVERTURNED = "overturned",
  FINALIZED = "finalized",
  WITHDRAWN = "withdrawn",
}

/**
 * Constitutional rule definition
 */
export interface ConstitutionalRule {
  /** Unique rule ID */
  id: string;

  /** Rule version */
  version: string;

  /** Rule category */
  category: RuleCategory;

  /** Human-readable title */
  title: string;

  /** Detailed description */
  description: string;

  /** Rule condition/predicate */
  condition: string;

  /** Default violation severity */
  severity: ViolationSeverity;

  /** Whether rule can be waived */
  waivable: boolean;

  /** Required evidence types for compliance */
  requiredEvidence: string[];

  /** Precedents referencing this rule */
  precedents: string[];

  /** When rule was added */
  effectiveDate: Date;

  /** When rule expires (if applicable) */
  expirationDate?: Date;

  /** Metadata and tags */
  metadata: Record<string, any>;
}

/**
 * Constitutional violation detection
 */
export interface ConstitutionalViolation {
  /** Unique violation ID */
  id: string;

  /** Rule that was violated */
  ruleId: string;

  /** Violation severity */
  severity: ViolationSeverity;

  /** Description of the violation */
  description: string;

  /** Evidence of the violation */
  evidence: string[];

  /** Location in code/system */
  location?: {
    file?: string;
    line?: number;
    function?: string;
  };

  /** When violation was detected */
  detectedAt: Date;

  /** Agent that committed the violation */
  violator?: string;

  /** Context about the violation */
  context: Record<string, any>;
}

/**
 * Verdict reasoning step
 */
export interface ReasoningStep {
  /** Step number */
  step: number;

  /** Description of reasoning */
  description: string;

  /** Evidence supporting this step */
  evidence: string[];

  /** Constitutional rule references */
  ruleReferences: string[];

  /** Confidence in this reasoning (0-1) */
  confidence: number;
}

/**
 * Constitutional verdict
 */
export interface Verdict {
  /** Unique verdict ID */
  id: string;

  /** Arbitration session ID */
  sessionId: string;

  /** Verdict outcome */
  outcome: VerdictOutcome;

  /** Complete reasoning chain */
  reasoning: ReasoningStep[];

  /** Constitutional rules applied */
  rulesApplied: string[];

  /** Evidence considered */
  evidence: string[];

  /** Precedents cited */
  precedents: string[];

  /** Conditions for conditional approval */
  conditions?: string[];

  /** Time-bounded expiration */
  expiresAt?: Date;

  /** Confidence in verdict (0-1) */
  confidence: number;

  /** Issuing authority/agent */
  issuedBy: string;

  /** When verdict was issued */
  issuedAt: Date;

  /** Audit trail */
  auditLog: Array<{
    timestamp: Date;
    action: string;
    actor: string;
    details: string;
  }>;
}

/**
 * Waiver request
 */
export interface WaiverRequest {
  /** Unique waiver ID */
  id: string;

  /** Rule to waive */
  ruleId: string;

  /** Requesting agent */
  requestedBy: string;

  /** Justification for waiver */
  justification: string;

  /** Supporting evidence */
  evidence: string[];

  /** Requested duration in milliseconds */
  requestedDuration: number;

  /** When request was made */
  requestedAt: Date;

  /** Context for the request */
  context: Record<string, any>;
}

/**
 * Waiver decision
 */
export interface WaiverDecision {
  /** Waiver request ID */
  requestId: string;

  /** Decision status */
  status: WaiverStatus;

  /** Reasoning for decision */
  reasoning: string;

  /** Approved duration (if approved) */
  approvedDuration?: number;

  /** Expiration time (if approved) */
  expiresAt?: Date;

  /** Conditions for approval */
  conditions?: string[];

  /** Decided by */
  decidedBy: string;

  /** When decision was made */
  decidedAt: Date;

  /** Automatic expiration handler */
  autoRevokeAt?: Date;
}

/**
 * Precedent applicability criteria
 */
export type PrecedentApplicability = {
  category: RuleCategory;
  severity: ViolationSeverity;
  conditions: string[];
};

/**
 * Precedent record
 */
export interface Precedent {
  /** Unique precedent ID */
  id: string;

  /** Case title/description */
  title: string;

  /** Constitutional rules involved */
  rulesInvolved: string[];

  /** Verdict issued */
  verdict: Verdict;

  /** Key facts of the case */
  keyFacts: string[];

  /** Reasoning summary */
  reasoningSummary: string;

  /** Applicability criteria */
  applicability: PrecedentApplicability;

  /** Times this precedent has been cited */
  citationCount: number;

  /** Last cited date */
  lastCitedAt?: Date;

  /** Created date */
  createdAt: Date;

  /** Precedent metadata */
  metadata: Record<string, any>;
}

/**
 * Arbitration session
 */
export interface ArbitrationSession {
  /** Unique session ID */
  id: string;

  /** Current state */
  state: ArbitrationState;

  /** Violation being arbitrated */
  violation: ConstitutionalViolation;

  /** Constitutional rules being evaluated */
  rulesEvaluated: ConstitutionalRule[];

  /** Evidence collected */
  evidence: string[];

  /** Participating agents */
  participants: string[];

  /** Debate session ID (if debate initiated) */
  debateSessionId?: string;

  /** Waiver request (if applicable) */
  waiverRequest?: WaiverRequest;

  /** Generated verdict */
  verdict?: Verdict;

  /** Related precedents */
  precedents: Precedent[];

  /** Session start time */
  startTime: Date;

  /** Session end time */
  endTime?: Date;

  /** Session metadata */
  metadata: Record<string, any>;
}

/**
 * Appeal request for arbitration
 */
export interface Appeal {
  /** Unique appeal ID */
  id: string;

  /** Arbitration session ID */
  sessionId: string;

  /** Original verdict ID being appealed */
  originalVerdictId: string;

  /** Appellant agent ID */
  appellantId: string;

  /** Grounds for appeal */
  grounds: string;

  /** New evidence provided */
  newEvidence: string[];

  /** Appeal status */
  status: AppealStatus;

  /** Appeal level (1, 2, 3, etc.) */
  level: number;

  /** Reviewers assigned */
  reviewers?: string[];

  /** When appeal was submitted */
  submittedAt: Date;

  /** When review completed */
  reviewedAt?: Date;

  /** Appeal metadata */
  metadata: Record<string, any>;
}

/**
 * @deprecated Use Appeal instead
 * Appeal request for arbitration
 */
export interface ArbitrationAppeal {
  /** Unique appeal ID */
  id: string;

  /** Verdict being appealed */
  verdictId: string;

  /** Original session */
  sessionId: string;

  /** Appealing agent */
  appellant: string;

  /** Grounds for appeal */
  grounds: string;

  /** Additional evidence */
  newEvidence: string[];

  /** When appeal was filed */
  filedAt: Date;

  /** Appeal status */
  status: "pending" | "accepted" | "rejected" | "withdrawn";

  /** Review outcome */
  reviewOutcome?: {
    decision: "uphold" | "overturn" | "modify";
    reasoning: string;
    revisedVerdict?: Verdict;
    reviewedBy: string;
    reviewedAt: Date;
  };
}

/**
 * Constitutional rule engine configuration
 */
export interface RuleEngineConfig {
  /** Strictness mode */
  strictMode: boolean;

  /** Enable precedent-based interpretation */
  usePrecedents: boolean;

  /** Maximum rule evaluation time */
  evaluationTimeoutMs: number;

  /** Require evidence for all violations */
  requireEvidence: boolean;

  /** Enable waiver system */
  enableWaivers: boolean;
}

/**
 * Arbitration protocol configuration
 */
export interface ArbitrationProtocolConfig {
  /** Rule engine configuration */
  ruleEngine: RuleEngineConfig;

  /** Performance budgets */
  performance: {
    constitutionalDecisionP95Ms: number;
    verdictGenerationP95Ms: number;
    waiverEvaluationP95Ms: number;
    appealProcessingP95Ms: number;
    debateCoordinationP95Ms: number;
  };

  /** Concurrency settings */
  concurrency: {
    maxConcurrentSessions: number;
    sessionQueueSize: number;
  };

  /** Audit settings */
  audit: {
    logAllDecisions: boolean;
    retentionDays: number;
  };
}

/**
 * Custom error for arbitration failures
 */
export class ArbitrationError extends Error {
  constructor(
    message: string,
    public code: string,
    public sessionId?: string,
    public ruleId?: string
  ) {
    super(message);
    this.name = "ArbitrationError";
  }
}
