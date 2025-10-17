/**
 * MCP Client for E2E Testing
 *
 * @author @darianrosebrook
 * @description MCP client to interact with Agent Agency server in E2E tests
 */

import { spawn } from "child_process";
import { EventEmitter } from "events";

export interface MCPRequest {
  jsonrpc: "2.0";
  id: number;
  method: string;
  params?: any;
}

export interface MCPResponse {
  jsonrpc: "2.0";
  id: number;
  result?: any;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
}

export interface MCPNotification {
  jsonrpc: "2.0";
  method: string;
  params?: any;
}

export class MCPClient extends EventEmitter {
  private serverProcess: any;
  private requestId = 1;
  private pendingRequests = new Map<
    number,
    {
      resolve: (value: any) => void;
      reject: (error: any) => void;
      timeout: NodeJS.Timeout;
    }
  >();

  constructor(private serverPath: string, private serverArgs: string[] = []) {
    super();
  }

  /**
   * Start the MCP server process
   */
  async start(): Promise<void> {
    return new Promise((resolve, reject) => {
      console.log("🚀 Starting MCP server for E2E testing...");

      this.serverProcess = spawn(
        "node",
        [this.serverPath, ...this.serverArgs],
        {
          stdio: ["pipe", "pipe", "pipe"],
          cwd: process.cwd(),
        }
      );

      let startupComplete = false;
      const startupTimeout: NodeJS.Timeout = setTimeout(() => {
        if (!startupComplete) {
          reject(new Error("Server startup timeout"));
        }
      }, 30000); // 30 second timeout

      const checkStartup = (data: Buffer) => {
        const output = data.toString();
        console.log("🔍 Checking startup:", output);

        if (
          output.includes("Server started") ||
          output.includes("listening") ||
          output.includes("ready") ||
          output.includes("connected and ready")
        ) {
          if (!startupComplete) {
            startupComplete = true;
            clearTimeout(startupTimeout);
            console.log("✅ Server startup detected");

            // Now set up response handling
            this.setupResponseHandler();
            resolve();
          }
        }
      };

      this.serverProcess.stdout.on("data", checkStartup);
      this.serverProcess.stderr.on("data", (data: Buffer) => {
        console.error("❌ Server stderr:", data.toString());
      });

      this.serverProcess.on("error", (error: Error) => {
        console.error("💥 Server process error:", error);
        reject(error);
      });
    });
  }

  /**
   * Stop the MCP server process
   */
  async stop(): Promise<void> {
    return new Promise((resolve) => {
      if (this.serverProcess) {
        console.log("🛑 Stopping MCP server...");

        // Reject any pending requests
        for (const [_id, { reject }] of this.pendingRequests) {
          reject(new Error("Server stopped"));
        }

        this.pendingRequests.clear();

        // Remove all event listeners to prevent memory leaks
        this.serverProcess.removeAllListeners();

        // Kill the process
        this.serverProcess.kill("SIGTERM");

        // Wait for it to exit
        this.serverProcess.on("exit", () => {
          console.log("✅ Server stopped");
          this.serverProcess = null;
          resolve();
        });

        // Force kill after timeout
        setTimeout(() => {
          if (this.serverProcess) {
            this.serverProcess.kill("SIGKILL");
            this.serverProcess = null;
          }
          resolve();
        }, 5000);
      } else {
        resolve();
      }
    });
  }

  /**
   * Send a JSON-RPC request and wait for response
   */
  async sendRequest(
    method: string,
    params?: any,
    timeout = 30000
  ): Promise<any> {
    const id = this.requestId++;
    const request: MCPRequest = {
      jsonrpc: "2.0",
      id,
      method,
      params,
    };

    return new Promise((resolve, reject) => {
      const timeoutHandle = setTimeout(() => {
        this.pendingRequests.delete(id);
        reject(new Error(`Request timeout: ${method}`));
      }, timeout);

      this.pendingRequests.set(id, {
        resolve,
        reject,
        timeout: timeoutHandle,
      });

      const requestJson = JSON.stringify(request) + "\n";
      console.log("📤 Sending request:", requestJson.trim());

      if (this.serverProcess && this.serverProcess.stdin) {
        this.serverProcess.stdin.write(requestJson);
      } else {
        reject(new Error("Server process not running"));
      }
    });
  }

