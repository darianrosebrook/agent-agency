// @ts-nocheck
import crypto from "crypto";
import fs from "fs";
import path from "path";
import {
  ArbiterController,
  ChainOfThoughtEntry,
  ObserverConfig,
  ObserverEventPayload,
  ObserverMetricsSnapshot,
  ObserverProgressSummary,
  ObserverStatusSummary,
  ObserverStore,
  SubmitTaskPayload,
  SubmitTaskResult,
} from "../types";
import { ArbiterRuntime } from "../../orchestrator/runtime/ArbiterRuntime";
import { AsyncFileWriter } from "./AsyncFileWriter";
import { Redactor } from "../redactor";

interface PersistedEvent extends ObserverEventPayload {
  seq: number;
  schemaVersion: string;
  sourceVersion: string;
}

interface PersistedCotEntry extends ChainOfThoughtEntry {
  seq: number;
  schemaVersion: string;
  sourceVersion: string;
}

const MAX_EVENTS_IN_MEMORY = 5000;
const MAX_COT_IN_MEMORY = 5000;

function encodeCursor(seq: number | undefined): string | undefined {
  if (seq === undefined) return undefined;
  return Buffer.from(String(seq), "utf8").toString("base64");
}

function decodeCursor(cursor?: string): number | undefined {
  if (!cursor) return undefined;
  try {
    const decoded = Buffer.from(cursor, "base64").toString("utf8");
    const parsed = Number(decoded);
    return Number.isFinite(parsed) ? parsed : undefined;
  } catch {
    return undefined;
  }
}

/**
 * ObserverStoreImpl combines in-memory caches with JSONL persistence. It also
 * acts as the bridge controller for basic arbiter lifecycle commands.
 */
export class ObserverStoreImpl implements ObserverStore, ArbiterController {
  private readonly config: ObserverConfig;
  private readonly redactor: Redactor;
  private readonly runtime: ArbiterRuntime | null;
  private readonly eventsWriter: AsyncFileWriter;
  private readonly cotWriter: AsyncFileWriter;
  private readonly metricsPath: string;
  private readonly startedAt = Date.now();
  private nextEventSeq = 1;
  private nextCotSeq = 1;
  private degraded = false;
  private backpressureEvents = 0;
  private lastFlushTime = Date.now();
  private pendingWrites = 0;
  private readonly events: PersistedEvent[] = [];
  private readonly cotEntries: PersistedCotEntry[] = [];
  private readonly reasoningCounters = {
    observations: 0,
    analyses: 0,
    plans: 0,
    decisions: 0,
    executions: 0,
    verifications: 0,
  };
  private readonly reasoningByTask = new Map<string, number>();
  private readonly debateBreadth = new Map<string, Set<string>>();
  private totalTasks = 0;
  private successfulTasks = 0;
  private policyViolations = 0;
  private aggregateBudgetDebit = 0;
  private aggregateBudgetLimit = 0;

  constructor(config: ObserverConfig, runtime?: ArbiterRuntime) {
    this.config = config;
    this.redactor = new Redactor(config);
    this.runtime = runtime ?? null;
    this.eventsWriter = new AsyncFileWriter(config, "events");
    this.cotWriter = new AsyncFileWriter(config, "cot");
    this.metricsPath = path.join(config.dataDir, "metrics.json");
  }

  // ---------------- ObserverStore interface ----------------

  getStatus(): ObserverStatusSummary {
    const runtimeStatus = this.runtime?.getStatus();
    const isRunning = runtimeStatus ? runtimeStatus.running : true;
    const effectiveQueueDepth = runtimeStatus
      ? runtimeStatus.queueDepth
      : this.pendingWrites;

    return {
      status: !isRunning
        ? "stopped"
        : this.degraded
        ? "degraded"
        : "running",
      startedAt: new Date(this.startedAt).toISOString(),
      uptimeMs: Date.now() - this.startedAt,
      queueDepth: effectiveQueueDepth,
      maxQueueSize: this.config.maxQueueSize,
      observerDegraded: this.degraded,
      lastFlushMs: Date.now() - this.lastFlushTime,
      activeFile: this.eventsWriter.getActiveFile(),
      backpressureEvents: this.backpressureEvents,
      authConfigured: Boolean(this.config.authToken),
    };
  }

