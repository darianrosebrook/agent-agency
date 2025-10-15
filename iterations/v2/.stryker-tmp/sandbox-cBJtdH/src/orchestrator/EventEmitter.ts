/**
 * @fileoverview Event Emitter for Arbiter Orchestration (ARBITER-005)
 *
 * Provides comprehensive event emission and handling system for observability,
 * monitoring, debugging, and system integration.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


// Re-export commonly used types
function stryNS_9fa48() {
  var g = typeof globalThis === 'object' && globalThis && globalThis.Math === Math && globalThis || new Function("return this")();
  var ns = g.__stryker__ || (g.__stryker__ = {});
  if (ns.activeMutant === undefined && g.process && g.process.env && g.process.env.__STRYKER_ACTIVE_MUTANT__) {
    ns.activeMutant = g.process.env.__STRYKER_ACTIVE_MUTANT__;
  }
  function retrieveNS() {
    return ns;
  }
  stryNS_9fa48 = retrieveNS;
  return retrieveNS();
}
stryNS_9fa48();
function stryCov_9fa48() {
  var ns = stryNS_9fa48();
  var cov = ns.mutantCoverage || (ns.mutantCoverage = {
    static: {},
    perTest: {}
  });
  function cover() {
    var c = cov.static;
    if (ns.currentTestId) {
      c = cov.perTest[ns.currentTestId] = cov.perTest[ns.currentTestId] || {};
    }
    var a = arguments;
    for (var i = 0; i < a.length; i++) {
      c[a[i]] = (c[a[i]] || 0) + 1;
    }
  }
  stryCov_9fa48 = cover;
  cover.apply(null, arguments);
}
function stryMutAct_9fa48(id) {
  var ns = stryNS_9fa48();
  function isActive(id) {
    if (ns.activeMutant === id) {
      if (ns.hitCount !== void 0 && ++ns.hitCount > ns.hitLimit) {
        throw new Error('Stryker: Hit count limit reached (' + ns.hitCount + ')');
      }
      return true;
    }
    return false;
  }
  stryMutAct_9fa48 = isActive;
  return isActive(id);
}
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
export type EventHandler<T extends BaseEvent = BaseEvent> = (event: T) => void | Promise<void>;

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
    if (stryMutAct_9fa48("940")) {
      {}
    } else {
      stryCov_9fa48("940");
      this.config = stryMutAct_9fa48("941") ? {} : (stryCov_9fa48("941"), {
        enabled: stryMutAct_9fa48("942") ? false : (stryCov_9fa48("942"), true),
        storage: stryMutAct_9fa48("943") ? {} : (stryCov_9fa48("943"), {
          maxEvents: 10000,
          retentionMs: stryMutAct_9fa48("944") ? 24 * 60 * 60 / 1000 : (stryCov_9fa48("944"), (stryMutAct_9fa48("945") ? 24 * 60 / 60 : (stryCov_9fa48("945"), (stryMutAct_9fa48("946") ? 24 / 60 : (stryCov_9fa48("946"), 24 * 60)) * 60)) * 1000),
          // 24 hours
          persistentStorage: stryMutAct_9fa48("947") ? true : (stryCov_9fa48("947"), false)
        }),
        asyncHandlers: stryMutAct_9fa48("948") ? false : (stryCov_9fa48("948"), true),
        handlerTimeoutMs: 5000,
        ...config
      });
      if (stryMutAct_9fa48("950") ? false : stryMutAct_9fa48("949") ? true : (stryCov_9fa48("949", "950"), this.config.enabled)) {
        if (stryMutAct_9fa48("951")) {
          {}
        } else {
          stryCov_9fa48("951");
          this.startCleanupTimer();
        }
      }
    }
  }

  /**
   * Emit an event to all registered handlers
   */
  async emit<T extends BaseEvent>(event: T): Promise<void> {
    if (stryMutAct_9fa48("952")) {
      {}
    } else {
      stryCov_9fa48("952");
      if (stryMutAct_9fa48("955") ? false : stryMutAct_9fa48("954") ? true : stryMutAct_9fa48("953") ? this.config.enabled : (stryCov_9fa48("953", "954", "955"), !this.config.enabled)) {
        if (stryMutAct_9fa48("956")) {
          {}
        } else {
          stryCov_9fa48("956");
          return;
        }
      }

      // Add to in-memory storage
      this.storeEvent(event);

      // Emit to all matching handlers
      await this.emitToHandlers(event);
    }
  }

  /**
   * Register an event handler
   */
  on<T extends BaseEvent>(eventType: string, handler: EventHandler<T>): void {
    if (stryMutAct_9fa48("957")) {
      {}
    } else {
      stryCov_9fa48("957");
      if (stryMutAct_9fa48("960") ? false : stryMutAct_9fa48("959") ? true : stryMutAct_9fa48("958") ? this.handlers.has(eventType) : (stryCov_9fa48("958", "959", "960"), !this.handlers.has(eventType))) {
        if (stryMutAct_9fa48("961")) {
          {}
        } else {
          stryCov_9fa48("961");
          this.handlers.set(eventType, new Set());
        }
      }
      this.handlers.get(eventType)!.add(handler as EventHandler);
    }
  }

  /**
   * Remove an event handler
   */
  off<T extends BaseEvent>(eventType: string, handler: EventHandler<T>): void {
    if (stryMutAct_9fa48("962")) {
      {}
    } else {
      stryCov_9fa48("962");
      const handlers = this.handlers.get(eventType);
      if (stryMutAct_9fa48("964") ? false : stryMutAct_9fa48("963") ? true : (stryCov_9fa48("963", "964"), handlers)) {
        if (stryMutAct_9fa48("965")) {
          {}
        } else {
          stryCov_9fa48("965");
          handlers.delete(handler as EventHandler);
          if (stryMutAct_9fa48("968") ? handlers.size !== 0 : stryMutAct_9fa48("967") ? false : stryMutAct_9fa48("966") ? true : (stryCov_9fa48("966", "967", "968"), handlers.size === 0)) {
            if (stryMutAct_9fa48("969")) {
              {}
            } else {
              stryCov_9fa48("969");
              this.handlers.delete(eventType);
            }
          }
        }
      }
    }
  }

  /**
   * Register a handler for multiple event types
   */
  onMultiple<T extends BaseEvent>(eventTypes: string[], handler: EventHandler<T>): void {
    if (stryMutAct_9fa48("970")) {
      {}
    } else {
      stryCov_9fa48("970");
      eventTypes.forEach(stryMutAct_9fa48("971") ? () => undefined : (stryCov_9fa48("971"), type => this.on(type, handler)));
    }
  }

  /**
   * Register a filtered event handler
   */
  onFiltered<T extends BaseEvent>(filter: EventFilter, handler: EventHandler<T>): void {
    if (stryMutAct_9fa48("972")) {
      {}
    } else {
      stryCov_9fa48("972");
      const filteredHandler = (event: BaseEvent) => {
        if (stryMutAct_9fa48("973")) {
          {}
        } else {
          stryCov_9fa48("973");
          if (stryMutAct_9fa48("975") ? false : stryMutAct_9fa48("974") ? true : (stryCov_9fa48("974", "975"), this.matchesFilter(event, filter))) {
            if (stryMutAct_9fa48("976")) {
              {}
            } else {
              stryCov_9fa48("976");
              return handler(event as T);
            }
          }
        }
      };
      this.on("filtered", filteredHandler);
    }
  }

  /**
   * Get events matching a filter
   */
  getEvents(filter?: EventFilter, limit = 100): BaseEvent[] {
    if (stryMutAct_9fa48("978")) {
      {}
    } else {
      stryCov_9fa48("978");
      let events = [...this.events];
      if (stryMutAct_9fa48("981") ? false : stryMutAct_9fa48("980") ? true : (stryCov_9fa48("980", "981"), filter)) {
        if (stryMutAct_9fa48("982")) {
          {}
        } else {
          stryCov_9fa48("982");
          events = stryMutAct_9fa48("983") ? events : (stryCov_9fa48("983"), events.filter(stryMutAct_9fa48("984") ? () => undefined : (stryCov_9fa48("984"), event => this.matchesFilter(event, filter))));
        }
      }
      return stryMutAct_9fa48("985") ? events : (stryCov_9fa48("985"), events.slice(stryMutAct_9fa48("986") ? +limit : (stryCov_9fa48("986"), -limit)));
    }
  }

  /**
   * Get event statistics
   */
  getStats(): EventStats {
    if (stryMutAct_9fa48("987")) {
      {}
    } else {
      stryCov_9fa48("987");
      const stats = stryMutAct_9fa48("988") ? {} : (stryCov_9fa48("988"), {
        totalEvents: this.events.length,
        eventsByType: new Map<string, number>(),
        eventsBySeverity: new Map<EventSeverity, number>(),
        eventsBySource: new Map<string, number>(),
        oldestEvent: stryMutAct_9fa48("989") ? this.events[0].timestamp : (stryCov_9fa48("989"), this.events[0]?.timestamp),
        newestEvent: stryMutAct_9fa48("990") ? this.events[this.events.length - 1].timestamp : (stryCov_9fa48("990"), this.events[stryMutAct_9fa48("991") ? this.events.length + 1 : (stryCov_9fa48("991"), this.events.length - 1)]?.timestamp),
        handlersByType: new Map<string, number>()
      });

      // Count events by type, severity, source
      for (const event of this.events) {
        if (stryMutAct_9fa48("992")) {
          {}
        } else {
          stryCov_9fa48("992");
          stats.eventsByType.set(event.type, stryMutAct_9fa48("993") ? (stats.eventsByType.get(event.type) || 0) - 1 : (stryCov_9fa48("993"), (stryMutAct_9fa48("996") ? stats.eventsByType.get(event.type) && 0 : stryMutAct_9fa48("995") ? false : stryMutAct_9fa48("994") ? true : (stryCov_9fa48("994", "995", "996"), stats.eventsByType.get(event.type) || 0)) + 1));
          stats.eventsBySeverity.set(event.severity, stryMutAct_9fa48("997") ? (stats.eventsBySeverity.get(event.severity) || 0) - 1 : (stryCov_9fa48("997"), (stryMutAct_9fa48("1000") ? stats.eventsBySeverity.get(event.severity) && 0 : stryMutAct_9fa48("999") ? false : stryMutAct_9fa48("998") ? true : (stryCov_9fa48("998", "999", "1000"), stats.eventsBySeverity.get(event.severity) || 0)) + 1));
          stats.eventsBySource.set(event.source, stryMutAct_9fa48("1001") ? (stats.eventsBySource.get(event.source) || 0) - 1 : (stryCov_9fa48("1001"), (stryMutAct_9fa48("1004") ? stats.eventsBySource.get(event.source) && 0 : stryMutAct_9fa48("1003") ? false : stryMutAct_9fa48("1002") ? true : (stryCov_9fa48("1002", "1003", "1004"), stats.eventsBySource.get(event.source) || 0)) + 1));
        }
      }

      // Count handlers by type
      for (const [type, handlers] of Array.from(this.handlers)) {
        if (stryMutAct_9fa48("1005")) {
          {}
        } else {
          stryCov_9fa48("1005");
          stats.handlersByType.set(type, handlers.size);
        }
      }
      return stats;
    }
  }

  /**
   * Clear all events
   */
  clear(): void {
    if (stryMutAct_9fa48("1006")) {
      {}
    } else {
      stryCov_9fa48("1006");
      this.events = [];
      this.persistentEvents.clear();
    }
  }

  /**
   * Shutdown the event emitter
   */
  shutdown(): void {
    if (stryMutAct_9fa48("1008")) {
      {}
    } else {
      stryCov_9fa48("1008");
      if (stryMutAct_9fa48("1010") ? false : stryMutAct_9fa48("1009") ? true : (stryCov_9fa48("1009", "1010"), this.cleanupTimer)) {
        if (stryMutAct_9fa48("1011")) {
          {}
        } else {
          stryCov_9fa48("1011");
          clearInterval(this.cleanupTimer);
          this.cleanupTimer = undefined;
        }
      }
    }
  }

  /**
   * Store event in memory and persistent storage
   */
  private storeEvent(event: BaseEvent): void {
    if (stryMutAct_9fa48("1012")) {
      {}
    } else {
      stryCov_9fa48("1012");
      // Add to in-memory storage
      this.events.push(event);

      // Maintain max events limit
      if (stryMutAct_9fa48("1016") ? this.events.length <= this.config.storage.maxEvents : stryMutAct_9fa48("1015") ? this.events.length >= this.config.storage.maxEvents : stryMutAct_9fa48("1014") ? false : stryMutAct_9fa48("1013") ? true : (stryCov_9fa48("1013", "1014", "1015", "1016"), this.events.length > this.config.storage.maxEvents)) {
        if (stryMutAct_9fa48("1017")) {
          {}
        } else {
          stryCov_9fa48("1017");
          this.events = stryMutAct_9fa48("1018") ? this.events : (stryCov_9fa48("1018"), this.events.slice(stryMutAct_9fa48("1019") ? +this.config.storage.maxEvents : (stryCov_9fa48("1019"), -this.config.storage.maxEvents)));
        }
      }

      // Add to persistent storage if enabled
      if (stryMutAct_9fa48("1021") ? false : stryMutAct_9fa48("1020") ? true : (stryCov_9fa48("1020", "1021"), this.config.storage.persistentStorage)) {
        if (stryMutAct_9fa48("1022")) {
          {}
        } else {
          stryCov_9fa48("1022");
          this.persistentEvents.set(event.id, event);
        }
      }
    }
  }

  /**
   * Emit event to all matching handlers
   */
  private async emitToHandlers(event: BaseEvent): Promise<void> {
    if (stryMutAct_9fa48("1023")) {
      {}
    } else {
      stryCov_9fa48("1023");
      const handlers = this.handlers.get(event.type);
      if (stryMutAct_9fa48("1026") ? !handlers && handlers.size === 0 : stryMutAct_9fa48("1025") ? false : stryMutAct_9fa48("1024") ? true : (stryCov_9fa48("1024", "1025", "1026"), (stryMutAct_9fa48("1027") ? handlers : (stryCov_9fa48("1027"), !handlers)) || (stryMutAct_9fa48("1029") ? handlers.size !== 0 : stryMutAct_9fa48("1028") ? false : (stryCov_9fa48("1028", "1029"), handlers.size === 0)))) {
        if (stryMutAct_9fa48("1030")) {
          {}
        } else {
          stryCov_9fa48("1030");
          return;
        }
      }
      const promises: Promise<void>[] = [];
      for (const handler of Array.from(handlers)) {
        if (stryMutAct_9fa48("1032")) {
          {}
        } else {
          stryCov_9fa48("1032");
          if (stryMutAct_9fa48("1034") ? false : stryMutAct_9fa48("1033") ? true : (stryCov_9fa48("1033", "1034"), this.config.asyncHandlers)) {
            if (stryMutAct_9fa48("1035")) {
              {}
            } else {
              stryCov_9fa48("1035");
              // Async handling with timeout
              const promise = this.executeHandlerWithTimeout(handler, event);
              promises.push(promise);
            }
          } else {
            if (stryMutAct_9fa48("1036")) {
              {}
            } else {
              stryCov_9fa48("1036");
              // Synchronous handling
              try {
                if (stryMutAct_9fa48("1037")) {
                  {}
                } else {
                  stryCov_9fa48("1037");
                  const result = handler(event);
                  if (stryMutAct_9fa48("1039") ? false : stryMutAct_9fa48("1038") ? true : (stryCov_9fa48("1038", "1039"), result instanceof Promise)) {
                    if (stryMutAct_9fa48("1040")) {
                      {}
                    } else {
                      stryCov_9fa48("1040");
                      promises.push(result);
                    }
                  }
                }
              } catch (error) {
                if (stryMutAct_9fa48("1041")) {
                  {}
                } else {
                  stryCov_9fa48("1041");
                  console.error(`Event handler error for ${event.type}:`, error);
                }
              }
            }
          }
        }
      }

      // Wait for all handlers to complete (with error handling)
      await Promise.allSettled(promises);
    }
  }

  /**
   * Execute handler with timeout protection
   */
  private async executeHandlerWithTimeout(handler: EventHandler, event: BaseEvent): Promise<void> {
    if (stryMutAct_9fa48("1043")) {
      {}
    } else {
      stryCov_9fa48("1043");
      return new Promise(resolve => {
        if (stryMutAct_9fa48("1044")) {
          {}
        } else {
          stryCov_9fa48("1044");
          const timeout = setTimeout(() => {
            if (stryMutAct_9fa48("1045")) {
              {}
            } else {
              stryCov_9fa48("1045");
              console.warn(`Event handler timeout for ${event.type} after ${this.config.handlerTimeoutMs}ms`);
              resolve();
            }
          }, this.config.handlerTimeoutMs);
          try {
            if (stryMutAct_9fa48("1047")) {
              {}
            } else {
              stryCov_9fa48("1047");
              const result = handler(event);
              if (stryMutAct_9fa48("1049") ? false : stryMutAct_9fa48("1048") ? true : (stryCov_9fa48("1048", "1049"), result instanceof Promise)) {
                if (stryMutAct_9fa48("1050")) {
                  {}
                } else {
                  stryCov_9fa48("1050");
                  result.then(() => {
                    if (stryMutAct_9fa48("1051")) {
                      {}
                    } else {
                      stryCov_9fa48("1051");
                      clearTimeout(timeout);
                      resolve();
                    }
                  }).catch(error => {
                    if (stryMutAct_9fa48("1052")) {
                      {}
                    } else {
                      stryCov_9fa48("1052");
                      clearTimeout(timeout);
                      console.error(`Async event handler error for ${event.type}:`, error);
                      resolve();
                    }
                  });
                }
              } else {
                if (stryMutAct_9fa48("1054")) {
                  {}
                } else {
                  stryCov_9fa48("1054");
                  clearTimeout(timeout);
                  resolve();
                }
              }
            }
          } catch (error) {
            if (stryMutAct_9fa48("1055")) {
              {}
            } else {
              stryCov_9fa48("1055");
              clearTimeout(timeout);
              console.error(`Event handler error for ${event.type}:`, error);
              resolve();
            }
          }
        }
      });
    }
  }

  /**
   * Check if event matches filter
   */
  private matchesFilter(event: BaseEvent, filter: EventFilter): boolean {
    if (stryMutAct_9fa48("1057")) {
      {}
    } else {
      stryCov_9fa48("1057");
      if (stryMutAct_9fa48("1060") ? filter.types || !filter.types.includes(event.type) : stryMutAct_9fa48("1059") ? false : stryMutAct_9fa48("1058") ? true : (stryCov_9fa48("1058", "1059", "1060"), filter.types && (stryMutAct_9fa48("1061") ? filter.types.includes(event.type) : (stryCov_9fa48("1061"), !filter.types.includes(event.type))))) {
        if (stryMutAct_9fa48("1062")) {
          {}
        } else {
          stryCov_9fa48("1062");
          return stryMutAct_9fa48("1063") ? true : (stryCov_9fa48("1063"), false);
        }
      }
      if (stryMutAct_9fa48("1066") ? filter.severities || !filter.severities.includes(event.severity) : stryMutAct_9fa48("1065") ? false : stryMutAct_9fa48("1064") ? true : (stryCov_9fa48("1064", "1065", "1066"), filter.severities && (stryMutAct_9fa48("1067") ? filter.severities.includes(event.severity) : (stryCov_9fa48("1067"), !filter.severities.includes(event.severity))))) {
        if (stryMutAct_9fa48("1068")) {
          {}
        } else {
          stryCov_9fa48("1068");
          return stryMutAct_9fa48("1069") ? true : (stryCov_9fa48("1069"), false);
        }
      }
      if (stryMutAct_9fa48("1072") ? filter.sources || !filter.sources.includes(event.source) : stryMutAct_9fa48("1071") ? false : stryMutAct_9fa48("1070") ? true : (stryCov_9fa48("1070", "1071", "1072"), filter.sources && (stryMutAct_9fa48("1073") ? filter.sources.includes(event.source) : (stryCov_9fa48("1073"), !filter.sources.includes(event.source))))) {
        if (stryMutAct_9fa48("1074")) {
          {}
        } else {
          stryCov_9fa48("1074");
          return stryMutAct_9fa48("1075") ? true : (stryCov_9fa48("1075"), false);
        }
      }
      if (stryMutAct_9fa48("1078") ? filter.agentIds && event.agentId || !filter.agentIds.includes(event.agentId) : stryMutAct_9fa48("1077") ? false : stryMutAct_9fa48("1076") ? true : (stryCov_9fa48("1076", "1077", "1078"), (stryMutAct_9fa48("1080") ? filter.agentIds || event.agentId : stryMutAct_9fa48("1079") ? true : (stryCov_9fa48("1079", "1080"), filter.agentIds && event.agentId)) && (stryMutAct_9fa48("1081") ? filter.agentIds.includes(event.agentId) : (stryCov_9fa48("1081"), !filter.agentIds.includes(event.agentId))))) {
        if (stryMutAct_9fa48("1082")) {
          {}
        } else {
          stryCov_9fa48("1082");
          return stryMutAct_9fa48("1083") ? true : (stryCov_9fa48("1083"), false);
        }
      }
      if (stryMutAct_9fa48("1086") ? filter.taskIds && event.taskId || !filter.taskIds.includes(event.taskId) : stryMutAct_9fa48("1085") ? false : stryMutAct_9fa48("1084") ? true : (stryCov_9fa48("1084", "1085", "1086"), (stryMutAct_9fa48("1088") ? filter.taskIds || event.taskId : stryMutAct_9fa48("1087") ? true : (stryCov_9fa48("1087", "1088"), filter.taskIds && event.taskId)) && (stryMutAct_9fa48("1089") ? filter.taskIds.includes(event.taskId) : (stryCov_9fa48("1089"), !filter.taskIds.includes(event.taskId))))) {
        if (stryMutAct_9fa48("1090")) {
          {}
        } else {
          stryCov_9fa48("1090");
          return stryMutAct_9fa48("1091") ? true : (stryCov_9fa48("1091"), false);
        }
      }
      if (stryMutAct_9fa48("1094") ? filter.customFilter || !filter.customFilter(event) : stryMutAct_9fa48("1093") ? false : stryMutAct_9fa48("1092") ? true : (stryCov_9fa48("1092", "1093", "1094"), filter.customFilter && (stryMutAct_9fa48("1095") ? filter.customFilter(event) : (stryCov_9fa48("1095"), !filter.customFilter(event))))) {
        if (stryMutAct_9fa48("1096")) {
          {}
        } else {
          stryCov_9fa48("1096");
          return stryMutAct_9fa48("1097") ? true : (stryCov_9fa48("1097"), false);
        }
      }
      return stryMutAct_9fa48("1098") ? false : (stryCov_9fa48("1098"), true);
    }
  }

  /**
   * Start periodic cleanup of old events
   */
  private startCleanupTimer(): void {
    if (stryMutAct_9fa48("1099")) {
      {}
    } else {
      stryCov_9fa48("1099");
      this.cleanupTimer = setInterval(() => {
        if (stryMutAct_9fa48("1100")) {
          {}
        } else {
          stryCov_9fa48("1100");
          this.cleanupOldEvents();
        }
      }, 60000); // Clean up every minute
    }
  }

  /**
   * Clean up events older than retention period
   */
  private cleanupOldEvents(): void {
    if (stryMutAct_9fa48("1101")) {
      {}
    } else {
      stryCov_9fa48("1101");
      const cutoff = stryMutAct_9fa48("1102") ? Date.now() + this.config.storage.retentionMs : (stryCov_9fa48("1102"), Date.now() - this.config.storage.retentionMs);
      this.events = stryMutAct_9fa48("1103") ? this.events : (stryCov_9fa48("1103"), this.events.filter(stryMutAct_9fa48("1104") ? () => undefined : (stryCov_9fa48("1104"), event => stryMutAct_9fa48("1108") ? event.timestamp.getTime() <= cutoff : stryMutAct_9fa48("1107") ? event.timestamp.getTime() >= cutoff : stryMutAct_9fa48("1106") ? false : stryMutAct_9fa48("1105") ? true : (stryCov_9fa48("1105", "1106", "1107", "1108"), event.timestamp.getTime() > cutoff))));
    }
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
export const globalEventEmitter = new EventEmitter(stryMutAct_9fa48("1109") ? {} : (stryCov_9fa48("1109"), {
  enabled: stryMutAct_9fa48("1110") ? false : (stryCov_9fa48("1110"), true),
  storage: stryMutAct_9fa48("1111") ? {} : (stryCov_9fa48("1111"), {
    maxEvents: 10000,
    retentionMs: stryMutAct_9fa48("1112") ? 24 * 60 * 60 / 1000 : (stryCov_9fa48("1112"), (stryMutAct_9fa48("1113") ? 24 * 60 / 60 : (stryCov_9fa48("1113"), (stryMutAct_9fa48("1114") ? 24 / 60 : (stryCov_9fa48("1114"), 24 * 60)) * 60)) * 1000),
    // 24 hours
    persistentStorage: stryMutAct_9fa48("1115") ? true : (stryCov_9fa48("1115"), false)
  }),
  asyncHandlers: stryMutAct_9fa48("1116") ? false : (stryCov_9fa48("1116"), true),
  handlerTimeoutMs: 5000
}));

/**
 * Convenience functions for global event emitter
 */
export const events = stryMutAct_9fa48("1117") ? {} : (stryCov_9fa48("1117"), {
  emit: stryMutAct_9fa48("1118") ? () => undefined : (stryCov_9fa48("1118"), <T extends BaseEvent,>(event: T) => globalEventEmitter.emit(event)),
  on: stryMutAct_9fa48("1119") ? () => undefined : (stryCov_9fa48("1119"), <T extends BaseEvent,>(eventType: string, handler: EventHandler<T>) => globalEventEmitter.on(eventType, handler)),
  off: stryMutAct_9fa48("1120") ? () => undefined : (stryCov_9fa48("1120"), <T extends BaseEvent,>(eventType: string, handler: EventHandler<T>) => globalEventEmitter.off(eventType, handler)),
  getEvents: stryMutAct_9fa48("1121") ? () => undefined : (stryCov_9fa48("1121"), (filter?: EventFilter, limit?: number) => globalEventEmitter.getEvents(filter, limit)),
  getStats: stryMutAct_9fa48("1122") ? () => undefined : (stryCov_9fa48("1122"), () => globalEventEmitter.getStats()),
  clear: stryMutAct_9fa48("1123") ? () => undefined : (stryCov_9fa48("1123"), () => globalEventEmitter.clear()),
  shutdown: stryMutAct_9fa48("1124") ? () => undefined : (stryCov_9fa48("1124"), () => globalEventEmitter.shutdown())
});