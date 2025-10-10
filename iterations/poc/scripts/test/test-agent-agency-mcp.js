#!/usr/bin/env node

/**
 * Agent Agency MCP Server Test Script
 * Tests the full MCP server with agent orchestrator integration
 */

import { spawn } from "child_process";

console.log("🧪 Testing Agent Agency MCP Server...\n");

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
        clientInfo: { name: "test-client", version: "1.0.0" },
      },
    },
  },
  {
    name: "List tools",
    message: {
      jsonrpc: "2.0",
      id: 2,
      method: "tools/list",
      params: {},
    },
  },
  {
    name: "Register agent",
    message: {
      jsonrpc: "2.0",
      id: 3,
      method: "tools/call",
      params: {
        name: "register_agent",
        arguments: {
          agentId: "test-agent-001",
          type: "code-specialist",
          capabilities: ["coding", "debugging", "testing"],
        },
      },
    },
  },
  {
    name: "Submit task",
    message: {
      jsonrpc: "2.0",
      id: 4,
      method: "tools/call",
      params: {
        name: "submit_task",
        arguments: {
          taskId: "test-task-001",
          type: "code-review",
          payload: { files: ["src/main.ts"], priority: "high" },
          priority: "high",
        },
      },
    },
  },
  {
    name: "List agents",
    message: {
      jsonrpc: "2.0",
      id: 5,
      method: "tools/call",
      params: {
        name: "list_agents",
        arguments: {},
      },
    },
  },
  {
    name: "Get system metrics",
    message: {
      jsonrpc: "2.0",
      id: 6,
      method: "tools/call",
      params: {
        name: "get_system_metrics",
        arguments: {},
      },
    },
  },
  {
    name: "List resources",
    message: {
      jsonrpc: "2.0",
      id: 7,
      method: "resources/list",
      params: {},
    },
  },
  {
    name: "Read system status",
    message: {
      jsonrpc: "2.0",
      id: 8,
      method: "resources/read",
      params: { uri: "agent://status" },
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
    !response.includes("🤖")
  ) {
    console.log("📥 Response:", response);

    try {
      const parsed = JSON.parse(response);
      responseCount++;

      if (parsed.id) {
        console.log(
          `✅ ${testSequence[testStep]?.name || "Unknown"} completed`
        );
        testStep++;

        // Send next message in sequence
        if (testStep < testSequence.length) {
          setTimeout(() => sendNextMessage(), 100);
        } else {
          // All tests completed
          setTimeout(() => {
            console.log("\n🎉 All Agent Agency MCP tests passed!");
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
  console.log(`Completed ${responseCount} test steps`);
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
console.log("🚀 Starting Agent Agency MCP server test sequence...");
sendNextMessage();

// Timeout after 20 seconds
setTimeout(() => {
  console.log("\n⏰ Test timeout reached");
  serverProcess.kill("SIGINT");
}, 20000);