  getMetrics(): ObserverMetricsSnapshot {
    const runtimeStatus = this.runtime?.getStatus();
    const runtimeMetrics = this.runtime?.getMetrics();

    const reasoningDepthValues = Array.from(this.reasoningByTask.values());
    const reasoningDepthAvg =
      reasoningDepthValues.length === 0
        ? 0
        : reasoningDepthValues.reduce((a, b) => a + b, 0) /
          reasoningDepthValues.length;
    const reasoningDepthP95 =
      reasoningDepthValues.length === 0
        ? 0
        : percentile(reasoningDepthValues, 0.95);
    const debateBreadthValues = Array.from(this.debateBreadth.values()).map(
      (set) => set.size
    );
    const debateBreadthAvg =
      debateBreadthValues.length === 0
        ? 0
        : debateBreadthValues.reduce((a, b) => a + b, 0) /
          debateBreadthValues.length;

    const toolBudgetUtilization =
      this.aggregateBudgetLimit > 0
        ? this.aggregateBudgetDebit / this.aggregateBudgetLimit
        : 0;

    const totalTasks = runtimeMetrics?.totalTasks ?? this.totalTasks;
    const completedTasks = runtimeMetrics?.completedTasks ?? this.successfulTasks;
    const taskSuccessRate =
      totalTasks === 0 ? 0 : completedTasks / totalTasks;

    return {
      timestamp: new Date().toISOString(),
      reasoningDepthAvg,
      reasoningDepthP95,
      debateBreadthAvg,
      taskSuccessRate,
      toolBudgetUtilization,
      activeTasks:
        runtimeStatus?.activeTasks ?? countActiveTasks(this.events),
      queuedTasks:
        runtimeStatus?.queueDepth ?? countQueuedTasks(this.events),
      policyViolations: this.policyViolations,
      observerDegraded: this.degraded,
      queueDepth: runtimeStatus?.queueDepth ?? this.pendingWrites,
    };
  }

  getProgress(): ObserverProgressSummary {
    const runtimeStatus = this.runtime?.getStatus();
    return {
      status: runtimeStatus
        ? runtimeStatus.running
          ? this.degraded
            ? "degraded"
            : "running"
          : "not_started"
        : this.degraded
        ? "degraded"
        : "running",
      reasoningSteps: { ...this.reasoningCounters },
      totalReasoningSteps: Object.values(this.reasoningCounters).reduce(
        (memo, value) => memo + value,
        0
      ),
      uptimeMinutes: Math.round((Date.now() - this.startedAt) / 60000),
    };
  }

  async listEvents(params: {
    cursor?: string;
    limit?: number;
    since?: Date;
    until?: Date;
    type?: string;
    taskId?: string;
    severity?: "debug" | "info" | "warn" | "error";
  }): Promise<{ events: ObserverEventPayload[]; nextCursor?: string }> {
    const lastSeq = decodeCursor(params.cursor) ?? 0;
    const limit = Math.min(params.limit ?? 100, 500);

    const filtered = this.events.filter((event) => {
      if (event.seq <= lastSeq) return false;
      if (params.type && event.type !== params.type) return false;
      if (params.taskId && event.taskId !== params.taskId) return false;
      if (params.severity && event.severity !== params.severity) return false;
      if (params.since && new Date(event.timestamp) < params.since) {
        return false;
      }
      if (params.until && new Date(event.timestamp) > params.until) {
        return false;
      }
      return true;
    });

    const slice = filtered.slice(0, limit);
    const nextSeq = slice.length ? slice[slice.length - 1].seq : undefined;

    return {
      events: slice.map((event) => ({
        id: event.id,
        type: event.type,
        severity: event.severity,
        source: event.source,
        taskId: event.taskId,
        agentId: event.agentId,
        timestamp: event.timestamp,
        traceId: event.traceId,
        spanId: event.spanId,
        correlationId: event.correlationId,
        metadata: event.metadata,
      })),
      nextCursor: encodeCursor(nextSeq),
    };
  }

  async listChainOfThought(params: {
    taskId?: string;
    cursor?: string;
    limit?: number;
    since?: Date;
  }): Promise<{ entries: ChainOfThoughtEntry[]; nextCursor?: string }> {
    const lastSeq = decodeCursor(params.cursor) ?? 0;
    const limit = Math.min(params.limit ?? 50, 200);

    const filtered = this.cotEntries.filter((entry) => {
      if (entry.seq <= lastSeq) return false;
      if (params.taskId && entry.taskId !== params.taskId) return false;
      if (params.since && new Date(entry.timestamp) < params.since) {
        return false;
      }
      return true;
    });

    const slice = filtered.slice(0, limit);
    const nextSeq = slice.length ? slice[slice.length - 1].seq : undefined;

    return {
      entries: slice.map(
        ({
          seq: _seq,
          schemaVersion: _schema,
          sourceVersion: _source,
          ...rest
        }) => rest
      ),
      nextCursor: encodeCursor(nextSeq),
    };
  }

