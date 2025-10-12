# Phase 0.3: Production Infrastructure Implementation Plan

**Date**: October 12, 2025  
**Status**: 🔄 In Progress  
**Expected Duration**: 3-4 hours

---

## Overview

Add production-grade infrastructure to ARBITER foundation components (001-004) to ensure reliability, observability, and operational excellence.

---

## Goals

1. **Observability**: Full visibility into system behavior
2. **Resilience**: Handle failures gracefully
3. **Configuration**: Flexible, environment-aware settings
4. **Health Monitoring**: Track system health in real-time
5. **Graceful Operations**: Clean startup and shutdown

---

## Implementation Order

### 1. Centralized Configuration (45 minutes)

**Why First**: Foundation for all other infrastructure

**Deliverables**:

- `src/config/AppConfig.ts` - Centralized configuration management
- Environment-aware settings
- Type-safe configuration
- Validation on load

**Key Features**:

- Environment variables with defaults
- Type-safe access
- Validation rules
- Configuration reload without restart

---

### 2. Distributed Tracing (60 minutes)

**Why Second**: Essential for debugging and performance analysis

**Deliverables**:

- `src/observability/TracingProvider.ts` - OpenTelemetry integration
- Automatic span creation
- Trace context propagation
- Performance metrics collection

**Key Features**:

- OpenTelemetry SDK integration
- Automatic instrumentation
- Custom span creation
- Trace correlation across components

**Traces to Capture**:

- Agent registration/query
- Task routing decisions
- CAWS validation
- Performance tracking events

---

### 3. Health Monitoring (45 minutes)

**Why Third**: Enables production readiness checks

**Deliverables**:

- `src/health/HealthMonitor.ts` - Component health tracking
- Health check endpoints
- Dependency monitoring
- Alerting integration

**Key Features**:

- Liveness checks (is service running?)
- Readiness checks (can service handle traffic?)
- Component-level health status
- Dependency health tracking

**Health Checks**:

- Agent registry availability
- Router responsiveness
- Validator functionality
- Performance tracker operational

---

### 4. Circuit Breakers (45 minutes)

**Why Fourth**: Resilience against cascading failures

**Deliverables**:

- `src/resilience/CircuitBreaker.ts` - Circuit breaker implementation
- Automatic failure detection
- Fallback strategies
- Self-healing recovery

**Key Features**:

- Configurable failure thresholds
- Timeout handling
- Exponential backoff
- State tracking (closed/open/half-open)

**Protected Operations**:

- Agent registry queries
- Task routing (with fallback)
- External validations
- Performance data writes

---

### 5. Graceful Shutdown (30 minutes)

**Why Last**: Ensures clean termination

**Deliverables**:

- `src/lifecycle/ShutdownManager.ts` - Coordinated shutdown
- Signal handling
- Resource cleanup
- In-flight request completion

**Key Features**:

- SIGTERM/SIGINT handling
- Drain active connections
- Close database connections
- Flush pending metrics
- Timeout protection

---

## Detailed Implementation

### 1. Centralized Configuration

```typescript
// src/config/AppConfig.ts
import { z } from "zod";

/**
 * Application configuration schema
 */
const configSchema = z.object({
  // Environment
  env: z.enum(["development", "staging", "production"]).default("development"),

  // Server
  server: z.object({
    port: z.number().default(3000),
    host: z.string().default("localhost"),
  }),

  // Agent Registry
  registry: z.object({
    maxAgents: z.number().default(1000),
    cacheEnabled: z.boolean().default(true),
    cacheTTL: z.number().default(300), // seconds
  }),

  // Task Routing
  routing: z.object({
    maxRoutingTimeMs: z.number().default(100),
    explorationRate: z.number().min(0).max(1).default(0.1),
  }),

  // Performance Tracking
  performance: z.object({
    bufferSize: z.number().default(1000),
    flushIntervalMs: z.number().default(5000),
  }),

  // Observability
  observability: z.object({
    tracingEnabled: z.boolean().default(true),
    metricsEnabled: z.boolean().default(true),
    logLevel: z.enum(["debug", "info", "warn", "error"]).default("info"),
  }),

  // Resilience
  resilience: z.object({
    circuitBreakerEnabled: z.boolean().default(true),
    failureThreshold: z.number().default(5),
    timeoutMs: z.number().default(5000),
  }),
});

export type AppConfig = z.infer<typeof configSchema>;

export class ConfigManager {
  private static instance: ConfigManager;
  private config: AppConfig;

  private constructor() {
    this.config = this.loadConfig();
  }

  static getInstance(): ConfigManager {
    if (!ConfigManager.instance) {
      ConfigManager.instance = new ConfigManager();
    }
    return ConfigManager.instance;
  }

  private loadConfig(): AppConfig {
    const raw = {
      env: process.env.NODE_ENV || "development",
      server: {
        port: Number(process.env.PORT) || 3000,
        host: process.env.HOST || "localhost",
      },
      registry: {
        maxAgents: Number(process.env.MAX_AGENTS) || 1000,
        cacheEnabled: process.env.CACHE_ENABLED !== "false",
        cacheTTL: Number(process.env.CACHE_TTL) || 300,
      },
      routing: {
        maxRoutingTimeMs: Number(process.env.MAX_ROUTING_TIME_MS) || 100,
        explorationRate: Number(process.env.EXPLORATION_RATE) || 0.1,
      },
      performance: {
        bufferSize: Number(process.env.PERF_BUFFER_SIZE) || 1000,
        flushIntervalMs: Number(process.env.PERF_FLUSH_INTERVAL_MS) || 5000,
      },
      observability: {
        tracingEnabled: process.env.TRACING_ENABLED !== "false",
        metricsEnabled: process.env.METRICS_ENABLED !== "false",
        logLevel: (process.env.LOG_LEVEL as any) || "info",
      },
      resilience: {
        circuitBreakerEnabled: process.env.CIRCUIT_BREAKER_ENABLED !== "false",
        failureThreshold: Number(process.env.FAILURE_THRESHOLD) || 5,
        timeoutMs: Number(process.env.TIMEOUT_MS) || 5000,
      },
    };

    return configSchema.parse(raw);
  }

  get(): AppConfig {
    return this.config;
  }

  reload(): void {
    this.config = this.loadConfig();
  }
}
```

