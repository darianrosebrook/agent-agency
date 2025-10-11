/**
 * @fileoverview Task Queue implementation for Arbiter Orchestration (ARBITER-005)
 *
 * Manages the queue of tasks waiting for routing and assignment to agents.
 * Provides priority-based queuing, capacity management, and thread-safe operations.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck
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
import { ITaskQueue, Task, TaskState, TaskStatus } from "../types/arbiter-orchestration";
import { DatabaseClientFactory, IDatabaseClient } from "./DatabaseClient";
import { events } from "./EventEmitter";
import { Mutex } from "./Mutex";
import { EventSeverity, EventTypes, TaskDequeuedEvent } from "./OrchestratorEvents";
import { AuthCredentials, Permission, SecurityManager } from "./SecurityManager";
import { ValidationUtils, validateTask } from "./Validation";

/**
 * Priority queue implementation with efficient insertion and removal
 */
class PriorityQueue<T> {
  private items: Array<{
    item: T;
    priority: number;
  }> = [];
  enqueue(item: T, priority: number): void {
    if (stryMutAct_9fa48("1289")) {
      {}
    } else {
      stryCov_9fa48("1289");
      const queueItem = stryMutAct_9fa48("1290") ? {} : (stryCov_9fa48("1290"), {
        item,
        priority
      });
      let added = stryMutAct_9fa48("1291") ? true : (stryCov_9fa48("1291"), false);

      // Insert in priority order (higher priority first)
      for (let i = 0; stryMutAct_9fa48("1294") ? i >= this.items.length : stryMutAct_9fa48("1293") ? i <= this.items.length : stryMutAct_9fa48("1292") ? false : (stryCov_9fa48("1292", "1293", "1294"), i < this.items.length); stryMutAct_9fa48("1295") ? i-- : (stryCov_9fa48("1295"), i++)) {
        if (stryMutAct_9fa48("1296")) {
          {}
        } else {
          stryCov_9fa48("1296");
          if (stryMutAct_9fa48("1300") ? priority <= this.items[i].priority : stryMutAct_9fa48("1299") ? priority >= this.items[i].priority : stryMutAct_9fa48("1298") ? false : stryMutAct_9fa48("1297") ? true : (stryCov_9fa48("1297", "1298", "1299", "1300"), priority > this.items[i].priority)) {
            if (stryMutAct_9fa48("1301")) {
              {}
            } else {
              stryCov_9fa48("1301");
              this.items.splice(i, 0, queueItem);
              added = stryMutAct_9fa48("1302") ? false : (stryCov_9fa48("1302"), true);
              break;
            }
          }
        }
      }
      if (stryMutAct_9fa48("1305") ? false : stryMutAct_9fa48("1304") ? true : stryMutAct_9fa48("1303") ? added : (stryCov_9fa48("1303", "1304", "1305"), !added)) {
        if (stryMutAct_9fa48("1306")) {
          {}
        } else {
          stryCov_9fa48("1306");
          this.items.push(queueItem);
        }
      }
    }
  }
  dequeue(): T | null {
    if (stryMutAct_9fa48("1307")) {
      {}
    } else {
      stryCov_9fa48("1307");
      return stryMutAct_9fa48("1308") ? this.items.shift()?.item && null : (stryCov_9fa48("1308"), (stryMutAct_9fa48("1309") ? this.items.shift().item : (stryCov_9fa48("1309"), this.items.shift()?.item)) ?? null);
    }
  }
  peek(): T | null {
    if (stryMutAct_9fa48("1310")) {
      {}
    } else {
      stryCov_9fa48("1310");
      return stryMutAct_9fa48("1311") ? this.items[0]?.item && null : (stryCov_9fa48("1311"), (stryMutAct_9fa48("1312") ? this.items[0].item : (stryCov_9fa48("1312"), this.items[0]?.item)) ?? null);
    }
  }
  size(): number {
    if (stryMutAct_9fa48("1313")) {
      {}
    } else {
      stryCov_9fa48("1313");
      return this.items.length;
    }
  }
  clear(): void {
    if (stryMutAct_9fa48("1314")) {
      {}
    } else {
      stryCov_9fa48("1314");
      this.items = [];
    }
  }
  toArray(): T[] {
    if (stryMutAct_9fa48("1316")) {
      {}
    } else {
      stryCov_9fa48("1316");
      return this.items.map(stryMutAct_9fa48("1317") ? () => undefined : (stryCov_9fa48("1317"), item => item.item));
    }
  }
}

/**
 * Task Queue Configuration
 */
export interface TaskQueueConfig {
  /** Maximum number of tasks that can be queued */
  maxCapacity: number;

  /** Default task timeout in milliseconds */
  defaultTimeoutMs: number;

  /** Maximum retry attempts for failed tasks */
  maxRetries: number;

  /** Queue processing priority mode */
  priorityMode: "fifo" | "priority" | "deadline";

  /** Enable queue persistence */
  persistenceEnabled: boolean;

  /** Database client for persistence */
  databaseClient?: IDatabaseClient;

  /** Security manager for authentication/authorization */
  securityManager?: SecurityManager;
}

/**
 * Task Queue Statistics
 */
export interface TaskQueueStats {
  /** Current queue depth */
  depth: number;

  /** Maximum queue depth reached */
  maxDepth: number;

  /** Total tasks enqueued */
  totalEnqueued: number;

  /** Total tasks dequeued */
  totalDequeued: number;

  /** Average wait time in milliseconds */
  averageWaitTimeMs: number;

  /** Tasks by priority distribution */
  priorityDistribution: Record<number, number>;

  /** Tasks by status distribution */
  statusDistribution: Record<TaskStatus, number>;
}

/**
 * Task Queue Implementation
 *
 * Thread-safe task queue with priority management and capacity controls.
 * Supports multiple priority modes and provides comprehensive statistics.
 */
