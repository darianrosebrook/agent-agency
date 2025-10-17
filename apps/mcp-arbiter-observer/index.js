/**
 * MCP Arbiter Observer Client
 *
 * Connects to the Arbiter observer bridge over HTTP to provide control and
 * observability tooling via Model Context Protocol.
 *
 * Environment variables:
 *  - OBSERVER_URL (default http://127.0.0.1:4387)
 *  - OBSERVER_AUTH_TOKEN (optional bearer token)
 *
 * This is an ES module compiled artifact (no build step required).
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  InitializeRequestSchema,
  InitializedNotificationSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";

const DEFAULT_BASE_URL = process.env.OBSERVER_URL ?? "http://127.0.0.1:4387";
const AUTH_TOKEN = process.env.OBSERVER_AUTH_TOKEN;

class HttpClient {
  constructor(baseUrl, authToken) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
    this.authToken = authToken;
  }

  async request(path, options = {}) {
    const url = `${this.baseUrl}${path}`;
    const headers = new Headers(options.headers ?? {});
    headers.set("Accept", "application/json");
    if (!headers.has("Content-Type") && options.body) {
      headers.set("Content-Type", "application/json");
    }
    if (this.authToken) {
      headers.set("Authorization", `Bearer ${this.authToken}`);
    }

    const response = await fetch(url, { ...options, headers });
    const text = await response.text();
    let body = text;
    try {
      body = text ? JSON.parse(text) : {};
    } catch {
      // ignore non-json responses
    }

    if (!response.ok) {
      const errorMessage =
        body?.error ??
        `HTTP ${response.status} ${response.statusText}` ??
        "Unknown error";
      throw new Error(errorMessage);
    }
    return body;
  }

  get(path, params) {
    const qs = params
      ? `?${new URLSearchParams(
          Object.fromEntries(
            Object.entries(params).filter(([, value]) => value !== undefined)
          )
        ).toString()}`
      : "";
    return this.request(`${path}${qs}`, { method: "GET" });
  }

  post(path, body) {
    return this.request(path, {
      method: "POST",
      body: body ? JSON.stringify(body) : undefined,
    });
  }
}

class ArbiterObserverServer extends Server {
  constructor() {
    super(
      {
        name: "arbiter-observer",
        version: "2.0.0",
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );

    this.client = new HttpClient(DEFAULT_BASE_URL, AUTH_TOKEN);
    this.setupHandlers();
  }

  setupHandlers() {
    this.setRequestHandler(ListToolsRequestSchema, async () => ({
      tools: [
        {
          name: "arbiter_start",
          description: "Ensure the Arbiter orchestrator is running",
          inputSchema: { type: "object", properties: {} },
        },
        {
          name: "arbiter_stop",
          description: "Request graceful shutdown of the Arbiter orchestrator",
          inputSchema: { type: "object", properties: {} },
        },
        {
          name: "arbiter_status",
          description: "Retrieve observer status information",
          inputSchema: { type: "object", properties: {} },
        },
        {
          name: "arbiter_execute",
          description:
            "Submit a task description/spec for autonomous execution or run a management command",
          inputSchema: {
            type: "object",
            properties: {
              description: {
                type: "string",
                description: "High-level task description",
              },
              specPath: {
                type: "string",
                description: "Path to working spec file",
              },
              metadata: {
                type: "object",
                description: "Additional metadata to forward with the task",
              },
              command: {
                type: "string",
                description:
                  "Management command to forward to /observer/commands",
              },
            },
          },
        },
        {
          name: "arbiter_logs",
          description: "Fetch recent Arbiter events",
          inputSchema: {
            type: "object",
            properties: {
              limit: {
                type: "number",
                description: "Number of log entries to fetch (default 50)",
                default: 50,
              },
              cursor: {
                type: "string",
                description: "Pagination cursor returned by previous call",
              },
              severity: {
                type: "string",
                enum: ["debug", "info", "warn", "error"],
                description: "Filter by severity",
              },
            },
          },
        },
        {
          name: "arbiter_cot",
          description: "Fetch chain-of-thought entries",
          inputSchema: {
            type: "object",
            properties: {
              taskId: {
                type: "string",
                description: "Optional task ID to filter",
              },
              limit: {
                type: "number",
                description: "Number of entries (default 20)",
                default: 20,
              },
              cursor: {
                type: "string",
                description: "Pagination cursor",
              },
            },
          },
        },
        {
          name: "arbiter_metrics",
          description: "Retrieve aggregated metrics snapshot",
          inputSchema: { type: "object", properties: {} },
        },
        {
          name: "arbiter_progress",
          description: "Retrieve reasoning progress counters",
          inputSchema: { type: "object", properties: {} },
        },
        {
          name: "arbiter_observe",
          description: "Append an observation note to the log",
          inputSchema: {
            type: "object",
            properties: {
              message: {
                type: "string",
                description: "Observation text",
              },
              taskId: {
                type: "string",
                description: "Optional task ID",
              },
              author: {
                type: "string",
                description: "Optional author identifier",
              },
            },
            required: ["message"],
          },
        },
        {
          name: "file_read",
          description: "Read the contents of a file",
          inputSchema: {
            type: "object",
            properties: {
              target_file: {
                type: "string",
                description:
                  "Path to the file to read, relative to workspace root",
              },
              offset: {
                type: "number",
                description: "Optional line number to start reading from",
              },
              limit: {
                type: "number",
                description: "Optional number of lines to read",
              },
            },
            required: ["target_file"],
          },
        },
        {
          name: "file_search_replace",
          description: "Search and replace text in a file",
          inputSchema: {
            type: "object",
            properties: {
              file_path: {
                type: "string",
                description:
                  "Path to the file to modify, relative to workspace root",
              },
              old_string: {
                type: "string",
                description: "Text to replace",
              },
              new_string: {
                type: "string",
                description: "Replacement text",
              },
              replace_all: {
                type: "boolean",
                description: "Whether to replace all occurrences",
                default: false,
              },
            },
            required: ["file_path", "old_string", "new_string"],
          },
        },
        {
          name: "file_write",
          description: "Write content to a file (overwrites existing file)",
          inputSchema: {
            type: "object",
            properties: {
              file_path: {
                type: "string",
                description:
                  "Path to the file to write, relative to workspace root",
              },
              contents: {
                type: "string",
                description: "Content to write to the file",
              },
            },
            required: ["file_path", "contents"],
          },
        },
        {
          name: "run_terminal_cmd",
          description: "Execute a terminal command",
          inputSchema: {
            type: "object",
            properties: {
              command: {
                type: "string",
                description: "Terminal command to execute",
              },
              is_background: {
                type: "boolean",
                description: "Whether to run in background",
                default: false,
              },
              explanation: {
                type: "string",
                description: "Explanation of why this command is needed",
              },
            },
            required: ["command"],
          },
        },
      ],
    }));

    this.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args = {} } = request.params;

      try {
        switch (name) {
          case "arbiter_start":
            return this.wrapResponse(
              await this.client.post("/observer/arbiter/start")
            );
          case "arbiter_stop":
            return this.wrapResponse(
              await this.client.post("/observer/arbiter/stop")
            );
          case "arbiter_status":
            return this.wrapResponse(await this.client.get("/observer/status"));
          case "arbiter_execute":
            return this.handleExecute(args);
          case "arbiter_logs":
            return this.wrapResponse(
              await this.client.get("/observer/logs", {
                limit: args.limit ?? 50,
                cursor: args.cursor,
                severity: args.severity,
              })
            );
          case "arbiter_cot":
            return this.wrapResponse(
              await this.client.get(
                args.taskId
                  ? `/observer/tasks/${encodeURIComponent(args.taskId)}/cot`
                  : "/observer/cot",
                {
                  limit: args.limit ?? 20,
                  cursor: args.cursor,
                }
              )
            );
          case "arbiter_metrics":
            return this.wrapResponse(
              await this.client.get("/observer/metrics")
            );
          case "arbiter_progress":
            return this.wrapResponse(
              await this.client.get("/observer/progress")
            );
          case "arbiter_observe":
            return this.wrapResponse(
              await this.client.post("/observer/observations", {
                message: args.message,
                taskId: args.taskId,
                author: args.author,
              })
            );
          case "file_read":
            return await this.handleFileRead(args);
          case "file_search_replace":
            return await this.handleFileSearchReplace(args);
          case "file_write":
            return await this.handleFileWrite(args);
          case "run_terminal_cmd":
            return await this.handleTerminalCmd(args);
          default:
            throw new Error(`Unknown tool: ${name}`);
        }
      } catch (error) {
        return this.errorResponse(
          error instanceof Error ? error.message : "Unknown error"
        );
      }
    });

    this.setRequestHandler(InitializeRequestSchema, async () => ({
      protocolVersion: "2024-11-05",
      capabilities: {
        tools: {},
      },
      serverInfo: {
        name: "arbiter-observer",
        version: "2.0.0",
      },
    }));

    this.setRequestHandler(InitializedNotificationSchema, async () => {
      console.log("Arbiter Observer MCP initialized");
      return {};
    });
  }

  async handleExecute(args) {
    if (args.command) {
      const result = await this.client.post("/observer/commands", {
        command: args.command,
      });
      return this.wrapResponse(result);
    }

    if (!args.description && !args.specPath) {
      return this.errorResponse("Provide either description or specPath");
    }

    const payload = {
      description: args.description,
      specPath: args.specPath,
      metadata: args.metadata,
    };
    const result = await this.client.post("/observer/tasks", payload);
    return this.wrapResponse(result);
  }

  async handleFileRead(args) {
    try {
      const fs = await import("fs/promises");
      const path = await import("path");

      const fullPath = path.resolve(process.cwd(), args.target_file);
      let content = await fs.readFile(fullPath, "utf-8");

      // Handle offset and limit
      if (args.offset || args.limit) {
        const lines = content.split("\n");
        const startLine = (args.offset || 1) - 1; // Convert to 0-based
        const endLine = args.limit ? startLine + args.limit : lines.length;

        content = lines.slice(startLine, endLine).join("\n");
      }

      return this.wrapResponse(content);
    } catch (error) {
      throw new Error(
        `Failed to read file ${args.target_file}: ${error.message}`
      );
    }
  }

  async handleFileSearchReplace(args) {
    try {
      const fs = await import("fs/promises");
      const path = await import("path");

      const fullPath = path.resolve(process.cwd(), args.file_path);
      let content = await fs.readFile(fullPath, "utf-8");

      if (args.replace_all) {
        content = content.split(args.old_string).join(args.new_string);
      } else {
        const index = content.indexOf(args.old_string);
        if (index === -1) {
          throw new Error(`Text "${args.old_string}" not found in file`);
        }
        content = content.replace(args.old_string, args.new_string);
      }

      await fs.writeFile(fullPath, content, "utf-8");
      return this.wrapResponse(
        `Successfully replaced text in ${args.file_path}`
      );
    } catch (error) {
      throw new Error(
        `Failed to modify file ${args.file_path}: ${error.message}`
      );
    }
  }

  async handleFileWrite(args) {
    try {
      const fs = await import("fs/promises");
      const path = await import("path");

      const fullPath = path.resolve(process.cwd(), args.file_path);
      await fs.writeFile(fullPath, args.contents, "utf-8");
      return this.wrapResponse(`Successfully wrote to ${args.file_path}`);
    } catch (error) {
      throw new Error(
        `Failed to write file ${args.file_path}: ${error.message}`
      );
    }
  }

  async handleTerminalCmd(args) {
    try {
      const { exec } = await import("child_process");
      const util = await import("util");
      const execAsync = util.promisify(exec);

      // Basic security: prevent dangerous commands
      const dangerousPatterns = [
        /rm\s+-rf\s+\//,
        />/,
        /sudo/,
        /chmod\s+777/,
        /dd\s+if=/,
      ];

      if (dangerousPatterns.some((pattern) => pattern.test(args.command))) {
        throw new Error("Command contains potentially dangerous operations");
      }

      const { stdout, stderr } = await execAsync(args.command, {
        cwd: process.cwd(),
        timeout: 30000, // 30 second timeout
        maxBuffer: 1024 * 1024, // 1MB buffer
      });

      const result = {
        command: args.command,
        stdout: stdout.trim(),
        stderr: stderr.trim(),
        exitCode: 0,
      };

      return this.wrapResponse(JSON.stringify(result, null, 2));
    } catch (error) {
      if (error.code) {
        // exec error with exit code
        const result = {
          command: args.command,
          stdout: error.stdout?.trim() || "",
          stderr: error.stderr?.trim() || "",
          exitCode: error.code,
          error: error.message,
        };
        return this.wrapResponse(JSON.stringify(result, null, 2));
      }
      throw new Error(`Command execution failed: ${error.message}`);
    }
  }

  wrapResponse(body) {
    const text =
      typeof body === "string" ? body : JSON.stringify(body, null, 2);
    return {
      content: [{ type: "text", text }],
    };
  }

  errorResponse(message) {
    return {
      isError: true,
      content: [{ type: "text", text: `Error: ${message}` }],
    };
  }
}

async function main() {
  const server = new ArbiterObserverServer();
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.log("Arbiter Observer MCP server ready");
}

main().catch((error) => {
  console.error("Observer MCP failed:", error);
  process.exit(1);
});
