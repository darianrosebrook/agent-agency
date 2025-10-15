/**
 * Arbiter Reasoning Engine
 *
 * Main orchestrator for multi-agent debate coordination and conflict resolution.
 * Coordinates debate state machine, argument structuring, evidence aggregation,
 * and consensus formation to resolve conflicts through structured argumentation.
 *
 * @author @darianrosebrook
 * @module reasoning/ArbiterReasoningEngine
 */
// @ts-nocheck


import {
  AgentRole,
  Argument,
  ConsensusAlgorithm,
  ConsensusResult,
  DeadlockDetection,
  DeadlockResolutionStrategy,
  DebateConfig,
  DebateParticipant,
  DebateSession,
  DebateState,
  DebateTimeoutError,
  DebateVote,
  Evidence,
  ReasoningEngineError,
} from "@/types/reasoning";
import { ArgumentStructure } from "./ArgumentStructure";
import { ConsensusEngine } from "./ConsensusEngine";
import { DebateStateMachine } from "./DebateStateMachine";
import { EvidenceAggregator } from "./EvidenceAggregator";

/**
 * Configuration options for the reasoning engine
 */
export interface ReasoningEngineConfig {
  maxDebateDuration: number; // milliseconds
  defaultConsensusAlgorithm: ConsensusAlgorithm;
  minimumParticipants: number;
  maximumParticipants: number;
  enableDeadlockDetection: boolean;
  deadlockDetectionRounds: number;
}

/**
 * Main Arbiter Reasoning Engine
 */
export class ArbiterReasoningEngine {
  private static readonly DEFAULT_CONFIG: ReasoningEngineConfig = {
    maxDebateDuration: 300000, // 5 minutes
    defaultConsensusAlgorithm: ConsensusAlgorithm.WEIGHTED_MAJORITY,
    minimumParticipants: 2,
    maximumParticipants: 10,
    enableDeadlockDetection: true,
    deadlockDetectionRounds: 3,
  };

  private readonly config: ReasoningEngineConfig;
  private readonly activeSessions: Map<string, DebateSession>;

  constructor(config: Partial<ReasoningEngineConfig> = {}) {
    this.config = { ...ArbiterReasoningEngine.DEFAULT_CONFIG, ...config };
    this.activeSessions = new Map();
  }

  /**
   * Initiates a new debate session
   */
  public async initiateDebate(
    topic: string,
    participants: Array<{ agentId: string; role: AgentRole; weight?: number }>
  ): Promise<DebateSession> {
    // Validate topic
    if (!topic || topic.trim().length === 0) {
      throw new ReasoningEngineError(
        "Debate topic cannot be empty",
        "EMPTY_TOPIC"
      );
    }

    // Validate participant count
    if (participants.length < this.config.minimumParticipants) {
      throw new ReasoningEngineError(
        `Insufficient participants: ${participants.length} (minimum: ${this.config.minimumParticipants})`,
        "INSUFFICIENT_PARTICIPANTS"
      );
    }

    if (participants.length > this.config.maximumParticipants) {
      throw new ReasoningEngineError(
        `Too many participants: ${participants.length} (maximum: ${this.config.maximumParticipants})`,
        "TOO_MANY_PARTICIPANTS"
      );
    }

    // Validate no duplicate participant IDs
    const participantIds = new Set<string>();
    for (const participant of participants) {
      if (participantIds.has(participant.agentId)) {
        throw new ReasoningEngineError(
          `Duplicate participant ID: ${participant.agentId}`,
          "DUPLICATE_PARTICIPANT"
        );
      }
      participantIds.add(participant.agentId);
    }

    // Create debate configuration
    const debateConfig: DebateConfig = {
      id: this.generateDebateId(),
      topic,
      maxParticipants: this.config.maximumParticipants,
      maxDuration: this.config.maxDebateDuration,
      consensusAlgorithm: this.config.defaultConsensusAlgorithm,
      deadlockStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
      requiresUnanimous: false,
      allowAppeals: true,
    };

    // Initialize session
    const session = DebateStateMachine.initializeSession(
      debateConfig.id,
      debateConfig
    );

    // Add participants
    const debateParticipants: DebateParticipant[] = participants.map((p) => ({
      agentId: p.agentId,
      role: p.role,
      weight: p.weight ?? 1,
      argumentsPresented: [],
      votesCast: [],
    }));

    const updatedSession: DebateSession = {
      ...session,
      participants: debateParticipants,
    };

    // Transition to agents assigned
    const finalSession = DebateStateMachine.transition(
      updatedSession,
      DebateState.AGENTS_ASSIGNED
    );

    // Store active session
    this.activeSessions.set(finalSession.id, finalSession);

    return finalSession;
  }