export class TaskQueue implements ITaskQueue {
  private queue: PriorityQueue<Task>;
  private config: TaskQueueConfig;
  private stats: TaskQueueStats;
  private taskStates: Map<string, TaskState> = new Map();
  private mutex: Mutex = new Mutex();
  private dbClient?: IDatabaseClient;
  private securityManager?: SecurityManager;
  private initialized: boolean = stryMutAct_9fa48("1318") ? true : (stryCov_9fa48("1318"), false);
  constructor(config: Partial<TaskQueueConfig> = {}) {
    if (stryMutAct_9fa48("1319")) {
      {}
    } else {
      stryCov_9fa48("1319");
      const finalConfig = stryMutAct_9fa48("1320") ? {} : (stryCov_9fa48("1320"), {
        maxCapacity: 1000,
        defaultTimeoutMs: 30000,
        maxRetries: 3,
        priorityMode: "priority",
        persistenceEnabled: stryMutAct_9fa48("1322") ? true : (stryCov_9fa48("1322"), false),
        ...config
      });

      // Validate configuration
      const configValidation = ValidationUtils.validateTaskQueueConfig(finalConfig);
      if (stryMutAct_9fa48("1325") ? false : stryMutAct_9fa48("1324") ? true : stryMutAct_9fa48("1323") ? configValidation.isValid : (stryCov_9fa48("1323", "1324", "1325"), !configValidation.isValid)) {
        if (stryMutAct_9fa48("1326")) {
          {}
        } else {
          stryCov_9fa48("1326");
          throw new Error(`Invalid TaskQueue configuration:\n${ValidationUtils.formatValidationResult(configValidation)}`);
        }
      }
      this.config = finalConfig as TaskQueueConfig;

      // Initialize database client if persistence is enabled
      if (stryMutAct_9fa48("1329") ? false : stryMutAct_9fa48("1328") ? true : (stryCov_9fa48("1328", "1329"), this.config.persistenceEnabled)) {
        if (stryMutAct_9fa48("1330")) {
          {}
        } else {
          stryCov_9fa48("1330");
          this.dbClient = stryMutAct_9fa48("1333") ? this.config.databaseClient && DatabaseClientFactory.createMockClient() : stryMutAct_9fa48("1332") ? false : stryMutAct_9fa48("1331") ? true : (stryCov_9fa48("1331", "1332", "1333"), this.config.databaseClient || DatabaseClientFactory.createMockClient());
        }
      }

      // Initialize security manager
      this.securityManager = this.config.securityManager;
      this.queue = new PriorityQueue<Task>();
      this.stats = stryMutAct_9fa48("1334") ? {} : (stryCov_9fa48("1334"), {
        depth: 0,
        maxDepth: 0,
        totalEnqueued: 0,
        totalDequeued: 0,
        averageWaitTimeMs: 0,
        priorityDistribution: {},
        statusDistribution: stryMutAct_9fa48("1335") ? {} : (stryCov_9fa48("1335"), {
          [TaskStatus.QUEUED]: 0,
          [TaskStatus.ROUTING]: 0,
          [TaskStatus.ASSIGNED]: 0,
          [TaskStatus.EXECUTING]: 0,
          [TaskStatus.VALIDATING]: 0,
          [TaskStatus.COMPLETED]: 0,
          [TaskStatus.FAILED]: 0,
          [TaskStatus.TIMEOUT]: 0,
          [TaskStatus.CANCELED]: 0
        })
      });
    }
  }

  /**
   * Initialize the task queue (connect to database, load persisted state)
   */
  async initialize(): Promise<void> {
    if (stryMutAct_9fa48("1336")) {
      {}
    } else {
      stryCov_9fa48("1336");
      if (stryMutAct_9fa48("1338") ? false : stryMutAct_9fa48("1337") ? true : (stryCov_9fa48("1337", "1338"), this.initialized)) {
        if (stryMutAct_9fa48("1339")) {
          {}
        } else {
          stryCov_9fa48("1339");
          return;
        }
      }
      try {
        if (stryMutAct_9fa48("1340")) {
          {}
        } else {
          stryCov_9fa48("1340");
          // Connect to database if persistence is enabled
          if (stryMutAct_9fa48("1343") ? this.config.persistenceEnabled || this.dbClient : stryMutAct_9fa48("1342") ? false : stryMutAct_9fa48("1341") ? true : (stryCov_9fa48("1341", "1342", "1343"), this.config.persistenceEnabled && this.dbClient)) {
            if (stryMutAct_9fa48("1344")) {
              {}
            } else {
              stryCov_9fa48("1344");
              await this.dbClient.connect();

              // Load persisted tasks and restore queue state
              await this.loadPersistedState();
            }
          }
          this.initialized = stryMutAct_9fa48("1345") ? false : (stryCov_9fa48("1345"), true);
          console.log("TaskQueue initialized successfully");
        }
      } catch (error) {
        if (stryMutAct_9fa48("1347")) {
          {}
        } else {
          stryCov_9fa48("1347");
          console.error("Failed to initialize TaskQueue:", error);
          throw error;
        }
      }
    }
  }

  /**
   * Shutdown the task queue (disconnect from database)
   */
  async shutdown(): Promise<void> {
    if (stryMutAct_9fa48("1349")) {
      {}
    } else {
      stryCov_9fa48("1349");
      try {
        if (stryMutAct_9fa48("1350")) {
          {}
        } else {
          stryCov_9fa48("1350");
          if (stryMutAct_9fa48("1353") ? this.config.persistenceEnabled || this.dbClient : stryMutAct_9fa48("1352") ? false : stryMutAct_9fa48("1351") ? true : (stryCov_9fa48("1351", "1352", "1353"), this.config.persistenceEnabled && this.dbClient)) {
            if (stryMutAct_9fa48("1354")) {
              {}
            } else {
              stryCov_9fa48("1354");
              await this.dbClient.disconnect();
            }
          }
          this.initialized = stryMutAct_9fa48("1355") ? true : (stryCov_9fa48("1355"), false);
          console.log("TaskQueue shutdown successfully");
        }
      } catch (error) {
        if (stryMutAct_9fa48("1357")) {
          {}
        } else {
          stryCov_9fa48("1357");
          console.error("Error during TaskQueue shutdown:", error);
        }
      }
    }
  }

