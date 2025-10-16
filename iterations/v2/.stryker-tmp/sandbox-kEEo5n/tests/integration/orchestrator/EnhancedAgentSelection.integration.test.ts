/**
 * Enhanced Agent Selection Integration Tests
 *
 * Tests the Arbiter Orchestrator's enhanced agent selection that integrates
 * Workspace State Manager for context-aware decision making.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { jest } from "@jest/globals";
import { ArbiterOrchestrator } from "../../../src/orchestrator/ArbiterOrchestrator.js";
import { WorkspaceStateManager } from "../../../src/workspace/WorkspaceStateManager.js";

// Mock external dependencies
jest.mock("chokidar", () => ({
  watch: jest.fn(() => ({
    on: jest.fn(),
    close: jest.fn().mockReturnValue(Promise.resolve()),
  })),
}));

jest.mock("fs/promises", () => ({
  stat: jest.fn(),
}));

jest.mock("path", () => ({
  resolve: jest.fn((...args: string[]) => args.join("/")),
  relative: jest.fn((from: string, to: string) => to.replace(from + "/", "")),
  extname: jest.fn((path: string) => {
    const match = path.match(/\.[^.]+$/);
    return match ? match[0] : "";
  }),
}));

describe("Enhanced Agent Selection with Workspace Integration", () => {
  let orchestrator: ArbiterOrchestrator;
  let workspaceManager: WorkspaceStateManager;
  let systemHealthMonitor: any;

  // Mock agents with different capabilities and workspace familiarity
  const mockAgents = [
    {
      id: "agent-typescript-expert",
      capabilities: ["typescript", "analysis", "testing"],
      currentLoad: 2,
      maxLoad: 5,
      performance: { quality: 0.9, speed: 0.85, reliability: 0.95 },
    },
    {
      id: "agent-generalist",
      capabilities: ["analysis", "writing", "communication"],
      currentLoad: 1,
      maxLoad: 4,
      performance: { quality: 0.8, speed: 0.9, reliability: 0.85 },
    },
    {
      id: "agent-workspace-newbie",
      capabilities: ["typescript", "analysis", "computation"],
      currentLoad: 0,
      maxLoad: 3,
      performance: { quality: 0.75, speed: 0.95, reliability: 0.8 },
    },
  ];

  // Mock tasks with different requirements
  const mockTasks = {
    typescriptAnalysis: {
      id: "task-ts-analysis",
      type: "analysis",
      requiredCapabilities: ["typescript", "analysis"],
      description: "Analyze TypeScript codebase for performance improvements",
      keywords: ["typescript", "performance", "analysis"],
    },
    generalCommunication: {
      id: "task-communication",
      type: "communication",
      requiredCapabilities: ["writing", "communication"],
      description: "Write comprehensive documentation for the system",
      keywords: ["documentation", "writing", "guide"],
    },
  };

  beforeEach(async () => {
    // Create workspace manager with mock data
    workspaceManager = new WorkspaceStateManager({
      workspaceRoot: "/workspace",
      watcher: {
        watchPaths: ["src"],
        ignorePatterns: ["**/node_modules/**"],
        debounceMs: 100,
        recursive: true,
        followSymlinks: false,
        maxFileSize: 1024 * 1024,
        detectBinaryFiles: true,
      },
      defaultContextCriteria: {
        maxFiles: 10,
        maxSizeBytes: 1024 * 1024,
        priorityExtensions: [".ts", ".js"],
        excludeExtensions: [".log"],
        excludeDirectories: ["node_modules"],
        includeBinaryFiles: false,
        relevanceKeywords: [],
        recencyWeight: 0.3,
      },
      snapshotRetentionDays: 30,
      enablePersistence: false,
      compressionLevel: 6,
    });

    // Create system health monitor
    const { SystemHealthMonitor } = await import(
      "../../../src/monitoring/SystemHealthMonitor.js"
    );
    systemHealthMonitor = new SystemHealthMonitor();

    // Mock workspace scanning
    const mockFiles = [
      {
        path: "/workspace/src/main.ts",
        relativePath: "src/main.ts",
        size: 2048,
        mtime: new Date("2024-01-01T10:00:00Z"),
        mode: 0o644,
        isBinary: false,
        extension: ".ts",
        mimeType: "application/typescript",
      },
      {
        path: "/workspace/src/utils.ts",
        relativePath: "src/utils.ts",
        size: 1024,
        mtime: new Date("2024-01-01T11:00:00Z"),
        mode: 0o644,
        isBinary: false,
        extension: ".ts",
        mimeType: "application/typescript",
      },
    ];

    jest
      .spyOn(workspaceManager as any, "scanWorkspace")
      .mockResolvedValue(mockFiles);

    // Create orchestrator with workspace integration
    orchestrator = new ArbiterOrchestrator(
      {
        taskQueue: {},
        taskAssignment: {},
        agentRegistry: {},
        security: {
          auditLoggingEnabled: false,
          maxAuditEvents: 1000,
          inputSanitizationEnabled: true,
          secureErrorResponsesEnabled: true,
          sessionTimeoutMinutes: 60,
        },
        healthMonitor: {},
        recoveryManager: {},
        knowledgeSeeker: {},
        prompting: {} as any,
        database: {
          host: "localhost",
          port: 5432,
          database: "test",
          user: "test",
        },
      },
      workspaceManager,
      systemHealthMonitor
    );

    // Initialize workspace manager first
    await workspaceManager.initialize();

    await orchestrator.initialize();
  });

  afterEach(async () => {
    if (orchestrator) {
      await orchestrator.shutdown();
    }
    jest.clearAllMocks();
  });

  describe("Workspace-Aware Agent Selection", () => {
    it("should prefer agents with workspace familiarity for relevant tasks", async () => {
      // Mock agent workspace activity - typescript expert has high activity
      jest
        .spyOn(orchestrator as any, "getAgentWorkspaceActivity")
        .mockImplementation((...args: any[]) => {
          const agentId = args[0] as string;
          if (agentId === "agent-typescript-expert") return Promise.resolve(40); // High activity
          if (agentId === "agent-generalist") return Promise.resolve(10); // Low activity
          return Promise.resolve(5); // Very low activity
        });

      // Mock agent familiarity - typescript expert is very familiar
      jest
        .spyOn(orchestrator as any, "calculateAgentFamiliarity")
        .mockImplementation((...args: any[]) => {
          const agentId = args[0] as string;
          if (agentId === "agent-typescript-expert") return 0.8; // High familiarity
          if (agentId === "agent-generalist") return 0.4; // Medium familiarity
          return 0.2; // Low familiarity
        });

      // Test TypeScript analysis task
      const assignment = await (orchestrator as any).selectBestAgent(
        mockTasks.typescriptAnalysis,
        mockAgents
      );

      // Should select the TypeScript expert due to workspace familiarity
      expect(assignment.id).toBe("agent-typescript-expert");
    });

    it("should consider task keywords for workspace relevance", async () => {
      // Mock workspace context generation to return high relevance for TS files
      jest.spyOn(workspaceManager, "generateContext").mockReturnValue({
        files: [
          {
            path: "/workspace/src/main.ts",
            relativePath: "src/main.ts",
            size: 2048,
            mtime: new Date(),
            mode: 0o644,
            isBinary: false,
            extension: ".ts",
            mimeType: "application/typescript",
          },
        ],
        totalSize: 2048,
        criteria: {} as any,
        relevanceScores: new Map([
          ["/workspace/src/main.ts", 0.9], // High relevance for TS files
        ]),
        timestamp: new Date(),
      });

      const assignment = await (orchestrator as any).selectBestAgent(
        mockTasks.typescriptAnalysis,
        mockAgents
      );

      // Should consider workspace relevance in scoring
      expect(assignment).toBeDefined();
    });

    it("should gracefully handle missing workspace data", async () => {
      // Create orchestrator without workspace manager
      const orchestratorWithoutWorkspace = new ArbiterOrchestrator({
        taskQueue: {},
        taskAssignment: {},
        agentRegistry: {},
        security: {
          auditLoggingEnabled: false,
          maxAuditEvents: 1000,
          inputSanitizationEnabled: true,
          secureErrorResponsesEnabled: true,
          sessionTimeoutMinutes: 60,
        },
        healthMonitor: {},
        recoveryManager: {},
        knowledgeSeeker: {},
        prompting: {} as any,
      });

      await orchestratorWithoutWorkspace.initialize();

      const assignment = await (
        orchestratorWithoutWorkspace as any
      ).selectBestAgent(mockTasks.typescriptAnalysis, mockAgents);

      // Should still work with neutral workspace scores
      expect(assignment).toBeDefined();
      expect(mockAgents.map((a) => a.id)).toContain(assignment.id);
    });
  });

  describe("Enhanced Scoring Algorithm", () => {
    it("should calculate comprehensive scores with multiple factors", async () => {
      const task = mockTasks.typescriptAnalysis;
      const agent = mockAgents[0]; // typescript expert

      // Mock the helper methods
      jest
        .spyOn(orchestrator as any, "getAgentWorkspaceActivity")
        .mockResolvedValue(30);
      jest
        .spyOn(orchestrator as any, "calculateAgentFamiliarity")
        .mockReturnValue(0.7);
      jest.spyOn(workspaceManager, "generateContext").mockReturnValue({
        files: [],
        totalSize: 0,
        criteria: {} as any,
        relevanceScores: new Map(),
        timestamp: new Date(),
      });

      const factors = await (orchestrator as any).calculateEnhancedScore(
        task,
        agent
      );

      // Should return all scoring factors
      expect(factors).toHaveProperty("capability");
      expect(factors).toHaveProperty("loadBalancing");
      expect(factors).toHaveProperty("performance");
      expect(factors).toHaveProperty("workspace");
      expect(factors).toHaveProperty("health");
      expect(factors).toHaveProperty("resources");

      // All factors should be numbers between 0 and 1
      Object.values(factors).forEach((score) => {
        expect(typeof score).toBe("number");
        expect(score).toBeGreaterThanOrEqual(0);
        expect(score).toBeLessThanOrEqual(1);
      });
    });

    it("should weight workspace context appropriately", async () => {
      const task = mockTasks.typescriptAnalysis;

      // Mock high workspace relevance for one agent
      jest
        .spyOn(orchestrator as any, "getAgentWorkspaceActivity")
        .mockImplementation((...args: any[]) =>
          Promise.resolve(
            (args[0] as string) === "agent-typescript-expert" ? 50 : 5
          )
        );
      jest
        .spyOn(orchestrator as any, "calculateAgentFamiliarity")
        .mockImplementation((...args: any[]) =>
          (args[0] as string) === "agent-typescript-expert" ? 0.9 : 0.3
        );

      const expertFactors = await (orchestrator as any).calculateEnhancedScore(
        task,
        mockAgents[0] // typescript expert
      );
      const generalistFactors = await (
        orchestrator as any
      ).calculateEnhancedScore(
        task,
        mockAgents[1] // generalist
      );

      // Expert should have higher workspace score
      expect(expertFactors.workspace).toBeGreaterThan(
        generalistFactors.workspace
      );
    });
  });

  describe("Task Keyword Extraction", () => {
    it("should extract relevant keywords from tasks", () => {
      const keywords1 = (orchestrator as any).extractTaskKeywords(
        mockTasks.typescriptAnalysis
      );
      const keywords2 = (orchestrator as any).extractTaskKeywords(
        mockTasks.generalCommunication
      );

      expect(keywords1).toContain("analysis");
      expect(keywords1).toContain("typescript");
      expect(keywords1).toContain("performance");

      expect(keywords2).toContain("communication");
      expect(keywords2).toContain("documentation");
      expect(keywords2).toContain("writing");

      // Both should contain common keywords
      expect(keywords1).toContain("src");
      expect(keywords1).toContain("test");
      expect(keywords2).toContain("src");
      expect(keywords2).toContain("test");
    });

    it("should deduplicate keywords", () => {
      const taskWithDuplicates = {
        id: "test-task",
        type: "analysis",
        description: "analysis analysis analysis", // Repeated words
        keywords: ["test", "test", "analysis"], // Duplicate keywords
      };

      const keywords = (orchestrator as any).extractTaskKeywords(
        taskWithDuplicates
      );

      // Should not have duplicates
      const uniqueKeywords = [...new Set(keywords)];
      expect(keywords.length).toBe(uniqueKeywords.length);
    });
  });

  describe("Integration with Task Assignment", () => {
    it("should use enhanced selection in task assignment workflow", async () => {
      // Mock the agent finding and assignment methods
      jest
        .spyOn(orchestrator as any, "findAvailableAgents")
        .mockResolvedValue(mockAgents);
      jest
        .spyOn(orchestrator as any, "createTaskAssignment")
        .mockImplementation(async (task: any, agent: any) => ({
          id: "assignment-123",
          taskId: task.id,
          agentId: agent.id,
        }));
      jest
        .spyOn(orchestrator as any, "checkAssignmentCompliance")
        .mockResolvedValue({ compliant: true });

      // Mock workspace factors to favor the typescript expert
      jest
        .spyOn(orchestrator as any, "getAgentWorkspaceActivity")
        .mockImplementation((...args: any[]) => {
          const agentId = args[0] as string;
          return Promise.resolve(
            agentId === "agent-typescript-expert" ? 50 : 5
          );
        });
      jest
        .spyOn(orchestrator as any, "calculateAgentFamiliarity")
        .mockImplementation((...args: any[]) => {
          const agentId = args[0] as string;
          return agentId === "agent-typescript-expert" ? 0.9 : 0.3;
        });

      const result = await orchestrator["assignTaskToAgent"](
        mockTasks.typescriptAnalysis
      );

      // Should have created an assignment
      expect(result).toBeDefined();
      expect(result.taskId).toBe("task-ts-analysis");
    });

    it("should log agent selection details for debugging", async () => {
      const consoleSpy = jest
        .spyOn(console, "log")
        .mockImplementation(() => {});

      await (orchestrator as any).selectBestAgent(
        mockTasks.typescriptAnalysis,
        mockAgents.slice(0, 2) // Just first two agents
      );

      // Should log selection details
      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining("Agent selection for task")
      );

      consoleSpy.mockRestore();
    });
  });
});
