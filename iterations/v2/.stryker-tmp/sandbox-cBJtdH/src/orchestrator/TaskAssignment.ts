/**
 * @fileoverview Task Assignment implementation for Arbiter Orchestration (ARBITER-005)
 *
 * Manages the assignment of tasks to agents based on routing decisions.
 * Tracks assignment lifecycle, timeouts, and provides reassignment capabilities.
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
import { RoutingDecision, Task, TaskAssignment, TaskExecution, TaskResult } from "../types/arbiter-orchestration";
import { DatabaseClientFactory, IDatabaseClient } from "./DatabaseClient";

/**
 * Assignment Configuration
 */
export interface AssignmentConfig {
  /** Maximum time to wait for agent acknowledgment */
  acknowledgmentTimeoutMs: number;

  /** Maximum assignment duration */
  maxAssignmentDurationMs: number;

  /** Enable automatic reassignment on failure */
  autoReassignmentEnabled: boolean;

  /** Maximum number of reassignment attempts */
  maxReassignmentAttempts: number;

  /** Progress check interval */
  progressCheckIntervalMs: number;

  /** Enable persistence */
  persistenceEnabled: boolean;

  /** Database client for persistence */
  databaseClient?: IDatabaseClient;
}

/**
 * Assignment Status Updates
 */
export interface AssignmentStatusUpdate {
  status?: string;
  acknowledgedAt?: Date;
  startedAt?: Date;
  completedAt?: Date;
  progress?: number;
  errorMessage?: string;
  errorCode?: string;
}

/**
 * Assignment Statistics
 */
export interface AssignmentStats {
  /** Total assignments created */
  totalCreated: number;

  /** Currently active assignments */
  activeCount: number;

  /** Successful completions */
  completedCount: number;

  /** Failed assignments */
  failedCount: number;

  /** Timeout assignments */
  timeoutCount: number;

  /** Reassigned tasks */
  reassignedCount: number;

  /** Average assignment duration */
  averageDurationMs: number;

  /** Assignment success rate (0-1) */
  successRate: number;
}

/**
 * Task Assignment Manager
 *
 * Handles the lifecycle of task assignments from creation to completion.
 * Provides monitoring, timeout handling, and reassignment capabilities.
 */
export class TaskAssignmentManager {
  private assignments: Map<string, TaskAssignment> = new Map();
  private executions: Map<string, TaskExecution> = new Map();
  private config: AssignmentConfig;
  private stats: AssignmentStats;
  private timeouts: Map<string, ReturnType<typeof setTimeout>> = new Map();
  private progressChecks: Map<string, ReturnType<typeof setInterval>> = new Map();
  private dbClient?: IDatabaseClient;
  private initialized: boolean = stryMutAct_9fa48("1125") ? true : (stryCov_9fa48("1125"), false);
  constructor(config: Partial<AssignmentConfig> = {}) {
    if (stryMutAct_9fa48("1126")) {
      {}
    } else {
      stryCov_9fa48("1126");
      this.config = stryMutAct_9fa48("1127") ? {} : (stryCov_9fa48("1127"), {
        acknowledgmentTimeoutMs: 5000,
        maxAssignmentDurationMs: 300000,
        // 5 minutes
        autoReassignmentEnabled: stryMutAct_9fa48("1128") ? false : (stryCov_9fa48("1128"), true),
        maxReassignmentAttempts: 3,
        progressCheckIntervalMs: 30000,
        // 30 seconds
        persistenceEnabled: stryMutAct_9fa48("1129") ? true : (stryCov_9fa48("1129"), false),
        ...config
      });

      // Initialize database client if persistence is enabled
      if (stryMutAct_9fa48("1131") ? false : stryMutAct_9fa48("1130") ? true : (stryCov_9fa48("1130", "1131"), this.config.persistenceEnabled)) {
        if (stryMutAct_9fa48("1132")) {
          {}
        } else {
          stryCov_9fa48("1132");
          this.dbClient = stryMutAct_9fa48("1135") ? this.config.databaseClient && DatabaseClientFactory.createMockClient() : stryMutAct_9fa48("1134") ? false : stryMutAct_9fa48("1133") ? true : (stryCov_9fa48("1133", "1134", "1135"), this.config.databaseClient || DatabaseClientFactory.createMockClient());
        }
      }
      this.stats = stryMutAct_9fa48("1136") ? {} : (stryCov_9fa48("1136"), {
        totalCreated: 0,
        activeCount: 0,
        completedCount: 0,
        failedCount: 0,
        timeoutCount: 0,
        reassignedCount: 0,
        averageDurationMs: 0,
        successRate: 0
      });
    }
  }

  /**
   * Initialize the assignment manager (connect to database)
   */
  async initialize(): Promise<void> {
    if (stryMutAct_9fa48("1137")) {
      {}
    } else {
      stryCov_9fa48("1137");
      if (stryMutAct_9fa48("1139") ? false : stryMutAct_9fa48("1138") ? true : (stryCov_9fa48("1138", "1139"), this.initialized)) {
        if (stryMutAct_9fa48("1140")) {
          {}
        } else {
          stryCov_9fa48("1140");
          return;
        }
      }
      try {
        if (stryMutAct_9fa48("1141")) {
          {}
        } else {
          stryCov_9fa48("1141");
          // Connect to database if persistence is enabled
          if (stryMutAct_9fa48("1144") ? this.config.persistenceEnabled || this.dbClient : stryMutAct_9fa48("1143") ? false : stryMutAct_9fa48("1142") ? true : (stryCov_9fa48("1142", "1143", "1144"), this.config.persistenceEnabled && this.dbClient)) {
            if (stryMutAct_9fa48("1145")) {
              {}
            } else {
              stryCov_9fa48("1145");
              await this.dbClient.connect();
            }
          }
          this.initialized = stryMutAct_9fa48("1146") ? false : (stryCov_9fa48("1146"), true);
          console.log("TaskAssignmentManager initialized successfully");
        }
      } catch (error) {
        if (stryMutAct_9fa48("1148")) {
          {}
        } else {
          stryCov_9fa48("1148");
          console.error("Failed to initialize TaskAssignmentManager:", error);
          throw error;
        }
      }
    }
  }