  /**
   * Load persisted state from database
   */
  private async loadPersistedState(): Promise<void> {
    if (stryMutAct_9fa48("1359")) {
      {}
    } else {
      stryCov_9fa48("1359");
      if (stryMutAct_9fa48("1362") ? false : stryMutAct_9fa48("1361") ? true : stryMutAct_9fa48("1360") ? this.dbClient : (stryCov_9fa48("1360", "1361", "1362"), !this.dbClient)) {
        if (stryMutAct_9fa48("1363")) {
          {}
        } else {
          stryCov_9fa48("1363");
          return;
        }
      }
      try {
        if (stryMutAct_9fa48("1364")) {
          {}
        } else {
          stryCov_9fa48("1364");
          // Load queued tasks from database
          const result = await this.dbClient.query(`
        SELECT * FROM task_queue
        WHERE status = 'queued'
        ORDER BY priority DESC, created_at ASC
      `);
          for (const row of result.rows) {
            if (stryMutAct_9fa48("1366")) {
              {}
            } else {
              stryCov_9fa48("1366");
              const task: Task = stryMutAct_9fa48("1367") ? {} : (stryCov_9fa48("1367"), {
                id: row.task_id,
                description: row.description,
                type: row.task_type,
                priority: row.priority,
                timeoutMs: row.timeout_ms,
                attempts: row.attempts,
                maxAttempts: row.max_attempts,
                requiredCapabilities: stryMutAct_9fa48("1370") ? row.required_capabilities && {} : stryMutAct_9fa48("1369") ? false : stryMutAct_9fa48("1368") ? true : (stryCov_9fa48("1368", "1369", "1370"), row.required_capabilities || {}),
                budget: stryMutAct_9fa48("1371") ? {} : (stryCov_9fa48("1371"), {
                  maxFiles: row.budget_max_files,
                  maxLoc: row.budget_max_loc
                }),
                createdAt: new Date(row.created_at),
                metadata: stryMutAct_9fa48("1374") ? row.task_metadata && {} : stryMutAct_9fa48("1373") ? false : stryMutAct_9fa48("1372") ? true : (stryCov_9fa48("1372", "1373", "1374"), row.task_metadata || {})
              });

              // Restore task state
              const taskState: TaskState = stryMutAct_9fa48("1375") ? {} : (stryCov_9fa48("1375"), {
                task,
                status: TaskStatus.QUEUED,
                attempts: row.attempts,
                maxAttempts: row.max_attempts,
                routingHistory: []
              });
              this.queue.enqueue(task, this.calculatePriority(task));
              this.taskStates.set(task.id, taskState);

              // Update stats for loaded task
              stryMutAct_9fa48("1377") ? this.stats.depth-- : (stryCov_9fa48("1377"), this.stats.depth++);
              this.stats.maxDepth = stryMutAct_9fa48("1378") ? Math.min(this.stats.maxDepth, this.stats.depth) : (stryCov_9fa48("1378"), Math.max(this.stats.maxDepth, this.stats.depth));
              stryMutAct_9fa48("1379") ? this.stats.statusDistribution[TaskStatus.QUEUED]-- : (stryCov_9fa48("1379"), this.stats.statusDistribution[TaskStatus.QUEUED]++);
            }
          }
          console.log(`Loaded ${result.rows.length} persisted tasks from database`);
        }
      } catch (error) {
        if (stryMutAct_9fa48("1381")) {
          {}
        } else {
          stryCov_9fa48("1381");
          console.error("Failed to load persisted state:", error);
          throw error;
        }
      }
    }
  }

  /**
   * Persist a task to the database
   */
  private async persistTask(task: Task, taskState: TaskState): Promise<void> {
    if (stryMutAct_9fa48("1383")) {
      {}
    } else {
      stryCov_9fa48("1383");
      if (stryMutAct_9fa48("1386") ? false : stryMutAct_9fa48("1385") ? true : stryMutAct_9fa48("1384") ? this.dbClient : (stryCov_9fa48("1384", "1385", "1386"), !this.dbClient)) {
        if (stryMutAct_9fa48("1387")) {
          {}
        } else {
          stryCov_9fa48("1387");
          return;
        }
      }
      try {
        if (stryMutAct_9fa48("1388")) {
          {}
        } else {
          stryCov_9fa48("1388");
          await this.dbClient.query(`
        INSERT INTO task_queue (
          task_id, task_type, description, priority, timeout_ms,
          attempts, max_attempts, budget_max_files, budget_max_loc,
          required_capabilities, task_metadata, status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (task_id) DO UPDATE SET
          status = EXCLUDED.status,
          attempts = EXCLUDED.attempts,
          updated_at = NOW()
      `, [task.id, task.type, task.description, stryMutAct_9fa48("1393") ? task.priority && 1 : stryMutAct_9fa48("1392") ? false : stryMutAct_9fa48("1391") ? true : (stryCov_9fa48("1391", "1392", "1393"), task.priority || 1), stryMutAct_9fa48("1396") ? task.timeoutMs && this.config.defaultTimeoutMs : stryMutAct_9fa48("1395") ? false : stryMutAct_9fa48("1394") ? true : (stryCov_9fa48("1394", "1395", "1396"), task.timeoutMs || this.config.defaultTimeoutMs), taskState.attempts, taskState.maxAttempts, stryMutAct_9fa48("1399") ? task.budget?.maxFiles && 40 : stryMutAct_9fa48("1398") ? false : stryMutAct_9fa48("1397") ? true : (stryCov_9fa48("1397", "1398", "1399"), (stryMutAct_9fa48("1400") ? task.budget.maxFiles : (stryCov_9fa48("1400"), task.budget?.maxFiles)) || 40), stryMutAct_9fa48("1403") ? task.budget?.maxLoc && 1500 : stryMutAct_9fa48("1402") ? false : stryMutAct_9fa48("1401") ? true : (stryCov_9fa48("1401", "1402", "1403"), (stryMutAct_9fa48("1404") ? task.budget.maxLoc : (stryCov_9fa48("1404"), task.budget?.maxLoc)) || 1500), JSON.stringify(stryMutAct_9fa48("1407") ? task.requiredCapabilities && {} : stryMutAct_9fa48("1406") ? false : stryMutAct_9fa48("1405") ? true : (stryCov_9fa48("1405", "1406", "1407"), task.requiredCapabilities || {})), JSON.stringify(stryMutAct_9fa48("1410") ? task.metadata && {} : stryMutAct_9fa48("1409") ? false : stryMutAct_9fa48("1408") ? true : (stryCov_9fa48("1408", "1409", "1410"), task.metadata || {})), "queued"]);
        }
      } catch (error) {
        if (stryMutAct_9fa48("1412")) {
          {}
        } else {
          stryCov_9fa48("1412");
          console.error(`Failed to persist task ${task.id}:`, error);
          // Don't throw - queue should continue working even if persistence fails
        }
      }
    }
  }

