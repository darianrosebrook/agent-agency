/**
 * @fileoverview GPT-5 Prompting Engine Integration Tests
 *
 * Tests the complete prompting engine integration with all GPT-5 prompting techniques
 * including reasoning effort control, agent eagerness management, and structured prompts.
 */

import { PromptingEngine } from "../../../src/orchestrator/prompting/PromptingEngine";
import {
  AgentControlConfig,
  Task,
  TaskContext,
} from "../../../src/types/agent-prompting";

describe("GPT-5 Prompting Engine Integration", () => {
  let promptingEngine: PromptingEngine;
  let testConfig: AgentControlConfig;

  beforeEach(() => {
    // Configure the prompting engine with GPT-5 inspired settings
    testConfig = {
      reasoningEffort: {
        default: "medium",
        complexityMapping: {
          trivial: "low",
          standard: "medium",
          complex: "high",
          expert: "high",
        },
        dynamicAdjustment: true,
      },
      eagerness: {
        default: "balanced",
        taskTypeMapping: {
          analysis: "thorough",
          creation: "balanced",
          modification: "balanced",
          research: "exhaustive",
          planning: "thorough",
          execution: "minimal",
        },
        minimalMaxCalls: 5,
        balancedMaxCalls: 10,
      },
      toolBudget: {
        enabled: true,
        defaultBudgets: {
          analysis: {
            maxCalls: 15,
            usedCalls: 0,
            resetIntervalMs: 3600000,
            lastResetAt: new Date(),
            escalationRules: [],
          },
          creation: {
            maxCalls: 10,
            usedCalls: 0,
            resetIntervalMs: 3600000,
            lastResetAt: new Date(),
            escalationRules: [],
          },
          modification: {
            maxCalls: 8,
            usedCalls: 0,
            resetIntervalMs: 3600000,
            lastResetAt: new Date(),
            escalationRules: [],
          },
          research: {
            maxCalls: 20,
            usedCalls: 0,
            resetIntervalMs: 3600000,
            lastResetAt: new Date(),
            escalationRules: [],
          },
          planning: {
            maxCalls: 12,
            usedCalls: 0,
            resetIntervalMs: 3600000,
            lastResetAt: new Date(),
            escalationRules: [],
          },
          execution: {
            maxCalls: 6,
            usedCalls: 0,
            resetIntervalMs: 3600000,
            lastResetAt: new Date(),
            escalationRules: [],
          },
        },
        globalLimits: {
          maxConcurrentBudgets: 100,
          totalDailyCalls: 1000,
        },
      },
      contextGathering: {
        strategy: "parallel",
        maxParallelQueries: 5,
        earlyStopCriteria: {
          convergenceThreshold: 0.8,
          maxTimeMs: 30000,
          minQualityScore: 0.7,
        },
        depthLimits: {
          veryLow: 2,
          low: 3,
          medium: 5,
          high: 8,
        },
      },
      selfReflection: {
        enabled: true,
        complexityThreshold: "complex",
        defaultRubric: {
          categories: [],
          acceptanceThreshold: 0.8,
          maxIterations: 3,
        },
      },
    };

    promptingEngine = new PromptingEngine({
      enabled: true,
      ...testConfig,
      monitoring: {
        enableMetrics: false,
        enableTracing: false,
        metricsPrefix: "test",
      },
    });
  });

  describe("Reasoning Effort Control", () => {
    it("should select appropriate reasoning effort for trivial tasks", async () => {
      const task: Task = {
        id: "test-trivial",
        complexity: "trivial",
        type: "execution",
        description: "Add two numbers",
      };

      const context: TaskContext = {
        complexity: "trivial",
        type: "execution",
        accuracyRequirement: "draft",
      };

      const result = await promptingEngine.processTask(task, context);

      expect(result.reasoningEffort).toBe("low");
      expect(result.metadata.confidence).toBeGreaterThan(0.7);
    });

    it("should select high reasoning effort for complex expert tasks", async () => {
      const task: Task = {
        id: "test-expert",
        complexity: "expert",
        type: "research",
        description:
          "Design a comprehensive machine learning architecture for real-time fraud detection",
      };

      const context: TaskContext = {
        complexity: "expert",
        type: "research",
        accuracyRequirement: "critical",
        timeBudgetMs: 7200000, // 2 hours
      };

      const result = await promptingEngine.processTask(task, context);

      expect(result.reasoningEffort).toBe("high");
      expect(result.metadata.appliedOptimizations).toContain(
        "self-reflection-enabled"
      );
    });

    it("should adapt reasoning effort based on time pressure", async () => {
      const task: Task = {
        id: "test-time-pressure",
        complexity: "complex",
        type: "analysis",
        description: "Analyze quarterly sales data and identify trends",
      };

      const context: TaskContext = {
        complexity: "complex",
        type: "analysis",
        accuracyRequirement: "high",
        timeBudgetMs: 300000, // 5 minutes - tight deadline
      };

      const result = await promptingEngine.processTask(task, context);

      // Should potentially reduce effort due to time pressure
      expect(["medium", "low"]).toContain(result.reasoningEffort);
    });
  });

  describe("Agent Eagerness Management", () => {
    it("should set minimal eagerness for simple execution tasks", async () => {
      const task: Task = {
        id: "test-minimal",
        complexity: "trivial",
        type: "execution",
        description: "Format a date string",
      };

      const context: TaskContext = {
        complexity: "trivial",
        type: "execution",
        accuracyRequirement: "standard",
      };

      const result = await promptingEngine.processTask(task, context);

      expect(result.eagerness).toBe("minimal");
      expect(result.toolBudget.maxCalls).toBeLessThanOrEqual(5);
    });

    it("should set exhaustive eagerness for research tasks", async () => {
      const task: Task = {
        id: "test-research",
        complexity: "complex",
        type: "research",
        description:
          "Research and compare different authentication protocols for microservices",
      };

      const context: TaskContext = {
        complexity: "complex",
        type: "research",
        accuracyRequirement: "high",
      };

      const result = await promptingEngine.processTask(task, context);

      expect(result.eagerness).toBe("exhaustive");
      expect(result.toolBudget.maxCalls).toBeGreaterThan(15);
    });

    it("should balance thoroughness with efficiency for planning tasks", async () => {
      const task: Task = {
        id: "test-planning",
        complexity: "standard",
        type: "planning",
        description: "Create a project roadmap for the next quarter",
      };

      const context: TaskContext = {
        complexity: "standard",
        type: "planning",
        accuracyRequirement: "high",
        timeBudgetMs: 1800000, // 30 minutes
      };

      const result = await promptingEngine.processTask(task, context);

      expect(result.eagerness).toBe("exhaustive"); // Planning tasks with high accuracy get exhaustive eagerness
      expect(result.toolBudget.maxCalls).toBeGreaterThan(8);
    });
  });

  describe("Tool Budget Management", () => {
    it("should allocate conservative budgets for simple tasks", async () => {
      const task: Task = {
        id: "test-budget-simple",
        complexity: "trivial",
        type: "modification",
        description: "Rename a variable",
      };

      const context: TaskContext = {
        complexity: "trivial",
        type: "modification",
        accuracyRequirement: "standard",
      };

      const result = await promptingEngine.processTask(task, context);

      expect(result.toolBudget.maxCalls).toBeLessThanOrEqual(8);
      expect(result.toolBudget.usedCalls).toBe(0);
    });

    it("should allocate generous budgets for research tasks", async () => {
      const task: Task = {
        id: "test-budget-research",
        complexity: "complex",
        type: "research",
        description:
          "Investigate performance optimization techniques for React applications",
      };

      const context: TaskContext = {
        complexity: "complex",
        type: "research",
        accuracyRequirement: "high",
      };

      const result = await promptingEngine.processTask(task, context);

      expect(result.toolBudget.maxCalls).toBeGreaterThanOrEqual(20);
    });

    it("should include escalation rules for complex tasks", async () => {
      const task: Task = {
        id: "test-escalation",
        complexity: "expert",
        type: "creation",
        description:
          "Build a complete authentication system with multiple providers",
      };

      const context: TaskContext = {
        complexity: "expert",
        type: "creation",
        accuracyRequirement: "critical",
      };

      const result = await promptingEngine.processTask(task, context);

      expect(result.toolBudget.escalationRules).toHaveLength(2); // Expert tasks get 2 escalation rules
      expect(result.toolBudget.escalationRules[0]).toHaveProperty("trigger");
      expect(result.toolBudget.escalationRules[0]).toHaveProperty(
        "additionalCalls"
      );
    });
  });

  describe("XML Structured Prompts", () => {
    it("should process XML structured instructions correctly", async () => {
      const task: Task = {
        id: "test-xml",
        complexity: "standard",
        type: "creation",
        description: "Create a user registration component",
      };

      const context: TaskContext = {
        complexity: "standard",
        type: "creation",
        accuracyRequirement: "high",
      };

      const xmlInstructions = `
<code_editing_rules>
  <guiding_principles>
    - Components should be functional and reusable
    - Use TypeScript for type safety
    - Follow React best practices
  </guiding_principles>
  <frontend_stack_defaults>
    - Styling: TailwindCSS
    - State: React hooks
    - Forms: React Hook Form
  </frontend_stack_defaults>
  <persistence>
    - Continue working through validation errors
    - Ask for clarification only when absolutely necessary
    - Prefer making reasonable assumptions over stopping
  </persistence>
</code_editing_rules>
      `;

      const result = await promptingEngine.processTask(
        task,
        context,
        xmlInstructions
      );

      expect(result.structuredInstructions).toHaveLength(1);
      expect(result.structuredInstructions[0].tag).toBe("code_editing_rules");
      expect(result.structuredInstructions[0].children).toHaveLength(3);
      expect(result.metadata.appliedOptimizations).toContain(
        "xml-instructions-processed"
      );
    });

    it("should handle invalid XML gracefully", async () => {
      const task: Task = {
        id: "test-invalid-xml",
        complexity: "standard",
        type: "analysis",
        description: "Analyze code quality",
      };

      const context: TaskContext = {
        complexity: "standard",
        type: "analysis",
        accuracyRequirement: "standard",
      };

      const invalidXml = "<unclosed><nested>content";
      const result = await promptingEngine.processTask(
        task,
        context,
        invalidXml
      );

      // Should still work with conservative defaults
      expect(result.reasoningEffort).toBeDefined();
      expect(result.eagerness).toBeDefined();
      expect(result.toolBudget).toBeDefined();
    });
  });

  describe("Self-Reflection Rubrics", () => {
    it("should generate self-reflection rubrics for complex tasks", async () => {
      const task: Task = {
        id: "test-reflection",
        complexity: "expert",
        type: "planning",
        description:
          "Design a comprehensive architecture for a distributed task orchestration system",
      };

      const context: TaskContext = {
        complexity: "expert",
        type: "planning",
        accuracyRequirement: "critical",
      };

      const result = await promptingEngine.processTask(task, context);

      expect(result.reflectionRubric).toBeDefined();
      expect(result.reflectionRubric!.categories.length).toBeGreaterThan(3);
      expect(result.reflectionRubric!.acceptanceThreshold).toBeGreaterThan(0.8);
    });

    it("should skip self-reflection for simple tasks", async () => {
      const task: Task = {
        id: "test-no-reflection",
        complexity: "trivial",
        type: "execution",
        description: "Add error handling to a function",
      };

      const context: TaskContext = {
        complexity: "trivial",
        type: "execution",
        accuracyRequirement: "standard",
      };

      const result = await promptingEngine.processTask(task, context);

      expect(result.reflectionRubric).toBeUndefined();
    });
  });

  describe("System Health and Monitoring", () => {
    it("should report healthy status when all components are functional", async () => {
      const status = await promptingEngine.getStatus();

      expect(status.healthy).toBe(true);
      expect(status.components.reasoningController).toBe(true);
      expect(status.components.eagernessManager).toBe(true);
      expect(status.components.budgetManager).toBe(true);
      expect(status.components.contextCoordinator).toBe(true);
      expect(status.components.reflectionManager).toBe(true);
      expect(status.components.xmlProcessor).toBe(true);
    });

    it("should track processing statistics", async () => {
      const task: Task = {
        id: "test-stats",
        complexity: "standard",
        type: "analysis",
        description: "Analyze test coverage",
      };

      const context: TaskContext = {
        complexity: "standard",
        type: "analysis",
        accuracyRequirement: "standard",
      };

      // Process multiple tasks to generate stats
      await promptingEngine.processTask(task, context);
      await promptingEngine.processTask(
        { ...task, id: "test-stats-2" },
        context
      );

      const status = await promptingEngine.getStatus();

      expect(status.activeConfigs.totalProcessed).toBeGreaterThanOrEqual(2);
      expect(status.activeConfigs.currentComplexity).toBeDefined();
      expect(status.activeConfigs.currentEffort).toBeDefined();
    });
  });

  describe("Performance and Optimization", () => {
    it("should process tasks within reasonable time limits", async () => {
      const task: Task = {
        id: "test-performance",
        complexity: "complex",
        type: "creation",
        description: "Create a comprehensive data validation system",
      };

      const context: TaskContext = {
        complexity: "complex",
        type: "creation",
        accuracyRequirement: "high",
      };

      const startTime = Date.now();
      const result = await promptingEngine.processTask(task, context);
      const processingTime = Date.now() - startTime;

      expect(processingTime).toBeLessThan(1000); // Should process in under 1 second
      expect(result.metadata.processingTimeMs).toBeLessThan(1000);
      expect(result.metadata.confidence).toBeGreaterThan(0.5);
    });

    it("should apply appropriate optimizations based on task characteristics", async () => {
      const task: Task = {
        id: "test-optimization",
        complexity: "trivial",
        type: "execution",
        description: "Quick fix: remove unused import",
      };

      const context: TaskContext = {
        complexity: "trivial",
        type: "execution",
        accuracyRequirement: "draft",
        timeBudgetMs: 60000, // 1 minute
      };

      const result = await promptingEngine.processTask(task, context);

      expect(result.metadata.appliedOptimizations).toContain(
        "low-reasoning-effort"
      );
      expect(result.metadata.appliedOptimizations).toContain(
        "minimal-tool-budget"
      );
    });
  });

  describe("Error Handling and Resilience", () => {
    it("should provide conservative defaults when components fail", async () => {
      // Create a configuration that would cause issues
      const badConfig = {
        enabled: true,
        reasoningEffort: {} as any, // Invalid config
        eagerness: {} as any,
        toolBudget: {} as any,
        contextGathering: {} as any,
        selfReflection: {} as any,
        monitoring: {
          enableMetrics: false,
          enableTracing: false,
          metricsPrefix: "test",
        },
      };

      const badEngine = new PromptingEngine(badConfig);

      const task: Task = {
        id: "test-error-handling",
        complexity: "standard",
        type: "analysis",
        description: "Analyze error handling",
      };

      const context: TaskContext = {
        complexity: "standard",
        type: "analysis",
        accuracyRequirement: "standard",
      };

      const result = await badEngine.processTask(task, context);

      // Should still return valid defaults
      expect(result.reasoningEffort).toBe("medium");
      expect(result.eagerness).toBe("balanced");
      expect(result.toolBudget).toBeDefined();
      expect(result.metadata.appliedOptimizations).toContain(
        "conservative-fallback"
      );
    });
  });
});
