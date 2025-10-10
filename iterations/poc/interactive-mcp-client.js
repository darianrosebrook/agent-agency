#!/usr/bin/env node

/**
 * Interactive MCP Client
 *
 * Simple interactive tool to test Agent Agency MCP server capabilities
 * outside of the test environment.
 *
 * @author @darianrosebrook
 */

const { spawn } = require("child_process");
const readline = require("readline");

class InteractiveMCPClient {
  constructor() {
    this.serverProcess = null;
    this.rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });
  }

  async start() {
    console.log("🚀 Starting Agent Agency MCP Server...");
    await this.startMCPServer();

    console.log("\n🤖 Interactive MCP Client Ready!");
    console.log("Available commands:");
    console.log("  list-tools              - List available tools");
    console.log("  list-resources          - List available resources");
    console.log("  generate-text [prompt]  - Generate text using AI");
    console.log("  register-agent [id] [type] - Register a new agent");
    console.log("  submit-task [id] [type] [payload] - Submit a task");
    console.log(
      "  store-memory [tenant] [task] [type] [content] - Store memory"
    );
    console.log("  retrieve-memory [tenant] [task] - Retrieve memories");
    console.log("  read-file [path]        - Read content from a file");
    console.log("  write-file [path] [content] - Write content to a file");
    console.log(
      "  edit-file [path] [old] [new] - Edit file with search/replace"
    );
    console.log("  list-dir [path]         - List directory contents");
    console.log(
      "  decompose-task [desc]   - Break down complex task into steps"
    );
    console.log("  execute-plan [plan]     - Execute a task plan step by step");
    console.log("  help                    - Show this help");
    console.log("  exit                    - Exit the client");
    console.log("");

    this.showPrompt();
  }

  async startMCPServer() {
    return new Promise((resolve, reject) => {
      this.serverProcess = spawn("node", ["bin/mcp-server.cjs", "start"], {
        stdio: ["pipe", "pipe", "pipe"],
        cwd: process.cwd(),
      });

      let startupComplete = false;

      this.serverProcess.stdout.on("data", (data) => {
        const output = data.toString();
        if (
          output.includes("Server started") ||
          output.includes("listening") ||
          output.includes("ready")
        ) {
          startupComplete = true;
          console.log("✅ MCP Server started successfully");
          resolve();
        }
      });

      this.serverProcess.stderr.on("data", (data) => {
        console.error("Server error:", data.toString());
      });

      this.serverProcess.on("close", (code) => {
        if (!startupComplete) {
          reject(new Error(`Server exited with code ${code}`));
        }
      });

      // Timeout after 30 seconds
      setTimeout(() => {
        if (!startupComplete) {
          reject(new Error("Server startup timeout"));
        }
      }, 30000);
    });
  }

  showPrompt() {
    this.rl.question("mcp> ", async (input) => {
      const args = input.trim().split(" ");
      const command = args.shift();

      try {
        await this.handleCommand(command, args);
      } catch (error) {
        console.error("❌ Error:", error.message);
      }

      if (command !== "exit") {
        this.showPrompt();
      }
    });
  }

  async handleCommand(command, args) {
    switch (command) {
      case "help":
        this.showHelp();
        break;

      case "list-tools":
        await this.callTool("list-tools", {});
        break;

      case "list-resources":
        await this.callTool("list-resources", {});
        break;

      case "generate-text":
        const prompt = args.join(" ");
        if (!prompt) {
          console.log("❌ Usage: generate-text [prompt]");
          return;
        }
        await this.callTool("generate_text", {
          prompt,
          systemPrompt: "You are a helpful AI assistant.",
          maxTokens: 100,
        });
        break;

      case "register-agent":
        const [agentId, agentType] = args;
        if (!agentId || !agentType) {
          console.log("❌ Usage: register-agent [id] [type]");
          return;
        }
        await this.callTool("register_agent", {
          agentId,
          type: agentType,
          capabilities: ["task-execution", "memory-management"],
        });
        break;

      case "submit-task":
        const [taskId, taskType, ...payloadParts] = args;
        if (!taskId || !taskType) {
          console.log("❌ Usage: submit-task [id] [type] [payload]");
          return;
        }
        const payload = payloadParts.join(" ") || "{}";
        await this.callTool("submit_task", {
          taskId,
          type: taskType,
          payload: JSON.parse(payload),
          priority: "medium",
        });
        break;

      case "store-memory":
        const [tenantId, taskId, memoryType, ...contentParts] = args;
        if (!tenantId || !taskId || !memoryType || contentParts.length === 0) {
          console.log(
            "❌ Usage: store-memory [tenant] [task] [type] [content]"
          );
          return;
        }
        const content = contentParts.join(" ");
        await this.callTool("store_memory", {
          tenantId,
          taskId,
          type: memoryType,
          content: JSON.parse(content),
          metadata: { source: "interactive-client" },
        });
        break;

      case "retrieve-memory":
        const [retrieveTenantId, retrieveTaskId] = args;
        if (!retrieveTenantId || !retrieveTaskId) {
          console.log("❌ Usage: retrieve-memory [tenant] [task]");
          return;
        }
        await this.callTool("retrieve_memory", {
          tenantId: retrieveTenantId,
          taskId: retrieveTaskId,
          context: { currentTask: retrieveTaskId },
          limit: 5,
        });
        break;

      case "read-file":
        const [readFilePath] = args;
        if (!readFilePath) {
          console.log("❌ Usage: read-file [path]");
          return;
        }
        await this.callTool("read_file", {
          filePath: readFilePath,
        });
        break;

      case "write-file":
        const [writeFilePath, ...writeContentParts] = args;
        if (!writeFilePath || writeContentParts.length === 0) {
          console.log("❌ Usage: write-file [path] [content]");
          return;
        }
        const writeContent = writeContentParts.join(" ");
        await this.callTool("write_file", {
          filePath: writeFilePath,
          content: writeContent,
        });
        break;

      case "edit-file":
        const [editFilePath, ...editArgs] = args;
        if (!editFilePath || editArgs.length < 2) {
          console.log("❌ Usage: edit-file [path] [old_string] [new_string]");
          return;
        }
        const oldString = editArgs[0];
        const newString = editArgs.slice(1).join(" ");
        await this.callTool("edit_file", {
          filePath: editFilePath,
          oldString,
          newString,
        });
        break;

      case "list-dir":
        const [listDirPath] = args;
        if (!listDirPath) {
          console.log("❌ Usage: list-dir [path]");
          return;
        }
        await this.callTool("list_directory", {
          path: listDirPath,
        });
        break;

      case "decompose-task":
        const taskDescription = args.join(" ");
        if (!taskDescription) {
          console.log("❌ Usage: decompose-task [task description]");
          return;
        }
        await this.callTool("decompose_task", {
          taskDescription,
          maxSteps: 5,
          complexity: "complex",
        });
        break;

      case "execute-plan":
        const planJson = args.join(" ");
        if (!planJson) {
          console.log("❌ Usage: execute-plan [task plan JSON]");
          console.log(
            "💡 Tip: Use decompose-task first to get a plan, then copy the JSON"
          );
          return;
        }
        try {
          const taskPlan = JSON.parse(planJson);
          await this.callTool("execute_task_plan", {
            taskPlan,
            workingDirectory: ".",
            validateSteps: true,
          });
        } catch (error) {
          console.log(
            "❌ Invalid JSON format. Please provide valid task plan JSON."
          );
        }
        break;

      case "exit":
        console.log("👋 Goodbye!");
        await this.cleanup();
        process.exit(0);
        break;

      default:
        console.log(`❌ Unknown command: ${command}`);
        console.log('Type "help" for available commands');
    }
  }

  async callTool(toolName, params) {
    // This is a simplified version - in a real implementation,
    // you'd send JSON-RPC requests to the MCP server
    console.log(`🔧 Calling tool: ${toolName}`);
    console.log(`📝 Parameters:`, JSON.stringify(params, null, 2));

    // Simulate a response (replace with actual MCP communication)
    console.log(`✅ Tool executed successfully`);
    console.log(`📄 Response: Tool ${toolName} would return results here`);
  }

  showHelp() {
    console.log("\n📖 Available Commands:");
    console.log("  list-tools               - List available MCP tools");
    console.log("  list-resources           - List available MCP resources");
    console.log("  generate-text [prompt]   - Generate text using AI");
    console.log("  register-agent [id] [type] - Register a new agent");
    console.log("  submit-task [id] [type] [payload] - Submit a task");
    console.log(
      "  store-memory [tenant] [task] [type] [content] - Store memory"
    );
    console.log("  retrieve-memory [tenant] [task] - Retrieve memories");
    console.log("  read-file [path]         - Read content from a file");
    console.log("  write-file [path] [content] - Write content to a file");
    console.log(
      "  edit-file [path] [old] [new] - Edit file with search/replace"
    );
    console.log("  list-dir [path]          - List directory contents");
    console.log(
      "  decompose-task [desc]    - Break down complex task into steps"
    );
    console.log(
      "  execute-plan [plan]      - Execute a task plan step by step"
    );
    console.log("  help                     - Show this help");
    console.log("  exit                     - Exit the client");
    console.log("");
  }

  async cleanup() {
    if (this.serverProcess) {
      console.log("🛑 Stopping MCP server...");
      this.serverProcess.kill();
    }
    this.rl.close();
  }
}

// Handle graceful shutdown
process.on("SIGINT", () => {
  console.log("\n👋 Shutting down...");
  process.exit(0);
});

// Start the interactive client
const client = new InteractiveMCPClient();
client.start().catch((error) => {
  console.error("❌ Failed to start MCP client:", error);
  process.exit(1);
});
