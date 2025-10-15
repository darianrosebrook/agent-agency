/**
 * Agent Registry Database Client
 *
 * PostgreSQL client for the Agent Registry Manager (ARBITER-001).
 * Provides ACID-compliant persistence for agent profiles, capabilities, and performance history.
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
import { AgentCapabilities, AgentProfile, AgentQuery, AgentQueryResult, DatabaseConfig, PerformanceHistory, PerformanceMetrics, ProgrammingLanguage, Specialization, TaskType } from "../types/agent-registry.js";
import { Logger } from "../utils/Logger.js";
export interface AgentRegistryDatabaseConfig extends DatabaseConfig {
  maxConnections: number;
  connectionTimeoutMs: number;
  queryTimeoutMs: number;
  retryAttempts: number;
  retryDelayMs: number;
}
export class AgentRegistryDbClient {
  private pool: Pool;
  private logger: Logger;
  private config: AgentRegistryDatabaseConfig;
  constructor(config: AgentRegistryDatabaseConfig | {
    host: string;
    port: number;
    database: string;
    user: string;
    password: string;
  }) {
    if (stryMutAct_9fa48("0")) {
      {}
    } else {
      stryCov_9fa48("0");
      // Handle legacy constructor for backward compatibility
      if (stryMutAct_9fa48("3") ? "host" in config && "user" in config || !("maxConnections" in config) : stryMutAct_9fa48("2") ? false : stryMutAct_9fa48("1") ? true : (stryCov_9fa48("1", "2", "3"), (stryMutAct_9fa48("5") ? "host" in config || "user" in config : stryMutAct_9fa48("4") ? true : (stryCov_9fa48("4", "5"), "host" in config && "user" in config)) && (stryMutAct_9fa48("8") ? "maxConnections" in config : (stryCov_9fa48("8"), !("maxConnections" in config))))) {
        if (stryMutAct_9fa48("10")) {
          {}
        } else {
          stryCov_9fa48("10");
          this.config = stryMutAct_9fa48("11") ? {} : (stryCov_9fa48("11"), {
            host: config.host,
            port: config.port,
            database: config.database,
            username: config.user,
            password: config.password,
            maxConnections: 10,
            connectionTimeoutMs: 10000,
            queryTimeoutMs: 30000,
            retryAttempts: 3,
            retryDelayMs: 1000
          });
        }
      } else {
        if (stryMutAct_9fa48("12")) {
          {}
        } else {
          stryCov_9fa48("12");
          this.config = config as AgentRegistryDatabaseConfig;
        }
      }
      this.logger = new Logger("AgentRegistryDbClient");
      this.pool = new Pool(stryMutAct_9fa48("14") ? {} : (stryCov_9fa48("14"), {
        host: this.config.host,
        port: this.config.port,
        database: this.config.database,
        user: this.config.username,
        password: this.config.password,
        max: this.config.maxConnections,
        connectionTimeoutMillis: this.config.connectionTimeoutMs,
        query_timeout: this.config.queryTimeoutMs,
        ssl: stryMutAct_9fa48("15") ? true : (stryCov_9fa48("15"), false) // Disable SSL for tests
      }));
      this.setupPoolErrorHandling();
    }
  }

  /**
   * Initialize database connection and verify schema
   */
  async initialize(): Promise<void> {
    if (stryMutAct_9fa48("16")) {
      {}
    } else {
      stryCov_9fa48("16");
      try {
        if (stryMutAct_9fa48("17")) {
          {}
        } else {
          stryCov_9fa48("17");
          this.logger.info("Initializing Agent Registry database client...");

          // Test connection
          const client = await this.pool.connect();
          try {
            if (stryMutAct_9fa48("19")) {
              {}
            } else {
              stryCov_9fa48("19");
              await client.query("SELECT 1");
              this.logger.info("Database connection established");
            }
          } finally {
            if (stryMutAct_9fa48("22")) {
              {}
            } else {
              stryCov_9fa48("22");
              client.release();
            }
          }

          // Verify schema exists
          await this.verifySchema();
          this.logger.info("Database schema verified");
          this.logger.info("Agent Registry database client initialized successfully");
        }
      } catch (error) {
        if (stryMutAct_9fa48("25")) {
          {}
        } else {
          stryCov_9fa48("25");
          this.logger.error("Failed to initialize database client:", error);
          throw new Error(`Database initialization failed: ${error instanceof Error ? error.message : String(error)}`);
        }
      }
    }
  }

  /**
   * Clean shutdown of database connections
   */
  async shutdown(): Promise<void> {
    if (stryMutAct_9fa48("28")) {
      {}
    } else {
      stryCov_9fa48("28");
      try {
        if (stryMutAct_9fa48("29")) {
          {}
        } else {
          stryCov_9fa48("29");
          this.logger.info("Shutting down Agent Registry database client...");
          await this.pool.end();
          this.logger.info("Database connections closed");
        }
      } catch (error) {
        if (stryMutAct_9fa48("32")) {
          {}
        } else {
          stryCov_9fa48("32");
          this.logger.error("Error during database shutdown:", error);
        }
      }
    }
  }

  /**
   * Register a new agent profile
   */
  async registerAgent(profile: Omit<AgentProfile, "id" | "registeredAt" | "lastActiveAt" | "createdAt" | "updatedAt">): Promise<string> {
    if (stryMutAct_9fa48("34")) {
      {}
    } else {
      stryCov_9fa48("34");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("35")) {
          {}
        } else {
          stryCov_9fa48("35");
          await client.query("BEGIN");

          // Insert agent profile
          const profileResult = await client.query(`
        INSERT INTO agent_profiles (
          name, model_family, active_tasks, queued_tasks, utilization_percent
        ) VALUES ($1, $2, $3, $4, $5)
        RETURNING id
      `, [profile.name, profile.modelFamily, stryMutAct_9fa48("41") ? profile.currentLoad?.activeTasks && 0 : stryMutAct_9fa48("40") ? false : stryMutAct_9fa48("39") ? true : (stryCov_9fa48("39", "40", "41"), (stryMutAct_9fa48("42") ? profile.currentLoad.activeTasks : (stryCov_9fa48("42"), profile.currentLoad?.activeTasks)) || 0), stryMutAct_9fa48("45") ? profile.currentLoad?.queuedTasks && 0 : stryMutAct_9fa48("44") ? false : stryMutAct_9fa48("43") ? true : (stryCov_9fa48("43", "44", "45"), (stryMutAct_9fa48("46") ? profile.currentLoad.queuedTasks : (stryCov_9fa48("46"), profile.currentLoad?.queuedTasks)) || 0), stryMutAct_9fa48("49") ? profile.currentLoad?.utilizationPercent && 0 : stryMutAct_9fa48("48") ? false : stryMutAct_9fa48("47") ? true : (stryCov_9fa48("47", "48", "49"), (stryMutAct_9fa48("50") ? profile.currentLoad.utilizationPercent : (stryCov_9fa48("50"), profile.currentLoad?.utilizationPercent)) || 0)]);
          const agentId = profileResult.rows[0].id;

          // Insert capabilities if provided
          if (stryMutAct_9fa48("52") ? false : stryMutAct_9fa48("51") ? true : (stryCov_9fa48("51", "52"), profile.capabilities)) {
            if (stryMutAct_9fa48("53")) {
              {}
            } else {
              stryCov_9fa48("53");
              // Insert task types
              for (const taskType of profile.capabilities.taskTypes) {
                if (stryMutAct_9fa48("54")) {
                  {}
                } else {
                  stryCov_9fa48("54");
                  await client.query(`
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `, [agentId, `task_${taskType}`, 1.0,
                  // Default score for task types
                  JSON.stringify(stryMutAct_9fa48("58") ? {} : (stryCov_9fa48("58"), {
                    type: "task",
                    value: taskType
                  }))]);
                }
              }

              // Insert languages
              for (const language of profile.capabilities.languages) {
                if (stryMutAct_9fa48("60")) {
                  {}
                } else {
                  stryCov_9fa48("60");
                  await client.query(`
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `, [agentId, `lang_${language}`, 1.0,
                  // Default score for languages
                  JSON.stringify(stryMutAct_9fa48("64") ? {} : (stryCov_9fa48("64"), {
                    type: "language",
                    value: language
                  }))]);
                }
              }

              // Insert specializations
              for (const specialization of profile.capabilities.specializations) {
                if (stryMutAct_9fa48("66")) {
                  {}
                } else {
                  stryCov_9fa48("66");
                  await client.query(`
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `, [agentId, `spec_${specialization}`, 1.0,
                  // Default score for specializations
                  JSON.stringify(stryMutAct_9fa48("70") ? {} : (stryCov_9fa48("70"), {
                    type: "specialization",
                    value: specialization
                  }))]);
                }
              }
            }
          }

          // Insert performance history if provided
          if (stryMutAct_9fa48("73") ? false : stryMutAct_9fa48("72") ? true : (stryCov_9fa48("72", "73"), profile.performanceHistory)) {
            if (stryMutAct_9fa48("74")) {
              {}
            } else {
              stryCov_9fa48("74");
              await client.query(`
          INSERT INTO agent_performance_history (
            agent_id, task_type, success_rate, average_latency,
            total_tasks, quality_score, confidence_score, metadata
          ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        `, [agentId, "general",
              // Default task type for overall performance
              profile.performanceHistory.successRate, profile.performanceHistory.averageLatency, profile.performanceHistory.taskCount, profile.performanceHistory.averageQuality, 1.0,
              // Default confidence score
              JSON.stringify({})]);
            }
          }
          await client.query("COMMIT");
          this.logger.info(`Agent registered with ID: ${agentId}`);
          return agentId;
        }
      } catch (error) {
        if (stryMutAct_9fa48("80")) {
          {}
        } else {
          stryCov_9fa48("80");
          await client.query("ROLLBACK");
          this.logger.error("Failed to register agent:", error);
          throw new Error(`Agent registration failed: ${error instanceof Error ? error.message : String(error)}`);
        }
      } finally {
        if (stryMutAct_9fa48("84")) {
          {}
        } else {
          stryCov_9fa48("84");
          client.release();
        }
      }
    }
  }

  /**
   * Get agent profile by ID
   */
  async getAgent(agentId: string): Promise<AgentProfile | null> {
    if (stryMutAct_9fa48("85")) {
      {}
    } else {
      stryCov_9fa48("85");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("86")) {
          {}
        } else {
          stryCov_9fa48("86");
          // Get profile
          const profileResult = await client.query(`
        SELECT * FROM agent_profiles WHERE id = $1
      `, [agentId]);
          if (stryMutAct_9fa48("91") ? profileResult.rows.length !== 0 : stryMutAct_9fa48("90") ? false : stryMutAct_9fa48("89") ? true : (stryCov_9fa48("89", "90", "91"), profileResult.rows.length === 0)) {
            if (stryMutAct_9fa48("92")) {
              {}
            } else {
              stryCov_9fa48("92");
              return null;
            }
          }
          const profileRow = profileResult.rows[0];

          // Get capabilities
          const capabilitiesResult = await client.query(`
        SELECT capability_name, score, metadata FROM agent_capabilities
        WHERE agent_id = $1 ORDER BY capability_name
      `, [agentId]);

          // Reconstruct capabilities from database records
          const taskTypes: TaskType[] = [];
          const languages: ProgrammingLanguage[] = [];
          const specializations: Specialization[] = [];
          capabilitiesResult.rows.forEach(row => {
            if (stryMutAct_9fa48("98")) {
              {}
            } else {
              stryCov_9fa48("98");
              const metadata = stryMutAct_9fa48("101") ? row.metadata && {} : stryMutAct_9fa48("100") ? false : stryMutAct_9fa48("99") ? true : (stryCov_9fa48("99", "100", "101"), row.metadata || {});
              if (stryMutAct_9fa48("104") ? metadata.type !== "task" : stryMutAct_9fa48("103") ? false : stryMutAct_9fa48("102") ? true : (stryCov_9fa48("102", "103", "104"), metadata.type === "task")) {
                if (stryMutAct_9fa48("106")) {
                  {}
                } else {
                  stryCov_9fa48("106");
                  taskTypes.push(metadata.value);
                }
              } else if (stryMutAct_9fa48("109") ? metadata.type !== "language" : stryMutAct_9fa48("108") ? false : stryMutAct_9fa48("107") ? true : (stryCov_9fa48("107", "108", "109"), metadata.type === "language")) {
                if (stryMutAct_9fa48("111")) {
                  {}
                } else {
                  stryCov_9fa48("111");
                  languages.push(metadata.value);
                }
              } else if (stryMutAct_9fa48("114") ? metadata.type !== "specialization" : stryMutAct_9fa48("113") ? false : stryMutAct_9fa48("112") ? true : (stryCov_9fa48("112", "113", "114"), metadata.type === "specialization")) {
                if (stryMutAct_9fa48("116")) {
                  {}
                } else {
                  stryCov_9fa48("116");
                  specializations.push(metadata.value);
                }
              }
            }
          });
          const capabilities: AgentCapabilities = stryMutAct_9fa48("117") ? {} : (stryCov_9fa48("117"), {
            taskTypes,
            languages,
            specializations
          });

          // Get performance history (take the most recent record)
          const performanceResult = await client.query(`
        SELECT * FROM agent_performance_history
        WHERE agent_id = $1 ORDER BY recorded_at DESC LIMIT 1
      `, [agentId]);
          const performanceHistory: PerformanceHistory = (stryMutAct_9fa48("123") ? performanceResult.rows.length <= 0 : stryMutAct_9fa48("122") ? performanceResult.rows.length >= 0 : stryMutAct_9fa48("121") ? false : stryMutAct_9fa48("120") ? true : (stryCov_9fa48("120", "121", "122", "123"), performanceResult.rows.length > 0)) ? stryMutAct_9fa48("124") ? {} : (stryCov_9fa48("124"), {
            successRate: performanceResult.rows[0].success_rate,
            averageQuality: performanceResult.rows[0].quality_score,
            averageLatency: performanceResult.rows[0].average_latency,
            taskCount: performanceResult.rows[0].total_tasks
          }) : stryMutAct_9fa48("125") ? {} : (stryCov_9fa48("125"), {
            successRate: 0,
            averageQuality: 0,
            averageLatency: 0,
            taskCount: 0
          });
          return stryMutAct_9fa48("126") ? {} : (stryCov_9fa48("126"), {
            id: profileRow.id,
            name: profileRow.name,
            modelFamily: profileRow.model_family,
            capabilities,
            performanceHistory,
            currentLoad: stryMutAct_9fa48("127") ? {} : (stryCov_9fa48("127"), {
              activeTasks: stryMutAct_9fa48("130") ? profileRow.active_tasks && 0 : stryMutAct_9fa48("129") ? false : stryMutAct_9fa48("128") ? true : (stryCov_9fa48("128", "129", "130"), profileRow.active_tasks || 0),
              queuedTasks: stryMutAct_9fa48("133") ? profileRow.queued_tasks && 0 : stryMutAct_9fa48("132") ? false : stryMutAct_9fa48("131") ? true : (stryCov_9fa48("131", "132", "133"), profileRow.queued_tasks || 0),
              utilizationPercent: stryMutAct_9fa48("136") ? profileRow.utilization_percent && 0 : stryMutAct_9fa48("135") ? false : stryMutAct_9fa48("134") ? true : (stryCov_9fa48("134", "135", "136"), profileRow.utilization_percent || 0)
            }),
            registeredAt: profileRow.registered_at,
            lastActiveAt: profileRow.last_active_at
          });
        }
      } catch (error) {
        if (stryMutAct_9fa48("137")) {
          {}
        } else {
          stryCov_9fa48("137");
          this.logger.error("Failed to get agent:", error);
          throw new Error(`Failed to retrieve agent: ${error instanceof Error ? error.message : String(error)}`);
        }
      } finally {
        if (stryMutAct_9fa48("140")) {
          {}
        } else {
          stryCov_9fa48("140");
          client.release();
        }
      }
    }
  }

  /**
   * Update agent profile
   */
  async updateAgent(agentId: string, updates: Partial<AgentProfile>): Promise<void> {
    if (stryMutAct_9fa48("141")) {
      {}
    } else {
      stryCov_9fa48("141");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("142")) {
          {}
        } else {
          stryCov_9fa48("142");
          await client.query("BEGIN");

          // Update profile
          const updateFields: string[] = [];
          const values: any[] = [];
          let paramIndex = 1;
          if (stryMutAct_9fa48("148") ? updates.name === undefined : stryMutAct_9fa48("147") ? false : stryMutAct_9fa48("146") ? true : (stryCov_9fa48("146", "147", "148"), updates.name !== undefined)) {
            if (stryMutAct_9fa48("149")) {
              {}
            } else {
              stryCov_9fa48("149");
              updateFields.push(`name = $${stryMutAct_9fa48("151") ? paramIndex-- : (stryCov_9fa48("151"), paramIndex++)}`);
              values.push(updates.name);
            }
          }
          if (stryMutAct_9fa48("154") ? updates.lastActiveAt === undefined : stryMutAct_9fa48("153") ? false : stryMutAct_9fa48("152") ? true : (stryCov_9fa48("152", "153", "154"), updates.lastActiveAt !== undefined)) {
            if (stryMutAct_9fa48("155")) {
              {}
            } else {
              stryCov_9fa48("155");
              updateFields.push(`last_active_at = $${stryMutAct_9fa48("157") ? paramIndex-- : (stryCov_9fa48("157"), paramIndex++)}`);
              values.push(updates.lastActiveAt);
            }
          }
          if (stryMutAct_9fa48("160") ? updates.currentLoad?.activeTasks === undefined : stryMutAct_9fa48("159") ? false : stryMutAct_9fa48("158") ? true : (stryCov_9fa48("158", "159", "160"), (stryMutAct_9fa48("161") ? updates.currentLoad.activeTasks : (stryCov_9fa48("161"), updates.currentLoad?.activeTasks)) !== undefined)) {
            if (stryMutAct_9fa48("162")) {
              {}
            } else {
              stryCov_9fa48("162");
              updateFields.push(`active_tasks = $${stryMutAct_9fa48("164") ? paramIndex-- : (stryCov_9fa48("164"), paramIndex++)}`);
              values.push(updates.currentLoad.activeTasks);
            }
          }
          if (stryMutAct_9fa48("167") ? updates.currentLoad?.queuedTasks === undefined : stryMutAct_9fa48("166") ? false : stryMutAct_9fa48("165") ? true : (stryCov_9fa48("165", "166", "167"), (stryMutAct_9fa48("168") ? updates.currentLoad.queuedTasks : (stryCov_9fa48("168"), updates.currentLoad?.queuedTasks)) !== undefined)) {
            if (stryMutAct_9fa48("169")) {
              {}
            } else {
              stryCov_9fa48("169");
              updateFields.push(`queued_tasks = $${stryMutAct_9fa48("171") ? paramIndex-- : (stryCov_9fa48("171"), paramIndex++)}`);
              values.push(updates.currentLoad.queuedTasks);
            }
          }
          if (stryMutAct_9fa48("174") ? updates.currentLoad?.utilizationPercent === undefined : stryMutAct_9fa48("173") ? false : stryMutAct_9fa48("172") ? true : (stryCov_9fa48("172", "173", "174"), (stryMutAct_9fa48("175") ? updates.currentLoad.utilizationPercent : (stryCov_9fa48("175"), updates.currentLoad?.utilizationPercent)) !== undefined)) {
            if (stryMutAct_9fa48("176")) {
              {}
            } else {
              stryCov_9fa48("176");
              updateFields.push(`utilization_percent = $${stryMutAct_9fa48("178") ? paramIndex-- : (stryCov_9fa48("178"), paramIndex++)}`);
              values.push(updates.currentLoad.utilizationPercent);
            }
          }
          if (stryMutAct_9fa48("182") ? updateFields.length <= 0 : stryMutAct_9fa48("181") ? updateFields.length >= 0 : stryMutAct_9fa48("180") ? false : stryMutAct_9fa48("179") ? true : (stryCov_9fa48("179", "180", "181", "182"), updateFields.length > 0)) {
            if (stryMutAct_9fa48("183")) {
              {}
            } else {
              stryCov_9fa48("183");
              updateFields.push(`updated_at = NOW()`);
              values.push(agentId);
              await client.query(`
          UPDATE agent_profiles
          SET ${updateFields.join(", ")}
          WHERE id = $${paramIndex}
        `, values);
            }
          }

          // Update capabilities if provided
          if (stryMutAct_9fa48("188") ? false : stryMutAct_9fa48("187") ? true : (stryCov_9fa48("187", "188"), updates.capabilities)) {
            if (stryMutAct_9fa48("189")) {
              {}
            } else {
              stryCov_9fa48("189");
              // Delete existing capabilities
              await client.query("DELETE FROM agent_capabilities WHERE agent_id = $1", [agentId]);

              // Insert new capabilities - task types
              for (const taskType of updates.capabilities.taskTypes) {
                if (stryMutAct_9fa48("192")) {
                  {}
                } else {
                  stryCov_9fa48("192");
                  await client.query(`
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `, [agentId, `task_${taskType}`, 1.0, JSON.stringify(stryMutAct_9fa48("196") ? {} : (stryCov_9fa48("196"), {
                    type: "task",
                    value: taskType
                  }))]);
                }
              }

              // Insert new capabilities - languages
              for (const language of updates.capabilities.languages) {
                if (stryMutAct_9fa48("198")) {
                  {}
                } else {
                  stryCov_9fa48("198");
                  await client.query(`
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `, [agentId, `lang_${language}`, 1.0, JSON.stringify(stryMutAct_9fa48("202") ? {} : (stryCov_9fa48("202"), {
                    type: "language",
                    value: language
                  }))]);
                }
              }

              // Insert new capabilities - specializations
              for (const specialization of updates.capabilities.specializations) {
                if (stryMutAct_9fa48("204")) {
                  {}
                } else {
                  stryCov_9fa48("204");
                  await client.query(`
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `, [agentId, `spec_${specialization}`, 1.0, JSON.stringify(stryMutAct_9fa48("208") ? {} : (stryCov_9fa48("208"), {
                    type: "specialization",
                    value: specialization
                  }))]);
                }
              }
            }
          }
          await client.query("COMMIT");
          this.logger.info(`Agent updated: ${agentId}`);
        }
      } catch (error) {
        if (stryMutAct_9fa48("212")) {
          {}
        } else {
          stryCov_9fa48("212");
          await client.query("ROLLBACK");
          this.logger.error("Failed to update agent:", error);
          throw new Error(`Agent update failed: ${error instanceof Error ? error.message : String(error)}`);
        }
      } finally {
        if (stryMutAct_9fa48("216")) {
          {}
        } else {
          stryCov_9fa48("216");
          client.release();
        }
      }
    }
  }

  /**
   * Delete agent profile
   */
  async deleteAgent(agentId: string): Promise<void> {
    if (stryMutAct_9fa48("217")) {
      {}
    } else {
      stryCov_9fa48("217");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("218")) {
          {}
        } else {
          stryCov_9fa48("218");
          await client.query("BEGIN");

          // Delete in reverse dependency order
          await client.query("DELETE FROM agent_performance_history WHERE agent_id = $1", [agentId]);
          await client.query("DELETE FROM agent_capabilities WHERE agent_id = $1", [agentId]);
          await client.query("DELETE FROM agent_profiles WHERE id = $1", [agentId]);
          await client.query("COMMIT");
          this.logger.info(`Agent deleted: ${agentId}`);
        }
      } catch (error) {
        if (stryMutAct_9fa48("228")) {
          {}
        } else {
          stryCov_9fa48("228");
          await client.query("ROLLBACK");
          this.logger.error("Failed to delete agent:", error);
          throw new Error(`Agent deletion failed: ${error instanceof Error ? error.message : String(error)}`);
        }
      } finally {
        if (stryMutAct_9fa48("232")) {
          {}
        } else {
          stryCov_9fa48("232");
          client.release();
        }
      }
    }
  }

  /**
   * Query agents with advanced filtering
   */
  async queryAgents(query: AgentQuery): Promise<AgentQueryResult[]> {
    if (stryMutAct_9fa48("233")) {
      {}
    } else {
      stryCov_9fa48("233");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("234")) {
          {}
        } else {
          stryCov_9fa48("234");
          const whereConditions: string[] = [];
          const values: any[] = [];
          let paramIndex = 1;

          // Build WHERE conditions based on available AgentQuery fields
          if (stryMutAct_9fa48("239") ? query.maxUtilization === undefined : stryMutAct_9fa48("238") ? false : stryMutAct_9fa48("237") ? true : (stryCov_9fa48("237", "238", "239"), query.maxUtilization !== undefined)) {
            if (stryMutAct_9fa48("240")) {
              {}
            } else {
              stryCov_9fa48("240");
              whereConditions.push(`p.utilization_percent <= $${stryMutAct_9fa48("242") ? paramIndex-- : (stryCov_9fa48("242"), paramIndex++)}`);
              values.push(query.maxUtilization);
            }
          }
          if (stryMutAct_9fa48("245") ? query.minSuccessRate === undefined : stryMutAct_9fa48("244") ? false : stryMutAct_9fa48("243") ? true : (stryCov_9fa48("243", "244", "245"), query.minSuccessRate !== undefined)) {
            if (stryMutAct_9fa48("246")) {
              {}
            } else {
              stryCov_9fa48("246");
              whereConditions.push(`
          EXISTS (
            SELECT 1 FROM agent_performance_history ph
            WHERE ph.agent_id = p.id AND ph.success_rate >= $${paramIndex}
          )
        `);
              values.push(query.minSuccessRate);
            }
          }

          // Add capability filtering for languages
          if (stryMutAct_9fa48("250") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("249") ? false : stryMutAct_9fa48("248") ? true : (stryCov_9fa48("248", "249", "250"), query.languages && (stryMutAct_9fa48("253") ? query.languages.length <= 0 : stryMutAct_9fa48("252") ? query.languages.length >= 0 : stryMutAct_9fa48("251") ? true : (stryCov_9fa48("251", "252", "253"), query.languages.length > 0)))) {
            if (stryMutAct_9fa48("254")) {
              {}
            } else {
              stryCov_9fa48("254");
              const placeholders = query.languages.map(stryMutAct_9fa48("255") ? () => undefined : (stryCov_9fa48("255"), () => `$${stryMutAct_9fa48("257") ? paramIndex-- : (stryCov_9fa48("257"), paramIndex++)}`)).join(", ");
              whereConditions.push(`
          EXISTS (
            SELECT 1 FROM agent_capabilities c
            WHERE c.agent_id = p.id AND c.metadata->>'type' = 'language'
            AND c.metadata->>'value' = ANY(ARRAY[${placeholders}])
          )
        `);
              values.push(...query.languages);
            }
          }

          // Add capability filtering for specializations
          if (stryMutAct_9fa48("262") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("261") ? false : stryMutAct_9fa48("260") ? true : (stryCov_9fa48("260", "261", "262"), query.specializations && (stryMutAct_9fa48("265") ? query.specializations.length <= 0 : stryMutAct_9fa48("264") ? query.specializations.length >= 0 : stryMutAct_9fa48("263") ? true : (stryCov_9fa48("263", "264", "265"), query.specializations.length > 0)))) {
            if (stryMutAct_9fa48("266")) {
              {}
            } else {
              stryCov_9fa48("266");
              const placeholders = query.specializations.map(stryMutAct_9fa48("267") ? () => undefined : (stryCov_9fa48("267"), () => `$${stryMutAct_9fa48("269") ? paramIndex-- : (stryCov_9fa48("269"), paramIndex++)}`)).join(", ");
              whereConditions.push(`
          EXISTS (
            SELECT 1 FROM agent_capabilities c
            WHERE c.agent_id = p.id AND c.metadata->>'type' = 'specialization'
            AND c.metadata->>'value' = ANY(ARRAY[${placeholders}])
          )
        `);
              values.push(...query.specializations);
            }
          }
          const whereClause = (stryMutAct_9fa48("275") ? whereConditions.length <= 0 : stryMutAct_9fa48("274") ? whereConditions.length >= 0 : stryMutAct_9fa48("273") ? false : stryMutAct_9fa48("272") ? true : (stryCov_9fa48("272", "273", "274", "275"), whereConditions.length > 0)) ? `WHERE ${whereConditions.join(" AND ")}` : "";
          const result = await client.query(`
        SELECT p.* FROM agent_profiles p
        ${whereClause}
        ORDER BY p.last_active_at DESC
        LIMIT 50
      `, values);

          // Convert results to AgentQueryResult format
          const queryResults: AgentQueryResult[] = [];
          for (const row of result.rows) {
            if (stryMutAct_9fa48("281")) {
              {}
            } else {
              stryCov_9fa48("281");
              // Reconstruct the full agent profile
              const agent = await this.getAgent(row.id);
              if (stryMutAct_9fa48("283") ? false : stryMutAct_9fa48("282") ? true : (stryCov_9fa48("282", "283"), agent)) {
                if (stryMutAct_9fa48("284")) {
                  {}
                } else {
                  stryCov_9fa48("284");
                  queryResults.push(stryMutAct_9fa48("285") ? {} : (stryCov_9fa48("285"), {
                    agent,
                    matchScore: 0.8,
                    // Placeholder - would calculate based on query criteria
                    matchReason: "Matches query criteria"
                  }));
                }
              }
            }
          }
          this.logger.debug(`Found ${queryResults.length} agents matching query`);
          return queryResults;
        }
      } catch (error) {
        if (stryMutAct_9fa48("288")) {
          {}
        } else {
          stryCov_9fa48("288");
          this.logger.error("Failed to query agents:", error);
          throw new Error(`Agent query failed: ${error instanceof Error ? error.message : String(error)}`);
        }
      } finally {
        if (stryMutAct_9fa48("291")) {
          {}
        } else {
          stryCov_9fa48("291");
          client.release();
        }
      }
    }
  }

  /**
   * Record performance metrics for an agent
   */
  async recordPerformance(agentId: string, performance: PerformanceMetrics): Promise<void> {
    if (stryMutAct_9fa48("292")) {
      {}
    } else {
      stryCov_9fa48("292");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("293")) {
          {}
        } else {
          stryCov_9fa48("293");
          await client.query(`
        INSERT INTO agent_performance_history (
          agent_id, task_type, success_rate, average_latency,
          total_tasks, quality_score, confidence_score, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
      `, [agentId, stryMutAct_9fa48("298") ? performance.taskType && "general" : stryMutAct_9fa48("297") ? false : stryMutAct_9fa48("296") ? true : (stryCov_9fa48("296", "297", "298"), performance.taskType || "general"), performance.success ? 1.0 : 0.0, performance.latencyMs, 1,
          // Single task
          performance.qualityScore, 1.0,
          // Default confidence
          JSON.stringify(stryMutAct_9fa48("300") ? {} : (stryCov_9fa48("300"), {
            tokensUsed: performance.tokensUsed
          }))]);
          this.logger.debug(`Performance recorded for agent: ${agentId}`);
        }
      } catch (error) {
        if (stryMutAct_9fa48("302")) {
          {}
        } else {
          stryCov_9fa48("302");
          this.logger.error("Failed to record performance:", error);
          throw new Error(`Performance recording failed: ${error instanceof Error ? error.message : String(error)}`);
        }
      } finally {
        if (stryMutAct_9fa48("305")) {
          {}
        } else {
          stryCov_9fa48("305");
          client.release();
        }
      }
    }
  }

  /**
   * Get performance statistics for an agent
   */
  async getAgentStats(agentId: string): Promise<{
    totalTasks: number;
    averageSuccessRate: number;
    averageLatency: number;
    averageQuality: number;
    taskTypeBreakdown: Record<string, {
      count: number;
      successRate: number;
      avgLatency: number;
    }>;
  }> {
    if (stryMutAct_9fa48("306")) {
      {}
    } else {
      stryCov_9fa48("306");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("307")) {
          {}
        } else {
          stryCov_9fa48("307");
          const result = await client.query(`
        SELECT
          task_type,
          COUNT(*) as task_count,
          AVG(success_rate) as avg_success_rate,
          AVG(average_latency) as avg_latency,
          AVG(quality_score) as avg_quality
        FROM agent_performance_history
        WHERE agent_id = $1
        GROUP BY task_type
      `, [agentId]);
          let totalTasks = 0;
          let totalSuccessRate = 0;
          let totalLatency = 0;
          let totalQuality = 0;
          const taskTypeBreakdown: Record<string, {
            count: number;
            successRate: number;
            avgLatency: number;
          }> = {};
          for (const row of result.rows) {
            if (stryMutAct_9fa48("310")) {
              {}
            } else {
              stryCov_9fa48("310");
              const count = parseInt(row.task_count);
              stryMutAct_9fa48("311") ? totalTasks -= count : (stryCov_9fa48("311"), totalTasks += count);
              stryMutAct_9fa48("312") ? totalSuccessRate -= row.avg_success_rate * count : (stryCov_9fa48("312"), totalSuccessRate += stryMutAct_9fa48("313") ? row.avg_success_rate / count : (stryCov_9fa48("313"), row.avg_success_rate * count));
              stryMutAct_9fa48("314") ? totalLatency -= row.avg_latency * count : (stryCov_9fa48("314"), totalLatency += stryMutAct_9fa48("315") ? row.avg_latency / count : (stryCov_9fa48("315"), row.avg_latency * count));
              stryMutAct_9fa48("316") ? totalQuality -= (row.avg_quality || 0) * count : (stryCov_9fa48("316"), totalQuality += stryMutAct_9fa48("317") ? (row.avg_quality || 0) / count : (stryCov_9fa48("317"), (stryMutAct_9fa48("320") ? row.avg_quality && 0 : stryMutAct_9fa48("319") ? false : stryMutAct_9fa48("318") ? true : (stryCov_9fa48("318", "319", "320"), row.avg_quality || 0)) * count));
              taskTypeBreakdown[row.task_type] = stryMutAct_9fa48("321") ? {} : (stryCov_9fa48("321"), {
                count,
                successRate: row.avg_success_rate,
                avgLatency: row.avg_latency
              });
            }
          }
          return stryMutAct_9fa48("322") ? {} : (stryCov_9fa48("322"), {
            totalTasks,
            averageSuccessRate: (stryMutAct_9fa48("326") ? totalTasks <= 0 : stryMutAct_9fa48("325") ? totalTasks >= 0 : stryMutAct_9fa48("324") ? false : stryMutAct_9fa48("323") ? true : (stryCov_9fa48("323", "324", "325", "326"), totalTasks > 0)) ? stryMutAct_9fa48("327") ? totalSuccessRate * totalTasks : (stryCov_9fa48("327"), totalSuccessRate / totalTasks) : 0,
            averageLatency: (stryMutAct_9fa48("331") ? totalTasks <= 0 : stryMutAct_9fa48("330") ? totalTasks >= 0 : stryMutAct_9fa48("329") ? false : stryMutAct_9fa48("328") ? true : (stryCov_9fa48("328", "329", "330", "331"), totalTasks > 0)) ? stryMutAct_9fa48("332") ? totalLatency * totalTasks : (stryCov_9fa48("332"), totalLatency / totalTasks) : 0,
            averageQuality: (stryMutAct_9fa48("336") ? totalTasks <= 0 : stryMutAct_9fa48("335") ? totalTasks >= 0 : stryMutAct_9fa48("334") ? false : stryMutAct_9fa48("333") ? true : (stryCov_9fa48("333", "334", "335", "336"), totalTasks > 0)) ? stryMutAct_9fa48("337") ? totalQuality * totalTasks : (stryCov_9fa48("337"), totalQuality / totalTasks) : 0,
            taskTypeBreakdown
          });
        }
      } catch (error) {
        if (stryMutAct_9fa48("338")) {
          {}
        } else {
          stryCov_9fa48("338");
          this.logger.error("Failed to get agent stats:", error);
          throw new Error(`Agent stats retrieval failed: ${error instanceof Error ? error.message : String(error)}`);
        }
      } finally {
        if (stryMutAct_9fa48("341")) {
          {}
        } else {
          stryCov_9fa48("341");
          client.release();
        }
      }
    }
  }

  /**
   * Health check for database connectivity
   */
  async healthCheck(): Promise<{
    healthy: boolean;
    latency: number;
    error?: string;
  }> {
    if (stryMutAct_9fa48("342")) {
      {}
    } else {
      stryCov_9fa48("342");
      const startTime = Date.now();
      try {
        if (stryMutAct_9fa48("343")) {
          {}
        } else {
          stryCov_9fa48("343");
          const client = await this.pool.connect();
          try {
            if (stryMutAct_9fa48("344")) {
              {}
            } else {
              stryCov_9fa48("344");
              await client.query("SELECT 1");
              const latency = stryMutAct_9fa48("346") ? Date.now() + startTime : (stryCov_9fa48("346"), Date.now() - startTime);
              return stryMutAct_9fa48("347") ? {} : (stryCov_9fa48("347"), {
                healthy: stryMutAct_9fa48("348") ? false : (stryCov_9fa48("348"), true),
                latency
              });
            }
          } finally {
            if (stryMutAct_9fa48("349")) {
              {}
            } else {
              stryCov_9fa48("349");
              client.release();
            }
          }
        }
      } catch (error) {
        if (stryMutAct_9fa48("350")) {
          {}
        } else {
          stryCov_9fa48("350");
          const latency = stryMutAct_9fa48("351") ? Date.now() + startTime : (stryCov_9fa48("351"), Date.now() - startTime);
          return stryMutAct_9fa48("352") ? {} : (stryCov_9fa48("352"), {
            healthy: stryMutAct_9fa48("353") ? true : (stryCov_9fa48("353"), false),
            latency,
            error: error instanceof Error ? error.message : String(error)
          });
        }
      }
    }
  }

  /**
   * Verify database schema exists and is correct
   */
  private async verifySchema(): Promise<void> {
    if (stryMutAct_9fa48("354")) {
      {}
    } else {
      stryCov_9fa48("354");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("355")) {
          {}
        } else {
          stryCov_9fa48("355");
          // Check if required tables exist
          const tables = ["agent_profiles", "agent_capabilities", "agent_performance_history"];
          for (const table of tables) {
            if (stryMutAct_9fa48("360")) {
              {}
            } else {
              stryCov_9fa48("360");
              const result = await client.query(`
          SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = $1
          )
        `, [table]);
              if (stryMutAct_9fa48("365") ? false : stryMutAct_9fa48("364") ? true : stryMutAct_9fa48("363") ? result.rows[0].exists : (stryCov_9fa48("363", "364", "365"), !result.rows[0].exists)) {
                if (stryMutAct_9fa48("366")) {
                  {}
                } else {
                  stryCov_9fa48("366");
                  throw new Error(`Required table '${table}' does not exist`);
                }
              }
            }
          }
          this.logger.debug("All required tables verified");
        }
      } catch (error) {
        if (stryMutAct_9fa48("369")) {
          {}
        } else {
          stryCov_9fa48("369");
          throw new Error(`Schema verification failed: ${error instanceof Error ? error.message : String(error)}`);
        }
      } finally {
        if (stryMutAct_9fa48("371")) {
          {}
        } else {
          stryCov_9fa48("371");
          client.release();
        }
      }
    }
  }

  /**
   * Setup pool error handling
   */
  private setupPoolErrorHandling(): void {
    if (stryMutAct_9fa48("372")) {
      {}
    } else {
      stryCov_9fa48("372");
      this.pool.on("error", err => {
        if (stryMutAct_9fa48("374")) {
          {}
        } else {
          stryCov_9fa48("374");
          this.logger.error("Unexpected database pool error:", err);
        }
      });
      this.pool.on("connect", () => {
        if (stryMutAct_9fa48("377")) {
          {}
        } else {
          stryCov_9fa48("377");
          this.logger.debug("New database connection established");
        }
      });
      this.pool.on("remove", () => {
        if (stryMutAct_9fa48("380")) {
          {}
        } else {
          stryCov_9fa48("380");
          this.logger.debug("Database connection removed from pool");
        }
      });
    }
  }

  /**
   * Update agent performance metrics (legacy method for compatibility)
   */
  async updatePerformance(agentId: string, metrics: PerformanceMetrics): Promise<void> {
    if (stryMutAct_9fa48("382")) {
      {}
    } else {
      stryCov_9fa48("382");
      await this.recordPerformance(agentId, metrics);
    }
  }

  /**
   * Update agent load (active and queued tasks)
   */
  async updateLoad(agentId: string, activeTasksDelta: number, queuedTasksDelta: number): Promise<void> {
    if (stryMutAct_9fa48("383")) {
      {}
    } else {
      stryCov_9fa48("383");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("384")) {
          {}
        } else {
          stryCov_9fa48("384");
          // Update with atomic increment/decrement
          await client.query(`
        UPDATE agent_profiles
        SET
          active_tasks = GREATEST(0, active_tasks + $2),
          queued_tasks = GREATEST(0, queued_tasks + $3),
          utilization_percent = LEAST(100, GREATEST(0,
            CASE
              WHEN active_tasks + queued_tasks + $2 + $3 = 0 THEN 0
              ELSE ((active_tasks + $2) * 100.0) / (active_tasks + queued_tasks + $2 + $3)
            END
          )),
          updated_at = NOW()
        WHERE id = $1
      `, [agentId, activeTasksDelta, queuedTasksDelta]);
          this.logger.debug(`Updated load for agent: ${agentId} (+${activeTasksDelta} active, +${queuedTasksDelta} queued)`);
        }
      } catch (error) {
        if (stryMutAct_9fa48("388")) {
          {}
        } else {
          stryCov_9fa48("388");
          this.logger.error("Failed to update agent load:", error);
          throw new Error(`Load update failed: ${error instanceof Error ? error.message : String(error)}`);
        }
      } finally {
        if (stryMutAct_9fa48("391")) {
          {}
        } else {
          stryCov_9fa48("391");
          client.release();
        }
      }
    }
  }

  /**
   * Unregister an agent (delete from database)
   */
  async unregisterAgent(agentId: string): Promise<boolean> {
    if (stryMutAct_9fa48("392")) {
      {}
    } else {
      stryCov_9fa48("392");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("393")) {
          {}
        } else {
          stryCov_9fa48("393");
          await client.query("BEGIN");

          // Delete in reverse dependency order
          await client.query("DELETE FROM agent_performance_history WHERE agent_id = $1", [agentId]);
          await client.query("DELETE FROM agent_capabilities WHERE agent_id = $1", [agentId]);
          const result = await client.query("DELETE FROM agent_profiles WHERE id = $1", [agentId]);
          await client.query("COMMIT");
          const deleted = stryMutAct_9fa48("405") ? (result.rowCount ?? 0) <= 0 : stryMutAct_9fa48("404") ? (result.rowCount ?? 0) >= 0 : stryMutAct_9fa48("403") ? false : stryMutAct_9fa48("402") ? true : (stryCov_9fa48("402", "403", "404", "405"), (stryMutAct_9fa48("406") ? result.rowCount && 0 : (stryCov_9fa48("406"), result.rowCount ?? 0)) > 0);
          if (stryMutAct_9fa48("408") ? false : stryMutAct_9fa48("407") ? true : (stryCov_9fa48("407", "408"), deleted)) {
            if (stryMutAct_9fa48("409")) {
              {}
            } else {
              stryCov_9fa48("409");
              this.logger.info(`Agent unregistered: ${agentId}`);
            }
          }
          return deleted;
        }
      } catch (error) {
        if (stryMutAct_9fa48("411")) {
          {}
        } else {
          stryCov_9fa48("411");
          await client.query("ROLLBACK");
          this.logger.error("Failed to unregister agent:", error);
          throw new Error(`Agent unregistration failed: ${error instanceof Error ? error.message : String(error)}`);
        }
      } finally {
        if (stryMutAct_9fa48("415")) {
          {}
        } else {
          stryCov_9fa48("415");
          client.release();
        }
      }
    }
  }

  /**
   * Get registry statistics
   */
  async getStats(): Promise<{
    totalAgents: number;
    activeAgents: number;
    totalCapabilities: number;
    averagePerformance: number;
  }> {
    if (stryMutAct_9fa48("416")) {
      {}
    } else {
      stryCov_9fa48("416");
      const client = await this.pool.connect();
      try {
        if (stryMutAct_9fa48("417")) {
          {}
        } else {
          stryCov_9fa48("417");
          const result = await client.query(`
        SELECT
          (SELECT COUNT(*) FROM agent_profiles) as total_agents,
          (SELECT COUNT(*) FROM agent_profiles WHERE last_active_at > NOW() - INTERVAL '1 hour') as active_agents,
          (SELECT COUNT(*) FROM agent_capabilities) as total_capabilities,
          (SELECT COALESCE(AVG(success_rate), 0) FROM agent_performance_history) as avg_performance
      `);
          const stats = result.rows[0];
          return stryMutAct_9fa48("419") ? {} : (stryCov_9fa48("419"), {
            totalAgents: parseInt(stats.total_agents),
            activeAgents: parseInt(stats.active_agents),
            totalCapabilities: parseInt(stats.total_capabilities),
            averagePerformance: stryMutAct_9fa48("422") ? parseFloat(stats.avg_performance) && 0 : stryMutAct_9fa48("421") ? false : stryMutAct_9fa48("420") ? true : (stryCov_9fa48("420", "421", "422"), parseFloat(stats.avg_performance) || 0)
          });
        }
      } catch (error) {
        if (stryMutAct_9fa48("423")) {
          {}
        } else {
          stryCov_9fa48("423");
          this.logger.error("Failed to get registry stats:", error);
          throw new Error(`Stats retrieval failed: ${error instanceof Error ? error.message : String(error)}`);
        }
      } finally {
        if (stryMutAct_9fa48("426")) {
          {}
        } else {
          stryCov_9fa48("426");
          client.release();
        }
      }
    }
  }

  /**
   * Execute query with retry logic
   */
  private async executeWithRetry<T>(operation: (client: PoolClient) => Promise<T>,
  // eslint-disable-line no-unused-vars
  operationName: string): Promise<T> {
    if (stryMutAct_9fa48("427")) {
      {}
    } else {
      stryCov_9fa48("427");
      let lastError: Error = new Error("Unknown error");
      for (let attempt = 1; stryMutAct_9fa48("431") ? attempt > this.config.retryAttempts : stryMutAct_9fa48("430") ? attempt < this.config.retryAttempts : stryMutAct_9fa48("429") ? false : (stryCov_9fa48("429", "430", "431"), attempt <= this.config.retryAttempts); stryMutAct_9fa48("432") ? attempt-- : (stryCov_9fa48("432"), attempt++)) {
        if (stryMutAct_9fa48("433")) {
          {}
        } else {
          stryCov_9fa48("433");
          const client = await this.pool.connect();
          try {
            if (stryMutAct_9fa48("434")) {
              {}
            } else {
              stryCov_9fa48("434");
              const result = await operation(client);
              return result;
            }
          } catch (error) {
            if (stryMutAct_9fa48("435")) {
              {}
            } else {
              stryCov_9fa48("435");
              lastError = error instanceof Error ? error : new Error(String(error));
              this.logger.warn(`${operationName} failed on attempt ${attempt}:`, lastError.message);
              if (stryMutAct_9fa48("440") ? attempt >= this.config.retryAttempts : stryMutAct_9fa48("439") ? attempt <= this.config.retryAttempts : stryMutAct_9fa48("438") ? false : stryMutAct_9fa48("437") ? true : (stryCov_9fa48("437", "438", "439", "440"), attempt < this.config.retryAttempts)) {
                if (stryMutAct_9fa48("441")) {
                  {}
                } else {
                  stryCov_9fa48("441");
                  await new Promise(stryMutAct_9fa48("442") ? () => undefined : (stryCov_9fa48("442"), resolve => setTimeout(resolve, this.config.retryDelayMs)));
                }
              }
            }
          } finally {
            if (stryMutAct_9fa48("443")) {
              {}
            } else {
              stryCov_9fa48("443");
              client.release();
            }
          }
        }
      }
      throw new Error(`${operationName} failed after ${this.config.retryAttempts} attempts: ${lastError.message}`);
    }
  }
}