  /**
   * Update task status in database
   */
  private async updateTaskStatusInDb(taskId: string, status: TaskStatus): Promise<void> {
    if (stryMutAct_9fa48("1414")) {
      {}
    } else {
      stryCov_9fa48("1414");
      if (stryMutAct_9fa48("1417") ? !this.config.persistenceEnabled && !this.dbClient : stryMutAct_9fa48("1416") ? false : stryMutAct_9fa48("1415") ? true : (stryCov_9fa48("1415", "1416", "1417"), (stryMutAct_9fa48("1418") ? this.config.persistenceEnabled : (stryCov_9fa48("1418"), !this.config.persistenceEnabled)) || (stryMutAct_9fa48("1419") ? this.dbClient : (stryCov_9fa48("1419"), !this.dbClient)))) {
        if (stryMutAct_9fa48("1420")) {
          {}
        } else {
          stryCov_9fa48("1420");
          return;
        }
      }
      try {
        if (stryMutAct_9fa48("1421")) {
          {}
        } else {
          stryCov_9fa48("1421");
          await this.dbClient.query(`
        UPDATE task_queue
        SET status = $1, updated_at = NOW()
        WHERE task_id = $2
      `, [status, taskId]);
        }
      } catch (error) {
        if (stryMutAct_9fa48("1424")) {
          {}
        } else {
          stryCov_9fa48("1424");
          console.error(`Failed to update task status ${taskId}:`, error);
        }
      }
    }
  }

  /**
   * Enqueue a task for processing (standard interface method - no auth)
   */
  async enqueue(task: Task): Promise<void> {
    if (stryMutAct_9fa48("1426")) {
      {}
    } else {
      stryCov_9fa48("1426");
      // For backward compatibility - no security checks
      // Validate input
      validateTask(task);
      await this.acquireLock();
      try {
        if (stryMutAct_9fa48("1427")) {
          {}
        } else {
          stryCov_9fa48("1427");
          // Check capacity
          if (stryMutAct_9fa48("1431") ? this.stats.depth < this.config.maxCapacity : stryMutAct_9fa48("1430") ? this.stats.depth > this.config.maxCapacity : stryMutAct_9fa48("1429") ? false : stryMutAct_9fa48("1428") ? true : (stryCov_9fa48("1428", "1429", "1430", "1431"), this.stats.depth >= this.config.maxCapacity)) {
            if (stryMutAct_9fa48("1432")) {
              {}
            } else {
              stryCov_9fa48("1432");
              throw new Error(`Queue capacity exceeded: ${this.config.maxCapacity}`);
            }
          }

          // Set default timeout if not provided
          const timeoutMs = stryMutAct_9fa48("1436") ? task.timeoutMs && this.config.defaultTimeoutMs : stryMutAct_9fa48("1435") ? false : stryMutAct_9fa48("1434") ? true : (stryCov_9fa48("1434", "1435", "1436"), task.timeoutMs || this.config.defaultTimeoutMs);

          // Create task state
          const taskState: TaskState = stryMutAct_9fa48("1437") ? {} : (stryCov_9fa48("1437"), {
            task: stryMutAct_9fa48("1438") ? {} : (stryCov_9fa48("1438"), {
              ...task,
              timeoutMs
            }),
            status: TaskStatus.QUEUED,
            attempts: 0,
            maxAttempts: this.config.maxRetries,
            routingHistory: []
          });

          // Store task state
          this.taskStates.set(task.id, taskState);

          // Add to queue with appropriate priority
          const priority = this.calculatePriority(task);
          this.queue.enqueue(task, priority);

          // Persist to database if enabled
          if (stryMutAct_9fa48("1442") ? this.config.persistenceEnabled || this.dbClient : stryMutAct_9fa48("1441") ? false : stryMutAct_9fa48("1440") ? true : (stryCov_9fa48("1440", "1441", "1442"), this.config.persistenceEnabled && this.dbClient)) {
            if (stryMutAct_9fa48("1443")) {
              {}
            } else {
              stryCov_9fa48("1443");
              await this.persistTask(task, taskState);
            }
          }

          // Update statistics
          stryMutAct_9fa48("1444") ? this.stats.depth-- : (stryCov_9fa48("1444"), this.stats.depth++);
          this.stats.maxDepth = stryMutAct_9fa48("1445") ? Math.min(this.stats.maxDepth, this.stats.depth) : (stryCov_9fa48("1445"), Math.max(this.stats.maxDepth, this.stats.depth));
          stryMutAct_9fa48("1446") ? this.stats.totalEnqueued-- : (stryCov_9fa48("1446"), this.stats.totalEnqueued++);
          this.stats.priorityDistribution[priority] = stryMutAct_9fa48("1447") ? (this.stats.priorityDistribution[priority] || 0) - 1 : (stryCov_9fa48("1447"), (stryMutAct_9fa48("1450") ? this.stats.priorityDistribution[priority] && 0 : stryMutAct_9fa48("1449") ? false : stryMutAct_9fa48("1448") ? true : (stryCov_9fa48("1448", "1449", "1450"), this.stats.priorityDistribution[priority] || 0)) + 1);
          stryMutAct_9fa48("1451") ? this.stats.statusDistribution[TaskStatus.QUEUED]-- : (stryCov_9fa48("1451"), this.stats.statusDistribution[TaskStatus.QUEUED]++);
        }
      } finally {
        if (stryMutAct_9fa48("1452")) {
          {}
        } else {
          stryCov_9fa48("1452");
          this.releaseLock();
        }
      }
    }
  }