  /**
   * Persist assignment to database
   */
  private async persistAssignment(assignment: TaskAssignment): Promise<void> {
    if (stryMutAct_9fa48("1150")) {
      {}
    } else {
      stryCov_9fa48("1150");
      if (stryMutAct_9fa48("1153") ? false : stryMutAct_9fa48("1152") ? true : stryMutAct_9fa48("1151") ? this.dbClient : (stryCov_9fa48("1151", "1152", "1153"), !this.dbClient)) {
        if (stryMutAct_9fa48("1154")) {
          {}
        } else {
          stryCov_9fa48("1154");
          return;
        }
      }
      try {
        if (stryMutAct_9fa48("1155")) {
          {}
        } else {
          stryCov_9fa48("1155");
          await this.dbClient.query(`
        INSERT INTO task_assignments (
          assignment_id, task_id, agent_id, agent_name, agent_model_family,
          assigned_at, deadline, assignment_timeout_ms, routing_confidence,
          routing_strategy, routing_reason, status, acknowledged_at,
          started_at, completed_at, progress, last_progress_update,
          error_message, error_code, assignment_metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        ON CONFLICT (assignment_id) DO UPDATE SET
          status = EXCLUDED.status,
          acknowledged_at = EXCLUDED.acknowledged_at,
          started_at = EXCLUDED.started_at,
          completed_at = EXCLUDED.completed_at,
          progress = EXCLUDED.progress,
          last_progress_update = EXCLUDED.last_progress_update,
          error_message = EXCLUDED.error_message,
          error_code = EXCLUDED.error_code,
          updated_at = NOW()
      `, [assignment.id, assignment.task.id, assignment.agent.id, assignment.agent.name, assignment.agent.modelFamily, assignment.assignedAt, assignment.deadline, 300000,
          // assignmentTimeoutMs - default 5 minutes
          assignment.routingDecision.confidence, assignment.routingDecision.strategy, assignment.routingDecision.reason, "pending",
          // status - default
          null,
          // acknowledgedAt
          null,
          // startedAt
          null,
          // completedAt
          0,
          // progress
          null,
          // lastProgressUpdate
          null,
          // errorMessage
          null,
          // errorCode
          JSON.stringify({})]);
        }
      } catch (error) {
        if (stryMutAct_9fa48("1159")) {
          {}
        } else {
          stryCov_9fa48("1159");
          console.error(`Failed to persist assignment ${assignment.id}:`, error);
          // Don't throw - assignment should continue working even if persistence fails
        }
      }
    }
  }

