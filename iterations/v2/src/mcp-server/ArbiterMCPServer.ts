/**
 * Arbiter MCP Server
 *
 * Model Context Protocol server that exposes Arbiter orchestration tools to AI agents.
 * Provides real-time validation, task assignment, progress monitoring, and verdict generation.
 *
 * @author @darianrosebrook
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  InitializeRequestSchema,
  InitializedNotificationSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { CAWSPolicyAdapter } from "../caws-integration/adapters/CAWSPolicyAdapter.js";
import { CAWSValidationAdapter } from "../caws-integration/adapters/CAWSValidationAdapter.js";
import { ArbiterOrchestrator } from "../orchestrator/ArbiterOrchestrator";
import { TerminalSessionManager } from "../orchestrator/TerminalSessionManager";
import type { WorkingSpec } from "../types/caws-types.js";
import {
  handleTerminalCloseSession,
  handleTerminalCreateSession,
  handleTerminalExecuteCommand,
  handleTerminalGetStatus,
} from "./handlers/terminal-handlers.js";
import {
  TERMINAL_TOOLS,
  type TerminalToolName,
} from "./tools/terminal-tools.js";
import type {
  ArbiterToolName,
  ArbiterValidationResult,
  ArbiterVerdictResult,
  ProgressMonitoringResult,
  TaskAssignmentResult,
} from "./types/mcp-types.js";

/**
 * Arbiter MCP Server
 *
 * Extends MCP Server with arbiter-specific orchestration tools.
 */
export class ArbiterMCPServer extends Server {
  private validationAdapter: CAWSValidationAdapter;
  private policyAdapter: CAWSPolicyAdapter;
  private orchestrator: ArbiterOrchestrator | null = null;
  private terminalManager: TerminalSessionManager;
  private projectRoot: string;
  private version: string = "1.0.0";
  private tools: Array<any> = [];

  /**
   * Create a new Arbiter MCP Server
   *
   * @param projectRoot Project root directory
   * @param orchestrator Optional Arbiter Orchestrator instance for knowledge tools
   */
  constructor(
    projectRoot: string = process.cwd(),
    orchestrator?: ArbiterOrchestrator
  ) {
    super(
      {
        name: "arbiter-mcp-server",
        version: "1.0.0",
      },
      {
        capabilities: {
          tools: {},
          resources: {},
          logging: {},
        },
      }
    );

    this.projectRoot = projectRoot;
    this.orchestrator = orchestrator || null;

    // Initialize adapters
    this.validationAdapter = new CAWSValidationAdapter({
      projectRoot,
    });
    this.policyAdapter = new CAWSPolicyAdapter({
      projectRoot,
    });

    // Initialize terminal session manager
    this.terminalManager = new TerminalSessionManager({
      projectRoot,
      allowedCommandsPath: `${projectRoot}/apps/tools/caws/tools-allow.json`,
    });

    this.policyAdapter = new CAWSPolicyAdapter({
      projectRoot,
      enableCaching: true,
      cacheTTL: 300000, // 5 minutes
    });

    // Initialize tools array with arbiter, terminal, and file editing tools
    this.tools = [...ARBITER_TOOLS, ...TERMINAL_TOOLS, ...FILE_EDITING_TOOLS];

    // Setup will be called after construction

    // Register knowledge tools if orchestrator provided
    if (this.orchestrator) {
      this.registerKnowledgeTools();
    }
  }

  /**
   * Initialize the server (call after construction)
   */
  initialize(): void {
    this.setupToolHandlers();
  }

  /**
   * Set the orchestrator instance (can be called after construction)
   */
  setOrchestrator(orchestrator: ArbiterOrchestrator): void {
    this.orchestrator = orchestrator;
    this.registerKnowledgeTools();
  }

  /**
   * Get the orchestrator instance
   */
  getOrchestrator(): ArbiterOrchestrator {
    if (!this.orchestrator) {
      throw new Error(
        "Orchestrator not initialized. Call setOrchestrator() first."
      );
    }
    return this.orchestrator;
  }

  /**
   * Register knowledge tools with MCP server
   */
  private registerKnowledgeTools(): void {
    if (!this.orchestrator) {
      return;
    }

    try {
      // Add knowledge tools to tools array
      const knowledgeTools = [
        {
          name: "knowledge_search",
          description:
            "Search for information using intelligent research capabilities. " +
            "Queries multiple search providers, processes results for relevance and credibility, " +
            "and returns high-quality research findings.",
          inputSchema: {
            type: "object",
            properties: {
              query: {
                type: "string",
                description: "The search query or question to research",
              },
              queryType: {
                type: "string",
                enum: [
                  "factual",
                  "explanatory",
                  "comparative",
                  "trend",
                  "technical",
                ],
                default: "factual",
              },
              maxResults: {
                type: "number",
                minimum: 1,
                maximum: 20,
                default: 5,
              },
              relevanceThreshold: {
                type: "number",
                minimum: 0,
                maximum: 1,
                default: 0.7,
              },
            },
            required: ["query"],
          },
        },
        {
          name: "knowledge_status",
          description: "Get current status of the Knowledge Seeker system",
          inputSchema: {
            type: "object",
            properties: {},
          },
        },
      ];

      // Add knowledge tools to the tools array (avoiding duplicates)
      for (const tool of knowledgeTools) {
        if (!this.tools.find((t) => t.name === tool.name)) {
          this.tools.push(tool);
        }
      }

      console.error("[Arbiter MCP] Knowledge tools registered successfully");
    } catch (error) {
      console.error("[Arbiter MCP] Failed to register knowledge tools:", error);
    }
  }