---

### 2. Distributed Tracing

```typescript
// src/observability/TracingProvider.ts
import { trace, context, SpanStatusCode } from "@opentelemetry/api";
import { NodeTracerProvider } from "@opentelemetry/sdk-trace-node";
import { Resource } from "@opentelemetry/resources";
import { SemanticResourceAttributes } from "@opentelemetry/semantic-conventions";

export class TracingProvider {
  private provider: NodeTracerProvider;
  private tracer: any;

  constructor(serviceName: string = "arbiter-orchestrator") {
    this.provider = new NodeTracerProvider({
      resource: new Resource({
        [SemanticResourceAttributes.SERVICE_NAME]: serviceName,
      }),
    });

    this.provider.register();
    this.tracer = trace.getTracer(serviceName);
  }

  /**
   * Create a span for an operation
   */
  async traceOperation<T>(
    name: string,
    operation: () => Promise<T>,
    attributes?: Record<string, any>
  ): Promise<T> {
    const span = this.tracer.startSpan(name, {
      attributes,
    });

    try {
      const result = await context.with(
        trace.setSpan(context.active(), span),
        operation
      );
      span.setStatus({ code: SpanStatusCode.OK });
      return result;
    } catch (error) {
      span.setStatus({
        code: SpanStatusCode.ERROR,
        message: error instanceof Error ? error.message : "Unknown error",
      });
      throw error;
    } finally {
      span.end();
    }
  }

  /**
   * Get current tracer
   */
  getTracer() {
    return this.tracer;
  }
}
```

---

### 3. Health Monitoring

```typescript
// src/health/HealthMonitor.ts
export enum HealthStatus {
  HEALTHY = "healthy",
  DEGRADED = "degraded",
  UNHEALTHY = "unhealthy",
}

export interface ComponentHealth {
  name: string;
  status: HealthStatus;
  message?: string;
  lastCheck: Date;
  details?: Record<string, any>;
}

export interface SystemHealth {
  status: HealthStatus;
  components: ComponentHealth[];
  timestamp: Date;
}

export type HealthCheck = () => Promise<ComponentHealth>;

export class HealthMonitor {
  private checks: Map<string, HealthCheck> = new Map();
  private lastResults: Map<string, ComponentHealth> = new Map();

  registerCheck(name: string, check: HealthCheck): void {
    this.checks.set(name, check);
  }

  async checkHealth(): Promise<SystemHealth> {
    const results: ComponentHealth[] = [];

    for (const [name, check] of this.checks) {
      try {
        const result = await check();
        this.lastResults.set(name, result);
        results.push(result);
      } catch (error) {
        const failedCheck: ComponentHealth = {
          name,
          status: HealthStatus.UNHEALTHY,
          message: error instanceof Error ? error.message : "Check failed",
          lastCheck: new Date(),
        };
        this.lastResults.set(name, failedCheck);
        results.push(failedCheck);
      }
    }

    // Determine overall system health
    const unhealthy = results.some((r) => r.status === HealthStatus.UNHEALTHY);
    const degraded = results.some((r) => r.status === HealthStatus.DEGRADED);

    const status = unhealthy
      ? HealthStatus.UNHEALTHY
      : degraded
      ? HealthStatus.DEGRADED
      : HealthStatus.HEALTHY;

    return {
      status,
      components: results,
      timestamp: new Date(),
    };
  }

  async isReady(): Promise<boolean> {
    const health = await this.checkHealth();
    return health.status !== HealthStatus.UNHEALTHY;
  }

  async isLive(): Promise<boolean> {
    // Liveness is less strict - just check if service is responsive
    return true;
  }
}
```

