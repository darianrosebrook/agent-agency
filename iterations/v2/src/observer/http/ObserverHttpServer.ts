import fs from "fs";
import http, { IncomingMessage, Server, ServerResponse } from "http";
import { AddressInfo } from "net";
import { URL } from "url";
import { authorizeRequest, ObserverAuthError } from "../auth";
import {
  ArbiterController,
  ObserverConfig,
  ObserverEventPayload,
  ObserverStore,
  SubmitTaskPayload,
} from "../types";
import { SseManager } from "./SseManager";

interface RequestContext {
  req: IncomingMessage;
  res: ServerResponse;
  url: URL;
  origin?: string;
}

export class ObserverHttpServer {
  private readonly config: ObserverConfig;
  private readonly store: ObserverStore;
  private readonly controller: ArbiterController;
  private readonly sse: SseManager;
  private server?: Server;

  constructor(
    config: ObserverConfig,
    store: ObserverStore,
    controller: ArbiterController
  ) {
    this.config = config;
    this.store = store;
    this.controller = controller;
    this.sse = new SseManager(config);
  }

  async start(): Promise<void> {
    if (this.server) {
      return;
    }

    this.server = http.createServer((req, res) => {
      this.handleRequest(req, res).catch((error) => {
        this.handleError(res, error);
      });
    });

    const listenPromise = new Promise<void>((resolve, reject) => {
      if (this.config.socketPath) {
        // Ensure previous socket removed
        try {
          if (fs.existsSync(this.config.socketPath)) {
            fs.unlinkSync(this.config.socketPath);
          }
        } catch (error) {
          console.warn(
            `Failed to remove existing socket ${this.config.socketPath}:`,
            error
          );
        }
        this.server!.listen(this.config.socketPath, () => resolve());
      } else {
        this.server!.listen(this.config.port, this.config.bind, () =>
          resolve()
        );
      }
      this.server!.once("error", reject);
    });

    await listenPromise;
  }

  async stop(): Promise<void> {
    if (!this.server) return;
    this.sse.stop();
    await new Promise<void>((resolve) => this.server!.close(() => resolve()));
    if (this.config.socketPath) {
      try {
        if (fs.existsSync(this.config.socketPath)) {
          fs.unlinkSync(this.config.socketPath);
        }
      } catch (error) {
        console.warn(
          `Failed to remove socket ${this.config.socketPath} on shutdown:`,
          error
        );
      }
    }
    this.server = undefined;
  }

  getAddress(): AddressInfo | string | null {
    return this.server?.address() ?? null;
  }

  publishEvent(event: ObserverEventPayload): void {
    this.sse.broadcastEvent(event);
  }

  publishRaw(eventName: string, payload: unknown): void {
    this.sse.broadcastRaw(eventName, payload);
  }