  /**
   * Update assignment status in database
   */
  private async updateAssignmentStatusInDb(assignmentId: string, updates: AssignmentStatusUpdate): Promise<void> {
    if (stryMutAct_9fa48("1161")) {
      {}
    } else {
      stryCov_9fa48("1161");
      if (stryMutAct_9fa48("1164") ? !this.config.persistenceEnabled && !this.dbClient : stryMutAct_9fa48("1163") ? false : stryMutAct_9fa48("1162") ? true : (stryCov_9fa48("1162", "1163", "1164"), (stryMutAct_9fa48("1165") ? this.config.persistenceEnabled : (stryCov_9fa48("1165"), !this.config.persistenceEnabled)) || (stryMutAct_9fa48("1166") ? this.dbClient : (stryCov_9fa48("1166"), !this.dbClient)))) {
        if (stryMutAct_9fa48("1167")) {
          {}
        } else {
          stryCov_9fa48("1167");
          return;
        }
      }
      try {
        if (stryMutAct_9fa48("1168")) {
          {}
        } else {
          stryCov_9fa48("1168");
          const setParts: string[] = [];
          const values: any[] = [];
          let paramIndex = 1;
          if (stryMutAct_9fa48("1173") ? updates.status === undefined : stryMutAct_9fa48("1172") ? false : stryMutAct_9fa48("1171") ? true : (stryCov_9fa48("1171", "1172", "1173"), updates.status !== undefined)) {
            if (stryMutAct_9fa48("1174")) {
              {}
            } else {
              stryCov_9fa48("1174");
              setParts.push(`status = $${stryMutAct_9fa48("1176") ? paramIndex-- : (stryCov_9fa48("1176"), paramIndex++)}`);
              values.push(updates.status);
            }
          }
          if (stryMutAct_9fa48("1179") ? updates.acknowledgedAt === undefined : stryMutAct_9fa48("1178") ? false : stryMutAct_9fa48("1177") ? true : (stryCov_9fa48("1177", "1178", "1179"), updates.acknowledgedAt !== undefined)) {
            if (stryMutAct_9fa48("1180")) {
              {}
            } else {
              stryCov_9fa48("1180");
              setParts.push(`acknowledged_at = $${stryMutAct_9fa48("1182") ? paramIndex-- : (stryCov_9fa48("1182"), paramIndex++)}`);
              values.push(updates.acknowledgedAt);
            }
          }
          if (stryMutAct_9fa48("1185") ? updates.startedAt === undefined : stryMutAct_9fa48("1184") ? false : stryMutAct_9fa48("1183") ? true : (stryCov_9fa48("1183", "1184", "1185"), updates.startedAt !== undefined)) {
            if (stryMutAct_9fa48("1186")) {
              {}
            } else {
              stryCov_9fa48("1186");
              setParts.push(`started_at = $${stryMutAct_9fa48("1188") ? paramIndex-- : (stryCov_9fa48("1188"), paramIndex++)}`);
              values.push(updates.startedAt);
            }
          }
          if (stryMutAct_9fa48("1191") ? updates.completedAt === undefined : stryMutAct_9fa48("1190") ? false : stryMutAct_9fa48("1189") ? true : (stryCov_9fa48("1189", "1190", "1191"), updates.completedAt !== undefined)) {
            if (stryMutAct_9fa48("1192")) {
              {}
            } else {
              stryCov_9fa48("1192");
              setParts.push(`completed_at = $${stryMutAct_9fa48("1194") ? paramIndex-- : (stryCov_9fa48("1194"), paramIndex++)}`);
              values.push(updates.completedAt);
            }
          }
          if (stryMutAct_9fa48("1197") ? updates.progress === undefined : stryMutAct_9fa48("1196") ? false : stryMutAct_9fa48("1195") ? true : (stryCov_9fa48("1195", "1196", "1197"), updates.progress !== undefined)) {
            if (stryMutAct_9fa48("1198")) {
              {}
            } else {
              stryCov_9fa48("1198");
              setParts.push(`progress = $${stryMutAct_9fa48("1200") ? paramIndex-- : (stryCov_9fa48("1200"), paramIndex++)}, last_progress_update = NOW()`);
              values.push(updates.progress);
            }
          }
          if (stryMutAct_9fa48("1203") ? updates.errorMessage === undefined : stryMutAct_9fa48("1202") ? false : stryMutAct_9fa48("1201") ? true : (stryCov_9fa48("1201", "1202", "1203"), updates.errorMessage !== undefined)) {
            if (stryMutAct_9fa48("1204")) {
              {}
            } else {
              stryCov_9fa48("1204");
              setParts.push(`error_message = $${stryMutAct_9fa48("1206") ? paramIndex-- : (stryCov_9fa48("1206"), paramIndex++)}`);
              values.push(updates.errorMessage);
            }
          }
          if (stryMutAct_9fa48("1209") ? updates.errorCode === undefined : stryMutAct_9fa48("1208") ? false : stryMutAct_9fa48("1207") ? true : (stryCov_9fa48("1207", "1208", "1209"), updates.errorCode !== undefined)) {
            if (stryMutAct_9fa48("1210")) {
              {}
            } else {
              stryCov_9fa48("1210");
              setParts.push(`error_code = $${stryMutAct_9fa48("1212") ? paramIndex-- : (stryCov_9fa48("1212"), paramIndex++)}`);
              values.push(updates.errorCode);
            }
          }
          if (stryMutAct_9fa48("1215") ? setParts.length !== 0 : stryMutAct_9fa48("1214") ? false : stryMutAct_9fa48("1213") ? true : (stryCov_9fa48("1213", "1214", "1215"), setParts.length === 0)) {
            if (stryMutAct_9fa48("1216")) {
              {}
            } else {
              stryCov_9fa48("1216");
              return; // Nothing to update
            }
          }
          values.push(assignmentId); // Add assignment_id at the end

          await this.dbClient.query(`
        UPDATE task_assignments
        SET ${setParts.join(", ")}, updated_at = NOW()
        WHERE assignment_id = $${paramIndex}
      `, values);
        }
      } catch (error) {
        if (stryMutAct_9fa48("1219")) {
          {}
        } else {
          stryCov_9fa48("1219");
          console.error(`Failed to update assignment status ${assignmentId}:`, error);
        }
      }
    }
  }