  /**
   * Submits an argument to the debate
   */
  public async submitArgument(
    debateId: string,
    agentId: string,
    claim: string,
    evidence: Evidence[],
    reasoning: string
  ): Promise<DebateSession> {
    const session = this.getActiveSession(debateId);

    // Validate agent is participant
    const participant = session.participants.find((p) => p.agentId === agentId);
    if (!participant) {
      throw new ReasoningEngineError(
        `Agent ${agentId} is not a participant in debate ${debateId}`,
        "AGENT_NOT_PARTICIPANT",
        debateId
      );
    }

    // Check if debate timed out
    if (DebateStateMachine.isExpired(session)) {
      throw new DebateTimeoutError(debateId);
    }

    // Create and validate argument
    const argument = ArgumentStructure.createArgument(
      agentId,
      claim,
      evidence,
      reasoning
    );

    const validation = ArgumentStructure.validateArgument(argument);
    if (!validation.valid) {
      throw new ReasoningEngineError(
        `Invalid argument: ${validation.errors.join(", ")}`,
        "INVALID_ARGUMENT",
        debateId
      );
    }

    // Add argument to session
    const updatedSession: DebateSession = {
      ...session,
      arguments: [...session.arguments, argument],
      participants: session.participants.map((p) =>
        p.agentId === agentId
          ? {
              ...p,
              argumentsPresented: [...p.argumentsPresented, argument.id],
            }
          : p
      ),
    };

    // Transition state if needed
    let finalSession = updatedSession;
    if (session.state === DebateState.AGENTS_ASSIGNED) {
      finalSession = DebateStateMachine.transition(
        updatedSession,
        DebateState.ARGUMENTS_PRESENTED
      );
    }

    // Update active session
    this.activeSessions.set(debateId, finalSession);

    return finalSession;
  }

  /**
   * Aggregates all evidence and transitions to deliberation
   */
  public async aggregateEvidence(debateId: string): Promise<DebateSession> {
    const session = this.getActiveSession(debateId);

    // Validate state
    if (session.state !== DebateState.ARGUMENTS_PRESENTED) {
      throw new ReasoningEngineError(
        `Cannot aggregate evidence in state ${session.state}`,
        "INVALID_STATE",
        debateId
      );
    }

    // Aggregate evidence across all arguments
    const aggregation = EvidenceAggregator.aggregateEvidence(session.arguments);

    // Validate evidence quality
    const allEvidence = session.arguments.flatMap((arg) => arg.evidence);
    const qualityCheck =
      EvidenceAggregator.validateEvidenceQuality(allEvidence);

    // Add evidence aggregation to reasoning chain
    const updatedSession: DebateSession = {
      ...session,
      reasoningChain: [
        ...session.reasoningChain,
        `Evidence aggregated: ${aggregation.summary}`,
        qualityCheck.valid
          ? "Evidence quality validated"
          : `Evidence quality issues: ${qualityCheck.issues.join(", ")}`,
      ],
    };

    // Transition to evidence aggregated state
    const evidenceSession = DebateStateMachine.transition(
      updatedSession,
      DebateState.EVIDENCE_AGGREGATED
    );

    // Immediately transition to deliberation
    const finalSession = DebateStateMachine.transition(
      evidenceSession,
      DebateState.DELIBERATION
    );

    this.activeSessions.set(debateId, finalSession);

    return finalSession;
  }

  /**
   * Collects votes from participants
   */
  public async submitVote(
    debateId: string,
    agentId: string,
    position: "for" | "against" | "abstain",
    confidence: number,
    reasoning: string
  ): Promise<DebateSession> {
    const session = this.getActiveSession(debateId);

    // Validate state
    if (
      session.state !== DebateState.DELIBERATION &&
      session.state !== DebateState.CONSENSUS_FORMING
    ) {
      throw new ReasoningEngineError(
        `Cannot submit vote in state ${session.state}`,
        "INVALID_STATE",
        debateId
      );
    }

    // Validate agent is participant
    const participant = session.participants.find((p) => p.agentId === agentId);
    if (!participant) {
      throw new ReasoningEngineError(
        `Agent ${agentId} is not a participant`,
        "AGENT_NOT_PARTICIPANT",
        debateId
      );
    }

    // Validate confidence range
    if (confidence < 0 || confidence > 1) {
      throw new ReasoningEngineError(
        `Invalid confidence value: ${confidence} (must be between 0 and 1)`,
        "INVALID_CONFIDENCE",
        debateId
      );
    }

    // Create vote
    const vote: DebateVote = {
      agentId,
      position,
      confidence,
      reasoning,
      timestamp: new Date(),
    };

    // Add vote to participant
    const updatedSession: DebateSession = {
      ...session,
      participants: session.participants.map((p) =>
        p.agentId === agentId ? { ...p, votesCast: [...p.votesCast, vote] } : p
      ),
    };

    // Transition to consensus forming if needed
    let finalSession = updatedSession;
    if (session.state === DebateState.DELIBERATION) {
      finalSession = DebateStateMachine.transition(
        updatedSession,
        DebateState.CONSENSUS_FORMING
      );
    }

    this.activeSessions.set(debateId, finalSession);

    return finalSession;
  }

