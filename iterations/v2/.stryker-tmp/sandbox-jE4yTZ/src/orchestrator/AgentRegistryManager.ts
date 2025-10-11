/**
 * Agent Registry Manager
 *
 * @author @darianrosebrook
 * @module orchestrator/AgentRegistryManager
 *
 * Central registry for managing agent profiles, capabilities, and performance history.
 * Implements ARBITER-001 specification with capability tracking and atomic updates.
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
import { AgentRegistryDatabaseConfig, AgentRegistryDbClient } from "../database/AgentRegistryDbClient.js";
import { AgentRegistrySecurity, SecurityContext } from "../security/AgentRegistrySecurity.js";
import type { AgentId, AgentProfile, AgentQuery, AgentQueryResult, AgentRegistryConfig, PerformanceMetrics, RegistryStats } from "../types/agent-registry";
import { RegistryError, RegistryErrorType } from "../types/agent-registry";
import { AgentProfileHelper } from "./AgentProfile";

/**
 * Default configuration for the agent registry.
 */
const DEFAULT_CONFIG: AgentRegistryConfig = stryMutAct_9fa48("445") ? {} : (stryCov_9fa48("445"), {
  maxAgents: 1000,
  staleAgentThresholdMs: stryMutAct_9fa48("446") ? 24 * 60 * 60 / 1000 : (stryCov_9fa48("446"), (stryMutAct_9fa48("447") ? 24 * 60 / 60 : (stryCov_9fa48("447"), (stryMutAct_9fa48("448") ? 24 / 60 : (stryCov_9fa48("448"), 24 * 60)) * 60)) * 1000),
  // 24 hours
  enableAutoCleanup: stryMutAct_9fa48("449") ? false : (stryCov_9fa48("449"), true),
  cleanupIntervalMs: stryMutAct_9fa48("450") ? 60 * 60 / 1000 : (stryCov_9fa48("450"), (stryMutAct_9fa48("451") ? 60 / 60 : (stryCov_9fa48("451"), 60 * 60)) * 1000),
  // 1 hour
  enablePersistence: stryMutAct_9fa48("452") ? true : (stryCov_9fa48("452"), false),
  // Disabled by default for backward compatibility
  enableSecurity: stryMutAct_9fa48("453") ? false : (stryCov_9fa48("453"), true) // Security enabled by default
});

/**
 * Agent Registry Manager
 *
 * Maintains the catalog of available agents with their capabilities,
 * performance history, and current load status.
 *
 * @remarks
 * Thread-safe: Uses Map for O(1) lookups with atomic updates.
 * Invariants:
 * - Agent profiles are immutable except for performance metrics
 * - Performance history updates are atomic and isolated per agent
 * - Registry queries never block agent registration operations
 * - All capability changes are versioned and auditable
 */
export class AgentRegistryManager {
  private readonly agents: Map<AgentId, AgentProfile>;
  private readonly config: AgentRegistryConfig;
  private cleanupTimer?: ReturnType<typeof setInterval>;
  private readonly maxConcurrentTasksPerAgent: number = 10;
  private dbClient?: AgentRegistryDbClient;
  private securityManager?: AgentRegistrySecurity;
  constructor(config: Partial<AgentRegistryConfig> = {}) {
    if (stryMutAct_9fa48("454")) {
      {}
    } else {
      stryCov_9fa48("454");
      this.agents = new Map();
      this.config = stryMutAct_9fa48("455") ? {} : (stryCov_9fa48("455"), {
        ...DEFAULT_CONFIG,
        ...config
      });

      // Initialize database client if persistence is enabled
      if (stryMutAct_9fa48("458") ? this.config.enablePersistence || this.config.database : stryMutAct_9fa48("457") ? false : stryMutAct_9fa48("456") ? true : (stryCov_9fa48("456", "457", "458"), this.config.enablePersistence && this.config.database)) {
        if (stryMutAct_9fa48("459")) {
          {}
        } else {
          stryCov_9fa48("459");
          const dbConfig: AgentRegistryDatabaseConfig = stryMutAct_9fa48("460") ? {} : (stryCov_9fa48("460"), {
            host: this.config.database.host,
            port: this.config.database.port,
            database: this.config.database.database,
            username: this.config.database.username,
            password: this.config.database.password,
            ssl: this.config.database.ssl,
            maxConnections: 10,
            connectionTimeoutMs: 10000,
            queryTimeoutMs: 30000,
            retryAttempts: 3,
            retryDelayMs: 1000
          });
          this.dbClient = new AgentRegistryDbClient(dbConfig);
        }
      }

      // Initialize security manager if security is enabled
      if (stryMutAct_9fa48("462") ? false : stryMutAct_9fa48("461") ? true : (stryCov_9fa48("461", "462"), this.config.enableSecurity)) {
        if (stryMutAct_9fa48("463")) {
          {}
        } else {
          stryCov_9fa48("463");
          this.securityManager = new AgentRegistrySecurity(this.config.security);
        }
      }
      if (stryMutAct_9fa48("465") ? false : stryMutAct_9fa48("464") ? true : (stryCov_9fa48("464", "465"), this.config.enableAutoCleanup)) {
        if (stryMutAct_9fa48("466")) {
          {}
        } else {
          stryCov_9fa48("466");
          this.startAutoCleanup();
        }
      }
    }
  }

  /**
   * Initialize the registry manager.
   *
   * Must be called before using the registry if persistence is enabled.
   */
  async initialize(): Promise<void> {
    if (stryMutAct_9fa48("467")) {
      {}
    } else {
      stryCov_9fa48("467");
      if (stryMutAct_9fa48("470") ? this.config.enablePersistence || this.dbClient : stryMutAct_9fa48("469") ? false : stryMutAct_9fa48("468") ? true : (stryCov_9fa48("468", "469", "470"), this.config.enablePersistence && this.dbClient)) {
        if (stryMutAct_9fa48("471")) {
          {}
        } else {
          stryCov_9fa48("471");
          await this.dbClient.initialize();

          // Load existing agents from database
          await this.loadAgentsFromDatabase();
        }
      }
    }
  }

