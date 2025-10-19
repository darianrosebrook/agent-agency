import type { AgentProfile } from "../../types/agent-registry";

/**
 * Seed dataset used by the Arbiter runtime when wiring the real
 * AgentRegistryManager. The profiles mirror the production agents
 * that cover documentation, file_editing, testing, and research
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
      taskTypes: ["documentation", "file_editing", "testing", "script", "analysis"],
      languages: ["TypeScript", "JavaScript"],
      specializations: ["Frontend architecture", "API design"], // Legacy support
      specializationsV2: [
        {
          type: "Frontend architecture",
          level: "expert",
          successRate: 0.94,
          taskCount: 45,
          averageQuality: 0.91,
          lastUsed: "2025-10-15T10:30:00Z",
        },
        {
          type: "API design",
          level: "intermediate",
          successRate: 0.88,
          taskCount: 28,
          averageQuality: 0.85,
          lastUsed: "2025-10-14T15:20:00Z",
        },
      ],
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
      taskTypes: ["file_editing", "refactoring", "code-review", "script", "analysis"],
      languages: ["TypeScript", "Python", "Go"],
      specializations: ["Performance optimization", "Backend architecture"], // Legacy support
      specializationsV2: [
        {
          type: "Performance optimization",
          level: "expert",
          successRate: 0.92,
          taskCount: 38,
          averageQuality: 0.89,
          lastUsed: "2025-10-15T08:15:00Z",
        },
        {
          type: "Backend architecture",
          level: "expert",
          successRate: 0.95,
          taskCount: 42,
          averageQuality: 0.93,
          lastUsed: "2025-10-15T12:45:00Z",
        },
      ],
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
      taskTypes: [
        "testing",
        "code-review",
        "documentation",
        "file_editing",
        "script",
        "analysis",
      ] as const,
      languages: ["TypeScript", "JavaScript"],
      specializations: ["API design", "DevOps"], // Legacy support
      specializationsV2: [
        {
          type: "API design",
          level: "intermediate",
          successRate: 0.91,
          taskCount: 31,
          averageQuality: 0.87,
          lastUsed: "2025-10-14T16:30:00Z",
        },
        {
          type: "DevOps",
          level: "novice",
          successRate: 0.82,
          taskCount: 12,
          averageQuality: 0.78,
          lastUsed: "2025-10-13T09:15:00Z",
        },
      ],
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
      taskTypes: [
        "research",
        "api-design",
        "documentation",
        "script-execution",
        "file_editing",
        "script",
        "analysis",
      ] as const,
      languages: ["Python", "TypeScript"],
      specializations: ["Security audit", "Database design"], // Legacy support
      specializationsV2: [
        {
          type: "Security audit",
          level: "expert",
          successRate: 0.96,
          taskCount: 29,
          averageQuality: 0.94,
          lastUsed: "2025-10-15T11:00:00Z",
        },
        {
          type: "Database design",
          level: "intermediate",
          successRate: 0.89,
          taskCount: 18,
          averageQuality: 0.86,
          lastUsed: "2025-10-14T14:20:00Z",
        },
      ],
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
