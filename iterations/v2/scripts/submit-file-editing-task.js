#!/usr/bin/env node

/**
 * File Editing Task Submission Script
 *
 * This script submits properly formatted file editing tasks to the v2 arbiter
 * to enable actual file modifications instead of just documentation generation.
 *
 * @author @darianrosebrook
 */

import { spawn } from "child_process";
import fs from "fs/promises";
import path from "path";

const OBSERVER_URL = "http://127.0.0.1:4387";

/**
 * Submit a file editing task to the arbiter
 */
async function submitFileEditingTask(taskConfig) {
  const {
    description,
    operations,
    projectRoot,
    priority = "high",
  } = taskConfig;

  const taskPayload = {
    description,
    type: "file_editing", // Add top-level type
    priority: priority === "high" ? 8 : priority === "medium" ? 5 : 3,
    timeoutMs: 120000, // 2 minutes
    budget: {
      maxFiles: 50,
      maxLoc: 2000,
    },
    task: {
      // Add full task specification
      type: "file_editing",
      payload: {
        operations,
        projectRoot: projectRoot || process.cwd(),
        timeout: 120000,
      },
    },
    metadata: {
      task: {
        type: "file_editing",
        payload: {
          operations,
          projectRoot: projectRoot || process.cwd(),
          timeout: 120000,
        },
      },
    },
  };

  try {
    const response = await fetch(`${OBSERVER_URL}/observer/tasks`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(taskPayload),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const result = await response.json();
    console.log("✅ Task submitted successfully:", result);
    return result;
  } catch (error) {
    console.error("❌ Failed to submit task:", error.message);
    throw error;
  }
}

/**
 * Monitor task progress
 */
async function monitorTask(taskId) {
  console.log(`\n🔍 Monitoring task: ${taskId}`);

  let completed = false;
  let attempts = 0;
  const maxAttempts = 60; // 5 minutes max

  while (!completed && attempts < maxAttempts) {
    try {
      const response = await fetch(`${OBSERVER_URL}/observer/tasks/${taskId}`);
      const task = await response.json();

      console.log(
        `📊 Status: ${task.state}, Progress: ${
          task.progress?.length || 0
        } steps`
      );

      if (task.state === "completed") {
        completed = true;
        console.log("✅ Task completed successfully!");

        // Show verification results
        if (task.verification) {
          console.log(
            `🔍 Verification: ${task.verification.verdict} (confidence: ${task.verification.confidence})`
          );
        }

        return task;
      } else if (task.state === "failed") {
        console.log("❌ Task failed");
        return task;
      }

      // Wait 5 seconds before next check
      await new Promise((resolve) => setTimeout(resolve, 5000));
      attempts++;
    } catch (error) {
      console.error("❌ Error monitoring task:", error.message);
      attempts++;
    }
  }

  if (!completed) {
    console.log("⏰ Task monitoring timeout");
  }
}

/**
 * Fix TypeScript compilation errors in playground
 */
async function fixTypeScriptErrors() {
  console.log("🔧 Fixing TypeScript compilation errors...");

  const operations = [
    {
      type: "file_search_replace",
      file_path: "playground/broken-types.ts",
      old_string: "const userId: string = 123;",
      new_string: "const userId: number = 123;",
    },
    {
      type: "file_search_replace",
      file_path: "playground/broken-types.ts",
      old_string: "// Missing import\nconst result = fetchUserData(userId);",
      new_string:
        "// Import added\nimport { fetchUserData } from './utils';\nconst result = fetchUserData(userId);",
    },
    {
      type: "file_search_replace",
      file_path: "playground/broken-types.ts",
      old_string: "function calculateTotal(items: number[]): string {",
      new_string: "function calculateTotal(items: number[]): number {",
    },
    {
      type: "file_search_replace",
      file_path: "playground/broken-types.ts",
      old_string:
        'const unusedVar = "this should be removed or prefixed with underscore";',
      new_string:
        'const _unusedVar = "this should be removed or prefixed with underscore";',
    },
  ];

  const result = await submitFileEditingTask({
    description:
      "Fix TypeScript compilation errors in playground/broken-types.ts",
    operations,
    projectRoot: process.cwd(),
  });

  return await monitorTask(result.taskId);
}

/**
 * Fix Rust compilation errors in playground
 */
async function fixRustErrors() {
  console.log("🔧 Fixing Rust compilation errors...");

  const operations = [
    {
      type: "file_search_replace",
      file_path: "playground/broken-rust.rs",
      old_string: "#[derive(Debug)]\npub struct User {",
      new_string:
        "#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]\npub struct User {",
    },
    {
      type: "file_search_replace",
      file_path: "playground/broken-rust.rs",
      old_string:
        "// Duplicate struct definition (should be removed)\n#[derive(Debug)]\npub struct User {",
      new_string: "// Duplicate struct definition removed",
    },
    {
      type: "file_search_replace",
      file_path: "playground/broken-rust.rs",
      old_string: "let user_id: String = 123;",
      new_string: "let user_id: u32 = 123;",
    },
    {
      type: "file_search_replace",
      file_path: "playground/broken-rust.rs",
      old_string: "fn calculate_total(items: Vec<u32>) -> String {",
      new_string: "fn calculate_total(items: Vec<u32>) -> u32 {",
    },
  ];

  const result = await submitFileEditingTask({
    description: "Fix Rust compilation errors in playground/broken-rust.rs",
    operations,
    projectRoot: process.cwd(),
  });

  return await monitorTask(result.taskId);
}

/**
 * Fix Python errors in playground
 */
async function fixPythonErrors() {
  console.log("🔧 Fixing Python errors...");

  const operations = [
    {
      type: "file_search_replace",
      file_path: "playground/broken-python.py",
      old_string:
        "def calculate_total(items: List[int]) -> str:  # Should return int, not str",
      new_string:
        "def calculate_total(items: List[int]) -> int:  # Fixed return type",
    },
    {
      type: "file_search_replace",
      file_path: "playground/broken-python.py",
      old_string: "userAge = 25  # Should be user_age",
      new_string: "user_age = 25  # Fixed naming convention",
    },
    {
      type: "file_search_replace",
      file_path: "playground/broken-python.py",
      old_string:
        'unused_var = "this should be removed or prefixed with underscore"',
      new_string:
        '_unused_var = "this should be removed or prefixed with underscore"',
    },
    {
      type: "file_search_replace",
      file_path: "playground/broken-python.py",
      old_string:
        'def broken_indentation():\nprint("This has wrong indentation")',
      new_string:
        'def broken_indentation():\n    print("This has wrong indentation")',
    },
    {
      type: "file_search_replace",
      file_path: "playground/broken-python.py",
      old_string:
        "def function_without_return(x: int) -> int:\n    x * 2  # Should be: return x * 2",
      new_string:
        "def function_without_return(x: int) -> int:\n    return x * 2  # Fixed return statement",
    },
  ];

  const result = await submitFileEditingTask({
    description: "Fix Python errors in playground/broken-python.py",
    operations,
    projectRoot: process.cwd(),
  });

  return await monitorTask(result.taskId);
}

/**
 * Main execution
 */
async function main() {
  const command = process.argv[2];

  try {
    switch (command) {
      case "typescript":
        await fixTypeScriptErrors();
        break;
      case "rust":
        await fixRustErrors();
        break;
      case "python":
        await fixPythonErrors();
        break;
      case "all":
        console.log("🚀 Fixing all playground files...");
        await fixTypeScriptErrors();
        await fixRustErrors();
        await fixPythonErrors();
        break;
      default:
        console.log(
          "Usage: node submit-file-editing-task.js [typescript|rust|python|all]"
        );
        process.exit(1);
    }

    console.log("\n✅ All tasks completed!");
  } catch (error) {
    console.error("❌ Script failed:", error.message);
    process.exit(1);
  }
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export { submitFileEditingTask, monitorTask };
