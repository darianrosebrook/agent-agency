/**
 * @fileoverview Structured Logging System
 *
 * Provides consistent, structured logging across the embedding infrastructure
 * with proper log levels, context, and performance tracking.
 *
 * @author @darianrosebrook
 */

export enum LogLevel {
  ERROR = 0,
  WARN = 1,
  INFO = 2,
  DEBUG = 3,
  TRACE = 4,
}

export interface LogContext {
  component?: string;
  operation?: string;
  userId?: string;
  sessionId?: string;
  requestId?: string;
  duration?: number;
  error?: Error;
  metric?: string;
  value?: number;
  unit?: string;
  rss?: number;
  heapTotal?: number;
  heapUsed?: number;
  external?: number;
  circuitBreaker?: string;
  event?: string;
  attempt?: number;
  maxAttempts?: number;
  metadata?: Record<string, any>;
}

export interface LogEntry {
  timestamp: string;
  level: LogLevel;
  component: string;
  message: string;
  context?: LogContext;
  error?: {
    name: string;
    message: string;
    stack?: string;
  };
}

/**
 * Structured logger with consistent formatting and context
 */
export class StructuredLogger {
  private component: string;
  private minLevel: LogLevel;
  private contextDefaults: Partial<LogContext>;

  constructor(component: string, minLevel: LogLevel = LogLevel.INFO) {
    this.component = component;
    this.minLevel = minLevel;
    this.contextDefaults = { component };
  }

  /**
   * Create child logger with additional context
   */
  child(additionalContext: Partial<LogContext>): StructuredLogger {
    const child = new StructuredLogger(this.component, this.minLevel);
    child.contextDefaults = { ...this.contextDefaults, ...additionalContext };
    return child;
  }

  /**
   * Set minimum log level
   */
  setMinLevel(level: LogLevel): void {
    this.minLevel = level;
  }

  /**
   * Log error message
   */
  error(message: string, context?: Partial<LogContext>): void {
    this.log(LogLevel.ERROR, message, context);
  }

  /**
   * Log warning message
   */
  warn(message: string, context?: Partial<LogContext>): void {
    this.log(LogLevel.WARN, message, context);
  }

  /**
   * Log info message
   */
  info(message: string, context?: Partial<LogContext>): void {
    this.log(LogLevel.INFO, message, context);
  }

  /**
   * Log debug message
   */
  debug(message: string, context?: Partial<LogContext>): void {
    this.log(LogLevel.DEBUG, message, context);
  }

  /**
   * Log trace message
   */
  trace(message: string, context?: Partial<LogContext>): void {
    this.log(LogLevel.TRACE, message, context);
  }

  /**
   * Log operation with timing
   */
  async time<T>(
    operation: string,
    fn: () => Promise<T>,
    context?: Partial<LogContext>
  ): Promise<T> {
    const startTime = Date.now();

    try {
      this.debug(`Starting operation: ${operation}`, context);
      const result = await fn();
      const duration = Date.now() - startTime;

      this.info(`Completed operation: ${operation}`, {
        ...context,
        operation,
        duration,
      });

      return result;
    } catch (error) {
      const duration = Date.now() - startTime;

      this.error(`Failed operation: ${operation}`, {
        ...context,
        operation,
        duration,
        error: error as Error,
      });

      throw error;
    }
  }

  /**
   * Internal logging method
   */
  private log(
    level: LogLevel,
    message: string,
    context?: Partial<LogContext>
  ): void {
    if (level > this.minLevel) {
      return;
    }

    const fullContext: LogContext = {
      component: this.component,
      ...this.contextDefaults,
      ...context,
    };

    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      component: this.component,
      message,
      context: fullContext,
    };

    // Add error details if present
    if (fullContext.error) {
      entry.error = {
        name: fullContext.error.name,
        message: fullContext.error.message,
        stack: fullContext.error.stack,
      };
    }

    // Output log entry
    this.output(entry);
  }

  /**
   * Output log entry (can be overridden for custom output)
   */
  protected output(entry: LogEntry): void {
    const levelName = LogLevel[entry.level];
    const contextStr = entry.context ? ` ${JSON.stringify(entry.context)}` : "";
    const errorStr = entry.error
      ? `\n${entry.error.stack || entry.error.message}`
      : "";

    console.log(
      `[${entry.timestamp}] ${levelName} ${entry.component}: ${entry.message}${contextStr}${errorStr}`
    );
  }
}

