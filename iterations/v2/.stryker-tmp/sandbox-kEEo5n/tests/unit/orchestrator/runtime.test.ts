// @ts-nocheck
import {
  describe,
  expect,
  it,
  beforeEach,
  afterEach,
  jest,
} from "@jest/globals";
import { createHash } from "crypto";
import fs from "fs";
import os from "os";
import path from "path";
import {
  ArbiterRuntime,
  NoEligibleAgentsError,
} from "@/orchestrator/runtime/ArbiterRuntime";
import { runtimeAgentSeeds } from "@/orchestrator/runtime/runtimeAgentDataset";
import {
  TaskOrchestratorEvents,
  TaskExecution as RunnerTaskExecution,
  WorkerExecutionResult,
} from "@/types/task-runner";
import { TaskState } from "@/types/task-state";
import type { ArtifactManifest } from "@/orchestrator/workers/ArtifactSandbox";
import { EventEmitter } from "events";

type FakeBehavior = "complete" | "idle";

class FakeTaskOrchestrator extends EventEmitter {
  private behavior: FakeBehavior;

  constructor(
    private readonly artifactsRoot: string,
    behavior: FakeBehavior = "complete"
  ) {
    super();
    this.behavior = behavior;
  }

  setBehavior(behavior: FakeBehavior): void {
    this.behavior = behavior;
  }

  async submitTask(task: any): Promise<string> {
    if (this.behavior === "idle") {
      return task.id;
    }

    const agentId =
      task.assignedAgent ?? runtimeAgentSeeds[0]?.id ?? "runtime-docsmith";
    const taskDir = path.join(this.artifactsRoot, task.id);
    const summaryPath = path.join(taskDir, "summary.md");
    const summaryContent =
      (task.metadata?.description as string | undefined) ??
      `Task ${task.id} summary`;

    fs.mkdirSync(taskDir, { recursive: true });
    fs.writeFileSync(summaryPath, summaryContent, "utf8");

    const contents = fs.readFileSync(summaryPath);
    const digest = createHash("sha256").update(contents).digest("hex");

    const manifest: ArtifactManifest = {
      taskId: task.id,
      files: [
        {
          path: "summary.md",
          size: contents.length,
          sha256: digest,
          createdAt: new Date().toISOString(),
        },
      ],
      totalSize: contents.length,
      createdAt: new Date().toISOString(),
    };

    const result: WorkerExecutionResult = {
      success: true,
      result: { summaryPath: "summary.md" },
      metrics: {
        executionTime: 5,
        cpuUsage: 0,
        memoryUsage: 0,
        outputSize: contents.length,
      },
      artifacts: {
        manifest,
        rootPath: taskDir,
      },
    };

    const execution: RunnerTaskExecution = {
      executionId: `exec-${task.id}`,
      taskId: task.id,
      agentId,
      startedAt: new Date(),
      completedAt: new Date(),
      status: "completed",
      attempts: 1,
      result,
      artifacts: result.artifacts,
    };

    setImmediate(() => {
      this.emit(TaskOrchestratorEvents.TASK_COMPLETED, execution);
    });

    return task.id;
  }

  async shutdown(): Promise<void> {
    return Promise.resolve();
  }
}