  async getTask(taskId: string): Promise<{
    taskId: string;
    state: string;
    progress: string[];
    lastUpdated: string;
    currentPlan?: string;
    nextActions?: string[];
    redacted?: boolean;
    caws?: {
      passed: boolean;
      verdict: string;
      remediation?: string[];
    };
    verification?: {
      verdict: string;
      confidence: number;
      reasoning: string[];
    };
  } | null> {
    const runtimeSnapshot = this.runtime?.getTaskSnapshot(taskId);

    if (runtimeSnapshot) {
      const runtimeProgress = this.events
        .filter((event) => event.taskId === taskId)
        .sort((a, b) => a.seq - b.seq)
        .map((event) => {
          const summary = event.metadata?.note || event.metadata?.step;
          return `${event.timestamp} ${event.type}${summary ? `: ${summary}` : ""}`;
        });

      if (runtimeSnapshot.outputPath) {
        runtimeProgress.push(
          `${runtimeSnapshot.updatedAt.toISOString()} artifact: ${path.relative(
            process.cwd(),
            runtimeSnapshot.outputPath
          )}`
        );
      }

      const planSummary = runtimeSnapshot.plan.length
        ? runtimeSnapshot.plan.join("\n")
        : undefined;

      return {
        taskId,
        state: runtimeSnapshot.state,
        progress: runtimeProgress.length
          ? runtimeProgress
          : runtimeSnapshot.plan.map(
              (step, index) =>
                `${runtimeSnapshot.createdAt.toISOString()} plan[${index + 1}] ${step}`
            ),
        lastUpdated: runtimeSnapshot.updatedAt.toISOString(),
        currentPlan: planSummary,
        nextActions: runtimeSnapshot.nextActions,
        redacted: false,
        caws: runtimeSnapshot.cawsResult
          ? {
              passed: runtimeSnapshot.cawsResult.passed,
              verdict: runtimeSnapshot.cawsResult.verdict,
              remediation: runtimeSnapshot.cawsResult.remediation,
            }
          : undefined,
        verification: runtimeSnapshot.verificationResult
          ? {
              verdict: runtimeSnapshot.verificationResult.verdict,
              confidence: runtimeSnapshot.verificationResult.confidence,
              reasoning: runtimeSnapshot.verificationResult.reasoning,
            }
          : undefined,
      };
    }

    const relevant = this.events
      .filter((event) => event.taskId === taskId)
      .sort((a, b) => a.seq - b.seq);

    if (!relevant.length) {
      return null;
    }

    const progress = relevant.map(
      (event) =>
        `${event.timestamp} ${event.type}${
          event.metadata?.note ? `: ${event.metadata.note}` : ""
        }`
    );

    const lastEvent = relevant[relevant.length - 1];
    const state =
      lastEvent.type === "task.completed"
        ? "completed"
        : lastEvent.type === "task.failed"
        ? "failed"
        : "running";

    const planEntry = this.cotEntries.find(
      (entry) => entry.taskId === taskId && entry.phase === "plan"
    );
    const decisionEntries = this.cotEntries.filter(
      (entry) => entry.taskId === taskId && entry.phase === "decision"
    );

    return {
      taskId,
      state,
      progress,
      lastUpdated: lastEvent.timestamp,
      currentPlan: planEntry?.content,
      nextActions: decisionEntries.map((entry) => entry.content ?? ""),
      redacted: planEntry?.redacted ?? false,
    };
  }

  async appendObservation(note: {
    message: string;
    taskId?: string;
    author?: string;
  }): Promise<{ id: string; timestamp: string }> {
    const id = cryptoRandomId();
    const timestamp = new Date().toISOString();
    const record: ObserverEventPayload = {
      id,
      type: "observer.observation",
      severity: "info",
      source: "observer",
      taskId: note.taskId,
      timestamp,
      metadata: {
        message: this.redactor.redactText(note.message).text ?? "[REDACTED]",
        author: note.author ?? "external",
      },
    };
    this.recordEvent(record);
    return { id, timestamp };
  }

  // ---------------- ArbiterController interface ----------------