  /**
   * Enqueue a task for processing (requires authentication)
   */
  async enqueueWithCredentials(task: Task, credentials: AuthCredentials): Promise<void> {
    if (stryMutAct_9fa48("1453")) {
      {}
    } else {
      stryCov_9fa48("1453");
      // Validate input
      validateTask(task);

      // Authenticate and authorize
      if (stryMutAct_9fa48("1455") ? false : stryMutAct_9fa48("1454") ? true : (stryCov_9fa48("1454", "1455"), this.securityManager)) {
        if (stryMutAct_9fa48("1456")) {
          {}
        } else {
          stryCov_9fa48("1456");
          const context = this.securityManager.authenticate(credentials);
          if (stryMutAct_9fa48("1459") ? false : stryMutAct_9fa48("1458") ? true : stryMutAct_9fa48("1457") ? context : (stryCov_9fa48("1457", "1458", "1459"), !context)) {
            if (stryMutAct_9fa48("1460")) {
              {}
            } else {
              stryCov_9fa48("1460");
              throw new Error("Authentication failed");
            }
          }
          if (stryMutAct_9fa48("1464") ? false : stryMutAct_9fa48("1463") ? true : stryMutAct_9fa48("1462") ? this.securityManager.authorize(context, Permission.SUBMIT_TASK) : (stryCov_9fa48("1462", "1463", "1464"), !this.securityManager.authorize(context, Permission.SUBMIT_TASK))) {
            if (stryMutAct_9fa48("1465")) {
              {}
            } else {
              stryCov_9fa48("1465");
              throw new Error("Authorization failed: insufficient permissions");
            }
          }
          if (stryMutAct_9fa48("1469") ? false : stryMutAct_9fa48("1468") ? true : stryMutAct_9fa48("1467") ? this.securityManager.checkRateLimit(context, "submitTask") : (stryCov_9fa48("1467", "1468", "1469"), !this.securityManager.checkRateLimit(context, "submitTask"))) {
            if (stryMutAct_9fa48("1471")) {
              {}
            } else {
              stryCov_9fa48("1471");
              throw new Error("Rate limit exceeded for task submission");
            }
          }

          // Sanitize input
          this.securityManager.sanitizeInput(context, "enqueue_task", task);
        }
      }
      await this.acquireLock();
      try {
        if (stryMutAct_9fa48("1474")) {
          {}
        } else {
          stryCov_9fa48("1474");
          // Check capacity
          if (stryMutAct_9fa48("1478") ? this.stats.depth < this.config.maxCapacity : stryMutAct_9fa48("1477") ? this.stats.depth > this.config.maxCapacity : stryMutAct_9fa48("1476") ? false : stryMutAct_9fa48("1475") ? true : (stryCov_9fa48("1475", "1476", "1477", "1478"), this.stats.depth >= this.config.maxCapacity)) {
            if (stryMutAct_9fa48("1479")) {
              {}
            } else {
              stryCov_9fa48("1479");
              throw new Error(`Queue capacity exceeded: ${this.config.maxCapacity}`);
            }
          }

          // Set default timeout if not provided
          const timeoutMs = stryMutAct_9fa48("1483") ? task.timeoutMs && this.config.defaultTimeoutMs : stryMutAct_9fa48("1482") ? false : stryMutAct_9fa48("1481") ? true : (stryCov_9fa48("1481", "1482", "1483"), task.timeoutMs || this.config.defaultTimeoutMs);

          // Create task state
          const taskState: TaskState = stryMutAct_9fa48("1484") ? {} : (stryCov_9fa48("1484"), {
            task: stryMutAct_9fa48("1485") ? {} : (stryCov_9fa48("1485"), {
              ...task,
              timeoutMs
            }),
            status: TaskStatus.QUEUED,
            attempts: 0,
            maxAttempts: this.config.maxRetries,
            routingHistory: []
          });

          // Store task state
          this.taskStates.set(task.id, taskState);

          // Add to queue with appropriate priority
          const priority = this.calculatePriority(task);
          this.queue.enqueue(task, priority);

          // Persist to database if enabled
          if (stryMutAct_9fa48("1489") ? this.config.persistenceEnabled || this.dbClient : stryMutAct_9fa48("1488") ? false : stryMutAct_9fa48("1487") ? true : (stryCov_9fa48("1487", "1488", "1489"), this.config.persistenceEnabled && this.dbClient)) {
            if (stryMutAct_9fa48("1490")) {
              {}
            } else {
              stryCov_9fa48("1490");
              await this.persistTask(task, taskState);
            }
          }

          // Update statistics
          stryMutAct_9fa48("1491") ? this.stats.depth-- : (stryCov_9fa48("1491"), this.stats.depth++);
          this.stats.maxDepth = stryMutAct_9fa48("1492") ? Math.min(this.stats.maxDepth, this.stats.depth) : (stryCov_9fa48("1492"), Math.max(this.stats.maxDepth, this.stats.depth));
          stryMutAct_9fa48("1493") ? this.stats.totalEnqueued-- : (stryCov_9fa48("1493"), this.stats.totalEnqueued++);
          this.stats.priorityDistribution[priority] = stryMutAct_9fa48("1494") ? (this.stats.priorityDistribution[priority] || 0) - 1 : (stryCov_9fa48("1494"), (stryMutAct_9fa48("1497") ? this.stats.priorityDistribution[priority] && 0 : stryMutAct_9fa48("1496") ? false : stryMutAct_9fa48("1495") ? true : (stryCov_9fa48("1495", "1496", "1497"), this.stats.priorityDistribution[priority] || 0)) + 1);
          stryMutAct_9fa48("1498") ? this.stats.statusDistribution[TaskStatus.QUEUED]-- : (stryCov_9fa48("1498"), this.stats.statusDistribution[TaskStatus.QUEUED]++);
        }
      } finally {
        if (stryMutAct_9fa48("1499")) {
          {}
        } else {
          stryCov_9fa48("1499");
          this.releaseLock();
        }
      }
    }
  }

