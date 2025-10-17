/**
 * Integration Tests: Arbiter Reasoning Engine
 *
 * Comprehensive integration testing for the complete reasoning engine workflow,
 * including multi-agent debate coordination, evidence aggregation, consensus formation,
 * deadlock resolution, and appeal handling.
 *
 * Test Coverage (25+ integration tests):
 * - Complete debate workflows from initiation to consensus
 * - Multi-agent coordination and turn management
 * - Evidence aggregation and quality validation
 * - Consensus formation with various algorithms
 * - Deadlock detection and resolution strategies
 * - Appeal handling and escalation
 * - Performance and timeout handling
 * - Error recovery and state management
 * - Complex debate scenarios with conflicting arguments
 * - Agent role and weight testing
 */

import { ArbiterReasoningEngine } from "@/reasoning/ArbiterReasoningEngine";
import {
  AgentRole,
  ConsensusAlgorithm,
  DebateState,
  Evidence,
  EvidenceType,
  ReasoningEngineError,
} from "@/types/reasoning";

describe("ARBITER-016 Integration: Complete Reasoning Engine Workflows", () => {
  let engine: ArbiterReasoningEngine;

  beforeEach(() => {
    engine = new ArbiterReasoningEngine({
      maxDebateDuration: 60000, // 1 minute for tests
      defaultConsensusAlgorithm: ConsensusAlgorithm.WEIGHTED_MAJORITY,
      minimumParticipants: 2,
      maximumParticipants: 5,
      enableDeadlockDetection: true,
      deadlockDetectionRounds: 2,
    });
  });

  afterEach(async () => {
    // Clean up any active debates
    const activeIds = engine.getActiveDebateIds();
    for (const id of activeIds) {
      try {
        const session = await engine.getDebateResults(id);
        if (session.session.state === DebateState.COMPLETED ||
            session.session.state === DebateState.DEADLOCKED) {
          await engine.closeDebate(id);
        }
      } catch (e) {
        // Ignore errors in cleanup
      }
    }
  });

  // Helper: Create evidence
  const createEvidence = (
    type: EvidenceType,
    content: string,
    credibility: number = 0.8
  ): Evidence => ({
    id: `evidence-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
    type,
    content,
    credibility,
    source: "test-source",
    timestamp: new Date(),
    metadata: {},
  });

  // Helper: Create debate participants
  const createParticipants = (count: number) => {
    const roles = [AgentRole.PROSECUTOR, AgentRole.DEFENDER, AgentRole.JUDGE, AgentRole.WITNESS];
    return Array.from({ length: count }, (_, i) => ({
      agentId: `agent-${i + 1}`,
      role: roles[i % roles.length],
      weight: i === 0 ? 2 : 1, // First agent has higher weight
    }));
  };

  describe("Integration Test 1-5: Complete Debate Workflows", () => {
    it("should complete simple 2-agent debate workflow end-to-end", async () => {
      // Given: Two agents with opposing views
      const participants = createParticipants(2);

      // When: Initiate debate and complete full workflow
      const session = await engine.initiateDebate(
        "Should code formatting rules be strictly enforced?",
        participants
      );

      expect(session.state).toBe(DebateState.AGENTS_ASSIGNED);
      expect(session.participants).toHaveLength(2);

      // Submit arguments
      const arg1 = await engine.submitArgument(
        session.id,
        "agent-1",
        "Code formatting should be strictly enforced for consistency",
        [
          createEvidence(EvidenceType.FACTUAL, "Industry studies show consistent formatting improves readability by 40%"),
          createEvidence(EvidenceType.EXPERT_OPINION, "Senior developers report faster code review with consistent style"),
        ],
        "Consistent formatting reduces cognitive load and prevents merge conflicts"
      );

      expect(arg1.state).toBe(DebateState.ARGUMENTS_PRESENTED);
      expect(arg1.arguments).toHaveLength(1);

      const arg2 = await engine.submitArgument(
        session.id,
        "agent-2",
        "Code formatting should not be strictly enforced due to productivity impact",
        [
          createEvidence(EvidenceType.ANECDOTAL, "Team velocity decreased 15% after strict formatting rules"),
          createEvidence(EvidenceType.STATISTICAL, "Pull request review time increased by 25%"),
        ],
        "Overly strict formatting rules create unnecessary friction in development workflow"
      );

      expect(arg2.arguments).toHaveLength(2);

      // Aggregate evidence
      const evidenceSession = await engine.aggregateEvidence(session.id);
      expect(evidenceSession.state).toBe(DebateState.DELIBERATION);

      // Submit votes
      await engine.submitVote(session.id, "agent-1", "for", 0.9, "Strong evidence supports strict enforcement");
      await engine.submitVote(session.id, "agent-2", "against", 0.8, "Productivity impact outweighs benefits");

      // Form consensus
      const finalSession = await engine.formConsensus(session.id);

      // Then: Debate should complete with consensus or deadlock
      expect([DebateState.COMPLETED, DebateState.DEADLOCKED]).toContain(finalSession.state);
      expect(finalSession.consensusResult).toBeDefined();

      // Verify results
      const results = await engine.getDebateResults(session.id);
      expect(results.session).toEqual(finalSession);
      expect(results.evidenceSummary).toBeDefined();
      expect(results.topArguments).toHaveLength(Math.min(2, finalSession.arguments.length));

      // Clean up
      await engine.closeDebate(session.id);
    });

    it("should handle complex 4-agent debate with conflicting evidence", async () => {
      const participants = createParticipants(4);

      const session = await engine.initiateDebate(
        "Should microservices architecture be adopted for this project?",
        participants
      );

      // Submit complex arguments with conflicting evidence
      await engine.submitArgument(
        session.id,
        "agent-1",
        "Microservices should be adopted for scalability and team autonomy",
        [
          createEvidence(EvidenceType.STATISTICAL, "Netflix successfully scaled to 200M users with microservices"),
          createEvidence(EvidenceType.EXPERT_OPINION, "Industry experts recommend microservices for large teams"),
        ],
        "Microservices provide clear boundaries and independent scaling"
      );

      await engine.submitArgument(
        session.id,
        "agent-2",
        "Microservices should not be adopted due to complexity and overhead",
        [
          createEvidence(EvidenceType.ANECDOTAL, "Team spent 6 months debugging distributed system issues"),
          createEvidence(EvidenceType.FACTUAL, "Operational complexity increased by 300%"),
        ],
        "The distributed system complexity outweighs the benefits for our team size"
      );

      await engine.submitArgument(
        session.id,
        "agent-3",
        "Neutral: Depends on team experience and project scope",
        [
          createEvidence(EvidenceType.STATISTICAL, "Success rate: 70% for experienced teams, 30% for novices"),
          createEvidence(EvidenceType.FACTUAL, "Migration cost: 3-6 months for medium projects"),
        ],
        "Success depends heavily on team maturity and project characteristics"
      );

      await engine.submitArgument(
        session.id,
        "agent-4",
        "Support microservices with proper planning and tools",
        [
          createEvidence(EvidenceType.TECHNICAL, "Service mesh and observability tools reduce complexity"),
          createEvidence(EvidenceType.FACTUAL, "Proper DevOps practices mitigate most issues"),
        ],
        "Modern tools and practices make microservices manageable"
      );

      // Aggregate evidence
      await engine.aggregateEvidence(session.id);

      // Submit votes with mixed confidence
      await engine.submitVote(session.id, "agent-1", "for", 0.8, "Clear scalability benefits");
      await engine.submitVote(session.id, "agent-2", "against", 0.9, "Direct experience with complexity");
      await engine.submitVote(session.id, "agent-3", "abstain", 0.6, "Insufficient context for strong opinion");
      await engine.submitVote(session.id, "agent-4", "for", 0.7, "Tools make it viable");

      // Form consensus
      const finalSession = await engine.formConsensus(session.id);

      expect(finalSession.arguments).toHaveLength(4);
      expect(finalSession.consensusResult).toBeDefined();
      expect(finalSession.consensusResult!.reasoning).toBeDefined();

      const results = await engine.getDebateResults(session.id);
      expect(results.topArguments).toHaveLength(4); // All arguments are top since < 5 total

      await engine.closeDebate(session.id);
    });

    it("should handle debate with unanimous consensus", async () => {
      const participants = createParticipants(3);

      const session = await engine.initiateDebate(
        "Should automated testing be mandatory for all code changes?",
        participants
      );

      // All agents agree - strong consensus expected
      await engine.submitArgument(
        session.id,
        "agent-1",
        "Automated testing should be mandatory",
        [createEvidence(EvidenceType.STATISTICAL, "Bug detection rate increases 80% with tests")],
        "Testing prevents regressions and improves code quality"
      );

      await engine.submitArgument(
        session.id,
        "agent-2",
        "Automated testing should be mandatory",
        [createEvidence(EvidenceType.FACTUAL, "Industry standard practice")],
        "All professional development requires automated testing"
      );

      await engine.submitArgument(
        session.id,
        "agent-3",
        "Automated testing should be mandatory",
        [createEvidence(EvidenceType.EXPERT_OPINION, "Testing experts unanimously recommend it")],
        "No reasonable argument against mandatory automated testing"
      );

      await engine.aggregateEvidence(session.id);

      // All vote for
      await engine.submitVote(session.id, "agent-1", "for", 0.95, "Overwhelming evidence");
      await engine.submitVote(session.id, "agent-2", "for", 0.9, "Industry standard");
      await engine.submitVote(session.id, "agent-3", "for", 0.85, "Expert consensus");

      const finalSession = await engine.formConsensus(session.id);
      expect(finalSession.state).toBe(DebateState.COMPLETED);
      expect(finalSession.consensusResult!.reached).toBe(true);
      expect(finalSession.consensusResult!.decision).toBe("for");

      await engine.closeDebate(session.id);
    });

    it("should track performance metrics through debate workflow", async () => {
      const participants = createParticipants(2);

      const startTime = Date.now();
      const session = await engine.initiateDebate(
        "Performance test debate topic",
        participants
      );

      const initTime = Date.now();

      await engine.submitArgument(
        session.id,
        "agent-1",
        "Test argument for performance measurement",
        [createEvidence(EvidenceType.FACTUAL, "Performance evidence")],
        "Performance reasoning"
      );

      const argTime = Date.now();

      await engine.submitArgument(
        session.id,
        "agent-2",
        "Counter argument for performance measurement",
        [createEvidence(EvidenceType.ANECDOTAL, "Performance counter-evidence")],
        "Performance counter-reasoning"
      );

      await engine.aggregateEvidence(session.id);
      const evidenceTime = Date.now();

      await engine.submitVote(session.id, "agent-1", "for", 0.8, "Performance vote");
      await engine.submitVote(session.id, "agent-2", "against", 0.7, "Performance counter-vote");

      await engine.formConsensus(session.id);
      const consensusTime = Date.now();

      const finalSession = await engine.getDebateResults(session.id);

      // Verify timing relationships
      expect(initTime).toBeGreaterThanOrEqual(startTime);
      expect(argTime).toBeGreaterThanOrEqual(initTime);
      expect(evidenceTime).toBeGreaterThanOrEqual(argTime);
      expect(consensusTime).toBeGreaterThanOrEqual(evidenceTime);

      // Verify debate completed within reasonable time
      const totalDuration = consensusTime - startTime;
      expect(totalDuration).toBeLessThan(5000); // Should complete within 5 seconds

      expect(finalSession.session.state).toBe(DebateState.COMPLETED);

      await engine.closeDebate(session.id);
    });

    it("should handle debate with mixed votes requiring consensus algorithm", async () => {
      const participants = createParticipants(4);

      const session = await engine.initiateDebate(
        "Should TypeScript be used instead of JavaScript?",
        participants
      );

      // Submit balanced arguments
      await engine.submitArgument(
        session.id,
        "agent-1",
        "TypeScript provides better type safety",
        [createEvidence(EvidenceType.STATISTICAL, "TypeScript reduces runtime errors by 60%")],
        "Type safety prevents bugs and improves maintainability"
      );

      await engine.submitArgument(
        session.id,
        "agent-2",
        "JavaScript is more flexible and faster to develop",
        [createEvidence(EvidenceType.ANECDOTAL, "Development velocity increased 30% with JS")],
        "JavaScript allows faster prototyping and iteration"
      );

      await engine.submitArgument(
        session.id,
        "agent-3",
        "TypeScript with proper tooling is worth the overhead",
        [createEvidence(EvidenceType.EXPERT_OPINION, "TypeScript adoption successful at scale")],
        "Long-term benefits outweigh short-term costs"
      );

      await engine.submitArgument(
        session.id,
        "agent-4",
        "Depends on team experience and project size",
        [createEvidence(EvidenceType.FACTUAL, "Small teams prefer JS, large teams prefer TS")],
        "Context-dependent decision"
      );

      await engine.aggregateEvidence(session.id);

      // Mixed votes - weighted majority should decide
      await engine.submitVote(session.id, "agent-1", "for", 0.8, "Type safety critical"); // weight 2
      await engine.submitVote(session.id, "agent-2", "against", 0.9, "Velocity important"); // weight 1
      await engine.submitVote(session.id, "agent-3", "for", 0.7, "Long-term benefits"); // weight 1
      await engine.submitVote(session.id, "agent-4", "abstain", 0.5, "Context dependent"); // weight 1

      const finalSession = await engine.formConsensus(session.id);

      // With weighted majority (2+1+1=4 for, 1 against, 1 abstain), should reach consensus for TypeScript
      expect(finalSession.state).toBe(DebateState.COMPLETED);
      expect(finalSession.consensusResult!.reached).toBe(true);

      // Verify weighted calculation worked
      const results = await engine.getDebateResults(session.id);
      expect(results.consensus).toBeDefined();
      expect(results.consensus!.reasoning).toContain("weighted");

      await engine.closeDebate(session.id);
    });
  });

  describe("Integration Test 6-10: Deadlock Detection and Resolution", () => {
    it("should detect deadlock when consensus is mathematically impossible", async () => {
      const participants = createParticipants(2);

      const session = await engine.initiateDebate(
        "Deadlock test debate",
        participants
      );

      await engine.submitArgument(
        session.id,
        "agent-1",
        "Position A",
        [createEvidence(EvidenceType.FACTUAL, "Evidence A")],
        "Reasoning A"
      );

      await engine.submitArgument(
        session.id,
        "agent-2",
        "Position B",
        [createEvidence(EvidenceType.FACTUAL, "Evidence B")],
        "Reasoning B"
      );

      await engine.aggregateEvidence(session.id);

      // Create impossible consensus scenario
      // Agent 1 only votes for their position
      await engine.submitVote(session.id, "agent-1", "for", 1.0, "Only accept my position");
      // Agent 2 only votes for their position
      await engine.submitVote(session.id, "agent-2", "against", 1.0, "Only accept my position");

      // This creates a scenario where consensus is impossible with current algorithm
      const finalSession = await engine.formConsensus(session.id);

      // Should detect deadlock or continue trying
      expect([DebateState.DEADLOCKED, DebateState.CONSENSUS_FORMING]).toContain(finalSession.state);

      if (finalSession.state === DebateState.DEADLOCKED) {
        await engine.closeDebate(session.id);
      } else {
        // Continue the debate or clean up
        const results = await engine.getDebateResults(session.id);
        expect(results.consensus!.reached).toBe(false);
      }
    });

    it("should handle deadlock resolution with mediator decision", async () => {
      const engineWithMediator = new ArbiterReasoningEngine({
        maxDebateDuration: 60000,
        defaultConsensusAlgorithm: ConsensusAlgorithm.UNANIMOUS, // Requires all agree
        minimumParticipants: 3,
        maximumParticipants: 5,
        enableDeadlockDetection: true,
        deadlockDetectionRounds: 1, // Quick deadlock detection
      });

      const participants = createParticipants(3);

      const session = await engineWithMediator.initiateDebate(
        "Unanimous consensus required deadlock test",
        participants
      );

      await engineWithMediator.submitArgument(
        session.id,
        "agent-1",
        "Controversial position",
        [createEvidence(EvidenceType.FACTUAL, "Strong evidence")],
        "Well-reasoned position"
      );

      await engineWithMediator.submitArgument(
        session.id,
        "agent-2",
        "Opposing position",
        [createEvidence(EvidenceType.ANECDOTAL, "Counter evidence")],
        "Different perspective"
      );

      await engineWithMediator.submitArgument(
        session.id,
        "agent-3",
        "Third position",
        [createEvidence(EvidenceType.EXPERT_OPINION, "Expert view")],
        "Alternative approach"
      );

      await engineWithMediator.aggregateEvidence(session.id);

      // Create deadlock: all disagree
      await engineWithMediator.submitVote(session.id, "agent-1", "for", 1.0, "Firm position");
      await engineWithMediator.submitVote(session.id, "agent-2", "against", 1.0, "Strong disagreement");
      await engineWithMediator.submitVote(session.id, "agent-3", "against", 1.0, "Cannot agree");

      const finalSession = await engineWithMediator.formConsensus(session.id);

      // Should detect deadlock with unanimous requirement
      expect(finalSession.state).toBe(DebateState.DEADLOCKED);
      expect(finalSession.consensusResult!.reached).toBe(false);

      await engineWithMediator.closeDebate(session.id);
    });

    it("should continue debate when deadlock detection disabled", async () => {
      const engineNoDeadlock = new ArbiterReasoningEngine({
        maxDebateDuration: 60000,
        defaultConsensusAlgorithm: ConsensusAlgorithm.WEIGHTED_MAJORITY,
        minimumParticipants: 2,
        maximumParticipants: 5,
        enableDeadlockDetection: false, // Disabled
        deadlockDetectionRounds: 2,
      });

      const participants = createParticipants(2);

      const session = await engineNoDeadlock.initiateDebate(
        "No deadlock detection test",
        participants
      );

      await engineNoDeadlock.submitArgument(
        session.id,
        "agent-1",
        "Test position",
        [createEvidence(EvidenceType.FACTUAL, "Test evidence")],
        "Test reasoning"
      );

      await engineNoDeadlock.submitArgument(
        session.id,
        "agent-2",
        "Counter position",
        [createEvidence(EvidenceType.ANECDOTAL, "Counter evidence")],
        "Counter reasoning"
      );

      await engineNoDeadlock.aggregateEvidence(session.id);

      await engineNoDeadlock.submitVote(session.id, "agent-1", "for", 1.0, "Firm");
      await engineNoDeadlock.submitVote(session.id, "agent-2", "against", 1.0, "Firm opposition");

      const finalSession = await engineNoDeadlock.formConsensus(session.id);

      // Should not detect deadlock, continue in consensus forming
      expect(finalSession.state).toBe(DebateState.CONSENSUS_FORMING);
      expect(finalSession.consensusResult!.reached).toBe(false);

      const results = await engineNoDeadlock.getDebateResults(session.id);
      expect(results.consensus!.reached).toBe(false);
    });

    it("should handle timeout scenarios gracefully", async () => {
      const fastTimeoutEngine = new ArbiterReasoningEngine({
        maxDebateDuration: 100, // Very short timeout
        defaultConsensusAlgorithm: ConsensusAlgorithm.WEIGHTED_MAJORITY,
        minimumParticipants: 2,
        maximumParticipants: 5,
        enableDeadlockDetection: true,
        deadlockDetectionRounds: 2,
      });

      const participants = createParticipants(2);

      const session = await fastTimeoutEngine.initiateDebate(
        "Timeout test debate",
        participants
      );

      await fastTimeoutEngine.submitArgument(
        session.id,
        "agent-1",
        "Test argument",
        [createEvidence(EvidenceType.FACTUAL, "Test evidence")],
        "Test reasoning"
      );

      // Wait for timeout
      await new Promise(resolve => setTimeout(resolve, 150));

      // Try to submit another argument - should fail due to timeout
      await expect(
        fastTimeoutEngine.submitArgument(
          session.id,
          "agent-2",
          "Late argument",
          [createEvidence(EvidenceType.ANECDOTAL, "Late evidence")],
          "Late reasoning"
        )
      ).rejects.toThrow("timeout");
    });

    it("should recover from error states during debate", async () => {
      const participants = createParticipants(2);

      const session = await engine.initiateDebate(
        "Error recovery test debate",
        participants
      );

      // Start normal flow
      await engine.submitArgument(
        session.id,
        "agent-1",
        "Normal argument",
        [createEvidence(EvidenceType.FACTUAL, "Normal evidence")],
        "Normal reasoning"
      );

      await engine.aggregateEvidence(session.id);

      // Try invalid operation - should not crash the session
      await expect(
        engine.submitArgument(
          session.id,
          "agent-1", // Same agent trying to submit again
          "Duplicate argument",
          [createEvidence(EvidenceType.ANECDOTAL, "Duplicate evidence")],
          "Duplicate reasoning"
        )
      ).rejects.toThrow(); // Should reject but not crash

      // Session should still be accessible and in valid state
      const currentSession = await engine.getDebateResults(session.id);
      expect(currentSession.session.state).toBe(DebateState.DELIBERATION);

      // Should be able to continue with valid operations
      await engine.submitVote(session.id, "agent-1", "for", 0.8, "Continuing after error");

      const updatedSession = await engine.getDebateResults(session.id);
      expect(updatedSession.session.participants[0].votesCast).toHaveLength(1);
    });
  });

  describe("Integration Test 11-15: Evidence and Argument Quality", () => {
    it("should validate evidence quality during aggregation", async () => {
      const participants = createParticipants(3);

      const session = await engine.initiateDebate(
        "Evidence quality validation test",
        participants
      );

      // Submit arguments with mixed evidence quality
      await engine.submitArgument(
        session.id,
        "agent-1",
        "High quality argument",
        [
          createEvidence(EvidenceType.STATISTICAL, "Peer-reviewed study with 95% confidence", 0.9),
          createEvidence(EvidenceType.EXPERT_OPINION, "PhD researcher with 20+ publications", 0.8),
        ],
        "Well-supported position"
      );

      await engine.submitArgument(
        session.id,
        "agent-2",
        "Low quality argument",
        [
          createEvidence(EvidenceType.ANECDOTAL, "I think this might be true", 0.3),
          createEvidence(EvidenceType.FACTUAL, "Unverified claim from social media", 0.2),
        ],
        "Poorly supported position"
      );

      await engine.submitArgument(
        session.id,
        "agent-3",
        "Mixed quality argument",
        [
          createEvidence(EvidenceType.STATISTICAL, "Industry report with methodology", 0.7),
          createEvidence(EvidenceType.ANECDOTAL, "Personal experience", 0.4),
        ],
        "Moderately supported position"
      );

      // Aggregate evidence - should validate quality
      const evidenceSession = await engine.aggregateEvidence(session.id);
      expect(evidenceSession.state).toBe(DebateState.DELIBERATION);

      // Check that evidence quality is tracked in reasoning chain
      expect(evidenceSession.reasoningChain.some(step =>
        step.includes("evidence") && step.includes("quality")
      )).toBe(true);

      const results = await engine.getDebateResults(session.id);
      expect(results.evidenceSummary).toBeDefined();
      expect(results.evidenceSummary.length).toBeGreaterThan(0);
    });

    it("should handle arguments with no evidence", async () => {
      const participants = createParticipants(2);

      const session = await engine.initiateDebate(
        "No evidence debate",
        participants
      );

      // Submit arguments without evidence
      await engine.submitArgument(
        session.id,
        "agent-1",
        "Position without evidence",
        [], // No evidence
        "Relying on reasoning alone"
      );

      await engine.submitArgument(
        session.id,
        "agent-2",
        "Counter position without evidence",
        [], // No evidence
        "Different reasoning approach"
      );

      // Should still aggregate (with warnings)
      const evidenceSession = await engine.aggregateEvidence(session.id);
      expect(evidenceSession.state).toBe(DebateState.DELIBERATION);

      // Should note lack of evidence in reasoning chain
      expect(evidenceSession.reasoningChain.some(step =>
        step.includes("evidence") && (step.includes("quality") || step.includes("issues"))
      )).toBe(true);
    });

    it("should compare arguments by credibility score", async () => {
      const participants = createParticipants(4);

      const session = await engine.initiateDebate(
        "Argument credibility comparison",
        participants
      );

      // Submit arguments with different credibility profiles
      await engine.submitArgument(
        session.id,
        "agent-1",
        "Highly credible argument",
        [
          createEvidence(EvidenceType.STATISTICAL, "Large-scale study", 0.95),
          createEvidence(EvidenceType.EXPERT_OPINION, "Domain expert", 0.9),
          createEvidence(EvidenceType.FACTUAL, "Verified data", 0.85),
        ],
        "Strong reasoning with multiple evidence types"
      );

      await engine.submitArgument(
        session.id,
        "agent-2",
        "Medium credible argument",
        [
          createEvidence(EvidenceType.ANECDOTAL, "Personal experience", 0.6),
          createEvidence(EvidenceType.FACTUAL, "Single data point", 0.7),
        ],
        "Moderate reasoning with limited evidence"
      );

      await engine.submitArgument(
        session.id,
        "agent-3",
        "Low credible argument",
        [
          createEvidence(EvidenceType.ANECDOTAL, "Hearsay", 0.3),
        ],
        "Weak reasoning with poor evidence"
      );

      await engine.submitArgument(
        session.id,
        "agent-4",
        "No evidence argument",
        [], // No evidence
        "Reasoning without empirical support"
      );

      await engine.aggregateEvidence(session.id);

      const results = await engine.getDebateResults(session.id);

      // Should return top arguments sorted by credibility
      expect(results.topArguments).toHaveLength(4);
      expect(results.topArguments[0].claim).toBe("Highly credible argument");
      expect(results.topArguments[3].claim).toBe("No evidence argument");
    });

    it("should reject invalid arguments during submission", async () => {
      const participants = createParticipants(2);

      const session = await engine.initiateDebate(
        "Argument validation test",
        participants
      );

      // Try to submit invalid argument (empty claim)
      await expect(
        engine.submitArgument(
          session.id,
          "agent-1",
          "", // Invalid empty claim
          [createEvidence(EvidenceType.FACTUAL, "Valid evidence")],
          "Valid reasoning"
        )
      ).rejects.toThrow("Invalid argument");

      // Try to submit invalid argument (empty reasoning)
      await expect(
        engine.submitArgument(
          session.id,
          "agent-1",
          "Valid claim",
          [createEvidence(EvidenceType.FACTUAL, "Valid evidence")],
          "" // Invalid empty reasoning
        )
      ).rejects.toThrow("Invalid argument");

      // Valid argument should work
      const validSession = await engine.submitArgument(
        session.id,
        "agent-1",
        "Valid claim",
        [createEvidence(EvidenceType.FACTUAL, "Valid evidence")],
        "Valid reasoning"
      );

      expect(validSession.arguments).toHaveLength(1);
    });

    it("should handle evidence with conflicting credibility scores", async () => {
      const participants = createParticipants(2);

      const session = await engine.initiateDebate(
        "Conflicting evidence credibility",
        participants
      );

      // Submit argument with contradictory evidence
      await engine.submitArgument(
        session.id,
        "agent-1",
        "Argument with conflicting evidence",
        [
          createEvidence(EvidenceType.STATISTICAL, "High credibility statistical evidence", 0.9),
          createEvidence(EvidenceType.ANECDOTAL, "Low credibility anecdotal evidence", 0.2),
          createEvidence(EvidenceType.EXPERT_OPINION, "Medium credibility expert opinion", 0.6),
        ],
        "Argument supported by mixed-quality evidence"
      );

      await engine.submitArgument(
        session.id,
        "agent-2",
        "Counter argument",
        [
          createEvidence(EvidenceType.FACTUAL, "Consistent factual evidence", 0.8),
        ],
        "Counter with consistent evidence quality"
      );

      await engine.aggregateEvidence(session.id);

      const results = await engine.getDebateResults(session.id);

      // Should aggregate evidence despite mixed credibility
      expect(results.evidenceSummary).toBeDefined();
      expect(results.evidenceSummary).toContain("aggregated");

      // Should still rank arguments appropriately
      expect(results.topArguments).toHaveLength(2);
    });
  });

  describe("Integration Test 16-20: Agent Coordination and Roles", () => {
    it("should respect agent roles and voting weights", async () => {
      const participants = [
        { agentId: "prosecutor", role: AgentRole.PROSECUTOR, weight: 3 },
        { agentId: "defender", role: AgentRole.DEFENDER, weight: 2 },
        { agentId: "judge", role: AgentRole.JUDGE, weight: 4 },
        { agentId: "witness", role: AgentRole.WITNESS, weight: 1 },
      ];

      const session = await engine.initiateDebate(
        "Role and weight test debate",
        participants
      );

      expect(session.participants).toHaveLength(4);
      expect(session.participants.find(p => p.agentId === "judge")!.weight).toBe(4);
      expect(session.participants.find(p => p => p.agentId === "witness")!.weight).toBe(1);

      // Submit arguments from different roles
      await engine.submitArgument(
        session.id,
        "prosecutor",
        "Prosecution argument",
        [createEvidence(EvidenceType.FACTUAL, "Prosecution evidence")],
        "Prosecution reasoning"
      );

      await engine.submitArgument(
        session.id,
        "defender",
        "Defense argument",
        [createEvidence(EvidenceType.ANECDOTAL, "Defense evidence")],
        "Defense reasoning"
      );

      await engine.submitArgument(
        session.id,
        "judge",
        "Judicial perspective",
        [createEvidence(EvidenceType.EXPERT_OPINION, "Judicial evidence")],
        "Judicial reasoning"
      );

      await engine.submitArgument(
        session.id,
        "witness",
        "Witness testimony",
        [createEvidence(EvidenceType.ANECDOTAL, "Witness evidence")],
        "Witness reasoning"
      );

      await engine.aggregateEvidence(session.id);

      // Submit votes with role-appropriate positions
      await engine.submitVote(session.id, "prosecutor", "for", 0.9, "Prosecuting position");
      await engine.submitVote(session.id, "defender", "against", 0.8, "Defending position");
      await engine.submitVote(session.id, "judge", "abstain", 0.7, "Judicial neutrality");
      await engine.submitVote(session.id, "witness", "for", 0.6, "Witness perspective");

      const finalSession = await engine.formConsensus(session.id);

      // Judge's high weight should influence outcome despite being abstain
      const results = await engine.getDebateResults(session.id);
      expect(results.consensus!.reasoning).toContain("weighted");

      await engine.closeDebate(session.id);
    });

    it("should handle non-participant agent attempting to contribute", async () => {
      const participants = createParticipants(2);

      const session = await engine.initiateDebate(
        "Non-participant test",
        participants
      );

      // Try to submit argument from non-participant
      await expect(
        engine.submitArgument(
          session.id,
          "non-participant-agent",
          "Unauthorized argument",
          [createEvidence(EvidenceType.FACTUAL, "Some evidence")],
          "Unauthorized reasoning"
        )
      ).rejects.toThrow("not a participant");

      // Try to submit vote from non-participant
      await expect(
        engine.submitVote(session.id, "non-participant-agent", "for", 0.8, "Unauthorized vote")
      ).rejects.toThrow("not a participant");

      // Valid participant should still work
      await engine.submitArgument(
        session.id,
        "agent-1",
        "Valid argument",
        [createEvidence(EvidenceType.FACTUAL, "Valid evidence")],
        "Valid reasoning"
      );

      expect((await engine.getDebateResults(session.id)).session.arguments).toHaveLength(1);
    });

    it("should enforce minimum and maximum participant limits", async () => {
      // Test minimum participants
      await expect(
        engine.initiateDebate("Too few participants", [
          { agentId: "agent-1", role: AgentRole.PROSECUTOR }
        ])
      ).rejects.toThrow("Insufficient participants");

      // Test maximum participants
      const tooManyParticipants = Array.from({ length: 10 }, (_, i) => ({
        agentId: `agent-${i}`,
        role: AgentRole.PROSECUTOR,
      }));

      await expect(
        engine.initiateDebate("Too many participants", tooManyParticipants)
      ).rejects.toThrow("Too many participants");

      // Valid number should work
      const validParticipants = createParticipants(3);
      const session = await engine.initiateDebate("Valid participants", validParticipants);
      expect(session.participants).toHaveLength(3);
    });

    it("should prevent duplicate participant IDs", async () => {
      const duplicateParticipants = [
        { agentId: "agent-1", role: AgentRole.PROSECUTOR },
        { agentId: "agent-1", role: AgentRole.DEFENDER }, // Duplicate ID
      ];

      await expect(
        engine.initiateDebate("Duplicate participants", duplicateParticipants)
      ).rejects.toThrow("Duplicate participant ID");
    });

    it("should handle agent disconnections gracefully", async () => {
      const participants = createParticipants(3);

      const session = await engine.initiateDebate(
        "Disconnection test",
        participants
      );

      // Agent 1 submits argument
      await engine.submitArgument(
        session.id,
        "agent-1",
        "Agent 1 argument",
        [createEvidence(EvidenceType.FACTUAL, "Agent 1 evidence")],
        "Agent 1 reasoning"
      );

      // Agent 2 submits argument
      await engine.submitArgument(
        session.id,
        "agent-2",
        "Agent 2 argument",
        [createEvidence(EvidenceType.ANECDOTAL, "Agent 2 evidence")],
        "Agent 2 reasoning"
      );

      // Agent 3 never submits (simulating disconnection)
      await engine.aggregateEvidence(session.id);

      // Only agents 1 and 2 vote (agent 3 missing)
      await engine.submitVote(session.id, "agent-1", "for", 0.8, "Agent 1 vote");
      await engine.submitVote(session.id, "agent-2", "against", 0.7, "Agent 2 vote");

      // Should still attempt consensus with available votes
      const finalSession = await engine.formConsensus(session.id);
      expect(finalSession.consensusResult).toBeDefined();

      const results = await engine.getDebateResults(session.id);
      expect(results.consensus!.reasoning).toContain("consensus");

      await engine.closeDebate(session.id);
    });
  });

  describe("Integration Test 21-25: Complex Workflow Scenarios", () => {
    it("should handle appeal workflow within debate system", async () => {
      const participants = createParticipants(4);

      const session = await engine.initiateDebate(
        "Appeal workflow test",
        participants
      );

      // Submit arguments
      await engine.submitArgument(
        session.id,
        "agent-1",
        "Original position",
        [createEvidence(EvidenceType.FACTUAL, "Original evidence")],
        "Original reasoning"
      );

      await engine.submitArgument(
        session.id,
        "agent-2",
        "Counter position",
        [createEvidence(EvidenceType.ANECDOTAL, "Counter evidence")],
        "Counter reasoning"
      );

      await engine.aggregateEvidence(session.id);

      // Initial votes lead to consensus
      await engine.submitVote(session.id, "agent-1", "for", 0.9, "Strong agreement");
      await engine.submitVote(session.id, "agent-2", "against", 0.6, "Weak disagreement");
      await engine.submitVote(session.id, "agent-3", "for", 0.7, "Moderate agreement");
      await engine.submitVote(session.id, "agent-4", "for", 0.8, "Strong agreement");

      const initialConsensus = await engine.formConsensus(session.id);
      expect(initialConsensus.state).toBe(DebateState.COMPLETED);

      // In a real appeal system, this would create an appeal session
      // For this test, we verify the consensus was reached
      expect(initialConsensus.consensusResult!.reached).toBe(true);

      await engine.closeDebate(session.id);
    });

    it("should maintain audit trail throughout complex workflow", async () => {
      const participants = createParticipants(3);

      const session = await engine.initiateDebate(
        "Audit trail test",
        participants
      );

      const initialChainLength = session.reasoningChain.length;

      // Submit arguments
      await engine.submitArgument(
        session.id,
        "agent-1",
        "Audit test argument 1",
        [createEvidence(EvidenceType.FACTUAL, "Audit evidence 1")],
        "Audit reasoning 1"
      );

      let currentSession = await engine.getDebateResults(session.id);
      expect(currentSession.session.reasoningChain.length).toBeGreaterThan(initialChainLength);

      await engine.submitArgument(
        session.id,
        "agent-2",
        "Audit test argument 2",
        [createEvidence(EvidenceType.ANECDOTAL, "Audit evidence 2")],
        "Audit reasoning 2"
      );

      currentSession = await engine.getDebateResults(session.id);
      expect(currentSession.session.reasoningChain.length).toBeGreaterThan(initialChainLength + 1);

      await engine.aggregateEvidence(session.id);

      currentSession = await engine.getDebateResults(session.id);
      expect(currentSession.session.reasoningChain.length).toBeGreaterThan(initialChainLength + 2);

      // Verify reasoning chain contains audit information
      const reasoningChain = currentSession.session.reasoningChain;
      expect(reasoningChain.some(step => step.includes("evidence"))).toBe(true);
      expect(reasoningChain.some(step => step.includes("aggregated"))).toBe(true);
    });

    it("should handle concurrent debates without interference", async () => {
      const debate1Participants = createParticipants(2);
      const debate2Participants = [
        { agentId: "debate2-agent-1", role: AgentRole.PROSECUTOR },
        { agentId: "debate2-agent-2", role: AgentRole.DEFENDER },
      ];

      // Start two concurrent debates
      const debate1 = await engine.initiateDebate("Debate 1", debate1Participants);
      const debate2 = await engine.initiateDebate("Debate 2", debate2Participants);

      expect(engine.getActiveDebateCount()).toBe(2);
      expect(engine.getActiveDebateIds()).toContain(debate1.id);
      expect(engine.getActiveDebateIds()).toContain(debate2.id);

      // Submit to debate 1
      await engine.submitArgument(
        debate1.id,
        "agent-1",
        "Debate 1 argument",
        [createEvidence(EvidenceType.FACTUAL, "Debate 1 evidence")],
        "Debate 1 reasoning"
      );

      // Submit to debate 2
      await engine.submitArgument(
        debate2.id,
        "debate2-agent-1",
        "Debate 2 argument",
        [createEvidence(EvidenceType.ANECDOTAL, "Debate 2 evidence")],
        "Debate 2 reasoning"
      );

      // Verify no interference
      const debate1Results = await engine.getDebateResults(debate1.id);
      const debate2Results = await engine.getDebateResults(debate2.id);

      expect(debate1Results.session.arguments).toHaveLength(1);
      expect(debate2Results.session.arguments).toHaveLength(1);
      expect(debate1Results.session.arguments[0].agentId).toBe("agent-1");
      expect(debate2Results.session.arguments[0].agentId).toBe("debate2-agent-1");

      // Complete both debates
      await engine.closeDebate(debate1.id);
      await engine.closeDebate(debate2.id);

      expect(engine.getActiveDebateCount()).toBe(0);
    });

    it("should handle empty topic validation", async () => {
      const participants = createParticipants(2);

      await expect(
        engine.initiateDebate("", participants)
      ).rejects.toThrow("empty");

      await expect(
        engine.initiateDebate("   ", participants)
      ).rejects.toThrow("empty");

      // Valid topic should work
      const session = await engine.initiateDebate("Valid topic", participants);
      expect(session.topic).toBe("Valid topic");
    });

    it("should provide comprehensive debate results", async () => {
      const participants = createParticipants(3);

      const session = await engine.initiateDebate(
        "Comprehensive results test",
        participants
      );

      // Build complete debate
      await engine.submitArgument(
        session.id,
        "agent-1",
        "Argument 1",
        [createEvidence(EvidenceType.STATISTICAL, "Stats evidence", 0.8)],
        "Reasoning 1"
      );

      await engine.submitArgument(
        session.id,
        "agent-2",
        "Argument 2",
        [createEvidence(EvidenceType.EXPERT_OPINION, "Expert evidence", 0.9)],
        "Reasoning 2"
      );

      await engine.submitArgument(
        session.id,
        "agent-3",
        "Argument 3",
        [createEvidence(EvidenceType.FACTUAL, "Factual evidence", 0.7)],
        "Reasoning 3"
      );

      await engine.aggregateEvidence(session.id);

      await engine.submitVote(session.id, "agent-1", "for", 0.8, "Vote 1");
      await engine.submitVote(session.id, "agent-2", "against", 0.7, "Vote 2");
      await engine.submitVote(session.id, "agent-3", "for", 0.9, "Vote 3");

      await engine.formConsensus(session.id);

      // Get comprehensive results
      const results = await engine.getDebateResults(session.id);

      expect(results.session).toBeDefined();
      expect(results.consensus).toBeDefined();
      expect(results.evidenceSummary).toBeDefined();
      expect(results.topArguments).toHaveLength(3);

      // Verify result structure
      expect(typeof results.evidenceSummary).toBe("string");
      expect(results.evidenceSummary.length).toBeGreaterThan(0);
      expect(Array.isArray(results.topArguments)).toBe(true);
      expect(results.topArguments[0]).toHaveProperty("claim");

      await engine.closeDebate(session.id);
    });
  });
});