  /**
   * Create a new task assignment
   */
  async createAssignment(task: Task, routingDecision: RoutingDecision, onAssignmentTimeout?: (assignment: TaskAssignment) => void, onProgressTimeout?: (assignment: TaskAssignment) => void): Promise<TaskAssignment> {
    if (stryMutAct_9fa48("1221")) {
      {}
    } else {
      stryCov_9fa48("1221");
      const assignment: TaskAssignment = stryMutAct_9fa48("1222") ? {} : (stryCov_9fa48("1222"), {
        id: `assignment-${task.id}-${Date.now()}`,
        task,
        agent: routingDecision.selectedAgent,
        routingDecision,
        assignedAt: new Date(),
        deadline: new Date(stryMutAct_9fa48("1224") ? Date.now() - this.config.maxAssignmentDurationMs : (stryCov_9fa48("1224"), Date.now() + this.config.maxAssignmentDurationMs))
      });

      // Store assignment
      this.assignments.set(assignment.id, assignment);
      stryMutAct_9fa48("1225") ? this.stats.totalCreated-- : (stryCov_9fa48("1225"), this.stats.totalCreated++);
      stryMutAct_9fa48("1226") ? this.stats.activeCount-- : (stryCov_9fa48("1226"), this.stats.activeCount++);

      // Set acknowledgment timeout
      const ackTimeout = setTimeout(() => {
        if (stryMutAct_9fa48("1227")) {
          {}
        } else {
          stryCov_9fa48("1227");
          this.handleAcknowledgmentTimeout(assignment, onAssignmentTimeout);
        }
      }, this.config.acknowledgmentTimeoutMs);
      this.timeouts.set(`${assignment.id}-ack`, ackTimeout);

      // Set progress check interval
      const progressCheck = setInterval(() => {
        if (stryMutAct_9fa48("1229")) {
          {}
        } else {
          stryCov_9fa48("1229");
          this.checkProgressTimeout(assignment, onProgressTimeout);
        }
      }, this.config.progressCheckIntervalMs);
      this.progressChecks.set(assignment.id, progressCheck);

      // Persist assignment to database if enabled
      if (stryMutAct_9fa48("1231") ? false : stryMutAct_9fa48("1230") ? true : (stryCov_9fa48("1230", "1231"), this.config.persistenceEnabled)) {
        if (stryMutAct_9fa48("1232")) {
          {}
        } else {
          stryCov_9fa48("1232");
          // Need to extend assignment with additional properties for database
          const dbAssignment = assignment as any;
          dbAssignment.taskId = task.id;
          dbAssignment.agentId = routingDecision.selectedAgent.id;
          dbAssignment.agentName = routingDecision.selectedAgent.name;
          dbAssignment.agentModelFamily = routingDecision.selectedAgent.modelFamily;
          dbAssignment.assignmentTimeoutMs = this.config.maxAssignmentDurationMs;
          dbAssignment.status = "assigned";
          dbAssignment.progress = 0;
          dbAssignment.lastProgressUpdate = new Date();
          await this.persistAssignment(dbAssignment);
        }
      }
      return assignment;
    }
  }

  /**
   * Acknowledge assignment (agent confirmed receipt)
   */
  async acknowledgeAssignment(assignmentId: string): Promise<boolean> {
    if (stryMutAct_9fa48("1234")) {
      {}
    } else {
      stryCov_9fa48("1234");
      const assignment = this.assignments.get(assignmentId);
      if (stryMutAct_9fa48("1237") ? false : stryMutAct_9fa48("1236") ? true : stryMutAct_9fa48("1235") ? assignment : (stryCov_9fa48("1235", "1236", "1237"), !assignment)) {
        if (stryMutAct_9fa48("1238")) {
          {}
        } else {
          stryCov_9fa48("1238");
          return stryMutAct_9fa48("1239") ? true : (stryCov_9fa48("1239"), false);
        }
      }

      // Clear acknowledgment timeout
      const ackTimeoutKey = `${assignmentId}-ack`;
      const ackTimeout = this.timeouts.get(ackTimeoutKey);
      if (stryMutAct_9fa48("1242") ? false : stryMutAct_9fa48("1241") ? true : (stryCov_9fa48("1241", "1242"), ackTimeout)) {
        if (stryMutAct_9fa48("1243")) {
          {}
        } else {
          stryCov_9fa48("1243");
          clearTimeout(ackTimeout);
          this.timeouts.delete(ackTimeoutKey);
        }
      }

      // Create execution record
      const execution: TaskExecution = stryMutAct_9fa48("1244") ? {} : (stryCov_9fa48("1244"), {
        id: `execution-${assignment.task.id}-${Date.now()}`,
        assignment,
        startedAt: new Date(),
        status: "running",
        progress: 0,
        metadata: {}
      });
      this.executions.set(assignmentId, execution);

      // Update status in database
      if (stryMutAct_9fa48("1248") ? false : stryMutAct_9fa48("1247") ? true : (stryCov_9fa48("1247", "1248"), this.config.persistenceEnabled)) {
        if (stryMutAct_9fa48("1249")) {
          {}
        } else {
          stryCov_9fa48("1249");
          await this.updateAssignmentStatusInDb(assignmentId, stryMutAct_9fa48("1250") ? {} : (stryCov_9fa48("1250"), {
            acknowledgedAt: new Date(),
            startedAt: new Date()
          }));
        }
      }
      return stryMutAct_9fa48("1251") ? false : (stryCov_9fa48("1251"), true);
    }
  }

  /**
   * Update execution progress
   */
  async updateProgress(assignmentId: string, progress: number, status: TaskExecution["status"] = "running", metadata?: Record<string, any>): Promise<boolean> {
    if (stryMutAct_9fa48("1253")) {
      {}
    } else {
      stryCov_9fa48("1253");
      const execution = this.executions.get(assignmentId);
      if (stryMutAct_9fa48("1256") ? false : stryMutAct_9fa48("1255") ? true : stryMutAct_9fa48("1254") ? execution : (stryCov_9fa48("1254", "1255", "1256"), !execution)) {
        if (stryMutAct_9fa48("1257")) {
          {}
        } else {
          stryCov_9fa48("1257");
          return stryMutAct_9fa48("1258") ? true : (stryCov_9fa48("1258"), false);
        }
      }
      execution.progress = stryMutAct_9fa48("1259") ? Math.min(0, Math.min(1, progress)) : (stryCov_9fa48("1259"), Math.max(0, stryMutAct_9fa48("1260") ? Math.max(1, progress) : (stryCov_9fa48("1260"), Math.min(1, progress))));
      execution.status = status;
      if (stryMutAct_9fa48("1262") ? false : stryMutAct_9fa48("1261") ? true : (stryCov_9fa48("1261", "1262"), metadata)) {
        if (stryMutAct_9fa48("1263")) {
          {}
        } else {
          stryCov_9fa48("1263");
          execution.metadata = stryMutAct_9fa48("1264") ? {} : (stryCov_9fa48("1264"), {
            ...execution.metadata,
            ...metadata
          });
        }
      }

      // Reset progress timeout on any update
      this.resetProgressTimeout(assignmentId);

      // Update progress in database
      if (stryMutAct_9fa48("1266") ? false : stryMutAct_9fa48("1265") ? true : (stryCov_9fa48("1265", "1266"), this.config.persistenceEnabled)) {
        if (stryMutAct_9fa48("1267")) {
          {}
        } else {
          stryCov_9fa48("1267");
          await this.updateAssignmentStatusInDb(assignmentId, stryMutAct_9fa48("1268") ? {} : (stryCov_9fa48("1268"), {
            progress: execution.progress
          }));
        }
      }
      return stryMutAct_9fa48("1269") ? false : (stryCov_9fa48("1269"), true);
    }
  }

