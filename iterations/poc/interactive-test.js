#!/usr/bin/env node

/**
 * Interactive Agent Agency Test Script
 *
 * @author @darianrosebrook
 * @description Interactive CLI for testing Agent Agency MCP server with file editing capabilities
 */

const { spawn } = require("child_process");
const readline = require("readline");
const path = require("path");

class InteractiveTester {
  constructor() {
    this.serverProcess = null;
    this.rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });
  }

  async start() {
    console.log("🤖 Agent Agency Interactive Test Environment");
    console.log("==========================================");
    console.log();
    console.log("This tool allows you to:");
    console.log("• Start the MCP server");
    console.log("• Test file operations (read, write, list)");
    console.log("• Run agent tasks with actual file editing");
    console.log("• Debug and monitor system behavior");
    console.log();

    await this.showMenu();
  }

  async showMenu() {
    console.log("Available commands:");
    console.log("1. Start MCP Server");
    console.log("2. Test File Operations");
    console.log("3. Create Test File");
    console.log("4. Run Code Generation Task");
    console.log("5. Run Text Transformation Task");
    console.log("6. Run Design Token Task");
    console.log("7. Stop Server");
    console.log("8. Exit");
    console.log();

    this.rl.question("Choose an option (1-8): ", (answer) => {
      this.handleMenuChoice(answer.trim());
    });
  }

  async handleMenuChoice(choice) {
    try {
      switch (choice) {
        case "1":
          await this.startServer();
          break;
        case "2":
          await this.testFileOperations();
          break;
        case "3":
          await this.createTestFile();
          break;
        case "4":
          await this.runCodeGeneration();
          break;
        case "5":
          await this.runTextTransformation();
          break;
        case "6":
          await this.runDesignTokenTask();
          break;
        case "7":
          await this.stopServer();
          break;
        case "8":
          await this.exit();
          return;
        default:
          console.log("❌ Invalid choice. Please try again.");
      }
    } catch (error) {
      console.error("❌ Error:", error.message);
    }

    console.log();
    await this.showMenu();
  }

  async startServer() {
    if (this.serverProcess) {
      console.log("⚠️  Server is already running");
      return;
    }

    console.log("🚀 Starting MCP server...");

    return new Promise((resolve, reject) => {
      const serverPath = path.join(__dirname, "bin", "mcp-server.cjs");

      this.serverProcess = spawn("node", [serverPath], {
        stdio: ["pipe", "pipe", "pipe"],
        cwd: __dirname,
      });

      let startupComplete = false;

      const checkStartup = (data) => {
        const output = data.toString();
        console.log("📝 Server output:", output.trim());

        if (
          output.includes("Server started") ||
          output.includes("listening") ||
          output.includes("ready")
        ) {
          startupComplete = true;
          console.log("✅ Server started successfully!");
          resolve();
        }
      };

      this.serverProcess.stdout.on("data", checkStartup);
      this.serverProcess.stderr.on("data", checkStartup);

      this.serverProcess.on("error", (error) => {
        console.error("❌ Failed to start server:", error);
        reject(error);
      });

      // Timeout after 30 seconds
      setTimeout(() => {
        if (!startupComplete) {
          console.error("❌ Server startup timeout");
          this.serverProcess.kill();
          reject(new Error("Server startup timeout"));
        }
      }, 30000);
    });
  }

  async testFileOperations() {
    console.log("📁 Testing File Operations");
    console.log("------------------------");

    // Test writing a file
    const testFile = "test-output.txt";
    const testContent = `Test file created at ${new Date().toISOString()}\nThis file was created by the Agent Agency interactive test.`;

    console.log(`📝 Writing test file: ${testFile}`);
    console.log(`Content: ${testContent.substring(0, 50)}...`);

    // Note: In a real implementation, we'd use the MCP client to call the file tools
    // For now, we'll simulate the file operations

    console.log("✅ File operations test completed (simulation)");
  }

  async createTestFile() {
    console.log("📄 Creating Test File");
    console.log("-------------------");

    this.rl.question("Enter filename: ", (filename) => {
      this.rl.question("Enter file content: ", (content) => {
        console.log(`📝 Creating file: ${filename}`);
        console.log(`Content length: ${content.length} characters`);

        // In real implementation, this would use MCP file tools
        console.log("✅ Test file creation completed (simulation)");
        this.showMenu();
      });
    });
  }

  async runCodeGeneration() {
    console.log("💻 Running Code Generation Task");
    console.log("------------------------------");

    const spec = `Create a React TypeScript component for a user profile card with:
- Name, email, and avatar display
- Edit button that toggles edit mode
- Form fields for editing name and email
- Save/Cancel buttons in edit mode
- TypeScript interfaces for props and state`;

    console.log("🔧 Generating component...");
    console.log(`Spec: ${spec.substring(0, 100)}...`);

    console.log("✅ Code generation completed (simulation)");
  }

  async runTextTransformation() {
    console.log("📝 Running Text Transformation Task");
    console.log("----------------------------------");

    const input =
      "hey team, this is a really casual message that needs to be made more professional. It's got some informal language and could use better structure. Let's make it work better for our stakeholders.";

    console.log("🔄 Transforming text...");
    console.log(`Input: ${input.substring(0, 80)}...`);

    console.log("✅ Text transformation completed (simulation)");
  }

  async runDesignTokenTask() {
    console.log("🎨 Running Design Token Task");
    console.log("----------------------------");

    const componentSpec = `Create a Button component using design tokens:
- Background color from tokens.colors.bg.primary
- Text color from tokens.colors.text.primary
- Border radius from tokens.borderRadius.md
- Padding from tokens.space.md
- Hover states using token variants`;

    console.log("🎯 Applying design tokens...");
    console.log(`Spec: ${componentSpec.substring(0, 100)}...`);

    console.log("✅ Design token application completed (simulation)");
  }

  async stopServer() {
    if (!this.serverProcess) {
      console.log("⚠️  Server is not running");
      return;
    }

    console.log("🛑 Stopping server...");

    return new Promise((resolve) => {
      this.serverProcess.kill("SIGTERM");

      this.serverProcess.on("exit", () => {
        console.log("✅ Server stopped");
        this.serverProcess = null;
        resolve();
      });

      // Force kill after 5 seconds
      setTimeout(() => {
        if (this.serverProcess) {
          this.serverProcess.kill("SIGKILL");
          this.serverProcess = null;
          resolve();
        }
      }, 5000);
    });
  }

  async exit() {
    console.log("👋 Exiting interactive test environment...");

    if (this.serverProcess) {
      await this.stopServer();
    }

    this.rl.close();
    process.exit(0);
  }
}

// Handle graceful shutdown
process.on("SIGINT", () => {
  console.log("\n👋 Received SIGINT, shutting down gracefully...");
  process.exit(0);
});

process.on("SIGTERM", () => {
  console.log("\n👋 Received SIGTERM, shutting down gracefully...");
  process.exit(0);
});

// Start the interactive tester
const tester = new InteractiveTester();
tester.start().catch((error) => {
  console.error("❌ Failed to start interactive tester:", error);
  process.exit(1);
});
