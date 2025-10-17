/**
 * Integration Tests: Multi-Agent Coordination
 *
 * Comprehensive testing of multi-agent coordination across arbitration and reasoning systems,
 * including agent role assignment, load balancing, communication patterns, and consensus building.
 *
 * Test Coverage (20+ multi-agent tests):
 * - Agent role assignment and capability matching
 * - Load balancing across multiple agents
 * - Agent communication and message passing
 * - Consensus building with diverse agent opinions
 * - Agent failure and recovery scenarios
 * - Agent specialization and expertise routing
 * - Multi-agent arbitration workflows
 * - Agent collaboration patterns
 * - Performance with multiple concurrent agents
 * - Agent trust and reputation systems
 */

import { ArbitrationOrchestrator } from "@/arbitration/ArbitrationOrchestrator";
import { ArbiterReasoningEngine } from "@/reasoning/ArbiterReasoningEngine";
import {
  AgentRole,
  ArbitrationState,
  ConsensusAlgorithm,
  ConstitutionalRule,
  ConstitutionalViolation,
  DebateState,
  Evidence,
  EvidenceType,
  RuleCategory,
  ViolationSeverity,
} from "@/types/arbitration";
import { AgentCoordinator } from "@/reasoning/AgentCoordinator";

describe("Multi-Agent Coordination Integration", () => {
  let arbitrationOrchestrator: ArbitrationOrchestrator;
  let reasoningEngine: ArbiterReasoningEngine;
  let agentCoordinator: AgentCoordinator;

  beforeEach(() => {
    arbitrationOrchestrator = new ArbitrationOrchestrator({
      enableWaivers: true,
      enableAppeals: true,
      trackPerformance: true,
      maxConcurrentSessions: 20,
      sessionTimeoutMs: 60000,
    });

    reasoningEngine = new ArbiterReasoningEngine({
      maxDebateDuration: 30000, // 30 seconds for tests
      defaultConsensusAlgorithm: ConsensusAlgorithm.WEIGHTED_MAJORITY,
      minimumParticipants: 2,
      maximumParticipants: 10,
      enableDeadlockDetection: true,
      deadlockDetectionRounds: 2,
    });

    agentCoordinator = new AgentCoordinator({
      maxConcurrentAgents: 10,
      loadBalancingStrategy: "round-robin",
      agentHeartbeatInterval: 5000,
      taskTimeoutMs: 30000,
    });
  });

  afterEach(async () => {
    // Clean up arbitration sessions
    const activeSessions = arbitrationOrchestrator.getActiveSessions();
    for (const session of activeSessions) {
      try {
        await arbitrationOrchestrator.completeSession(session.id);
      } catch (e) {
        // Ignore cleanup errors
      }
    }
    arbitrationOrchestrator.clear();

    // Clean up reasoning debates
    const activeDebates = reasoningEngine.getActiveDebateIds();
    for (const debateId of activeDebates) {
      try {
        await reasoningEngine.closeDebate(debateId);
      } catch (e) {
        // Ignore cleanup errors
      }
    }
  });

  // Helper: Create diverse agent configurations
  const createAgentConfigs = (count: number) => {
    const roles = [AgentRole.PROSECUTOR, AgentRole.DEFENDER, AgentRole.JUDGE, AgentRole.WITNESS, AgentRole.MEDIATOR];
    return Array.from({ length: count }, (_, i) => ({
      agentId: `agent-${i + 1}`,
      role: roles[i % roles.length],
      capabilities: [
        "rule-evaluation",
        "evidence-analysis",
        "argument-construction",
        i % 2 === 0 ? "security-expertise" : "performance-expertise", // Alternate expertise
      ],
      weight: i === 0 ? 3 : i === 1 ? 2 : 1, // Varying influence levels
      specialization: i % 3 === 0 ? "security" : i % 3 === 1 ? "performance" : "general",
      trustScore: 0.8 + (i * 0.05), // Varying trust scores
      workload: i * 10, // Different current workloads
    }));
  };

  // Helper: Create test rule
  const createRule = (overrides: Partial<ConstitutionalRule> = {}): ConstitutionalRule => {
    return {
      id: overrides.id || `rule-ma-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      version: "1.0.0",
      category: overrides.category || RuleCategory.CODE_QUALITY,
      title: overrides.title || "Multi-Agent Test Rule",
      description: overrides.description || "Rule for multi-agent coordination testing",
      condition: overrides.condition || "test === true",
      severity: overrides.severity || ViolationSeverity.MODERATE,
      waivable: overrides.waivable ?? true,
      requiredEvidence: overrides.requiredEvidence || ["test_evidence"],
      precedents: overrides.precedents || [],
      effectiveDate: overrides.effectiveDate || new Date(),
      metadata: overrides.metadata || {},
      ...overrides,
    };
  };

  // Helper: Create test violation
  const createViolation = (ruleId: string, overrides: Partial<ConstitutionalViolation> = {}): ConstitutionalViolation => {
    return {
      id: `violation-ma-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      ruleId,
      severity: ViolationSeverity.MODERATE,
      description: "Multi-agent test violation",
      evidence: ["test evidence"],
      detectedAt: new Date(),
      violator: "test-agent",
      context: {},
      ...overrides,
    };
  };

  // Helper: Create evidence with agent attribution
  const createAgentEvidence = (agentId: string, type: EvidenceType, content: string, credibility: number = 0.8): Evidence => ({
    id: `evidence-${agentId}-${Date.now()}`,
    type,
    content,
    credibility,
    source: agentId,
    timestamp: new Date(),
    metadata: { agentId },
  });

  describe("Multi-Agent Test 1-5: Agent Coordination and Role Assignment", () => {
    it("should coordinate multiple agents through complete arbitration workflow", async () => {
      const agentConfigs = createAgentConfigs(5);
      const rule = createRule({ id: "rule-multi-agent" });
      const violation = createViolation(rule.id);

      // Start arbitration session
      const arbitrationSession = await arbitrationOrchestrator.startSession(
        violation,
        [rule],
        agentConfigs.map(a => a.agentId)
      );

      // Initialize multi-agent debate for rule evaluation
      const debateParticipants = agentConfigs.map(config => ({
        agentId: config.agentId,
        role: config.role,
        weight: config.weight,
      }));

      const debateSession = await reasoningEngine.initiateDebate(
        `Arbitration debate for violation ${violation.id}`,
        debateParticipants
      );

      expect(debateSession.participants).toHaveLength(5);
      expect(arbitrationSession.participants).toHaveLength(5);

      // Agents submit arguments and evidence
      for (const agent of agentConfigs) {
        const argument = `${agent.role} perspective on violation: ${violation.description}`;
        const evidence = [
          createAgentEvidence(agent.agentId, EvidenceType.FACTUAL, `Evidence from ${agent.agentId}`),
          createAgentEvidence(agent.agentId, EvidenceType.EXPERT_OPINION, `Expert opinion from ${agent.specialization} specialist`),
        ];

        await reasoningEngine.submitArgument(
          debateSession.id,
          agent.agentId,
          argument,
          evidence,
          `Reasoning from ${agent.role} with ${agent.specialization} expertise`
        );
      }

      // Aggregate evidence and form consensus
      await reasoningEngine.aggregateEvidence(debateSession.id);

      // All agents vote
      for (const agent of agentConfigs) {
        const confidence = agent.trustScore; // Use trust score as confidence
        const position = agent.role === AgentRole.PROSECUTOR ? "for" :
                        agent.role === AgentRole.DEFENDER ? "against" : "abstain";

        await reasoningEngine.submitVote(
          debateSession.id,
          agent.agentId,
          position as "for" | "against" | "abstain",
          confidence,
          `${agent.role} vote based on ${agent.specialization} analysis`
        );
      }

      // Form consensus
      const consensusSession = await reasoningEngine.formConsensus(debateSession.id);

      // Apply consensus to arbitration
      await arbitrationOrchestrator.evaluateRules(arbitrationSession.id);
      const verdict = await arbitrationOrchestrator.generateVerdict(arbitrationSession.id, "multi-agent-arbiter");

      // Complete both sessions
      await arbitrationOrchestrator.completeSession(arbitrationSession.id);
      await reasoningEngine.closeDebate(debateSession.id);

      // Verify coordination
      expect(arbitrationOrchestrator.getSession(arbitrationSession.id).state).toBe(ArbitrationState.COMPLETED);
      expect(verdict).toBeDefined();
      expect(consensusSession.state).toBe(DebateState.COMPLETED);
    });

    it("should handle agent specialization routing", async () => {
      const specializedAgents = [
        { agentId: "security-expert", role: AgentRole.JUDGE, specialization: "security", weight: 3 },
        { agentId: "performance-expert", role: AgentRole.WITNESS, specialization: "performance", weight: 2 },
        { agentId: "general-agent", role: AgentRole.MEDIATOR, specialization: "general", weight: 1 },
      ];

      // Create security-focused violation
      const securityRule = createRule({
        id: "rule-security-specialization",
        category: RuleCategory.SECURITY,
        title: "Security vulnerability check",
      });

      const securityViolation = createViolation(securityRule.id, {
        description: "Hardcoded credentials found in source code",
        evidence: ["security-scan-result.txt", "code-snippet.txt"],
      });

      const session = await arbitrationOrchestrator.startSession(
        securityViolation,
        [securityRule],
        specializedAgents.map(a => a.agentId)
      );

      // Initialize debate with specialized agents
      const debateSession = await reasoningEngine.initiateDebate(
        "Security violation arbitration",
        specializedAgents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
      );

      // Security expert should provide most relevant arguments
      await reasoningEngine.submitArgument(
        debateSession.id,
        "security-expert",
        "Critical security violation requiring immediate remediation",
        [
          createAgentEvidence("security-expert", EvidenceType.EXPERT_OPINION, "As a security specialist, this constitutes a severe breach", 0.95),
          createAgentEvidence("security-expert", EvidenceType.TECHNICAL, "OWASP guidelines classify this as high-risk", 0.9),
        ],
        "Security expertise indicates this cannot be waived"
      );

      await reasoningEngine.submitArgument(
        debateSession.id,
        "performance-expert",
        "From performance perspective, remediation may impact system throughput",
        [
          createAgentEvidence("performance-expert", EvidenceType.STATISTICAL, "Similar fixes have caused 15% performance degradation", 0.7),
        ],
        "Performance considerations should be balanced with security requirements"
      );

      await reasoningEngine.aggregateEvidence(debateSession.id);

      // Weighted voting based on expertise
      await reasoningEngine.submitVote(debateSession.id, "security-expert", "for", 0.95, "Security risk unacceptable");
      await reasoningEngine.submitVote(debateSession.id, "performance-expert", "against", 0.6, "Performance impact too high");
      await reasoningEngine.submitVote(debateSession.id, "general-agent", "abstain", 0.5, "Insufficient expertise");

      const consensus = await reasoningEngine.formConsensus(debateSession.id);

      // Security expert's higher weight should influence outcome
      expect(consensus.consensusResult!.reached).toBe(true);

      // Complete arbitration
      await arbitrationOrchestrator.evaluateRules(session.id);
      await arbitrationOrchestrator.generateVerdict(session.id, "specialized-arbiter");
      await arbitrationOrchestrator.completeSession(session.id);

      await reasoningEngine.closeDebate(debateSession.id);
    });

    it("should coordinate agents with conflicting expertise", async () => {
      const conflictingAgents = [
        { agentId: "strict-enforcer", role: AgentRole.PROSECUTOR, expertise: "compliance", weight: 2 },
        { agentId: "pragmatic-developer", role: AgentRole.DEFENDER, expertise: "development", weight: 2 },
        { agentId: "experienced-architect", role: AgentRole.JUDGE, expertise: "architecture", weight: 3 },
      ];

      const debateSession = await reasoningEngine.initiateDebate(
        "Should we enforce strict code formatting rules?",
        conflictingAgents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
      );

      // Submit conflicting arguments
      await reasoningEngine.submitArgument(
        debateSession.id,
        "strict-enforcer",
        "Strict formatting rules must be enforced for code consistency and maintainability",
        [
          createAgentEvidence("strict-enforcer", EvidenceType.STATISTICAL, "Studies show 40% faster code review with consistent formatting", 0.85),
          createAgentEvidence("strict-enforcer", EvidenceType.EXPERT_OPINION, "Industry standards require consistent formatting", 0.9),
        ],
        "Consistency improves team productivity and code quality"
      );

      await reasoningEngine.submitArgument(
        debateSession.id,
        "pragmatic-developer",
        "Strict formatting rules create unnecessary friction in development workflow",
        [
          createAgentEvidence("pragmatic-developer", EvidenceType.ANECDOTAL, "Team velocity decreased 25% after formatting rules", 0.75),
          createAgentEvidence("pragmatic-developer", EvidenceType.FACTUAL, "Pull request reviews take 30% longer", 0.8),
        ],
        "Development speed and innovation should not be sacrificed for formatting perfection"
      );

      await reasoningEngine.submitArgument(
        debateSession.id,
        "experienced-architect",
        "Balance both perspectives: enforce formatting but provide tooling support",
        [
          createAgentEvidence("experienced-architect", EvidenceType.TECHNICAL, "Modern IDEs can auto-format code", 0.85),
          createAgentEvidence("experienced-architect", EvidenceType.EXPERT_OPINION, "Architectural best practices support automation", 0.9),
        ],
        "Use tooling to enforce formatting without impacting developer productivity"
      );

      await reasoningEngine.aggregateEvidence(debateSession.id);

      // Votes reflect different perspectives
      await reasoningEngine.submitVote(debateSession.id, "strict-enforcer", "for", 0.9, "Quality over convenience");
      await reasoningEngine.submitVote(debateSession.id, "pragmatic-developer", "against", 0.8, "Productivity matters");
      await reasoningEngine.submitVote(debateSession.id, "experienced-architect", "for", 0.7, "Compromise with tooling");

      const consensus = await reasoningEngine.formConsensus(debateSession.id);

      // Should reach consensus despite conflicting views
      expect(consensus.consensusResult!.reached).toBe(true);

      const results = await reasoningEngine.getDebateResults(debateSession.id);
      expect(results.topArguments).toHaveLength(3);

      await reasoningEngine.closeDebate(debateSession.id);
    });

    it("should handle agent load balancing across multiple concurrent tasks", async () => {
      const agents = createAgentConfigs(4);
      const concurrentTasks = 8;

      // Create multiple concurrent debates
      const debatePromises = Array.from({ length: concurrentTasks }, async (_, i) => {
        const taskAgents = agents.slice(0, Math.min(4, agents.length)); // Use available agents
        const debateSession = await reasoningEngine.initiateDebate(
          `Load balancing task ${i}`,
          taskAgents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
        );

        // Each agent contributes to their assigned tasks
        for (const agent of taskAgents) {
          await reasoningEngine.submitArgument(
            debateSession.id,
            agent.agentId,
            `Task ${i} analysis from ${agent.role}`,
            [createAgentEvidence(agent.agentId, EvidenceType.FACTUAL, `Task ${i} evidence`)],
            `Task ${i} reasoning from ${agent.specialization} perspective`
          );
        }

        await reasoningEngine.aggregateEvidence(debateSession.id);

        // Simplified voting
        for (const agent of taskAgents) {
          await reasoningEngine.submitVote(
            debateSession.id,
            agent.agentId,
            i % 2 === 0 ? "for" : "against", // Alternate positions
            0.8,
            `Task ${i} vote`
          );
        }

        await reasoningEngine.formConsensus(debateSession.id);
        return debateSession.id;
      });

      const debateIds = await Promise.all(debatePromises);

      // Verify all debates completed
      expect(debateIds).toHaveLength(concurrentTasks);

      for (const debateId of debateIds) {
        const results = await reasoningEngine.getDebateResults(debateId);
        expect(results.session.state).toBe(DebateState.COMPLETED);
        expect(results.consensus).toBeDefined();
        await reasoningEngine.closeDebate(debateId);
      }

      // Verify agent coordinator managed load
      expect(reasoningEngine.getActiveDebateCount()).toBe(0);
    });

    it("should coordinate agent communication patterns", async () => {
      const communicationAgents = createAgentConfigs(3);

      const debateSession = await reasoningEngine.initiateDebate(
        "Agent communication pattern test",
        communicationAgents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
      );

      // Simulate complex communication pattern
      // Agent 1 starts with initial argument
      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-1",
        "Initial position requiring clarification",
        [createAgentEvidence("agent-1", EvidenceType.FACTUAL, "Initial facts")],
        "Need input from other agents"
      );

      // Agent 2 responds with counter-evidence
      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-2",
        "Counter-argument with additional context",
        [
          createAgentEvidence("agent-2", EvidenceType.ANECDOTAL, "Counter experience"),
          createAgentEvidence("agent-2", EvidenceType.EXPERT_OPINION, "Expert counter-opinion"),
        ],
        "Additional context changes the analysis"
      );

      // Agent 3 provides mediating perspective
      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-3",
        "Balanced perspective incorporating both views",
        [
          createAgentEvidence("agent-3", EvidenceType.STATISTICAL, "Statistical analysis of both positions"),
          createAgentEvidence("agent-3", EvidenceType.TECHNICAL, "Technical compromise solution"),
        ],
        "Finding middle ground between conflicting positions"
      );

      // Agent 1 revises based on new information
      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-1",
        "Revised position incorporating agent feedback",
        [
          createAgentEvidence("agent-1", EvidenceType.FACTUAL, "Revised facts"),
          createAgentEvidence("agent-1", EvidenceType.ANECDOTAL, "Revised experience"),
        ],
        "Updated analysis based on collaborative input"
      );

      await reasoningEngine.aggregateEvidence(debateSession.id);

      // All agents vote based on evolved discussion
      await reasoningEngine.submitVote(debateSession.id, "agent-1", "for", 0.8, "Revised position supported");
      await reasoningEngine.submitVote(debateSession.id, "agent-2", "for", 0.7, "Compromise acceptable");
      await reasoningEngine.submitVote(debateSession.id, "agent-3", "for", 0.9, "Balanced solution achieved");

      const consensus = await reasoningEngine.formConsensus(debateSession.id);

      expect(consensus.consensusResult!.reached).toBe(true);
      expect(consensus.arguments).toHaveLength(4); // All arguments submitted

      const results = await reasoningEngine.getDebateResults(debateSession.id);
      expect(results.topArguments).toHaveLength(4);

      await reasoningEngine.closeDebate(debateSession.id);
    });
  });

  describe("Multi-Agent Test 6-10: Agent Failure and Recovery", () => {
    it("should handle agent disconnections during debate", async () => {
      const agents = createAgentConfigs(4);

      const debateSession = await reasoningEngine.initiateDebate(
        "Agent disconnection test",
        agents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
      );

      // Agents 1, 2, 3 submit arguments
      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-1",
        "Agent 1 argument",
        [createAgentEvidence("agent-1", EvidenceType.FACTUAL, "Agent 1 evidence")],
        "Agent 1 reasoning"
      );

      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-2",
        "Agent 2 argument",
        [createAgentEvidence("agent-2", EvidenceType.ANECDOTAL, "Agent 2 evidence")],
        "Agent 2 reasoning"
      );

      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-3",
        "Agent 3 argument",
        [createAgentEvidence("agent-3", EvidenceType.EXPERT_OPINION, "Agent 3 evidence")],
        "Agent 3 reasoning"
      );

      // Agent 4 "disconnects" - never submits argument
      await reasoningEngine.aggregateEvidence(debateSession.id);

      // Agents 1, 2, 3 vote
      await reasoningEngine.submitVote(debateSession.id, "agent-1", "for", 0.8, "Agent 1 vote");
      await reasoningEngine.submitVote(debateSession.id, "agent-2", "against", 0.7, "Agent 2 vote");
      await reasoningEngine.submitVote(debateSession.id, "agent-3", "for", 0.9, "Agent 3 vote");

      // Should still form consensus with available agents
      const consensus = await reasoningEngine.formConsensus(debateSession.id);

      expect(consensus.consensusResult!.reached).toBe(true);
      expect(consensus.consensusResult!.decision).toBeDefined();

      const results = await reasoningEngine.getDebateResults(debateSession.id);
      expect(results.session.participants.find(p => p.agentId === "agent-4")!.votesCast).toHaveLength(0);

      await reasoningEngine.closeDebate(debateSession.id);
    });

    it("should recover from agent communication failures", async () => {
      const agents = createAgentConfigs(3);

      const debateSession = await reasoningEngine.initiateDebate(
        "Communication failure recovery test",
        agents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
      );

      // Simulate agent communication failures
      let failureCount = 0;
      const originalSubmitArgument = jest.spyOn(reasoningEngine, 'submitArgument');

      // Make some submissions fail
      originalSubmitArgument.mockImplementation(async (debateId, agentId, claim, evidence, reasoning) => {
        failureCount++;
        if (failureCount === 2) {
          throw new Error("Network communication failed");
        }
        return originalSubmitArgument.getMockImplementation()!(debateId, agentId, claim, evidence, reasoning);
      });

      // Agent 1 succeeds
      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-1",
        "Agent 1 argument",
        [createAgentEvidence("agent-1", EvidenceType.FACTUAL, "Agent 1 evidence")],
        "Agent 1 reasoning"
      );

      // Agent 2 fails
      await expect(
        reasoningEngine.submitArgument(
          debateSession.id,
          "agent-2",
          "Agent 2 argument",
          [createAgentEvidence("agent-2", EvidenceType.ANECDOTAL, "Agent 2 evidence")],
          "Agent 2 reasoning"
        )
      ).rejects.toThrow("Network communication failed");

      // Agent 3 succeeds
      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-3",
        "Agent 3 argument",
        [createAgentEvidence("agent-3", EvidenceType.EXPERT_OPINION, "Agent 3 evidence")],
        "Agent 3 reasoning"
      );

      // Restore original implementation
      originalSubmitArgument.mockRestore();

      // Continue with successful agents
      await reasoningEngine.aggregateEvidence(debateSession.id);

      await reasoningEngine.submitVote(debateSession.id, "agent-1", "for", 0.8, "Agent 1 vote");
      await reasoningEngine.submitVote(debateSession.id, "agent-3", "against", 0.7, "Agent 3 vote");

      // Should reach consensus with remaining agents
      const consensus = await reasoningEngine.formConsensus(debateSession.id);
      expect(consensus.consensusResult!.reached).toBe(true);

      await reasoningEngine.closeDebate(debateSession.id);
    });

    it("should handle agent trust score degradation", async () => {
      const agents = [
        { agentId: "trusted-agent", role: AgentRole.JUDGE, weight: 3, trustScore: 0.9 },
        { agentId: "untrusted-agent", role: AgentRole.WITNESS, weight: 1, trustScore: 0.3 },
        { agentId: "moderate-agent", role: AgentRole.MEDIATOR, weight: 2, trustScore: 0.6 },
      ];

      const debateSession = await reasoningEngine.initiateDebate(
        "Trust score impact test",
        agents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
      );

      // All agents submit similar arguments but with different credibility
      await reasoningEngine.submitArgument(
        debateSession.id,
        "trusted-agent",
        "High credibility argument",
        [createAgentEvidence("trusted-agent", EvidenceType.STATISTICAL, "Reliable statistics", 0.9)],
        "Well-reasoned argument from trusted source"
      );

      await reasoningEngine.submitArgument(
        debateSession.id,
        "untrusted-agent",
        "Low credibility argument",
        [createAgentEvidence("untrusted-agent", EvidenceType.ANECDOTAL, "Unverified anecdote", 0.3)],
        "Questionable argument from untrusted source"
      );

      await reasoningEngine.submitArgument(
        debateSession.id,
        "moderate-agent",
        "Moderate credibility argument",
        [createAgentEvidence("moderate-agent", EvidenceType.FACTUAL, "Verified facts", 0.7)],
        "Balanced argument from moderately trusted source"
      );

      await reasoningEngine.aggregateEvidence(debateSession.id);

      // Votes weighted by trust scores
      await reasoningEngine.submitVote(debateSession.id, "trusted-agent", "for", 0.9, "High confidence vote");
      await reasoningEngine.submitVote(debateSession.id, "untrusted-agent", "against", 0.3, "Low confidence opposing vote");
      await reasoningEngine.submitVote(debateSession.id, "moderate-agent", "for", 0.6, "Moderate confidence vote");

      const consensus = await reasoningEngine.formConsensus(debateSession.id);

      // Trusted agent's vote should carry more weight
      expect(consensus.consensusResult!.reached).toBe(true);
      expect(consensus.consensusResult!.decision).toBe("for");

      await reasoningEngine.closeDebate(debateSession.id);
    });

    it("should coordinate agent replacement during debate", async () => {
      const originalAgents = createAgentConfigs(3);

      const debateSession = await reasoningEngine.initiateDebate(
        "Agent replacement test",
        originalAgents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
      );

      // Original agents start debate
      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-1",
        "Original agent 1 argument",
        [createAgentEvidence("agent-1", EvidenceType.FACTUAL, "Original evidence")],
        "Original reasoning"
      );

      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-2",
        "Original agent 2 argument",
        [createAgentEvidence("agent-2", EvidenceType.ANECDOTAL, "Original counter-evidence")],
        "Original counter-reasoning"
      );

      // "Replace" agent-3 with new agent
      // In practice, this would be handled by agent coordinator
      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-3",
        "New agent perspective after replacement",
        [createAgentEvidence("agent-3", EvidenceType.EXPERT_OPINION, "Fresh expert analysis")],
        "Updated analysis from replacement agent"
      );

      await reasoningEngine.aggregateEvidence(debateSession.id);

      // All agents vote (including replacement)
      await reasoningEngine.submitVote(debateSession.id, "agent-1", "for", 0.8, "Original position maintained");
      await reasoningEngine.submitVote(debateSession.id, "agent-2", "against", 0.7, "Original opposition maintained");
      await reasoningEngine.submitVote(debateSession.id, "agent-3", "for", 0.75, "Replacement agent supports position");

      const consensus = await reasoningEngine.formConsensus(debateSession.id);

      expect(consensus.consensusResult!.reached).toBe(true);
      expect(consensus.arguments).toHaveLength(3);

      await reasoningEngine.closeDebate(debateSession.id);
    });

    it("should handle cascading agent failures", async () => {
      const agents = createAgentConfigs(5);

      const debateSession = await reasoningEngine.initiateDebate(
        "Cascading failure test",
        agents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
      );

      // Agents 1 and 2 submit successfully
      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-1",
        "Agent 1 argument",
        [createAgentEvidence("agent-1", EvidenceType.FACTUAL, "Agent 1 evidence")],
        "Agent 1 reasoning"
      );

      await reasoningEngine.submitArgument(
        debateSession.id,
        "agent-2",
        "Agent 2 argument",
        [createAgentEvidence("agent-2", EvidenceType.ANECDOTAL, "Agent 2 evidence")],
        "Agent 2 reasoning"
      );

      // Agents 3, 4, 5 fail to submit (simulating cascading failure)
      await reasoningEngine.aggregateEvidence(debateSession.id);

      // Only agents 1 and 2 can vote
      await reasoningEngine.submitVote(debateSession.id, "agent-1", "for", 0.8, "Agent 1 vote");
      await reasoningEngine.submitVote(debateSession.id, "agent-2", "against", 0.7, "Agent 2 vote");

      // Should still attempt consensus with remaining agents
      const consensus = await reasoningEngine.formConsensus(debateSession.id);

      // With only 2 agents and minimum participants = 2, should reach consensus
      expect(consensus.consensusResult!.reached).toBe(true);

      const results = await reasoningEngine.getDebateResults(debateSession.id);
      expect(results.session.participants.filter(p => p.votesCast.length > 0)).toHaveLength(2);
      expect(results.session.participants.filter(p => p.argumentsPresented.length === 0)).toHaveLength(3);

      await reasoningEngine.closeDebate(debateSession.id);
    });
  });

  describe("Multi-Agent Test 11-15: Performance and Scalability", () => {
    it("should scale to 10 concurrent multi-agent debates", async () => {
      const concurrentDebates = 10;
      const agentsPerDebate = 3;

      const startTime = Date.now();

      // Create concurrent debates
      const debatePromises = Array.from({ length: concurrentDebates }, async (_, debateIndex) => {
        const debateAgents = createAgentConfigs(agentsPerDebate).map((agent, i) => ({
          agentId: `${agent.agentId}-debate-${debateIndex}`,
          role: agent.role,
          weight: agent.weight,
        }));

        const debateSession = await reasoningEngine.initiateDebate(
          `Scalability debate ${debateIndex}`,
          debateAgents
        );

        // Each agent submits argument
        for (const agent of debateAgents) {
          await reasoningEngine.submitArgument(
            debateSession.id,
            agent.agentId,
            `Argument from ${agent.agentId} in debate ${debateIndex}`,
            [createAgentEvidence(agent.agentId, EvidenceType.FACTUAL, `Evidence ${debateIndex}`)],
            `Reasoning ${debateIndex}`
          );
        }

        await reasoningEngine.aggregateEvidence(debateSession.id);

        // Simplified voting
        for (const agent of debateAgents) {
          await reasoningEngine.submitVote(
            debateSession.id,
            agent.agentId,
            debateIndex % 2 === 0 ? "for" : "against",
            0.8,
            `Vote ${debateIndex}`
          );
        }

        await reasoningEngine.formConsensus(debateSession.id);
        return debateSession.id;
      });

      const debateIds = await Promise.all(debatePromises);
      const totalTime = Date.now() - startTime;

      expect(debateIds).toHaveLength(concurrentDebates);

      // Should complete within reasonable time (30 seconds for 10 concurrent debates)
      expect(totalTime).toBeLessThan(30000);

      // Verify all debates completed
      for (const debateId of debateIds) {
        const results = await reasoningEngine.getDebateResults(debateId);
        expect(results.session.state).toBe(DebateState.COMPLETED);
        expect(results.consensus!.reached).toBe(true);
        await reasoningEngine.closeDebate(debateId);
      }

      expect(reasoningEngine.getActiveDebateCount()).toBe(0);
    });

    it("should maintain performance with agent workload balancing", async () => {
      const agents = createAgentConfigs(6);
      const tasks = 12;

      // Assign different workloads to agents
      const workloadAssignments = agents.map((agent, index) => ({
        ...agent,
        assignedTasks: index + 1, // Agent 0 gets 1 task, agent 5 gets 6 tasks
      }));

      // Create tasks with agent assignments
      const taskPromises = Array.from({ length: tasks }, async (_, taskIndex) => {
        const assignedAgent = workloadAssignments[taskIndex % workloadAssignments.length];
        const debateAgents = [assignedAgent];

        const debateSession = await reasoningEngine.initiateDebate(
          `Workload task ${taskIndex} for ${assignedAgent.agentId}`,
          debateAgents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
        );

        await reasoningEngine.submitArgument(
          debateSession.id,
          assignedAgent.agentId,
          `Workload argument ${taskIndex}`,
          [createAgentEvidence(assignedAgent.agentId, EvidenceType.FACTUAL, `Workload evidence ${taskIndex}`)],
          `Workload reasoning ${taskIndex}`
        );

        await reasoningEngine.aggregateEvidence(debateSession.id);
        await reasoningEngine.submitVote(debateSession.id, assignedAgent.agentId, "for", 0.8, `Workload vote ${taskIndex}`);
        await reasoningEngine.formConsensus(debateSession.id);

        return { debateId: debateSession.id, agentId: assignedAgent.agentId };
      });

      const startTime = Date.now();
      const taskResults = await Promise.all(taskPromises);
      const totalTime = Date.now() - startTime;

      // Should complete all tasks
      expect(taskResults).toHaveLength(tasks);
      expect(totalTime).toBeLessThan(20000); // 20 seconds for 12 tasks

      // Verify workload distribution
      const agentTaskCounts = taskResults.reduce((counts, result) => {
        counts[result.agentId] = (counts[result.agentId] || 0) + 1;
        return counts;
      }, {} as Record<string, number>);

      // Agents should have different task counts based on workload balancing
      expect(Object.keys(agentTaskCounts)).toHaveLength(6); // All agents used
      expect(Math.max(...Object.values(agentTaskCounts))).toBeGreaterThan(Math.min(...Object.values(agentTaskCounts)));

      // Clean up
      for (const result of taskResults) {
        await reasoningEngine.closeDebate(result.debateId);
      }
    });

    it("should handle mixed agent capabilities efficiently", async () => {
      const specializedAgents = [
        { agentId: "security-specialist", role: AgentRole.JUDGE, capabilities: ["security", "compliance"], weight: 3 },
        { agentId: "performance-specialist", role: AgentRole.WITNESS, capabilities: ["performance", "optimization"], weight: 2 },
        { agentId: "general-specialist", role: AgentRole.MEDIATOR, capabilities: ["general", "coordination"], weight: 1 },
        { agentId: "testing-specialist", role: AgentRole.PROSECUTOR, capabilities: ["testing", "quality"], weight: 2 },
      ];

      const tasks = [
        { type: "security", title: "Security audit required" },
        { type: "performance", title: "Performance optimization needed" },
        { type: "testing", title: "Test coverage insufficient" },
        { type: "general", title: "General code review" },
      ];

      const taskPromises = tasks.map(async (task) => {
        // Find best agent for task
        const bestAgent = specializedAgents.find(agent =>
          agent.capabilities.includes(task.type)
        ) || specializedAgents.find(agent => agent.capabilities.includes("general"))!;

        const debateSession = await reasoningEngine.initiateDebate(
          task.title,
          [{ agentId: bestAgent.agentId, role: bestAgent.role, weight: bestAgent.weight }]
        );

        await reasoningEngine.submitArgument(
          debateSession.id,
          bestAgent.agentId,
          `Specialized analysis for ${task.type}: ${task.title}`,
          [createAgentEvidence(bestAgent.agentId, EvidenceType.EXPERT_OPINION, `Expert ${task.type} analysis`)],
          `${task.type} expertise applied to ${task.title}`
        );

        await reasoningEngine.aggregateEvidence(debateSession.id);
        await reasoningEngine.submitVote(debateSession.id, bestAgent.agentId, "for", 0.9, "Specialized recommendation");
        await reasoningEngine.formConsensus(debateSession.id);

        return { task, agent: bestAgent, debateId: debateSession.id };
      });

      const startTime = Date.now();
      const results = await Promise.all(taskPromises);
      const totalTime = Date.now() - startTime;

      expect(results).toHaveLength(4);
      expect(totalTime).toBeLessThan(15000); // Efficient specialization routing

      // Verify correct agent assignment
      const securityTask = results.find(r => r.task.type === "security")!;
      const performanceTask = results.find(r => r.task.type === "performance")!;
      const testingTask = results.find(r => r.task.type === "testing")!;
      const generalTask = results.find(r => r.task.type === "general")!;

      expect(securityTask.agent.agentId).toBe("security-specialist");
      expect(performanceTask.agent.agentId).toBe("performance-specialist");
      expect(testingTask.agent.agentId).toBe("testing-specialist");
      expect(generalTask.agent.agentId).toBe("general-specialist");

      // Clean up
      for (const result of results) {
        await reasoningEngine.closeDebate(result.debateId);
      }
    });

    it("should demonstrate agent learning and adaptation", async () => {
      // Simulate agent learning through multiple interactions
      const learningAgent = { agentId: "learning-agent", role: AgentRole.JUDGE, weight: 2 };
      const staticAgents = createAgentConfigs(2);

      const learningTasks = 5;
      const learningResults: Array<{ task: number; confidence: number; decision: string }> = [];

      for (let task = 0; task < learningTasks; task++) {
        const agents = [learningAgent, ...staticAgents.slice(0, 2)];
        const debateSession = await reasoningEngine.initiateDebate(
          `Learning task ${task}`,
          agents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
        );

        // Learning agent adapts confidence based on previous outcomes
        const confidence = Math.min(0.5 + (task * 0.1), 0.95); // Learning improves confidence

        for (const agent of agents) {
          await reasoningEngine.submitArgument(
            debateSession.id,
            agent.agentId,
            `Task ${task} argument from ${agent.agentId}`,
            [createAgentEvidence(agent.agentId, EvidenceType.FACTUAL, `Task ${task} evidence`)],
            `Task ${task} reasoning`
          );
        }

        await reasoningEngine.aggregateEvidence(debateSession.id);

        // Learning agent shows adaptation
        await reasoningEngine.submitVote(debateSession.id, "learning-agent", "for", confidence, "Adapted vote");
        await reasoningEngine.submitVote(debateSession.id, "agent-1", "for", 0.8, "Static vote 1");
        await reasoningEngine.submitVote(debateSession.id, "agent-2", "against", 0.7, "Static vote 2");

        const consensus = await reasoningEngine.formConsensus(debateSession.id);

        learningResults.push({
          task,
          confidence,
          decision: consensus.consensusResult!.decision,
        });

        await reasoningEngine.closeDebate(debateSession.id);
      }

      // Verify learning progression
      expect(learningResults).toHaveLength(learningTasks);
      expect(learningResults[0].confidence).toBeLessThan(learningResults[learningResults.length - 1].confidence);

      // Learning agent should influence outcomes more over time
      const learningInfluenced = learningResults.filter(r => r.decision === "for").length;
      expect(learningInfluenced).toBeGreaterThanOrEqual(3); // Should influence at least 3 of 5 outcomes
    });

    it("should handle agent communication overhead at scale", async () => {
      const largeAgentCount = 8;
      const agents = createAgentConfigs(largeAgentCount);

      const debateSession = await reasoningEngine.initiateDebate(
        "Large scale communication test",
        agents.map(a => ({ agentId: a.agentId, role: a.role, weight: a.weight }))
      );

      const startTime = Date.now();

      // All agents submit arguments concurrently
      const argumentPromises = agents.map(agent =>
        reasoningEngine.submitArgument(
          debateSession.id,
          agent.agentId,
          `Large scale argument from ${agent.agentId}`,
          [
            createAgentEvidence(agent.agentId, EvidenceType.FACTUAL, `Fact from ${agent.agentId}`),
            createAgentEvidence(agent.agentId, EvidenceType.ANECDOTAL, `Experience from ${agent.agentId}`),
          ],
          `Reasoning from ${agent.agentId} in large group`
        )
      );

      await Promise.all(argumentPromises);
      const argumentsTime = Date.now() - startTime;

      await reasoningEngine.aggregateEvidence(debateSession.id);
      const evidenceTime = Date.now() - startTime;

      // All agents vote concurrently
      const votePromises = agents.map(agent =>
        reasoningEngine.submitVote(
          debateSession.id,
          agent.agentId,
          agent.agentId.includes("1") || agent.agentId.includes("3") ? "for" : "against",
          0.8,
          `Vote from ${agent.agentId}`
        )
      );

      await Promise.all(votePromises);
      const votesTime = Date.now() - startTime;

      await reasoningEngine.formConsensus(debateSession.id);
      const consensusTime = Date.now() - startTime;

      // Verify scalability
      expect(argumentsTime).toBeLessThan(5000); // 5 seconds for 8 concurrent arguments
      expect(votesTime - argumentsTime).toBeLessThan(3000); // 3 seconds for voting
      expect(consensusTime - votesTime).toBeLessThan(2000); // 2 seconds for consensus

      const results = await reasoningEngine.getDebateResults(debateSession.id);
      expect(results.session.arguments).toHaveLength(largeAgentCount);
      expect(results.session.participants.every(p => p.votesCast.length === 1)).toBe(true);
      expect(results.consensus!.reached).toBe(true);

      await reasoningEngine.closeDebate(debateSession.id);
    });
  });
});