  /**
   * Complete assignment with result
   */
  completeAssignment(assignmentId: string, result: TaskResult, onCompletion?: (assignment: TaskAssignment, result: TaskResult) => void): boolean {
    if (stryMutAct_9fa48("1270")) {
      {}
    } else {
      stryCov_9fa48("1270");
      const assignment = this.assignments.get(assignmentId);
      const execution = this.executions.get(assignmentId);
      if (stryMutAct_9fa48("1273") ? !assignment && !execution : stryMutAct_9fa48("1272") ? false : stryMutAct_9fa48("1271") ? true : (stryCov_9fa48("1271", "1272", "1273"), (stryMutAct_9fa48("1274") ? assignment : (stryCov_9fa48("1274"), !assignment)) || (stryMutAct_9fa48("1275") ? execution : (stryCov_9fa48("1275"), !execution)))) {
        if (stryMutAct_9fa48("1276")) {
          {}
        } else {
          stryCov_9fa48("1276");
          return stryMutAct_9fa48("1277") ? true : (stryCov_9fa48("1277"), false);
        }
      }

      // Update execution
      execution.status = "completed";
      execution.progress = 1;

      // Calculate duration
      const duration = stryMutAct_9fa48("1279") ? Date.now() + assignment.assignedAt.getTime() : (stryCov_9fa48("1279"), Date.now() - assignment.assignedAt.getTime());

      // Update statistics
      stryMutAct_9fa48("1280") ? this.stats.activeCount++ : (stryCov_9fa48("1280"), this.stats.activeCount--);
      stryMutAct_9fa48("1281") ? this.stats.completedCount-- : (stryCov_9fa48("1281"), this.stats.completedCount++);
      this.updateAverageDuration(duration);
      this.updateSuccessRate();

      // Clean up timers
      this.cleanupAssignmentTimers(assignmentId);

      // Remove from active tracking
      this.assignments.delete(assignmentId);
      this.executions.delete(assignmentId);

      // Notify completion
      if (stryMutAct_9fa48("1283") ? false : stryMutAct_9fa48("1282") ? true : (stryCov_9fa48("1282", "1283"), onCompletion)) {
        if (stryMutAct_9fa48("1284")) {
          {}
        } else {
          stryCov_9fa48("1284");
          onCompletion(assignment, result);
        }
      }
      return stryMutAct_9fa48("1285") ? false : (stryCov_9fa48("1285"), true);
    }
  }

  /**
   * Fail assignment
   */
  failAssignment(assignmentId: string, error: string, canRetry: boolean = stryMutAct_9fa48("1286") ? false : (stryCov_9fa48("1286"), true), onFailure?: (assignment: TaskAssignment, error: string) => void): boolean {
    if (stryMutAct_9fa48("1287")) {
      {}
    } else {
      stryCov_9fa48("1287");
      const assignment = this.assignments.get(assignmentId);
      const execution = this.executions.get(assignmentId);
      if (stryMutAct_9fa48("1290") ? false : stryMutAct_9fa48("1289") ? true : stryMutAct_9fa48("1288") ? assignment : (stryCov_9fa48("1288", "1289", "1290"), !assignment)) {
        if (stryMutAct_9fa48("1291")) {
          {}
        } else {
          stryCov_9fa48("1291");
          return stryMutAct_9fa48("1292") ? true : (stryCov_9fa48("1292"), false);
        }
      }

      // Update execution if exists
      if (stryMutAct_9fa48("1294") ? false : stryMutAct_9fa48("1293") ? true : (stryCov_9fa48("1293", "1294"), execution)) {
        if (stryMutAct_9fa48("1295")) {
          {}
        } else {
          stryCov_9fa48("1295");
          execution.status = "failed";
        }
      }

      // Update statistics
      stryMutAct_9fa48("1297") ? this.stats.activeCount++ : (stryCov_9fa48("1297"), this.stats.activeCount--);
      stryMutAct_9fa48("1298") ? this.stats.failedCount-- : (stryCov_9fa48("1298"), this.stats.failedCount++);

      // Clean up timers
      this.cleanupAssignmentTimers(assignmentId);

      // Handle reassignment if enabled and possible
      let reassigned = stryMutAct_9fa48("1299") ? true : (stryCov_9fa48("1299"), false);
      if (stryMutAct_9fa48("1302") ? canRetry && this.config.autoReassignmentEnabled || assignment.task.attempts < assignment.task.maxAttempts : stryMutAct_9fa48("1301") ? false : stryMutAct_9fa48("1300") ? true : (stryCov_9fa48("1300", "1301", "1302"), (stryMutAct_9fa48("1304") ? canRetry || this.config.autoReassignmentEnabled : stryMutAct_9fa48("1303") ? true : (stryCov_9fa48("1303", "1304"), canRetry && this.config.autoReassignmentEnabled)) && (stryMutAct_9fa48("1307") ? assignment.task.attempts >= assignment.task.maxAttempts : stryMutAct_9fa48("1306") ? assignment.task.attempts <= assignment.task.maxAttempts : stryMutAct_9fa48("1305") ? true : (stryCov_9fa48("1305", "1306", "1307"), assignment.task.attempts < assignment.task.maxAttempts)))) {
        if (stryMutAct_9fa48("1308")) {
          {}
        } else {
          stryCov_9fa48("1308");
          reassigned = this.attemptReassignment();
        }
      }

      // Remove from active tracking if not reassigned
      if (stryMutAct_9fa48("1311") ? false : stryMutAct_9fa48("1310") ? true : stryMutAct_9fa48("1309") ? reassigned : (stryCov_9fa48("1309", "1310", "1311"), !reassigned)) {
        if (stryMutAct_9fa48("1312")) {
          {}
        } else {
          stryCov_9fa48("1312");
          this.assignments.delete(assignmentId);
          this.executions.delete(assignmentId);
        }
      }

      // Notify failure
      if (stryMutAct_9fa48("1314") ? false : stryMutAct_9fa48("1313") ? true : (stryCov_9fa48("1313", "1314"), onFailure)) {
        if (stryMutAct_9fa48("1315")) {
          {}
        } else {
          stryCov_9fa48("1315");
          onFailure(assignment, error);
        }
      }
      return stryMutAct_9fa48("1316") ? false : (stryCov_9fa48("1316"), true);
    }
  }