  /**
   * Load existing agents from database into memory cache.
   */
  private async loadAgentsFromDatabase(): Promise<void> {
    if (stryMutAct_9fa48("472")) {
      {}
    } else {
      stryCov_9fa48("472");
      if (stryMutAct_9fa48("475") ? false : stryMutAct_9fa48("474") ? true : stryMutAct_9fa48("473") ? this.dbClient : (stryCov_9fa48("473", "474", "475"), !this.dbClient)) return;
      try {
        if (stryMutAct_9fa48("476")) {
          {}
        } else {
          stryCov_9fa48("476");
          // Query all agents (simplified query for loading)
          const result = await this.dbClient.queryAgents(stryMutAct_9fa48("477") ? {} : (stryCov_9fa48("477"), {
            taskType: "code-editing" // Required field
          }));

          // Load agents into memory cache
          for (const queryResult of result) {
            if (stryMutAct_9fa48("479")) {
              {}
            } else {
              stryCov_9fa48("479");
              this.agents.set(queryResult.agent.id, queryResult.agent);
            }
          }

          // Log successful loading
          console.log(`Loaded ${result.length} agents from database`);
        }
      } catch (error) {
        if (stryMutAct_9fa48("481")) {
          {}
        } else {
          stryCov_9fa48("481");
          throw new RegistryError(RegistryErrorType.DATABASE_ERROR, `Failed to load agents from database: ${error instanceof Error ? error.message : String(error)}`);
        }
      }
    }
  }

  /**
   * Register a new agent in the registry.
   *
   * @param agent - Agent to register (partial, will be filled with defaults)
   * @returns Complete agent profile with generated fields
   * @throws RegistryError if agent already exists or registry is full
   *
   * @remarks
   * Acceptance Criterion A1: Agent profile created with capability tracking initialized
   */
  async registerAgent(agent: Partial<AgentProfile>, securityContext?: SecurityContext): Promise<AgentProfile> {
    if (stryMutAct_9fa48("483")) {
      {}
    } else {
      stryCov_9fa48("483");
      // Security check: authenticate and authorize
      if (stryMutAct_9fa48("486") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("485") ? false : stryMutAct_9fa48("484") ? true : (stryCov_9fa48("484", "485", "486"), this.config.enableSecurity && this.securityManager)) {
        if (stryMutAct_9fa48("487")) {
          {}
        } else {
          stryCov_9fa48("487");
          if (stryMutAct_9fa48("490") ? false : stryMutAct_9fa48("489") ? true : stryMutAct_9fa48("488") ? securityContext : (stryCov_9fa48("488", "489", "490"), !securityContext)) {
            if (stryMutAct_9fa48("491")) {
              {}
            } else {
              stryCov_9fa48("491");
              throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, "Security context required when security is enabled");
            }
          }
          const authorized = await this.securityManager.authorize(securityContext, "create" as any, "agent", stryMutAct_9fa48("496") ? agent.id && "unknown" : stryMutAct_9fa48("495") ? false : stryMutAct_9fa48("494") ? true : (stryCov_9fa48("494", "495", "496"), agent.id || "unknown"), agent);
          if (stryMutAct_9fa48("500") ? false : stryMutAct_9fa48("499") ? true : stryMutAct_9fa48("498") ? authorized : (stryCov_9fa48("498", "499", "500"), !authorized)) {
            if (stryMutAct_9fa48("501")) {
              {}
            } else {
              stryCov_9fa48("501");
              await this.securityManager.logAuditEvent(stryMutAct_9fa48("502") ? {} : (stryCov_9fa48("502"), {
                id: this.generateId(),
                timestamp: new Date(),
                eventType: "agent_registration" as any,
                actor: stryMutAct_9fa48("503") ? {} : (stryCov_9fa48("503"), {
                  tenantId: securityContext.tenantId,
                  userId: securityContext.userId,
                  sessionId: securityContext.sessionId
                }),
                resource: stryMutAct_9fa48("504") ? {} : (stryCov_9fa48("504"), {
                  type: "agent",
                  id: stryMutAct_9fa48("508") ? agent.id && "unknown" : stryMutAct_9fa48("507") ? false : stryMutAct_9fa48("506") ? true : (stryCov_9fa48("506", "507", "508"), agent.id || "unknown")
                }),
                action: "create" as any,
                details: stryMutAct_9fa48("510") ? {} : (stryCov_9fa48("510"), {
                  agentData: agent
                }),
                result: "failure",
                errorMessage: "Authorization failed",
                ipAddress: securityContext.ipAddress,
                userAgent: securityContext.userAgent
              }));
              throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, "Not authorized to register agents");
            }
          }
        }
      }