  /**
   * Setup MCP request handlers
   */
  public setupToolHandlers(): void {
    // Handle MCP initialization
    this.setRequestHandler(InitializeRequestSchema, async (request) => {
      const { protocolVersion, clientInfo } = request.params;

      console.error(
        `[Arbiter MCP] Initialization: ${clientInfo?.name || "unknown client"}`
      );

      return {
        protocolVersion,
        capabilities: {
          tools: {
            listChanged: false,
          },
          resources: {
            listChanged: false,
          },
          logging: {},
        },
        serverInfo: {
          name: "arbiter-mcp-server",
          version: this.version,
        },
      };
    });

    // Handle client initialized notification
    this.setNotificationHandler(InitializedNotificationSchema, () => {
      console.error("[Arbiter MCP] Client initialized - ready for requests");
    });

    // List available tools
    this.setRequestHandler(ListToolsRequestSchema, () => {
      try {
        return { tools: this.tools };
      } catch (error) {
        console.error("[Arbiter MCP] Error listing tools:", error);
        throw error;
      }
    });

    // Handle tool calls
    this.setRequestHandler(
      CallToolRequestSchema,
      this.handleCallTool.bind(this)
    );
  }

  /**
   * Handle tool calls (internal method)
   */
  async handleCallTool(request: any) {
    const { name, arguments: args } = request.params;

    try {
      const toolArgs = (args || {}) as Record<string, any>;

      switch (
        name as
          | ArbiterToolName
          | "knowledge_search"
          | "knowledge_status"
          | TerminalToolName
          | "file_read"
          | "file_search_replace"
          | "file_write"
          | "run_terminal_cmd"
      ) {
        case "arbiter_validate":
          return await this.handleValidate(toolArgs);
        case "arbiter_assign_task":
          return await this.handleAssignTask(toolArgs);
        case "arbiter_monitor_progress":
          return await this.handleMonitorProgress(toolArgs);
        case "arbiter_generate_verdict":
          return await this.handleGenerateVerdict(toolArgs);
        case "knowledge_search":
          return await this.handleKnowledgeSearch(toolArgs);
        case "knowledge_status":
          return await this.handleKnowledgeStatus();

        // Terminal tools
        case "terminal_create_session":
          return await handleTerminalCreateSession(
            this.terminalManager,
            toolArgs as any // MCPCreateSessionArgs
          );
        case "terminal_execute_command":
          return await handleTerminalExecuteCommand(
            this.terminalManager,
            toolArgs as any // MCPExecuteCommandArgs
          );
        case "terminal_close_session":
          return await handleTerminalCloseSession(
            this.terminalManager,
            toolArgs as any // MCPCloseSessionArgs
          );
        case "terminal_get_status":
          return await handleTerminalGetStatus(
            this.terminalManager,
            toolArgs as any // MCPGetStatusArgs
          );

        // File editing tools
        case "file_read":
          return await this.handleFileRead(toolArgs);
        case "file_search_replace":
          return await this.handleFileSearchReplace(toolArgs);
        case "file_write":
          return await this.handleFileWrite(toolArgs);
        case "run_terminal_cmd":
          return await this.handleTerminalCmd(toolArgs);

        default:
          throw new Error(`Unknown tool: ${name}`);
      }
    } catch (error) {
      console.error(`[Arbiter MCP] Tool error (${name}):`, error);
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                success: false,
                error: error instanceof Error ? error.message : "Unknown error",
                tool: name,
              },
              null,
              2
            ),
          },
        ],
        isError: true,
      } as any;
    }
  }

  /**
   * Handle arbiter_validate tool call
   *
   * Validates a working spec using CAWS CLI integration.
   */
  private async handleValidate(args: Record<string, any>): Promise<{
    content: Array<{ type: string; text: string }>;
    isError?: boolean;
  }> {
    try {
      // If spec provided directly, validate it
      if (args.spec) {
        const result = await this.validationAdapter.validateSpec({
          spec: args.spec,
          projectRoot: args.projectRoot || this.projectRoot,
          options: {
            autoFix: args.autoFix ?? false,
            suggestions: args.suggestions ?? true,
          },
        });

        // Format result for MCP
        const validationResult: ArbiterValidationResult = {
          success: result.success,
          valid: result.success && result.data !== undefined,
          errors: result.success
            ? []
            : [
                {
                  field: "spec",
                  message: result.error?.message || "Validation failed",
                  severity: "error",
                },
              ],
          warnings: [],
          suggestions: args.suggestions ? [] : undefined,
          cawsVersion: "3.4.0",
          durationMs: result.durationMs,
          orchestrationContext: args.orchestrationContext,
        };

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(validationResult, null, 2),
            },
          ],
        };
      }

      // If spec path provided, validate from file
      if (args.specPath) {
        const result = await this.validationAdapter.validateExistingSpec({
          autoFix: args.autoFix ?? false,
          suggestions: args.suggestions ?? true,
        });

        const validationResult: ArbiterValidationResult = {
          success: result.success,
          valid: result.success && result.data !== undefined,
          errors: result.success
            ? []
            : [
                {
                  field: "spec_file",
                  message: result.error?.message || "Validation failed",
                  severity: "error",
                },
              ],
          warnings: [],
          suggestions: args.suggestions ? [] : undefined,
          cawsVersion: "3.4.0",
          durationMs: result.durationMs,
          orchestrationContext: args.orchestrationContext,
        };

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(validationResult, null, 2),
            },
          ],
        };
      }

      throw new Error("Either spec or specPath must be provided");
    } catch (error) {
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                success: false,
                error:
                  error instanceof Error ? error.message : "Validation error",
              },
              null,
              2
            ),
          },
        ],
        isError: true,
      };
    }
  }

  /**
   * Handle arbiter_assign_task tool call
   *
   * Assigns a task to the most appropriate agent based on capabilities and workload.
   */
  private async handleAssignTask(args: Record<string, any>): Promise<{
    content: Array<{ type: string; text: string }>;
    isError?: boolean;
  }> {
    try {
      // For now, implement a simple assignment algorithm
      // In production, this would use the TaskRoutingManager with multi-armed bandit

      const { spec, availableAgents = [], strategy = "capability" } = args;

      // Validate the spec first
      const validation = await this.validationAdapter.validateSpec({
        spec,
        projectRoot: this.projectRoot,
        options: { autoFix: false },
      });

      if (!validation.success) {
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  success: false,
                  error: "Cannot assign invalid task specification",
                  validationError: validation.error?.message,
                },
                null,
                2
              ),
            },
          ],
          isError: true,
        };
      }

      // Derive budget for task complexity estimation
      const budgetResult = await this.policyAdapter.deriveBudget({
        spec,
        projectRoot: this.projectRoot,
        applyWaivers: true,
      });

      // Dynamic agent selection using orchestrator's agent registry
      let selectedAgent: string;

      if (availableAgents && availableAgents.length > 0) {
        // Use provided available agents
        selectedAgent = availableAgents[0];
      } else if (this.orchestrator) {
        try {
          // Query orchestrator's agent registry for available agents
          const registryAgents =
            await this.getAvailableAgentsFromOrchestrator();
          if (registryAgents && registryAgents.length > 0) {
            // Select agent with best performance or most suitable capabilities
            const bestAgent = registryAgents.reduce(
              (best: any, current: any) => {
                const bestScore =
                  best.performanceHistory?.averageSuccessRate || 0.5;
                const currentScore =
                  current.performanceHistory?.averageSuccessRate || 0.5;
                return currentScore > bestScore ? current : best;
              }
            );
            selectedAgent =
              bestAgent.id ||
              bestAgent.agentId ||
              `agent-${Date.now().toString(36)}`;
          } else {
            selectedAgent = `agent-dynamic-${Date.now().toString(36)}`;
          }
        } catch (error) {
          // Fallback to dynamic generation if registry query fails
          selectedAgent = `agent-fallback-${Date.now().toString(36)}`;
        }
      } else {
        // Fallback to dynamic generation
        selectedAgent = `agent-dynamic-${Date.now().toString(36)}`;
      }

      const estimatedHours = this.estimateEffort(
        spec,
        budgetResult.data?.effective
      );

      const assignment: TaskAssignmentResult = {
        success: true,
        agentId: selectedAgent,
        agentName: `Agent ${selectedAgent}`,
        reason: `Selected using ${strategy} strategy`,
        capabilitiesMatched: [spec.mode, `tier-${spec.risk_tier}`],
        estimatedEffort: {
          hours: estimatedHours,
          confidence: 0.7,
        },
        priority: args.priority || "medium",
      };

      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(assignment, null, 2),
          },
        ],
      };
    } catch (error) {
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                success: false,
                error:
                  error instanceof Error ? error.message : "Assignment error",
              },
              null,
              2
            ),
          },
        ],
        isError: true,
      };
    }
  }

  /**
   * Handle arbiter_monitor_progress tool call
   *
   * Monitors task progress, budget usage, and generates alerts.
   */
  private async handleMonitorProgress(args: Record<string, any>): Promise<{
    content: Array<{ type: string; text: string }>;
    isError?: boolean;
  }> {
    try {
      // Read working spec to get acceptance criteria
      const specResult = await this.validationAdapter.validateExistingSpec();

      if (!specResult.success || !specResult.data) {
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  success: false,
                  error: "Cannot monitor progress: working spec not found",
                },
                null,
                2
              ),
            },
          ],
          isError: true,
        };
      }

      // Get current budget usage (placeholder - would read from actual files)
      const currentUsage = {
        files: 15,
        loc: 850,
      };

      // Derive budgets
      const budgetResult = await this.policyAdapter.deriveBudget({
        spec: specResult.data.spec,
        projectRoot: args.projectRoot || this.projectRoot,
        applyWaivers: true,
      });

      const budget = budgetResult.data?.effective || {
        max_files: 100,
        max_loc: 10000,
      };

      // Calculate percentages
      const filesPercentage = (currentUsage.files / budget.max_files) * 100;
      const locPercentage = (currentUsage.loc / budget.max_loc) * 100;

      // Generate alerts based on thresholds
      const alerts: Array<{
        severity: "info" | "warning" | "critical";
        message: string;
        threshold?: number;
      }> = [];

      const warningThreshold = args.thresholds?.warning ?? 0.8;
      const criticalThreshold = args.thresholds?.critical ?? 0.95;

      if (filesPercentage >= criticalThreshold * 100) {
        alerts.push({
          severity: "critical",
          message: `Files budget at ${filesPercentage.toFixed(1)}% (${
            currentUsage.files
          }/${budget.max_files})`,
          threshold: criticalThreshold,
        });
      } else if (filesPercentage >= warningThreshold * 100) {
        alerts.push({
          severity: "warning",
          message: `Files budget at ${filesPercentage.toFixed(1)}% (${
            currentUsage.files
          }/${budget.max_files})`,
          threshold: warningThreshold,
        });
      }

      if (locPercentage >= criticalThreshold * 100) {
        alerts.push({
          severity: "critical",
          message: `LOC budget at ${locPercentage.toFixed(1)}% (${
            currentUsage.loc
          }/${budget.max_loc})`,
          threshold: criticalThreshold,
        });
      } else if (locPercentage >= warningThreshold * 100) {
        alerts.push({
          severity: "warning",
          message: `LOC budget at ${locPercentage.toFixed(1)}% (${
            currentUsage.loc
          }/${budget.max_loc})`,
          threshold: warningThreshold,
        });
      }

      // Mock acceptance criteria progress
      const acceptanceCriteria =
        specResult.data.spec.acceptance?.map((criterion, index) => ({
          id: criterion.id,
          status:
            index === 0
              ? ("completed" as const)
              : index === 1
              ? ("in_progress" as const)
              : ("pending" as const),
          testsWritten: index === 0 ? 5 : index === 1 ? 3 : 0,
          testsPassing: index === 0 ? 5 : index === 1 ? 2 : 0,
          coverage: index === 0 ? 95 : index === 1 ? 70 : 0,
        })) || [];

      const overallProgress = acceptanceCriteria.length
        ? (acceptanceCriteria.filter((c) => c.status === "completed").length /
            acceptanceCriteria.length) *
          100
        : 0;

      const monitoring: ProgressMonitoringResult = {
        taskId: args.taskId,
        status: "in_progress",
        budgetUsage: {
          files: {
            current: currentUsage.files,
            limit: budget.max_files,
            percentage: filesPercentage,
          },
          loc: {
            current: currentUsage.loc,
            limit: budget.max_loc,
            percentage: locPercentage,
          },
        },
        alerts,
        acceptanceCriteria,
        overallProgress,
        timeTracking: {
          started: new Date().toISOString(),
          estimated: new Date(Date.now() + 3600000 * 2).toISOString(), // 2 hours from now
        },
      };

      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(monitoring, null, 2),
          },
        ],
      };
    } catch (error) {
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                success: false,
                error:
                  error instanceof Error ? error.message : "Monitoring error",
              },
              null,
              2
            ),
          },
        ],
        isError: true,
      };
    }
  }

  /**
   * Handle arbiter_generate_verdict tool call
   *
   * Generates final verdict on task completion with quality assessment.
   */
  private async handleGenerateVerdict(args: Record<string, any>): Promise<{
    content: Array<{ type: string; text: string }>;
    isError?: boolean;
  }> {
    try {
      const { taskId, spec, artifacts = {}, qualityGates = [] } = args;

      // Validate final spec
      const validation = await this.validationAdapter.validateSpec({
        spec,
        projectRoot: this.projectRoot,
        options: { autoFix: false, suggestions: true },
      });

      // Check budget compliance
      const budgetResult = await this.policyAdapter.deriveBudget({
        spec,
        projectRoot: this.projectRoot,
        applyWaivers: true,
      });

      const budget = budgetResult.data?.effective || {
        max_files: 100,
        max_loc: 10000,
      };
      const filesChanged = artifacts.filesChanged?.length || 0;
      const locChanged = 0; // Would count from actual files

      const filesWithinBudget = filesChanged <= budget.max_files;
      const locWithinBudget = locChanged <= budget.max_loc;

      // Calculate quality score
      const gatesPassed = qualityGates.filter((g: any) => g.passed).length;
      const gatesTotal = qualityGates.length;
      const gateScore = gatesTotal > 0 ? (gatesPassed / gatesTotal) * 100 : 100;

      const coverageScore = artifacts.coverage || 0;
      const mutationScore = artifacts.mutationScore || 0;

      const qualityScore = Math.round(
        gateScore * 0.4 + coverageScore * 0.3 + mutationScore * 0.3
      );

      // Determine decision
      let decision: "approved" | "rejected" | "conditional" = "approved";
      const recommendations: string[] = [];
      const requiredActions: string[] = [];

      if (!validation.success) {
        decision = "rejected";
        recommendations.push("Fix spec validation errors");
      }

      if (!filesWithinBudget || !locWithinBudget) {
        decision = "conditional";
        requiredActions.push("Reduce scope to meet budget constraints");
      }

      if (qualityScore < 70) {
        decision = decision === "approved" ? "conditional" : decision;
        requiredActions.push(
          "Improve quality score to at least 70 (currently " +
            qualityScore +
            ")"
        );
      }

      if (coverageScore < 80) {
        recommendations.push("Increase test coverage to meet 80% threshold");
      }

      const verdict: ArbiterVerdictResult = {
        decision,
        taskId,
        agentId: args.agentId || "unknown",
        qualityScore,
        qualityGates: {
          total: gatesTotal,
          passed: gatesPassed,
          failed: gatesTotal - gatesPassed,
          details: qualityGates.map((gate: any) => ({
            gate: gate.gate,
            passed: gate.passed,
            score: gate.score,
            message: gate.details || (gate.passed ? "Passed" : "Failed"),
          })),
        },
        budgetCompliance: {
          filesWithinBudget,
          locWithinBudget,
          waiversUsed: budgetResult.data?.waiversApplied || [],
        },
        recommendations:
          recommendations.length > 0 ? recommendations : undefined,
        requiredActions:
          requiredActions.length > 0 ? requiredActions : undefined,
        timestamp: new Date().toISOString(),
      };

      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(verdict, null, 2),
          },
        ],
      };
    } catch (error) {
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                success: false,
                error: error instanceof Error ? error.message : "Verdict error",
              },
              null,
              2
            ),
          },
        ],
        isError: true,
      };
    }
  }

  /**
   * Handle knowledge search tool invocation
   */
  private async handleKnowledgeSearch(args: Record<string, any>): Promise<{
    content: Array<{ type: string; text: string }>;
    isError?: boolean;
  }> {
    try {
      if (!this.orchestrator) {
        throw new Error(
          "Knowledge search requires orchestrator integration. Call setOrchestrator() first."
        );
      }

      const {
        query,
        queryType,
        maxResults,
        relevanceThreshold,
        timeoutMs,
        context,
      } = args;

      if (!query || typeof query !== "string") {
        throw new Error("Query is required and must be a string");
      }

      // Build knowledge query
      const knowledgeQuery = {
        id: `mcp-query-${Date.now()}-${Math.random()
          .toString(36)
          .substring(2, 9)}`,
        query,
        queryType: queryType || "factual",
        maxResults: maxResults || 5,
        relevanceThreshold: relevanceThreshold || 0.7,
        timeoutMs: timeoutMs || 10000,
        context: context || {},
        metadata: {
          requesterId: "mcp-tool",
          priority: 5,
          createdAt: new Date(),
          tags: ["mcp", "knowledge-search"],
        },
      };

      // Execute query through orchestrator
      const response = await this.orchestrator.processKnowledgeQuery(
        knowledgeQuery
      );

      // Format response for MCP
      const result = {
        success: true,
        query: response.query.query,
        summary: response.summary,
        confidence: response.confidence,
        results: response.results.map((r: any) => ({
          title: r.title,
          url: r.url,
          snippet: r.content.substring(0, 200) + "...",
          relevance: r.relevanceScore,
          credibility: r.credibilityScore,
          quality: r.quality,
          domain: r.domain,
          publishedAt: r.publishedAt,
        })),
        sourcesUsed: response.sourcesUsed,
        metadata: {
          totalResults: response.metadata.totalResultsFound,
          filtered: response.metadata.resultsFiltered,
          processingTime: response.metadata.processingTimeMs,
          cached: response.metadata.cacheUsed,
          providers: response.metadata.providersQueried,
        },
      };

      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
      };
    } catch (error) {
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                success: false,
                error:
                  error instanceof Error
                    ? error.message
                    : "Knowledge search error",
              },
              null,
              2
            ),
          },
        ],
        isError: true,
      };
    }
  }

  /**
   * Handle knowledge status tool invocation
   */
  private async handleKnowledgeStatus(): Promise<{
    content: Array<{ type: string; text: string }>;
    isError?: boolean;
  }> {
    try {
      if (!this.orchestrator) {
        throw new Error(
          "Knowledge status requires orchestrator integration. Call setOrchestrator() first."
        );
      }

      const status = await this.orchestrator.getKnowledgeStatus();

      const result = {
        success: true,
        enabled: status.enabled,
        providers: status.providers.map((p: any) => ({
          name: p.name,
          available: p.available,
          health: {
            responseTime: p.health.responseTimeMs,
            errorRate: p.health.errorRate,
            requestsThisMinute: p.health.requestsThisMinute,
          },
        })),
        cache: {
          queryCache: status.cacheStats.queryCacheSize,
          resultCache: status.cacheStats.resultCacheSize,
          hitRate: status.cacheStats.hitRate,
        },
        processing: {
          activeQueries: status.processingStats.activeQueries,
          queuedQueries: status.processingStats.queuedQueries,
        },
      };

      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
      };
    } catch (error) {
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                success: false,
                error:
                  error instanceof Error
                    ? error.message
                    : "Knowledge status error",
              },
              null,
              2
            ),
          },
        ],
        isError: true,
      };
    }
  }

  /**
   * Get available agents from orchestrator's agent registry
   */
  private async getAvailableAgentsFromOrchestrator(): Promise<any[]> {
    try {
      // Access the orchestrator's agent registry through a public method
      if (
        this.orchestrator &&
        typeof (this.orchestrator as any).getAvailableAgents === "function"
      ) {
        return await (this.orchestrator as any).getAvailableAgents();
      }
      return [];
    } catch (error) {
      console.warn("Failed to get available agents from orchestrator", error);
      return [];
    }
  }

  /**
   * Estimate effort for a task
   *
   * @param spec Working spec
   * @param budget Budget limits
   * @returns Estimated hours
   */
  private estimateEffort(
    spec: WorkingSpec,
    budget?: { max_files: number; max_loc: number }
  ): number {
    // Simple estimation based on spec complexity
    let hours = 2; // Base hours

    // Add hours based on acceptance criteria
    hours += spec.acceptance.length * 0.5;

    // Add hours based on risk tier
    hours += spec.risk_tier === 1 ? 4 : spec.risk_tier === 2 ? 2 : 1;

    // Add hours based on budget
    if (budget) {
      hours += Math.log2(budget.max_files / 10);
    }

    return Math.round(hours * 10) / 10; // Round to 1 decimal
  }

  /**
   * Handle file_read tool call
   */
  private async handleFileRead(args: Record<string, any>): Promise<{
    content: Array<{ type: string; text: string }>;
    isError?: boolean;
  }> {
    try {
      const fs = await import("fs/promises");
      const path = await import("path");

      const targetFile = args.target_file;
      const offset = args.offset;
      const limit = args.limit;

      if (!targetFile) {
        throw new Error("target_file is required");
      }

      const fullPath = path.resolve(this.projectRoot, targetFile);
      let content = await fs.readFile(fullPath, "utf-8");

      if (offset !== undefined || limit !== undefined) {
        const lines = content.split("\n");
        const start = offset ? offset - 1 : 0;
        const end = limit ? start + limit : lines.length;
        content = lines.slice(start, end).join("\n");
      }

      return {
        content: [
          {
            type: "text",
            text: content,
          },
        ],
      };
    } catch (error) {
      console.error("[Arbiter MCP] File read error:", error);
      return {
        content: [
          {
            type: "text",
            text: `Error reading file: ${
              error instanceof Error ? error.message : "Unknown error"
            }`,
          },
        ],
        isError: true,
      };
    }
  }

  /**
   * Handle file_search_replace tool call
   */
  private async handleFileSearchReplace(args: Record<string, any>): Promise<{
    content: Array<{ type: string; text: string }>;
    isError?: boolean;
  }> {
    try {
      const fs = await import("fs/promises");
      const path = await import("path");

      const filePath = args.file_path;
      const oldString = args.old_string;
      const newString = args.new_string;
      const replaceAll = args.replace_all || false;

      if (!filePath || !oldString || newString === undefined) {
        throw new Error("file_path, old_string, and new_string are required");
      }

      const fullPath = path.resolve(this.projectRoot, filePath);
      let content = await fs.readFile(fullPath, "utf-8");

      if (replaceAll) {
        content = content.replaceAll(oldString, newString);
      } else {
        content = content.replace(oldString, newString);
      }

      await fs.writeFile(fullPath, content, "utf-8");

      return {
        content: [
          {
            type: "text",
            text: `Successfully updated file: ${filePath}`,
          },
        ],
      };
    } catch (error) {
      console.error("[Arbiter MCP] File search replace error:", error);
      return {
        content: [
          {
            type: "text",
            text: `Error updating file: ${
              error instanceof Error ? error.message : "Unknown error"
            }`,
          },
        ],
        isError: true,
      };
    }
  }

  /**
   * Handle file_write tool call
   */
  private async handleFileWrite(args: Record<string, any>): Promise<{
    content: Array<{ type: string; text: string }>;
    isError?: boolean;
  }> {
    try {
      const fs = await import("fs/promises");
      const path = await import("path");

      const filePath = args.file_path;
      const contents = args.contents;

      if (!filePath || contents === undefined) {
        throw new Error("file_path and contents are required");
      }

      const fullPath = path.resolve(this.projectRoot, filePath);
      await fs.writeFile(fullPath, contents, "utf-8");

      return {
        content: [
          {
            type: "text",
            text: `Successfully wrote file: ${filePath}`,
          },
        ],
      };
    } catch (error) {
      console.error("[Arbiter MCP] File write error:", error);
      return {
        content: [
          {
            type: "text",
            text: `Error writing file: ${
              error instanceof Error ? error.message : "Unknown error"
            }`,
          },
        ],
        isError: true,
      };
    }
  }

  /**
   * Handle run_terminal_cmd tool call
   */
  private async handleTerminalCmd(args: Record<string, any>): Promise<{
    content: Array<{ type: string; text: string }>;
    isError?: boolean;
  }> {
    try {
      const { exec } = await import("child_process");
      const util = await import("util");
      const execAsync = util.promisify(exec);

      const command = args.command;
      const isBackground = args.is_background || false;

      if (!command) {
        throw new Error("command is required");
      }

      // Basic security checks
      const dangerousPatterns = [
        /rm\s+-rf\s+\//,
        />/,
        /sudo/,
        /chmod\s+777/,
        /dd\s+if=/,
      ];

      if (dangerousPatterns.some((pattern) => pattern.test(command))) {
        throw new Error("Command contains potentially dangerous operations");
      }

      if (isBackground) {
        // Run in background
        const _child = exec(command, { cwd: this.projectRoot });
        return {
          content: [
            {
              type: "text",
              text: `Command started in background: ${command}`,
            },
          ],
        };
      } else {
        // Run synchronously
        const { stdout, stderr } = await execAsync(command, {
          cwd: this.projectRoot,
        });
        return {
          content: [
            {
              type: "text",
              text: `Command: ${command}\nOutput:\n${stdout}${
                stderr ? `\nErrors:\n${stderr}` : ""
              }`,
            },
          ],
        };
      }
    } catch (error) {
      console.error("[Arbiter MCP] Terminal command error:", error);
      return {
        content: [
          {
            type: "text",
            text: `Error executing command: ${
              error instanceof Error ? error.message : "Unknown error"
            }`,
          },
        ],
        isError: true,
      };
    }
  }
}

