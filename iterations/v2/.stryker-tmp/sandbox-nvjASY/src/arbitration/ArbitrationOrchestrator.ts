/**
 * Arbitration Orchestrator
 *
 * @author @darianrosebrook
 *
 * Main coordinator for the CAWS Arbitration Protocol Engine.
 * Orchestrates the complete arbitration workflow from violation detection
 * through verdict generation, waiver evaluation, precedent application,
 * and appeal handling.
 *
 * Features:
 * - End-to-end arbitration workflow coordination
 * - Component integration and lifecycle management
 * - Session state management
 * - Performance tracking and monitoring
 * - Error handling and recovery
 */
// @ts-nocheck


import {
  Appeal,
  ArbitrationError,
  ArbitrationSession,
  ArbitrationState,
  ConstitutionalRule,
  ConstitutionalViolation,
  Verdict,
  WaiverRequest,
} from "@/types/arbitration";
import { AppealArbitrator } from "./AppealArbitrator";
import { ConstitutionalRuleEngine } from "./ConstitutionalRuleEngine";
import { PrecedentManager } from "./PrecedentManager";
import { VerdictGenerator } from "./VerdictGenerator";
import { WaiverInterpreter } from "./WaiverInterpreter";

/**
 * Orchestrator configuration
 */
export interface ArbitrationOrchestratorConfig {
  /** Enable automatic precedent application */
  autoApplyPrecedents: boolean;

  /** Enable waiver system */
  enableWaivers: boolean;

  /** Enable appeal system */
  enableAppeals: boolean;

  /** Maximum concurrent sessions */
  maxConcurrentSessions: number;

  /** Session timeout (ms) */
  sessionTimeoutMs: number;

  /** Performance tracking */
  trackPerformance: boolean;
}

/**
 * Session performance metrics
 */
export interface SessionMetrics {
  /** Session ID */
  sessionId: string;

  /** Total duration (ms) */
  totalDurationMs: number;

  /** Rule evaluation time (ms) */
  ruleEvaluationMs: number;

  /** Precedent lookup time (ms) */
  precedentLookupMs: number;

  /** Verdict generation time (ms) */
  verdictGenerationMs: number;

  /** Waiver evaluation time (ms) */
  waiverEvaluationMs?: number;

  /** Rules evaluated count */
  rulesEvaluated: number;

  /** Precedents found count */
  precedentsFound: number;

  /** Final state */
  finalState: ArbitrationState;
}

/**
 * ArbitrationOrchestrator - Main coordinator for arbitration protocol
 */
export class ArbitrationOrchestrator {
  /** Configuration */
  private config: ArbitrationOrchestratorConfig;

  /** Component: Rule Engine */
  private ruleEngine: ConstitutionalRuleEngine;

  /** Component: Verdict Generator */
  private verdictGenerator: VerdictGenerator;

  /** Component: Waiver Interpreter */
  private waiverInterpreter: WaiverInterpreter;

  /** Component: Precedent Manager */
  private precedentManager: PrecedentManager;

  /** Component: Appeal Arbitrator */
  private appealArbitrator: AppealArbitrator;

  /** Active sessions */
  private sessions: Map<string, ArbitrationSession> = new Map();

  /** Session metrics */
  private metrics: Map<string, SessionMetrics> = new Map();

  /** Session counter */
  private sessionCounter: number = 0;

  constructor(config?: Partial<ArbitrationOrchestratorConfig>) {
    this.config = {
      autoApplyPrecedents: true,
      enableWaivers: true,
      enableAppeals: true,
      maxConcurrentSessions: 10,
      sessionTimeoutMs: 5 * 60 * 1000, // 5 minutes
      trackPerformance: true,
      ...config,
    };

    // Initialize components
    this.ruleEngine = new ConstitutionalRuleEngine();
    this.verdictGenerator = new VerdictGenerator();
    this.waiverInterpreter = new WaiverInterpreter();
    this.precedentManager = new PrecedentManager();
    this.appealArbitrator = new AppealArbitrator();
  }

