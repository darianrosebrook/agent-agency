/**
 * @fileoverview PostgreSQL Database Client for Agent Registry (ARBITER-001)
 *
 * Provides persistent storage for agent profiles, capabilities, and performance history.
 * Implements ACID transactions and connection pooling for production reliability.
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
import { Pool, PoolClient } from "pg";
import { AgentId, AgentProfile, AgentQuery, PerformanceMetrics, RegistryStats } from "../types/agent-registry";

/**
 * Database Configuration
 */
export interface DatabaseConfig {
  /** PostgreSQL connection string or config */
  host: string;
  port: number;
  database: string;
  user: string;
  password: string;

  /** Connection pool settings */
  poolMin: number;
  poolMax: number;
  poolIdleTimeoutMs: number;
  poolConnectionTimeoutMs: number;

  /** Query timeouts */
  queryTimeoutMs: number;

  /** Enable query logging */
  enableQueryLogging: boolean;

  /** Enable connection retries */
  enableRetries: boolean;
  maxRetries: number;
  retryDelayMs: number;
}

/**
 * Database Client for Agent Registry
 *
 * Provides ACID-compliant persistent storage for agent registry data.
 */
export class AgentRegistryDatabaseClient {
  private pool: Pool;
  private config: DatabaseConfig;
  constructor(config: Partial<DatabaseConfig> = {}) {
    if (stryMutAct_9fa48("0")) {
      {}
    } else {
      stryCov_9fa48("0");
      this.config = stryMutAct_9fa48("1") ? {} : (stryCov_9fa48("1"), {
        host: stryMutAct_9fa48("4") ? process.env.DB_HOST && "localhost" : stryMutAct_9fa48("3") ? false : stryMutAct_9fa48("2") ? true : (stryCov_9fa48("2", "3", "4"), process.env.DB_HOST || (stryMutAct_9fa48("5") ? "" : (stryCov_9fa48("5"), "localhost"))),
        port: parseInt(stryMutAct_9fa48("8") ? process.env.DB_PORT && "5432" : stryMutAct_9fa48("7") ? false : stryMutAct_9fa48("6") ? true : (stryCov_9fa48("6", "7", "8"), process.env.DB_PORT || (stryMutAct_9fa48("9") ? "" : (stryCov_9fa48("9"), "5432")))),
        database: stryMutAct_9fa48("12") ? process.env.DB_NAME && "agent_agency_v2" : stryMutAct_9fa48("11") ? false : stryMutAct_9fa48("10") ? true : (stryCov_9fa48("10", "11", "12"), process.env.DB_NAME || (stryMutAct_9fa48("13") ? "" : (stryCov_9fa48("13"), "agent_agency_v2"))),
        user: stryMutAct_9fa48("16") ? process.env.DB_USER && "postgres" : stryMutAct_9fa48("15") ? false : stryMutAct_9fa48("14") ? true : (stryCov_9fa48("14", "15", "16"), process.env.DB_USER || (stryMutAct_9fa48("17") ? "" : (stryCov_9fa48("17"), "postgres"))),
        password: stryMutAct_9fa48("20") ? process.env.DB_PASSWORD && "" : stryMutAct_9fa48("19") ? false : stryMutAct_9fa48("18") ? true : (stryCov_9fa48("18", "19", "20"), process.env.DB_PASSWORD || (stryMutAct_9fa48("21") ? "Stryker was here!" : (stryCov_9fa48("21"), ""))),
        poolMin: 2,
        poolMax: 10,
        poolIdleTimeoutMs: 30000,
        poolConnectionTimeoutMs: 10000,
        queryTimeoutMs: 5000,
        enableQueryLogging: stryMutAct_9fa48("22") ? true : (stryCov_9fa48("22"), false),
        enableRetries: stryMutAct_9fa48("23") ? false : (stryCov_9fa48("23"), true),
        maxRetries: 3,
        retryDelayMs: 1000,
        ...config
      });
      this.pool = new Pool(stryMutAct_9fa48("24") ? {} : (stryCov_9fa48("24"), {
        host: this.config.host,
        port: this.config.port,
        database: this.config.database,
        user: this.config.user,
        password: this.config.password,
        min: this.config.poolMin,
        max: this.config.poolMax,
        idleTimeoutMillis: this.config.poolIdleTimeoutMs,
        connectionTimeoutMillis: this.config.poolConnectionTimeoutMs,
        statement_timeout: this.config.queryTimeoutMs
      }));
      this.pool.on(stryMutAct_9fa48("25") ? "" : (stryCov_9fa48("25"), "error"), err => {
        if (stryMutAct_9fa48("26")) {
          {}
        } else {
          stryCov_9fa48("26");
          console.error(stryMutAct_9fa48("27") ? "" : (stryCov_9fa48("27"), "Unexpected database pool error:"), err);
        }
      });
    }
  }

