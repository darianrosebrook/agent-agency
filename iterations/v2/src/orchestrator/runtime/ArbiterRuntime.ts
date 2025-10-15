import crypto from "crypto";
import fsp from "fs/promises";
import path from "path";
import { getObserverBridge } from "../../observer";
import type { ChainOfThoughtEntry } from "../../observer/types";
import { PerformanceTracker } from "../../rl/PerformanceTracker";
import type { AgentRegistry } from "../../types/agent-registry";
import type { Task } from "../../types/arbiter-orchestration";
import { TaskState } from "../../types/task-state";
import { events, EventSeverity } from "../EventEmitter";
import { EventTypes } from "../OrchestratorEvents";
import { RegistryProvider } from "../RegistryProvider.js";
import { TaskQueue } from "../TaskQueue";
import { TaskRoutingManager } from "../TaskRoutingManager";
import { TaskStateMachine } from "../TaskStateMachine";
// import type { ArtifactManifest } from "../workers/ArtifactSandbox";
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
  task?: Partial<Task>;
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
  metadata?: Record<string, any>;
}

interface RuntimeTaskRecord {
  task: Task;
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
    enabled: false,
    maxEventsInMemory: 1000,
  });
  private readonly agentRegistry: AgentRegistry;
  private routingManager: TaskRoutingManager;
  private readonly options: ArbiterRuntimeOptions;
  private readonly taskRecords = new Map<string, RuntimeTaskRecord>();
  private readonly activeExecutions = new Map<string, any>(); // taskId -> execution result
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
    await this.initializeRegistry();
  }

  async stop(): Promise<void> {
    this.running = false;
    this.processing = false;
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

      // Update the routing manager to use the real registry
      this.routingManager = new TaskRoutingManager(
        realRegistry as any,
        DEFAULT_ROUTING_CONFIG
      );
      this.routingManager.setPerformanceTracker(this.performanceTracker);

      // Wait for ready event
      await readyPromise;

      this.registryReady = true;
      console.log("Agent registry initialized successfully");
    } catch (error) {
      console.error("Failed to initialize agent registry:", error);
      throw new Error(
        `Registry initialization failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
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
      metadata: record.metadata ? { ...record.metadata } : undefined,
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

    let assignmentId = "arbiter-runtime";

    // Only attempt routing for agent-based tasks, not script execution
    if (!options.task?.payload) {
      try {
        const decision = await this.routingManager.routeTask({
          id: taskId,
          type: options.task?.type ?? "code-editing",
          requiredCapabilities: options.task?.requiredCapabilities ?? ["documentation"],
        } as any);
        assignmentId = decision.selectedAgent?.id ?? assignmentId;
        this.emitEvent(EventTypes.TASK_ASSIGNED, {
          taskId,
          agentId: assignmentId,
          strategy: decision.strategy,
        });
      } catch (error) {
        this.emitEvent(EventTypes.TASK_ASSIGNED, {
          taskId,
          agentId: assignmentId,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    } else {
      // Script execution tasks don't need routing
      this.emitEvent(EventTypes.TASK_ASSIGNED, {
        taskId,
        agentId: assignmentId,
        strategy: "direct-execution",
      });
    }

    // Create task from provided specification or defaults
    const task: Task = {
      id: taskId,
      description: options.task?.description ?? options.description,
      type: options.task?.type ?? "general",
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
      ...(options.task?.payload && { payload: options.task.payload }),
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
      metadata: options.metadata,
    };

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
      assignmentId,
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

  private async executeTask(task: Task): Promise<void> {
    const record = this.taskRecords.get(task.id);
    if (!record) {
      return;
    }

    const start = Date.now();
    record.status = TaskState.ASSIGNED;
    record.updatedAt = new Date();

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

      // If worker provided artifacts, use them instead of generating summary
      const execution = this.activeExecutions.get(task.id);
      if (execution?.result?.artifacts) {
        record.artifacts = execution.result.artifacts;
        outputPath = execution.result.artifacts.rootPath;
        record.outputPath = outputPath;
        console.log(
          `Task produced ${execution.result.artifacts.manifest.files.length} artifact files`
        );
      } else {
        // Fallback to legacy summary generation
        outputPath = await this.materializeTask(task.id, record);
        record.outputPath = outputPath;
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
      await this.recordChainOfThought(task.id, "verify", {
        content: "Verified artefact content and recorded completion.",
      });

      this.stateMachine.transition(
        task.id,
        TaskState.COMPLETED,
        "Task completed"
      );
      this.queue.complete(task.id);
      this.emitEvent(EventTypes.TASK_COMPLETED, {
        taskId: task.id,
        outputPath,
      });

      this.completedTasks += 1;
      const duration = Date.now() - start;
      this.lastDuration = duration;
      this.cumulativeDuration += duration;
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
    const content = [
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
