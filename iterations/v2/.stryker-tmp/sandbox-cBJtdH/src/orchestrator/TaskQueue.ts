/**
 * Task Queue Management
 *
 * Manages queued tasks and processing state for the orchestrator.
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
import { EventEmitter } from "events";
import { Task } from "../types/arbiter-orchestration";
import { TaskQueueStats } from "../types/orchestrator-events";
export class TaskQueue extends EventEmitter {
  private queue: Task[] = [];
  private processing: Map<string, Task> = new Map();
  private timestamps: Map<string, Date> = new Map();

  /**
   * Add task to queue
   */
  enqueue(task: Task): void {
    if (stryMutAct_9fa48("1408")) {
      {}
    } else {
      stryCov_9fa48("1408");
      if (stryMutAct_9fa48("1410") ? false : stryMutAct_9fa48("1409") ? true : (stryCov_9fa48("1409", "1410"), this.hasTask(task.id))) {
        if (stryMutAct_9fa48("1411")) {
          {}
        } else {
          stryCov_9fa48("1411");
          throw new Error(`Task ${task.id} is already in queue`);
        }
      }
      this.queue.push(task);
      this.timestamps.set(task.id, new Date());
      this.emit("task:enqueued", stryMutAct_9fa48("1414") ? {} : (stryCov_9fa48("1414"), {
        taskId: task.id,
        task
      }));
    }
  }

  /**
   * Get next task from queue
   */
  dequeue(): Task | undefined {
    if (stryMutAct_9fa48("1415")) {
      {}
    } else {
      stryCov_9fa48("1415");
      const task = this.queue.shift();
      if (stryMutAct_9fa48("1417") ? false : stryMutAct_9fa48("1416") ? true : (stryCov_9fa48("1416", "1417"), task)) {
        if (stryMutAct_9fa48("1418")) {
          {}
        } else {
          stryCov_9fa48("1418");
          this.processing.set(task.id, task);
          this.emit("task:dequeued", stryMutAct_9fa48("1420") ? {} : (stryCov_9fa48("1420"), {
            taskId: task.id,
            task
          }));
        }
      }
      return task;
    }
  }

  /**
   * Peek at next task without removing it
   */
  peek(): Task | undefined {
    if (stryMutAct_9fa48("1421")) {
      {}
    } else {
      stryCov_9fa48("1421");
      return this.queue[0];
    }
  }

  /**
   * Remove task from queue or processing
   */
  remove(taskId: string): boolean {
    if (stryMutAct_9fa48("1422")) {
      {}
    } else {
      stryCov_9fa48("1422");
      // Check queue first
      const queueIndex = this.queue.findIndex(stryMutAct_9fa48("1423") ? () => undefined : (stryCov_9fa48("1423"), t => stryMutAct_9fa48("1426") ? t.id !== taskId : stryMutAct_9fa48("1425") ? false : stryMutAct_9fa48("1424") ? true : (stryCov_9fa48("1424", "1425", "1426"), t.id === taskId)));
      if (stryMutAct_9fa48("1430") ? queueIndex < 0 : stryMutAct_9fa48("1429") ? queueIndex > 0 : stryMutAct_9fa48("1428") ? false : stryMutAct_9fa48("1427") ? true : (stryCov_9fa48("1427", "1428", "1429", "1430"), queueIndex >= 0)) {
        if (stryMutAct_9fa48("1431")) {
          {}
        } else {
          stryCov_9fa48("1431");
          this.queue.splice(queueIndex, 1);
          this.timestamps.delete(taskId);
          this.emit("task:removed", stryMutAct_9fa48("1433") ? {} : (stryCov_9fa48("1433"), {
            taskId,
            from: "queue"
          }));
          return stryMutAct_9fa48("1435") ? false : (stryCov_9fa48("1435"), true);
        }
      }

      // Check processing
      if (stryMutAct_9fa48("1437") ? false : stryMutAct_9fa48("1436") ? true : (stryCov_9fa48("1436", "1437"), this.processing.has(taskId))) {
        if (stryMutAct_9fa48("1438")) {
          {}
        } else {
          stryCov_9fa48("1438");
          this.processing.delete(taskId);
          this.timestamps.delete(taskId);
          this.emit("task:removed", stryMutAct_9fa48("1440") ? {} : (stryCov_9fa48("1440"), {
            taskId,
            from: "processing"
          }));
          return stryMutAct_9fa48("1442") ? false : (stryCov_9fa48("1442"), true);
        }
      }
      return stryMutAct_9fa48("1443") ? true : (stryCov_9fa48("1443"), false);
    }
  }

  /**
   * Check if task exists in queue or processing
   */
  hasTask(taskId: string): boolean {
    if (stryMutAct_9fa48("1444")) {
      {}
    } else {
      stryCov_9fa48("1444");
      return stryMutAct_9fa48("1447") ? this.isQueued(taskId) && this.isProcessing(taskId) : stryMutAct_9fa48("1446") ? false : stryMutAct_9fa48("1445") ? true : (stryCov_9fa48("1445", "1446", "1447"), this.isQueued(taskId) || this.isProcessing(taskId));
    }
  }

  /**
   * Check if task is queued
   */
  isQueued(taskId: string): boolean {
    if (stryMutAct_9fa48("1448")) {
      {}
    } else {
      stryCov_9fa48("1448");
      return stryMutAct_9fa48("1449") ? this.queue.every(t => t.id === taskId) : (stryCov_9fa48("1449"), this.queue.some(stryMutAct_9fa48("1450") ? () => undefined : (stryCov_9fa48("1450"), t => stryMutAct_9fa48("1453") ? t.id !== taskId : stryMutAct_9fa48("1452") ? false : stryMutAct_9fa48("1451") ? true : (stryCov_9fa48("1451", "1452", "1453"), t.id === taskId))));
    }
  }

  /**
   * Check if task is being processed
   */
  isProcessing(taskId: string): boolean {
    if (stryMutAct_9fa48("1454")) {
      {}
    } else {
      stryCov_9fa48("1454");
      return this.processing.has(taskId);
    }
  }

  /**
   * Mark task as no longer processing (completed/failed)
   */
  complete(taskId: string): boolean {
    if (stryMutAct_9fa48("1455")) {
      {}
    } else {
      stryCov_9fa48("1455");
      if (stryMutAct_9fa48("1457") ? false : stryMutAct_9fa48("1456") ? true : (stryCov_9fa48("1456", "1457"), this.processing.has(taskId))) {
        if (stryMutAct_9fa48("1458")) {
          {}
        } else {
          stryCov_9fa48("1458");
          this.processing.delete(taskId);
          this.timestamps.delete(taskId);
          this.emit("task:completed", stryMutAct_9fa48("1460") ? {} : (stryCov_9fa48("1460"), {
            taskId
          }));
          return stryMutAct_9fa48("1461") ? false : (stryCov_9fa48("1461"), true);
        }
      }
      return stryMutAct_9fa48("1462") ? true : (stryCov_9fa48("1462"), false);
    }
  }

  /**
   * Get queue statistics
   */
  getStats(): TaskQueueStats {
    if (stryMutAct_9fa48("1463")) {
      {}
    } else {
      stryCov_9fa48("1463");
      const queuedTimestamps = this.queue.map(task => this.timestamps.get(task.id)).filter(Boolean) as Date[];
      const oldestQueued = (stryMutAct_9fa48("1467") ? queuedTimestamps.length <= 0 : stryMutAct_9fa48("1466") ? queuedTimestamps.length >= 0 : stryMutAct_9fa48("1465") ? false : stryMutAct_9fa48("1464") ? true : (stryCov_9fa48("1464", "1465", "1466", "1467"), queuedTimestamps.length > 0)) ? new Date(stryMutAct_9fa48("1468") ? Math.max(...queuedTimestamps.map(t => t.getTime())) : (stryCov_9fa48("1468"), Math.min(...queuedTimestamps.map(stryMutAct_9fa48("1469") ? () => undefined : (stryCov_9fa48("1469"), t => t.getTime()))))) : undefined;
      return stryMutAct_9fa48("1470") ? {} : (stryCov_9fa48("1470"), {
        queued: this.queue.length,
        processing: this.processing.size,
        total: stryMutAct_9fa48("1471") ? this.queue.length - this.processing.size : (stryCov_9fa48("1471"), this.queue.length + this.processing.size),
        oldestQueued
      });
    }
  }

  /**
   * Get all queued tasks
   */
  getQueuedTasks(): Task[] {
    if (stryMutAct_9fa48("1472")) {
      {}
    } else {
      stryCov_9fa48("1472");
      return [...this.queue];
    }
  }

  /**
   * Get all processing tasks
   */
  getProcessingTasks(): Task[] {
    if (stryMutAct_9fa48("1474")) {
      {}
    } else {
      stryCov_9fa48("1474");
      return Array.from(this.processing.values());
    }
  }

  /**
   * Get task by ID
   */
  getTask(taskId: string): Task | undefined {
    if (stryMutAct_9fa48("1475")) {
      {}
    } else {
      stryCov_9fa48("1475");
      return stryMutAct_9fa48("1478") ? this.queue.find(t => t.id === taskId) && this.processing.get(taskId) : stryMutAct_9fa48("1477") ? false : stryMutAct_9fa48("1476") ? true : (stryCov_9fa48("1476", "1477", "1478"), this.queue.find(stryMutAct_9fa48("1479") ? () => undefined : (stryCov_9fa48("1479"), t => stryMutAct_9fa48("1482") ? t.id !== taskId : stryMutAct_9fa48("1481") ? false : stryMutAct_9fa48("1480") ? true : (stryCov_9fa48("1480", "1481", "1482"), t.id === taskId))) || this.processing.get(taskId));
    }
  }

  /**
   * Get queue size
   */
  size(): number {
    if (stryMutAct_9fa48("1483")) {
      {}
    } else {
      stryCov_9fa48("1483");
      return this.queue.length;
    }
  }

  /**
   * Check if queue is empty
   */
  isEmpty(): boolean {
    if (stryMutAct_9fa48("1484")) {
      {}
    } else {
      stryCov_9fa48("1484");
      return stryMutAct_9fa48("1487") ? this.queue.length !== 0 : stryMutAct_9fa48("1486") ? false : stryMutAct_9fa48("1485") ? true : (stryCov_9fa48("1485", "1486", "1487"), this.queue.length === 0);
    }
  }

  /**
   * Clear all tasks
   */
  clear(): void {
    if (stryMutAct_9fa48("1488")) {
      {}
    } else {
      stryCov_9fa48("1488");
      const taskIds = [...this.queue.map(stryMutAct_9fa48("1490") ? () => undefined : (stryCov_9fa48("1490"), t => t.id)), ...Array.from(this.processing.keys())];
      this.queue = [];
      this.processing.clear();
      this.timestamps.clear();
      this.emit("queue:cleared", stryMutAct_9fa48("1493") ? {} : (stryCov_9fa48("1493"), {
        taskIds
      }));
    }
  }

  /**
   * Get tasks older than specified duration
   */
  getStaleTasks(maxAgeMs: number): Task[] {
    if (stryMutAct_9fa48("1494")) {
      {}
    } else {
      stryCov_9fa48("1494");
      const now = Date.now();
      const stale: Task[] = [];

      // Check queued tasks
      for (const task of Array.from(this.queue)) {
        if (stryMutAct_9fa48("1496")) {
          {}
        } else {
          stryCov_9fa48("1496");
          const timestamp = this.timestamps.get(task.id);
          if (stryMutAct_9fa48("1499") ? timestamp || now - timestamp.getTime() > maxAgeMs : stryMutAct_9fa48("1498") ? false : stryMutAct_9fa48("1497") ? true : (stryCov_9fa48("1497", "1498", "1499"), timestamp && (stryMutAct_9fa48("1502") ? now - timestamp.getTime() <= maxAgeMs : stryMutAct_9fa48("1501") ? now - timestamp.getTime() >= maxAgeMs : stryMutAct_9fa48("1500") ? true : (stryCov_9fa48("1500", "1501", "1502"), (stryMutAct_9fa48("1503") ? now + timestamp.getTime() : (stryCov_9fa48("1503"), now - timestamp.getTime())) > maxAgeMs)))) {
            if (stryMutAct_9fa48("1504")) {
              {}
            } else {
              stryCov_9fa48("1504");
              stale.push(task);
            }
          }
        }
      }

      // Check processing tasks
      for (const task of Array.from(this.processing.values())) {
        if (stryMutAct_9fa48("1505")) {
          {}
        } else {
          stryCov_9fa48("1505");
          const timestamp = this.timestamps.get(task.id);
          if (stryMutAct_9fa48("1508") ? timestamp || now - timestamp.getTime() > maxAgeMs : stryMutAct_9fa48("1507") ? false : stryMutAct_9fa48("1506") ? true : (stryCov_9fa48("1506", "1507", "1508"), timestamp && (stryMutAct_9fa48("1511") ? now - timestamp.getTime() <= maxAgeMs : stryMutAct_9fa48("1510") ? now - timestamp.getTime() >= maxAgeMs : stryMutAct_9fa48("1509") ? true : (stryCov_9fa48("1509", "1510", "1511"), (stryMutAct_9fa48("1512") ? now + timestamp.getTime() : (stryCov_9fa48("1512"), now - timestamp.getTime())) > maxAgeMs)))) {
            if (stryMutAct_9fa48("1513")) {
              {}
            } else {
              stryCov_9fa48("1513");
              stale.push(task);
            }
          }
        }
      }
      return stale;
    }
  }

  /**
   * Enqueue task with credentials (stub for security integration)
   */
  async enqueueWithCredentials(task: Task, credentials: any): Promise<void> {
    if (stryMutAct_9fa48("1514")) {
      {}
    } else {
      stryCov_9fa48("1514");
      this.enqueue(task);
    }
  }

  /**
   * Initialize the task queue
   */
  async initialize(): Promise<void> {
    if (stryMutAct_9fa48("1515")) {
      {}
    } else {
      stryCov_9fa48("1515");
      this.emit("initialized");
    }
  }

  /**
   * Shutdown the task queue
   */
  async shutdown(): Promise<void> {
    if (stryMutAct_9fa48("1517")) {
      {}
    } else {
      stryCov_9fa48("1517");
      this.queue = [];
      this.processing.clear();
      this.timestamps.clear();
      this.emit("shutdown");
    }
  }

  /**
   * Get task state by ID
   */
  getTaskState(taskId: string): {
    task: Task;
    status: string;
  } | undefined {
    if (stryMutAct_9fa48("1520")) {
      {}
    } else {
      stryCov_9fa48("1520");
      const queuedTask = this.queue.find(stryMutAct_9fa48("1521") ? () => undefined : (stryCov_9fa48("1521"), t => stryMutAct_9fa48("1524") ? t.id !== taskId : stryMutAct_9fa48("1523") ? false : stryMutAct_9fa48("1522") ? true : (stryCov_9fa48("1522", "1523", "1524"), t.id === taskId)));
      if (stryMutAct_9fa48("1526") ? false : stryMutAct_9fa48("1525") ? true : (stryCov_9fa48("1525", "1526"), queuedTask)) {
        if (stryMutAct_9fa48("1527")) {
          {}
        } else {
          stryCov_9fa48("1527");
          return stryMutAct_9fa48("1528") ? {} : (stryCov_9fa48("1528"), {
            task: queuedTask,
            status: "queued"
          });
        }
      }
      const processingTask = this.processing.get(taskId);
      if (stryMutAct_9fa48("1531") ? false : stryMutAct_9fa48("1530") ? true : (stryCov_9fa48("1530", "1531"), processingTask)) {
        if (stryMutAct_9fa48("1532")) {
          {}
        } else {
          stryCov_9fa48("1532");
          return stryMutAct_9fa48("1533") ? {} : (stryCov_9fa48("1533"), {
            task: processingTask,
            status: "processing"
          });
        }
      }
      return undefined;
    }
  }

  /** Get queue length (alias for size)
   */
  getQueueLength(): number {
    if (stryMutAct_9fa48("1535")) {
      {}
    } else {
      stryCov_9fa48("1535");
      return this.queue.length;
    }
  }
}

// TODO: SecureTaskQueue removed pending SecurityManager implementation
// Re-implement when security infrastructure is available