import { describe, expect, it, beforeAll, afterAll } from "@jest/globals";
import fs from "fs";
import os from "os";
import path from "path";
import { ObserverStoreImpl } from "@/observer/persistence/ObserverStoreImpl";
import { ObserverConfig } from "@/observer/types";

let tempDir: string;

function buildConfig(): ObserverConfig {
  return {
    bind: "127.0.0.1",
    port: 4387,
    socketPath: null,
    authToken: undefined,
    allowedOrigins: new Set(["null", "file://"]),
    dataDir: tempDir,
    maxClients: 4,
    flushIntervalMs: 20,
    heartbeatIntervalMs: 20000,
    maxQueueSize: 100,
    rotateMB: 4,
    retentionDays: 7,
    sampleRates: { "*": 1 },
    redactionRules: [{ name: "token", pattern: /sk-[A-Za-z0-9]{10,}/g }],
    privacyMode: "standard",
  };
}

describe("ObserverStoreImpl", () => {
  beforeAll(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "observer-store-"));
  });

  afterAll(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  it("records events and exposes them via listEvents", async () => {
    const store = new ObserverStoreImpl(buildConfig());

    store.recordEvent({
      id: "event-1",
      type: "task.submitted",
      severity: "info",
      source: "test",
      taskId: "task-1",
      timestamp: new Date().toISOString(),
      metadata: { note: "submitted sk-SECRET1234567890" },
    });

    await new Promise((resolve) => setTimeout(resolve, 50));
    const { events } = await store.listEvents({ limit: 10 });
    expect(events).toHaveLength(1);
    expect(events[0].metadata?.note).toContain("[REDACTED:token]");

    const status = store.getStatus();
    expect(status.queueDepth).toBeGreaterThanOrEqual(0);

    await store.shutdown();
  });

  it("tracks reasoning chain-of-thought entries", async () => {
    const store = new ObserverStoreImpl(buildConfig());
    store.recordChainOfThought({
      id: "cot-1",
      taskId: "task-2",
      phase: "analysis",
      timestamp: new Date().toISOString(),
      content: "Analyzing sk-SECRET1234567890",
    });

    await new Promise((resolve) => setTimeout(resolve, 50));
    const progress = store.getProgress();
    expect(progress.reasoningSteps.analyses).toBeGreaterThanOrEqual(1);

    const cot = await store.listChainOfThought({ taskId: "task-2", limit: 5 });
    expect(cot.entries[0].redacted).toBe(true);
    expect(cot.entries[0].content).toContain("[REDACTED:token]");

    await store.shutdown();
  });
});
