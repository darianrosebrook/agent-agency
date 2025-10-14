/**
 * @fileoverview Event Emitter for Arbiter Orchestration (ARBITER-005)
 *
 * Provides comprehensive event emission and handling system for observability,
 * monitoring, debugging, and system integration.
 *
 * @author @darianrosebrook
 */

// Re-export commonly used types
export { VerificationPriority } from "../types/verification";

/**
 * Event severity levels
 */
export enum EventSeverity {
  DEBUG = "debug",
  INFO = "info",
  WARN = "warn",
  ERROR = "error",
  CRITICAL = "critical",
}

/**
 * Base event interface
 */
export interface BaseEvent {
  /** Unique event identifier */
  id: string;

  /** Event type identifier */
  type: string;

  /** Timestamp when event occurred */
  timestamp: Date;

  /** Event severity */
  severity: EventSeverity;

  /** Source component that emitted the event */
  source: string;

  /** Correlation ID for tracing related events */
  correlationId?: string;

  /** Session ID if applicable */
  sessionId?: string;

  /** Agent ID if applicable */
  agentId?: string;

  /** Task ID if applicable */
  taskId?: string;

  /** Additional metadata */
  metadata?: Record<string, any>;
}

/**
 * Event handler function type
 */
export type EventHandler<T extends BaseEvent = BaseEvent> = (
  event: T
) => void | Promise<void>;

/**
 * Event filter for selective handling
 */
export interface EventFilter {
  /** Filter by event types */
  types?: string[];

  /** Filter by severity levels */
  severities?: EventSeverity[];

  /** Filter by source components */
  sources?: string[];

  /** Filter by agent IDs */
  agentIds?: string[];

  /** Filter by task IDs */
  taskIds?: string[];

  /** Custom filter function */
  customFilter?: (event: BaseEvent) => boolean;
}

/**
 * Event storage configuration
 */
export interface EventStorageConfig {
  /** Maximum number of events to store in memory */
  maxEvents: number;

  /** Event retention period in milliseconds */
  retentionMs: number;

  /** Enable persistent storage */
  persistentStorage: boolean;

  /** Storage path for persistent events */
  storagePath?: string;
}

/**
 * Event emitter configuration
 */
export interface EventEmitterConfig {
  /** Enable event emission */
  enabled: boolean;

  /** Storage configuration */
  storage: EventStorageConfig;

  /** Enable async event handling */
  asyncHandlers: boolean;

  /** Maximum handler execution time (ms) */
  handlerTimeoutMs: number;
}

/**
 * EventEmitter - Core event emission and handling system
 */
export class EventEmitter {
  private config: EventEmitterConfig;
  private events: BaseEvent[] = [];
  private handlers = new Map<string, Set<EventHandler>>();
  private persistentEvents = new Map<string, BaseEvent>();
  private cleanupTimer?: ReturnType<typeof setInterval>;

  constructor(config: Partial<EventEmitterConfig> = {}) {
    this.config = {
      enabled: true,
      storage: {
        maxEvents: 10000,
        retentionMs: 24 * 60 * 60 * 1000, // 24 hours
        persistentStorage: false,
      },
      asyncHandlers: true,
      handlerTimeoutMs: 5000,
      ...config,
    };

    if (this.config.enabled) {
      this.startCleanupTimer();
    }
  }

  /**
   * Emit an event to all registered handlers
   */
  async emit<T extends BaseEvent>(event: T): Promise<void> {
    if (!this.config.enabled) {
      return;
    }

    // Add to in-memory storage
    this.storeEvent(event);

    // Emit to all matching handlers
    await this.emitToHandlers(event);
  }

  /**
   * Register an event handler
   */
  on<T extends BaseEvent>(eventType: string, handler: EventHandler<T>): void {
    if (!this.handlers.has(eventType)) {
      this.handlers.set(eventType, new Set());
    }
    this.handlers.get(eventType)!.add(handler as EventHandler);
  }

  /**
   * Remove an event handler
   */
  off<T extends BaseEvent>(eventType: string, handler: EventHandler<T>): void {
    const handlers = this.handlers.get(eventType);
    if (handlers) {
      handlers.delete(handler as EventHandler);
      if (handlers.size === 0) {
        this.handlers.delete(eventType);
      }
    }
  }

  /**
   * Register a handler for multiple event types
   */
  onMultiple<T extends BaseEvent>(
    eventTypes: string[],
    handler: EventHandler<T>
  ): void {
    eventTypes.forEach((type) => this.on(type, handler));
  }

  /**
   * Register a filtered event handler
   */
  onFiltered<T extends BaseEvent>(
    filter: EventFilter,
    handler: EventHandler<T>
  ): void {
    const filteredHandler = (event: BaseEvent) => {
      if (this.matchesFilter(event, filter)) {
        return handler(event as T);
      }
    };
    this.on("filtered", filteredHandler);
  }

  /**
   * Get events matching a filter
   */
  getEvents(filter?: EventFilter, limit = 100): BaseEvent[] {
    let events = [...this.events];

    if (filter) {
      events = events.filter((event) => this.matchesFilter(event, filter));
    }

    return events.slice(-limit);
  }

  /**
   * Get event statistics
   */
  getStats(): EventStats {
    const stats = {
      totalEvents: this.events.length,
      eventsByType: new Map<string, number>(),
      eventsBySeverity: new Map<EventSeverity, number>(),
      eventsBySource: new Map<string, number>(),
      oldestEvent: this.events[0]?.timestamp,
      newestEvent: this.events[this.events.length - 1]?.timestamp,
      handlersByType: new Map<string, number>(),
    };

    // Count events by type, severity, source
    for (const event of this.events) {
      stats.eventsByType.set(
        event.type,
        (stats.eventsByType.get(event.type) || 0) + 1
      );
      stats.eventsBySeverity.set(
        event.severity,
        (stats.eventsBySeverity.get(event.severity) || 0) + 1
      );
      stats.eventsBySource.set(
        event.source,
        (stats.eventsBySource.get(event.source) || 0) + 1
      );
    }

    // Count handlers by type
    for (const [type, handlers] of Array.from(this.handlers)) {
      stats.handlersByType.set(type, handlers.size);
    }

    return stats;
  }