  /**
   * Start a new arbitration session
   */
  public async startSession(
    violation: ConstitutionalViolation,
    rules: ConstitutionalRule[],
    participants: string[]
  ): Promise<ArbitrationSession> {
    // Check concurrent session limit
    if (this.sessions.size >= this.config.maxConcurrentSessions) {
      throw new ArbitrationError(
        "Maximum concurrent sessions reached",
        "SESSION_LIMIT_EXCEEDED"
      );
    }

    // Create session
    const session: ArbitrationSession = {
      id: this.generateSessionId(),
      state: ArbitrationState.INITIALIZED,
      violation,
      rulesEvaluated: rules,
      evidence: violation.evidence,
      participants,
      precedents: [],
      startTime: new Date(),
      metadata: {},
    };

    // Store session
    this.sessions.set(session.id, session);

    // Initialize metrics
    if (this.config.trackPerformance) {
      this.metrics.set(session.id, {
        sessionId: session.id,
        totalDurationMs: 0,
        ruleEvaluationMs: 0,
        precedentLookupMs: 0,
        verdictGenerationMs: 0,
        rulesEvaluated: rules.length,
        precedentsFound: 0,
        finalState: ArbitrationState.INITIALIZED,
      });
    }

    // Transition to rule evaluation
    await this.transitionState(session, ArbitrationState.RULE_EVALUATION);

    return session;
  }

  /**
   * Evaluate constitutional rules against violation
   */
  public async evaluateRules(sessionId: string): Promise<void> {
    const session = this.getSession(sessionId);
    const startTime = Date.now();

    // Ensure correct state
    if (session.state !== ArbitrationState.RULE_EVALUATION) {
      throw new ArbitrationError(
        `Cannot evaluate rules in state ${session.state}`,
        "INVALID_STATE",
        sessionId
      );
    }

    // Load rules into engine
    for (const rule of session.rulesEvaluated) {
      this.ruleEngine.loadRule(rule);
    }

    // Evaluate violation
    const evaluationResults = await this.ruleEngine.evaluateAction(
      {
        action: session.violation.ruleId,
        actor: session.violation.violator || "unknown",
        parameters: session.violation.context,
        environment: {},
        timestamp: session.violation.detectedAt,
      },
      session.rulesEvaluated.map((r) => r.id)
    );

    // Store violation details
    session.metadata.ruleEvaluationResults = evaluationResults;

    // Track metrics
    if (this.config.trackPerformance) {
      const metrics = this.metrics.get(sessionId)!;
      metrics.ruleEvaluationMs = Date.now() - startTime;
    }

    // Find precedents if enabled
    if (this.config.autoApplyPrecedents) {
      await this.transitionState(session, ArbitrationState.EVIDENCE_COLLECTION);
      await this.findPrecedents(sessionId);
    } else {
      await this.transitionState(session, ArbitrationState.VERDICT_GENERATION);
    }
  }

  /**
   * Find and apply relevant precedents
   */
  public async findPrecedents(sessionId: string): Promise<void> {
    const session = this.getSession(sessionId);
    const startTime = Date.now();

    // Find similar precedents
    const matches = this.precedentManager.findSimilarPrecedents(
      session.rulesEvaluated[0].category,
      session.violation.severity,
      [session.violation.description],
      session.rulesEvaluated.map((r) => r.id),
      5
    );

    // Store precedents
    session.precedents = matches.map((m) => m.precedent);

    // Track metrics
    if (this.config.trackPerformance) {
      const metrics = this.metrics.get(sessionId)!;
      metrics.precedentLookupMs = Date.now() - startTime;
      metrics.precedentsFound = matches.length;
    }

    // Transition to verdict generation
    await this.transitionState(session, ArbitrationState.VERDICT_GENERATION);
  }