  /**
   * Initialize database connection and verify schema
   */
  async initialize(): Promise<void> {
    if (stryMutAct_9fa48("28")) {
      {}
    } else {
      stryCov_9fa48("28");
      try {
        if (stryMutAct_9fa48("29")) {
          {}
        } else {
          stryCov_9fa48("29");
          const client = await this.pool.connect();
          try {
            if (stryMutAct_9fa48("30")) {
              {}
            } else {
              stryCov_9fa48("30");
              // Verify connection
              await client.query(stryMutAct_9fa48("31") ? "" : (stryCov_9fa48("31"), "SELECT 1"));

              // Verify schema exists
              const schemaCheck = await client.query(stryMutAct_9fa48("32") ? `` : (stryCov_9fa48("32"), `
          SELECT table_name 
          FROM information_schema.tables 
          WHERE table_schema = 'public' 
          AND table_name IN ('agent_profiles', 'agent_capabilities', 'performance_history')
        `));
              if (stryMutAct_9fa48("36") ? schemaCheck.rows.length >= 3 : stryMutAct_9fa48("35") ? schemaCheck.rows.length <= 3 : stryMutAct_9fa48("34") ? false : stryMutAct_9fa48("33") ? true : (stryCov_9fa48("33", "34", "35", "36"), schemaCheck.rows.length < 3)) {
                if (stryMutAct_9fa48("37")) {
                  {}
                } else {
                  stryCov_9fa48("37");
                  throw new Error(stryMutAct_9fa48("38") ? "" : (stryCov_9fa48("38"), "Database schema not initialized. Run migrations first: psql < migrations/001_create_agent_registry_tables.sql"));
                }
              }
            }
          } finally {
            if (stryMutAct_9fa48("39")) {
              {}
            } else {
              stryCov_9fa48("39");
              client.release();
            }
          }
        }
      } catch (error) {
        if (stryMutAct_9fa48("40")) {
          {}
        } else {
          stryCov_9fa48("40");
          throw new Error(stryMutAct_9fa48("41") ? `` : (stryCov_9fa48("41"), `Database initialization failed: ${error instanceof Error ? error.message : String(error)}`));
        }
      }
    }
  }

