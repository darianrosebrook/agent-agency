/**
 * @fileoverview Event Emitter for Arbiter Orchestration (ARBITER-005)
 *
 * Provides comprehensive event emission and handling system for observability,
 * monitoring, debugging, and system integration.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


/**
 * Event severity levels
 */function stryNS_9fa48() {
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
    if (stryMutAct_9fa48("821")) {
      {}
    } else {
      stryCov_9fa48("821");
      this.config = stryMutAct_9fa48("822") ? {} : (stryCov_9fa48("822"), {
        enabled: stryMutAct_9fa48("823") ? false : (stryCov_9fa48("823"), true),
        storage: stryMutAct_9fa48("824") ? {} : (stryCov_9fa48("824"), {
          maxEvents: 10000,
          retentionMs: stryMutAct_9fa48("825") ? 24 * 60 * 60 / 1000 : (stryCov_9fa48("825"), (stryMutAct_9fa48("826") ? 24 * 60 / 60 : (stryCov_9fa48("826"), (stryMutAct_9fa48("827") ? 24 / 60 : (stryCov_9fa48("827"), 24 * 60)) * 60)) * 1000),
          // 24 hours
          persistentStorage: stryMutAct_9fa48("828") ? true : (stryCov_9fa48("828"), false)
        }),
        asyncHandlers: stryMutAct_9fa48("829") ? false : (stryCov_9fa48("829"), true),
        handlerTimeoutMs: 5000,
        ...config
      });
      if (stryMutAct_9fa48("831") ? false : stryMutAct_9fa48("830") ? true : (stryCov_9fa48("830", "831"), this.config.enabled)) {
        if (stryMutAct_9fa48("832")) {
          {}
        } else {
          stryCov_9fa48("832");
          this.startCleanupTimer();
        }
      }
    }
  }

  /**
   * Emit an event to all registered handlers
   */
  async emit<T extends BaseEvent>(event: T): Promise<void> {
    if (stryMutAct_9fa48("833")) {
      {}
    } else {
      stryCov_9fa48("833");
      if (stryMutAct_9fa48("836") ? false : stryMutAct_9fa48("835") ? true : stryMutAct_9fa48("834") ? this.config.enabled : (stryCov_9fa48("834", "835", "836"), !this.config.enabled)) {
        if (stryMutAct_9fa48("837")) {
          {}
        } else {
          stryCov_9fa48("837");
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
    if (stryMutAct_9fa48("838")) {
      {}
    } else {
      stryCov_9fa48("838");
      if (stryMutAct_9fa48("841") ? false : stryMutAct_9fa48("840") ? true : stryMutAct_9fa48("839") ? this.handlers.has(eventType) : (stryCov_9fa48("839", "840", "841"), !this.handlers.has(eventType))) {
        if (stryMutAct_9fa48("842")) {
          {}
        } else {
          stryCov_9fa48("842");
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
    if (stryMutAct_9fa48("843")) {
      {}
    } else {
      stryCov_9fa48("843");
      const handlers = this.handlers.get(eventType);
      if (stryMutAct_9fa48("845") ? false : stryMutAct_9fa48("844") ? true : (stryCov_9fa48("844", "845"), handlers)) {
        if (stryMutAct_9fa48("846")) {
          {}
        } else {
          stryCov_9fa48("846");
          handlers.delete(handler as EventHandler);
          if (stryMutAct_9fa48("849") ? handlers.size !== 0 : stryMutAct_9fa48("848") ? false : stryMutAct_9fa48("847") ? true : (stryCov_9fa48("847", "848", "849"), handlers.size === 0)) {
            if (stryMutAct_9fa48("850")) {
              {}
            } else {
              stryCov_9fa48("850");
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
    if (stryMutAct_9fa48("851")) {
      {}
    } else {
      stryCov_9fa48("851");
      eventTypes.forEach(stryMutAct_9fa48("852") ? () => undefined : (stryCov_9fa48("852"), type => this.on(type, handler)));
    }
  }

  /**
   * Register a filtered event handler
   */
  onFiltered<T extends BaseEvent>(filter: EventFilter, handler: EventHandler<T>): void {
    if (stryMutAct_9fa48("853")) {
      {}
    } else {
      stryCov_9fa48("853");
      const filteredHandler = (event: BaseEvent) => {
        if (stryMutAct_9fa48("854")) {
          {}
        } else {
          stryCov_9fa48("854");
          if (stryMutAct_9fa48("856") ? false : stryMutAct_9fa48("855") ? true : (stryCov_9fa48("855", "856"), this.matchesFilter(event, filter))) {
            if (stryMutAct_9fa48("857")) {
              {}
            } else {
              stryCov_9fa48("857");
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
    if (stryMutAct_9fa48("859")) {
      {}
    } else {
      stryCov_9fa48("859");
      let events = [...this.events];
      if (stryMutAct_9fa48("862") ? false : stryMutAct_9fa48("861") ? true : (stryCov_9fa48("861", "862"), filter)) {
        if (stryMutAct_9fa48("863")) {
          {}
        } else {
          stryCov_9fa48("863");
          events = stryMutAct_9fa48("864") ? events : (stryCov_9fa48("864"), events.filter(stryMutAct_9fa48("865") ? () => undefined : (stryCov_9fa48("865"), event => this.matchesFilter(event, filter))));
        }
      }
      return stryMutAct_9fa48("866") ? events : (stryCov_9fa48("866"), events.slice(stryMutAct_9fa48("867") ? +limit : (stryCov_9fa48("867"), -limit)));
    }
  }

  /**
   * Get event statistics
   */
  getStats(): EventStats {
    if (stryMutAct_9fa48("868")) {
      {}
    } else {
      stryCov_9fa48("868");
      const stats = stryMutAct_9fa48("869") ? {} : (stryCov_9fa48("869"), {
        totalEvents: this.events.length,
        eventsByType: new Map<string, number>(),
        eventsBySeverity: new Map<EventSeverity, number>(),
        eventsBySource: new Map<string, number>(),
        oldestEvent: stryMutAct_9fa48("870") ? this.events[0].timestamp : (stryCov_9fa48("870"), this.events[0]?.timestamp),
        newestEvent: stryMutAct_9fa48("871") ? this.events[this.events.length - 1].timestamp : (stryCov_9fa48("871"), this.events[stryMutAct_9fa48("872") ? this.events.length + 1 : (stryCov_9fa48("872"), this.events.length - 1)]?.timestamp),
        handlersByType: new Map<string, number>()
      });

      // Count events by type, severity, source
      for (const event of this.events) {
        if (stryMutAct_9fa48("873")) {
          {}
        } else {
          stryCov_9fa48("873");
          stats.eventsByType.set(event.type, stryMutAct_9fa48("874") ? (stats.eventsByType.get(event.type) || 0) - 1 : (stryCov_9fa48("874"), (stryMutAct_9fa48("877") ? stats.eventsByType.get(event.type) && 0 : stryMutAct_9fa48("876") ? false : stryMutAct_9fa48("875") ? true : (stryCov_9fa48("875", "876", "877"), stats.eventsByType.get(event.type) || 0)) + 1));
          stats.eventsBySeverity.set(event.severity, stryMutAct_9fa48("878") ? (stats.eventsBySeverity.get(event.severity) || 0) - 1 : (stryCov_9fa48("878"), (stryMutAct_9fa48("881") ? stats.eventsBySeverity.get(event.severity) && 0 : stryMutAct_9fa48("880") ? false : stryMutAct_9fa48("879") ? true : (stryCov_9fa48("879", "880", "881"), stats.eventsBySeverity.get(event.severity) || 0)) + 1));
          stats.eventsBySource.set(event.source, stryMutAct_9fa48("882") ? (stats.eventsBySource.get(event.source) || 0) - 1 : (stryCov_9fa48("882"), (stryMutAct_9fa48("885") ? stats.eventsBySource.get(event.source) && 0 : stryMutAct_9fa48("884") ? false : stryMutAct_9fa48("883") ? true : (stryCov_9fa48("883", "884", "885"), stats.eventsBySource.get(event.source) || 0)) + 1));
        }
      }

      // Count handlers by type
      for (const [type, handlers] of Array.from(this.handlers)) {
        if (stryMutAct_9fa48("886")) {
          {}
        } else {
          stryCov_9fa48("886");
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
    if (stryMutAct_9fa48("887")) {
      {}
    } else {
      stryCov_9fa48("887");
      this.events = [];
      this.persistentEvents.clear();
    }
  }

  /**
   * Shutdown the event emitter
   */
  shutdown(): void {
    if (stryMutAct_9fa48("889")) {
      {}
    } else {
      stryCov_9fa48("889");
      if (stryMutAct_9fa48("891") ? false : stryMutAct_9fa48("890") ? true : (stryCov_9fa48("890", "891"), this.cleanupTimer)) {
        if (stryMutAct_9fa48("892")) {
          {}
        } else {
          stryCov_9fa48("892");
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
    if (stryMutAct_9fa48("893")) {
      {}
    } else {
      stryCov_9fa48("893");
      // Add to in-memory storage
      this.events.push(event);

      // Maintain max events limit
      if (stryMutAct_9fa48("897") ? this.events.length <= this.config.storage.maxEvents : stryMutAct_9fa48("896") ? this.events.length >= this.config.storage.maxEvents : stryMutAct_9fa48("895") ? false : stryMutAct_9fa48("894") ? true : (stryCov_9fa48("894", "895", "896", "897"), this.events.length > this.config.storage.maxEvents)) {
        if (stryMutAct_9fa48("898")) {
          {}
        } else {
          stryCov_9fa48("898");
          this.events = stryMutAct_9fa48("899") ? this.events : (stryCov_9fa48("899"), this.events.slice(stryMutAct_9fa48("900") ? +this.config.storage.maxEvents : (stryCov_9fa48("900"), -this.config.storage.maxEvents)));
        }
      }

      // Add to persistent storage if enabled
      if (stryMutAct_9fa48("902") ? false : stryMutAct_9fa48("901") ? true : (stryCov_9fa48("901", "902"), this.config.storage.persistentStorage)) {
        if (stryMutAct_9fa48("903")) {
          {}
        } else {
          stryCov_9fa48("903");
          this.persistentEvents.set(event.id, event);
        }
      }
    }
  }

  /**
   * Emit event to all matching handlers
   */
  private async emitToHandlers(event: BaseEvent): Promise<void> {
    if (stryMutAct_9fa48("904")) {
      {}
    } else {
      stryCov_9fa48("904");
      const handlers = this.handlers.get(event.type);
      if (stryMutAct_9fa48("907") ? !handlers && handlers.size === 0 : stryMutAct_9fa48("906") ? false : stryMutAct_9fa48("905") ? true : (stryCov_9fa48("905", "906", "907"), (stryMutAct_9fa48("908") ? handlers : (stryCov_9fa48("908"), !handlers)) || (stryMutAct_9fa48("910") ? handlers.size !== 0 : stryMutAct_9fa48("909") ? false : (stryCov_9fa48("909", "910"), handlers.size === 0)))) {
        if (stryMutAct_9fa48("911")) {
          {}
        } else {
          stryCov_9fa48("911");
          return;
        }
      }
      const promises: Promise<void>[] = [];
      for (const handler of Array.from(handlers)) {
        if (stryMutAct_9fa48("913")) {
          {}
        } else {
          stryCov_9fa48("913");
          if (stryMutAct_9fa48("915") ? false : stryMutAct_9fa48("914") ? true : (stryCov_9fa48("914", "915"), this.config.asyncHandlers)) {
            if (stryMutAct_9fa48("916")) {
              {}
            } else {
              stryCov_9fa48("916");
              // Async handling with timeout
              const promise = this.executeHandlerWithTimeout(handler, event);
              promises.push(promise);
            }
          } else {
            if (stryMutAct_9fa48("917")) {
              {}
            } else {
              stryCov_9fa48("917");
              // Synchronous handling
              try {
                if (stryMutAct_9fa48("918")) {
                  {}
                } else {
                  stryCov_9fa48("918");
                  const result = handler(event);
                  if (stryMutAct_9fa48("920") ? false : stryMutAct_9fa48("919") ? true : (stryCov_9fa48("919", "920"), result instanceof Promise)) {
                    if (stryMutAct_9fa48("921")) {
                      {}
                    } else {
                      stryCov_9fa48("921");
                      promises.push(result);
                    }
                  }
                }
              } catch (error) {
                if (stryMutAct_9fa48("922")) {
                  {}
                } else {
                  stryCov_9fa48("922");
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
    if (stryMutAct_9fa48("924")) {
      {}
    } else {
      stryCov_9fa48("924");
      return new Promise(resolve => {
        if (stryMutAct_9fa48("925")) {
          {}
        } else {
          stryCov_9fa48("925");
          const timeout = setTimeout(() => {
            if (stryMutAct_9fa48("926")) {
              {}
            } else {
              stryCov_9fa48("926");
              console.warn(`Event handler timeout for ${event.type} after ${this.config.handlerTimeoutMs}ms`);
              resolve();
            }
          }, this.config.handlerTimeoutMs);
          try {
            if (stryMutAct_9fa48("928")) {
              {}
            } else {
              stryCov_9fa48("928");
              const result = handler(event);
              if (stryMutAct_9fa48("930") ? false : stryMutAct_9fa48("929") ? true : (stryCov_9fa48("929", "930"), result instanceof Promise)) {
                if (stryMutAct_9fa48("931")) {
                  {}
                } else {
                  stryCov_9fa48("931");
                  result.then(() => {
                    if (stryMutAct_9fa48("932")) {
                      {}
                    } else {
                      stryCov_9fa48("932");
                      clearTimeout(timeout);
                      resolve();
                    }
                  }).catch(error => {
                    if (stryMutAct_9fa48("933")) {
                      {}
                    } else {
                      stryCov_9fa48("933");
                      clearTimeout(timeout);
                      console.error(`Async event handler error for ${event.type}:`, error);
                      resolve();
                    }
                  });
                }
              } else {
                if (stryMutAct_9fa48("935")) {
                  {}
                } else {
                  stryCov_9fa48("935");
                  clearTimeout(timeout);
                  resolve();
                }
              }
            }
          } catch (error) {
            if (stryMutAct_9fa48("936")) {
              {}
            } else {
              stryCov_9fa48("936");
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
    if (stryMutAct_9fa48("938")) {
      {}
    } else {
      stryCov_9fa48("938");
      if (stryMutAct_9fa48("941") ? filter.types || !filter.types.includes(event.type) : stryMutAct_9fa48("940") ? false : stryMutAct_9fa48("939") ? true : (stryCov_9fa48("939", "940", "941"), filter.types && (stryMutAct_9fa48("942") ? filter.types.includes(event.type) : (stryCov_9fa48("942"), !filter.types.includes(event.type))))) {
        if (stryMutAct_9fa48("943")) {
          {}
        } else {
          stryCov_9fa48("943");
          return stryMutAct_9fa48("944") ? true : (stryCov_9fa48("944"), false);
        }
      }
      if (stryMutAct_9fa48("947") ? filter.severities || !filter.severities.includes(event.severity) : stryMutAct_9fa48("946") ? false : stryMutAct_9fa48("945") ? true : (stryCov_9fa48("945", "946", "947"), filter.severities && (stryMutAct_9fa48("948") ? filter.severities.includes(event.severity) : (stryCov_9fa48("948"), !filter.severities.includes(event.severity))))) {
        if (stryMutAct_9fa48("949")) {
          {}
        } else {
          stryCov_9fa48("949");
          return stryMutAct_9fa48("950") ? true : (stryCov_9fa48("950"), false);
        }
      }
      if (stryMutAct_9fa48("953") ? filter.sources || !filter.sources.includes(event.source) : stryMutAct_9fa48("952") ? false : stryMutAct_9fa48("951") ? true : (stryCov_9fa48("951", "952", "953"), filter.sources && (stryMutAct_9fa48("954") ? filter.sources.includes(event.source) : (stryCov_9fa48("954"), !filter.sources.includes(event.source))))) {
        if (stryMutAct_9fa48("955")) {
          {}
        } else {
          stryCov_9fa48("955");
          return stryMutAct_9fa48("956") ? true : (stryCov_9fa48("956"), false);
        }
      }
      if (stryMutAct_9fa48("959") ? filter.agentIds && event.agentId || !filter.agentIds.includes(event.agentId) : stryMutAct_9fa48("958") ? false : stryMutAct_9fa48("957") ? true : (stryCov_9fa48("957", "958", "959"), (stryMutAct_9fa48("961") ? filter.agentIds || event.agentId : stryMutAct_9fa48("960") ? true : (stryCov_9fa48("960", "961"), filter.agentIds && event.agentId)) && (stryMutAct_9fa48("962") ? filter.agentIds.includes(event.agentId) : (stryCov_9fa48("962"), !filter.agentIds.includes(event.agentId))))) {
        if (stryMutAct_9fa48("963")) {
          {}
        } else {
          stryCov_9fa48("963");
          return stryMutAct_9fa48("964") ? true : (stryCov_9fa48("964"), false);
        }
      }
      if (stryMutAct_9fa48("967") ? filter.taskIds && event.taskId || !filter.taskIds.includes(event.taskId) : stryMutAct_9fa48("966") ? false : stryMutAct_9fa48("965") ? true : (stryCov_9fa48("965", "966", "967"), (stryMutAct_9fa48("969") ? filter.taskIds || event.taskId : stryMutAct_9fa48("968") ? true : (stryCov_9fa48("968", "969"), filter.taskIds && event.taskId)) && (stryMutAct_9fa48("970") ? filter.taskIds.includes(event.taskId) : (stryCov_9fa48("970"), !filter.taskIds.includes(event.taskId))))) {
        if (stryMutAct_9fa48("971")) {
          {}
        } else {
          stryCov_9fa48("971");
          return stryMutAct_9fa48("972") ? true : (stryCov_9fa48("972"), false);
        }
      }
      if (stryMutAct_9fa48("975") ? filter.customFilter || !filter.customFilter(event) : stryMutAct_9fa48("974") ? false : stryMutAct_9fa48("973") ? true : (stryCov_9fa48("973", "974", "975"), filter.customFilter && (stryMutAct_9fa48("976") ? filter.customFilter(event) : (stryCov_9fa48("976"), !filter.customFilter(event))))) {
        if (stryMutAct_9fa48("977")) {
          {}
        } else {
          stryCov_9fa48("977");
          return stryMutAct_9fa48("978") ? true : (stryCov_9fa48("978"), false);
        }
      }
      return stryMutAct_9fa48("979") ? false : (stryCov_9fa48("979"), true);
    }
  }

  /**
   * Start periodic cleanup of old events
   */
  private startCleanupTimer(): void {
    if (stryMutAct_9fa48("980")) {
      {}
    } else {
      stryCov_9fa48("980");
      this.cleanupTimer = setInterval(() => {
        if (stryMutAct_9fa48("981")) {
          {}
        } else {
          stryCov_9fa48("981");
          this.cleanupOldEvents();
        }
      }, 60000); // Clean up every minute
    }
  }

  /**
   * Clean up events older than retention period
   */
  private cleanupOldEvents(): void {
    if (stryMutAct_9fa48("982")) {
      {}
    } else {
      stryCov_9fa48("982");
      const cutoff = stryMutAct_9fa48("983") ? Date.now() + this.config.storage.retentionMs : (stryCov_9fa48("983"), Date.now() - this.config.storage.retentionMs);
      this.events = stryMutAct_9fa48("984") ? this.events : (stryCov_9fa48("984"), this.events.filter(stryMutAct_9fa48("985") ? () => undefined : (stryCov_9fa48("985"), event => stryMutAct_9fa48("989") ? event.timestamp.getTime() <= cutoff : stryMutAct_9fa48("988") ? event.timestamp.getTime() >= cutoff : stryMutAct_9fa48("987") ? false : stryMutAct_9fa48("986") ? true : (stryCov_9fa48("986", "987", "988", "989"), event.timestamp.getTime() > cutoff))));
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
export const globalEventEmitter = new EventEmitter(stryMutAct_9fa48("990") ? {} : (stryCov_9fa48("990"), {
  enabled: stryMutAct_9fa48("991") ? false : (stryCov_9fa48("991"), true),
  storage: stryMutAct_9fa48("992") ? {} : (stryCov_9fa48("992"), {
    maxEvents: 10000,
    retentionMs: stryMutAct_9fa48("993") ? 24 * 60 * 60 / 1000 : (stryCov_9fa48("993"), (stryMutAct_9fa48("994") ? 24 * 60 / 60 : (stryCov_9fa48("994"), (stryMutAct_9fa48("995") ? 24 / 60 : (stryCov_9fa48("995"), 24 * 60)) * 60)) * 1000),
    // 24 hours
    persistentStorage: stryMutAct_9fa48("996") ? true : (stryCov_9fa48("996"), false)
  }),
  asyncHandlers: stryMutAct_9fa48("997") ? false : (stryCov_9fa48("997"), true),
  handlerTimeoutMs: 5000
}));

/**
 * Convenience functions for global event emitter
 */
export const events = stryMutAct_9fa48("998") ? {} : (stryCov_9fa48("998"), {
  emit: stryMutAct_9fa48("999") ? () => undefined : (stryCov_9fa48("999"), <T extends BaseEvent,>(event: T) => globalEventEmitter.emit(event)),
  on: stryMutAct_9fa48("1000") ? () => undefined : (stryCov_9fa48("1000"), <T extends BaseEvent,>(eventType: string, handler: EventHandler<T>) => globalEventEmitter.on(eventType, handler)),
  off: stryMutAct_9fa48("1001") ? () => undefined : (stryCov_9fa48("1001"), <T extends BaseEvent,>(eventType: string, handler: EventHandler<T>) => globalEventEmitter.off(eventType, handler)),
  getEvents: stryMutAct_9fa48("1002") ? () => undefined : (stryCov_9fa48("1002"), (filter?: EventFilter, limit?: number) => globalEventEmitter.getEvents(filter, limit)),
  getStats: stryMutAct_9fa48("1003") ? () => undefined : (stryCov_9fa48("1003"), () => globalEventEmitter.getStats()),
  clear: stryMutAct_9fa48("1004") ? () => undefined : (stryCov_9fa48("1004"), () => globalEventEmitter.clear()),
  shutdown: stryMutAct_9fa48("1005") ? () => undefined : (stryCov_9fa48("1005"), () => globalEventEmitter.shutdown())
});