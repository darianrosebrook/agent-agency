import crypto from "crypto";
import fsp from "fs/promises";
import yaml from "js-yaml";
import path from "path";
import { CAWSValidator } from "../../caws-validator/CAWSValidator";
import type { CAWSValidationResult } from "../../caws-validator/types/validation-types";
import { getObserverBridge } from "../../observer";
import type { ChainOfThoughtEntry } from "../../observer/types";
import { PerformanceTracker } from "../../rl/PerformanceTracker";
import type { AgentRegistry } from "../../types/agent-registry";
import type { Task as ArbiterTask } from "../../types/arbiter-orchestration";
import type { WorkingSpec } from "../../types/caws-types";
import {
  Task as RunnerTask,
  TaskExecution as RunnerTaskExecution,
  TaskOrchestratorConfig,
  TaskOrchestratorEvents,
  TaskPriority,
  WorkerExecutionResult,
} from "../../types/task-runner";
import { TaskState } from "../../types/task-state";
import type { VerificationResult } from "../../types/verification";
import {
  VerificationEngineConfig,
  VerificationPriority,
  VerificationType,
  VerificationVerdict,
} from "../../types/verification";
import { VerificationEngineImpl } from "../../verification/VerificationEngine";
import type {
  ConversationContext,
  EvidenceManifest,
} from "../../verification/types";
import { EventSeverity, events } from "../EventEmitter";
import { EventTypes } from "../OrchestratorEvents";
import { RegistryProvider } from "../RegistryProvider.js";
import { TaskOrchestrator } from "../TaskOrchestrator";
import { TaskQueue } from "../TaskQueue";
import { TaskRoutingManager } from "../TaskRoutingManager";
import { TaskStateMachine } from "../TaskStateMachine";
import { runtimeAgentSeeds } from "./runtimeAgentDataset.js";

// Temporary inline type until import is fixed
interface ArtifactFileEntry {
  path: string;
  size: number;
  sha256: string;
  mimeType?: string;
  createdAt: string;
}

interface ArtifactManifest {
  taskId: string;
  files: ArtifactFileEntry[];
  totalSize: number;
  createdAt: string;
  completedAt?: string;
}

export interface SubmitTaskOptions {
  description: string;
  specPath?: string;
  metadata?: Record<string, any>;
  // Allow full task specification for script execution or direct task submission
  task?: Partial<ArbiterTask>;
}

export interface RuntimeStatus {
  running: boolean;
  registryReady: boolean;
  queueDepth: number;
  activeTasks: number;
  completedTasks: number;
  failedTasks: number;
}

export interface RuntimeMetrics {
  totalTasks: number;
  completedTasks: number;
  failedTasks: number;
  averageDurationMs: number;
  lastTaskDurationMs?: number;
}

export interface RuntimeTaskSnapshot {
  taskId: string;
  state: TaskState;
  description: string;
  createdAt: Date;
  updatedAt: Date;
  plan: string[];
  nextActions: string[];
  outputPath?: string;
  artifacts?: {
    manifest: ArtifactManifest;
    rootPath: string;
  };
  assignedAgentId?: string;
  metadata?: Record<string, any>;
  cawsResult?: {
    passed: boolean;
    verdict: string;
    remediation?: string[];
  };
  verificationResult?: VerificationResult;
}

interface RuntimeTaskRecord {
  task: ArbiterTask;
  description: string;
  plan: string[];
  nextActions: string[];
  outputPath?: string;
  createdAt: Date;
  updatedAt: Date;
  status: TaskState;
  metadata?: Record<string, any>;
  artifacts?: {
    manifest: ArtifactManifest;
    rootPath: string;
  };
  cawsResult?: {
    passed: boolean;
    verdict: string;
    remediation?: string[];
  };
  verificationResult?: VerificationResult;
}

interface ArbiterRuntimeOptions {
  outputDir: string;
}

// Registry interface is now imported from types

const DEFAULT_ROUTING_CONFIG = {
  enableBandit: false,
  minAgentsRequired: 1,
  maxAgentsToConsider: 3,
  defaultStrategy: "capability-match" as const,
  maxRoutingTimeMs: 50,
  loadBalancingWeight: 0.3,
  capabilityMatchWeight: 0.7,
};

const DEFAULT_ORCHESTRATOR_TIMEOUT_MS = 60_000;

export class NoEligibleAgentsError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "NoEligibleAgentsError";
  }
}

/**
 * Arbiter runtime powered by in-process queueing and routing primitives.
 * Tasks submitted through the observer bridge are queued, routed to mock
 * agents via `TaskRoutingManager`, executed deterministically, and
 * persisted as Markdown artefacts for downstream auditing.
 */
export class ArbiterRuntime {
  private readonly queue = new TaskQueue();
  private readonly stateMachine = new TaskStateMachine();
  private readonly performanceTracker = new PerformanceTracker({
    enabled: true,
    maxEventsInMemory: 10000,
    retentionPeriodMs: 7 * 24 * 60 * 60 * 1000, // 7 days
    batchSize: 100,
    anonymizeData: true,
  });
  private readonly cawsValidator: CAWSValidator;
  private readonly verificationEngine: VerificationEngineImpl;
  private readonly verificationConfig: VerificationEngineConfig;
  private readonly agentRegistry: AgentRegistry;
  private routingManager: TaskRoutingManager;
  private taskOrchestrator: TaskOrchestrator | null = null;
  private workerArtifactsRoot: string | null = null;
  private readonly options: ArbiterRuntimeOptions;
  private readonly taskRecords = new Map<string, RuntimeTaskRecord>();
  private readonly completionResolvers = new Map<
    string,
    { resolve: () => void; reject: (_error: unknown) => void }
  >();

