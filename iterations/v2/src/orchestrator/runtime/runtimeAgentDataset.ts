import type { AgentProfile } from "../../types/agent-registry";

/**
 * Seed dataset used by the Arbiter runtime when wiring the real
 * AgentRegistryManager. The profiles mirror the production agents
 * that cover documentation, code-editing, testing, and research
 * capabilities so the routing manager can make meaningful decisions.
 */
export interface RuntimeAgentSeed
  extends Pick<
    AgentProfile,
    | "id"
    | "name"
    | "modelFamily"
    | "capabilities"
    | "performanceHistory"
    | "currentLoad"
  > {}

export const runtimeAgentSeeds: RuntimeAgentSeed[] = [
  {
    id: "runtime-docsmith",
    name: "Documentation Smith",
    modelFamily: "gpt-4",
    capabilities: {
      taskTypes: ["documentation", "code-editing", "testing"],
      languages: ["TypeScript", "JavaScript"],
      specializations: ["Frontend architecture", "API design"],
    },
    performanceHistory: {
      successRate: 0.94,
      averageQuality: 0.91,
      averageLatency: 4200,
      taskCount: 72,
    },
    currentLoad: {
      activeTasks: 0,
      queuedTasks: 1,
      utilizationPercent: 8,
    },
  },
  {
    id: "runtime-refactorer",
    name: "Refactor Sage",
    modelFamily: "claude-3.5",
    capabilities: {
      taskTypes: ["code-editing", "refactoring", "code-review"],
      languages: ["TypeScript", "Python", "Go"],
      specializations: ["Performance optimization", "Backend architecture"],
    },
    performanceHistory: {
      successRate: 0.9,
      averageQuality: 0.88,
      averageLatency: 5100,
      taskCount: 65,
    },
    currentLoad: {
      activeTasks: 1,
      queuedTasks: 0,
      utilizationPercent: 18,
    },
  },
  {
    id: "runtime-tester",
    name: "Test Pilot",
    modelFamily: "gemini-pro",
    capabilities: {
      taskTypes: ["testing", "code-review", "documentation"] as const,
      languages: ["TypeScript", "JavaScript"],
      specializations: ["API design", "DevOps"],
    },
    performanceHistory: {
      successRate: 0.92,
      averageQuality: 0.86,
      averageLatency: 4800,
      taskCount: 54,
    },
    currentLoad: {
      activeTasks: 0,
      queuedTasks: 0,
      utilizationPercent: 6,
    },
  },
  {
    id: "runtime-researcher",
    name: "Knowledge Weaver",
    modelFamily: "mixtral",
    capabilities: {
      taskTypes: ["research", "api-design", "documentation"] as const,
      languages: ["Python", "TypeScript"],
      specializations: ["Security audit", "Database design"],
    },
    performanceHistory: {
      successRate: 0.88,
      averageQuality: 0.84,
      averageLatency: 5300,
      taskCount: 41,
    },
    currentLoad: {
      activeTasks: 0,
      queuedTasks: 2,
      utilizationPercent: 14,
    },
  },
];