---

### 4. Circuit Breaker

```typescript
// src/resilience/CircuitBreaker.ts
export enum CircuitState {
  CLOSED = "closed", // Normal operation
  OPEN = "open", // Failing, reject requests
  HALF_OPEN = "half-open", // Testing if recovered
}

export interface CircuitBreakerConfig {
  failureThreshold: number; // Failures before opening
  successThreshold: number; // Successes before closing from half-open
  timeout: number; // Time to wait before half-open (ms)
  timeoutMs: number; // Operation timeout (ms)
}

export class CircuitBreaker {
  private state: CircuitState = CircuitState.CLOSED;
  private failureCount = 0;
  private successCount = 0;
  private nextAttempt = Date.now();

  constructor(private config: CircuitBreakerConfig) {}

  async execute<T>(
    operation: () => Promise<T>,
    fallback?: () => T
  ): Promise<T> {
    if (this.state === CircuitState.OPEN) {
      if (Date.now() < this.nextAttempt) {
        if (fallback) {
          return fallback();
        }
        throw new Error("Circuit breaker is OPEN");
      }
      // Try transitioning to half-open
      this.state = CircuitState.HALF_OPEN;
      this.successCount = 0;
    }

    try {
      const result = await this.executeWithTimeout(operation);
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      if (fallback) {
        return fallback();
      }
      throw error;
    }
  }

  private async executeWithTimeout<T>(operation: () => Promise<T>): Promise<T> {
    return Promise.race([
      operation(),
      new Promise<T>((_, reject) =>
        setTimeout(
          () => reject(new Error("Operation timeout")),
          this.config.timeoutMs
        )
      ),
    ]);
  }

  private onSuccess(): void {
    this.failureCount = 0;

    if (this.state === CircuitState.HALF_OPEN) {
      this.successCount++;
      if (this.successCount >= this.config.successThreshold) {
        this.state = CircuitState.CLOSED;
        this.successCount = 0;
      }
    }
  }

  private onFailure(): void {
    this.failureCount++;

    if (this.failureCount >= this.config.failureThreshold) {
      this.state = CircuitState.OPEN;
      this.nextAttempt = Date.now() + this.config.timeout;
    }
  }

  getState(): CircuitState {
    return this.state;
  }

  reset(): void {
    this.state = CircuitState.CLOSED;
    this.failureCount = 0;
    this.successCount = 0;
  }
}
```

---

### 5. Graceful Shutdown

```typescript
// src/lifecycle/ShutdownManager.ts
export type ShutdownHook = () => Promise<void>;

export class ShutdownManager {
  private hooks: ShutdownHook[] = [];
  private isShuttingDown = false;

  constructor(private shutdownTimeout: number = 30000) {
    this.registerSignalHandlers();
  }

  registerShutdownHook(hook: ShutdownHook): void {
    this.hooks.push(hook);
  }

  private registerSignalHandlers(): void {
    process.on("SIGTERM", () => this.shutdown("SIGTERM"));
    process.on("SIGINT", () => this.shutdown("SIGINT"));
  }

  private async shutdown(signal: string): Promise<void> {
    if (this.isShuttingDown) {
      return;
    }

    this.isShuttingDown = true;
    console.log(`Received ${signal}, starting graceful shutdown...`);

    const shutdownPromise = this.executeShutdownHooks();
    const timeoutPromise = new Promise<void>((resolve) =>
      setTimeout(() => {
        console.warn("Shutdown timeout reached, forcing exit");
        resolve();
      }, this.shutdownTimeout)
    );

    await Promise.race([shutdownPromise, timeoutPromise]);

    console.log("Graceful shutdown complete");
    process.exit(0);
  }

  private async executeShutdownHooks(): Promise<void> {
    for (const hook of this.hooks) {
      try {
        await hook();
      } catch (error) {
        console.error("Error executing shutdown hook:", error);
      }
    }
  }
}
```

---

## Testing Strategy

### Unit Tests

- Configuration loading and validation
- Circuit breaker state transitions
- Health check execution
- Shutdown hook registration

### Integration Tests

- Tracing across components
- Circuit breaker with real operations
- Health checks with actual components
- Graceful shutdown scenarios

---

## Acceptance Criteria

1. ✅ Configuration loads from environment variables
2. ✅ Traces are created for all major operations
3. ✅ Health checks report accurate status
4. ✅ Circuit breakers protect against failures
5. ✅ Shutdown completes within timeout
6. ✅ All tests passing (unit + integration)

---

## Implementation Checklist

- [ ] Create centralized configuration
- [ ] Implement tracing provider
- [ ] Build health monitoring system
- [ ] Add circuit breaker implementation
- [ ] Create shutdown manager
- [ ] Write unit tests for each component
- [ ] Write integration tests
- [ ] Integrate with existing components
- [ ] Update documentation
- [ ] Performance validation

---

**Status**: Ready to implement!