  /**
   * Get assignment by ID
   */
  getAssignment(assignmentId: string): TaskAssignment | null {
    if (stryMutAct_9fa48("1317")) {
      {}
    } else {
      stryCov_9fa48("1317");
      return stryMutAct_9fa48("1320") ? this.assignments.get(assignmentId) && null : stryMutAct_9fa48("1319") ? false : stryMutAct_9fa48("1318") ? true : (stryCov_9fa48("1318", "1319", "1320"), this.assignments.get(assignmentId) || null);
    }
  }

  /**
   * Get execution by assignment ID
   */
  getExecution(assignmentId: string): TaskExecution | null {
    if (stryMutAct_9fa48("1321")) {
      {}
    } else {
      stryCov_9fa48("1321");
      return stryMutAct_9fa48("1324") ? this.executions.get(assignmentId) && null : stryMutAct_9fa48("1323") ? false : stryMutAct_9fa48("1322") ? true : (stryCov_9fa48("1322", "1323", "1324"), this.executions.get(assignmentId) || null);
    }
  }

  /**
   * Get all active assignments
   */
  getActiveAssignments(): TaskAssignment[] {
    if (stryMutAct_9fa48("1325")) {
      {}
    } else {
      stryCov_9fa48("1325");
      return Array.from(this.assignments.values());
    }
  }

  /**
   * Get assignment statistics
   */
  getStats(): AssignmentStats {
    if (stryMutAct_9fa48("1326")) {
      {}
    } else {
      stryCov_9fa48("1326");
      return stryMutAct_9fa48("1327") ? {} : (stryCov_9fa48("1327"), {
        ...this.stats
      });
    }
  }

  /**
   * Force timeout an assignment
   */
  timeoutAssignment(assignmentId: string, onTimeout?: (assignment: TaskAssignment) => void): boolean {
    if (stryMutAct_9fa48("1328")) {
      {}
    } else {
      stryCov_9fa48("1328");
      const assignment = this.assignments.get(assignmentId);
      if (stryMutAct_9fa48("1331") ? false : stryMutAct_9fa48("1330") ? true : stryMutAct_9fa48("1329") ? assignment : (stryCov_9fa48("1329", "1330", "1331"), !assignment)) {
        if (stryMutAct_9fa48("1332")) {
          {}
        } else {
          stryCov_9fa48("1332");
          return stryMutAct_9fa48("1333") ? true : (stryCov_9fa48("1333"), false);
        }
      }

      // Update statistics
      stryMutAct_9fa48("1334") ? this.stats.activeCount++ : (stryCov_9fa48("1334"), this.stats.activeCount--);
      stryMutAct_9fa48("1335") ? this.stats.timeoutCount-- : (stryCov_9fa48("1335"), this.stats.timeoutCount++);

      // Clean up timers
      this.cleanupAssignmentTimers(assignmentId);

      // Remove from tracking
      this.assignments.delete(assignmentId);
      this.executions.delete(assignmentId);

      // Notify timeout
      if (stryMutAct_9fa48("1337") ? false : stryMutAct_9fa48("1336") ? true : (stryCov_9fa48("1336", "1337"), onTimeout)) {
        if (stryMutAct_9fa48("1338")) {
          {}
        } else {
          stryCov_9fa48("1338");
          onTimeout(assignment);
        }
      }
      return stryMutAct_9fa48("1339") ? false : (stryCov_9fa48("1339"), true);
    }
  }

