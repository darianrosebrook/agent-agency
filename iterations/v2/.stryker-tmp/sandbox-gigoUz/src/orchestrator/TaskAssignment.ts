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
  private initialized: boolean = stryMutAct_9fa48("1006") ? true : (stryCov_9fa48("1006"), false);
  constructor(config: Partial<AssignmentConfig> = {}) {
    if (stryMutAct_9fa48("1007")) {
      {}
    } else {
      stryCov_9fa48("1007");
      this.config = stryMutAct_9fa48("1008") ? {} : (stryCov_9fa48("1008"), {
        acknowledgmentTimeoutMs: 5000,
        maxAssignmentDurationMs: 300000,
        // 5 minutes
        autoReassignmentEnabled: stryMutAct_9fa48("1009") ? false : (stryCov_9fa48("1009"), true),
        maxReassignmentAttempts: 3,
        progressCheckIntervalMs: 30000,
        // 30 seconds
        persistenceEnabled: stryMutAct_9fa48("1010") ? true : (stryCov_9fa48("1010"), false),
        ...config
      });

      // Initialize database client if persistence is enabled
      if (stryMutAct_9fa48("1012") ? false : stryMutAct_9fa48("1011") ? true : (stryCov_9fa48("1011", "1012"), this.config.persistenceEnabled)) {
        if (stryMutAct_9fa48("1013")) {
          {}
        } else {
          stryCov_9fa48("1013");
          this.dbClient = stryMutAct_9fa48("1016") ? this.config.databaseClient && DatabaseClientFactory.createMockClient() : stryMutAct_9fa48("1015") ? false : stryMutAct_9fa48("1014") ? true : (stryCov_9fa48("1014", "1015", "1016"), this.config.databaseClient || DatabaseClientFactory.createMockClient());
        }
      }
      this.stats = stryMutAct_9fa48("1017") ? {} : (stryCov_9fa48("1017"), {
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
    if (stryMutAct_9fa48("1018")) {
      {}
    } else {
      stryCov_9fa48("1018");
      if (stryMutAct_9fa48("1020") ? false : stryMutAct_9fa48("1019") ? true : (stryCov_9fa48("1019", "1020"), this.initialized)) {
        if (stryMutAct_9fa48("1021")) {
          {}
        } else {
          stryCov_9fa48("1021");
          return;
        }
      }
      try {
        if (stryMutAct_9fa48("1022")) {
          {}
        } else {
          stryCov_9fa48("1022");
          // Connect to database if persistence is enabled
          if (stryMutAct_9fa48("1025") ? this.config.persistenceEnabled || this.dbClient : stryMutAct_9fa48("1024") ? false : stryMutAct_9fa48("1023") ? true : (stryCov_9fa48("1023", "1024", "1025"), this.config.persistenceEnabled && this.dbClient)) {
            if (stryMutAct_9fa48("1026")) {
              {}
            } else {
              stryCov_9fa48("1026");
              await this.dbClient.connect();
            }
          }
          this.initialized = stryMutAct_9fa48("1027") ? false : (stryCov_9fa48("1027"), true);
          console.log("TaskAssignmentManager initialized successfully");
        }
      } catch (error) {
        if (stryMutAct_9fa48("1029")) {
          {}
        } else {
          stryCov_9fa48("1029");
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
    if (stryMutAct_9fa48("1031")) {
      {}
    } else {
      stryCov_9fa48("1031");
      if (stryMutAct_9fa48("1034") ? false : stryMutAct_9fa48("1033") ? true : stryMutAct_9fa48("1032") ? this.dbClient : (stryCov_9fa48("1032", "1033", "1034"), !this.dbClient)) {
        if (stryMutAct_9fa48("1035")) {
          {}
        } else {
          stryCov_9fa48("1035");
          return;
        }
      }
      try {
        if (stryMutAct_9fa48("1036")) {
          {}
        } else {
          stryCov_9fa48("1036");
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
        if (stryMutAct_9fa48("1040")) {
          {}
        } else {
          stryCov_9fa48("1040");
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
    if (stryMutAct_9fa48("1042")) {
      {}
    } else {
      stryCov_9fa48("1042");
      if (stryMutAct_9fa48("1045") ? !this.config.persistenceEnabled && !this.dbClient : stryMutAct_9fa48("1044") ? false : stryMutAct_9fa48("1043") ? true : (stryCov_9fa48("1043", "1044", "1045"), (stryMutAct_9fa48("1046") ? this.config.persistenceEnabled : (stryCov_9fa48("1046"), !this.config.persistenceEnabled)) || (stryMutAct_9fa48("1047") ? this.dbClient : (stryCov_9fa48("1047"), !this.dbClient)))) {
        if (stryMutAct_9fa48("1048")) {
          {}
        } else {
          stryCov_9fa48("1048");
          return;
        }
      }
      try {
        if (stryMutAct_9fa48("1049")) {
          {}
        } else {
          stryCov_9fa48("1049");
          const setParts: string[] = [];
          const values: any[] = [];
          let paramIndex = 1;
          if (stryMutAct_9fa48("1054") ? updates.status === undefined : stryMutAct_9fa48("1053") ? false : stryMutAct_9fa48("1052") ? true : (stryCov_9fa48("1052", "1053", "1054"), updates.status !== undefined)) {
            if (stryMutAct_9fa48("1055")) {
              {}
            } else {
              stryCov_9fa48("1055");
              setParts.push(`status = $${stryMutAct_9fa48("1057") ? paramIndex-- : (stryCov_9fa48("1057"), paramIndex++)}`);
              values.push(updates.status);
            }
          }
          if (stryMutAct_9fa48("1060") ? updates.acknowledgedAt === undefined : stryMutAct_9fa48("1059") ? false : stryMutAct_9fa48("1058") ? true : (stryCov_9fa48("1058", "1059", "1060"), updates.acknowledgedAt !== undefined)) {
            if (stryMutAct_9fa48("1061")) {
              {}
            } else {
              stryCov_9fa48("1061");
              setParts.push(`acknowledged_at = $${stryMutAct_9fa48("1063") ? paramIndex-- : (stryCov_9fa48("1063"), paramIndex++)}`);
              values.push(updates.acknowledgedAt);
            }
          }
          if (stryMutAct_9fa48("1066") ? updates.startedAt === undefined : stryMutAct_9fa48("1065") ? false : stryMutAct_9fa48("1064") ? true : (stryCov_9fa48("1064", "1065", "1066"), updates.startedAt !== undefined)) {
            if (stryMutAct_9fa48("1067")) {
              {}
            } else {
              stryCov_9fa48("1067");
              setParts.push(`started_at = $${stryMutAct_9fa48("1069") ? paramIndex-- : (stryCov_9fa48("1069"), paramIndex++)}`);
              values.push(updates.startedAt);
            }
          }
          if (stryMutAct_9fa48("1072") ? updates.completedAt === undefined : stryMutAct_9fa48("1071") ? false : stryMutAct_9fa48("1070") ? true : (stryCov_9fa48("1070", "1071", "1072"), updates.completedAt !== undefined)) {
            if (stryMutAct_9fa48("1073")) {
              {}
            } else {
              stryCov_9fa48("1073");
              setParts.push(`completed_at = $${stryMutAct_9fa48("1075") ? paramIndex-- : (stryCov_9fa48("1075"), paramIndex++)}`);
              values.push(updates.completedAt);
            }
          }
          if (stryMutAct_9fa48("1078") ? updates.progress === undefined : stryMutAct_9fa48("1077") ? false : stryMutAct_9fa48("1076") ? true : (stryCov_9fa48("1076", "1077", "1078"), updates.progress !== undefined)) {
            if (stryMutAct_9fa48("1079")) {
              {}
            } else {
              stryCov_9fa48("1079");
              setParts.push(`progress = $${stryMutAct_9fa48("1081") ? paramIndex-- : (stryCov_9fa48("1081"), paramIndex++)}, last_progress_update = NOW()`);
              values.push(updates.progress);
            }
          }
          if (stryMutAct_9fa48("1084") ? updates.errorMessage === undefined : stryMutAct_9fa48("1083") ? false : stryMutAct_9fa48("1082") ? true : (stryCov_9fa48("1082", "1083", "1084"), updates.errorMessage !== undefined)) {
            if (stryMutAct_9fa48("1085")) {
              {}
            } else {
              stryCov_9fa48("1085");
              setParts.push(`error_message = $${stryMutAct_9fa48("1087") ? paramIndex-- : (stryCov_9fa48("1087"), paramIndex++)}`);
              values.push(updates.errorMessage);
            }
          }
          if (stryMutAct_9fa48("1090") ? updates.errorCode === undefined : stryMutAct_9fa48("1089") ? false : stryMutAct_9fa48("1088") ? true : (stryCov_9fa48("1088", "1089", "1090"), updates.errorCode !== undefined)) {
            if (stryMutAct_9fa48("1091")) {
              {}
            } else {
              stryCov_9fa48("1091");
              setParts.push(`error_code = $${stryMutAct_9fa48("1093") ? paramIndex-- : (stryCov_9fa48("1093"), paramIndex++)}`);
              values.push(updates.errorCode);
            }
          }
          if (stryMutAct_9fa48("1096") ? setParts.length !== 0 : stryMutAct_9fa48("1095") ? false : stryMutAct_9fa48("1094") ? true : (stryCov_9fa48("1094", "1095", "1096"), setParts.length === 0)) {
            if (stryMutAct_9fa48("1097")) {
              {}
            } else {
              stryCov_9fa48("1097");
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
        if (stryMutAct_9fa48("1100")) {
          {}
        } else {
          stryCov_9fa48("1100");
          console.error(`Failed to update assignment status ${assignmentId}:`, error);
        }
      }
    }
  }

  /**
   * Create a new task assignment
   */
  async createAssignment(task: Task, routingDecision: RoutingDecision, onAssignmentTimeout?: (assignment: TaskAssignment) => void, onProgressTimeout?: (assignment: TaskAssignment) => void): Promise<TaskAssignment> {
    if (stryMutAct_9fa48("1102")) {
      {}
    } else {
      stryCov_9fa48("1102");
      const assignment: TaskAssignment = stryMutAct_9fa48("1103") ? {} : (stryCov_9fa48("1103"), {
        id: `assignment-${task.id}-${Date.now()}`,
        task,
        agent: routingDecision.selectedAgent,
        routingDecision,
        assignedAt: new Date(),
        deadline: new Date(stryMutAct_9fa48("1105") ? Date.now() - this.config.maxAssignmentDurationMs : (stryCov_9fa48("1105"), Date.now() + this.config.maxAssignmentDurationMs))
      });

      // Store assignment
      this.assignments.set(assignment.id, assignment);
      stryMutAct_9fa48("1106") ? this.stats.totalCreated-- : (stryCov_9fa48("1106"), this.stats.totalCreated++);
      stryMutAct_9fa48("1107") ? this.stats.activeCount-- : (stryCov_9fa48("1107"), this.stats.activeCount++);

      // Set acknowledgment timeout
      const ackTimeout = setTimeout(() => {
        if (stryMutAct_9fa48("1108")) {
          {}
        } else {
          stryCov_9fa48("1108");
          this.handleAcknowledgmentTimeout(assignment, onAssignmentTimeout);
        }
      }, this.config.acknowledgmentTimeoutMs);
      this.timeouts.set(`${assignment.id}-ack`, ackTimeout);

      // Set progress check interval
      const progressCheck = setInterval(() => {
        if (stryMutAct_9fa48("1110")) {
          {}
        } else {
          stryCov_9fa48("1110");
          this.checkProgressTimeout(assignment, onProgressTimeout);
        }
      }, this.config.progressCheckIntervalMs);
      this.progressChecks.set(assignment.id, progressCheck);

      // Persist assignment to database if enabled
      if (stryMutAct_9fa48("1112") ? false : stryMutAct_9fa48("1111") ? true : (stryCov_9fa48("1111", "1112"), this.config.persistenceEnabled)) {
        if (stryMutAct_9fa48("1113")) {
          {}
        } else {
          stryCov_9fa48("1113");
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
    if (stryMutAct_9fa48("1115")) {
      {}
    } else {
      stryCov_9fa48("1115");
      const assignment = this.assignments.get(assignmentId);
      if (stryMutAct_9fa48("1118") ? false : stryMutAct_9fa48("1117") ? true : stryMutAct_9fa48("1116") ? assignment : (stryCov_9fa48("1116", "1117", "1118"), !assignment)) {
        if (stryMutAct_9fa48("1119")) {
          {}
        } else {
          stryCov_9fa48("1119");
          return stryMutAct_9fa48("1120") ? true : (stryCov_9fa48("1120"), false);
        }
      }

      // Clear acknowledgment timeout
      const ackTimeoutKey = `${assignmentId}-ack`;
      const ackTimeout = this.timeouts.get(ackTimeoutKey);
      if (stryMutAct_9fa48("1123") ? false : stryMutAct_9fa48("1122") ? true : (stryCov_9fa48("1122", "1123"), ackTimeout)) {
        if (stryMutAct_9fa48("1124")) {
          {}
        } else {
          stryCov_9fa48("1124");
          clearTimeout(ackTimeout);
          this.timeouts.delete(ackTimeoutKey);
        }
      }

      // Create execution record
      const execution: TaskExecution = stryMutAct_9fa48("1125") ? {} : (stryCov_9fa48("1125"), {
        id: `execution-${assignment.task.id}-${Date.now()}`,
        assignment,
        startedAt: new Date(),
        status: "running",
        progress: 0,
        metadata: {}
      });
      this.executions.set(assignmentId, execution);

      // Update status in database
      if (stryMutAct_9fa48("1129") ? false : stryMutAct_9fa48("1128") ? true : (stryCov_9fa48("1128", "1129"), this.config.persistenceEnabled)) {
        if (stryMutAct_9fa48("1130")) {
          {}
        } else {
          stryCov_9fa48("1130");
          await this.updateAssignmentStatusInDb(assignmentId, stryMutAct_9fa48("1131") ? {} : (stryCov_9fa48("1131"), {
            acknowledgedAt: new Date(),
            startedAt: new Date()
          }));
        }
      }
      return stryMutAct_9fa48("1132") ? false : (stryCov_9fa48("1132"), true);
    }
  }

  /**
   * Update execution progress
   */
  async updateProgress(assignmentId: string, progress: number, status: TaskExecution["status"] = "running", metadata?: Record<string, any>): Promise<boolean> {
    if (stryMutAct_9fa48("1134")) {
      {}
    } else {
      stryCov_9fa48("1134");
      const execution = this.executions.get(assignmentId);
      if (stryMutAct_9fa48("1137") ? false : stryMutAct_9fa48("1136") ? true : stryMutAct_9fa48("1135") ? execution : (stryCov_9fa48("1135", "1136", "1137"), !execution)) {
        if (stryMutAct_9fa48("1138")) {
          {}
        } else {
          stryCov_9fa48("1138");
          return stryMutAct_9fa48("1139") ? true : (stryCov_9fa48("1139"), false);
        }
      }
      execution.progress = stryMutAct_9fa48("1140") ? Math.min(0, Math.min(1, progress)) : (stryCov_9fa48("1140"), Math.max(0, stryMutAct_9fa48("1141") ? Math.max(1, progress) : (stryCov_9fa48("1141"), Math.min(1, progress))));
      execution.status = status;
      if (stryMutAct_9fa48("1143") ? false : stryMutAct_9fa48("1142") ? true : (stryCov_9fa48("1142", "1143"), metadata)) {
        if (stryMutAct_9fa48("1144")) {
          {}
        } else {
          stryCov_9fa48("1144");
          execution.metadata = stryMutAct_9fa48("1145") ? {} : (stryCov_9fa48("1145"), {
            ...execution.metadata,
            ...metadata
          });
        }
      }

      // Reset progress timeout on any update
      this.resetProgressTimeout(assignmentId);

      // Update progress in database
      if (stryMutAct_9fa48("1147") ? false : stryMutAct_9fa48("1146") ? true : (stryCov_9fa48("1146", "1147"), this.config.persistenceEnabled)) {
        if (stryMutAct_9fa48("1148")) {
          {}
        } else {
          stryCov_9fa48("1148");
          await this.updateAssignmentStatusInDb(assignmentId, stryMutAct_9fa48("1149") ? {} : (stryCov_9fa48("1149"), {
            progress: execution.progress
          }));
        }
      }
      return stryMutAct_9fa48("1150") ? false : (stryCov_9fa48("1150"), true);
    }
  }

  /**
   * Complete assignment with result
   */
  completeAssignment(assignmentId: string, result: TaskResult, onCompletion?: (assignment: TaskAssignment, result: TaskResult) => void): boolean {
    if (stryMutAct_9fa48("1151")) {
      {}
    } else {
      stryCov_9fa48("1151");
      const assignment = this.assignments.get(assignmentId);
      const execution = this.executions.get(assignmentId);
      if (stryMutAct_9fa48("1154") ? !assignment && !execution : stryMutAct_9fa48("1153") ? false : stryMutAct_9fa48("1152") ? true : (stryCov_9fa48("1152", "1153", "1154"), (stryMutAct_9fa48("1155") ? assignment : (stryCov_9fa48("1155"), !assignment)) || (stryMutAct_9fa48("1156") ? execution : (stryCov_9fa48("1156"), !execution)))) {
        if (stryMutAct_9fa48("1157")) {
          {}
        } else {
          stryCov_9fa48("1157");
          return stryMutAct_9fa48("1158") ? true : (stryCov_9fa48("1158"), false);
        }
      }

      // Update execution
      execution.status = "completed";
      execution.progress = 1;

      // Calculate duration
      const duration = stryMutAct_9fa48("1160") ? Date.now() + assignment.assignedAt.getTime() : (stryCov_9fa48("1160"), Date.now() - assignment.assignedAt.getTime());

      // Update statistics
      stryMutAct_9fa48("1161") ? this.stats.activeCount++ : (stryCov_9fa48("1161"), this.stats.activeCount--);
      stryMutAct_9fa48("1162") ? this.stats.completedCount-- : (stryCov_9fa48("1162"), this.stats.completedCount++);
      this.updateAverageDuration(duration);
      this.updateSuccessRate();

      // Clean up timers
      this.cleanupAssignmentTimers(assignmentId);

      // Remove from active tracking
      this.assignments.delete(assignmentId);
      this.executions.delete(assignmentId);

      // Notify completion
      if (stryMutAct_9fa48("1164") ? false : stryMutAct_9fa48("1163") ? true : (stryCov_9fa48("1163", "1164"), onCompletion)) {
        if (stryMutAct_9fa48("1165")) {
          {}
        } else {
          stryCov_9fa48("1165");
          onCompletion(assignment, result);
        }
      }
      return stryMutAct_9fa48("1166") ? false : (stryCov_9fa48("1166"), true);
    }
  }

  /**
   * Fail assignment
   */
  failAssignment(assignmentId: string, error: string, canRetry: boolean = stryMutAct_9fa48("1167") ? false : (stryCov_9fa48("1167"), true), onFailure?: (assignment: TaskAssignment, error: string) => void): boolean {
    if (stryMutAct_9fa48("1168")) {
      {}
    } else {
      stryCov_9fa48("1168");
      const assignment = this.assignments.get(assignmentId);
      const execution = this.executions.get(assignmentId);
      if (stryMutAct_9fa48("1171") ? false : stryMutAct_9fa48("1170") ? true : stryMutAct_9fa48("1169") ? assignment : (stryCov_9fa48("1169", "1170", "1171"), !assignment)) {
        if (stryMutAct_9fa48("1172")) {
          {}
        } else {
          stryCov_9fa48("1172");
          return stryMutAct_9fa48("1173") ? true : (stryCov_9fa48("1173"), false);
        }
      }

      // Update execution if exists
      if (stryMutAct_9fa48("1175") ? false : stryMutAct_9fa48("1174") ? true : (stryCov_9fa48("1174", "1175"), execution)) {
        if (stryMutAct_9fa48("1176")) {
          {}
        } else {
          stryCov_9fa48("1176");
          execution.status = "failed";
        }
      }

      // Update statistics
      stryMutAct_9fa48("1178") ? this.stats.activeCount++ : (stryCov_9fa48("1178"), this.stats.activeCount--);
      stryMutAct_9fa48("1179") ? this.stats.failedCount-- : (stryCov_9fa48("1179"), this.stats.failedCount++);

      // Clean up timers
      this.cleanupAssignmentTimers(assignmentId);

      // Handle reassignment if enabled and possible
      let reassigned = stryMutAct_9fa48("1180") ? true : (stryCov_9fa48("1180"), false);
      if (stryMutAct_9fa48("1183") ? canRetry && this.config.autoReassignmentEnabled || assignment.task.attempts < assignment.task.maxAttempts : stryMutAct_9fa48("1182") ? false : stryMutAct_9fa48("1181") ? true : (stryCov_9fa48("1181", "1182", "1183"), (stryMutAct_9fa48("1185") ? canRetry || this.config.autoReassignmentEnabled : stryMutAct_9fa48("1184") ? true : (stryCov_9fa48("1184", "1185"), canRetry && this.config.autoReassignmentEnabled)) && (stryMutAct_9fa48("1188") ? assignment.task.attempts >= assignment.task.maxAttempts : stryMutAct_9fa48("1187") ? assignment.task.attempts <= assignment.task.maxAttempts : stryMutAct_9fa48("1186") ? true : (stryCov_9fa48("1186", "1187", "1188"), assignment.task.attempts < assignment.task.maxAttempts)))) {
        if (stryMutAct_9fa48("1189")) {
          {}
        } else {
          stryCov_9fa48("1189");
          reassigned = this.attemptReassignment();
        }
      }

      // Remove from active tracking if not reassigned
      if (stryMutAct_9fa48("1192") ? false : stryMutAct_9fa48("1191") ? true : stryMutAct_9fa48("1190") ? reassigned : (stryCov_9fa48("1190", "1191", "1192"), !reassigned)) {
        if (stryMutAct_9fa48("1193")) {
          {}
        } else {
          stryCov_9fa48("1193");
          this.assignments.delete(assignmentId);
          this.executions.delete(assignmentId);
        }
      }

      // Notify failure
      if (stryMutAct_9fa48("1195") ? false : stryMutAct_9fa48("1194") ? true : (stryCov_9fa48("1194", "1195"), onFailure)) {
        if (stryMutAct_9fa48("1196")) {
          {}
        } else {
          stryCov_9fa48("1196");
          onFailure(assignment, error);
        }
      }
      return stryMutAct_9fa48("1197") ? false : (stryCov_9fa48("1197"), true);
    }
  }

  /**
   * Get assignment by ID
   */
  getAssignment(assignmentId: string): TaskAssignment | null {
    if (stryMutAct_9fa48("1198")) {
      {}
    } else {
      stryCov_9fa48("1198");
      return stryMutAct_9fa48("1201") ? this.assignments.get(assignmentId) && null : stryMutAct_9fa48("1200") ? false : stryMutAct_9fa48("1199") ? true : (stryCov_9fa48("1199", "1200", "1201"), this.assignments.get(assignmentId) || null);
    }
  }

  /**
   * Get execution by assignment ID
   */
  getExecution(assignmentId: string): TaskExecution | null {
    if (stryMutAct_9fa48("1202")) {
      {}
    } else {
      stryCov_9fa48("1202");
      return stryMutAct_9fa48("1205") ? this.executions.get(assignmentId) && null : stryMutAct_9fa48("1204") ? false : stryMutAct_9fa48("1203") ? true : (stryCov_9fa48("1203", "1204", "1205"), this.executions.get(assignmentId) || null);
    }
  }

  /**
   * Get all active assignments
   */
  getActiveAssignments(): TaskAssignment[] {
    if (stryMutAct_9fa48("1206")) {
      {}
    } else {
      stryCov_9fa48("1206");
      return Array.from(this.assignments.values());
    }
  }

  /**
   * Get assignment statistics
   */
  getStats(): AssignmentStats {
    if (stryMutAct_9fa48("1207")) {
      {}
    } else {
      stryCov_9fa48("1207");
      return stryMutAct_9fa48("1208") ? {} : (stryCov_9fa48("1208"), {
        ...this.stats
      });
    }
  }

  /**
   * Force timeout an assignment
   */
  timeoutAssignment(assignmentId: string, onTimeout?: (assignment: TaskAssignment) => void): boolean {
    if (stryMutAct_9fa48("1209")) {
      {}
    } else {
      stryCov_9fa48("1209");
      const assignment = this.assignments.get(assignmentId);
      if (stryMutAct_9fa48("1212") ? false : stryMutAct_9fa48("1211") ? true : stryMutAct_9fa48("1210") ? assignment : (stryCov_9fa48("1210", "1211", "1212"), !assignment)) {
        if (stryMutAct_9fa48("1213")) {
          {}
        } else {
          stryCov_9fa48("1213");
          return stryMutAct_9fa48("1214") ? true : (stryCov_9fa48("1214"), false);
        }
      }

      // Update statistics
      stryMutAct_9fa48("1215") ? this.stats.activeCount++ : (stryCov_9fa48("1215"), this.stats.activeCount--);
      stryMutAct_9fa48("1216") ? this.stats.timeoutCount-- : (stryCov_9fa48("1216"), this.stats.timeoutCount++);

      // Clean up timers
      this.cleanupAssignmentTimers(assignmentId);

      // Remove from tracking
      this.assignments.delete(assignmentId);
      this.executions.delete(assignmentId);

      // Notify timeout
      if (stryMutAct_9fa48("1218") ? false : stryMutAct_9fa48("1217") ? true : (stryCov_9fa48("1217", "1218"), onTimeout)) {
        if (stryMutAct_9fa48("1219")) {
          {}
        } else {
          stryCov_9fa48("1219");
          onTimeout(assignment);
        }
      }
      return stryMutAct_9fa48("1220") ? false : (stryCov_9fa48("1220"), true);
    }
  }

  /**
   * Clean shutdown - cancel all active assignments
   */
  async shutdown(): Promise<void> {
    if (stryMutAct_9fa48("1221")) {
      {}
    } else {
      stryCov_9fa48("1221");
      // Clear all timers
      for (const timeout of Array.from(this.timeouts.values())) {
        if (stryMutAct_9fa48("1222")) {
          {}
        } else {
          stryCov_9fa48("1222");
          clearTimeout(timeout);
        }
      }
      this.timeouts.clear();
      for (const interval of Array.from(this.progressChecks.values())) {
        if (stryMutAct_9fa48("1223")) {
          {}
        } else {
          stryCov_9fa48("1223");
          clearInterval(interval);
        }
      }
      this.progressChecks.clear();

      // Cancel all active assignments
      const activeIds = Array.from(this.assignments.keys());
      for (const assignmentId of activeIds) {
        if (stryMutAct_9fa48("1224")) {
          {}
        } else {
          stryCov_9fa48("1224");
          this.failAssignment(assignmentId, "System shutdown", stryMutAct_9fa48("1226") ? true : (stryCov_9fa48("1226"), false));
        }
      }
    }
  }

  /**
   * Handle acknowledgment timeout
   */
  private handleAcknowledgmentTimeout(assignment: TaskAssignment, onTimeout?: (assignment: TaskAssignment) => void): void {
    if (stryMutAct_9fa48("1227")) {
      {}
    } else {
      stryCov_9fa48("1227");
      // Agent didn't acknowledge within timeout
      this.failAssignment(assignment.id, "Acknowledgment timeout", stryMutAct_9fa48("1229") ? false : (stryCov_9fa48("1229"), true), assignment => {
        if (stryMutAct_9fa48("1230")) {
          {}
        } else {
          stryCov_9fa48("1230");
          if (stryMutAct_9fa48("1232") ? false : stryMutAct_9fa48("1231") ? true : (stryCov_9fa48("1231", "1232"), onTimeout)) {
            if (stryMutAct_9fa48("1233")) {
              {}
            } else {
              stryCov_9fa48("1233");
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
    if (stryMutAct_9fa48("1234")) {
      {}
    } else {
      stryCov_9fa48("1234");
      const execution = this.executions.get(assignment.id);
      if (stryMutAct_9fa48("1237") ? false : stryMutAct_9fa48("1236") ? true : stryMutAct_9fa48("1235") ? execution : (stryCov_9fa48("1235", "1236", "1237"), !execution)) {
        if (stryMutAct_9fa48("1238")) {
          {}
        } else {
          stryCov_9fa48("1238");
          return;
        }
      }
      const timeSinceLastUpdate = stryMutAct_9fa48("1239") ? Date.now() + execution.startedAt.getTime() : (stryCov_9fa48("1239"), Date.now() - execution.startedAt.getTime());
      if (stryMutAct_9fa48("1243") ? timeSinceLastUpdate <= this.config.maxAssignmentDurationMs : stryMutAct_9fa48("1242") ? timeSinceLastUpdate >= this.config.maxAssignmentDurationMs : stryMutAct_9fa48("1241") ? false : stryMutAct_9fa48("1240") ? true : (stryCov_9fa48("1240", "1241", "1242", "1243"), timeSinceLastUpdate > this.config.maxAssignmentDurationMs)) {
        if (stryMutAct_9fa48("1244")) {
          {}
        } else {
          stryCov_9fa48("1244");
          this.timeoutAssignment(assignment.id, onTimeout);
        }
      }
    }
  }

  /**
   * Reset progress timeout
   */
  private resetProgressTimeout(assignmentId: string): void {
    if (stryMutAct_9fa48("1245")) {
      {}
    } else {
      stryCov_9fa48("1245");
      const progressCheck = this.progressChecks.get(assignmentId);
      if (stryMutAct_9fa48("1247") ? false : stryMutAct_9fa48("1246") ? true : (stryCov_9fa48("1246", "1247"), progressCheck)) {
        if (stryMutAct_9fa48("1248")) {
          {}
        } else {
          stryCov_9fa48("1248");
          clearInterval(progressCheck);
          const newProgressCheck = setInterval(() => {
            if (stryMutAct_9fa48("1249")) {
              {}
            } else {
              stryCov_9fa48("1249");
              const assignment = this.assignments.get(assignmentId);
              if (stryMutAct_9fa48("1251") ? false : stryMutAct_9fa48("1250") ? true : (stryCov_9fa48("1250", "1251"), assignment)) {
                if (stryMutAct_9fa48("1252")) {
                  {}
                } else {
                  stryCov_9fa48("1252");
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
    if (stryMutAct_9fa48("1253")) {
      {}
    } else {
      stryCov_9fa48("1253");
      // This would typically call back to the routing system
      // For now, we'll just mark it as reassigned in statistics
      stryMutAct_9fa48("1254") ? this.stats.reassignedCount-- : (stryCov_9fa48("1254"), this.stats.reassignedCount++);
      return stryMutAct_9fa48("1255") ? false : (stryCov_9fa48("1255"), true); // Assume reassignment was successful
    }
  }

  /**
   * Update average duration statistic
   */
  private updateAverageDuration(duration: number): void {
    if (stryMutAct_9fa48("1256")) {
      {}
    } else {
      stryCov_9fa48("1256");
      const totalCompletions = this.stats.completedCount;
      if (stryMutAct_9fa48("1259") ? totalCompletions !== 1 : stryMutAct_9fa48("1258") ? false : stryMutAct_9fa48("1257") ? true : (stryCov_9fa48("1257", "1258", "1259"), totalCompletions === 1)) {
        if (stryMutAct_9fa48("1260")) {
          {}
        } else {
          stryCov_9fa48("1260");
          this.stats.averageDurationMs = duration;
        }
      } else {
        if (stryMutAct_9fa48("1261")) {
          {}
        } else {
          stryCov_9fa48("1261");
          const prevAverage = this.stats.averageDurationMs;
          this.stats.averageDurationMs = stryMutAct_9fa48("1262") ? (prevAverage * (totalCompletions - 1) + duration) * totalCompletions : (stryCov_9fa48("1262"), (stryMutAct_9fa48("1263") ? prevAverage * (totalCompletions - 1) - duration : (stryCov_9fa48("1263"), (stryMutAct_9fa48("1264") ? prevAverage / (totalCompletions - 1) : (stryCov_9fa48("1264"), prevAverage * (stryMutAct_9fa48("1265") ? totalCompletions + 1 : (stryCov_9fa48("1265"), totalCompletions - 1)))) + duration)) / totalCompletions);
        }
      }
    }
  }

  /**
   * Update success rate statistic
   */
  private updateSuccessRate(): void {
    if (stryMutAct_9fa48("1266")) {
      {}
    } else {
      stryCov_9fa48("1266");
      const totalResolved = stryMutAct_9fa48("1267") ? this.stats.completedCount + this.stats.failedCount - this.stats.timeoutCount : (stryCov_9fa48("1267"), (stryMutAct_9fa48("1268") ? this.stats.completedCount - this.stats.failedCount : (stryCov_9fa48("1268"), this.stats.completedCount + this.stats.failedCount)) + this.stats.timeoutCount);
      if (stryMutAct_9fa48("1272") ? totalResolved <= 0 : stryMutAct_9fa48("1271") ? totalResolved >= 0 : stryMutAct_9fa48("1270") ? false : stryMutAct_9fa48("1269") ? true : (stryCov_9fa48("1269", "1270", "1271", "1272"), totalResolved > 0)) {
        if (stryMutAct_9fa48("1273")) {
          {}
        } else {
          stryCov_9fa48("1273");
          this.stats.successRate = stryMutAct_9fa48("1274") ? this.stats.completedCount * totalResolved : (stryCov_9fa48("1274"), this.stats.completedCount / totalResolved);
        }
      }
    }
  }

  /**
   * Clean up timers for an assignment
   */
  private cleanupAssignmentTimers(assignmentId: string): void {
    if (stryMutAct_9fa48("1275")) {
      {}
    } else {
      stryCov_9fa48("1275");
      // Clear acknowledgment timeout
      const ackTimeoutKey = `${assignmentId}-ack`;
      const ackTimeout = this.timeouts.get(ackTimeoutKey);
      if (stryMutAct_9fa48("1278") ? false : stryMutAct_9fa48("1277") ? true : (stryCov_9fa48("1277", "1278"), ackTimeout)) {
        if (stryMutAct_9fa48("1279")) {
          {}
        } else {
          stryCov_9fa48("1279");
          clearTimeout(ackTimeout);
          this.timeouts.delete(ackTimeoutKey);
        }
      }

      // Clear progress check
      const progressCheck = this.progressChecks.get(assignmentId);
      if (stryMutAct_9fa48("1281") ? false : stryMutAct_9fa48("1280") ? true : (stryCov_9fa48("1280", "1281"), progressCheck)) {
        if (stryMutAct_9fa48("1282")) {
          {}
        } else {
          stryCov_9fa48("1282");
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
    if (stryMutAct_9fa48("1283")) {
      {}
    } else {
      stryCov_9fa48("1283");
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
    if (stryMutAct_9fa48("1284")) {
      {}
    } else {
      stryCov_9fa48("1284");
      return this.manager.createAssignment(task, routingDecision, stryMutAct_9fa48("1285") ? callbacks.onAcknowledgmentTimeout : (stryCov_9fa48("1285"), callbacks?.onAcknowledgmentTimeout), stryMutAct_9fa48("1286") ? callbacks.onProgressTimeout : (stryCov_9fa48("1286"), callbacks?.onProgressTimeout));
    }
  }

  /**
   * Get assignment manager instance
   */
  getManager(): TaskAssignmentManager {
    if (stryMutAct_9fa48("1287")) {
      {}
    } else {
      stryCov_9fa48("1287");
      return this.manager;
    }
  }
}