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
      ],
    }));

    this.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args = {} } = request.params;

      try {
        switch (name) {
          case "arbiter_start":
            return this.wrapResponse(await this.client.post("/observer/arbiter/start"));
          case "arbiter_stop":
            return this.wrapResponse(await this.client.post("/observer/arbiter/stop"));
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
            return this.wrapResponse(await this.client.get("/observer/metrics"));
          case "arbiter_progress":
            return this.wrapResponse(await this.client.get("/observer/progress"));
          case "arbiter_observe":
            return this.wrapResponse(
              await this.client.post("/observer/observations", {
                message: args.message,
                taskId: args.taskId,
                author: args.author,
              })
            );
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

  wrapResponse(body) {
    const text = typeof body === "string" ? body : JSON.stringify(body, null, 2);
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

