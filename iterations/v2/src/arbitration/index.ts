/**
 * CAWS Arbitration System - Main Entry Point
 *
 * @author @darianrosebrook
 *
 * Unified export point for the complete CAWS Arbitration System,
 * integrating constitutional rule enforcement (ARBITER-015) with
 * multi-agent debate coordination (ARBITER-016).
 *
 * Features:
 * - Constitutional rule enforcement and arbitration
 * - Multi-agent debate and consensus building
 * - Verdict generation with precedent support
 * - Waiver evaluation and appeal handling
 * - Complete audit trail and provenance
 */

// ============================================================================
// ARBITER-015: CAWS Arbitration Protocol
// ============================================================================

/**
 * Core Arbitration Components
 */
export { ArbitrationOrchestrator } from "./ArbitrationOrchestrator";
export type {
  ArbitrationOrchestratorConfig,
  SessionMetrics,
} from "./ArbitrationOrchestrator";

export { ConstitutionalRuleEngine } from "./ConstitutionalRuleEngine";
export type {
  EvaluationContext,
  RuleEvaluationResult,
} from "./ConstitutionalRuleEngine";

export { VerdictGenerator } from "./VerdictGenerator";
export type {
  VerdictGenerationResult,
  VerdictGeneratorConfig,
} from "./VerdictGenerator";

export { WaiverInterpreter } from "./WaiverInterpreter";
export type {
  WaiverEvaluationResult,
  WaiverInterpreterConfig,
} from "./WaiverInterpreter";

export { PrecedentManager } from "./PrecedentManager";
export type {
  PrecedentManagerConfig,
  PrecedentSearchCriteria,
  SimilarityMatch,
} from "./PrecedentManager";

export { AppealArbitrator } from "./AppealArbitrator";
export type {
  AppealArbitratorConfig,
  AppealDecision,
} from "./AppealArbitrator";

// ============================================================================
// ARBITER-016: Arbiter Reasoning Engine
// ============================================================================

/**
 * Core Reasoning Components
 */
export { ArbiterReasoningEngine } from "../reasoning/ArbiterReasoningEngine";
export type { ReasoningEngineConfig } from "../reasoning/ArbiterReasoningEngine";

export { ArgumentStructure } from "../reasoning/ArgumentStructure";
export { ConsensusEngine } from "../reasoning/ConsensusEngine";
export { DebateStateMachine } from "../reasoning/DebateStateMachine";
export { EvidenceAggregator } from "../reasoning/EvidenceAggregator";

/**
 * Multi-Agent Coordination Components
 */
export { AgentCoordinator } from "../reasoning/AgentCoordinator";
export type { AssignmentStrategy } from "../reasoning/AgentCoordinator";

export { TurnManager } from "../reasoning/TurnManager";
export type {
  FairnessPolicy,
  TurnSchedulingAlgorithm,
} from "../reasoning/TurnManager";

export { AppealHandler } from "../reasoning/AppealHandler";
export { DeadlockResolver } from "../reasoning/DeadlockResolver";

// ============================================================================
// Shared Types
// ============================================================================

export type {
  AgentCapability,
  AgentRole,
  Appeal,
  AppealStatus,
  ArbitrationProtocolConfig,
  // Arbitration Types
  ArbitrationSession,
  ArbitrationState,
  ConsensusAlgorithm,
  ConsensusResult,
  ConstitutionalRule,
  ConstitutionalViolation,
  DebateArgument,
  DebateConfig,
  DebateEvidence,
  DebateParticipant,
  // Reasoning Types
  DebateSession,
  DebateState,
  DebateVote,
  Precedent,
  PrecedentApplicability,
  ReasoningStep,
  RuleCategory,
  RuleEngineConfig,
  Verdict,
  VerdictOutcome,
  ViolationSeverity,
  WaiverDecision,
  WaiverRequest,
  WaiverStatus,
} from "@/types/arbitration";

export type {
  AgentCapability as ReasoningAgentCapability,
  AgentRole as ReasoningAgentRole,
  ConsensusAlgorithm as ReasoningConsensusAlgorithm,
  ConsensusResult as ReasoningConsensusResult,
  DebateArgument as ReasoningDebateArgument,
  DebateConfig as ReasoningDebateConfig,
  DebateEvidence as ReasoningDebateEvidence,
  DebateParticipant as ReasoningDebateParticipant,
  DebateSession as ReasoningDebateSession,
  DebateState as ReasoningDebateState,
  DebateVote as ReasoningDebateVote,
} from "@/types/reasoning";

// Error classes
export { ArbitrationError } from "@/types/arbitration";
export { ReasoningEngineError } from "@/types/reasoning";

// ============================================================================
// Integration Utilities
// ============================================================================

/**
 * Create integrated arbitration system with debate capabilities
 */
export function createArbitrationSystem(config?: {
  arbitration?: Partial<ArbitrationOrchestratorConfig>;
  reasoning?: Partial<ReasoningEngineConfig>;
}) {
  const orchestrator = new ArbitrationOrchestrator(config?.arbitration);
  const reasoningEngine = new ArbiterReasoningEngine(config?.reasoning);

  return {
    orchestrator,
    reasoningEngine,
    components: {
      ...orchestrator.getComponents(),
      reasoningEngine,
    },
  };
}

/**
 * Type guard for arbitration session
 */
export function isArbitrationSession(
  session: any
): session is ArbitrationSession {
  return (
    session &&
    typeof session.id === "string" &&
    typeof session.state === "string" &&
    session.violation !== undefined
  );
}

/**
 * Type guard for debate session
 */
export function isDebateSession(session: any): session is DebateSession {
  return (
    session &&
    typeof session.id === "string" &&
    typeof session.state === "string" &&
    session.topic !== undefined
  );
}

// Import statements at top
import type { ArbitrationSession } from "@/types/arbitration";
import type { DebateSession } from "@/types/reasoning";
import type { ReasoningEngineConfig } from "../reasoning/ArbiterReasoningEngine";
import { ArbiterReasoningEngine } from "../reasoning/ArbiterReasoningEngine";
import type { ArbitrationOrchestratorConfig } from "./ArbitrationOrchestrator";
import { ArbitrationOrchestrator } from "./ArbitrationOrchestrator";