/**
 * Arbiter MCP tools definitions
 */
const ARBITER_TOOLS = [
  {
    name: "arbiter_validate",
    description:
      "Validate a working spec using CAWS CLI integration with orchestration context",
    inputSchema: {
      type: "object",
      properties: {
        spec: {
          type: "object",
          description: "Working spec object to validate",
        },
        specPath: {
          type: "string",
          description: "Path to working spec file",
        },
        projectRoot: {
          type: "string",
          description: "Project root directory",
        },
        autoFix: {
          type: "boolean",
          description: "Enable automatic fixes",
          default: false,
        },
        suggestions: {
          type: "boolean",
          description: "Include improvement suggestions",
          default: true,
        },
        orchestrationContext: {
          type: "object",
          description: "Orchestration metadata",
          properties: {
            taskId: { type: "string" },
            agentId: { type: "string" },
            timestamp: { type: "string" },
          },
        },
      },
    },
  },
  {
    name: "arbiter_assign_task",
    description:
      "Assign a task to the most appropriate agent based on capabilities and workload",
    inputSchema: {
      type: "object",
      properties: {
        spec: {
          type: "object",
          description: "Working spec for the task",
        },
        availableAgents: {
          type: "array",
          items: { type: "string" },
          description: "List of available agent IDs",
        },
        strategy: {
          type: "string",
          enum: ["capability", "performance", "round-robin", "least-loaded"],
          description: "Agent selection strategy",
          default: "capability",
        },
        priority: {
          type: "string",
          enum: ["low", "medium", "high", "critical"],
          description: "Task priority level",
          default: "medium",
        },
      },
      required: ["spec"],
    },
  },
  {
    name: "arbiter_monitor_progress",
    description:
      "Monitor task progress including budget usage, alerts, and acceptance criteria",
    inputSchema: {
      type: "object",
      properties: {
        taskId: {
          type: "string",
          description: "Task ID to monitor",
        },
        projectRoot: {
          type: "string",
          description: "Project root directory",
        },
        detailed: {
          type: "boolean",
          description: "Include detailed metrics",
          default: false,
        },
        thresholds: {
          type: "object",
          description: "Alert thresholds",
          properties: {
            warning: {
              type: "number",
              description: "Warning threshold (0-1)",
              default: 0.8,
            },
            critical: {
              type: "number",
              description: "Critical threshold (0-1)",
              default: 0.95,
            },
          },
        },
      },
      required: ["taskId"],
    },
  },
  {
    name: "arbiter_generate_verdict",
    description:
      "Generate final verdict on task completion with quality assessment and recommendations",
    inputSchema: {
      type: "object",
      properties: {
        taskId: {
          type: "string",
          description: "Task ID for verdict",
        },
        spec: {
          type: "object",
          description: "Working spec used for the task",
        },
        artifacts: {
          type: "object",
          description: "Implementation artifacts",
          properties: {
            filesChanged: {
              type: "array",
              items: { type: "string" },
            },
            testsAdded: { type: "number" },
            coverage: { type: "number" },
            mutationScore: { type: "number" },
          },
        },
        qualityGates: {
          type: "array",
          items: {
            type: "object",
            properties: {
              gate: { type: "string" },
              passed: { type: "boolean" },
              score: { type: "number" },
              details: { type: "string" },
            },
          },
        },
        agentId: {
          type: "string",
          description: "Agent ID who completed the task",
        },
      },
      required: ["taskId", "spec"],
    },
  },
];