  /**
   * Dequeue the next task for processing
   */
  async dequeue(): Promise<Task | null> {
    if (stryMutAct_9fa48("1500")) {
      {}
    } else {
      stryCov_9fa48("1500");
      await this.acquireLock();
      try {
        if (stryMutAct_9fa48("1501")) {
          {}
        } else {
          stryCov_9fa48("1501");
          const task = this.queue.dequeue();
          if (stryMutAct_9fa48("1503") ? false : stryMutAct_9fa48("1502") ? true : (stryCov_9fa48("1502", "1503"), task)) {
            if (stryMutAct_9fa48("1504")) {
              {}
            } else {
              stryCov_9fa48("1504");
              // Update task state
              const taskState = this.taskStates.get(task.id);
              if (stryMutAct_9fa48("1506") ? false : stryMutAct_9fa48("1505") ? true : (stryCov_9fa48("1505", "1506"), taskState)) {
                if (stryMutAct_9fa48("1507")) {
                  {}
                } else {
                  stryCov_9fa48("1507");
                  taskState.status = TaskStatus.ROUTING;
                  taskState.routingHistory = [];
                  this.updateStatusStats(TaskStatus.QUEUED, TaskStatus.ROUTING);

                  // Update status in database
                  await this.updateTaskStatusInDb(task.id, TaskStatus.ROUTING);
                }
              }

              // Update statistics
              stryMutAct_9fa48("1509") ? this.stats.depth++ : (stryCov_9fa48("1509"), this.stats.depth--);
              stryMutAct_9fa48("1510") ? this.stats.totalDequeued-- : (stryCov_9fa48("1510"), this.stats.totalDequeued++);

              // Emit task dequeued event
              const dequeuedEvent: TaskDequeuedEvent = stryMutAct_9fa48("1511") ? {} : (stryCov_9fa48("1511"), {
                id: `event-${Date.now()}-${stryMutAct_9fa48("1513") ? Math.random().toString(36) : (stryCov_9fa48("1513"), Math.random().toString(36).substr(2, 9))}`,
                type: EventTypes.TASK_DEQUEUED,
                timestamp: new Date(),
                severity: EventSeverity.INFO,
                source: "TaskQueue",
                taskId: task.id,
                queueDepth: this.stats.depth,
                waitTimeMs: this.calculateWaitTime(task.id),
                metadata: stryMutAct_9fa48("1515") ? {} : (stryCov_9fa48("1515"), {
                  taskType: task.type,
                  priority: task.priority
                })
              });
              events.emit(dequeuedEvent);
            }
          }
          return stryMutAct_9fa48("1518") ? task && null : stryMutAct_9fa48("1517") ? false : stryMutAct_9fa48("1516") ? true : (stryCov_9fa48("1516", "1517", "1518"), task || null);
        }
      } finally {
        if (stryMutAct_9fa48("1519")) {
          {}
        } else {
          stryCov_9fa48("1519");
          this.releaseLock();
        }
      }
    }
  }

  /**
   * Peek at the next task without removing it
   */
  async peek(): Promise<Task | null> {
    if (stryMutAct_9fa48("1520")) {
      {}
    } else {
      stryCov_9fa48("1520");
      // Peek is read-only, no lock needed
      return this.queue.peek();
    }
  }

  /**
   * Get current queue size
   */
  async size(): Promise<number> {
    if (stryMutAct_9fa48("1521")) {
      {}
    } else {
      stryCov_9fa48("1521");
      return this.stats.depth;
    }
  }

  /**
   * Clear all tasks from the queue
   */
  async clear(): Promise<void> {
    if (stryMutAct_9fa48("1522")) {
      {}
    } else {
      stryCov_9fa48("1522");
      await this.acquireLock();
      try {
        if (stryMutAct_9fa48("1523")) {
          {}
        } else {
          stryCov_9fa48("1523");
          // Cancel all queued tasks
          const queuedTasks = this.queue.toArray();
          for (const task of queuedTasks) {
            if (stryMutAct_9fa48("1524")) {
              {}
            } else {
              stryCov_9fa48("1524");
              const taskState = this.taskStates.get(task.id);
              if (stryMutAct_9fa48("1526") ? false : stryMutAct_9fa48("1525") ? true : (stryCov_9fa48("1525", "1526"), taskState)) {
                if (stryMutAct_9fa48("1527")) {
                  {}
                } else {
                  stryCov_9fa48("1527");
                  taskState.status = TaskStatus.CANCELED;
                  taskState.lastError = "Queue cleared";
                  this.updateStatusStats(TaskStatus.QUEUED, TaskStatus.CANCELED);

                  // Update status in database
                  await this.updateTaskStatusInDb(task.id, TaskStatus.CANCELED);
                }
              }
            }
          }

          // Clear queue and reset statistics
          this.queue.clear();
          this.stats.depth = 0;
          this.taskStates.clear();
        }
      } finally {
        if (stryMutAct_9fa48("1529")) {
          {}
        } else {
          stryCov_9fa48("1529");
          this.releaseLock();
        }
      }
    }
  }

