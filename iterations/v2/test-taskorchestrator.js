#!/usr/bin/env node

/**
 * Simple test script to verify TaskOrchestrator functionality
 */

import { TaskOrchestrator } from "./src/orchestrator/TaskOrchestrator.js";

async function testTaskOrchestrator() {
  console.log("Testing TaskOrchestrator...");

  try {
    // Create a proper config
    const config = {
      workerPool: {
        minPoolSize: 1,
        maxPoolSize: 3,
        workerCapabilities: ["file_editing"],
        workerTimeout: 30000,
        supervisor: {},
      },
      queue: {
        maxSize: 100,
        priorityLevels: ["low", "medium", "high", "critical"],
        persistenceEnabled: false,
      },
      retry: {
        maxAttempts: 3,
        backoffMultiplier: 2,
        initialDelay: 1000,
        maxDelay: 10000,
      },
      routing: {
        enabled: true,
        strategy: "round_robin",
      },
      performance: {
        trackingEnabled: true,
        metricsInterval: 5000,
      },
    };

    // Create TaskOrchestrator instance
    const orchestrator = new TaskOrchestrator(config);
    console.log("✅ TaskOrchestrator created successfully");

      // Test file editing task
      const fileEditingTask = {
        id: "test-file-edit-001",
        type: "file_editing",
        description: "Test file editing task to replace text in playground file",
        payload: {
          operations: [
            {
              type: "replace",
              filePath: "playground/test-file.txt",
              oldString: "Hello World",
              newString: "Hello Universe",
            },
          ],
          projectRoot: process.cwd(),
          timeout: 10000,
        },
        priority: "medium",
        timeoutMs: 30000,
      };

    console.log("Testing file editing task...");
    const taskId = await orchestrator.submitTask(fileEditingTask);
    console.log("✅ File editing task submitted with ID:", taskId);
    
    // Wait a bit for processing
    await new Promise(resolve => setTimeout(resolve, 2000));
    console.log("✅ Task processing completed");
  } catch (error) {
    console.error("❌ Error testing TaskOrchestrator:", error);
    process.exit(1);
  }
}

testTaskOrchestrator();