/**
 * File editing tools for code modification
 */
const FILE_EDITING_TOOLS = [
  {
    name: "file_read",
    description: "Read the contents of a file",
    inputSchema: {
      type: "object",
      properties: {
        target_file: {
          type: "string",
          description: "Path to the file to read, relative to workspace root",
        },
        offset: {
          type: "number",
          description: "Optional line number to start reading from",
        },
        limit: {
          type: "number",
          description: "Optional number of lines to read",
        },
      },
      required: ["target_file"],
    },
  },
  {
    name: "file_search_replace",
    description: "Search and replace text in a file",
    inputSchema: {
      type: "object",
      properties: {
        file_path: {
          type: "string",
          description: "Path to the file to modify, relative to workspace root",
        },
        old_string: {
          type: "string",
          description: "Text to replace",
        },
        new_string: {
          type: "string",
          description: "Replacement text",
        },
        replace_all: {
          type: "boolean",
          description: "Whether to replace all occurrences",
          default: false,
        },
      },
      required: ["file_path", "old_string", "new_string"],
    },
  },
  {
    name: "file_write",
    description: "Write content to a file (overwrites existing file)",
    inputSchema: {
      type: "object",
      properties: {
        file_path: {
          type: "string",
          description: "Path to the file to write, relative to workspace root",
        },
        contents: {
          type: "string",
          description: "Content to write to the file",
        },
      },
      required: ["file_path", "contents"],
    },
  },
  {
    name: "run_terminal_cmd",
    description: "Execute a terminal command",
    inputSchema: {
      type: "object",
      properties: {
        command: {
          type: "string",
          description: "Terminal command to execute",
        },
        is_background: {
          type: "boolean",
          description: "Whether to run in background",
          default: false,
        },
        explanation: {
          type: "string",
          description: "Explanation of why this command is needed",
        },
      },
      required: ["command"],
    },
  },
];

/**
 * Call a tool directly (for testing purposes)
 *
 * @param request Tool call request
 * @returns Tool call response
 */
export async function callTool(
  server: ArbiterMCPServer,
  request: { name: string; arguments?: Record<string, any> }
) {
  try {
    const response = await server.handleCallTool({
      params: {
        name: request.name,
        arguments: request.arguments || {},
      },
    } as any);
    return response;
  } catch (error) {
    console.error("[Arbiter MCP] Tool call error:", error);
    return {
      content: [
        {
          type: "text",
          text: JSON.stringify({
            success: false,
            error: "TOOL_CALL_ERROR",
            message: error instanceof Error ? error.message : "Unknown error",
          }),
        },
      ],
    };
  }
}

/**
 * Main execution function
 *
 * Starts the Arbiter MCP Server with stdio transport.
 */
export async function main(): Promise<void> {
  const projectRoot = process.env.PROJECT_ROOT || process.cwd();
  const server = new ArbiterMCPServer(projectRoot);

  const transport = new StdioServerTransport();
  await server.connect(transport);

  console.error(
    `[Arbiter MCP] Server started (project: ${projectRoot.split("/").pop()})`
  );
}
