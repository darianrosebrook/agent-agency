/**
 * Unit tests for AgentCoordinator
 *
 * Tests agent registration, role assignment, load balancing,
 * and capability matching.
 */

import {
  AgentCapability,
  AgentCoordinator,
  LoadBalancingStrategy,
  RoleAssignmentRequest,
} from "@/reasoning/AgentCoordinator";
import { AgentRole, ReasoningEngineError } from "@/types/reasoning";

describe("AgentCoordinator", () => {
  let coordinator: AgentCoordinator;

  beforeEach(() => {
    coordinator = new AgentCoordinator({
      loadBalancingStrategy: LoadBalancingStrategy.HYBRID,
      maxAgentsPerDebate: 10,
      minAgentsPerDebate: 2,
      enableCapabilityMatching: true,
      expertiseMatchThreshold: 0.5,
    });
  });

  describe("registerAgent", () => {
    it("should register agent with valid capabilities", () => {
      const capability: AgentCapability = {
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: ["testing", "debugging"],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      };

      coordinator.registerAgent(capability);

      const registered = coordinator.getAgentCapability("agent-1");
      expect(registered).toBeDefined();
      expect(registered?.agentId).toBe("agent-1");
      expect(registered?.availabilityScore).toBe(1.0);
    });

    it("should throw error for empty agent ID", () => {
      const capability: AgentCapability = {
        agentId: "",
        roles: [AgentRole.PROPONENT],
        expertise: [],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      };

      expect(() => coordinator.registerAgent(capability)).toThrow(
        ReasoningEngineError
      );
      expect(() => coordinator.registerAgent(capability)).toThrow(
        "Agent ID cannot be empty"
      );
    });

    it("should throw error for agent with no roles", () => {
      const capability: AgentCapability = {
        agentId: "agent-1",
        roles: [],
        expertise: [],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      };

      expect(() => coordinator.registerAgent(capability)).toThrow(
        ReasoningEngineError
      );
      expect(() => coordinator.registerAgent(capability)).toThrow(
        "must have at least one role"
      );
    });

    it("should throw error for invalid maxLoad", () => {
      const capability: AgentCapability = {
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: [],
        currentLoad: 0,
        maxLoad: 0,
        availabilityScore: 1.0,
      };

      expect(() => coordinator.registerAgent(capability)).toThrow(
        ReasoningEngineError
      );
      expect(() => coordinator.registerAgent(capability)).toThrow(
        "maxLoad must be at least 1"
      );
    });

    it("should calculate availability score correctly", () => {
      const capability: AgentCapability = {
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: [],
        currentLoad: 2,
        maxLoad: 5,
        availabilityScore: 0, // Will be recalculated
      };

      coordinator.registerAgent(capability);

      const registered = coordinator.getAgentCapability("agent-1");
      expect(registered?.availabilityScore).toBeCloseTo(0.6, 1); // 1 - (2/5) = 0.6
    });

    it("should allow updating agent capabilities", () => {
      const capability1: AgentCapability = {
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: ["testing"],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      };

      coordinator.registerAgent(capability1);

      const capability2: AgentCapability = {
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        expertise: ["testing", "debugging"],
        currentLoad: 1,
        maxLoad: 10,
        availabilityScore: 1.0,
      };

      coordinator.registerAgent(capability2);

      const registered = coordinator.getAgentCapability("agent-1");
      expect(registered?.roles).toHaveLength(2);
      expect(registered?.expertise).toHaveLength(2);
      expect(registered?.maxLoad).toBe(10);
    });
  });

  describe("unregisterAgent", () => {
    it("should unregister existing agent", () => {
      const capability: AgentCapability = {
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: [],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      };

      coordinator.registerAgent(capability);
      expect(coordinator.getAgentCapability("agent-1")).toBeDefined();

      const result = coordinator.unregisterAgent("agent-1");
      expect(result).toBe(true);
      expect(coordinator.getAgentCapability("agent-1")).toBeUndefined();
    });

    it("should return false for non-existent agent", () => {
      const result = coordinator.unregisterAgent("non-existent");
      expect(result).toBe(false);
    });
  });

  describe("assignRoles - validation", () => {
    it("should throw error for empty roles", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [],
        topic: "Test topic",
      };

      expect(() => coordinator.assignRoles(request)).toThrow(
        ReasoningEngineError
      );
      expect(() => coordinator.assignRoles(request)).toThrow(
        "At least one role is required"
      );
    });

    it("should throw error for too few agents", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT],
        topic: "Test topic",
      };

      expect(() => coordinator.assignRoles(request)).toThrow(
        ReasoningEngineError
      );
      expect(() => coordinator.assignRoles(request)).toThrow(
        "Minimum 2 agents required"
      );
    });

    it("should throw error for too many agents", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: Array(15).fill(AgentRole.PROPONENT),
        topic: "Test topic",
      };

      expect(() => coordinator.assignRoles(request)).toThrow(
        ReasoningEngineError
      );
      expect(() => coordinator.assignRoles(request)).toThrow(
        "Maximum 10 agents allowed"
      );
    });

    it("should throw error when no agents available", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "Test topic",
      };

      expect(() => coordinator.assignRoles(request)).toThrow(
        ReasoningEngineError
      );
      expect(() => coordinator.assignRoles(request)).toThrow(
        "No agents available"
      );
    });
  });

  describe("assignRoles - round robin", () => {
    beforeEach(() => {
      coordinator = new AgentCoordinator({
        loadBalancingStrategy: LoadBalancingStrategy.ROUND_ROBIN,
      });

      // Register 3 agents
      for (let i = 1; i <= 3; i++) {
        coordinator.registerAgent({
          agentId: `agent-${i}`,
          roles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
          expertise: [],
          currentLoad: 0,
          maxLoad: 5,
          availabilityScore: 1.0,
        });
      }
    });

    it("should assign agents in round-robin fashion", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "Test topic",
      };

      const result1 = coordinator.assignRoles(request);
      expect(result1.assignments).toHaveLength(2);

      const request2: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "Test topic 2",
      };

      const result2 = coordinator.assignRoles(request2);
      expect(result2.assignments).toHaveLength(2);

      // Should cycle through agents
      expect(result1.assignments[0].agentId).not.toBe(
        result2.assignments[0].agentId
      );
    });

    it("should throw error when role not available", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.MEDIATOR, AgentRole.OPPONENT],
        topic: "Test topic",
      };

      expect(() => coordinator.assignRoles(request)).toThrow(
        ReasoningEngineError
      );
      expect(() => coordinator.assignRoles(request)).toThrow(
        "No agents available for role"
      );
    });
  });

  describe("assignRoles - least loaded", () => {
    beforeEach(() => {
      coordinator = new AgentCoordinator({
        loadBalancingStrategy: LoadBalancingStrategy.LEAST_LOADED,
      });

      // Register agents with different loads
      coordinator.registerAgent({
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: [],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      });

      coordinator.registerAgent({
        agentId: "agent-2",
        roles: [AgentRole.PROPONENT],
        expertise: [],
        currentLoad: 3,
        maxLoad: 5,
        availabilityScore: 0.4,
      });

      coordinator.registerAgent({
        agentId: "agent-3",
        roles: [AgentRole.OPPONENT],
        expertise: [],
        currentLoad: 1,
        maxLoad: 5,
        availabilityScore: 0.8,
      });
    });

    it("should assign least loaded agent", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "Test topic",
      };

      const result = coordinator.assignRoles(request);
      expect(result.assignments).toHaveLength(2);

      // Should prefer agent-1 (load: 0) over agent-2 (load: 3)
      const proponentAssignment = result.assignments.find(
        (a) => a.role === AgentRole.PROPONENT
      );
      expect(proponentAssignment?.agentId).toBe("agent-1");

      // Should assign agent-3 for opponent role
      const opponentAssignment = result.assignments.find(
        (a) => a.role === AgentRole.OPPONENT
      );
      expect(opponentAssignment?.agentId).toBe("agent-3");
    });
  });

  describe("assignRoles - capability based", () => {
    beforeEach(() => {
      coordinator = new AgentCoordinator({
        loadBalancingStrategy: LoadBalancingStrategy.CAPABILITY_BASED,
      });

      // Register agents with different expertise
      coordinator.registerAgent({
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: ["javascript", "testing"],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      });

      coordinator.registerAgent({
        agentId: "agent-2",
        roles: [AgentRole.PROPONENT],
        expertise: ["python", "debugging"],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      });

      coordinator.registerAgent({
        agentId: "agent-3",
        roles: [AgentRole.OPPONENT],
        expertise: ["security", "testing"],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      });
    });

    it("should assign based on expertise match", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "Testing best practices",
        expertiseKeywords: ["testing", "javascript"],
      };

      const result = coordinator.assignRoles(request);
      expect(result.assignments).toHaveLength(2);

      // Should prefer agent-1 (has both keywords)
      const proponentAssignment = result.assignments.find(
        (a) => a.role === AgentRole.PROPONENT
      );
      expect(proponentAssignment?.agentId).toBe("agent-1");
      expect(proponentAssignment?.matchScore).toBeCloseTo(1.0, 1); // 2/2 match
    });

    it("should handle no expertise keywords", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "General topic",
        expertiseKeywords: [],
      };

      const result = coordinator.assignRoles(request);
      expect(result.assignments).toHaveLength(2);
      expect(result.confidence).toBeCloseTo(1.0, 1); // All agents match equally
    });
  });

  describe("assignRoles - hybrid", () => {
    beforeEach(() => {
      coordinator = new AgentCoordinator({
        loadBalancingStrategy: LoadBalancingStrategy.HYBRID,
      });

      // Register agents with varying loads and expertise
      coordinator.registerAgent({
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: ["testing"],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      });

      coordinator.registerAgent({
        agentId: "agent-2",
        roles: [AgentRole.PROPONENT],
        expertise: ["testing", "debugging"],
        currentLoad: 3,
        maxLoad: 5,
        availabilityScore: 0.4,
      });

      coordinator.registerAgent({
        agentId: "agent-3",
        roles: [AgentRole.OPPONENT],
        expertise: [],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      });
    });

    it("should balance availability and expertise", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "Testing strategies",
        expertiseKeywords: ["testing", "debugging"],
      };

      const result = coordinator.assignRoles(request);
      expect(result.assignments).toHaveLength(2);

      // agent-2 has better expertise but worse availability
      // agent-1 has good expertise and perfect availability
      // Hybrid should consider both factors
      expect(result.confidence).toBeGreaterThan(0.5);
    });
  });

  describe("updateAgentLoad", () => {
    beforeEach(() => {
      coordinator.registerAgent({
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: [],
        currentLoad: 2,
        maxLoad: 5,
        availabilityScore: 0.6,
      });
    });

    it("should increment agent load", () => {
      coordinator.updateAgentLoad("agent-1", "debate-1", true);

      const capability = coordinator.getAgentCapability("agent-1");
      expect(capability?.currentLoad).toBe(3);
      expect(capability?.availabilityScore).toBeCloseTo(0.4, 1); // 1 - (3/5)
    });

    it("should decrement agent load", () => {
      coordinator.updateAgentLoad("agent-1", "debate-1", false);

      const capability = coordinator.getAgentCapability("agent-1");
      expect(capability?.currentLoad).toBe(1);
      expect(capability?.availabilityScore).toBeCloseTo(0.8, 1); // 1 - (1/5)
    });

    it("should not allow negative load", () => {
      coordinator.updateAgentLoad("agent-1", "debate-1", false);
      coordinator.updateAgentLoad("agent-1", "debate-2", false);
      coordinator.updateAgentLoad("agent-1", "debate-3", false);

      const capability = coordinator.getAgentCapability("agent-1");
      expect(capability?.currentLoad).toBe(0); // Not negative
      expect(capability?.availabilityScore).toBe(1.0);
    });

    it("should throw error for non-existent agent", () => {
      expect(() =>
        coordinator.updateAgentLoad("non-existent", "debate-1", true)
      ).toThrow(ReasoningEngineError);
      expect(() =>
        coordinator.updateAgentLoad("non-existent", "debate-1", true)
      ).toThrow("Agent non-existent not found");
    });
  });

  describe("getCoordinationStats", () => {
    it("should return correct stats for empty coordinator", () => {
      const stats = coordinator.getCoordinationStats();

      expect(stats.totalAgents).toBe(0);
      expect(stats.availableAgents).toBe(0);
      expect(stats.averageLoad).toBe(0);
      expect(stats.utilizationRate).toBe(0);
    });

    it("should return correct stats with agents", () => {
      coordinator.registerAgent({
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: [],
        currentLoad: 2,
        maxLoad: 5,
        availabilityScore: 0.6,
      });

      coordinator.registerAgent({
        agentId: "agent-2",
        roles: [AgentRole.OPPONENT],
        expertise: [],
        currentLoad: 3,
        maxLoad: 5,
        availabilityScore: 0.4,
      });

      coordinator.registerAgent({
        agentId: "agent-3",
        roles: [AgentRole.MEDIATOR],
        expertise: [],
        currentLoad: 5,
        maxLoad: 5,
        availabilityScore: 0,
      });

      const stats = coordinator.getCoordinationStats();

      expect(stats.totalAgents).toBe(3);
      expect(stats.availableAgents).toBe(2); // agent-3 is at max load
      expect(stats.averageLoad).toBeCloseTo(3.33, 1); // (2+3+5)/3
      expect(stats.utilizationRate).toBeCloseTo(0.67, 1); // 10/15
    });
  });

  describe("alternative assignments", () => {
    beforeEach(() => {
      coordinator = new AgentCoordinator({
        loadBalancingStrategy: LoadBalancingStrategy.CAPABILITY_BASED,
      });

      // Register multiple agents for each role
      for (let i = 1; i <= 4; i++) {
        coordinator.registerAgent({
          agentId: `agent-${i}`,
          roles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
          expertise: i <= 2 ? ["testing"] : [],
          currentLoad: 0,
          maxLoad: 5,
          availabilityScore: 1.0,
        });
      }
    });

    it("should generate alternative assignments", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "Testing",
        expertiseKeywords: ["testing"],
      };

      const result = coordinator.assignRoles(request);

      expect(result.assignments).toHaveLength(2);
      expect(result.alternativeAssignments).toBeDefined();
      expect(result.alternativeAssignments!.length).toBeGreaterThan(0);
    });

    it("should not include primary assignments in alternatives", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "Testing",
        expertiseKeywords: ["testing"],
      };

      const result = coordinator.assignRoles(request);

      const primaryAgentIds = result.assignments.map((a) => a.agentId);
      const alternativeAgentIds =
        result.alternativeAssignments?.map((a) => a.agentId) ?? [];

      // No overlap between primary and alternative assignments
      for (const altId of alternativeAgentIds) {
        expect(primaryAgentIds).not.toContain(altId);
      }
    });
  });

  describe("confidence calculation", () => {
    beforeEach(() => {
      coordinator = new AgentCoordinator({
        loadBalancingStrategy: LoadBalancingStrategy.CAPABILITY_BASED,
      });

      coordinator.registerAgent({
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: ["testing", "debugging"],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      });

      coordinator.registerAgent({
        agentId: "agent-2",
        roles: [AgentRole.OPPONENT],
        expertise: ["testing"],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      });
    });

    it("should have high confidence with good expertise match", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "Testing",
        expertiseKeywords: ["testing", "debugging"],
      };

      const result = coordinator.assignRoles(request);

      // agent-1 matches 2/2 keywords = 1.0
      // agent-2 matches 1/2 keywords = 0.5
      // Average: (1.0 + 0.5) / 2 = 0.75
      expect(result.confidence).toBeCloseTo(0.75, 1);
    });

    it("should have perfect confidence with no expertise requirements", () => {
      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "General topic",
        expertiseKeywords: [],
      };

      const result = coordinator.assignRoles(request);

      expect(result.confidence).toBe(1.0);
    });
  });

  describe("edge cases", () => {
    it("should handle agent with multiple roles", () => {
      coordinator.registerAgent({
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT, AgentRole.OPPONENT, AgentRole.MEDIATOR],
        expertise: [],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      });

      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "Test",
      };

      const result = coordinator.assignRoles(request);

      expect(result.assignments).toHaveLength(2);
      // Same agent can fulfill multiple roles
    });

    it("should handle whitespace in agent ID", () => {
      const capability: AgentCapability = {
        agentId: "   ",
        roles: [AgentRole.PROPONENT],
        expertise: [],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      };

      expect(() => coordinator.registerAgent(capability)).toThrow(
        ReasoningEngineError
      );
    });

    it("should handle case-insensitive expertise matching", () => {
      coordinator = new AgentCoordinator({
        loadBalancingStrategy: LoadBalancingStrategy.CAPABILITY_BASED,
      });

      coordinator.registerAgent({
        agentId: "agent-1",
        roles: [AgentRole.PROPONENT],
        expertise: ["JavaScript", "Testing"],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      });

      coordinator.registerAgent({
        agentId: "agent-2",
        roles: [AgentRole.OPPONENT],
        expertise: [],
        currentLoad: 0,
        maxLoad: 5,
        availabilityScore: 1.0,
      });

      const request: RoleAssignmentRequest = {
        requiredRoles: [AgentRole.PROPONENT, AgentRole.OPPONENT],
        topic: "Testing",
        expertiseKeywords: ["javascript", "testing"], // lowercase
      };

      const result = coordinator.assignRoles(request);

      const proponent = result.assignments.find(
        (a) => a.role === AgentRole.PROPONENT
      );
      expect(proponent?.matchScore).toBeCloseTo(1.0, 1); // Full match despite case difference
    });
  });
});