  /**
   * Register a new agent (INSERT)
   */
  async registerAgent(agent: AgentProfile): Promise<void> {
    if (stryMutAct_9fa48("42")) {
      {}
    } else {
      stryCov_9fa48("42");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("43")) {
          {}
        } else {
          stryCov_9fa48("43");
          await client.query(stryMutAct_9fa48("44") ? "" : (stryCov_9fa48("44"), "BEGIN"));

          // Insert agent profile
          await client.query(stryMutAct_9fa48("45") ? `` : (stryCov_9fa48("45"), `
        INSERT INTO agent_profiles (
          id, name, model_family, registered_at, last_active_at
        ) VALUES ($1, $2, $3, $4, $5)
        `), stryMutAct_9fa48("46") ? [] : (stryCov_9fa48("46"), [agent.id, agent.name, agent.modelFamily, agent.registeredAt, agent.lastActiveAt]));

          // Insert capabilities
          for (const taskType of agent.capabilities.taskTypes) {
            if (stryMutAct_9fa48("47")) {
              {}
            } else {
              stryCov_9fa48("47");
              await client.query(stryMutAct_9fa48("48") ? `` : (stryCov_9fa48("48"), `
          INSERT INTO agent_capabilities (agent_id, capability_type, capability_value)
          VALUES ($1, 'task_type', $2)
          `), stryMutAct_9fa48("49") ? [] : (stryCov_9fa48("49"), [agent.id, taskType]));
            }
          }
          for (const language of agent.capabilities.languages) {
            if (stryMutAct_9fa48("50")) {
              {}
            } else {
              stryCov_9fa48("50");
              await client.query(stryMutAct_9fa48("51") ? `` : (stryCov_9fa48("51"), `
          INSERT INTO agent_capabilities (agent_id, capability_type, capability_value)
          VALUES ($1, 'language', $2)
          `), stryMutAct_9fa48("52") ? [] : (stryCov_9fa48("52"), [agent.id, language]));
            }
          }
          for (const specialization of agent.capabilities.specializations) {
            if (stryMutAct_9fa48("53")) {
              {}
            } else {
              stryCov_9fa48("53");
              await client.query(stryMutAct_9fa48("54") ? `` : (stryCov_9fa48("54"), `
          INSERT INTO agent_capabilities (agent_id, capability_type, capability_value)
          VALUES ($1, 'specialization', $2)
          `), stryMutAct_9fa48("55") ? [] : (stryCov_9fa48("55"), [agent.id, specialization]));
            }
          }

          // Insert performance history
          await client.query(stryMutAct_9fa48("56") ? `` : (stryCov_9fa48("56"), `
        INSERT INTO performance_history (
          agent_id, success_rate, average_quality, average_latency, task_count
        ) VALUES ($1, $2, $3, $4, $5)
        `), stryMutAct_9fa48("57") ? [] : (stryCov_9fa48("57"), [agent.id, agent.performanceHistory.successRate, agent.performanceHistory.averageQuality, agent.performanceHistory.averageLatency, agent.performanceHistory.taskCount]));

          // Insert current load
          await client.query(stryMutAct_9fa48("58") ? `` : (stryCov_9fa48("58"), `
        INSERT INTO current_load (
          agent_id, active_tasks, queued_tasks, utilization_percent
        ) VALUES ($1, $2, $3, $4)
        `), stryMutAct_9fa48("59") ? [] : (stryCov_9fa48("59"), [agent.id, agent.currentLoad.activeTasks, agent.currentLoad.queuedTasks, agent.currentLoad.utilizationPercent]));
          await client.query(stryMutAct_9fa48("60") ? "" : (stryCov_9fa48("60"), "COMMIT"));
        }
      } catch (error) {
        if (stryMutAct_9fa48("61")) {
          {}
        } else {
          stryCov_9fa48("61");
          await client.query(stryMutAct_9fa48("62") ? "" : (stryCov_9fa48("62"), "ROLLBACK"));
          throw new Error(stryMutAct_9fa48("63") ? `` : (stryCov_9fa48("63"), `Failed to register agent: ${error instanceof Error ? error.message : String(error)}`));
        }
      } finally {
        if (stryMutAct_9fa48("64")) {
          {}
        } else {
          stryCov_9fa48("64");
          client.release();
        }
      }
    }
  }

  /**
   * Get agent profile by ID (SELECT)
   */
  async getAgent(agentId: AgentId): Promise<AgentProfile | null> {
    if (stryMutAct_9fa48("65")) {
      {}
    } else {
      stryCov_9fa48("65");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("66")) {
          {}
        } else {
          stryCov_9fa48("66");
          // Use the view that joins all data
          const result = await client.query(stryMutAct_9fa48("67") ? `` : (stryCov_9fa48("67"), `
        SELECT * FROM agent_profiles_with_capabilities
        WHERE id = $1
        `), stryMutAct_9fa48("68") ? [] : (stryCov_9fa48("68"), [agentId]));
          if (stryMutAct_9fa48("71") ? result.rows.length !== 0 : stryMutAct_9fa48("70") ? false : stryMutAct_9fa48("69") ? true : (stryCov_9fa48("69", "70", "71"), result.rows.length === 0)) {
            if (stryMutAct_9fa48("72")) {
              {}
            } else {
              stryCov_9fa48("72");
              return null;
            }
          }
          return this.mapRowToProfile(result.rows[0]);
        }
      } finally {
        if (stryMutAct_9fa48("73")) {
          {}
        } else {
          stryCov_9fa48("73");
          client.release();
        }
      }
    }
  }

  /**
   * Get all agents (SELECT)
   */
  async getAllAgents(): Promise<AgentProfile[]> {
    if (stryMutAct_9fa48("74")) {
      {}
    } else {
      stryCov_9fa48("74");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("75")) {
          {}
        } else {
          stryCov_9fa48("75");
          const result = await client.query(stryMutAct_9fa48("76") ? `` : (stryCov_9fa48("76"), `
        SELECT * FROM agent_profiles_with_capabilities
        ORDER BY last_active_at DESC
      `));
          return result.rows.map(stryMutAct_9fa48("77") ? () => undefined : (stryCov_9fa48("77"), row => this.mapRowToProfile(row)));
        }
      } finally {
        if (stryMutAct_9fa48("78")) {
          {}
        } else {
          stryCov_9fa48("78");
          client.release();
        }
      }
    }
  }

  /**
   * Query agents by capability
   */
  async queryAgentsByCapability(query: AgentQuery): Promise<AgentProfile[]> {
    if (stryMutAct_9fa48("79")) {
      {}
    } else {
      stryCov_9fa48("79");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("80")) {
          {}
        } else {
          stryCov_9fa48("80");
          let sql = stryMutAct_9fa48("81") ? `` : (stryCov_9fa48("81"), `
        SELECT DISTINCT ap.* 
        FROM agent_profiles_with_capabilities ap
        WHERE 1=1
      `);
          const params: any[] = stryMutAct_9fa48("82") ? ["Stryker was here"] : (stryCov_9fa48("82"), []);
          let paramIndex = 1;

          // Filter by task type
          if (stryMutAct_9fa48("84") ? false : stryMutAct_9fa48("83") ? true : (stryCov_9fa48("83", "84"), query.taskType)) {
            if (stryMutAct_9fa48("85")) {
              {}
            } else {
              stryCov_9fa48("85");
              sql += stryMutAct_9fa48("86") ? `` : (stryCov_9fa48("86"), ` AND $${paramIndex} = ANY(ap.task_types)`);
              params.push(query.taskType);
              stryMutAct_9fa48("87") ? paramIndex-- : (stryCov_9fa48("87"), paramIndex++);
            }
          }

          // Filter by languages
          if (stryMutAct_9fa48("90") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("89") ? false : stryMutAct_9fa48("88") ? true : (stryCov_9fa48("88", "89", "90"), query.languages && (stryMutAct_9fa48("93") ? query.languages.length <= 0 : stryMutAct_9fa48("92") ? query.languages.length >= 0 : stryMutAct_9fa48("91") ? true : (stryCov_9fa48("91", "92", "93"), query.languages.length > 0)))) {
            if (stryMutAct_9fa48("94")) {
              {}
            } else {
              stryCov_9fa48("94");
              sql += stryMutAct_9fa48("95") ? `` : (stryCov_9fa48("95"), ` AND ap.languages && $${paramIndex}::text[]`);
              params.push(query.languages);
              stryMutAct_9fa48("96") ? paramIndex-- : (stryCov_9fa48("96"), paramIndex++);
            }
          }

          // Filter by utilization
          if (stryMutAct_9fa48("99") ? query.maxUtilization === undefined : stryMutAct_9fa48("98") ? false : stryMutAct_9fa48("97") ? true : (stryCov_9fa48("97", "98", "99"), query.maxUtilization !== undefined)) {
            if (stryMutAct_9fa48("100")) {
              {}
            } else {
              stryCov_9fa48("100");
              sql += stryMutAct_9fa48("101") ? `` : (stryCov_9fa48("101"), ` AND ap.utilization_percent <= $${paramIndex}`);
              params.push(query.maxUtilization);
              stryMutAct_9fa48("102") ? paramIndex-- : (stryCov_9fa48("102"), paramIndex++);
            }
          }

          // Filter by success rate
          if (stryMutAct_9fa48("105") ? query.minSuccessRate === undefined : stryMutAct_9fa48("104") ? false : stryMutAct_9fa48("103") ? true : (stryCov_9fa48("103", "104", "105"), query.minSuccessRate !== undefined)) {
            if (stryMutAct_9fa48("106")) {
              {}
            } else {
              stryCov_9fa48("106");
              sql += stryMutAct_9fa48("107") ? `` : (stryCov_9fa48("107"), ` AND ap.success_rate >= $${paramIndex}`);
              params.push(query.minSuccessRate);
              stryMutAct_9fa48("108") ? paramIndex-- : (stryCov_9fa48("108"), paramIndex++);
            }
          }

          // Order by success rate
          sql += stryMutAct_9fa48("109") ? `` : (stryCov_9fa48("109"), ` ORDER BY ap.success_rate DESC`);
          const result = await client.query(sql, params);
          return result.rows.map(stryMutAct_9fa48("110") ? () => undefined : (stryCov_9fa48("110"), row => this.mapRowToProfile(row)));
        }
      } finally {
        if (stryMutAct_9fa48("111")) {
          {}
        } else {
          stryCov_9fa48("111");
          client.release();
        }
      }
    }
  }

  /**
   * Update performance history (UPDATE)
   */
  async updatePerformance(agentId: AgentId, metrics: PerformanceMetrics): Promise<void> {
    if (stryMutAct_9fa48("112")) {
      {}
    } else {
      stryCov_9fa48("112");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("113")) {
          {}
        } else {
          stryCov_9fa48("113");
          await client.query(stryMutAct_9fa48("114") ? "" : (stryCov_9fa48("114"), "BEGIN"));

          // Get current performance history
          const currentResult = await client.query(stryMutAct_9fa48("115") ? `` : (stryCov_9fa48("115"), `SELECT * FROM performance_history WHERE agent_id = $1`), stryMutAct_9fa48("116") ? [] : (stryCov_9fa48("116"), [agentId]));
          if (stryMutAct_9fa48("119") ? currentResult.rows.length !== 0 : stryMutAct_9fa48("118") ? false : stryMutAct_9fa48("117") ? true : (stryCov_9fa48("117", "118", "119"), currentResult.rows.length === 0)) {
            if (stryMutAct_9fa48("120")) {
              {}
            } else {
              stryCov_9fa48("120");
              throw new Error(stryMutAct_9fa48("121") ? `` : (stryCov_9fa48("121"), `Agent ${agentId} not found`));
            }
          }
          const current = currentResult.rows[0];

          // Calculate new running averages
          const taskCount = stryMutAct_9fa48("122") ? current.task_count - 1 : (stryCov_9fa48("122"), current.task_count + 1);
          const successRate = stryMutAct_9fa48("123") ? (current.success_rate * current.task_count + (metrics.success ? 1 : 0)) * taskCount : (stryCov_9fa48("123"), (stryMutAct_9fa48("124") ? current.success_rate * current.task_count - (metrics.success ? 1 : 0) : (stryCov_9fa48("124"), (stryMutAct_9fa48("125") ? current.success_rate / current.task_count : (stryCov_9fa48("125"), current.success_rate * current.task_count)) + (metrics.success ? 1 : 0))) / taskCount);
          const averageQuality = stryMutAct_9fa48("126") ? (current.average_quality * current.task_count + metrics.qualityScore) * taskCount : (stryCov_9fa48("126"), (stryMutAct_9fa48("127") ? current.average_quality * current.task_count - metrics.qualityScore : (stryCov_9fa48("127"), (stryMutAct_9fa48("128") ? current.average_quality / current.task_count : (stryCov_9fa48("128"), current.average_quality * current.task_count)) + metrics.qualityScore)) / taskCount);
          const averageLatency = stryMutAct_9fa48("129") ? (current.average_latency * current.task_count + metrics.latencyMs) * taskCount : (stryCov_9fa48("129"), (stryMutAct_9fa48("130") ? current.average_latency * current.task_count - metrics.latencyMs : (stryCov_9fa48("130"), (stryMutAct_9fa48("131") ? current.average_latency / current.task_count : (stryCov_9fa48("131"), current.average_latency * current.task_count)) + metrics.latencyMs)) / taskCount);

          // Update performance history
          await client.query(stryMutAct_9fa48("132") ? `` : (stryCov_9fa48("132"), `
        UPDATE performance_history
        SET success_rate = $1,
            average_quality = $2,
            average_latency = $3,
            task_count = $4,
            updated_at = CURRENT_TIMESTAMP
        WHERE agent_id = $5
        `), stryMutAct_9fa48("133") ? [] : (stryCov_9fa48("133"), [successRate, averageQuality, averageLatency, taskCount, agentId]));

          // Insert performance event for audit trail
          await client.query(stryMutAct_9fa48("134") ? `` : (stryCov_9fa48("134"), `
        INSERT INTO agent_performance_events (
          agent_id, success, quality_score, latency_ms, tokens_used, task_type
        ) VALUES ($1, $2, $3, $4, $5, $6)
        `), stryMutAct_9fa48("135") ? [] : (stryCov_9fa48("135"), [agentId, metrics.success, metrics.qualityScore, metrics.latencyMs, metrics.tokensUsed, metrics.taskType]));

          // Update last active timestamp
          await client.query(stryMutAct_9fa48("136") ? `` : (stryCov_9fa48("136"), `UPDATE agent_profiles SET last_active_at = CURRENT_TIMESTAMP WHERE id = $1`), stryMutAct_9fa48("137") ? [] : (stryCov_9fa48("137"), [agentId]));
          await client.query(stryMutAct_9fa48("138") ? "" : (stryCov_9fa48("138"), "COMMIT"));
        }
      } catch (error) {
        if (stryMutAct_9fa48("139")) {
          {}
        } else {
          stryCov_9fa48("139");
          await client.query(stryMutAct_9fa48("140") ? "" : (stryCov_9fa48("140"), "ROLLBACK"));
          throw new Error(stryMutAct_9fa48("141") ? `` : (stryCov_9fa48("141"), `Failed to update performance: ${error instanceof Error ? error.message : String(error)}`));
        }
      } finally {
        if (stryMutAct_9fa48("142")) {
          {}
        } else {
          stryCov_9fa48("142");
          client.release();
        }
      }
    }
  }

  /**
   * Update agent load (UPDATE)
   */
  async updateLoad(agentId: AgentId, activeTasksDelta: number, queuedTasksDelta: number): Promise<void> {
    if (stryMutAct_9fa48("143")) {
      {}
    } else {
      stryCov_9fa48("143");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("144")) {
          {}
        } else {
          stryCov_9fa48("144");
          // Update with atomic increment/decrement
          await client.query(stryMutAct_9fa48("145") ? `` : (stryCov_9fa48("145"), `
        UPDATE current_load
        SET active_tasks = GREATEST(0, active_tasks + $1),
            queued_tasks = GREATEST(0, queued_tasks + $2),
            utilization_percent = LEAST(100, (GREATEST(0, active_tasks + $1)::float / NULLIF(10, 0)) * 100),
            updated_at = CURRENT_TIMESTAMP
        WHERE agent_id = $3
        `), stryMutAct_9fa48("146") ? [] : (stryCov_9fa48("146"), [activeTasksDelta, queuedTasksDelta, agentId]));
        }
      } finally {
        if (stryMutAct_9fa48("147")) {
          {}
        } else {
          stryCov_9fa48("147");
          client.release();
        }
      }
    }
  }

  /**
   * Unregister agent (DELETE)
   */
  async unregisterAgent(agentId: AgentId): Promise<boolean> {
    if (stryMutAct_9fa48("148")) {
      {}
    } else {
      stryCov_9fa48("148");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("149")) {
          {}
        } else {
          stryCov_9fa48("149");
          await client.query(stryMutAct_9fa48("150") ? "" : (stryCov_9fa48("150"), "BEGIN"));

          // Delete cascades to all related tables (configured in migration)
          const result = await client.query(stryMutAct_9fa48("151") ? `` : (stryCov_9fa48("151"), `DELETE FROM agent_profiles WHERE id = $1`), stryMutAct_9fa48("152") ? [] : (stryCov_9fa48("152"), [agentId]));
          await client.query(stryMutAct_9fa48("153") ? "" : (stryCov_9fa48("153"), "COMMIT"));
          return stryMutAct_9fa48("156") ? result.rowCount !== null || result.rowCount > 0 : stryMutAct_9fa48("155") ? false : stryMutAct_9fa48("154") ? true : (stryCov_9fa48("154", "155", "156"), (stryMutAct_9fa48("158") ? result.rowCount === null : stryMutAct_9fa48("157") ? true : (stryCov_9fa48("157", "158"), result.rowCount !== null)) && (stryMutAct_9fa48("161") ? result.rowCount <= 0 : stryMutAct_9fa48("160") ? result.rowCount >= 0 : stryMutAct_9fa48("159") ? true : (stryCov_9fa48("159", "160", "161"), result.rowCount > 0)));
        }
      } catch (error) {
        if (stryMutAct_9fa48("162")) {
          {}
        } else {
          stryCov_9fa48("162");
          await client.query(stryMutAct_9fa48("163") ? "" : (stryCov_9fa48("163"), "ROLLBACK"));
          throw new Error(stryMutAct_9fa48("164") ? `` : (stryCov_9fa48("164"), `Failed to unregister agent: ${error instanceof Error ? error.message : String(error)}`));
        }
      } finally {
        if (stryMutAct_9fa48("165")) {
          {}
        } else {
          stryCov_9fa48("165");
          client.release();
        }
      }
    }
  }

  /**
   * Get registry statistics
   */
  async getStats(): Promise<RegistryStats> {
    if (stryMutAct_9fa48("166")) {
      {}
    } else {
      stryCov_9fa48("166");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("167")) {
          {}
        } else {
          stryCov_9fa48("167");
          const result = await client.query(stryMutAct_9fa48("168") ? `` : (stryCov_9fa48("168"), `
        SELECT 
          COUNT(*) as total_agents,
          COUNT(*) FILTER (WHERE active_tasks > 0) as active_agents,
          COUNT(*) FILTER (WHERE active_tasks = 0) as idle_agents,
          AVG(utilization_percent) as avg_utilization,
          AVG(success_rate) as avg_success_rate,
          AVG(average_quality) as avg_quality,
          MAX(last_active_at) as last_updated
        FROM agent_profiles_with_capabilities
      `));
          const stats = result.rows[0];
          return stryMutAct_9fa48("169") ? {} : (stryCov_9fa48("169"), {
            totalAgents: parseInt(stats.total_agents),
            activeAgents: parseInt(stats.active_agents),
            idleAgents: parseInt(stats.idle_agents),
            averageUtilization: stryMutAct_9fa48("172") ? parseFloat(stats.avg_utilization) && 0 : stryMutAct_9fa48("171") ? false : stryMutAct_9fa48("170") ? true : (stryCov_9fa48("170", "171", "172"), parseFloat(stats.avg_utilization) || 0),
            averageSuccessRate: stryMutAct_9fa48("175") ? parseFloat(stats.avg_success_rate) && 0 : stryMutAct_9fa48("174") ? false : stryMutAct_9fa48("173") ? true : (stryCov_9fa48("173", "174", "175"), parseFloat(stats.avg_success_rate) || 0),
            lastUpdated: stryMutAct_9fa48("178") ? stats.last_updated && new Date().toISOString() : stryMutAct_9fa48("177") ? false : stryMutAct_9fa48("176") ? true : (stryCov_9fa48("176", "177", "178"), stats.last_updated || new Date().toISOString())
          });
        }
      } finally {
        if (stryMutAct_9fa48("179")) {
          {}
        } else {
          stryCov_9fa48("179");
          client.release();
        }
      }
    }
  }

  /**
   * Clean up stale agents
   */
  async cleanupStaleAgents(staleThresholdMs: number): Promise<number> {
    if (stryMutAct_9fa48("180")) {
      {}
    } else {
      stryCov_9fa48("180");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("181")) {
          {}
        } else {
          stryCov_9fa48("181");
          const staleTimestamp = new Date(stryMutAct_9fa48("182") ? Date.now() + staleThresholdMs : (stryCov_9fa48("182"), Date.now() - staleThresholdMs)).toISOString();
          const result = await client.query(stryMutAct_9fa48("183") ? `` : (stryCov_9fa48("183"), `
        DELETE FROM agent_profiles 
        WHERE last_active_at < $1
        RETURNING id
        `), stryMutAct_9fa48("184") ? [] : (stryCov_9fa48("184"), [staleTimestamp]));
          return stryMutAct_9fa48("187") ? result.rowCount && 0 : stryMutAct_9fa48("186") ? false : stryMutAct_9fa48("185") ? true : (stryCov_9fa48("185", "186", "187"), result.rowCount || 0);
        }
      } finally {
        if (stryMutAct_9fa48("188")) {
          {}
        } else {
          stryCov_9fa48("188");
          client.release();
        }
      }
    }
  }

  /**
   * Health check
   */
  async healthCheck(): Promise<{
    healthy: boolean;
    latencyMs: number;
    poolStats: {
      total: number;
      idle: number;
      waiting: number;
    };
  }> {
    if (stryMutAct_9fa48("189")) {
      {}
    } else {
      stryCov_9fa48("189");
      const startTime = Date.now();
      try {
        if (stryMutAct_9fa48("190")) {
          {}
        } else {
          stryCov_9fa48("190");
          const client = await this.pool.connect();
          try {
            if (stryMutAct_9fa48("191")) {
              {}
            } else {
              stryCov_9fa48("191");
              await client.query(stryMutAct_9fa48("192") ? "" : (stryCov_9fa48("192"), "SELECT 1"));
              const latencyMs = stryMutAct_9fa48("193") ? Date.now() + startTime : (stryCov_9fa48("193"), Date.now() - startTime);
              return stryMutAct_9fa48("194") ? {} : (stryCov_9fa48("194"), {
                healthy: stryMutAct_9fa48("195") ? false : (stryCov_9fa48("195"), true),
                latencyMs,
                poolStats: stryMutAct_9fa48("196") ? {} : (stryCov_9fa48("196"), {
                  total: this.pool.totalCount,
                  idle: this.pool.idleCount,
                  waiting: this.pool.waitingCount
                })
              });
            }
          } finally {
            if (stryMutAct_9fa48("197")) {
              {}
            } else {
              stryCov_9fa48("197");
              client.release();
            }
          }
        }
      } catch (error) {
        if (stryMutAct_9fa48("198")) {
          {}
        } else {
          stryCov_9fa48("198");
          return stryMutAct_9fa48("199") ? {} : (stryCov_9fa48("199"), {
            healthy: stryMutAct_9fa48("200") ? true : (stryCov_9fa48("200"), false),
            latencyMs: stryMutAct_9fa48("201") ? Date.now() + startTime : (stryCov_9fa48("201"), Date.now() - startTime),
            poolStats: stryMutAct_9fa48("202") ? {} : (stryCov_9fa48("202"), {
              total: this.pool.totalCount,
              idle: this.pool.idleCount,
              waiting: this.pool.waitingCount
            })
          });
        }
      }
    }
  }

  /**
   * Close database connection pool
   */
  async close(): Promise<void> {
    if (stryMutAct_9fa48("203")) {
      {}
    } else {
      stryCov_9fa48("203");
      await this.pool.end();
    }
  }

  /**
   * Map database row to AgentProfile
   */
  private mapRowToProfile(row: any): AgentProfile {
    if (stryMutAct_9fa48("204")) {
      {}
    } else {
      stryCov_9fa48("204");
      return stryMutAct_9fa48("205") ? {} : (stryCov_9fa48("205"), {
        id: row.id,
        name: row.name,
        modelFamily: row.model_family,
        capabilities: stryMutAct_9fa48("206") ? {} : (stryCov_9fa48("206"), {
          taskTypes: stryMutAct_9fa48("209") ? row.task_types && [] : stryMutAct_9fa48("208") ? false : stryMutAct_9fa48("207") ? true : (stryCov_9fa48("207", "208", "209"), row.task_types || (stryMutAct_9fa48("210") ? ["Stryker was here"] : (stryCov_9fa48("210"), []))),
          languages: stryMutAct_9fa48("213") ? row.languages && [] : stryMutAct_9fa48("212") ? false : stryMutAct_9fa48("211") ? true : (stryCov_9fa48("211", "212", "213"), row.languages || (stryMutAct_9fa48("214") ? ["Stryker was here"] : (stryCov_9fa48("214"), []))),
          specializations: stryMutAct_9fa48("217") ? row.specializations && [] : stryMutAct_9fa48("216") ? false : stryMutAct_9fa48("215") ? true : (stryCov_9fa48("215", "216", "217"), row.specializations || (stryMutAct_9fa48("218") ? ["Stryker was here"] : (stryCov_9fa48("218"), [])))
        }),
        performanceHistory: stryMutAct_9fa48("219") ? {} : (stryCov_9fa48("219"), {
          successRate: stryMutAct_9fa48("222") ? parseFloat(row.success_rate) && 0 : stryMutAct_9fa48("221") ? false : stryMutAct_9fa48("220") ? true : (stryCov_9fa48("220", "221", "222"), parseFloat(row.success_rate) || 0),
          averageQuality: stryMutAct_9fa48("225") ? parseFloat(row.average_quality) && 0 : stryMutAct_9fa48("224") ? false : stryMutAct_9fa48("223") ? true : (stryCov_9fa48("223", "224", "225"), parseFloat(row.average_quality) || 0),
          averageLatency: stryMutAct_9fa48("228") ? parseFloat(row.average_latency) && 0 : stryMutAct_9fa48("227") ? false : stryMutAct_9fa48("226") ? true : (stryCov_9fa48("226", "227", "228"), parseFloat(row.average_latency) || 0),
          taskCount: stryMutAct_9fa48("231") ? parseInt(row.task_count) && 0 : stryMutAct_9fa48("230") ? false : stryMutAct_9fa48("229") ? true : (stryCov_9fa48("229", "230", "231"), parseInt(row.task_count) || 0)
        }),
        currentLoad: stryMutAct_9fa48("232") ? {} : (stryCov_9fa48("232"), {
          activeTasks: stryMutAct_9fa48("235") ? parseInt(row.active_tasks) && 0 : stryMutAct_9fa48("234") ? false : stryMutAct_9fa48("233") ? true : (stryCov_9fa48("233", "234", "235"), parseInt(row.active_tasks) || 0),
          queuedTasks: stryMutAct_9fa48("238") ? parseInt(row.queued_tasks) && 0 : stryMutAct_9fa48("237") ? false : stryMutAct_9fa48("236") ? true : (stryCov_9fa48("236", "237", "238"), parseInt(row.queued_tasks) || 0),
          utilizationPercent: stryMutAct_9fa48("241") ? parseFloat(row.utilization_percent) && 0 : stryMutAct_9fa48("240") ? false : stryMutAct_9fa48("239") ? true : (stryCov_9fa48("239", "240", "241"), parseFloat(row.utilization_percent) || 0)
        }),
        registeredAt: row.registered_at,
        lastActiveAt: row.last_active_at
      });
    }
  }

  /**
   * Execute query with retry logic
   */
  private async executeWithRetry<T>(operation: (client: PoolClient) => Promise<T>): Promise<T> {
    if (stryMutAct_9fa48("242")) {
      {}
    } else {
      stryCov_9fa48("242");
      let lastError: Error | null = null;
      for (let attempt = 0; stryMutAct_9fa48("245") ? attempt >= this.config.maxRetries : stryMutAct_9fa48("244") ? attempt <= this.config.maxRetries : stryMutAct_9fa48("243") ? false : (stryCov_9fa48("243", "244", "245"), attempt < this.config.maxRetries); stryMutAct_9fa48("246") ? attempt-- : (stryCov_9fa48("246"), attempt++)) {
        if (stryMutAct_9fa48("247")) {
          {}
        } else {
          stryCov_9fa48("247");
          try {
            if (stryMutAct_9fa48("248")) {
              {}
            } else {
              stryCov_9fa48("248");
              const client = await this.pool.connect();
              try {
                if (stryMutAct_9fa48("249")) {
                  {}
                } else {
                  stryCov_9fa48("249");
                  return await operation(client);
                }
              } finally {
                if (stryMutAct_9fa48("250")) {
                  {}
                } else {
                  stryCov_9fa48("250");
                  client.release();
                }
              }
            }
          } catch (error) {
            if (stryMutAct_9fa48("251")) {
              {}
            } else {
              stryCov_9fa48("251");
              lastError = error instanceof Error ? error : new Error(String(error));
              if (stryMutAct_9fa48("253") ? false : stryMutAct_9fa48("252") ? true : (stryCov_9fa48("252", "253"), this.config.enableQueryLogging)) {
                if (stryMutAct_9fa48("254")) {
                  {}
                } else {
                  stryCov_9fa48("254");
                  console.log(stryMutAct_9fa48("255") ? `` : (stryCov_9fa48("255"), `Query attempt ${stryMutAct_9fa48("256") ? attempt - 1 : (stryCov_9fa48("256"), attempt + 1)}/${this.config.maxRetries} failed:`), lastError.message);
                }
              }
              if (stryMutAct_9fa48("260") ? attempt >= this.config.maxRetries - 1 : stryMutAct_9fa48("259") ? attempt <= this.config.maxRetries - 1 : stryMutAct_9fa48("258") ? false : stryMutAct_9fa48("257") ? true : (stryCov_9fa48("257", "258", "259", "260"), attempt < (stryMutAct_9fa48("261") ? this.config.maxRetries + 1 : (stryCov_9fa48("261"), this.config.maxRetries - 1)))) {
                if (stryMutAct_9fa48("262")) {
                  {}
                } else {
                  stryCov_9fa48("262");
                  await new Promise(stryMutAct_9fa48("263") ? () => undefined : (stryCov_9fa48("263"), resolve => setTimeout(resolve, this.config.retryDelayMs)));
                }
              }
            }
          }
        }
      }
      throw stryMutAct_9fa48("266") ? lastError && new Error("Query failed after retries") : stryMutAct_9fa48("265") ? false : stryMutAct_9fa48("264") ? true : (stryCov_9fa48("264", "265", "266"), lastError || new Error(stryMutAct_9fa48("267") ? "" : (stryCov_9fa48("267"), "Query failed after retries")));
    }
  }
}