      // Validate input data with security layer
      if (stryMutAct_9fa48("516") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("515") ? false : stryMutAct_9fa48("514") ? true : (stryCov_9fa48("514", "515", "516"), this.config.enableSecurity && this.securityManager)) {
        if (stryMutAct_9fa48("517")) {
          {}
        } else {
          stryCov_9fa48("517");
          const validation = this.securityManager.validateAgentData(agent);
          if (stryMutAct_9fa48("520") ? false : stryMutAct_9fa48("519") ? true : stryMutAct_9fa48("518") ? validation.valid : (stryCov_9fa48("518", "519", "520"), !validation.valid)) {
            if (stryMutAct_9fa48("521")) {
              {}
            } else {
              stryCov_9fa48("521");
              throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, `Validation failed: ${validation.errors.join(", ")}`);
            }
          }
          // Use sanitized data if available
          if (stryMutAct_9fa48("525") ? false : stryMutAct_9fa48("524") ? true : (stryCov_9fa48("524", "525"), validation.sanitized)) {
            if (stryMutAct_9fa48("526")) {
              {}
            } else {
              stryCov_9fa48("526");
              agent = validation.sanitized;
            }
          }
        }
      } else {
        if (stryMutAct_9fa48("527")) {
          {}
        } else {
          stryCov_9fa48("527");
          // Fallback to basic validation
          AgentProfileHelper.validateProfile(agent);
        }
      }
      if (stryMutAct_9fa48("530") ? false : stryMutAct_9fa48("529") ? true : stryMutAct_9fa48("528") ? agent.id : (stryCov_9fa48("528", "529", "530"), !agent.id)) {
        if (stryMutAct_9fa48("531")) {
          {}
        } else {
          stryCov_9fa48("531");
          throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, "Agent ID is required");
        }
      }

      // Check if agent already exists
      if (stryMutAct_9fa48("534") ? false : stryMutAct_9fa48("533") ? true : (stryCov_9fa48("533", "534"), this.agents.has(agent.id))) {
        if (stryMutAct_9fa48("535")) {
          {}
        } else {
          stryCov_9fa48("535");
          throw new RegistryError(RegistryErrorType.AGENT_ALREADY_EXISTS, `Agent with ID ${agent.id} already exists`, stryMutAct_9fa48("537") ? {} : (stryCov_9fa48("537"), {
            agentId: agent.id
          }));
        }
      }

      // Check registry capacity
      if (stryMutAct_9fa48("541") ? this.agents.size < this.config.maxAgents : stryMutAct_9fa48("540") ? this.agents.size > this.config.maxAgents : stryMutAct_9fa48("539") ? false : stryMutAct_9fa48("538") ? true : (stryCov_9fa48("538", "539", "540", "541"), this.agents.size >= this.config.maxAgents)) {
        if (stryMutAct_9fa48("542")) {
          {}
        } else {
          stryCov_9fa48("542");
          throw new RegistryError(RegistryErrorType.REGISTRY_FULL, `Registry is full (max: ${this.config.maxAgents} agents)`, stryMutAct_9fa48("544") ? {} : (stryCov_9fa48("544"), {
            maxAgents: this.config.maxAgents,
            currentSize: this.agents.size
          }));
        }
      }

      // Create complete profile with defaults
      const now = new Date().toISOString();
      const profile: AgentProfile = stryMutAct_9fa48("545") ? {} : (stryCov_9fa48("545"), {
        id: agent.id,
        name: agent.name!,
        modelFamily: agent.modelFamily!,
        capabilities: agent.capabilities!,
        performanceHistory: stryMutAct_9fa48("546") ? agent.performanceHistory && AgentProfileHelper.createInitialPerformanceHistory() : (stryCov_9fa48("546"), agent.performanceHistory ?? AgentProfileHelper.createInitialPerformanceHistory()),
        currentLoad: stryMutAct_9fa48("547") ? agent.currentLoad && AgentProfileHelper.createInitialLoad() : (stryCov_9fa48("547"), agent.currentLoad ?? AgentProfileHelper.createInitialLoad()),
        registeredAt: now,
        lastActiveAt: now
      });

      // Initialize capability tracking
      await this.initializeCapabilityTracking(profile);

      // Store in registry
      this.agents.set(profile.id, profile);

      // Persist to database if enabled
      if (stryMutAct_9fa48("549") ? false : stryMutAct_9fa48("548") ? true : (stryCov_9fa48("548", "549"), this.dbClient)) {
        if (stryMutAct_9fa48("550")) {
          {}
        } else {
          stryCov_9fa48("550");
          try {
            if (stryMutAct_9fa48("551")) {
              {}
            } else {
              stryCov_9fa48("551");
              await this.dbClient.registerAgent(profile);
            }
          } catch (error) {
            if (stryMutAct_9fa48("552")) {
              {}
            } else {
              stryCov_9fa48("552");
              // Rollback in-memory storage on database failure
              this.agents.delete(profile.id);
              throw new RegistryError(RegistryErrorType.DATABASE_ERROR, `Failed to persist agent to database: ${error instanceof Error ? error.message : String(error)}`, stryMutAct_9fa48("554") ? {} : (stryCov_9fa48("554"), {
                agentId: profile.id
              }));
            }
          }
        }
      }

      // Audit log successful registration
      if (stryMutAct_9fa48("557") ? this.config.enableSecurity && this.securityManager || securityContext : stryMutAct_9fa48("556") ? false : stryMutAct_9fa48("555") ? true : (stryCov_9fa48("555", "556", "557"), (stryMutAct_9fa48("559") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("558") ? true : (stryCov_9fa48("558", "559"), this.config.enableSecurity && this.securityManager)) && securityContext)) {
        if (stryMutAct_9fa48("560")) {
          {}
        } else {
          stryCov_9fa48("560");
          await this.securityManager.logAuditEvent(stryMutAct_9fa48("561") ? {} : (stryCov_9fa48("561"), {
            id: this.generateId(),
            timestamp: new Date(),
            eventType: "agent_registration" as any,
            actor: stryMutAct_9fa48("562") ? {} : (stryCov_9fa48("562"), {
              tenantId: securityContext.tenantId,
              userId: securityContext.userId,
              sessionId: securityContext.sessionId
            }),
            resource: stryMutAct_9fa48("563") ? {} : (stryCov_9fa48("563"), {
              type: "agent",
              id: profile.id
            }),
            action: "create" as any,
            details: stryMutAct_9fa48("565") ? {} : (stryCov_9fa48("565"), {
              agentProfile: profile
            }),
            result: "success",
            ipAddress: securityContext.ipAddress,
            userAgent: securityContext.userAgent
          }));
        }
      }
      return AgentProfileHelper.cloneProfile(profile);
    }
  }

  /**
   * Get agent profile by ID.
   *
   * @param agentId - ID of the agent to retrieve
   * @returns Agent profile
   * @throws RegistryError if agent not found
   */
  async getProfile(agentId: AgentId, securityContext?: SecurityContext): Promise<AgentProfile> {
    if (stryMutAct_9fa48("567")) {
      {}
    } else {
      stryCov_9fa48("567");
      // Security check: authenticate and authorize
      if (stryMutAct_9fa48("570") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("569") ? false : stryMutAct_9fa48("568") ? true : (stryCov_9fa48("568", "569", "570"), this.config.enableSecurity && this.securityManager)) {
        if (stryMutAct_9fa48("571")) {
          {}
        } else {
          stryCov_9fa48("571");
          if (stryMutAct_9fa48("574") ? false : stryMutAct_9fa48("573") ? true : stryMutAct_9fa48("572") ? securityContext : (stryCov_9fa48("572", "573", "574"), !securityContext)) {
            if (stryMutAct_9fa48("575")) {
              {}
            } else {
              stryCov_9fa48("575");
              throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, "Security context required when security is enabled");
            }
          }
          const authorized = await this.securityManager.authorize(securityContext, "read" as any, "agent", agentId);
          if (stryMutAct_9fa48("580") ? false : stryMutAct_9fa48("579") ? true : stryMutAct_9fa48("578") ? authorized : (stryCov_9fa48("578", "579", "580"), !authorized)) {
            if (stryMutAct_9fa48("581")) {
              {}
            } else {
              stryCov_9fa48("581");
              await this.securityManager.logAuditEvent(stryMutAct_9fa48("582") ? {} : (stryCov_9fa48("582"), {
                id: this.generateId(),
                timestamp: new Date(),
                eventType: "agent_query" as any,
                actor: stryMutAct_9fa48("583") ? {} : (stryCov_9fa48("583"), {
                  tenantId: securityContext.tenantId,
                  userId: securityContext.userId,
                  sessionId: securityContext.sessionId
                }),
                resource: stryMutAct_9fa48("584") ? {} : (stryCov_9fa48("584"), {
                  type: "agent",
                  id: agentId
                }),
                action: "read" as any,
                details: stryMutAct_9fa48("586") ? {} : (stryCov_9fa48("586"), {
                  queryType: "getProfile"
                }),
                result: "failure",
                errorMessage: "Authorization failed",
                ipAddress: securityContext.ipAddress,
                userAgent: securityContext.userAgent
              }));
              throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, "Not authorized to access this agent");
            }
          }
        }
      }
      let profile = this.agents.get(agentId);

      // If not in memory cache, try to load from database
      if (stryMutAct_9fa48("593") ? !profile || this.dbClient : stryMutAct_9fa48("592") ? false : stryMutAct_9fa48("591") ? true : (stryCov_9fa48("591", "592", "593"), (stryMutAct_9fa48("594") ? profile : (stryCov_9fa48("594"), !profile)) && this.dbClient)) {
        if (stryMutAct_9fa48("595")) {
          {}
        } else {
          stryCov_9fa48("595");
          try {
            if (stryMutAct_9fa48("596")) {
              {}
            } else {
              stryCov_9fa48("596");
              const dbProfile = await this.dbClient.getAgent(agentId);
              if (stryMutAct_9fa48("598") ? false : stryMutAct_9fa48("597") ? true : (stryCov_9fa48("597", "598"), dbProfile)) {
                if (stryMutAct_9fa48("599")) {
                  {}
                } else {
                  stryCov_9fa48("599");
                  // Cache in memory for future requests
                  this.agents.set(agentId, dbProfile);
                  profile = dbProfile;
                }
              }
            }
          } catch (error) {
            if (stryMutAct_9fa48("600")) {
              {}
            } else {
              stryCov_9fa48("600");
              throw new RegistryError(RegistryErrorType.DATABASE_ERROR, `Failed to retrieve agent from database: ${error instanceof Error ? error.message : String(error)}`, stryMutAct_9fa48("602") ? {} : (stryCov_9fa48("602"), {
                agentId
              }));
            }
          }
        }
      }
      if (stryMutAct_9fa48("605") ? false : stryMutAct_9fa48("604") ? true : stryMutAct_9fa48("603") ? profile : (stryCov_9fa48("603", "604", "605"), !profile)) {
        if (stryMutAct_9fa48("606")) {
          {}
        } else {
          stryCov_9fa48("606");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, `Agent with ID ${agentId} not found`, stryMutAct_9fa48("608") ? {} : (stryCov_9fa48("608"), {
            agentId
          }));
        }
      }

      // Audit log successful profile access
      if (stryMutAct_9fa48("611") ? this.config.enableSecurity && this.securityManager || securityContext : stryMutAct_9fa48("610") ? false : stryMutAct_9fa48("609") ? true : (stryCov_9fa48("609", "610", "611"), (stryMutAct_9fa48("613") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("612") ? true : (stryCov_9fa48("612", "613"), this.config.enableSecurity && this.securityManager)) && securityContext)) {
        if (stryMutAct_9fa48("614")) {
          {}
        } else {
          stryCov_9fa48("614");
          await this.securityManager.logAuditEvent(stryMutAct_9fa48("615") ? {} : (stryCov_9fa48("615"), {
            id: this.generateId(),
            timestamp: new Date(),
            eventType: "agent_query" as any,
            actor: stryMutAct_9fa48("616") ? {} : (stryCov_9fa48("616"), {
              tenantId: securityContext.tenantId,
              userId: securityContext.userId,
              sessionId: securityContext.sessionId
            }),
            resource: stryMutAct_9fa48("617") ? {} : (stryCov_9fa48("617"), {
              type: "agent",
              id: agentId
            }),
            action: "read" as any,
            details: stryMutAct_9fa48("619") ? {} : (stryCov_9fa48("619"), {
              queryType: "getProfile",
              found: stryMutAct_9fa48("621") ? false : (stryCov_9fa48("621"), true)
            }),
            result: "success",
            ipAddress: securityContext.ipAddress,
            userAgent: securityContext.userAgent
          }));
        }
      }
      return AgentProfileHelper.cloneProfile(profile);
    }
  }

  /**
   * Query agents by capability and return sorted by performance.
   *
   * @param query - Query parameters with required capabilities
   * @returns Array of matching agents sorted by success rate (highest first)
   *
   * @remarks
   * Acceptance Criterion A2: Agents matching criteria returned sorted by performance history success rate
   * Performance Target: <50ms P95 latency
   */
  async getAgentsByCapability(query: AgentQuery): Promise<AgentQueryResult[]> {
    if (stryMutAct_9fa48("623")) {
      {}
    } else {
      stryCov_9fa48("623");
      const results: AgentQueryResult[] = [];
      for (const profile of Array.from(this.agents.values())) {
        if (stryMutAct_9fa48("625")) {
          {}
        } else {
          stryCov_9fa48("625");
          // Check task type match
          if (stryMutAct_9fa48("628") ? false : stryMutAct_9fa48("627") ? true : stryMutAct_9fa48("626") ? profile.capabilities.taskTypes.includes(query.taskType) : (stryCov_9fa48("626", "627", "628"), !profile.capabilities.taskTypes.includes(query.taskType))) {
            if (stryMutAct_9fa48("629")) {
              {}
            } else {
              stryCov_9fa48("629");
              continue;
            }
          }

          // Check language requirements if specified
          if (stryMutAct_9fa48("632") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("631") ? false : stryMutAct_9fa48("630") ? true : (stryCov_9fa48("630", "631", "632"), query.languages && (stryMutAct_9fa48("635") ? query.languages.length <= 0 : stryMutAct_9fa48("634") ? query.languages.length >= 0 : stryMutAct_9fa48("633") ? true : (stryCov_9fa48("633", "634", "635"), query.languages.length > 0)))) {
            if (stryMutAct_9fa48("636")) {
              {}
            } else {
              stryCov_9fa48("636");
              const hasAllLanguages = stryMutAct_9fa48("637") ? query.languages.some(lang => profile.capabilities.languages.includes(lang)) : (stryCov_9fa48("637"), query.languages.every(stryMutAct_9fa48("638") ? () => undefined : (stryCov_9fa48("638"), lang => profile.capabilities.languages.includes(lang))));
              if (stryMutAct_9fa48("641") ? false : stryMutAct_9fa48("640") ? true : stryMutAct_9fa48("639") ? hasAllLanguages : (stryCov_9fa48("639", "640", "641"), !hasAllLanguages)) {
                if (stryMutAct_9fa48("642")) {
                  {}
                } else {
                  stryCov_9fa48("642");
                  continue;
                }
              }
            }
          }

          // Check specialization requirements if specified
          if (stryMutAct_9fa48("645") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("644") ? false : stryMutAct_9fa48("643") ? true : (stryCov_9fa48("643", "644", "645"), query.specializations && (stryMutAct_9fa48("648") ? query.specializations.length <= 0 : stryMutAct_9fa48("647") ? query.specializations.length >= 0 : stryMutAct_9fa48("646") ? true : (stryCov_9fa48("646", "647", "648"), query.specializations.length > 0)))) {
            if (stryMutAct_9fa48("649")) {
              {}
            } else {
              stryCov_9fa48("649");
              const hasAllSpecializations = stryMutAct_9fa48("650") ? query.specializations.some(spec => profile.capabilities.specializations.includes(spec)) : (stryCov_9fa48("650"), query.specializations.every(stryMutAct_9fa48("651") ? () => undefined : (stryCov_9fa48("651"), spec => profile.capabilities.specializations.includes(spec))));
              if (stryMutAct_9fa48("654") ? false : stryMutAct_9fa48("653") ? true : stryMutAct_9fa48("652") ? hasAllSpecializations : (stryCov_9fa48("652", "653", "654"), !hasAllSpecializations)) {
                if (stryMutAct_9fa48("655")) {
                  {}
                } else {
                  stryCov_9fa48("655");
                  continue;
                }
              }
            }
          }

          // Check utilization threshold if specified
          if (stryMutAct_9fa48("658") ? query.maxUtilization !== undefined || profile.currentLoad.utilizationPercent > query.maxUtilization : stryMutAct_9fa48("657") ? false : stryMutAct_9fa48("656") ? true : (stryCov_9fa48("656", "657", "658"), (stryMutAct_9fa48("660") ? query.maxUtilization === undefined : stryMutAct_9fa48("659") ? true : (stryCov_9fa48("659", "660"), query.maxUtilization !== undefined)) && (stryMutAct_9fa48("663") ? profile.currentLoad.utilizationPercent <= query.maxUtilization : stryMutAct_9fa48("662") ? profile.currentLoad.utilizationPercent >= query.maxUtilization : stryMutAct_9fa48("661") ? true : (stryCov_9fa48("661", "662", "663"), profile.currentLoad.utilizationPercent > query.maxUtilization)))) {
            if (stryMutAct_9fa48("664")) {
              {}
            } else {
              stryCov_9fa48("664");
              continue;
            }
          }

          // Check minimum success rate if specified
          if (stryMutAct_9fa48("667") ? query.minSuccessRate !== undefined || profile.performanceHistory.successRate < query.minSuccessRate : stryMutAct_9fa48("666") ? false : stryMutAct_9fa48("665") ? true : (stryCov_9fa48("665", "666", "667"), (stryMutAct_9fa48("669") ? query.minSuccessRate === undefined : stryMutAct_9fa48("668") ? true : (stryCov_9fa48("668", "669"), query.minSuccessRate !== undefined)) && (stryMutAct_9fa48("672") ? profile.performanceHistory.successRate >= query.minSuccessRate : stryMutAct_9fa48("671") ? profile.performanceHistory.successRate <= query.minSuccessRate : stryMutAct_9fa48("670") ? true : (stryCov_9fa48("670", "671", "672"), profile.performanceHistory.successRate < query.minSuccessRate)))) {
            if (stryMutAct_9fa48("673")) {
              {}
            } else {
              stryCov_9fa48("673");
              continue;
            }
          }

          // Calculate match score
          const matchScore = this.calculateMatchScore(profile, query);
          const matchReason = this.explainMatchScore(profile, query, matchScore);
          results.push(stryMutAct_9fa48("674") ? {} : (stryCov_9fa48("674"), {
            agent: AgentProfileHelper.cloneProfile(profile),
            matchScore,
            matchReason
          }));
        }
      }

      // Sort by success rate (highest first), then by match score
      return stryMutAct_9fa48("675") ? results : (stryCov_9fa48("675"), results.sort((a, b) => {
        if (stryMutAct_9fa48("676")) {
          {}
        } else {
          stryCov_9fa48("676");
          const successDiff = stryMutAct_9fa48("677") ? b.agent.performanceHistory.successRate + a.agent.performanceHistory.successRate : (stryCov_9fa48("677"), b.agent.performanceHistory.successRate - a.agent.performanceHistory.successRate);
          if (stryMutAct_9fa48("681") ? Math.abs(successDiff) <= 0.01 : stryMutAct_9fa48("680") ? Math.abs(successDiff) >= 0.01 : stryMutAct_9fa48("679") ? false : stryMutAct_9fa48("678") ? true : (stryCov_9fa48("678", "679", "680", "681"), Math.abs(successDiff) > 0.01)) {
            if (stryMutAct_9fa48("682")) {
              {}
            } else {
              stryCov_9fa48("682");
              return successDiff;
            }
          }
          return stryMutAct_9fa48("683") ? b.matchScore + a.matchScore : (stryCov_9fa48("683"), b.matchScore - a.matchScore);
        }
      }));
    }
  }

  /**
   * Update performance metrics for an agent after task completion.
   *
   * @param agentId - ID of the agent to update
   * @param metrics - Performance metrics from the completed task
   * @returns Updated agent profile
   * @throws RegistryError if agent not found or update fails
   *
   * @remarks
   * Acceptance Criterion A3: Agent's running average performance history computed and persisted
   * Performance Target: <30ms P95 latency
   * Invariant: Performance history updates are atomic and isolated per agent
   */
  async updatePerformance(agentId: AgentId, metrics: PerformanceMetrics): Promise<AgentProfile> {
    if (stryMutAct_9fa48("684")) {
      {}
    } else {
      stryCov_9fa48("684");
      const profile = this.agents.get(agentId);
      if (stryMutAct_9fa48("687") ? false : stryMutAct_9fa48("686") ? true : stryMutAct_9fa48("685") ? profile : (stryCov_9fa48("685", "686", "687"), !profile)) {
        if (stryMutAct_9fa48("688")) {
          {}
        } else {
          stryCov_9fa48("688");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, `Agent with ID ${agentId} not found`, stryMutAct_9fa48("690") ? {} : (stryCov_9fa48("690"), {
            agentId
          }));
        }
      }
      try {
        if (stryMutAct_9fa48("691")) {
          {}
        } else {
          stryCov_9fa48("691");
          // Compute new running average (atomic operation)
          const newPerformanceHistory = AgentProfileHelper.updatePerformanceHistory(profile.performanceHistory, metrics);

          // Update profile with new performance history
          const updatedProfile: AgentProfile = stryMutAct_9fa48("692") ? {} : (stryCov_9fa48("692"), {
            ...profile,
            performanceHistory: newPerformanceHistory,
            lastActiveAt: new Date().toISOString()
          });

          // Atomically update in registry
          this.agents.set(agentId, updatedProfile);

          // Record performance metrics to database if enabled
          if (stryMutAct_9fa48("694") ? false : stryMutAct_9fa48("693") ? true : (stryCov_9fa48("693", "694"), this.dbClient)) {
            if (stryMutAct_9fa48("695")) {
              {}
            } else {
              stryCov_9fa48("695");
              try {
                if (stryMutAct_9fa48("696")) {
                  {}
                } else {
                  stryCov_9fa48("696");
                  await this.dbClient.recordPerformance(agentId, metrics);
                }
              } catch (error) {
                if (stryMutAct_9fa48("697")) {
                  {}
                } else {
                  stryCov_9fa48("697");
                  // Log database error but don't fail the operation
                  console.error(`Failed to record performance to database for agent ${agentId}:`, error);
                }
              }
            }
          }
          return AgentProfileHelper.cloneProfile(updatedProfile);
        }
      } catch (error) {
        if (stryMutAct_9fa48("699")) {
          {}
        } else {
          stryCov_9fa48("699");
          throw new RegistryError(RegistryErrorType.UPDATE_FAILED, `Failed to update performance for agent ${agentId}: ${(error as Error).message}`, stryMutAct_9fa48("701") ? {} : (stryCov_9fa48("701"), {
            agentId,
            metrics,
            error
          }));
        }
      }
    }
  }

  /**
   * Update agent's current load (active and queued tasks).
   *
   * @param agentId - ID of the agent to update
   * @param activeTasks - New active tasks count
   * @param queuedTasks - New queued tasks count
   * @returns Updated agent profile
   * @throws RegistryError if agent not found
   */
  async updateLoad(agentId: AgentId, activeTasks: number, queuedTasks: number): Promise<AgentProfile> {
    if (stryMutAct_9fa48("702")) {
      {}
    } else {
      stryCov_9fa48("702");
      const profile = this.agents.get(agentId);
      if (stryMutAct_9fa48("705") ? false : stryMutAct_9fa48("704") ? true : stryMutAct_9fa48("703") ? profile : (stryCov_9fa48("703", "704", "705"), !profile)) {
        if (stryMutAct_9fa48("706")) {
          {}
        } else {
          stryCov_9fa48("706");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, `Agent with ID ${agentId} not found`, stryMutAct_9fa48("708") ? {} : (stryCov_9fa48("708"), {
            agentId
          }));
        }
      }
      const utilizationPercent = stryMutAct_9fa48("709") ? activeTasks / this.maxConcurrentTasksPerAgent / 100 : (stryCov_9fa48("709"), (stryMutAct_9fa48("710") ? activeTasks * this.maxConcurrentTasksPerAgent : (stryCov_9fa48("710"), activeTasks / this.maxConcurrentTasksPerAgent)) * 100);
      const updatedProfile: AgentProfile = stryMutAct_9fa48("711") ? {} : (stryCov_9fa48("711"), {
        ...profile,
        currentLoad: stryMutAct_9fa48("712") ? {} : (stryCov_9fa48("712"), {
          activeTasks,
          queuedTasks,
          utilizationPercent: stryMutAct_9fa48("713") ? Math.max(100, utilizationPercent) : (stryCov_9fa48("713"), Math.min(100, utilizationPercent))
        }),
        lastActiveAt: new Date().toISOString()
      });
      this.agents.set(agentId, updatedProfile);
      return AgentProfileHelper.cloneProfile(updatedProfile);
    }
  }

  /**
   * Get registry statistics.
   *
   * @returns Current registry stats
   */
  async getStats(): Promise<RegistryStats> {
    if (stryMutAct_9fa48("714")) {
      {}
    } else {
      stryCov_9fa48("714");
      const allAgents = Array.from(this.agents.values());
      const activeAgents = stryMutAct_9fa48("715") ? allAgents : (stryCov_9fa48("715"), allAgents.filter(stryMutAct_9fa48("716") ? () => undefined : (stryCov_9fa48("716"), a => stryMutAct_9fa48("720") ? a.currentLoad.activeTasks <= 0 : stryMutAct_9fa48("719") ? a.currentLoad.activeTasks >= 0 : stryMutAct_9fa48("718") ? false : stryMutAct_9fa48("717") ? true : (stryCov_9fa48("717", "718", "719", "720"), a.currentLoad.activeTasks > 0))));
      const idleAgents = stryMutAct_9fa48("721") ? allAgents : (stryCov_9fa48("721"), allAgents.filter(stryMutAct_9fa48("722") ? () => undefined : (stryCov_9fa48("722"), a => stryMutAct_9fa48("725") ? a.currentLoad.activeTasks !== 0 : stryMutAct_9fa48("724") ? false : stryMutAct_9fa48("723") ? true : (stryCov_9fa48("723", "724", "725"), a.currentLoad.activeTasks === 0))));
      const totalUtilization = allAgents.reduce(stryMutAct_9fa48("726") ? () => undefined : (stryCov_9fa48("726"), (sum, a) => stryMutAct_9fa48("727") ? sum - a.currentLoad.utilizationPercent : (stryCov_9fa48("727"), sum + a.currentLoad.utilizationPercent)), 0);
      const averageUtilization = (stryMutAct_9fa48("731") ? allAgents.length <= 0 : stryMutAct_9fa48("730") ? allAgents.length >= 0 : stryMutAct_9fa48("729") ? false : stryMutAct_9fa48("728") ? true : (stryCov_9fa48("728", "729", "730", "731"), allAgents.length > 0)) ? stryMutAct_9fa48("732") ? totalUtilization * allAgents.length : (stryCov_9fa48("732"), totalUtilization / allAgents.length) : 0;
      const totalSuccessRate = allAgents.reduce(stryMutAct_9fa48("733") ? () => undefined : (stryCov_9fa48("733"), (sum, a) => stryMutAct_9fa48("734") ? sum - a.performanceHistory.successRate : (stryCov_9fa48("734"), sum + a.performanceHistory.successRate)), 0);
      const averageSuccessRate = (stryMutAct_9fa48("738") ? allAgents.length <= 0 : stryMutAct_9fa48("737") ? allAgents.length >= 0 : stryMutAct_9fa48("736") ? false : stryMutAct_9fa48("735") ? true : (stryCov_9fa48("735", "736", "737", "738"), allAgents.length > 0)) ? stryMutAct_9fa48("739") ? totalSuccessRate * allAgents.length : (stryCov_9fa48("739"), totalSuccessRate / allAgents.length) : 0;
      return stryMutAct_9fa48("740") ? {} : (stryCov_9fa48("740"), {
        totalAgents: allAgents.length,
        activeAgents: activeAgents.length,
        idleAgents: idleAgents.length,
        averageUtilization,
        averageSuccessRate,
        lastUpdated: new Date().toISOString()
      });
    }
  }

  /**
   * Remove an agent from the registry.
   *
   * @param agentId - ID of the agent to remove
   * @returns True if agent was removed
   */
  async unregisterAgent(agentId: AgentId): Promise<boolean> {
    if (stryMutAct_9fa48("741")) {
      {}
    } else {
      stryCov_9fa48("741");
      return this.agents.delete(agentId);
    }
  }

  /**
   * Initialize capability tracking for a new agent.
   */
  private async initializeCapabilityTracking(
  // eslint-disable-next-line @typescript-eslint/no-unused-vars, no-unused-vars
  _profile: AgentProfile): Promise<void> {
    // Capability tracking initialization
    // In production, this would set up monitoring for capability usage
    // and initialize any external tracking systems
    // For now, this is a no-op, but provides extension point
  }

  /**
   * Calculate match score for query result ranking.
   *
   * @param profile - Agent profile
   * @param query - Query parameters
   * @returns Match score (0.0 - 1.0)
   */
  private calculateMatchScore(profile: AgentProfile, query: AgentQuery): number {
    if (stryMutAct_9fa48("742")) {
      {}
    } else {
      stryCov_9fa48("742");
      let score = 0.0;
      let weights = 0.0;

      // Task type match (required, so always contributes)
      stryMutAct_9fa48("743") ? score -= 0.3 : (stryCov_9fa48("743"), score += 0.3);
      stryMutAct_9fa48("744") ? weights -= 0.3 : (stryCov_9fa48("744"), weights += 0.3);

      // Language matches (if specified)
      if (stryMutAct_9fa48("747") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("746") ? false : stryMutAct_9fa48("745") ? true : (stryCov_9fa48("745", "746", "747"), query.languages && (stryMutAct_9fa48("750") ? query.languages.length <= 0 : stryMutAct_9fa48("749") ? query.languages.length >= 0 : stryMutAct_9fa48("748") ? true : (stryCov_9fa48("748", "749", "750"), query.languages.length > 0)))) {
        if (stryMutAct_9fa48("751")) {
          {}
        } else {
          stryCov_9fa48("751");
          const matchedLanguages = stryMutAct_9fa48("752") ? query.languages.length : (stryCov_9fa48("752"), query.languages.filter(stryMutAct_9fa48("753") ? () => undefined : (stryCov_9fa48("753"), lang => profile.capabilities.languages.includes(lang))).length);
          stryMutAct_9fa48("754") ? score -= matchedLanguages / query.languages.length * 0.3 : (stryCov_9fa48("754"), score += stryMutAct_9fa48("755") ? matchedLanguages / query.languages.length / 0.3 : (stryCov_9fa48("755"), (stryMutAct_9fa48("756") ? matchedLanguages * query.languages.length : (stryCov_9fa48("756"), matchedLanguages / query.languages.length)) * 0.3));
          stryMutAct_9fa48("757") ? weights -= 0.3 : (stryCov_9fa48("757"), weights += 0.3);
        }
      }

      // Specialization matches (if specified)
      if (stryMutAct_9fa48("760") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("759") ? false : stryMutAct_9fa48("758") ? true : (stryCov_9fa48("758", "759", "760"), query.specializations && (stryMutAct_9fa48("763") ? query.specializations.length <= 0 : stryMutAct_9fa48("762") ? query.specializations.length >= 0 : stryMutAct_9fa48("761") ? true : (stryCov_9fa48("761", "762", "763"), query.specializations.length > 0)))) {
        if (stryMutAct_9fa48("764")) {
          {}
        } else {
          stryCov_9fa48("764");
          const matchedSpecs = stryMutAct_9fa48("765") ? query.specializations.length : (stryCov_9fa48("765"), query.specializations.filter(stryMutAct_9fa48("766") ? () => undefined : (stryCov_9fa48("766"), spec => profile.capabilities.specializations.includes(spec))).length);
          stryMutAct_9fa48("767") ? score -= matchedSpecs / query.specializations.length * 0.2 : (stryCov_9fa48("767"), score += stryMutAct_9fa48("768") ? matchedSpecs / query.specializations.length / 0.2 : (stryCov_9fa48("768"), (stryMutAct_9fa48("769") ? matchedSpecs * query.specializations.length : (stryCov_9fa48("769"), matchedSpecs / query.specializations.length)) * 0.2));
          stryMutAct_9fa48("770") ? weights -= 0.2 : (stryCov_9fa48("770"), weights += 0.2);
        }
      }

      // Performance bonus
      stryMutAct_9fa48("771") ? score -= profile.performanceHistory.successRate * 0.2 : (stryCov_9fa48("771"), score += stryMutAct_9fa48("772") ? profile.performanceHistory.successRate / 0.2 : (stryCov_9fa48("772"), profile.performanceHistory.successRate * 0.2));
      stryMutAct_9fa48("773") ? weights -= 0.2 : (stryCov_9fa48("773"), weights += 0.2);
      return (stryMutAct_9fa48("777") ? weights <= 0 : stryMutAct_9fa48("776") ? weights >= 0 : stryMutAct_9fa48("775") ? false : stryMutAct_9fa48("774") ? true : (stryCov_9fa48("774", "775", "776", "777"), weights > 0)) ? stryMutAct_9fa48("778") ? score * weights : (stryCov_9fa48("778"), score / weights) : 0;
    }
  }

  /**
   * Generate human-readable explanation of match score.
   *
   * @param profile - Agent profile
   * @param query - Query parameters
   * @returns Explanation string
   */
  private explainMatchScore(profile: AgentProfile, query: AgentQuery,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars, no-unused-vars
  _score: number): string {
    if (stryMutAct_9fa48("779")) {
      {}
    } else {
      stryCov_9fa48("779");
      const reasons: string[] = [];
      reasons.push(`Supports ${query.taskType}`);
      if (stryMutAct_9fa48("784") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("783") ? false : stryMutAct_9fa48("782") ? true : (stryCov_9fa48("782", "783", "784"), query.languages && (stryMutAct_9fa48("787") ? query.languages.length <= 0 : stryMutAct_9fa48("786") ? query.languages.length >= 0 : stryMutAct_9fa48("785") ? true : (stryCov_9fa48("785", "786", "787"), query.languages.length > 0)))) {
        if (stryMutAct_9fa48("788")) {
          {}
        } else {
          stryCov_9fa48("788");
          reasons.push(`Languages: ${query.languages.join(", ")}`);
        }
      }
      if (stryMutAct_9fa48("793") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("792") ? false : stryMutAct_9fa48("791") ? true : (stryCov_9fa48("791", "792", "793"), query.specializations && (stryMutAct_9fa48("796") ? query.specializations.length <= 0 : stryMutAct_9fa48("795") ? query.specializations.length >= 0 : stryMutAct_9fa48("794") ? true : (stryCov_9fa48("794", "795", "796"), query.specializations.length > 0)))) {
        if (stryMutAct_9fa48("797")) {
          {}
        } else {
          stryCov_9fa48("797");
          reasons.push(`Specializations: ${query.specializations.join(", ")}`);
        }
      }
      reasons.push(`${(stryMutAct_9fa48("801") ? profile.performanceHistory.successRate / 100 : (stryCov_9fa48("801"), profile.performanceHistory.successRate * 100)).toFixed(1)}% success rate`);
      reasons.push(`${profile.currentLoad.utilizationPercent.toFixed(0)}% utilized`);
      return reasons.join("; ");
    }
  }

  /**
   * Start automatic cleanup of stale agents.
   */
  private startAutoCleanup(): void {
    if (stryMutAct_9fa48("804")) {
      {}
    } else {
      stryCov_9fa48("804");
      this.cleanupTimer = setInterval(() => {
        if (stryMutAct_9fa48("805")) {
          {}
        } else {
          stryCov_9fa48("805");
          this.cleanupStaleAgents();
        }
      }, this.config.cleanupIntervalMs);
    }
  }

  /**
   * Clean up stale agents (inactive beyond threshold).
   */
  private cleanupStaleAgents(): void {
    if (stryMutAct_9fa48("806")) {
      {}
    } else {
      stryCov_9fa48("806");
      const now = new Date().toISOString();
      const staleAgents: AgentId[] = [];
      const agents = Array.from(this.agents.entries());
      for (const [agentId, profile] of agents) {
        if (stryMutAct_9fa48("808")) {
          {}
        } else {
          stryCov_9fa48("808");
          if (stryMutAct_9fa48("810") ? false : stryMutAct_9fa48("809") ? true : (stryCov_9fa48("809", "810"), AgentProfileHelper.isStale(profile, this.config.staleAgentThresholdMs, now))) {
            if (stryMutAct_9fa48("811")) {
              {}
            } else {
              stryCov_9fa48("811");
              staleAgents.push(agentId);
            }
          }
        }
      }
      for (const agentId of staleAgents) {
        if (stryMutAct_9fa48("812")) {
          {}
        } else {
          stryCov_9fa48("812");
          this.agents.delete(agentId);
        }
      }
    }
  }

  /**
   * Shutdown the registry manager and cleanup resources.
   */
  async shutdown(): Promise<void> {
    if (stryMutAct_9fa48("813")) {
      {}
    } else {
      stryCov_9fa48("813");
      if (stryMutAct_9fa48("815") ? false : stryMutAct_9fa48("814") ? true : (stryCov_9fa48("814", "815"), this.cleanupTimer)) {
        if (stryMutAct_9fa48("816")) {
          {}
        } else {
          stryCov_9fa48("816");
          clearInterval(this.cleanupTimer);
        }
      }
    }
  }

  /**
   * Generate a unique ID for audit events
   */
  private generateId(): string {
    if (stryMutAct_9fa48("817")) {
      {}
    } else {
      stryCov_9fa48("817");
      return `audit_${Date.now()}_${stryMutAct_9fa48("819") ? Math.random().toString(36) : (stryCov_9fa48("819"), Math.random().toString(36).substr(2, 9))}`;
    }
  }
}