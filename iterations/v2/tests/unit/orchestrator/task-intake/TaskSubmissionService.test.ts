/**
 * @fileoverview Tests for processTaskSubmission intake workflow.
 */

import {
  processTaskSubmission,
  TaskSubmissionDependencies,
} from "../../../../src/orchestrator/intake/TaskSubmissionService.js";
import { TaskIntakeProcessor } from "../../../../src/orchestrator/intake/TaskIntakeProcessor.js";

const intakeProcessor = new TaskIntakeProcessor({
  chunkSizeBytes: 128,
});

function buildDeps(
  overrides: Partial<TaskSubmissionDependencies> = {}
): TaskSubmissionDependencies {
  return {
    intakeProcessor,
    generateTaskId: () => "task-seq-1",
    ...overrides,
  };
}

describe("processTaskSubmission", () => {
  it("rejects invalid tasks without invoking runtime", async () => {
    const runtimeMock = {
      submitTask: jest.fn(),
    };

    const result = await processTaskSubmission(
      {
        id: "",
        type: "",
        description: "",
      },
      buildDeps({ runtime: runtimeMock })
    );

    expect(result.status).toBe("rejected");
    expect(runtimeMock.submitTask).not.toHaveBeenCalled();
    expect(result.message).toContain("MISSING_REQUIRED_FIELD");
  });

  it("forwards sanitized task payloads to runtime with intake telemetry", async () => {
    const runtimeMock = {
      submitTask: jest.fn().mockResolvedValue({
        taskId: "runtime-task",
        queued: true,
      }),
    };

    const result = await processTaskSubmission(
      {
        id: "client-1",
        type: "analysis",
        description: "Analyze arbiter intake behavior",
        priority: 4,
        timeoutMs: 60000,
        attempts: 0,
        maxAttempts: 3,
        requiredCapabilities: {},
        budget: { maxFiles: 5, maxLoc: 120 },
        metadata: { surface: "cli" },
        createdAt: new Date(),
      },
      buildDeps({ runtime: runtimeMock })
    );

    expect(result.status).toBe("accepted");
    expect(result.taskId).toBe("task-seq-1");
    expect(runtimeMock.submitTask).toHaveBeenCalledWith(
      expect.objectContaining({
        description: "Analyze arbiter intake behavior",
        task: expect.objectContaining({
          id: "task-seq-1",
          metadata: expect.objectContaining({
            originalTaskId: "client-1",
            intake: expect.objectContaining({
              chunkCount: expect.any(Number),
              rawSizeBytes: expect.any(Number),
            }),
          }),
        }),
      })
    );
  });
});
