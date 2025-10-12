/**
 * Distributed Tracing Provider
 *
 * Provides OpenTelemetry-based distributed tracing for the system.
 * Automatically instruments operations and propagates trace context.
 *
 * @author @darianrosebrook
 */

import {
  Span,
  SpanOptions,
  SpanStatusCode,
  context,
  trace,
} from "@opentelemetry/api";
import { NodeTracerProvider } from "@opentelemetry/sdk-trace-node";
import { SemanticResourceAttributes } from "@opentelemetry/semantic-conventions";

/**
 * Tracing provider for distributed tracing
 */
export class TracingProvider {
  private provider: NodeTracerProvider;
  private tracer: any;
  private serviceName: string;

  constructor(serviceName: string = "arbiter-orchestrator") {
    this.serviceName = serviceName;
    this.provider = new NodeTracerProvider({
      resource: new Resource({
        [SemanticResourceAttributes.SERVICE_NAME]: serviceName,
      }),
    });

    this.provider.register();
    this.tracer = trace.getTracer(serviceName);
  }

  /**
   * Trace an async operation
   *
   * Creates a span, executes the operation, and handles errors automatically.
   */
  async traceOperation<T>(
    name: string,
    operation: () => Promise<T>,
    attributes?: Record<string, any>
  ): Promise<T> {
    const span = this.tracer.startSpan(name, {
      attributes: attributes || {},
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
      span.recordException(error as Error);
      throw error;
    } finally {
      span.end();
    }
  }

  /**
   * Trace a synchronous operation
   */
  traceSync<T>(
    name: string,
    operation: () => T,
    attributes?: Record<string, any>
  ): T {
    const span = this.tracer.startSpan(name, {
      attributes: attributes || {},
    });

    try {
      const result = context.with(
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
      span.recordException(error as Error);
      throw error;
    } finally {
      span.end();
    }
  }

  /**
   * Start a span manually (for more control)
   */
  startSpan(name: string, options?: SpanOptions): Span {
    return this.tracer.startSpan(name, options);
  }

  /**
   * Get current tracer
   */
  getTracer() {
    return this.tracer;
  }

  /**
   * Get service name
   */
  getServiceName(): string {
    return this.serviceName;
  }

  /**
   * Shutdown tracing provider
   */
  async shutdown(): Promise<void> {
    await this.provider.shutdown();
  }
}