  /**
   * Get task state by ID
   */
  getTaskState(taskId: string): TaskState | null {
    if (stryMutAct_9fa48("1530")) {
      {}
    } else {
      stryCov_9fa48("1530");
      return stryMutAct_9fa48("1533") ? this.taskStates.get(taskId) && null : stryMutAct_9fa48("1532") ? false : stryMutAct_9fa48("1531") ? true : (stryCov_9fa48("1531", "1532", "1533"), this.taskStates.get(taskId) || null);
    }
  }

  /**
   * Update task status
   */
  updateTaskStatus(taskId: string, status: TaskStatus, error?: string): void {
    if (stryMutAct_9fa48("1534")) {
      {}
    } else {
      stryCov_9fa48("1534");
      const taskState = this.taskStates.get(taskId);
      if (stryMutAct_9fa48("1536") ? false : stryMutAct_9fa48("1535") ? true : (stryCov_9fa48("1535", "1536"), taskState)) {
        if (stryMutAct_9fa48("1537")) {
          {}
        } else {
          stryCov_9fa48("1537");
          const oldStatus = taskState.status;
          taskState.status = status;
          if (stryMutAct_9fa48("1539") ? false : stryMutAct_9fa48("1538") ? true : (stryCov_9fa48("1538", "1539"), error)) {
            if (stryMutAct_9fa48("1540")) {
              {}
            } else {
              stryCov_9fa48("1540");
              taskState.lastError = error;
            }
          }
          this.updateStatusStats(oldStatus, status);

          // Set timestamps
          if (stryMutAct_9fa48("1543") ? status !== TaskStatus.ASSIGNED : stryMutAct_9fa48("1542") ? false : stryMutAct_9fa48("1541") ? true : (stryCov_9fa48("1541", "1542", "1543"), status === TaskStatus.ASSIGNED)) {
            if (stryMutAct_9fa48("1544")) {
              {}
            } else {
              stryCov_9fa48("1544");
              taskState.startedAt = new Date();
            }
          } else if (stryMutAct_9fa48("1546") ? false : stryMutAct_9fa48("1545") ? true : (stryCov_9fa48("1545", "1546"), [TaskStatus.COMPLETED, TaskStatus.FAILED, TaskStatus.TIMEOUT, TaskStatus.CANCELED].includes(status))) {
            if (stryMutAct_9fa48("1548")) {
              {}
            } else {
              stryCov_9fa48("1548");
              taskState.completedAt = new Date();
            }
          }
        }
      }
    }
  }

  /**
   * Record routing decision
   */
  recordRoutingDecision(taskId: string, decision: any): void {
    if (stryMutAct_9fa48("1549")) {
      {}
    } else {
      stryCov_9fa48("1549");
      const taskState = this.taskStates.get(taskId);
      if (stryMutAct_9fa48("1551") ? false : stryMutAct_9fa48("1550") ? true : (stryCov_9fa48("1550", "1551"), taskState)) {
        if (stryMutAct_9fa48("1552")) {
          {}
        } else {
          stryCov_9fa48("1552");
          taskState.routingHistory.push(decision);
        }
      }
    }
  }

  /**
   * Get queue statistics
   */
  getStats(): TaskQueueStats {
    if (stryMutAct_9fa48("1553")) {
      {}
    } else {
      stryCov_9fa48("1553");
      return stryMutAct_9fa48("1554") ? {} : (stryCov_9fa48("1554"), {
        ...this.stats
      });
    }
  }

  /**
   * Get all queued tasks
   */
  getQueuedTasks(): Task[] {
    if (stryMutAct_9fa48("1555")) {
      {}
    } else {
      stryCov_9fa48("1555");
      return this.queue.toArray();
    }
  }

  /**
   * Calculate task priority based on configuration
   */
  private calculatePriority(task: Task): number {
    if (stryMutAct_9fa48("1556")) {
      {}
    } else {
      stryCov_9fa48("1556");
      switch (this.config.priorityMode) {
        case "fifo":
          if (stryMutAct_9fa48("1557")) {} else {
            stryCov_9fa48("1557");
            // Use creation timestamp for FIFO ordering
            return stryMutAct_9fa48("1559") ? +task.createdAt.getTime() : (stryCov_9fa48("1559"), -task.createdAt.getTime());
          }
        case "priority":
          if (stryMutAct_9fa48("1560")) {} else {
            stryCov_9fa48("1560");
            // Use task priority directly
            return task.priority;
          }
        case "deadline":
          if (stryMutAct_9fa48("1562")) {} else {
            stryCov_9fa48("1562");
            {
              if (stryMutAct_9fa48("1564")) {
                {}
              } else {
                stryCov_9fa48("1564");
                // Calculate urgency based on deadline proximity
                const timeToDeadline = stryMutAct_9fa48("1565") ? task.createdAt.getTime() + task.timeoutMs + Date.now() : (stryCov_9fa48("1565"), (stryMutAct_9fa48("1566") ? task.createdAt.getTime() - task.timeoutMs : (stryCov_9fa48("1566"), task.createdAt.getTime() + task.timeoutMs)) - Date.now());
                const urgency = stryMutAct_9fa48("1567") ? Math.min(0, 1 - timeToDeadline / (24 * 60 * 60 * 1000)) : (stryCov_9fa48("1567"), Math.max(0, stryMutAct_9fa48("1568") ? 1 + timeToDeadline / (24 * 60 * 60 * 1000) : (stryCov_9fa48("1568"), 1 - (stryMutAct_9fa48("1569") ? timeToDeadline * (24 * 60 * 60 * 1000) : (stryCov_9fa48("1569"), timeToDeadline / (stryMutAct_9fa48("1570") ? 24 * 60 * 60 / 1000 : (stryCov_9fa48("1570"), (stryMutAct_9fa48("1571") ? 24 * 60 / 60 : (stryCov_9fa48("1571"), (stryMutAct_9fa48("1572") ? 24 / 60 : (stryCov_9fa48("1572"), 24 * 60)) * 60)) * 1000))))))); // 0-1 over 24 hours
                return stryMutAct_9fa48("1573") ? task.priority - urgency * 10 : (stryCov_9fa48("1573"), task.priority + (stryMutAct_9fa48("1574") ? urgency / 10 : (stryCov_9fa48("1574"), urgency * 10)));
              }
            }
          }
        default:
          if (stryMutAct_9fa48("1575")) {} else {
            stryCov_9fa48("1575");
            return task.priority;
          }
      }
    }
  }