describe("ArbiterRuntime", () => {
  let tempDir: string;
  let runtime: ArbiterRuntime;
  let fakeOrchestrator: FakeTaskOrchestrator;

  beforeEach(async () => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "arbiter-runtime-"));
    runtime = new ArbiterRuntime({ outputDir: tempDir });

    (runtime as any).initializeTaskOrchestrator = async () => {
      const artifactsRoot = path.join(tempDir, "worker-artifacts");
      fs.mkdirSync(artifactsRoot, { recursive: true });
      fakeOrchestrator = new FakeTaskOrchestrator(artifactsRoot, "complete");
      (runtime as any).workerArtifactsRoot = artifactsRoot;
      (runtime as any).taskOrchestrator = fakeOrchestrator;
    };

    await runtime.start();
  });

  afterEach(async () => {
    await runtime.stop();
    fs.rmSync(tempDir, { recursive: true, force: true });
    jest.restoreAllMocks();
    jest.useRealTimers();
    delete process.env.ARBITER_ORCHESTRATOR_TIMEOUT_MS;
  });

  it("initializes the registry with seeded agents", () => {
    const status = runtime.getStatus();
    expect(status.running).toBe(true);
    expect(status.registryReady).toBe(true);

    const metrics = runtime.getMetrics();
    expect(metrics.totalTasks).toBe(0);
    expect(metrics.completedTasks).toBe(0);
    expect(metrics.failedTasks).toBe(0);

    expect(runtimeAgentSeeds.length).toBeGreaterThan(0);
  });

  it("executes tasks via TaskOrchestrator and surfaces worker artifacts", async () => {
    const specPath = path.join(
      __dirname,
      "../../fixtures/sample-working-spec.yaml"
    );
    const { taskId, assignmentId } = await runtime.submitTask({
      description: "Create a hello world summary file",
      metadata: { framework: "jest" },
      specPath,
    });

    const seededAgentIds = runtimeAgentSeeds.map((seed) => seed.id);
    expect(assignmentId).toBeDefined();
    if (assignmentId) {
      expect(seededAgentIds).toContain(assignmentId);
    }

    await runtime.waitForCompletion(taskId);

    const snapshot = runtime.getTaskSnapshot(taskId);
    expect(snapshot).not.toBeNull();
    expect(snapshot?.state).toBe(TaskState.COMPLETED);
    expect(snapshot?.assignedAgentId).toBeDefined();
    expect(snapshot?.cawsResult?.passed).toBe(true);
    expect(snapshot?.artifacts).toBeDefined();
    const manifest = snapshot?.artifacts?.manifest;
    expect(manifest).toBeDefined();
    expect(manifest?.files.length ?? 0).toBeGreaterThan(0);
    if (manifest && snapshot?.artifacts) {
      const [firstFile] = manifest.files;
      const artifactPath = path.join(
        snapshot.artifacts.rootPath,
        firstFile.path
      );
      expect(fs.existsSync(artifactPath)).toBe(true);
      const contents = fs.readFileSync(artifactPath, "utf8");
      expect(contents).toContain("Create a hello world summary file");
    }
    expect(snapshot?.verificationResult).toBeDefined();
    if (snapshot?.verificationResult) {
      expect(snapshot.verificationResult.verdict).toBeDefined();
    }
  });

  it("rejects submissions when no eligible agents are available", async () => {
    const routingManager = (runtime as any).routingManager;
    const routeSpy = jest
      .spyOn(routingManager, "routeTask")
      .mockRejectedValue(new Error("routing failure"));

    await expect(
      runtime.submitTask({
        description: "Impossible routing scenario",
        metadata: { priority: "high" },
      })
    ).rejects.toBeInstanceOf(NoEligibleAgentsError);

    expect(routeSpy).toHaveBeenCalled();
  });

  it("fails tasks when orchestrator emits no completion before timeout", async () => {
    jest.useFakeTimers();
    process.env.ARBITER_ORCHESTRATOR_TIMEOUT_MS = "25";
    fakeOrchestrator.setBehavior("idle");

    const { taskId } = await runtime.submitTask({
      description: "Force orchestrator timeout",
    });

    const waitPromise = runtime.waitForCompletion(taskId);

    if (typeof (jest as any).advanceTimersByTimeAsync === "function") {
      await (jest as any).advanceTimersByTimeAsync(30);
    } else {
      jest.advanceTimersByTime(30);
      await Promise.resolve();
    }

    await waitPromise;

    const snapshot = runtime.getTaskSnapshot(taskId);
    expect(snapshot?.state).toBe(TaskState.FAILED);
    expect(snapshot?.metadata?.assignedAgentId).toBeDefined();
  });

  it("persists worker-generated artifact manifests without legacy materialization", async () => {
    const materializeSpy = jest.spyOn(runtime as any, "materializeTask");

    const { taskId } = await runtime.submitTask({
      description: "Ensure artifact manifest is used",
    });

    await runtime.waitForCompletion(taskId);

    expect(materializeSpy).not.toHaveBeenCalled();

    const snapshot = runtime.getTaskSnapshot(taskId);
    expect(snapshot?.artifacts).toBeDefined();
    const manifest = snapshot?.artifacts?.manifest;
    expect(manifest?.taskId).toBe(taskId);
    expect(manifest?.files.length ?? 0).toBeGreaterThan(0);

    if (manifest && snapshot?.artifacts) {
      const [firstFile] = manifest.files;
      const fullPath = path.join(
        snapshot.artifacts.rootPath,
        firstFile.path
      );
      expect(fs.existsSync(fullPath)).toBe(true);
      const content = fs.readFileSync(fullPath, "utf8");
      expect(content).toContain("Ensure artifact manifest is used");
    }
  });
});