  private running = false;
  private processing = false;
  private registryReady = false;
  private totalTasks = 0;
  private completedTasks = 0;
  private failedTasks = 0;
  private cumulativeDuration = 0;
  private lastDuration?: number;

  constructor(options: ArbiterRuntimeOptions) {
    this.options = options;
    this.cawsValidator = new CAWSValidator({
      performanceTracker: this.performanceTracker,
    });
    this.verificationConfig = this.createVerificationConfig();
    this.verificationEngine = new VerificationEngineImpl(
      this.verificationConfig
    );
    // Registry will be initialized in start() method
    this.agentRegistry = null as any; // Temporary - will be replaced in initializeRegistry()
    this.routingManager = new TaskRoutingManager(
      this.agentRegistry as any,
      DEFAULT_ROUTING_CONFIG
    );
    this.routingManager.setPerformanceTracker(this.performanceTracker);
  }

  async start(): Promise<void> {
    if (this.running) return;
    this.running = true;
    await fsp.mkdir(this.options.outputDir, { recursive: true });

    // Start performance tracking
    await this.performanceTracker.startCollection();

    await this.initializeRegistry();
  }

  async stop(): Promise<void> {
    this.running = false;
    this.processing = false;
    if (this.taskOrchestrator) {
      await this.taskOrchestrator.shutdown();
      this.taskOrchestrator = null;
    }
  }