  private async handleRequest(
    req: IncomingMessage,
    res: ServerResponse
  ): Promise<void> {
    if (!req.url) {
      this.notFound(res);
      return;
    }

    const url = new URL(req.url, "http://localhost");
    const originHeader =
      (req.headers["origin"] as string | undefined) ??
      (req.headers["Origin"] as string | undefined);

    try {
      authorizeRequest(req, this.config);
    } catch (error) {
      if (error instanceof ObserverAuthError) {
        this.sendJson(res, error.status, { error: error.message }, originHeader);
      } else {
        this.sendJson(res, 401, { error: "Unauthorized" }, originHeader);
      }
      return;
    }

    const ctx: RequestContext = { req, res, url, origin: originHeader };

    if (req.method === "OPTIONS") {
      this.handleOptions(ctx);
      return;
    }

    if (req.method === "GET" && url.pathname === "/observer/status") {
      const status = this.store.getStatus();
      this.sendJson(res, 200, status, originHeader);
      return;
    }

    if (req.method === "GET" && url.pathname === "/observer/metrics") {
      const metrics = this.store.getMetrics();
      this.sendJson(res, 200, metrics, originHeader);
      return;
    }

    if (req.method === "GET" && url.pathname === "/observer/progress") {
      const progress = this.store.getProgress();
      this.sendJson(res, 200, progress, originHeader);
      return;
    }

    if (req.method === "GET" && url.pathname === "/observer/logs") {
      const payload = await this.handleListEvents(ctx);
      this.sendJson(res, 200, payload, originHeader);
      return;
    }

    if (req.method === "GET" && url.pathname === "/observer/cot") {
      const payload = await this.handleListChainOfThought(ctx, null);
      this.sendJson(res, 200, payload, originHeader);
      return;
    }

    if (
      req.method === "GET" &&
      url.pathname.startsWith("/observer/tasks/") &&
      !url.pathname.endsWith("/cot")
    ) {
      const taskId = url.pathname.split("/")[3];
      if (!taskId) {
        this.sendJson(res, 400, { error: "Task ID required" }, originHeader);
        return;
      }
      const task = await this.store.getTask(taskId);
      if (!task) {
        this.notFound(res, originHeader);
        return;
      }
      this.sendJson(res, 200, task, originHeader);
      return;
    }

    if (
      req.method === "GET" &&
      url.pathname.startsWith("/observer/tasks/") &&
      url.pathname.endsWith("/cot")
    ) {
      const taskId = url.pathname.split("/")[3];
      const payload = await this.handleListChainOfThought(ctx, taskId);
      this.sendJson(res, 200, payload, originHeader);
      return;
    }

    if (req.method === "POST" && url.pathname === "/observer/observations") {
      const body = await this.readJsonBody(ctx);
      if (!body || typeof body.message !== "string") {
        this.sendJson(res, 400, { error: "message is required" }, originHeader);
        return;
      }
      const record = await this.store.appendObservation({
        message: body.message,
        taskId:
          typeof body.taskId === "string" && body.taskId.length
            ? body.taskId
            : undefined,
        author:
          typeof body.author === "string" && body.author.length
            ? body.author
            : undefined,
      });
      this.sendJson(res, 201, record, originHeader);
      return;
    }

    if (req.method === "POST" && url.pathname === "/observer/tasks") {
      const body = await this.readJsonBody(ctx);
      const payload: SubmitTaskPayload = {
        description: typeof body?.description === "string" ? body.description : "",
        specPath:
          typeof body?.specPath === "string" ? body.specPath : undefined,
        metadata:
          body?.metadata && typeof body.metadata === "object"
            ? (body.metadata as Record<string, unknown>)
            : undefined,
      };
      if (!payload.description && !payload.specPath) {
        this.sendJson(
          res,
          400,
          { error: "description or specPath required" },
          originHeader
        );
        return;
      }
      const result = await this.controller.submitTask(payload);
      this.sendJson(res, 202, result, originHeader);
      return;
    }

    if (req.method === "POST" && url.pathname === "/observer/arbiter/start") {
      const result = await this.controller.ensureArbiterRunning();
      this.sendJson(res, 200, result, originHeader);
      return;
    }

    if (req.method === "POST" && url.pathname === "/observer/arbiter/stop") {
      const result = await this.controller.requestArbiterStop();
      this.sendJson(res, 200, result, originHeader);
      return;
    }

    if (req.method === "POST" && url.pathname === "/observer/commands") {
      const body = await this.readJsonBody(ctx);
      const command =
        typeof body?.command === "string" ? body.command.trim() : "";
      if (!command) {
        this.sendJson(res, 400, { error: "command is required" }, originHeader);
        return;
      }
      const result = await this.controller.executeCommand(command);
      this.sendJson(res, 200, result, originHeader);
      return;
    }

    if (req.method === "GET" && url.pathname === "/observer/events/stream") {
      this.handleSse(ctx);
      return;
    }

    this.notFound(res, originHeader);
  }

  private handleOptions({ res, origin }: RequestContext): void {
    this.setCorsHeaders(res, origin);
    res.writeHead(204);
    res.end();
  }

