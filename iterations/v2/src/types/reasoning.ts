/**
 * Arbiter Reasoning Engine - Type Definitions
 *
 * Core types for multi-agent debate coordination, evidence aggregation,
 * consensus formation, and conflict resolution.
 *
 * @author @darianrosebrook
 * @module types/reasoning
 */

/**
 * Debate state enumeration
 */
export enum DebateState {
  INITIALIZED = "initialized",
  AGENTS_ASSIGNED = "agents_assigned",
  ARGUMENTS_PRESENTED = "arguments_presented",
  EVIDENCE_AGGREGATED = "evidence_aggregated",
  DELIBERATION = "deliberation",
  CONSENSUS_FORMING = "consensus_forming",
  CONSENSUS_REACHED = "consensus_reached",
  DEADLOCKED = "deadlocked",
  RESOLUTION_IN_PROGRESS = "resolution_in_progress",
  COMPLETED = "completed",
  FAILED = "failed",
}

/**
 * Agent role in debate
 */
export enum AgentRole {
  PROPONENT = "proponent",
  OPPONENT = "opponent",
  MEDIATOR = "mediator",
  OBSERVER = "observer",
}

/**
 * Consensus algorithm types
 */
export enum ConsensusAlgorithm {
  SIMPLE_MAJORITY = "simple_majority",
  WEIGHTED_MAJORITY = "weighted_majority",
  UNANIMOUS = "unanimous",
  SUPERMAJORITY = "supermajority",
}

/**
 * Deadlock resolution strategy
 */
export enum DeadlockResolutionStrategy {
  MEDIATOR_DECISION = "mediator_decision",
  ESCALATION = "escalation",
  TIMEOUT_DEFAULT = "timeout_default",
  WEIGHTED_VOTING = "weighted_voting",
}

/**
 * Argument structure with claim, evidence, and reasoning
 */
export interface Argument {
  id: string;
  agentId: string;
  claim: string;
  evidence: Evidence[];
  reasoning: string;
  timestamp: Date;
  credibilityScore?: number;
}

/**
 * Evidence supporting an argument
 */
export interface Evidence {
  id: string;
  source: string;
  content: string;
  credibilityScore: number;
  verificationStatus: "verified" | "unverified" | "disputed";
  timestamp: Date;
}

/**
 * Agent participant in debate
 */
export interface DebateParticipant {
  agentId: string;
  role: AgentRole;
  weight: number;
  argumentsPresented: string[];
  votesCast: DebateVote[];
}

/**
 * Vote cast by agent in consensus formation
 */
export interface DebateVote {
  agentId: string;
  position: "for" | "against" | "abstain";
  confidence: number;
  reasoning: string;
  timestamp: Date;
}

/**
 * Consensus result
 */
export interface ConsensusResult {
  reached: boolean;
  algorithm: ConsensusAlgorithm;
  outcome: "accepted" | "rejected" | "modified";
  confidence: number;
  votingBreakdown: {
    for: number;
    against: number;
    abstain: number;
  };
  reasoning: string;
  timestamp: Date;
}

/**
 * Debate configuration
 */
export interface DebateConfig {
  id: string;
  topic: string;
  maxParticipants: number;
  maxDuration: number;
  consensusAlgorithm: ConsensusAlgorithm;
  deadlockStrategy: DeadlockResolutionStrategy;
  requiresUnanimous: boolean;
  allowAppeals: boolean;
}

/**
 * Debate session state
 */
export interface DebateSession {
  id: string;
  config: DebateConfig;
  state: DebateState;
  participants: DebateParticipant[];
  arguments: Argument[];
  consensusResult?: ConsensusResult;
  startTime: Date;
  endTime?: Date;
  reasoningChain: string[];
}

/**
 * Turn in debate sequence
 */
export interface DebateTurn {
  id: string;
  debateId: string;
  agentId: string;
  turnNumber: number;
  action: "present_argument" | "respond" | "vote" | "object";
  content: string;
  timestamp: Date;
  timeoutAt: Date;
}

/**
 * Deadlock detection result
 */
export interface DeadlockDetection {
  isDeadlocked: boolean;
  rounds: number;
  votingPattern: string;
  recommendedStrategy: DeadlockResolutionStrategy;
  confidence: number;
}

/**
 * Appeal request
 */
export interface AppealRequest {
  id: string;
  debateId: string;
  agentId: string;
  grounds: string;
  evidence: Evidence[];
  timestamp: Date;
}

/**
 * Appeal decision
 */
export interface AppealDecision {
  requestId: string;
  outcome: "granted" | "denied" | "modified";
  reasoning: string;
  revisedConsensus?: ConsensusResult;
  timestamp: Date;
}

/**
 * Evidence aggregation result
 */
export interface EvidenceAggregation {
  totalEvidence: number;
  averageCredibility: number;
  verifiedCount: number;
  disputedCount: number;
  sources: string[];
  summary: string;
}

/**
 * Debate metrics
 */
export interface DebateMetrics {
  debateId: string;
  duration: number;
  participantCount: number;
  argumentCount: number;
  evidenceCount: number;
  consensusTimeMs: number;
  deadlockOccurred: boolean;
  appealCount: number;
}

/**
 * Error types for reasoning engine
 */
export class ReasoningEngineError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly debateId?: string
  ) {
    super(message);
    this.name = "ReasoningEngineError";
  }
}

export class DebateTimeoutError extends ReasoningEngineError {
  constructor(debateId: string) {
    super(
      `Debate ${debateId} exceeded maximum duration`,
      "DEBATE_TIMEOUT",
      debateId
    );
    this.name = "DebateTimeoutError";
  }
}

export class ConsensusImpossibleError extends ReasoningEngineError {
  constructor(debateId: string, reason: string) {
    super(
      `Consensus impossible in debate ${debateId}: ${reason}`,
      "CONSENSUS_IMPOSSIBLE",
      debateId
    );
    this.name = "ConsensusImpossibleError";
  }
}

export class InvalidArgumentError extends ReasoningEngineError {
  constructor(message: string) {
    super(message, "INVALID_ARGUMENT");
    this.name = "InvalidArgumentError";
  }
}