  /**
   * Initialize the agent registry with seeded agents.
   */
  private async initializeRegistry(): Promise<void> {
    try {
      console.log("Initializing agent registry...");

      // Create registry provider for event handling
      const provider = RegistryProvider.createProvider({
        enableSecurity: false, // Disabled for runtime
        enablePersistence: false, // In-memory only
      });

      // Wait for registry ready event
      const readyPromise = new Promise<void>((resolve) => {
        provider.once("registry.ready", () => {
          console.log("Registry ready event received");
          resolve();
        });

        // Fallback timeout
        setTimeout(() => {
          console.log("Registry ready timeout - proceeding anyway");
          resolve();
        }, 5000);
      });

      // Create real registry with seeded agents
      const realRegistry = await provider.createRegistry({
        seeds: runtimeAgentSeeds,
        mode: "idempotent",
        emitReady: true,
      });

      // Replace the stub with the real registry
      (this as any).agentRegistry = realRegistry;

      if (
        realRegistry &&
        typeof (realRegistry as any).setPerformanceTracker === "function"
      ) {
        (realRegistry as any).setPerformanceTracker(this.performanceTracker);
      }

      // Update the routing manager to use the real registry
      this.routingManager = new TaskRoutingManager(
        realRegistry as any,
        DEFAULT_ROUTING_CONFIG
      );
      this.routingManager.setPerformanceTracker(this.performanceTracker);

      await this.initializeTaskOrchestrator(realRegistry);

      // Wait for ready event
      await readyPromise;

      this.registryReady = true;
      console.log("Agent registry initialized successfully");

      // Record agent registry initialization in Performance Tracker
      // Note: Agent registry doesn't expose getAgents() method, so we'll skip this for now
      // This would need to be implemented when the agent registry API is available
      console.log("Performance Tracker ready for agent registration tracking");
    } catch (error) {
      console.error("Failed to initialize agent registry:", error);
      throw new Error(
        `Registry initialization failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  private async initializeTaskOrchestrator(
    registry: AgentRegistry
  ): Promise<void> {
    if (this.taskOrchestrator) {
      await this.taskOrchestrator.shutdown();
      this.taskOrchestrator = null;
    }

    const artifactsRoot = path.join(this.options.outputDir, "worker-artifacts");
    await fsp.mkdir(artifactsRoot, { recursive: true });
    this.workerArtifactsRoot = artifactsRoot;

    const orchestratorConfig: TaskOrchestratorConfig = {
      workerPool: {
        minPoolSize: 1,
        maxPoolSize: 1,
        workerCapabilities: ["script"],
        workerTimeout: 60_000,
        artifactConfig: {
          rootPath: artifactsRoot,
          maxFileSizeBytes: 10 * 1024 * 1024,
          maxTotalFiles: 100,
          maxPathLength: 255,
        },
      },
      queue: {
        maxSize: 100,
        priorityLevels: [
          TaskPriority.LOW,
          TaskPriority.MEDIUM,
          TaskPriority.HIGH,
          TaskPriority.CRITICAL,
        ],
        persistenceEnabled: false,
      },
      retry: {
        maxAttempts: 1,
        backoffMultiplier: 2,
        initialDelay: 1_000,
        maxDelay: 10_000,
      },
      routing: {
        enabled: true,
        strategy: "load_balanced",
      },
      performance: {
        trackingEnabled: false,
        metricsInterval: 60_000,
      },
      pleading: {
        enabled: false,
        requiredApprovals: 0,
        timeoutHours: 1,
      },
    };

    this.taskOrchestrator = new TaskOrchestrator(
      orchestratorConfig,
      registry as any,
      this.performanceTracker
    );

    await this.taskOrchestrator.initialize();
  }

  getStatus(): RuntimeStatus {
    return {
      running: this.running,
      registryReady: this.registryReady,
      queueDepth: this.queue.size(),
      activeTasks: this.queue.getProcessingTasks().length,
      completedTasks: this.completedTasks,
      failedTasks: this.failedTasks,
    };
  }

  getMetrics(): RuntimeMetrics {
    return {
      totalTasks: this.totalTasks,
      completedTasks: this.completedTasks,
      failedTasks: this.failedTasks,
      averageDurationMs:
        this.completedTasks > 0
          ? Math.round(this.cumulativeDuration / this.completedTasks)
          : 0,
      lastTaskDurationMs: this.lastDuration,
    };
  }

  getTaskSnapshot(taskId: string): RuntimeTaskSnapshot | null {
    const record = this.taskRecords.get(taskId);
    if (!record) {
      return null;
    }

    return {
      taskId,
      state: record.status,
      description: record.description,
      createdAt: new Date(record.createdAt),
      updatedAt: new Date(record.updatedAt),
      plan: [...record.plan],
      nextActions: [...record.nextActions],
      outputPath: record.outputPath,
      artifacts: record.artifacts,
      assignedAgentId:
        (record.metadata?.assignedAgentId as string | undefined) ?? undefined,
      metadata: record.metadata ? { ...record.metadata } : undefined,
      cawsResult: record.cawsResult
        ? {
            passed: record.cawsResult.passed,
            verdict: record.cawsResult.verdict,
            remediation: record.cawsResult.remediation
              ? [...record.cawsResult.remediation]
              : undefined,
          }
        : undefined,
      verificationResult: record.verificationResult
        ? { ...record.verificationResult }
        : undefined,
    };
  }

  async waitForCompletion(taskId: string): Promise<void> {
    const record = this.taskRecords.get(taskId);
    if (
      record &&
      (record.status === TaskState.COMPLETED ||
        record.status === TaskState.FAILED)
    ) {
      return;
    }

    return new Promise<void>((resolve, reject) => {
      this.completionResolvers.set(taskId, { resolve, reject });
    });
  }

  async submitTask(
    options: SubmitTaskOptions
  ): Promise<{ taskId: string; assignmentId?: string; queued: boolean }> {
    if (!this.running) {
      await this.start();
    }

    if (!this.registryReady) {
      throw new Error("Registry not initialized - cannot accept tasks yet");
    }

    const taskId = crypto.randomUUID();
    const createdAt = new Date();
    const plan = this.generatePlan(options.description, options.metadata);

    let cawsValidation: CAWSValidationResult | null = null;
    let specMetadataPath: string | undefined;
    if (options.specPath) {
      const absoluteSpecPath = path.isAbsolute(options.specPath)
        ? options.specPath
        : path.resolve(process.cwd(), options.specPath);
      specMetadataPath = path.relative(process.cwd(), absoluteSpecPath);
      cawsValidation = await this.enforceCawsPolicies(taskId, absoluteSpecPath);
    }

    const fallbackAssignmentId = "arbiter-runtime";
    let assignmentId: string | undefined;
    let routingStrategy: string | undefined;

    // Check for script task payload in metadata
    const scriptTaskPayload = options.metadata?.task?.payload;
    const hasScriptPayload = options.task?.payload || scriptTaskPayload;

    // Only attempt routing for agent-based tasks, not direct script execution
    if (!hasScriptPayload) {
      try {
        const routingDecision = await this.routingManager.routeTask({
          id: taskId,
          description: options.task?.description ?? options.description,
          type: options.task?.type ?? "code-editing",
          requiredCapabilities: options.task?.requiredCapabilities ?? {},
          priority: options.task?.priority ?? 5,
          timeoutMs: options.task?.timeoutMs ?? 60_000,
          budget: options.task?.budget ?? {
            maxFiles: 10,
            maxLoc: 500,
          },
          createdAt: createdAt,
          metadata: options.task?.metadata ?? options.metadata ?? {},
          attempts: options.task?.attempts ?? 0,
          maxAttempts: options.task?.maxAttempts ?? 1,
        } as any);
        assignmentId = routingDecision.selectedAgent?.id;
        routingStrategy = routingDecision.strategy;
        this.emitEvent(EventTypes.TASK_ASSIGNED, {
          taskId,
          agentId: assignmentId ?? fallbackAssignmentId,
          strategy: routingDecision.strategy,
        });
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        this.emitEvent(EventTypes.TASK_ASSIGNED, {
          taskId,
          agentId: fallbackAssignmentId,
          error: message,
        });
        throw new NoEligibleAgentsError(
          `No eligible agents for task ${taskId}: ${message}`
        );
      }
    } else {
      // Script execution tasks don't need routing
      this.emitEvent(EventTypes.TASK_ASSIGNED, {
        taskId,
        agentId: fallbackAssignmentId,
        strategy: "direct-execution",
      });
    }

    // Create task from provided specification or defaults
    const task: ArbiterTask = {
      id: taskId,
      description: options.task?.description ?? options.description,
      type: options.task?.type ?? (scriptTaskPayload ? "script" : "general"),
      requiredCapabilities: options.task?.requiredCapabilities ?? {},
      priority: options.task?.priority ?? 5,
      timeoutMs: options.task?.timeoutMs ?? 60_000,
      budget: options.task?.budget ?? {
        maxFiles: 10,
        maxLoc: 500,
      },
      createdAt: options.task?.createdAt ?? createdAt,
      metadata: options.task?.metadata ?? options.metadata ?? {},
      attempts: options.task?.attempts ?? 0,
      maxAttempts: options.task?.maxAttempts ?? 1,
      ...(hasScriptPayload && {
        payload: options.task?.payload || scriptTaskPayload,
      }),
    };

    this.stateMachine.initializeTask(taskId);
    this.stateMachine.transition(taskId, TaskState.QUEUED, "Task submitted");

    const record: RuntimeTaskRecord = {
      task,
      description: options.description,
      plan,
      nextActions: [...plan],
      createdAt,
      updatedAt: createdAt,
      status: TaskState.QUEUED,
      metadata: {
        ...(options.metadata ?? {}),
        ...(specMetadataPath && {
          specPath: specMetadataPath,
        }),
        assignedAgentId: assignmentId ?? fallbackAssignmentId,
        ...(routingStrategy && { routingStrategy }),
      },
    };

    if (cawsValidation) {
      record.cawsResult = {
        passed: cawsValidation.passed,
        verdict: cawsValidation.verdict,
        remediation: cawsValidation.remediation,
      };
    }

    this.taskRecords.set(taskId, record);
    this.queue.enqueue(task);
    this.totalTasks += 1;

    await this.recordChainOfThought(taskId, "observation", {
      content: `Received task request: ${options.description}`,
    });
    await this.recordChainOfThought(taskId, "analysis", {
      content: `Generated ${plan.length} plan steps for execution.`,
    });
    await this.recordChainOfThought(taskId, "plan", {
      content: plan.map((step, index) => `${index + 1}. ${step}`).join("\n"),
    });

    this.processQueue().catch((error) => {
      console.error("ArbiterRuntime queue error", error);
    });

    return {
      taskId,
      assignmentId: assignmentId ?? fallbackAssignmentId,
      queued: true,
    };
  }

  async executeCommand(command: string): Promise<{
    acknowledged: boolean;
    note?: string;
    status?: RuntimeStatus;
    metrics?: RuntimeMetrics;
  }> {
    const trimmed = command.trim().toLowerCase();

    switch (trimmed) {
      case "status":
        return { acknowledged: true, status: this.getStatus() };
      case "metrics":
        return { acknowledged: true, metrics: this.getMetrics() };
      case "tasks:list": {
        const tasks = Array.from(this.taskRecords.entries()).map(
          ([id, record]) => ({
            id,
            state: record.status,
            updatedAt: record.updatedAt.toISOString(),
            outputPath: record.outputPath
              ? path.relative(process.cwd(), record.outputPath)
              : null,
          })
        );
        return { acknowledged: true, note: JSON.stringify(tasks, null, 2) };
      }
      default:
        return { acknowledged: false, note: `Unknown command: ${command}` };
    }
  }

  private async processQueue(): Promise<void> {
    if (this.processing) {
      return;
    }

    this.processing = true;
    try {
      while (this.running) {
        const task = this.queue.dequeue();
        if (!task) {
          break;
        }

        await this.executeTask(task);
      }
    } finally {
      this.processing = false;
    }
  }

  private async executeTask(task: ArbiterTask): Promise<void> {
    const record = this.taskRecords.get(task.id);
    if (!record) {
      return;
    }

    const start = Date.now();
    record.status = TaskState.ASSIGNED;
    record.updatedAt = new Date();

    // Start performance tracking for this task
    const assignedAgentId =
      (record.metadata?.assignedAgentId as string) || "arbiter-runtime";
    const routingDecision = {
      taskId: task.id,
      selectedAgent: assignedAgentId,
      routingStrategy: ((record.metadata?.routingStrategy as string) ||
        "direct-execution") as any, // Type assertion for compatibility
      confidence: 0.8, // Default confidence for runtime execution
      rationale: "Arbiter runtime execution",
      alternativesConsidered: [],
      timestamp: new Date().toISOString(),
    };

    const executionId = this.performanceTracker.startTaskExecution(
      task.id,
      assignedAgentId,
      routingDecision,
      {
        taskType: task.type,
        priority: task.priority,
        timeoutMs: task.timeoutMs,
        budget: task.budget,
      }
    );

    this.stateMachine.transition(task.id, TaskState.ASSIGNED, "Task assigned");
    this.emitEvent(EventTypes.TASK_ASSIGNED, {
      taskId: task.id,
      agentId: "arbiter-runtime",
    });

    this.stateMachine.transition(
      task.id,
      TaskState.RUNNING,
      "Task execution started"
    );
    record.status = TaskState.RUNNING;
    record.updatedAt = new Date();

    this.emitEvent(EventTypes.TASK_PROGRESS_UPDATED, {
      taskId: task.id,
      progress: 0,
      step: "Execution started",
    });

    for (const [index, step] of record.plan.entries()) {
      await this.recordChainOfThought(task.id, "decision", {
        content: `Executing step ${index + 1}: ${step}`,
      });
      this.emitEvent(EventTypes.TASK_PROGRESS_UPDATED, {
        taskId: task.id,
        progress: (index + 1) / record.plan.length,
        step,
      });
    }

    try {
      let outputPath: string;

      const currentAssignmentId = record.metadata?.assignedAgentId
        ? String(record.metadata.assignedAgentId)
        : undefined;
      const runOutcome = await this.runWithTaskOrchestrator(
        record,
        currentAssignmentId
      );
      const executionResult = runOutcome.result;
      const resolvedAgentId =
        runOutcome.assignedAgentId ?? currentAssignmentId ?? "arbiter-runtime";

      if (resolvedAgentId) {
        record.metadata = {
          ...record.metadata,
          assignedAgentId: resolvedAgentId,
        };
      }

      if (executionResult?.artifacts) {
        record.artifacts = {
          manifest: executionResult.artifacts.manifest,
          rootPath: executionResult.artifacts.rootPath,
        };
        const firstFile =
          executionResult.artifacts.manifest.files[0]?.path ?? "summary.md";
        outputPath = path.join(executionResult.artifacts.rootPath, firstFile);
        record.outputPath = outputPath;
        if (executionResult.logs?.length) {
          record.metadata = {
            ...(record.metadata ?? {}),
            logs: executionResult.logs,
          };
        }
      } else {
        outputPath = await this.materializeTask(task.id, record);
        record.outputPath = outputPath;
      }

      const verification = await this.verifyTaskOutput(
        task.id,
        record.outputPath!
      );
      record.verificationResult = verification;

      const failedVerification =
        (verification.verdict === VerificationVerdict.VERIFIED_FALSE ||
          verification.verdict === VerificationVerdict.CONTRADICTORY) &&
        verification.confidence >=
          this.verificationConfig.minConfidenceThreshold;
      const erroredVerification =
        verification.verdict === VerificationVerdict.ERROR;

      if (failedVerification || erroredVerification) {
        throw new Error(
          `Verification failed with verdict ${
            verification.verdict
          } (confidence ${(verification.confidence * 100).toFixed(1)}%)`
        );
      }

      record.status = TaskState.COMPLETED;
      record.nextActions = [];
      record.updatedAt = new Date();

      await this.recordChainOfThought(task.id, "execute", {
        content: `Task produced artefact at ${path.relative(
          process.cwd(),
          record.outputPath!
        )}`,
      });

      this.stateMachine.transition(
        task.id,
        TaskState.COMPLETED,
        "Task completed"
      );
      this.queue.complete(task.id);
      this.emitEvent(EventTypes.TASK_COMPLETED, {
        taskId: task.id,
        outputPath: record.outputPath,
      });

      this.completedTasks += 1;
      const duration = Date.now() - start;
      this.lastDuration = duration;
      this.cumulativeDuration += duration;

      // Record successful task completion in Performance Tracker
      if (executionId) {
        await this.performanceTracker.completeTaskExecution(executionId, {
          success: true,
          qualityScore: verification.confidence, // Use verification confidence as quality score
          efficiencyScore: 0.8, // Default efficiency score
          tokensConsumed: 0, // Would need to be extracted from execution result
          completionTimeMs: duration,
        });
      }
    } catch (error) {
      record.status = TaskState.FAILED;
      record.updatedAt = new Date();
      record.nextActions = [];

      const message = error instanceof Error ? error.message : String(error);
      await this.recordChainOfThought(task.id, "critique", {
        content: `Task execution failed: ${message}`,
      });

      this.stateMachine.transition(
        task.id,
        TaskState.FAILED,
        `Task failed: ${message}`
      );
      this.queue.complete(task.id);
      this.emitEvent(EventTypes.TASK_FAILED, {
        taskId: task.id,
        error: message,
      });

      this.failedTasks += 1;

      // Record failed task completion in Performance Tracker
      if (executionId) {
        await this.performanceTracker.completeTaskExecution(executionId, {
          success: false,
          qualityScore: 0.0,
          efficiencyScore: 0.0,
          tokensConsumed: 0,
          completionTimeMs: Date.now() - start,
        });
      }
    } finally {
      const resolver = this.completionResolvers.get(task.id);
      if (resolver) {
        resolver.resolve();
        this.completionResolvers.delete(task.id);
      }
    }
  }

  private async materializeTask(
    taskId: string,
    record: RuntimeTaskRecord
  ): Promise<string> {
    const taskDir = path.join(this.options.outputDir, taskId);
    await fsp.mkdir(taskDir, { recursive: true });

    const summaryPath = path.join(taskDir, "summary.md");
    const content = this.generateSummaryContent(taskId, record);

    await fsp.writeFile(summaryPath, content, "utf8");
    return summaryPath;
  }

  private generatePlan(
    description: string,
    metadata?: Record<string, any>
  ): string[] {
    const sentences = description
      .split(/[.!?\n]+/)
      .map((value) => value.trim())
      .filter(Boolean);

    const plan = sentences.length
      ? sentences.map((sentence) => {
          if (sentence.toLowerCase().startsWith("create")) {
            return `Generate artefact: ${sentence}`;
          }
          if (sentence.toLowerCase().startsWith("write")) {
            return `Draft required content: ${sentence}`;
          }
          if (sentence.toLowerCase().startsWith("add")) {
            return `Apply requested update: ${sentence}`;
          }
          return sentence;
        })
      : [
          "Analyze the request and gather context.",
          "Draft a solution addressing the requirements.",
          "Review the draft for completeness and accuracy.",
          "Summarize the outcome and store artefacts.",
        ];

    if (metadata?.framework) {
      plan.push(`Ensure compatibility with ${metadata.framework}.`);
    }

    plan.push("Prepare verification notes for observer review.");
    return plan;
  }

  private async runWithTaskOrchestrator(
    record: RuntimeTaskRecord,
    fallbackAssignedAgentId?: string
  ): Promise<{
    result: WorkerExecutionResult | null;
    assignedAgentId?: string;
  }> {
    if (!this.taskOrchestrator) {
      return { result: null, assignedAgentId: fallbackAssignedAgentId };
    }

    const runnerTask = this.toRunnerTask(record, fallbackAssignedAgentId);
    const orchestrator = this.taskOrchestrator;
    const taskId = runnerTask.id;
    const envTimeoutMs = Number(process.env.ARBITER_ORCHESTRATOR_TIMEOUT_MS);
    const timeoutMs =
      Number.isFinite(envTimeoutMs) && envTimeoutMs > 0
        ? envTimeoutMs
        : DEFAULT_ORCHESTRATOR_TIMEOUT_MS;

    return new Promise<{
      result: WorkerExecutionResult | null;
      assignedAgentId?: string;
    }>((resolve, reject) => {
      let settled = false;
      let latestAssignedAgent = fallbackAssignedAgentId;
      const timeoutHandle: ReturnType<typeof setTimeout> = setTimeout(() => {
        if (!settled) {
          settled = true;
          reject(new Error("Task execution timeout"));
        }
      }, timeoutMs);

      const cleanup = () => {
        orchestrator.off(TaskOrchestratorEvents.TASK_COMPLETED, onCompleted);
        orchestrator.off(TaskOrchestratorEvents.TASK_FAILED, onFailed);
        clearTimeout(timeoutHandle);
      };

      const resolveOnce = (value: {
        result: WorkerExecutionResult | null;
        assignedAgentId?: string;
      }) => {
        if (settled) {
          return;
        }
        settled = true;
        cleanup();
        resolve(value);
      };

      const rejectOnce = (error: unknown) => {
        if (settled) {
          return;
        }
        settled = true;
        cleanup();
        reject(
          error instanceof Error ? error : new Error(String(error ?? "error"))
        );
      };

      const onCompleted = (execution: RunnerTaskExecution) => {
        if (execution.taskId !== taskId) {
          return;
        }
        resolveOnce({
          result: execution.result ?? null,
          assignedAgentId: execution.agentId ?? latestAssignedAgent,
        });
      };

      const onFailed = (execution: RunnerTaskExecution) => {
        if (execution.taskId !== taskId) {
          return;
        }
        rejectOnce(
          new Error(
            execution.error ?? `Task ${execution.taskId} failed in worker`
          )
        );
      };

      orchestrator.on(TaskOrchestratorEvents.TASK_COMPLETED, onCompleted);
      orchestrator.on(TaskOrchestratorEvents.TASK_FAILED, onFailed);

      // Timeout already set in const declaration above

      orchestrator
        .submitTask(runnerTask)
        .then(() => {
          if (runnerTask.assignedAgent) {
            latestAssignedAgent = runnerTask.assignedAgent;
          }
        })
        .catch(rejectOnce);
    });
  }

  private toRunnerTask(
    record: RuntimeTaskRecord,
    fallbackAssignedAgentId?: string
  ): RunnerTask {
    const task = record.task;
    const priority = this.mapPriority(task.priority);
    const assignedAgentId =
      fallbackAssignedAgentId && fallbackAssignedAgentId !== "arbiter-runtime"
        ? fallbackAssignedAgentId
        : undefined;
    const artifactRoot =
      this.workerArtifactsRoot ??
      path.join(this.options.outputDir, "worker-artifacts");

    if (task.payload && task.payload.code) {
      return {
        id: task.id,
        type: "script",
        priority,
        payload: {
          code: task.payload.code,
          args: task.payload.args ?? [],
          timeout: task.payload.timeout ?? task.timeoutMs,
        },
        metadata: {
          ...task.metadata,
          description: task.description,
          plan: record.plan,
          traceId: task.metadata?.traceId ?? task.id,
          artifactRoot: path.join(artifactRoot, task.id),
        },
        assignedAgent:
          assignedAgentId !== undefined ? assignedAgentId : undefined,
        createdAt: task.createdAt,
        timeout: task.timeoutMs,
        requiredCapabilities: task.requiredCapabilities,
        budget: task.budget,
      };
    }

    const summary = this.generateSummaryContent(task.id, record);
    const script = this.generateWorkerScript(summary);

    return {
      id: task.id,
      type: "script",
      priority,
      payload: {
        code: script,
        timeout: task.timeoutMs ?? 60_000,
      },
      metadata: {
        description: task.description,
        plan: record.plan,
        specPath: record.metadata?.specPath,
        traceId: task.metadata?.traceId ?? task.id,
        artifactRoot: path.join(artifactRoot, task.id),
      },
      assignedAgent:
        assignedAgentId !== undefined ? assignedAgentId : undefined,
      createdAt: task.createdAt,
      timeout: task.timeoutMs,
      requiredCapabilities: task.requiredCapabilities,
      budget: task.budget,
    };
  }

  private mapPriority(priority: number): TaskPriority {
    if (priority >= 9) {
      return TaskPriority.CRITICAL;
    }
    if (priority >= 7) {
      return TaskPriority.HIGH;
    }
    if (priority >= 4) {
      return TaskPriority.MEDIUM;
    }
    return TaskPriority.LOW;
  }

  private generateWorkerScript(summary: string): string {
    const serializedSummary = JSON.stringify(summary);
    return `
const summary = ${serializedSummary};
return context.artifacts
  .writeFile("summary.md", summary)
  .then(() => {
    context.result = { summaryPath: "summary.md" };
  });
`;
  }

  private generateSummaryContent(
    taskId: string,
    record: RuntimeTaskRecord
  ): string {
    return [
      `# Arbiter Task ${taskId}`,
      "",
      `**Created:** ${record.createdAt.toISOString()}`,
      `**Last Updated:** ${new Date().toISOString()}`,
      "",
      "## Description",
      record.description,
      "",
      "## Plan",
      ...record.plan.map((step, index) => `${index + 1}. ${step}`),
      "",
      "## Metadata",
      "```json",
      JSON.stringify(record.metadata ?? {}, null, 2),
      "```",
    ].join("\n");
  }

  private async enforceCawsPolicies(
    taskId: string,
    specPath: string
  ): Promise<CAWSValidationResult> {
    const spec = await this.loadWorkingSpec(specPath);
    const validation = await this.cawsValidator.validateWorkingSpec(spec, {
      projectRoot: path.dirname(specPath),
      checkBudget: true,
    });

    this.emitEvent(EventTypes.CAWS_VALIDATION, {
      taskId,
      specId: spec.id,
      passed: validation.passed,
      verdict: validation.verdict,
      timestamp: validation.timestamp,
    });

    await this.recordChainOfThought(taskId, "verify", {
      content: validation.passed
        ? `CAWS validation passed for spec ${spec.id}`
        : `CAWS validation failed for spec ${spec.id}: ${validation.verdict}`,
      confidence: validation.passed ? 0.9 : 0.4,
    });

    // Record CAWS validation in Performance Tracker
    await this.performanceTracker.recordConstitutionalValidation({
      taskId,
      agentId: "arbiter-runtime", // CAWS validation is done by runtime
      validationResult: {
        valid: validation.passed,
        violations:
          validation.remediation?.map((msg) => ({
            severity: "medium" as const, // Default severity
            message: msg,
          })) ?? [],
        complianceScore: validation.passed ? 1.0 : 0.0,
        processingTimeMs: 0, // CAWS validation result doesn't expose this
        ruleCount: 0, // CAWS validation result doesn't expose this
      },
    });

    if (!validation.passed) {
      const remediation = validation.remediation?.join("; ") ?? "None provided";
      throw new Error(
        `CAWS validation failed for ${spec.id}. Verdict: ${validation.verdict}. Remediation: ${remediation}`
      );
    }

    return validation;
  }

  private async loadWorkingSpec(specPath: string): Promise<WorkingSpec> {
    try {
      const contents = await fsp.readFile(specPath, "utf8");
      const parsed = yaml.load(contents);
      if (!parsed || typeof parsed !== "object") {
        throw new Error("Invalid CAWS specification format");
      }
      return parsed as WorkingSpec;
    } catch (error) {
      throw new Error(
        `Failed to load working spec at ${specPath}: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  private createVerificationConfig(): VerificationEngineConfig {
    return {
      defaultTimeoutMs: 5000,
      maxConcurrentVerifications: 3,
      minConfidenceThreshold: 0.6,
      maxEvidencePerMethod: 5,
      cacheEnabled: false,
      cacheTtlMs: 60_000,
      retryAttempts: 1,
      retryDelayMs: 500,
      methods: [
        {
          type: VerificationType.CONSISTENCY_CHECK,
          enabled: true,
          priority: 1,
          timeoutMs: 2000,
          config: {},
        },
        {
          type: VerificationType.LOGICAL_VALIDATION,
          enabled: true,
          priority: 2,
          timeoutMs: 2000,
          config: {},
        },
        {
          type: VerificationType.FACT_CHECKING,
          enabled: true,
          priority: 3,
          timeoutMs: 4000,
          config: {},
        },
      ],
    };
  }

  private async verifyTaskOutput(
    taskId: string,
    outputPath: string
  ): Promise<VerificationResult> {
    try {
      const content = await fsp.readFile(outputPath, "utf8");
      const record = this.taskRecords.get(taskId);
      const conversationContext = this.buildVerificationConversationContext(
        taskId,
        record
      );
      const evidenceManifest = this.buildVerificationEvidence(record);
      const request = {
        id: taskId,
        content,
        priority: VerificationPriority.MEDIUM,
        metadata: {
          outputPath,
          taskType: record?.task.type,
          previousMessages: conversationContext.previousMessages,
          tenantId: conversationContext.tenantId,
        },
        timeoutMs: this.verificationConfig.defaultTimeoutMs,
        verificationTypes: this.verificationConfig.methods
          .filter((method) => method.enabled)
          .map((method) => method.type),
        conversationContext,
        evidenceManifest,
      };

      const result = await this.verificationEngine.verify(request);

      this.emitEvent(EventTypes.CAWS_COMPLIANCE, {
        taskId,
        verdict: result.verdict,
        confidence: result.confidence,
        processingTimeMs: result.processingTimeMs,
      });

      await this.recordChainOfThought(taskId, "verify", {
        content: `Verification verdict: ${result.verdict} (confidence ${(
          result.confidence * 100
        ).toFixed(1)}%)`,
        confidence: result.confidence,
      });

      return result;
    } catch (error) {
      this.emitEvent(EventTypes.CAWS_COMPLIANCE, {
        taskId,
        verdict: VerificationVerdict.ERROR,
        confidence: 0,
        error: error instanceof Error ? error.message : String(error),
      });
      await this.recordChainOfThought(taskId, "critique", {
        content: `Verification failed: ${
          error instanceof Error ? error.message : String(error)
        }`,
        confidence: 0.3,
      });
      throw error;
    }
  }

  private buildVerificationConversationContext(
    taskId: string,
    record?: RuntimeTaskRecord
  ): ConversationContext {
    const messages: string[] = [];

    if (record?.description) {
      messages.push(`Task description: ${record.description}`);
    }

    if (Array.isArray(record?.plan)) {
      messages.push(
        ...record.plan
          .filter((step): step is string => typeof step === "string")
          .map((step, index) => `Plan ${index + 1}: ${step}`)
      );
    }

    if (Array.isArray(record?.nextActions)) {
      messages.push(
        ...record.nextActions
          .filter((action): action is string => typeof action === "string")
          .map((action) => `Next action: ${action}`)
      );
    }

    if (record?.task?.metadata?.previousMessages) {
      messages.push(
        ...this.extractStringArray(record.task.metadata.previousMessages)
      );
    }

    if (record?.metadata?.previousMessages) {
      messages.push(
        ...this.extractStringArray(record.metadata.previousMessages)
      );
    }

    const tenantId =
      (record?.task?.metadata?.tenantId as string) ??
      (record?.metadata?.tenantId as string) ??
      "arbiter";

    return {
      conversationId: taskId,
      tenantId,
      previousMessages: messages,
      metadata: {
        taskMetadata: record?.task?.metadata ?? {},
        runtimeMetadata: record?.metadata ?? {},
      },
    };
  }

  private buildVerificationEvidence(
    record?: RuntimeTaskRecord
  ): EvidenceManifest {
    const artifactSources =
      record?.artifacts?.manifest?.files?.map((file) => ({
        name: file.path,
        type: file.mimeType ?? "artifact",
        reliability: 0.5,
        lastUpdated: file.createdAt,
        responseTime: 0,
      })) ?? [];

    return {
      sources: artifactSources,
      evidence: [],
      quality: 0,
      cawsCompliant: false,
    };
  }

  private extractStringArray(value: unknown): string[] {
    if (!value) {
      return [];
    }

    if (Array.isArray(value)) {
      return value
        .map((item) => (typeof item === "string" ? item.trim() : ""))
        .filter((item) => item.length > 0);
    }

    if (typeof value === "string") {
      const trimmed = value.trim();
      return trimmed.length > 0 ? [trimmed] : [];
    }

    return [];
  }

  // Registry stub method removed - now using real registry via RegistryProvider

  private async recordChainOfThought(
    taskId: string,
    phase:
      | "observation"
      | "analysis"
      | "plan"
      | "decision"
      | "execute"
      | "verify"
      | "hypothesis"
      | "critique"
      | "other",
    details: { content: string; confidence?: number }
  ): Promise<void> {
    const bridge = getObserverBridge();
    if (!bridge) return;

    const entry: ChainOfThoughtEntry = {
      id: `${taskId}-${phase}-${Date.now()}`,
      taskId,
      phase,
      agentId: "arbiter-runtime",
      agentRole: "ORCHESTRATOR",
      timestamp: new Date().toISOString(),
      confidence: details.confidence ?? 0.8,
      content: details.content,
    };

    await bridge.recordChainOfThought(entry);
  }

  private emitEvent(type: string, metadata: Record<string, any>): void {
    events.emit({
      id: `${type}-${Date.now()}-${crypto.randomUUID()}`,
      type,
      timestamp: new Date(),
      severity:
        type === EventTypes.TASK_FAILED
          ? EventSeverity.ERROR
          : EventSeverity.INFO,
      source: "ArbiterRuntime",
      metadata,
      taskId: metadata.taskId,
    });
  }
}