  private async handleListEvents(ctx: RequestContext) {
    const cursor = ctx.url.searchParams.get("cursor") ?? undefined;
    const limit = ctx.url.searchParams.get("limit");
    const since = ctx.url.searchParams.get("sinceTs");
    const until = ctx.url.searchParams.get("untilTs");
    const type = ctx.url.searchParams.get("type") ?? undefined;
    const taskId = ctx.url.searchParams.get("taskId") ?? undefined;
    const severity = ctx.url.searchParams.get(
      "severity"
    ) as ObserverEventPayload["severity"] | null;

    return await this.store.listEvents({
      cursor,
      limit: limit ? Number(limit) : undefined,
      since: since ? new Date(since) : undefined,
      until: until ? new Date(until) : undefined,
      type,
      taskId,
      severity: severity ?? undefined,
    });
  }

  private async handleListChainOfThought(
    ctx: RequestContext,
    explicitTaskId: string | null
  ) {
    const cursor = ctx.url.searchParams.get("cursor") ?? undefined;
    const limit = ctx.url.searchParams.get("limit");
    const since = ctx.url.searchParams.get("since");
    const taskId =
      explicitTaskId ??
      ctx.url.searchParams.get("taskId") ??
      ctx.url.searchParams.get("task_id") ??
      undefined;

    return await this.store.listChainOfThought({
      taskId: taskId ?? undefined,
      cursor,
      limit: limit ? Number(limit) : undefined,
      since: since ? new Date(since) : undefined,
    });
  }

  private async readJsonBody(ctx: RequestContext): Promise<any> {
    const chunks: Buffer[] = [];
    const { req } = ctx;

    await new Promise<void>((resolve, reject) => {
      req.on("data", (chunk) => {
        chunks.push(chunk);
        if (Buffer.concat(chunks).length > 2 * 1024 * 1024) {
          reject(new Error("Payload too large"));
        }
      });
      req.on("end", () => resolve());
      req.on("error", (error) => reject(error));
    });

    if (!chunks.length) return {};
    try {
      return JSON.parse(Buffer.concat(chunks).toString("utf-8"));
    } catch (error) {
      throw new Error("Invalid JSON payload");
    }
  }

  private handleSse(ctx: RequestContext): void {
    const severity = ctx.url.searchParams.get(
      "severity"
    ) as ObserverEventPayload["severity"] | null;
    const filters = {
      taskId: ctx.url.searchParams.get("taskId") ?? undefined,
      type: ctx.url.searchParams.get("type") ?? undefined,
      severity: severity ?? undefined,
    };
    const verbose =
      ctx.url.searchParams.get("verbose") === "1" ||
      ctx.url.searchParams.get("verbose") === "true";

    const client = this.sse.registerClient(ctx.res, filters, verbose);
    this.setCorsHeaders(ctx.res, ctx.origin);
    ctx.req.on("close", () => {
      this.sse.handleDisconnect(client.id);
    });
  }

  private setCorsHeaders(res: ServerResponse, origin?: string): void {
    if (origin) {
      res.setHeader("Access-Control-Allow-Origin", origin);
    } else {
      res.setHeader("Access-Control-Allow-Origin", "null");
    }
    res.setHeader(
      "Access-Control-Allow-Headers",
      "Content-Type, Authorization, X-Requested-With"
    );
    res.setHeader(
      "Access-Control-Allow-Methods",
      "GET, POST, OPTIONS"
    );
  }

  private sendJson(
    res: ServerResponse,
    status: number,
    payload: unknown,
    origin?: string
  ): void {
    this.setCorsHeaders(res, origin);
    res.writeHead(status, {
      "Content-Type": "application/json",
      "Cache-Control": "no-store",
    });
    res.end(JSON.stringify(payload));
  }

  private notFound(res: ServerResponse, origin?: string): void {
    this.sendJson(res, 404, { error: "Not Found" }, origin);
  }

  private handleError(res: ServerResponse, error: unknown): void {
    if (res.headersSent) {
      try {
        res.end();
      } catch {
        // ignore
      }
      return;
    }
    const message =
      error instanceof Error ? error.message : "Internal server error";
    this.sendJson(res, 500, { error: message });
  }
}