  /**
   * Attempts to form consensus from collected votes
   */
  public async formConsensus(debateId: string): Promise<DebateSession> {
    const session = this.getActiveSession(debateId);

    // Validate state (allow deliberation or consensus_forming)
    if (
      session.state !== DebateState.DELIBERATION &&
      session.state !== DebateState.CONSENSUS_FORMING
    ) {
      throw new ReasoningEngineError(
        `Cannot form consensus in state ${session.state}`,
        "INVALID_STATE",
        debateId
      );
    }

    // Collect all votes
    const allVotes = session.participants.flatMap((p) => p.votesCast);

    // Attempt consensus formation
    const consensusResult = ConsensusEngine.formConsensus(
      allVotes,
      session.participants,
      {
        algorithm: session.config.consensusAlgorithm,
        minimumParticipation: 0.67,
        confidenceThreshold: 0.6,
      }
    );

    // Update session with consensus result
    const updatedSession: DebateSession = {
      ...session,
      consensusResult,
      reasoningChain: [...session.reasoningChain, consensusResult.reasoning],
    };

    // Transition based on consensus result
    let finalSession: DebateSession;
    if (consensusResult.reached) {
      const consensusReachedSession = DebateStateMachine.transition(
        updatedSession,
        DebateState.CONSENSUS_REACHED
      );
      finalSession = DebateStateMachine.transition(
        consensusReachedSession,
        DebateState.COMPLETED
      );
    } else {
      // Check for deadlock
      const deadlock = this.detectDeadlock(updatedSession);
      if (deadlock.isDeadlocked) {
        finalSession = DebateStateMachine.transition(
          updatedSession,
          DebateState.DEADLOCKED
        );
      } else {
        // Continue consensus forming
        finalSession = updatedSession;
      }
    }

    this.activeSessions.set(debateId, finalSession);

    return finalSession;
  }

  /**
   * Gets complete debate results
   */
  public async getDebateResults(debateId: string): Promise<{
    session: DebateSession;
    consensus: ConsensusResult | undefined;
    evidenceSummary: string;
    topArguments: Argument[];
  }> {
    const session = this.getActiveSession(debateId);

    // Get evidence summary
    const aggregation = EvidenceAggregator.aggregateEvidence(session.arguments);

    // Get top arguments by credibility
    const topArguments = session.arguments
      .sort(ArgumentStructure.compareArguments)
      .slice(0, 5);

    return {
      session,
      consensus: session.consensusResult,
      evidenceSummary: aggregation.summary,
      topArguments,
    };
  }

  /**
   * Closes a debate session
   */
  public async closeDebate(debateId: string): Promise<void> {
    const session = this.getActiveSession(debateId);

    // Ensure debate is in terminal state
    if (!DebateStateMachine.isTerminalState(session.state)) {
      throw new ReasoningEngineError(
        `Cannot close debate in non-terminal state ${session.state}`,
        "INVALID_STATE",
        debateId
      );
    }

    // Remove from active sessions
    this.activeSessions.delete(debateId);
  }

  /**
   * Detects if debate is deadlocked
   */
  private detectDeadlock(session: DebateSession): DeadlockDetection {
    if (!this.config.enableDeadlockDetection) {
      return {
        isDeadlocked: false,
        rounds: 0,
        votingPattern: "none",
        recommendedStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
        confidence: 0,
      };
    }

    // Simple heuristic: if voting pattern hasn't changed for N rounds, it's deadlocked
    const allVotes = session.participants.flatMap((p) => p.votesCast);

    // Check if consensus is mathematically impossible
    const canReach = ConsensusEngine.canReachConsensus(
      allVotes,
      session.participants.length,
      session.config.consensusAlgorithm
    );

    if (!canReach) {
      return {
        isDeadlocked: true,
        rounds: this.config.deadlockDetectionRounds,
        votingPattern: "impossible",
        recommendedStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
        confidence: 1.0,
      };
    }

    // Not deadlocked yet
    return {
      isDeadlocked: false,
      rounds: 0,
      votingPattern: "progressing",
      recommendedStrategy: DeadlockResolutionStrategy.TIMEOUT_DEFAULT,
      confidence: 0.5,
    };
  }

  /**
   * Gets active session or throws error
   */
  private getActiveSession(debateId: string): DebateSession {
    const session = this.activeSessions.get(debateId);
    if (!session) {
      throw new ReasoningEngineError(
        `Debate ${debateId} not found`,
        "DEBATE_NOT_FOUND",
        debateId
      );
    }
    return session;
  }

  /**
   * Generates unique debate ID
   */
  private generateDebateId(): string {
    return `debate-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Gets count of active debates
   */
  public getActiveDebateCount(): number {
    return this.activeSessions.size;
  }

  /**
   * Lists all active debate IDs
   */
  public getActiveDebateIds(): string[] {
    return Array.from(this.activeSessions.keys());
  }
}