/**
 * Logger factory for creating component-specific loggers
 */
export class LoggerFactory {
  private static defaultMinLevel = LogLevel.INFO;

  static setDefaultMinLevel(level: LogLevel): void {
    this.defaultMinLevel = level;
  }

  static createEmbeddingLogger(): StructuredLogger {
    return new StructuredLogger("EmbeddingService", this.defaultMinLevel);
  }

  static createWorkspaceLogger(): StructuredLogger {
    return new StructuredLogger("WorkspaceStateManager", this.defaultMinLevel);
  }

  static createKnowledgeLogger(): StructuredLogger {
    return new StructuredLogger("KnowledgeSeeker", this.defaultMinLevel);
  }

  static createDatabaseLogger(): StructuredLogger {
    return new StructuredLogger("KnowledgeDatabase", this.defaultMinLevel);
  }

  static createCircuitBreakerLogger(name: string): StructuredLogger {
    return new StructuredLogger(`CircuitBreaker-${name}`, this.defaultMinLevel);
  }

  static createMonitorLogger(): StructuredLogger {
    return new StructuredLogger("SystemMonitor", this.defaultMinLevel);
  }
}

/**
 * Performance tracking logger
 */
export class PerformanceLogger extends StructuredLogger {
  constructor(component: string) {
    super(component, LogLevel.DEBUG);
  }

  /**
   * Log performance metric
   */
  logMetric(
    name: string,
    value: number,
    unit: string,
    context?: Partial<LogContext>
  ): void {
    this.info(`Performance metric: ${name}`, {
      ...context,
      metric: name,
      value,
      unit,
      operation: "performance_measurement",
    });
  }

  /**
   * Log operation timing
   */
  logTiming(
    operation: string,
    duration: number,
    context?: Partial<LogContext>
  ): void {
    this.info(`Operation timing: ${operation}`, {
      ...context,
      operation,
      duration,
      unit: "ms",
    });
  }

  /**
   * Log memory usage
   */
  logMemoryUsage(context?: Partial<LogContext>): void {
    const usage = process.memoryUsage();
    this.debug("Memory usage", {
      ...context,
      rss: usage.rss,
      heapTotal: usage.heapTotal,
      heapUsed: usage.heapUsed,
      external: usage.external,
      unit: "bytes",
    });
  }
}

/**
 * Error tracking logger
 */
export class ErrorLogger extends StructuredLogger {
  constructor(component: string) {
    super(component, LogLevel.ERROR);
  }

  /**
   * Log error with full context
   */
  logError(
    error: Error,
    operation?: string,
    context?: Partial<LogContext>
  ): void {
    this.error(`Error in ${operation || "operation"}: ${error.message}`, {
      ...context,
      operation,
      error,
    });
  }

  /**
   * Log circuit breaker events
   */
  logCircuitBreakerEvent(
    event: "opened" | "closed" | "half_open" | "failed",
    name: string,
    context?: Partial<LogContext>
  ): void {
    this.warn(`Circuit breaker ${event}: ${name}`, {
      ...context,
      circuitBreaker: name,
      event,
      operation: "circuit_breaker",
    });
  }

  /**
   * Log retry attempts
   */
  logRetry(
    attempt: number,
    maxAttempts: number,
    error: Error,
    context?: Partial<LogContext>
  ): void {
    this.warn(`Retry attempt ${attempt}/${maxAttempts}`, {
      ...context,
      attempt,
      maxAttempts,
      operation: "retry",
      error,
    });
  }
}

/**
 * Global logger configuration
 */
export class LoggerConfig {
  static setGlobalLogLevel(level: LogLevel): void {
    LoggerFactory.setDefaultMinLevel(level);
  }

  static setLogLevelFromEnv(): void {
    const envLevel = process.env.LOG_LEVEL?.toUpperCase();
    switch (envLevel) {
      case "ERROR":
        this.setGlobalLogLevel(LogLevel.ERROR);
        break;
      case "WARN":
        this.setGlobalLogLevel(LogLevel.WARN);
        break;
      case "INFO":
        this.setGlobalLogLevel(LogLevel.INFO);
        break;
      case "DEBUG":
        this.setGlobalLogLevel(LogLevel.DEBUG);
        break;
      case "TRACE":
        this.setGlobalLogLevel(LogLevel.TRACE);
        break;
      default:
        // Keep default INFO level
        break;
    }
  }
}
