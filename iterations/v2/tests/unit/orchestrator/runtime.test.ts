import { describe, expect, it, beforeAll, afterAll } from "@jest/globals";
import fs from "fs";
import os from "os";
import path from "path";
import { ArbiterRuntime } from "@/orchestrator/runtime/ArbiterRuntime";

describe("ArbiterRuntime", () => {
  let tempDir: string;
  let runtime: ArbiterRuntime;

  beforeAll(async () => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "arbiter-runtime-"));
    runtime = new ArbiterRuntime({ outputDir: tempDir });
    await runtime.start();
  });

  afterAll(async () => {
    await runtime.stop();
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  it("executes tasks and produces tangible artifacts", async () => {
    const { taskId } = await runtime.submitTask({
      description: "Create a hello world summary file",
      metadata: { framework: "jest" },
    });

    await runtime.waitForCompletion(taskId);

    const snapshot = runtime.getTaskSnapshot(taskId);
    expect(snapshot).not.toBeNull();
    expect(snapshot?.state).toBe("completed");
    expect(snapshot?.outputPath).toBeDefined();
    if (snapshot?.outputPath) {
      expect(fs.existsSync(snapshot.outputPath)).toBe(true);
      const contents = fs.readFileSync(snapshot.outputPath, "utf8");
      expect(contents).toContain("Create a hello world summary file");
    }
  });
});
