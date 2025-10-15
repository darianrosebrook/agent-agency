import crypto from "crypto";
import {
  BaseEvent,
  EventSeverity,
  globalEventEmitter,
} from "../orchestrator/EventEmitter";
import { EventTypes } from "../orchestrator/OrchestratorEvents";
import {
  ChainOfThoughtEntry,
  ObserverConfig,
  ObserverEventPayload,
} from "./types";
import { loadObserverConfig } from "./config";
import { ObserverStoreImpl } from "./persistence/ObserverStoreImpl";
import { ObserverHttpServer } from "./http/ObserverHttpServer";
import { ArbiterRuntime } from "../orchestrator/runtime/ArbiterRuntime";

/**
 * Arbiter Observer Bridge
 *
 * Coordinates event ingestion, persistence, and HTTP/SSE exposure.
 */
export class ObserverBridge {
  private readonly config: ObserverConfig;
  private readonly runtime: ArbiterRuntime | null;
  private readonly store: ObserverStoreImpl;
  private readonly server: ObserverHttpServer;
  private readonly handler: (event: BaseEvent) => void;
  private readonly registeredTypes: string[] = [];
  private metricsTimer?: NodeJS.Timeout;
  private started = false;

  constructor(runtime?: ArbiterRuntime | null, config?: ObserverConfig) {
    this.config = config ?? loadObserverConfig();
    this.runtime = runtime ?? null;
    this.store = new ObserverStoreImpl(this.config, this.runtime ?? undefined);
    this.server = new ObserverHttpServer(this.config, this.store, this.store);
    this.handler = this.handleEvent.bind(this);
  }

  async start(): Promise<void> {
    if (this.started) return;
    this.started = true;

    if (this.runtime) {
      await this.runtime.start();
    }
    await this.server.start();
    this.registerEventListeners();
    this.startMetricsHeartbeat();
  }

  async stop(): Promise<void> {
    if (!this.started) return;
    this.started = false;

    this.unregisterEventListeners();
    this.stopMetricsHeartbeat();
    await this.store.shutdown();
    await this.server.stop();
    if (this.runtime) {
      await this.runtime.stop();
    }
  }

  recordChainOfThought(entry: ChainOfThoughtEntry): void {
    this.store.recordChainOfThought(entry);
  }

  private registerEventListeners(): void {
    const types = Object.values(EventTypes);
    for (const type of types) {
      globalEventEmitter.on(type, this.handler);
      this.registeredTypes.push(type);
    }
  }

  private unregisterEventListeners(): void {
    for (const type of this.registeredTypes) {
      globalEventEmitter.off(type, this.handler as any);
    }
    this.registeredTypes.length = 0;
  }

  private handleEvent(event: BaseEvent): void {
    if (!this.shouldSample(event)) {
      return;
    }

    const payload: ObserverEventPayload = {
      id: event.id ?? crypto.randomUUID(),
      type: event.type,
      severity: mapSeverity(event.severity),
      source: event.source,
      taskId: event.taskId,
      agentId: event.agentId,
      timestamp: event.timestamp.toISOString(),
      traceId: event.metadata?.traceId,
      spanId: event.metadata?.spanId,
      correlationId: event.correlationId,
      metadata: event.metadata,
    };

    this.store.recordEvent(payload);
    this.server.publishEvent(payload);
  }

  private shouldSample(event: BaseEvent): boolean {
    const severity = mapSeverity(event.severity);
    const exactKey = `${event.type}`;
    const category = event.type.split(".")[0];
    const severityKey = `${category}.${severity}`;
    const globalKey = "*";

    const sampleRate =
      this.config.sampleRates[exactKey] ??
      this.config.sampleRates[severityKey] ??
      this.config.sampleRates[globalKey] ??
      1;

    if (sampleRate >= 1) {
      return true;
    }
    return Math.random() <= sampleRate;
  }

  private startMetricsHeartbeat(): void {
    this.metricsTimer = setInterval(() => {
      const metrics = this.store.getMetrics();
      this.server.publishRaw("metrics", metrics);
      const status = this.store.getStatus();
      this.server.publishRaw("status", status);
    }, Math.max(this.config.flushIntervalMs * 10, 5000));
  }

  private stopMetricsHeartbeat(): void {
    if (this.metricsTimer) {
      clearInterval(this.metricsTimer);
      this.metricsTimer = undefined;
    }
  }
}

function mapSeverity(severity: EventSeverity): "debug" | "info" | "warn" | "error" {
  switch (severity) {
    case EventSeverity.DEBUG:
      return "debug";
    case EventSeverity.INFO:
      return "info";
    case EventSeverity.WARN:
      return "warn";
    case EventSeverity.ERROR:
    case EventSeverity.CRITICAL:
      return "error";
    default:
      return "info";
  }
}
