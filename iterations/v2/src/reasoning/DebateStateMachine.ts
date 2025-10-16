/**
 * Debate State Machine
 *
 * Manages debate state transitions with clear invariants and validation.
 * Ensures debates progress through well-defined states with proper guards.
 *
 * @author @darianrosebrook
 * @module reasoning/DebateStateMachine
 */

import {
  DebateSession,
  DebateState,
  ReasoningEngineError,
} from "@/types/reasoning";

/**
 * State transition definition
 */
interface StateTransition {
  from: DebateState;
  to: DebateState;
  guard?: (_session: DebateSession) => boolean;
  action?: (_session: DebateSession) => void;
}

/**
 * Manages debate state transitions with validation
 */
export class DebateStateMachine {
  private static readonly TRANSITIONS: StateTransition[] = [
    // Initialization flow
    { from: DebateState.INITIALIZED, to: DebateState.AGENTS_ASSIGNED },
    { from: DebateState.AGENTS_ASSIGNED, to: DebateState.ARGUMENTS_PRESENTED },

    // Normal debate flow
    {
      from: DebateState.ARGUMENTS_PRESENTED,
      to: DebateState.EVIDENCE_AGGREGATED,
    },
    { from: DebateState.EVIDENCE_AGGREGATED, to: DebateState.DELIBERATION },
    { from: DebateState.DELIBERATION, to: DebateState.CONSENSUS_FORMING },
    { from: DebateState.CONSENSUS_FORMING, to: DebateState.CONSENSUS_REACHED },
    { from: DebateState.CONSENSUS_REACHED, to: DebateState.COMPLETED },

    // Deadlock flow
    { from: DebateState.CONSENSUS_FORMING, to: DebateState.DEADLOCKED },
    { from: DebateState.DEADLOCKED, to: DebateState.RESOLUTION_IN_PROGRESS },
    {
      from: DebateState.RESOLUTION_IN_PROGRESS,
      to: DebateState.CONSENSUS_FORMING,
    },
    { from: DebateState.RESOLUTION_IN_PROGRESS, to: DebateState.COMPLETED },

    // Failure paths
    { from: DebateState.INITIALIZED, to: DebateState.FAILED },
    { from: DebateState.AGENTS_ASSIGNED, to: DebateState.FAILED },
    { from: DebateState.ARGUMENTS_PRESENTED, to: DebateState.FAILED },
    { from: DebateState.EVIDENCE_AGGREGATED, to: DebateState.FAILED },
    { from: DebateState.DELIBERATION, to: DebateState.FAILED },
    { from: DebateState.DEADLOCKED, to: DebateState.FAILED },
  ];

  /**
   * Validates if a state transition is allowed
   */
  public static canTransition(
    session: DebateSession,
    toState: DebateState
  ): boolean {
    const transition = this.TRANSITIONS.find(
      (t) => t.from === session.state && t.to === toState
    );

    if (!transition) {
      return false;
    }

    // Check guard condition if present
    if (transition.guard && !transition.guard(session)) {
      return false;
    }

    return true;
  }

  /**
   * Transitions debate to new state with validation
   */
  public static transition(
    session: DebateSession,
    toState: DebateState
  ): DebateSession {
    if (!this.canTransition(session, toState)) {
      throw new ReasoningEngineError(
        `Invalid state transition from ${session.state} to ${toState}`,
        "INVALID_STATE_TRANSITION",
        session.id
      );
    }

    const transition = this.TRANSITIONS.find(
      (t) => t.from === session.state && t.to === toState
    );

    // Execute transition action if present
    if (transition?.action) {
      transition.action(session);
    }

    // Update state
    const updatedSession: DebateSession = {
      ...session,
      state: toState,
      reasoningChain: [
        ...session.reasoningChain,
        `Transitioned from ${session.state} to ${toState}`,
      ],
    };

    // Set end time if reaching terminal state
    if (toState === DebateState.COMPLETED || toState === DebateState.FAILED) {
      updatedSession.endTime = new Date();
    }

    return updatedSession;
  }

  /**
   * Checks if state is terminal (debate cannot progress further)
   */
  public static isTerminalState(state: DebateState): boolean {
    return state === DebateState.COMPLETED || state === DebateState.FAILED;
  }

  /**
   * Gets all valid next states from current state
   */
  public static getValidNextStates(currentState: DebateState): DebateState[] {
    return this.TRANSITIONS.filter((t) => t.from === currentState).map(
      (t) => t.to
    );
  }

  /**
   * Validates debate session invariants
   */
  public static validateInvariants(session: DebateSession): void {
    // Invariant: Must have at least 2 participants
    if (session.participants.length < 2) {
      throw new ReasoningEngineError(
        "Debate must have at least 2 participants",
        "INSUFFICIENT_PARTICIPANTS",
        session.id
      );
    }

    // Invariant: Cannot have consensus without being in consensus state
    if (
      session.consensusResult &&
      session.state !== DebateState.CONSENSUS_REACHED &&
      session.state !== DebateState.COMPLETED
    ) {
      throw new ReasoningEngineError(
        "Consensus result exists but not in consensus state",
        "INVALID_CONSENSUS_STATE",
        session.id
      );
    }

    // Invariant: End time only set for terminal states
    if (session.endTime && !this.isTerminalState(session.state)) {
      throw new ReasoningEngineError(
        "End time set but state is not terminal",
        "INVALID_END_TIME",
        session.id
      );
    }

    // Invariant: Completed debates must have consensus
    if (session.state === DebateState.COMPLETED && !session.consensusResult) {
      throw new ReasoningEngineError(
        "Completed debate missing consensus result",
        "MISSING_CONSENSUS",
        session.id
      );
    }
  }

  /**
   * Initializes a new debate session
   */
  public static initializeSession(
    id: string,
    config: DebateSession["config"]
  ): DebateSession {
    return {
      id,
      config,
      state: DebateState.INITIALIZED,
      participants: [],
      arguments: [],
      startTime: new Date(),
      reasoningChain: ["Debate session initialized"],
    };
  }

  /**
   * Checks if debate has exceeded time limit
   */
  public static isExpired(session: DebateSession): boolean {
    const elapsed = Date.now() - session.startTime.getTime();
    return elapsed > session.config.maxDuration;
  }

  /**
   * Gets current state summary
   */
  public static getStateSummary(session: DebateSession): string {
    const elapsed = Math.floor(
      (Date.now() - session.startTime.getTime()) / 1000
    );
    return (
      `Debate ${session.id} in state ${session.state} ` +
      `with ${session.participants.length} participants, ` +
      `${session.arguments.length} arguments, ` +
      `elapsed ${elapsed}s`
    );
  }
}