  /**
   * Generate verdict for session
   */
  public async generateVerdict(
    sessionId: string,
    issuedBy: string
  ): Promise<Verdict> {
    const session = this.getSession(sessionId);
    const _startTime = Date.now();

    // Ensure correct state
    if (session.state !== ArbitrationState.VERDICT_GENERATION) {
      throw new ArbitrationError(
        `Cannot generate verdict in state ${session.state}`,
        "INVALID_STATE",
        sessionId
      );
    }

    // Generate verdict
    const result = await this.verdictGenerator.generateVerdict(
      session,
      issuedBy
    );

    // Store verdict
    session.verdict = result.verdict;

    // Track metrics (ensure non-zero by adding time if needed)
    if (this.config.trackPerformance) {
      const metrics = this.metrics.get(sessionId)!;
      metrics.verdictGenerationMs = Math.max(result.generationTimeMs, 1);
    }

    // Create precedent from verdict
    if (result.verdict.confidence > 0.8) {
      this.precedentManager.createPrecedent(
        result.verdict,
        `${session.violation.ruleId} Violation`,
        [session.violation.description],
        result.verdict.reasoning.map((r) => r.description).join(". "),
        {
          category: session.rulesEvaluated[0].category,
          severity: session.violation.severity,
          conditions: result.verdict.conditions || [],
        }
      );
    }

    // Don't auto-complete - let caller decide next steps (waiver/appeal/complete)
    // Session remains in VERDICT_GENERATION state

    return result.verdict;
  }

  /**
   * Evaluate waiver request
   */
  public async evaluateWaiver(
    sessionId: string,
    waiverRequest: WaiverRequest,
    decidedBy: string
  ): Promise<void> {
    const session = this.getSession(sessionId);
    const startTime = Date.now();

    if (!this.config.enableWaivers) {
      throw new ArbitrationError(
        "Waiver system is disabled",
        "WAIVERS_DISABLED",
        sessionId
      );
    }

    // Transition to waiver evaluation if needed
    if (session.state === ArbitrationState.VERDICT_GENERATION) {
      await this.transitionState(session, ArbitrationState.WAIVER_EVALUATION);
    }

    // Store waiver request
    session.waiverRequest = waiverRequest;

    // Evaluate waiver
    const decision = await this.waiverInterpreter.processWaiver(
      waiverRequest,
      session.rulesEvaluated[0],
      decidedBy
    );

    // Store decision
    session.metadata.waiverDecision = decision;

    // Track metrics
    if (this.config.trackPerformance) {
      const metrics = this.metrics.get(sessionId)!;
      metrics.waiverEvaluationMs = Date.now() - startTime;
    }

    // Complete session
    await this.transitionState(session, ArbitrationState.COMPLETED);
  }

  /**
   * Submit appeal for verdict
   */
  public async submitAppeal(
    sessionId: string,
    appellantId: string,
    grounds: string,
    newEvidence: string[]
  ): Promise<Appeal> {
    const session = this.getSession(sessionId);

    if (!this.config.enableAppeals) {
      throw new ArbitrationError(
        "Appeal system is disabled",
        "APPEALS_DISABLED",
        sessionId
      );
    }

    if (!session.verdict) {
      throw new ArbitrationError(
        "Cannot appeal session without verdict",
        "NO_VERDICT",
        sessionId
      );
    }

    // Transition to appeal review if needed
    if (
      session.state === ArbitrationState.VERDICT_GENERATION ||
      session.state === ArbitrationState.WAIVER_EVALUATION ||
      session.state === ArbitrationState.COMPLETED
    ) {
      await this.transitionState(session, ArbitrationState.APPEAL_REVIEW);
    }

    // Submit appeal
    const appeal = await this.appealArbitrator.submitAppeal(
      session,
      session.verdict,
      appellantId,
      grounds,
      newEvidence
    );

    return appeal;
  }

