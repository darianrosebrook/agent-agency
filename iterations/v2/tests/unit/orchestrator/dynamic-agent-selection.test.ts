/**
 * Tests for dynamic agent selection in ArbiterOrchestrator
 */

import {
  ArbiterOrchestrator,
  defaultArbiterOrchestratorConfig,
} from "../../../src/orchestrator/ArbiterOrchestrator";

describe("Dynamic Agent Selection", () => {
  let orchestrator: ArbiterOrchestrator;
  let mockAgentRegistry;

  beforeEach(async () => {
    // Create mock agent registry
    mockAgentRegistry = {
      getAvailableAgents: jest.fn(),
    };

    // Create orchestrator
    orchestrator = new ArbiterOrchestrator(defaultArbiterOrchestratorConfig);

    // Initialize with mock components
    (orchestrator as any).components.taskQueue = { enqueue: jest.fn() };
    (orchestrator as any).components.taskAssignment = { assignTask: jest.fn() };
    (orchestrator as any).components.agentRegistry = mockAgentRegistry;
    (orchestrator as any).components.security = { validateAccess: jest.fn() };
    (orchestrator as any).components.healthMonitor = { checkHealth: jest.fn() };
    (orchestrator as any).components.recoveryManager = {
      handleFailure: jest.fn(),
    };
    (orchestrator as any).components.knowledgeSeeker = {
      searchKnowledge: jest.fn(),
    };

    // Mark as initialized
    (orchestrator as any).initialized = true;
  });

  describe("selectDebateParticipants", () => {
    it("should use real agents from registry when available", async () => {
      const mockAgents = [
        {
          id: "agent-1",
          capabilities: ["analysis", "reasoning"],
          performanceHistory: { averageSuccessRate: 0.9 },
        },
        {
          id: "agent-2",
          capabilities: ["criticism", "evaluation"],
          performanceHistory: { averageSuccessRate: 0.8 },
        },
        {
          id: "agent-3",
          capabilities: ["synthesis", "integration"],
          performanceHistory: { averageSuccessRate: 0.95 },
        },
      ];

      mockAgentRegistry.getAvailableAgents.mockResolvedValue(mockAgents);

      const task = { id: "test-task" };
      const participants = await (orchestrator as any).selectDebateParticipants(
        task
      );

      expect(participants).toHaveLength(3);
      expect(participants[0].agentId).toBe("agent-1");
      expect(participants[1].agentId).toBe("agent-2");
      expect(participants[2].agentId).toBe("agent-3");
      expect(participants[0].role).toBe("ANALYST");
      expect(participants[1].role).toBe("CRITIC");
      expect(participants[2].role).toBe("SYNTHESIZER");
    });

    it("should generate fallback participants when registry is empty", async () => {
      mockAgentRegistry.getAvailableAgents.mockResolvedValue([]);

      const task = { id: "test-task" };
      const participants = await (orchestrator as any).selectDebateParticipants(
        task
      );

      expect(participants).toHaveLength(3);
      expect(participants[0].agentId).toMatch(/^agent-analyzer-test-task-\d+$/);
      expect(participants[1].agentId).toMatch(/^agent-critic-test-task-\d+$/);
      expect(participants[2].agentId).toMatch(
        /^agent-synthesizer-test-task-\d+$/
      );
    });

    it("should generate fallback participants when registry query fails", async () => {
      mockAgentRegistry.getAvailableAgents.mockRejectedValue(
        new Error("Registry unavailable")
      );

      const task = { id: "test-task" };
      const participants = await (orchestrator as any).selectDebateParticipants(
        task
      );

      expect(participants).toHaveLength(3);
      expect(participants[0].agentId).toMatch(/^agent-analyzer-test-task-\d+$/);
      expect(participants[1].agentId).toMatch(/^agent-critic-test-task-\d+$/);
      expect(participants[2].agentId).toMatch(
        /^agent-synthesizer-test-task-\d+$/
      );
    });

    it("should select agents with best performance scores", async () => {
      const mockAgents = [
        {
          id: "agent-low-performance",
          capabilities: ["analysis", "reasoning"],
          performanceHistory: { averageSuccessRate: 0.3 },
        },
        {
          id: "agent-high-performance",
          capabilities: ["analysis", "reasoning", "data_processing"],
          performanceHistory: { averageSuccessRate: 0.9 },
        },
      ];

      mockAgentRegistry.getAvailableAgents.mockResolvedValue(mockAgents);

      const task = { id: "test-task" };
      const participants = await (orchestrator as any).selectDebateParticipants(
        task
      );

      expect(participants[0].agentId).toBe("agent-high-performance");
    });

    it("should handle missing performance history gracefully", async () => {
      const mockAgents = [
        {
          id: "agent-no-history",
          capabilities: ["analysis", "reasoning"],
        },
      ];

      mockAgentRegistry.getAvailableAgents.mockResolvedValue(mockAgents);

      const task = { id: "test-task" };
      const participants = await (orchestrator as any).selectDebateParticipants(
        task
      );

      expect(participants).toHaveLength(3);
      expect(participants[0].agentId).toBe("agent-no-history");
    });
  });

  describe("generateFallbackParticipants", () => {
    it("should generate unique agent IDs with task and timestamp", () => {
      const task = { id: "unique-task" };
      const participants = (orchestrator as any).generateFallbackParticipants(
        task
      );

      expect(participants).toHaveLength(3);
      expect(participants[0].agentId).toMatch(
        /^agent-analyzer-unique-task-\d+$/
      );
      expect(participants[1].agentId).toMatch(/^agent-critic-unique-task-\d+$/);
      expect(participants[2].agentId).toMatch(
        /^agent-synthesizer-unique-task-\d+$/
      );
    });

    it("should handle missing task ID", () => {
      const participants = (orchestrator as any).generateFallbackParticipants(
        {}
      );

      expect(participants).toHaveLength(3);
      expect(participants[0].agentId).toMatch(/^agent-analyzer-unknown-\d+$/);
    });
  });

  describe("calculateAgentScore", () => {
    it("should calculate score based on capability matching and performance", () => {
      const agent = {
        capabilities: ["analysis", "reasoning", "data_processing"],
        performanceHistory: { averageSuccessRate: 0.8 },
      };
      const requiredCapabilities = ["analysis", "reasoning", "data_processing"];

      const score = (orchestrator as any).calculateAgentScore(
        agent,
        requiredCapabilities
      );

      // Full capability match (1.0) * 0.7 + performance (0.8) * 0.3 = 0.94
      expect(score).toBeCloseTo(0.94, 2);
    });

    it("should handle partial capability matching", () => {
      const agent = {
        capabilities: ["analysis", "reasoning"],
        performanceHistory: { averageSuccessRate: 0.9 },
      };
      const requiredCapabilities = ["analysis", "reasoning", "data_processing"];

      const score = (orchestrator as any).calculateAgentScore(
        agent,
        requiredCapabilities
      );

      // Partial capability match (2/3 = 0.67) * 0.7 + performance (0.9) * 0.3 = 0.739
      expect(score).toBeCloseTo(0.739, 2);
    });

    it("should handle missing capabilities", () => {
      const agent = {
        performanceHistory: { averageSuccessRate: 0.8 },
      };
      const requiredCapabilities = ["analysis", "reasoning"];

      const score = (orchestrator as any).calculateAgentScore(
        agent,
        requiredCapabilities
      );

      expect(score).toBe(0);
    });

    it("should handle missing performance history", () => {
      const agent = {
        capabilities: ["analysis", "reasoning"],
      };
      const requiredCapabilities = ["analysis", "reasoning"];

      const score = (orchestrator as any).calculateAgentScore(
        agent,
        requiredCapabilities
      );

      // Full capability match (1.0) * 0.7 + default performance (0.5) * 0.3 = 0.85
      expect(score).toBeCloseTo(0.85, 2);
    });
  });
});
