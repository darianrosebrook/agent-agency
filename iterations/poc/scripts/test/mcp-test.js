#!/usr/bin/env node

/**
 * MCP Server Test Script
 * Tests the minimal MCP server using proper stdio communication
 */

import { spawn } from "child_process";

console.log("🧪 Testing MCP Server with proper stdio protocol...\n");

// Start the MCP server
const serverProcess = spawn("npx", ["tsx", "src/mcp/minimal-server.ts"], {
  cwd: process.cwd(),
  stdio: ["pipe", "pipe", "inherit"], // pipe stdin/stdout, inherit stderr
});

let responseCount = 0;
const expectedResponses = 4; // initialize, tools/list, tools/call, resources/list

// Listen for responses from server
serverProcess.stdout.on("data", (data) => {
  const response = data.toString().trim();
  if (response) {
    console.log("📥 Response:", response);
    responseCount++;

    try {
      const parsed = JSON.parse(response);
      if (parsed.id === 1) {
        console.log("✅ Server initialized successfully");
        // Send tools/list after initialization
        sendToolsList();
      } else if (parsed.id === 2) {
        console.log("✅ Tools listed successfully");
        // Send tool call after listing tools
        sendToolCall();
      } else if (parsed.id === 3) {
        console.log("✅ Tool called successfully");
        // Send resources/list after tool call
        sendResourcesList();
      } else if (parsed.id === 4) {
        console.log("✅ Resources listed successfully");
        // Test complete
        setTimeout(() => {
          console.log(
            "\n🎉 All tests passed! MCP server is working correctly."
          );
          serverProcess.kill("SIGINT");
        }, 1000);
      }
    } catch (e) {
      console.log("⚠️  Non-JSON response:", response);
    }
  }
});

serverProcess.on("close", (code) => {
  console.log(`\n🏁 Server process exited with code ${code}`);
  if (responseCount >= expectedResponses) {
    console.log("✅ MCP server test completed successfully!");
  } else {
    console.log(
      `❌ Test incomplete - only received ${responseCount}/${expectedResponses} responses`
    );
  }
});

serverProcess.on("error", (error) => {
  console.error("❌ Failed to start server:", error);
  process.exit(1);
});

// Test functions
function sendInitialize() {
  const message = {
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: {
      protocolVersion: "2024-11-05",
      capabilities: {},
      clientInfo: {
        name: "mcp-test-client",
        version: "1.0.0",
      },
    },
  };
  console.log("📤 Sending initialize...");
  serverProcess.stdin.write(JSON.stringify(message) + "\n");
}

function sendToolsList() {
  const message = {
    jsonrpc: "2.0",
    id: 2,
    method: "tools/list",
    params: {},
  };
  console.log("📤 Sending tools/list...");
  serverProcess.stdin.write(JSON.stringify(message) + "\n");
}

function sendToolCall() {
  const message = {
    jsonrpc: "2.0",
    id: 3,
    method: "tools/call",
    params: {
      name: "hello_world",
      arguments: { name: "Agent Agency" },
    },
  };
  console.log("📤 Sending tools/call (hello_world)...");
  serverProcess.stdin.write(JSON.stringify(message) + "\n");
}

function sendResourcesList() {
  const message = {
    jsonrpc: "2.0",
    id: 4,
    method: "resources/list",
    params: {},
  };
  console.log("📤 Sending resources/list...");
  serverProcess.stdin.write(JSON.stringify(message) + "\n");
}

// Start the test sequence
console.log("🚀 Starting MCP server test sequence...");
sendInitialize();

// Timeout after 15 seconds
setTimeout(() => {
  console.log("\n⏰ Test timeout reached");
  serverProcess.kill("SIGINT");
}, 15000);





