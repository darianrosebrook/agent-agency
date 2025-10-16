// @ts-nocheck
import crypto from "crypto";
import { ServerResponse } from "http";
import { ObserverConfig, ObserverSseClient, ObserverEventPayload } from "../types";

interface SseFilters {
  taskId?: string;
  type?: string;
  severity?: "debug" | "info" | "warn" | "error";
}

/**
 * SSE manager orchestrates connected clients, heartbeats, and filtered event
 * broadcasting. Each client is represented by a ServerResponse that stays open
 * until the consumer disconnects or the manager evicts it.
 */
export class SseManager {
  private readonly config: ObserverConfig;
  private readonly clients = new Map<string, ObserverSseClient>();
  private readonly order: string[] = [];
  private heartbeatTimer?: NodeJS.Timeout;
  private started = false;

  constructor(config: ObserverConfig) {
    this.config = config;
  }

  start(): void {
    if (this.started) return;
    this.started = true;
    this.heartbeatTimer = setInterval(() => {
      this.broadcastRaw("ping", {});
    }, this.config.heartbeatIntervalMs);
  }

  stop(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = undefined;
    }
    for (const client of this.clients.values()) {
      try {
        client.res.write("event: close\ndata: {}\n\n");
        client.res.end();
      } catch {
        // ignore
      }
    }
    this.clients.clear();
    this.order.length = 0;
    this.started = false;
  }

  registerClient(
    res: ServerResponse,
    filters: SseFilters,
    verbose: boolean
  ): ObserverSseClient {
    if (!this.started) {
      this.start();
    }

    if (this.order.length >= this.config.maxClients) {
      const oldestId = this.order.shift();
      if (oldestId) {
        this.disconnectClient(oldestId);
      }
    }

    const id = crypto.randomUUID();
    const client: ObserverSseClient = {
      id,
      res,
      filters,
      verbose,
      connectedAt: Date.now(),
    };
    this.clients.set(id, client);
    this.order.push(id);

    res.writeHead(200, {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache, no-transform",
      Connection: "keep-alive",
      "X-Accel-Buffering": "no",
    });
    res.write("\n"); // ensure headers flush

    return client;
  }

  disconnectClient(id: string): void {
    const client = this.clients.get(id);
    if (!client) return;
    this.clients.delete(id);
    const index = this.order.indexOf(id);
    if (index >= 0) {
      this.order.splice(index, 1);
    }
    try {
      client.res.write("event: close\ndata: {}\n\n");
      client.res.end();
    } catch {
      // ignore
    }
  }

  handleDisconnect(id: string): void {
    this.disconnectClient(id);
  }

  /**
   * Broadcast an observer event to all clients respecting filter predicates.
   */
  broadcastEvent(event: ObserverEventPayload): void {
    const data = JSON.stringify(event);
    for (const client of this.clients.values()) {
      if (!this.matchesFilters(client.filters, event)) {
        continue;
      }
      this.writeEvent(client, "event", client.verbose ? data : this.minifyEvent(event));
    }
  }

  broadcastRaw(eventName: string, payload: unknown): void {
    const data = JSON.stringify(payload);
    for (const client of this.clients.values()) {
      this.writeEvent(client, eventName, data);
    }
  }

  private writeEvent(
    client: ObserverSseClient,
    eventName: string,
    data: string
  ): void {
    try {
      client.res.write(`event: ${eventName}\n`);
      client.res.write(`data: ${data}\n\n`);
    } catch {
      this.disconnectClient(client.id);
    }
  }

  private matchesFilters(
    filters: SseFilters,
    event: ObserverEventPayload
  ): boolean {
    if (filters.taskId && event.taskId !== filters.taskId) {
      return false;
    }
    if (filters.type && event.type !== filters.type) {
      return false;
    }
    if (filters.severity && event.severity !== filters.severity) {
      return false;
    }
    return true;
  }

  private minifyEvent(event: ObserverEventPayload): string {
    const { id, type, severity, taskId, timestamp, source } = event;
    return JSON.stringify({ id, type, severity, taskId, timestamp, source });
  }
}