  /**
   * Clean shutdown - cancel all active assignments
   */
  async shutdown(): Promise<void> {
    if (stryMutAct_9fa48("1340")) {
      {}
    } else {
      stryCov_9fa48("1340");
      // Clear all timers
      for (const timeout of Array.from(this.timeouts.values())) {
        if (stryMutAct_9fa48("1341")) {
          {}
        } else {
          stryCov_9fa48("1341");
          clearTimeout(timeout);
        }
      }
      this.timeouts.clear();
      for (const interval of Array.from(this.progressChecks.values())) {
        if (stryMutAct_9fa48("1342")) {
          {}
        } else {
          stryCov_9fa48("1342");
          clearInterval(interval);
        }
      }
      this.progressChecks.clear();

      // Cancel all active assignments
      const activeIds = Array.from(this.assignments.keys());
      for (const assignmentId of activeIds) {
        if (stryMutAct_9fa48("1343")) {
          {}
        } else {
          stryCov_9fa48("1343");
          this.failAssignment(assignmentId, "System shutdown", stryMutAct_9fa48("1345") ? true : (stryCov_9fa48("1345"), false));
        }
      }
    }
  }

  /**
   * Handle acknowledgment timeout
   */
  private handleAcknowledgmentTimeout(assignment: TaskAssignment, onTimeout?: (assignment: TaskAssignment) => void): void {
    if (stryMutAct_9fa48("1346")) {
      {}
    } else {
      stryCov_9fa48("1346");
      // Agent didn't acknowledge within timeout
      this.failAssignment(assignment.id, "Acknowledgment timeout", stryMutAct_9fa48("1348") ? false : (stryCov_9fa48("1348"), true), assignment => {
        if (stryMutAct_9fa48("1349")) {
          {}
        } else {
          stryCov_9fa48("1349");
          if (stryMutAct_9fa48("1351") ? false : stryMutAct_9fa48("1350") ? true : (stryCov_9fa48("1350", "1351"), onTimeout)) {
            if (stryMutAct_9fa48("1352")) {
              {}
            } else {
              stryCov_9fa48("1352");
              onTimeout(assignment);
            }
          }
        }
      });
    }
  }

  /**
   * Check for progress timeout
   */
  private checkProgressTimeout(assignment: TaskAssignment, onTimeout?: (assignment: TaskAssignment) => void): void {
    if (stryMutAct_9fa48("1353")) {
      {}
    } else {
      stryCov_9fa48("1353");
      const execution = this.executions.get(assignment.id);
      if (stryMutAct_9fa48("1356") ? false : stryMutAct_9fa48("1355") ? true : stryMutAct_9fa48("1354") ? execution : (stryCov_9fa48("1354", "1355", "1356"), !execution)) {
        if (stryMutAct_9fa48("1357")) {
          {}
        } else {
          stryCov_9fa48("1357");
          return;
        }
      }
      const timeSinceLastUpdate = stryMutAct_9fa48("1358") ? Date.now() + execution.startedAt.getTime() : (stryCov_9fa48("1358"), Date.now() - execution.startedAt.getTime());
      if (stryMutAct_9fa48("1362") ? timeSinceLastUpdate <= this.config.maxAssignmentDurationMs : stryMutAct_9fa48("1361") ? timeSinceLastUpdate >= this.config.maxAssignmentDurationMs : stryMutAct_9fa48("1360") ? false : stryMutAct_9fa48("1359") ? true : (stryCov_9fa48("1359", "1360", "1361", "1362"), timeSinceLastUpdate > this.config.maxAssignmentDurationMs)) {
        if (stryMutAct_9fa48("1363")) {
          {}
        } else {
          stryCov_9fa48("1363");
          this.timeoutAssignment(assignment.id, onTimeout);
        }
      }
    }
  }

  /**
   * Reset progress timeout
   */
  private resetProgressTimeout(assignmentId: string): void {
    if (stryMutAct_9fa48("1364")) {
      {}
    } else {
      stryCov_9fa48("1364");
      const progressCheck = this.progressChecks.get(assignmentId);
      if (stryMutAct_9fa48("1366") ? false : stryMutAct_9fa48("1365") ? true : (stryCov_9fa48("1365", "1366"), progressCheck)) {
        if (stryMutAct_9fa48("1367")) {
          {}
        } else {
          stryCov_9fa48("1367");
          clearInterval(progressCheck);
          const newProgressCheck = setInterval(() => {
            if (stryMutAct_9fa48("1368")) {
              {}
            } else {
              stryCov_9fa48("1368");
              const assignment = this.assignments.get(assignmentId);
              if (stryMutAct_9fa48("1370") ? false : stryMutAct_9fa48("1369") ? true : (stryCov_9fa48("1369", "1370"), assignment)) {
                if (stryMutAct_9fa48("1371")) {
                  {}
                } else {
                  stryCov_9fa48("1371");
                  this.checkProgressTimeout(assignment);
                }
              }
            }
          }, this.config.progressCheckIntervalMs);
          this.progressChecks.set(assignmentId, newProgressCheck);
        }
      }
    }
  }

  /**
   * Attempt to reassign a failed task
   */
  private attemptReassignment(): boolean {
    if (stryMutAct_9fa48("1372")) {
      {}
    } else {
      stryCov_9fa48("1372");
      // This would typically call back to the routing system
      // For now, we'll just mark it as reassigned in statistics
      stryMutAct_9fa48("1373") ? this.stats.reassignedCount-- : (stryCov_9fa48("1373"), this.stats.reassignedCount++);
      return stryMutAct_9fa48("1374") ? false : (stryCov_9fa48("1374"), true); // Assume reassignment was successful
    }
  }

