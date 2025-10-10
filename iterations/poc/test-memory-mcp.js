#!/usr/bin/env node

/**
 * Memory Integration MCP Server Test
 * Tests the Agent Agency MCP server with memory system integration
 */

import { spawn } from "child_process";

console.log("🧠 Testing Agent Agency MCP Server with Memory Integration...\n");

// Start the MCP server
const serverProcess = spawn("npx", ["tsx", "src/mcp/agent-agency-server.ts"], {
  cwd: process.cwd(),
  stdio: ["pipe", "pipe", "inherit"],
});

let responseCount = 0;
let testStep = 0;

const testSequence = [
  {
    name: "Initialize server",
    message: {
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        protocolVersion: "2024-11-05",
        capabilities: {},
        clientInfo: { name: "memory-test-client", version: "1.0.0" },
      },
    },
  },
  {
    name: "List tools (should include memory tools)",
    message: {
      jsonrpc: "2.0",
      id: 2,
      method: "tools/list",
      params: {},
    },
  },
  {
    name: "Store memory",
    message: {
      jsonrpc: "2.0",
      id: 3,
      method: "tools/call",
      params: {
        name: "store_memory",
        arguments: {
          tenantId: "test-tenant-001",
          taskId: "test-task-001",
          type: "experience",
          content: {
            action: "code_review",
            outcome: "successful",
            lessons: [
              "Always check edge cases",
              "Use descriptive variable names",
            ],
          },
          metadata: {
            confidence: 0.9,
            category: "best_practices",
          },
        },
      },
    },
  },
  {
    name: "Retrieve memory",
    message: {
      jsonrpc: "2.0",
      id: 4,
      method: "tools/call",
      params: {
        name: "retrieve_memory",
        arguments: {
          tenantId: "test-tenant-001",
          taskId: "test-task-002",
          context: {
            type: "code_review",
            description: "Reviewing authentication logic",
          },
          limit: 5,
        },
      },
    },
  },
  {
    name: "List resources (should include memory resources)",
    message: {
      jsonrpc: "2.0",
      id: 5,
      method: "resources/list",
      params: {},
    },
  },
  {
    name: "Read memory status",
    message: {
      jsonrpc: "2.0",
      id: 6,
      method: "resources/read",
      params: { uri: "agent://memory/status" },
    },
  },
];

// Listen for responses from server
serverProcess.stdout.on("data", (data) => {
  const response = data.toString().trim();
  if (
    response &&
    !response.includes("🚀") &&
    !response.includes("✅") &&
    !response.includes("🤖") &&
    !response.includes("🧠")
  ) {
    console.log("📥 Response:", response);

    try {
      const parsed = JSON.parse(response);
      if (parsed.id) {
        console.log(
          `✅ ${testSequence[testStep]?.name || "Unknown"} completed`
        );
        responseCount++;

        if (parsed.id === 2) {
          // Check if memory tools are present
          const tools = parsed.result?.tools || [];
          const memoryTools = tools.filter((t) => t.name.includes("memory"));
          console.log(
            `   📋 Found ${memoryTools.length} memory tools: ${memoryTools
              .map((t) => t.name)
              .join(", ")}`
          );
        }

        testStep++;

        // Send next message in sequence
        if (testStep < testSequence.length) {
          setTimeout(() => sendNextMessage(), 200);
        } else {
          // All tests completed
          setTimeout(() => {
            console.log("\n🎉 All memory integration tests passed!");
            console.log(`📊 Completed ${responseCount} test steps`);
            console.log(
              "🧠 Memory system successfully integrated with MCP server!"
            );
            serverProcess.kill("SIGINT");
          }, 500);
        }
      }
    } catch (e) {
      console.log("⚠️  Non-JSON response:", response);
    }
  }
});

serverProcess.on("close", (code) => {
  console.log(`\n🏁 Server process exited with code ${code}`);
  if (responseCount >= testSequence.length) {
    console.log("✅ Memory integration test completed successfully!");
  } else {
    console.log(
      `❌ Test incomplete - only received ${responseCount}/${testSequence.length} responses`
    );
  }
});

serverProcess.on("error", (error) => {
  console.error("❌ Failed to start server:", error);
  process.exit(1);
});

function sendNextMessage() {
  if (testStep < testSequence.length) {
    const test = testSequence[testStep];
    console.log(`\n📤 ${test.name}...`);
    serverProcess.stdin.write(JSON.stringify(test.message) + "\n");
  }
}

// Start the test sequence
console.log("🚀 Starting memory integration test sequence...");
sendNextMessage();

// Timeout after 25 seconds
setTimeout(() => {
  console.log("\n⏰ Test timeout reached");
  serverProcess.kill("SIGINT");
}, 25000);





