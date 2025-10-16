import { beforeEach, describe, expect, it } from "@jest/globals";
import {
  SecureTaskQueue,
  type SecureTaskQueueOptions,
  TaskQueue,
  type TaskQueueAuditRecord,
  type TaskQueueAuditSink,
} from "@/orchestrator/TaskQueue";
import {
  SecurityManager,
  type AuthCredentials,
} from "@/orchestrator/SecurityManager";
import type { AgentProfile, Task } from "@/types/arbiter-orchestration";

describe("SecureTaskQueue", () => {
  let taskQueue: TaskQueue;
  let securityManager: SecurityManager;
  let auditRecords: TaskQueueAuditRecord[];
  let auditSink: TaskQueueAuditSink;
  let credentials: AuthCredentials;
  let baseTask: Task;
  let agentProfile: AgentProfile;

  const createSecureQueue = (
    options: SecureTaskQueueOptions = {}
  ): SecureTaskQueue => new SecureTaskQueue(taskQueue, securityManager, options);

  beforeEach(() => {
    taskQueue = new TaskQueue();
    auditRecords = [];
    auditSink = {
      record: jest.fn((record: TaskQueueAuditRecord) => {
        auditRecords.push(record);
      }),
    } as jest.Mocked<TaskQueueAuditSink>;

    securityManager = new SecurityManager({
      enabled: true,
      auditLogging: true,
      trustedAgents: [],
      adminAgents: [],
      maxSessionsPerAgent: 50,
      policies: {
        maxTaskDescriptionLength: 5000,
        maxMetadataSize: 20000,
        allowedTaskTypes: {
          "*": ["code-editing", "analysis", "research", "general"],
        },
        suspiciousPatterns: [],
      },
    });

    agentProfile = {
      id: "unit-test-agent",
      name: "Unit Test Agent",
      modelFamily: "gpt-4" as any,
      capabilities: {
        taskTypes: ["code-editing", "documentation"],
        languages: ["TypeScript"],
        specializations: [],
      },
      performanceHistory: {
        successRate: 0.9,
        averageQuality: 0.8,
        averageLatency: 1200,
        taskCount: 10,
      },
      currentLoad: {
        activeTasks: 0,
        queuedTasks: 0,
        utilizationPercent: 10,
      },
      registeredAt: new Date().toISOString(),
      lastActiveAt: new Date().toISOString(),
    };

    securityManager.registerAgent(agentProfile);

    credentials = {
      agentId: agentProfile.id,
      token: "valid-token-123456",
      metadata: {
        ipAddress: "127.0.0.1",
        userAgent: "secure-task-queue-tests",
        source: "test",
      },
    };

    baseTask = {
      id: "task-1",
      description: "Implement secure task queue behaviour",
      type: "code-editing",
      priority: 5,
      timeoutMs: 30000,
      budget: {
        maxFiles: 5,
        maxLoc: 200,
      },
      requiredCapabilities: {
        taskTypes: ["code-editing"],
      } as any,
      createdAt: new Date(),
      metadata: {
        requestId: "req-123",
        labels: ["unit-test"],
      },
      attempts: 0,
      maxAttempts: 3,
    };
  });

  it("enqueues task when credentials are valid", async () => {
    const secureQueue = createSecureQueue({ auditSink });

    await secureQueue.enqueue(baseTask, credentials);

    expect(taskQueue.size()).toBe(1);
    const queuedTask = taskQueue.peek();
    expect(queuedTask?.metadata.security.submittedBy).toBe(
      agentProfile.id
    );
    expect(auditRecords).toHaveLength(1);
    expect(auditRecords[0].action).toBe("enqueue");
  });

  it("enforces metadata size and description limits", async () => {
    const secureQueue = createSecureQueue({
      auditSink,
      metadataLimitBytes: 1024,
    });

    const oversizedMetadataTask: Task = {
      ...baseTask,
      metadata: {
        data: "x".repeat(4096),
      },
    };

    await expect(
      secureQueue.enqueue(oversizedMetadataTask, credentials)
    ).rejects.toThrow("Task metadata exceeds allowed size");
    expect(
      auditRecords.find((record) => record.action === "reject")
    ).toBeTruthy();

    const longDescriptionTask: Task = {
      ...baseTask,
      description: "y".repeat(6000),
    };

    const secureQueueDescription = createSecureQueue({
      auditSink,
      metadataLimitBytes: 20000,
    });

    await expect(
      secureQueueDescription.enqueue(longDescriptionTask, credentials)
    ).rejects.toThrow("Task description exceeds policy limit");
    expect(
      auditRecords.filter((record) => record.action === "reject")
    ).toHaveLength(2);
  });

  it("rejects task types not permitted for the agent", async () => {
    const secureQueue = createSecureQueue({
      auditSink,
      allowedTaskTypes: {
        "*": ["code-editing"],
      },
    });

    const disallowedTask: Task = {
      ...baseTask,
      id: "task-disallowed",
      type: "analysis",
    };

    await expect(
      secureQueue.enqueue(disallowedTask, credentials)
    ).rejects.toThrow("Task type analysis not permitted");
    expect(
      auditRecords.some((record) => record.action === "reject")
    ).toBe(true);
  });

  it("supports TaskQueue.enqueueWithCredentials helper", async () => {
    await taskQueue.enqueueWithCredentials(
      baseTask,
      credentials,
      securityManager,
      { auditSink }
    );

    expect(taskQueue.size()).toBe(1);
    expect(auditRecords).toHaveLength(1);

    const queuedTask = taskQueue.peek();
    expect(queuedTask?.metadata.security).toMatchObject({
      submittedBy: agentProfile.id,
      securityLevel: expect.anything(),
    });
  });

  it("propagates rate-limit violations", async () => {
    const secureQueue = createSecureQueue({ auditSink });

    // Exhaust the submitTask rate limit quickly
    const operations = Array.from({ length: 12 }).map((_, index) =>
      secureQueue.enqueue(
        { ...baseTask, id: `rate-limit-${index}` },
        credentials
      )
    );

    await expect(Promise.all(operations)).rejects.toThrow("Rate limit exceeded");
    expect(auditRecords.filter((r) => r.action === "enqueue").length).toBeGreaterThan(0);
    expect(
      auditRecords.filter((r) => r.action === "rate_limited").length
    ).toBeGreaterThan(0);
  });

  it("logs audit entries even when queue rejects task", async () => {
    const secureQueue = createSecureQueue({ auditSink });

    const invalidTask: Task = {
      ...baseTask,
      description: "z".repeat(6000),
    };

    await expect(
      secureQueue.enqueue(invalidTask, credentials)
    ).rejects.toThrow();

    expect(
      auditRecords.filter((record) => record.action === "reject")
    ).toHaveLength(1);
  });
});