  /**
   * Update average duration statistic
   */
  private updateAverageDuration(duration: number): void {
    if (stryMutAct_9fa48("1375")) {
      {}
    } else {
      stryCov_9fa48("1375");
      const totalCompletions = this.stats.completedCount;
      if (stryMutAct_9fa48("1378") ? totalCompletions !== 1 : stryMutAct_9fa48("1377") ? false : stryMutAct_9fa48("1376") ? true : (stryCov_9fa48("1376", "1377", "1378"), totalCompletions === 1)) {
        if (stryMutAct_9fa48("1379")) {
          {}
        } else {
          stryCov_9fa48("1379");
          this.stats.averageDurationMs = duration;
        }
      } else {
        if (stryMutAct_9fa48("1380")) {
          {}
        } else {
          stryCov_9fa48("1380");
          const prevAverage = this.stats.averageDurationMs;
          this.stats.averageDurationMs = stryMutAct_9fa48("1381") ? (prevAverage * (totalCompletions - 1) + duration) * totalCompletions : (stryCov_9fa48("1381"), (stryMutAct_9fa48("1382") ? prevAverage * (totalCompletions - 1) - duration : (stryCov_9fa48("1382"), (stryMutAct_9fa48("1383") ? prevAverage / (totalCompletions - 1) : (stryCov_9fa48("1383"), prevAverage * (stryMutAct_9fa48("1384") ? totalCompletions + 1 : (stryCov_9fa48("1384"), totalCompletions - 1)))) + duration)) / totalCompletions);
        }
      }
    }
  }

  /**
   * Update success rate statistic
   */
  private updateSuccessRate(): void {
    if (stryMutAct_9fa48("1385")) {
      {}
    } else {
      stryCov_9fa48("1385");
      const totalResolved = stryMutAct_9fa48("1386") ? this.stats.completedCount + this.stats.failedCount - this.stats.timeoutCount : (stryCov_9fa48("1386"), (stryMutAct_9fa48("1387") ? this.stats.completedCount - this.stats.failedCount : (stryCov_9fa48("1387"), this.stats.completedCount + this.stats.failedCount)) + this.stats.timeoutCount);
      if (stryMutAct_9fa48("1391") ? totalResolved <= 0 : stryMutAct_9fa48("1390") ? totalResolved >= 0 : stryMutAct_9fa48("1389") ? false : stryMutAct_9fa48("1388") ? true : (stryCov_9fa48("1388", "1389", "1390", "1391"), totalResolved > 0)) {
        if (stryMutAct_9fa48("1392")) {
          {}
        } else {
          stryCov_9fa48("1392");
          this.stats.successRate = stryMutAct_9fa48("1393") ? this.stats.completedCount * totalResolved : (stryCov_9fa48("1393"), this.stats.completedCount / totalResolved);
        }
      }
    }
  }

  /**
   * Clean up timers for an assignment
   */
  private cleanupAssignmentTimers(assignmentId: string): void {
    if (stryMutAct_9fa48("1394")) {
      {}
    } else {
      stryCov_9fa48("1394");
      // Clear acknowledgment timeout
      const ackTimeoutKey = `${assignmentId}-ack`;
      const ackTimeout = this.timeouts.get(ackTimeoutKey);
      if (stryMutAct_9fa48("1397") ? false : stryMutAct_9fa48("1396") ? true : (stryCov_9fa48("1396", "1397"), ackTimeout)) {
        if (stryMutAct_9fa48("1398")) {
          {}
        } else {
          stryCov_9fa48("1398");
          clearTimeout(ackTimeout);
          this.timeouts.delete(ackTimeoutKey);
        }
      }

      // Clear progress check
      const progressCheck = this.progressChecks.get(assignmentId);
      if (stryMutAct_9fa48("1400") ? false : stryMutAct_9fa48("1399") ? true : (stryCov_9fa48("1399", "1400"), progressCheck)) {
        if (stryMutAct_9fa48("1401")) {
          {}
        } else {
          stryCov_9fa48("1401");
          clearInterval(progressCheck);
          this.progressChecks.delete(assignmentId);
        }
      }
    }
  }
}

/**
 * Task Assignment Factory
 *
 * Provides utilities for creating and managing task assignments.
 */
export class TaskAssignmentFactory {
  private manager: TaskAssignmentManager;
  constructor(config?: Partial<AssignmentConfig>) {
    if (stryMutAct_9fa48("1402")) {
      {}
    } else {
      stryCov_9fa48("1402");
      this.manager = new TaskAssignmentManager(config);
    }
  }

  /**
   * Create assignment from routing decision
   */
  async createFromRouting(task: Task, routingDecision: RoutingDecision, callbacks?: {
    onAcknowledgmentTimeout?: (assignment: TaskAssignment) => void;
    onProgressTimeout?: (assignment: TaskAssignment) => void;
  }): Promise<TaskAssignment> {
    if (stryMutAct_9fa48("1403")) {
      {}
    } else {
      stryCov_9fa48("1403");
      return this.manager.createAssignment(task, routingDecision, stryMutAct_9fa48("1404") ? callbacks.onAcknowledgmentTimeout : (stryCov_9fa48("1404"), callbacks?.onAcknowledgmentTimeout), stryMutAct_9fa48("1405") ? callbacks.onProgressTimeout : (stryCov_9fa48("1405"), callbacks?.onProgressTimeout));
    }
  }

  /**
   * Get assignment manager instance
   */
  getManager(): TaskAssignmentManager {
    if (stryMutAct_9fa48("1406")) {
      {}
    } else {
      stryCov_9fa48("1406");
      return this.manager;
    }
  }
}