  async ensureArbiterRunning(): Promise<{ status: "running" | "starting" }> {
    try {
      if (this.runtime) {
        await this.runtime.start();
        const status = this.runtime.getStatus();
        this.recordSystemLog("observer.arbiter_start", "info", {
          message: "Observer requested arbiter start",
          runtime: status,
        });
        return { status: status.running ? "running" : "starting" };
      }

      this.recordSystemLog("observer.arbiter_start", "warn", {
        message: "No runtime available; start request ignored",
      });
      return { status: "running" };
    } catch (error) {
      this.recordSystemLog("observer.arbiter_start", "error", {
        message: "Failed to start arbiter runtime",
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  async requestArbiterStop(): Promise<{ status: "stopping" | "stopped" }> {
    if (this.runtime) {
      await this.runtime.stop();
      this.recordSystemLog("observer.arbiter_stop", "info", {
        message: "Observer requested arbiter stop",
      });
      return { status: "stopped" };
    }

    this.recordSystemLog("observer.arbiter_stop", "warn", {
      message: "No runtime available; stop request ignored",
    });
    return { status: "stopping" };
  }

  async submitTask(payload: SubmitTaskPayload): Promise<SubmitTaskResult> {
    if (!this.runtime) {
      this.recordSystemLog("observer.submit_task", "warn", {
        message: "Runtime unavailable; task submission ignored",
        payload,
      });
      return {
        taskId: cryptoRandomId(),
        assignmentId: undefined,
        queued: false,
      };
    }

    const result = await this.runtime.submitTask({
      description: payload.description,
      specPath: payload.specPath,
      metadata: payload.metadata,
    });

    this.recordSystemLog("observer.submit_task", "info", {
      message: "Task submitted to runtime",
      payload,
      result,
    });

    return {
      taskId: result.taskId,
      assignmentId: result.assignmentId,
      queued: result.queued,
    };
  }

  async executeCommand(
    command: string
  ): Promise<{ acknowledged: boolean; note?: string }> {
    if (this.runtime) {
      const response = await this.runtime.executeCommand(command);
      this.recordSystemLog("observer.command", response.acknowledged ? "info" : "warn", {
        message: "Runtime command executed",
        command,
        response,
      });
      return {
        acknowledged: response.acknowledged,
        note: response.note,
      };
    }

    this.recordSystemLog("observer.command", "warn", {
      message: "Runtime unavailable; command ignored",
      command,
    });
    return { acknowledged: false, note: "Runtime unavailable" };
  }

  // ---------------- Event ingestion APIs ----------------

  recordEvent(event: ObserverEventPayload): void {
    const seq = this.nextEventSeq++;
    const metadata = event.metadata
      ? this.redactor.redactObject(event.metadata)
      : undefined;

    if (this.pendingWrites >= this.config.maxQueueSize) {
      this.backpressureEvents += 1;
      if (event.severity === "debug" || event.severity === "info") {
        // Drop low-severity events under backpressure conditions
        return;
      }
    }

    const record: PersistedEvent = {
      ...event,
      metadata,
      seq,
      schemaVersion: "1.0.0",
      sourceVersion: "2.0.0",
    };

    this.events.push(record);
    if (this.events.length > MAX_EVENTS_IN_MEMORY) {
      this.events.shift();
    }

    this.pendingWrites += 1;
    this.eventsWriter
      .append(record)
      .then(() => {
        this.pendingWrites -= 1;
        this.lastFlushTime = Date.now();
        void this.writeMetricsSnapshot();
      })
      .catch((error) => this.handlePersistenceError(error));

    this.updateMetricsFromEvent(record);
  }

  recordChainOfThought(entry: ChainOfThoughtEntry): void {
    const seq = this.nextCotSeq++;
    const redacted = entry.content
      ? this.redactor.redactText(entry.content)
      : { redacted: false };

    if (this.pendingWrites >= this.config.maxQueueSize * 1.5) {
      this.backpressureEvents += 1;
      // Under extreme pressure, drop strictly informative COT phases
      if (
        entry.phase === "observation" ||
        entry.phase === "analysis" ||
        entry.phase === "plan"
      ) {
        return;
      }
    }

    const record: PersistedCotEntry = {
      ...entry,
      content: redacted.text,
      redacted: redacted.redacted,
      hash: redacted.hash,
      seq,
      schemaVersion: "1.0.0",
      sourceVersion: "2.0.0",
    };

    this.cotEntries.push(record);
    if (this.cotEntries.length > MAX_COT_IN_MEMORY) {
      this.cotEntries.shift();
    }

    this.pendingWrites += 1;
    this.cotWriter
      .append(record)
      .then(() => {
        this.pendingWrites -= 1;
        this.lastFlushTime = Date.now();
        void this.writeMetricsSnapshot();
      })
      .catch((error) => this.handlePersistenceError(error));

    this.updateReasoningCounters(record);
  }

  // ---------------- Internal helpers ----------------

  async shutdown(): Promise<void> {
    await this.eventsWriter.close();
    await this.cotWriter.close();
    await this.writeMetricsSnapshot();
  }

  private updateMetricsFromEvent(event: ObserverEventPayload): void {
    if (event.type === "task.completed") {
      this.totalTasks += 1;
      if (event.metadata?.success !== false) {
        this.successfulTasks += 1;
      }
    } else if (event.type === "task.failed") {
      this.totalTasks += 1;
    } else if (event.type === "policy.caws.violation") {
      this.policyViolations += 1;
    } else if (event.type === "caws.validation") {
      const passed = event.metadata?.passed;
      const verdict = String(event.metadata?.verdict ?? "").toLowerCase();
      if (passed === false || verdict === "fail" || verdict === "waiver-required") {
        this.policyViolations += 1;
      }
    } else if (event.type === "caws.compliance") {
      const verdict = String(event.metadata?.verdict ?? "").toLowerCase();
      if (
        verdict === "verified_false" ||
        verdict === "contradictory" ||
        verdict === "error"
      ) {
        this.policyViolations += 1;
      }
    } else if (event.type.startsWith("budget.")) {
      const debit = Number(event.metadata?.debit ?? 0);
      const limit = Number(event.metadata?.limit ?? 0);
      if (debit > 0) this.aggregateBudgetDebit += debit;
      if (limit > 0) this.aggregateBudgetLimit += limit;
    }
  }

  private updateReasoningCounters(entry: ChainOfThoughtEntry): void {
    switch (entry.phase) {
      case "observation":
        this.reasoningCounters.observations += 1;
        break;
      case "analysis":
        this.reasoningCounters.analyses += 1;
        break;
      case "plan":
        this.reasoningCounters.plans += 1;
        break;
      case "decision":
        this.reasoningCounters.decisions += 1;
        break;
      case "execute":
        this.reasoningCounters.executions += 1;
        break;
      case "verify":
        this.reasoningCounters.verifications += 1;
        break;
      default:
        break;
    }

    if (entry.taskId) {
      const count = this.reasoningByTask.get(entry.taskId) ?? 0;
      this.reasoningByTask.set(entry.taskId, count + 1);
      if (entry.agentId) {
        const set = this.debateBreadth.get(entry.taskId) ?? new Set<string>();
        set.add(entry.agentId);
        this.debateBreadth.set(entry.taskId, set);
      }
    }
  }

  private async writeMetricsSnapshot(): Promise<void> {
    const metrics = this.getMetrics();
    try {
      await fs.promises.writeFile(
        this.metricsPath,
        JSON.stringify(metrics, null, 2)
      );
    } catch (error) {
      console.warn("Failed to write metrics snapshot:", error);
    }
  }

  private handlePersistenceError(error: unknown): void {
    this.pendingWrites = Math.max(0, this.pendingWrites - 1);
    this.degraded = true;
    console.error("Observer persistence error:", error);
  }

  private recordSystemLog(
    type: string,
    severity: "info" | "warn" | "error",
    metadata?: Record<string, unknown>
  ): void {
    const event: ObserverEventPayload = {
      id: cryptoRandomId(),
      type,
      severity,
      source: "observer",
      timestamp: new Date().toISOString(),
      metadata: metadata ? this.redactor.redactObject(metadata) : undefined,
    };
    this.recordEvent(event);
  }
}

// ---------------- Utility helpers ----------------

function percentile(values: number[], pct: number): number {
  if (!values.length) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const idx = Math.min(sorted.length - 1, Math.floor(pct * sorted.length));
  return sorted[idx];
}

function countActiveTasks(events: PersistedEvent[]): number {
  const states = new Map<string, string>();
  for (const event of events) {
    if (!event.taskId) continue;
    if (event.type === "task.submitted") {
      states.set(event.taskId, "submitted");
    } else if (event.type === "task.completed" || event.type === "task.failed") {
      states.set(event.taskId, "terminal");
    }
  }
  let count = 0;
  for (const value of states.values()) {
    if (value !== "terminal") count += 1;
  }
  return count;
}

function countQueuedTasks(events: PersistedEvent[]): number {
  const states = new Map<string, string>();
  for (const event of events) {
    if (!event.taskId) continue;
    if (event.type === "task.submitted") {
      states.set(event.taskId, "queued");
    } else if (event.type === "task.assigned") {
      states.set(event.taskId, "assigned");
    } else if (event.type === "task.completed" || event.type === "task.failed") {
      states.delete(event.taskId);
    }
  }
  return Array.from(states.values()).filter((state) => state === "queued")
    .length;
}

function cryptoRandomId(): string {
  if (typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return (
    Math.random().toString(36).slice(2, 10) +
    Math.random().toString(36).slice(2, 10)
  );
}