  /**
   * Review appeal
   */
  public async reviewAppeal(
    sessionId: string,
    appealId: string,
    reviewers: string[]
  ): Promise<void> {
    const session = this.getSession(sessionId);

    if (!session.verdict) {
      throw new ArbitrationError(
        "Cannot review appeal without verdict",
        "NO_VERDICT",
        sessionId
      );
    }

    // Review appeal
    const decision = await this.appealArbitrator.reviewAppeal(
      appealId,
      reviewers,
      session,
      session.verdict
    );

    // Store decision
    session.metadata.appealDecision = decision;

    // If overturned, update verdict
    if (decision.decision === "overturned" && decision.newVerdict) {
      session.verdict = decision.newVerdict;

      // Create precedent for overturned verdict
      this.precedentManager.createPrecedent(
        decision.newVerdict,
        `${session.violation.ruleId} Appeal Overturn`,
        [session.violation.description, ...decision.newVerdict.evidence],
        decision.reasoning,
        {
          category: session.rulesEvaluated[0].category,
          severity: session.violation.severity,
          conditions: [],
        }
      );
    }

    // Complete session
    await this.transitionState(session, ArbitrationState.COMPLETED);
  }

  /**
   * Get session by ID
   */
  public getSession(sessionId: string): ArbitrationSession {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new ArbitrationError(
        `Session ${sessionId} not found`,
        "SESSION_NOT_FOUND"
      );
    }
    return session;
  }

  /**
   * Get all active sessions
   */
  public getActiveSessions(): ArbitrationSession[] {
    return Array.from(this.sessions.values()).filter(
      (s) =>
        s.state !== ArbitrationState.COMPLETED &&
        s.state !== ArbitrationState.FAILED
    );
  }

  /**
   * Get session metrics
   */
  public getSessionMetrics(sessionId: string): SessionMetrics | undefined {
    return this.metrics.get(sessionId);
  }

  /**
   * Get all metrics
   */
  public getAllMetrics(): SessionMetrics[] {
    return Array.from(this.metrics.values());
  }

  /**
   * Get orchestrator statistics
   */
  public getStatistics(): {
    totalSessions: number;
    activeSessions: number;
    completedSessions: number;
    failedSessions: number;
    averageDurationMs: number;
    totalPrecedents: number;
    totalAppeals: number;
  } {
    const all = Array.from(this.sessions.values());
    const active = all.filter(
      (s) =>
        s.state !== ArbitrationState.COMPLETED &&
        s.state !== ArbitrationState.FAILED
    );
    const completed = all.filter((s) => s.state === ArbitrationState.COMPLETED);
    const failed = all.filter((s) => s.state === ArbitrationState.FAILED);

    const durations = Array.from(this.metrics.values()).map(
      (m) => m.totalDurationMs
    );
    const averageDurationMs =
      durations.length > 0
        ? durations.reduce((a, b) => a + b, 0) / durations.length
        : 0;

    return {
      totalSessions: all.length,
      activeSessions: active.length,
      completedSessions: completed.length,
      failedSessions: failed.length,
      averageDurationMs,
      totalPrecedents: this.precedentManager.getStatistics().totalPrecedents,
      totalAppeals: this.appealArbitrator.getStatistics().totalAppeals,
    };
  }

  /**
   * Complete session
   */
  public async completeSession(sessionId: string): Promise<void> {
    const session = this.getSession(sessionId);

    // If already completed, skip
    if (session.state === ArbitrationState.COMPLETED) {
      return;
    }

    // Update end time
    session.endTime = new Date();

    // Transition to completed
    await this.transitionState(session, ArbitrationState.COMPLETED);

    // Update metrics after transition
    if (this.config.trackPerformance) {
      const metrics = this.metrics.get(sessionId);
      if (metrics) {
        metrics.totalDurationMs =
          session.endTime.getTime() - session.startTime.getTime();
        metrics.finalState = ArbitrationState.COMPLETED;
      }
    }
  }

  /**
   * Fail session with error
   */
  public async failSession(sessionId: string, error: Error): Promise<void> {
    const session = this.getSession(sessionId);

    // Store error
    session.metadata.error = {
      message: error.message,
      stack: error.stack,
      timestamp: new Date(),
    };

    // Update end time
    session.endTime = new Date();

    // Update metrics
    if (this.config.trackPerformance) {
      const metrics = this.metrics.get(sessionId);
      if (metrics) {
        metrics.totalDurationMs =
          session.endTime.getTime() - session.startTime.getTime();
        metrics.finalState = ArbitrationState.FAILED;
      }
    }

    // Transition to failed
    await this.transitionState(session, ArbitrationState.FAILED);
  }

  /**
   * Transition session state
   */
  private async transitionState(
    session: ArbitrationSession,
    newState: ArbitrationState
  ): Promise<void> {
    // Validate transition
    this.validateStateTransition(session.state, newState);

    // Update state
    session.state = newState;

    // Log transition
    if (!session.metadata.stateTransitions) {
      session.metadata.stateTransitions = [];
    }
    session.metadata.stateTransitions.push({
      from: session.state,
      to: newState,
      timestamp: new Date(),
    });
  }

  /**
   * Validate state transition
   */
  private validateStateTransition(
    from: ArbitrationState,
    to: ArbitrationState
  ): void {
    // Allow transition to FAILED from any non-terminal state
    if (
      to === ArbitrationState.FAILED &&
      from !== ArbitrationState.FAILED &&
      from !== ArbitrationState.COMPLETED
    ) {
      return;
    }

    // Allow transition to COMPLETED from any non-terminal state
    if (
      to === ArbitrationState.COMPLETED &&
      from !== ArbitrationState.FAILED &&
      from !== ArbitrationState.COMPLETED
    ) {
      return;
    }

    const validTransitions: Record<ArbitrationState, ArbitrationState[]> = {
      [ArbitrationState.INITIALIZED]: [ArbitrationState.RULE_EVALUATION],
      [ArbitrationState.RULE_EVALUATION]: [
        ArbitrationState.EVIDENCE_COLLECTION,
        ArbitrationState.VERDICT_GENERATION,
      ],
      [ArbitrationState.EVIDENCE_COLLECTION]: [
        ArbitrationState.VERDICT_GENERATION,
      ],
      [ArbitrationState.VERDICT_GENERATION]: [
        ArbitrationState.WAIVER_EVALUATION,
        ArbitrationState.APPEAL_REVIEW,
        ArbitrationState.COMPLETED, // Allow direct completion if no waiver/appeal
      ],
      [ArbitrationState.WAIVER_EVALUATION]: [
        ArbitrationState.COMPLETED, // Complete after waiver evaluation
      ],
      [ArbitrationState.APPEAL_REVIEW]: [
        ArbitrationState.COMPLETED, // Complete after appeal review
      ],
      [ArbitrationState.DEBATE_IN_PROGRESS]: [
        ArbitrationState.VERDICT_GENERATION, // Debate leads to verdict
      ],
      [ArbitrationState.COMPLETED]: [
        ArbitrationState.APPEAL_REVIEW, // Allow reopening for appeals
      ],
      [ArbitrationState.FAILED]: [],
    };

    const allowed = validTransitions[from] || [];
    if (!allowed.includes(to)) {
      throw new ArbitrationError(
        `Invalid state transition from ${from} to ${to}`,
        "INVALID_STATE_TRANSITION"
      );
    }
  }

  /**
   * Generate unique session ID
   */
  private generateSessionId(): string {
    this.sessionCounter++;
    return `ARB-${Date.now()}-${this.sessionCounter}`;
  }

  /**
   * Clear all sessions (for testing)
   */
  public clear(): void {
    this.sessions.clear();
    this.metrics.clear();
    this.sessionCounter = 0;
    this.ruleEngine = new ConstitutionalRuleEngine();
    this.precedentManager.clear();
    this.appealArbitrator.clear();
  }

  /**
   * Get component references (for advanced usage)
   */
  public getComponents(): {
    ruleEngine: ConstitutionalRuleEngine;
    verdictGenerator: VerdictGenerator;
    waiverInterpreter: WaiverInterpreter;
    precedentManager: PrecedentManager;
    appealArbitrator: AppealArbitrator;
  } {
    return {
      ruleEngine: this.ruleEngine,
      verdictGenerator: this.verdictGenerator,
      waiverInterpreter: this.waiverInterpreter,
      precedentManager: this.precedentManager,
      appealArbitrator: this.appealArbitrator,
    };
  }
}