  /**
   * Update status distribution statistics
   */
  private updateStatusStats(fromStatus: TaskStatus, toStatus: TaskStatus): void {
    if (stryMutAct_9fa48("1576")) {
      {}
    } else {
      stryCov_9fa48("1576");
      stryMutAct_9fa48("1577") ? this.stats.statusDistribution[fromStatus]++ : (stryCov_9fa48("1577"), this.stats.statusDistribution[fromStatus]--);
      stryMutAct_9fa48("1578") ? this.stats.statusDistribution[toStatus]-- : (stryCov_9fa48("1578"), this.stats.statusDistribution[toStatus]++);
    }
  }

  /**
   * Calculate estimated wait time for a task
   */
  private calculateEstimatedWaitTime(task: Task): number {
    if (stryMutAct_9fa48("1579")) {
      {}
    } else {
      stryCov_9fa48("1579");
      // Simple estimation based on queue depth and priority
      const baseTime = 5000; // 5 seconds base
      const depthMultiplier = stryMutAct_9fa48("1580") ? Math.max(this.stats.depth * 0.1, 2) : (stryCov_9fa48("1580"), Math.min(stryMutAct_9fa48("1581") ? this.stats.depth / 0.1 : (stryCov_9fa48("1581"), this.stats.depth * 0.1), 2)); // Max 2x multiplier
      const priorityMultiplier = (stryMutAct_9fa48("1585") ? task.priority > 5 : stryMutAct_9fa48("1584") ? task.priority < 5 : stryMutAct_9fa48("1583") ? false : stryMutAct_9fa48("1582") ? true : (stryCov_9fa48("1582", "1583", "1584", "1585"), task.priority <= 5)) ? 1 : 0.5; // High priority waits less
      return Math.round(stryMutAct_9fa48("1586") ? baseTime * depthMultiplier / priorityMultiplier : (stryCov_9fa48("1586"), (stryMutAct_9fa48("1587") ? baseTime / depthMultiplier : (stryCov_9fa48("1587"), baseTime * depthMultiplier)) * priorityMultiplier));
    }
  }

  /**
   * Calculate actual wait time for a dequeued task
   */
  private calculateWaitTime(taskId: string): number {
    if (stryMutAct_9fa48("1588")) {
      {}
    } else {
      stryCov_9fa48("1588");
      const taskState = this.taskStates.get(taskId);
      if (stryMutAct_9fa48("1591") ? false : stryMutAct_9fa48("1590") ? true : stryMutAct_9fa48("1589") ? taskState : (stryCov_9fa48("1589", "1590", "1591"), !taskState)) return 0;
      const enqueuedAt = stryMutAct_9fa48("1594") ? (taskState as any).enqueuedAt && new Date() : stryMutAct_9fa48("1593") ? false : stryMutAct_9fa48("1592") ? true : (stryCov_9fa48("1592", "1593", "1594"), (taskState as any).enqueuedAt || new Date());
      return stryMutAct_9fa48("1595") ? Date.now() + enqueuedAt.getTime() : (stryCov_9fa48("1595"), Date.now() - enqueuedAt.getTime());
    }
  }

  /**
   * Acquire exclusive lock for queue operations
   */
  private async acquireLock(): Promise<void> {
    if (stryMutAct_9fa48("1596")) {
      {}
    } else {
      stryCov_9fa48("1596");
      await this.mutex.acquire();
    }
  }

  /**
   * Release exclusive lock
   */
  private releaseLock(): void {
    if (stryMutAct_9fa48("1597")) {
      {}
    } else {
      stryCov_9fa48("1597");
      this.mutex.release();
    }
  }
}

/**
 * Secure Task Queue wrapper that adds authentication/authorization
 */
export class SecureTaskQueue implements ITaskQueue {
  constructor(private taskQueue: TaskQueue, private securityManager: SecurityManager) {}
  async enqueue(task: Task, credentials?: AuthCredentials): Promise<void> {
    if (stryMutAct_9fa48("1598")) {
      {}
    } else {
      stryCov_9fa48("1598");
      if (stryMutAct_9fa48("1601") ? false : stryMutAct_9fa48("1600") ? true : stryMutAct_9fa48("1599") ? credentials : (stryCov_9fa48("1599", "1600", "1601"), !credentials)) {
        if (stryMutAct_9fa48("1602")) {
          {}
        } else {
          stryCov_9fa48("1602");
          throw new Error("Authentication credentials required for secure queue");
        }
      }
      return this.taskQueue.enqueueWithCredentials(task, credentials);
    }
  }
  async dequeue(): Promise<Task | null> {
    if (stryMutAct_9fa48("1604")) {
      {}
    } else {
      stryCov_9fa48("1604");
      return this.taskQueue.dequeue();
    }
  }
  async peek(): Promise<Task | null> {
    if (stryMutAct_9fa48("1605")) {
      {}
    } else {
      stryCov_9fa48("1605");
      return this.taskQueue.peek();
    }
  }
  async size(): Promise<number> {
    if (stryMutAct_9fa48("1606")) {
      {}
    } else {
      stryCov_9fa48("1606");
      return this.taskQueue.size();
    }
  }
  async clear(): Promise<void> {
    if (stryMutAct_9fa48("1607")) {
      {}
    } else {
      stryCov_9fa48("1607");
      return this.taskQueue.clear();
    }
  }
}