  /**
   * Clear all events
   */
  clear(): void {
    this.events = [];
    this.persistentEvents.clear();
  }

  /**
   * Shutdown the event emitter
   */
  shutdown(): void {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = undefined;
    }
  }

  /**
   * Store event in memory and persistent storage
   */
  private storeEvent(event: BaseEvent): void {
    // Add to in-memory storage
    this.events.push(event);

    // Maintain max events limit
    if (this.events.length > this.config.storage.maxEvents) {
      this.events = this.events.slice(-this.config.storage.maxEvents);
    }

    // Add to persistent storage if enabled
    if (this.config.storage.persistentStorage) {
      this.persistentEvents.set(event.id, event);
    }
  }

  /**
   * Emit event to all matching handlers
   */
  private async emitToHandlers(event: BaseEvent): Promise<void> {
    const handlers = this.handlers.get(event.type);
    if (!handlers || handlers.size === 0) {
      return;
    }

    const promises: Promise<void>[] = [];

    for (const handler of Array.from(handlers)) {
      if (this.config.asyncHandlers) {
        // Async handling with timeout
        const promise = this.executeHandlerWithTimeout(handler, event);
        promises.push(promise);
      } else {
        // Synchronous handling
        try {
          const result = handler(event);
          if (result instanceof Promise) {
            promises.push(result);
          }
        } catch (error) {
          console.error(`Event handler error for ${event.type}:`, error);
        }
      }
    }

    // Wait for all handlers to complete (with error handling)
    await Promise.allSettled(promises);
  }

  /**
   * Execute handler with timeout protection
   */
  private async executeHandlerWithTimeout(
    handler: EventHandler,
    event: BaseEvent
  ): Promise<void> {
    return new Promise((resolve) => {
      const timeout = setTimeout(() => {
        console.warn(
          `Event handler timeout for ${event.type} after ${this.config.handlerTimeoutMs}ms`
        );
        resolve();
      }, this.config.handlerTimeoutMs);

      try {
        const result = handler(event);
        if (result instanceof Promise) {
          result
            .then(() => {
              clearTimeout(timeout);
              resolve();
            })
            .catch((error) => {
              clearTimeout(timeout);
              console.error(
                `Async event handler error for ${event.type}:`,
                error
              );
              resolve();
            });
        } else {
          clearTimeout(timeout);
          resolve();
        }
      } catch (error) {
        clearTimeout(timeout);
        console.error(`Event handler error for ${event.type}:`, error);
        resolve();
      }
    });
  }

  /**
   * Check if event matches filter
   */
  private matchesFilter(event: BaseEvent, filter: EventFilter): boolean {
    if (filter.types && !filter.types.includes(event.type)) {
      return false;
    }

    if (filter.severities && !filter.severities.includes(event.severity)) {
      return false;
    }

    if (filter.sources && !filter.sources.includes(event.source)) {
      return false;
    }

    if (
      filter.agentIds &&
      event.agentId &&
      !filter.agentIds.includes(event.agentId)
    ) {
      return false;
    }

    if (
      filter.taskIds &&
      event.taskId &&
      !filter.taskIds.includes(event.taskId)
    ) {
      return false;
    }

    if (filter.customFilter && !filter.customFilter(event)) {
      return false;
    }

    return true;
  }

  /**
   * Start periodic cleanup of old events
   */
  private startCleanupTimer(): void {
    this.cleanupTimer = setInterval(() => {
      this.cleanupOldEvents();
    }, 60000); // Clean up every minute
  }

  /**
   * Clean up events older than retention period
   */
  private cleanupOldEvents(): void {
    const cutoff = Date.now() - this.config.storage.retentionMs;
    this.events = this.events.filter(
      (event) => event.timestamp.getTime() > cutoff
    );
  }
}

/**
 * Event statistics
 */
export interface EventStats {
  totalEvents: number;
  eventsByType: Map<string, number>;
  eventsBySeverity: Map<EventSeverity, number>;
  eventsBySource: Map<string, number>;
  oldestEvent?: Date;
  newestEvent?: Date;
  handlersByType: Map<string, number>;
}

/**
 * Global event emitter instance
 */
export const globalEventEmitter = new EventEmitter({
  enabled: true,
  storage: {
    maxEvents: 10000,
    retentionMs: 24 * 60 * 60 * 1000, // 24 hours
    persistentStorage: false,
  },
  asyncHandlers: true,
  handlerTimeoutMs: 5000,
});

/**
 * Convenience functions for global event emitter
 */
export const events = {
  emit: <T extends BaseEvent>(event: T) => globalEventEmitter.emit(event),
  on: <T extends BaseEvent>(eventType: string, handler: EventHandler<T>) =>
    globalEventEmitter.on(eventType, handler),
  off: <T extends BaseEvent>(eventType: string, handler: EventHandler<T>) =>
    globalEventEmitter.off(eventType, handler),
  getEvents: (filter?: EventFilter, limit?: number) =>
    globalEventEmitter.getEvents(filter, limit),
  getStats: () => globalEventEmitter.getStats(),
  clear: () => globalEventEmitter.clear(),
  shutdown: () => globalEventEmitter.shutdown(),
};
