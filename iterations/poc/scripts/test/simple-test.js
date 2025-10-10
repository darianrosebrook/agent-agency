#!/usr/bin/env node

import { spawn } from "child_process";

console.log("🧪 Simple MCP Server Test...\n");

// Start server process
const server = spawn("npx", ["tsx", "src/mcp/minimal-server.ts"], {
  cwd: process.cwd(),
  stdio: ["pipe", "pipe", "inherit"],
});

let responseBuffer = "";

// Listen for server output
server.stdout.on("data", (data) => {
  const output = data.toString();
  responseBuffer += output;
  console.log("📥 Server Response:", output.trim());
});

// Send initialize message after a short delay
setTimeout(() => {
  console.log("📤 Sending initialize message...");
  const initMsg =
    JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        protocolVersion: "2024-11-05",
        capabilities: {},
        clientInfo: { name: "test-client", version: "1.0.0" },
      },
    }) + "\n";

  server.stdin.write(initMsg);
}, 1000);

// Send tools/list after initialize response
setTimeout(() => {
  console.log("📤 Sending tools/list message...");
  const listMsg =
    JSON.stringify({
      jsonrpc: "2.0",
      id: 2,
      method: "tools/list",
      params: {},
    }) + "\n";

  server.stdin.write(listMsg);
}, 2000);

// Clean up after test
setTimeout(() => {
  console.log("\n✅ Test completed, shutting down...");
  server.kill("SIGINT");

  // Check results
  if (responseBuffer.includes("agent-agency-mcp-minimal")) {
    console.log("✅ Server initialized successfully");
  }
  if (responseBuffer.includes("hello_world")) {
    console.log("✅ Tools registered successfully");
  }
}, 3000);





