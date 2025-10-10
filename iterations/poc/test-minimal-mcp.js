#!/usr/bin/env node

/**
 * Test script for the minimal MCP server
 */

import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

console.log("🧪 Testing Minimal MCP Server...\n");

// Start the MCP server
const serverProcess = spawn("node", ["dist/minimal-server.js"], {
  cwd: path.join(__dirname),
  stdio: ["pipe", "pipe", "pipe"],
});

let serverOutput = "";
let serverReady = false;

// Collect server output
serverProcess.stdout.on("data", (data) => {
  const output = data.toString();
  console.log("📤 Server:", output.trim());
  serverOutput += output;

  if (output.includes("MCP Server connected and ready") && !serverReady) {
    serverReady = true;
    testServer();
  }
});

serverProcess.stderr.on("data", (data) => {
  console.log("⚠️  Server Error:", data.toString().trim());
});

// Test function
function testServer() {
  console.log("\n🔍 Server appears ready, sending test messages...\n");

  // Test 1: Initialize
  const initMessage = {
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: {
      protocolVersion: "2024-11-05",
      capabilities: {},
      clientInfo: {
        name: "test-client",
        version: "1.0.0",
      },
    },
  };

  serverProcess.stdin.write(JSON.stringify(initMessage) + "\n");

  // Test 2: List tools
  setTimeout(() => {
    const listToolsMessage = {
      jsonrpc: "2.0",
      id: 2,
      method: "tools/list",
      params: {},
    };

    console.log("📤 Sending tools/list request...");
    serverProcess.stdin.write(JSON.stringify(listToolsMessage) + "\n");

    // Test 3: Call hello_world tool
    setTimeout(() => {
      const callToolMessage = {
        jsonrpc: "2.0",
        id: 3,
        method: "tools/call",
        params: {
          name: "hello_world",
          arguments: { name: "Test User" },
        },
      };

      console.log("📤 Sending tools/call request...");
      serverProcess.stdin.write(JSON.stringify(callToolMessage) + "\n");

      // Test 4: List resources
      setTimeout(() => {
        const listResourcesMessage = {
          jsonrpc: "2.0",
          id: 4,
          method: "resources/list",
          params: {},
        };

        console.log("📤 Sending resources/list request...");
        serverProcess.stdin.write(JSON.stringify(listResourcesMessage) + "\n");

        // Test 5: Read resource
        setTimeout(() => {
          const readResourceMessage = {
            jsonrpc: "2.0",
            id: 5,
            method: "resources/read",
            params: {
              uri: "agent://status",
            },
          };

          console.log("📤 Sending resources/read request...");
          serverProcess.stdin.write(JSON.stringify(readResourceMessage) + "\n");

          // Exit after tests
          setTimeout(() => {
            console.log("\n✅ Tests completed, shutting down...");
            serverProcess.kill("SIGINT");
          }, 2000);
        }, 1000);
      }, 1000);
    }, 1000);
  }, 500);
}

// Handle process events
serverProcess.on("close", (code) => {
  console.log(`\n🏁 Server process exited with code ${code}`);

  if (serverOutput.includes("Agent Agency Minimal MCP Server")) {
    console.log("✅ Server started successfully");
  }

  if (serverOutput.includes("Available tools:")) {
    console.log("✅ Tools registered successfully");
  }

  if (serverOutput.includes("Available resources:")) {
    console.log("✅ Resources registered successfully");
  }

  console.log("\n🎉 Test completed!");
});

serverProcess.on("error", (error) => {
  console.error("❌ Failed to start server:", error);
  process.exit(1);
});

// Timeout after 10 seconds
setTimeout(() => {
  console.log("\n⏰ Test timeout reached, shutting down...");
  serverProcess.kill("SIGINT");
}, 10000);