  /**
   * Send a JSON-RPC notification (no response expected)
   */
  sendNotification(method: string, params?: any): void {
    const notification: MCPNotification = {
      jsonrpc: "2.0",
      method,
      params,
    };

    const notificationJson = JSON.stringify(notification) + "\n";
    console.log("📤 Sending notification:", notificationJson.trim());

    if (this.serverProcess && this.serverProcess.stdin) {
      this.serverProcess.stdin.write(notificationJson);
    }
  }

  /**
   * Initialize the MCP connection
   */
  async initialize(
    clientInfo = { name: "e2e-test-client", version: "1.0.0" }
  ): Promise<any> {
    const response = await this.sendRequest("initialize", {
      protocolVersion: "2024-11-05",
      capabilities: {},
      clientInfo,
    });

    if (response.error) {
      throw new Error(`Initialize failed: ${response.error.message}`);
    }

    console.log("✅ MCP connection initialized");
    return response.result;
  }

  /**
   * List available tools
   */
  async listTools(): Promise<any[]> {
    const response = await this.sendRequest("tools/list");
    return response.result?.tools || [];
  }

  /**
   * List available resources
   */
  async listResources(): Promise<any[]> {
    const response = await this.sendRequest("resources/list");
    return response.result?.resources || [];
  }

  /**
   * Call a tool
   */
  async callTool(name: string, args?: any): Promise<any> {
    const response = await this.sendRequest("tools/call", {
      name,
      arguments: args || {},
    });
    return response.result;
  }

  /**
   * Read a resource
   */
  async readResource(uri: string): Promise<any> {
    const response = await this.sendRequest("resources/read", { uri });
    return response.result;
  }

  /**
   * Stop the MCP server
   */
  async shutdown(): Promise<void> {
    await this.stop();
  }

  /**
   * Set up response handler for JSON-RPC responses
   */
  private setupResponseHandler(): void {
    if (!this.serverProcess) return;

    const processOutput = (data: Buffer) => {
      const output = data.toString().trim();
      console.log("📥 Server response:", output);

      // Skip empty lines and obvious non-JSON lines (log messages, alerts, etc.)
      if (!output || (!output.startsWith("{") && !output.startsWith("["))) {
        console.log("ℹ️ Skipping non-JSON output:", output);
        return;
      }

      try {
        const response = JSON.parse(output);
        if (response.id && this.pendingRequests.has(response.id)) {
          const { resolve, reject, timeout } = this.pendingRequests.get(
            response.id
          )!;
          clearTimeout(timeout);
          this.pendingRequests.delete(response.id);

          if (response.error) {
            reject(new Error(response.error.message));
          } else {
            resolve(response);
          }
        } else if (response.method) {
          // Handle notification
          this.emit("notification", response);
        }
      } catch (_parseError) {
        // If it starts with [ or { but isn't valid JSON, it's likely a log message
        console.log("ℹ️ Skipping invalid JSON output:", output);
      }
    };

    this.serverProcess.stdout.on("data", processOutput);
    this.serverProcess.stderr.on("data", processOutput);
  }

  /**
   * Initialize response handling
   */
  init(): void {
    this.setupResponseHandler();
  }
}

/**
 * Create and configure MCP client for E2E testing
 */
export async function createMCPClient(): Promise<MCPClient> {
  const client = new MCPClient("bin/mcp-server.cjs", ["start"]);

  await client.start();
  await client.initialize();

  return client;
}

/**
 * Helper to wait for a condition with timeout
 */
export async function waitFor(
  condition: () => boolean | Promise<boolean>,
  timeout = 10000,
  interval = 500
): Promise<void> {
  const startTime = Date.now();
  while (Date.now() - startTime < timeout) {
    const result = await condition();
    if (result) return;
    await new Promise((resolve) => setTimeout(resolve, interval));
  }
  throw new Error("Condition not met within timeout");
}
