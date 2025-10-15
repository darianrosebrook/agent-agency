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
import { PerformanceTracker } from "../rl/PerformanceTracker";
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
  private performanceTracker?: PerformanceTracker;
  constructor(config: Partial<AgentRegistryConfig> = {}, performanceTracker?: PerformanceTracker) {
    if (stryMutAct_9fa48("454")) {
      {}
    } else {
      stryCov_9fa48("454");
      this.agents = new Map();
      this.config = stryMutAct_9fa48("455") ? {} : (stryCov_9fa48("455"), {
        ...DEFAULT_CONFIG,
        ...config
      });
      this.performanceTracker = performanceTracker;

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
   * Set the performance tracker for agent lifecycle tracking.
   *
   * @param tracker - Performance tracker instance
   */
  setPerformanceTracker(tracker: PerformanceTracker): void {
    if (stryMutAct_9fa48("467")) {
      {}
    } else {
      stryCov_9fa48("467");
      this.performanceTracker = tracker;
    }
  }

  /**
   * Initialize the registry manager.
   *
   * Must be called before using the registry if persistence is enabled.
   */
  async initialize(): Promise<void> {
    if (stryMutAct_9fa48("468")) {
      {}
    } else {
      stryCov_9fa48("468");
      if (stryMutAct_9fa48("471") ? this.config.enablePersistence || this.dbClient : stryMutAct_9fa48("470") ? false : stryMutAct_9fa48("469") ? true : (stryCov_9fa48("469", "470", "471"), this.config.enablePersistence && this.dbClient)) {
        if (stryMutAct_9fa48("472")) {
          {}
        } else {
          stryCov_9fa48("472");
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
    if (stryMutAct_9fa48("473")) {
      {}
    } else {
      stryCov_9fa48("473");
      if (stryMutAct_9fa48("476") ? false : stryMutAct_9fa48("475") ? true : stryMutAct_9fa48("474") ? this.dbClient : (stryCov_9fa48("474", "475", "476"), !this.dbClient)) return;
      try {
        if (stryMutAct_9fa48("477")) {
          {}
        } else {
          stryCov_9fa48("477");
          // Query all agents (simplified query for loading)
          const result = await this.dbClient.queryAgents(stryMutAct_9fa48("478") ? {} : (stryCov_9fa48("478"), {
            taskType: "code-editing" // Required field
          }));

          // Load agents into memory cache
          for (const queryResult of result) {
            if (stryMutAct_9fa48("480")) {
              {}
            } else {
              stryCov_9fa48("480");
              this.agents.set(queryResult.agent.id, queryResult.agent);
            }
          }

          // Log successful loading
          console.log(`Loaded ${result.length} agents from database`);
        }
      } catch (error) {
        if (stryMutAct_9fa48("482")) {
          {}
        } else {
          stryCov_9fa48("482");
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
    if (stryMutAct_9fa48("484")) {
      {}
    } else {
      stryCov_9fa48("484");
      // Security check: authenticate and authorize
      if (stryMutAct_9fa48("487") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("486") ? false : stryMutAct_9fa48("485") ? true : (stryCov_9fa48("485", "486", "487"), this.config.enableSecurity && this.securityManager)) {
        if (stryMutAct_9fa48("488")) {
          {}
        } else {
          stryCov_9fa48("488");
          if (stryMutAct_9fa48("491") ? false : stryMutAct_9fa48("490") ? true : stryMutAct_9fa48("489") ? securityContext : (stryCov_9fa48("489", "490", "491"), !securityContext)) {
            if (stryMutAct_9fa48("492")) {
              {}
            } else {
              stryCov_9fa48("492");
              throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, "Security context required when security is enabled");
            }
          }
          const authorized = await this.securityManager.authorize(securityContext, "create" as any, "agent", stryMutAct_9fa48("497") ? agent.id && "unknown" : stryMutAct_9fa48("496") ? false : stryMutAct_9fa48("495") ? true : (stryCov_9fa48("495", "496", "497"), agent.id || "unknown"), agent);
          if (stryMutAct_9fa48("501") ? false : stryMutAct_9fa48("500") ? true : stryMutAct_9fa48("499") ? authorized : (stryCov_9fa48("499", "500", "501"), !authorized)) {
            if (stryMutAct_9fa48("502")) {
              {}
            } else {
              stryCov_9fa48("502");
              await this.securityManager.logAuditEvent(stryMutAct_9fa48("503") ? {} : (stryCov_9fa48("503"), {
                id: this.generateId(),
                timestamp: new Date(),
                eventType: "agent_registration" as any,
                actor: stryMutAct_9fa48("504") ? {} : (stryCov_9fa48("504"), {
                  tenantId: securityContext.tenantId,
                  userId: securityContext.userId,
                  sessionId: securityContext.sessionId
                }),
                resource: stryMutAct_9fa48("505") ? {} : (stryCov_9fa48("505"), {
                  type: "agent",
                  id: stryMutAct_9fa48("509") ? agent.id && "unknown" : stryMutAct_9fa48("508") ? false : stryMutAct_9fa48("507") ? true : (stryCov_9fa48("507", "508", "509"), agent.id || "unknown")
                }),
                action: "create" as any,
                details: stryMutAct_9fa48("511") ? {} : (stryCov_9fa48("511"), {
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
      if (stryMutAct_9fa48("517") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("516") ? false : stryMutAct_9fa48("515") ? true : (stryCov_9fa48("515", "516", "517"), this.config.enableSecurity && this.securityManager)) {
        if (stryMutAct_9fa48("518")) {
          {}
        } else {
          stryCov_9fa48("518");
          const validation = this.securityManager.validateAgentData(agent);
          if (stryMutAct_9fa48("521") ? false : stryMutAct_9fa48("520") ? true : stryMutAct_9fa48("519") ? validation.valid : (stryCov_9fa48("519", "520", "521"), !validation.valid)) {
            if (stryMutAct_9fa48("522")) {
              {}
            } else {
              stryCov_9fa48("522");
              throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, `Validation failed: ${validation.errors.join(", ")}`);
            }
          }
          // Use sanitized data if available
          if (stryMutAct_9fa48("526") ? false : stryMutAct_9fa48("525") ? true : (stryCov_9fa48("525", "526"), validation.sanitized)) {
            if (stryMutAct_9fa48("527")) {
              {}
            } else {
              stryCov_9fa48("527");
              agent = validation.sanitized;
            }
          }
        }
      } else {
        if (stryMutAct_9fa48("528")) {
          {}
        } else {
          stryCov_9fa48("528");
          // Fallback to basic validation
          AgentProfileHelper.validateProfile(agent);
        }
      }
      if (stryMutAct_9fa48("531") ? false : stryMutAct_9fa48("530") ? true : stryMutAct_9fa48("529") ? agent.id : (stryCov_9fa48("529", "530", "531"), !agent.id)) {
        if (stryMutAct_9fa48("532")) {
          {}
        } else {
          stryCov_9fa48("532");
          throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, "Agent ID is required");
        }
      }

      // Check if agent already exists
      if (stryMutAct_9fa48("535") ? false : stryMutAct_9fa48("534") ? true : (stryCov_9fa48("534", "535"), this.agents.has(agent.id))) {
        if (stryMutAct_9fa48("536")) {
          {}
        } else {
          stryCov_9fa48("536");
          throw new RegistryError(RegistryErrorType.AGENT_ALREADY_EXISTS, `Agent with ID ${agent.id} already exists`, stryMutAct_9fa48("538") ? {} : (stryCov_9fa48("538"), {
            agentId: agent.id
          }));
        }
      }

      // Check registry capacity
      if (stryMutAct_9fa48("542") ? this.agents.size < this.config.maxAgents : stryMutAct_9fa48("541") ? this.agents.size > this.config.maxAgents : stryMutAct_9fa48("540") ? false : stryMutAct_9fa48("539") ? true : (stryCov_9fa48("539", "540", "541", "542"), this.agents.size >= this.config.maxAgents)) {
        if (stryMutAct_9fa48("543")) {
          {}
        } else {
          stryCov_9fa48("543");
          throw new RegistryError(RegistryErrorType.REGISTRY_FULL, `Registry is full (max: ${this.config.maxAgents} agents)`, stryMutAct_9fa48("545") ? {} : (stryCov_9fa48("545"), {
            maxAgents: this.config.maxAgents,
            currentSize: this.agents.size
          }));
        }
      }

      // Create complete profile with defaults
      const now = new Date().toISOString();
      const profile: AgentProfile = stryMutAct_9fa48("546") ? {} : (stryCov_9fa48("546"), {
        id: agent.id,
        name: agent.name!,
        modelFamily: agent.modelFamily!,
        capabilities: agent.capabilities!,
        performanceHistory: stryMutAct_9fa48("547") ? agent.performanceHistory && AgentProfileHelper.createInitialPerformanceHistory() : (stryCov_9fa48("547"), agent.performanceHistory ?? AgentProfileHelper.createInitialPerformanceHistory()),
        currentLoad: stryMutAct_9fa48("548") ? agent.currentLoad && AgentProfileHelper.createInitialLoad() : (stryCov_9fa48("548"), agent.currentLoad ?? AgentProfileHelper.createInitialLoad()),
        registeredAt: now,
        lastActiveAt: now
      });

      // Initialize capability tracking
      await this.initializeCapabilityTracking(profile);

      // Store in registry
      this.agents.set(profile.id, profile);

      // Persist to database if enabled
      if (stryMutAct_9fa48("550") ? false : stryMutAct_9fa48("549") ? true : (stryCov_9fa48("549", "550"), this.dbClient)) {
        if (stryMutAct_9fa48("551")) {
          {}
        } else {
          stryCov_9fa48("551");
          try {
            if (stryMutAct_9fa48("552")) {
              {}
            } else {
              stryCov_9fa48("552");
              await this.dbClient.registerAgent(profile);
            }
          } catch (error) {
            if (stryMutAct_9fa48("553")) {
              {}
            } else {
              stryCov_9fa48("553");
              // Rollback in-memory storage on database failure
              this.agents.delete(profile.id);
              throw new RegistryError(RegistryErrorType.DATABASE_ERROR, `Failed to persist agent to database: ${error instanceof Error ? error.message : String(error)}`, stryMutAct_9fa48("555") ? {} : (stryCov_9fa48("555"), {
                agentId: profile.id
              }));
            }
          }
        }
      }

      // Audit log successful registration
      if (stryMutAct_9fa48("558") ? this.config.enableSecurity && this.securityManager || securityContext : stryMutAct_9fa48("557") ? false : stryMutAct_9fa48("556") ? true : (stryCov_9fa48("556", "557", "558"), (stryMutAct_9fa48("560") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("559") ? true : (stryCov_9fa48("559", "560"), this.config.enableSecurity && this.securityManager)) && securityContext)) {
        if (stryMutAct_9fa48("561")) {
          {}
        } else {
          stryCov_9fa48("561");
          await this.securityManager.logAuditEvent(stryMutAct_9fa48("562") ? {} : (stryCov_9fa48("562"), {
            id: this.generateId(),
            timestamp: new Date(),
            eventType: "agent_registration" as any,
            actor: stryMutAct_9fa48("563") ? {} : (stryCov_9fa48("563"), {
              tenantId: securityContext.tenantId,
              userId: securityContext.userId,
              sessionId: securityContext.sessionId
            }),
            resource: stryMutAct_9fa48("564") ? {} : (stryCov_9fa48("564"), {
              type: "agent",
              id: profile.id
            }),
            action: "create" as any,
            details: stryMutAct_9fa48("566") ? {} : (stryCov_9fa48("566"), {
              agentProfile: profile
            }),
            result: "success",
            ipAddress: securityContext.ipAddress,
            userAgent: securityContext.userAgent
          }));
        }
      }

      // Record performance baseline for new agent
      if (stryMutAct_9fa48("569") ? false : stryMutAct_9fa48("568") ? true : (stryCov_9fa48("568", "569"), this.performanceTracker)) {
        if (stryMutAct_9fa48("570")) {
          {}
        } else {
          stryCov_9fa48("570");
          try {
            if (stryMutAct_9fa48("571")) {
              {}
            } else {
              stryCov_9fa48("571");
              await this.performanceTracker.recordAgentRegistration(profile.id, stryMutAct_9fa48("572") ? {} : (stryCov_9fa48("572"), {
                capabilities: profile.capabilities.taskTypes,
                baselineMetrics: this.calculateBaselineMetrics(profile),
                registrationTimestamp: profile.registeredAt
              }));
            }
          } catch (error) {
            if (stryMutAct_9fa48("573")) {
              {}
            } else {
              stryCov_9fa48("573");
              // Log but don't fail registration due to performance tracking issues
              console.warn(`Failed to record agent registration performance baseline for ${profile.id}:`, error);
            }
          }
        }
      }
      return AgentProfileHelper.cloneProfile(profile);
    }
  }

  /**
   * Update agent availability status.
   *
   * @param agentId - ID of the agent to update
   * @param status - New availability status
   * @param reason - Optional reason for status change
   * @param securityContext - Security context for authorization
   * @throws RegistryError if agent not found or unauthorized
   */
  async updateAgentStatus(agentId: AgentId, status: "available" | "busy" | "offline" | "maintenance", reason?: string, securityContext?: SecurityContext): Promise<void> {
    if (stryMutAct_9fa48("575")) {
      {}
    } else {
      stryCov_9fa48("575");
      // Security check: authenticate and authorize
      if (stryMutAct_9fa48("578") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("577") ? false : stryMutAct_9fa48("576") ? true : (stryCov_9fa48("576", "577", "578"), this.config.enableSecurity && this.securityManager)) {
        if (stryMutAct_9fa48("579")) {
          {}
        } else {
          stryCov_9fa48("579");
          if (stryMutAct_9fa48("582") ? false : stryMutAct_9fa48("581") ? true : stryMutAct_9fa48("580") ? securityContext : (stryCov_9fa48("580", "581", "582"), !securityContext)) {
            if (stryMutAct_9fa48("583")) {
              {}
            } else {
              stryCov_9fa48("583");
              throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, "Security context required when security is enabled");
            }
          }
          const authorized = await this.securityManager.authorize(securityContext, "update" as any, "agent", agentId);
          if (stryMutAct_9fa48("588") ? false : stryMutAct_9fa48("587") ? true : stryMutAct_9fa48("586") ? authorized : (stryCov_9fa48("586", "587", "588"), !authorized)) {
            if (stryMutAct_9fa48("589")) {
              {}
            } else {
              stryCov_9fa48("589");
              await this.securityManager.logAuditEvent(stryMutAct_9fa48("590") ? {} : (stryCov_9fa48("590"), {
                id: this.generateId(),
                timestamp: new Date(),
                eventType: "agent_status_update" as any,
                actor: stryMutAct_9fa48("591") ? {} : (stryCov_9fa48("591"), {
                  tenantId: securityContext.tenantId,
                  userId: securityContext.userId,
                  sessionId: securityContext.sessionId
                }),
                resource: stryMutAct_9fa48("592") ? {} : (stryCov_9fa48("592"), {
                  type: "agent",
                  id: agentId
                }),
                action: "update" as any,
                details: stryMutAct_9fa48("594") ? {} : (stryCov_9fa48("594"), {
                  status,
                  reason
                }),
                result: "failure",
                errorMessage: "Authorization failed",
                ipAddress: securityContext.ipAddress,
                userAgent: securityContext.userAgent
              }));
              throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, "Not authorized to update agent status");
            }
          }
        }
      }

      // Get current agent profile
      const profile = this.agents.get(agentId);
      if (stryMutAct_9fa48("600") ? false : stryMutAct_9fa48("599") ? true : stryMutAct_9fa48("598") ? profile : (stryCov_9fa48("598", "599", "600"), !profile)) {
        if (stryMutAct_9fa48("601")) {
          {}
        } else {
          stryCov_9fa48("601");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, `Agent with ID ${agentId} not found`, stryMutAct_9fa48("603") ? {} : (stryCov_9fa48("603"), {
            agentId
          }));
        }
      }

      // Get previous status for tracking
      const previousStatus = this.getAgentAvailabilityStatus(profile);

      // Update agent load status based on new availability
      const updatedProfile = AgentProfileHelper.cloneProfile(profile);
      updatedProfile.lastActiveAt = new Date().toISOString();

      // Update load based on status
      switch (status) {
        case "available":
          if (stryMutAct_9fa48("604")) {} else {
            stryCov_9fa48("604");
            updatedProfile.currentLoad = stryMutAct_9fa48("606") ? {} : (stryCov_9fa48("606"), {
              ...updatedProfile.currentLoad,
              activeTasks: 0,
              utilizationPercent: 0
            });
            break;
          }
        case "busy":
          if (stryMutAct_9fa48("607")) {} else {
            stryCov_9fa48("607");
            updatedProfile.currentLoad = stryMutAct_9fa48("609") ? {} : (stryCov_9fa48("609"), {
              ...updatedProfile.currentLoad,
              activeTasks: stryMutAct_9fa48("610") ? Math.min(updatedProfile.currentLoad.activeTasks, 1) : (stryCov_9fa48("610"), Math.max(updatedProfile.currentLoad.activeTasks, 1)),
              utilizationPercent: stryMutAct_9fa48("611") ? Math.min(updatedProfile.currentLoad.utilizationPercent, 50) : (stryCov_9fa48("611"), Math.max(updatedProfile.currentLoad.utilizationPercent, 50))
            });
            break;
          }
        case "offline":
        case "maintenance":
          if (stryMutAct_9fa48("613")) {} else {
            stryCov_9fa48("613");
            updatedProfile.currentLoad = stryMutAct_9fa48("615") ? {} : (stryCov_9fa48("615"), {
              ...updatedProfile.currentLoad,
              activeTasks: this.maxConcurrentTasksPerAgent,
              // Fully utilized = unavailable
              utilizationPercent: 100
            });
            break;
          }
      }

      // Store updated profile
      this.agents.set(agentId, updatedProfile);

      // Persist to database if enabled (TODO: implement updateAgentStatus in database client)

      // Audit log successful status update
      if (stryMutAct_9fa48("618") ? this.config.enableSecurity && this.securityManager || securityContext : stryMutAct_9fa48("617") ? false : stryMutAct_9fa48("616") ? true : (stryCov_9fa48("616", "617", "618"), (stryMutAct_9fa48("620") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("619") ? true : (stryCov_9fa48("619", "620"), this.config.enableSecurity && this.securityManager)) && securityContext)) {
        if (stryMutAct_9fa48("621")) {
          {}
        } else {
          stryCov_9fa48("621");
          await this.securityManager.logAuditEvent(stryMutAct_9fa48("622") ? {} : (stryCov_9fa48("622"), {
            id: this.generateId(),
            timestamp: new Date(),
            eventType: "agent_status_update" as any,
            actor: stryMutAct_9fa48("623") ? {} : (stryCov_9fa48("623"), {
              tenantId: securityContext.tenantId,
              userId: securityContext.userId,
              sessionId: securityContext.sessionId
            }),
            resource: stryMutAct_9fa48("624") ? {} : (stryCov_9fa48("624"), {
              type: "agent",
              id: agentId
            }),
            action: "update" as any,
            details: stryMutAct_9fa48("626") ? {} : (stryCov_9fa48("626"), {
              previousStatus,
              newStatus: status,
              reason
            }),
            result: "success",
            ipAddress: securityContext.ipAddress,
            userAgent: securityContext.userAgent
          }));
        }
      }

      // Record status change in performance tracker
      if (stryMutAct_9fa48("629") ? false : stryMutAct_9fa48("628") ? true : (stryCov_9fa48("628", "629"), this.performanceTracker)) {
        if (stryMutAct_9fa48("630")) {
          {}
        } else {
          stryCov_9fa48("630");
          try {
            if (stryMutAct_9fa48("631")) {
              {}
            } else {
              stryCov_9fa48("631");
              await this.performanceTracker.recordAgentStatusChange(agentId, status, stryMutAct_9fa48("632") ? {} : (stryCov_9fa48("632"), {
                previousStatus,
                reason
              }));
            }
          } catch (error) {
            if (stryMutAct_9fa48("633")) {
              {}
            } else {
              stryCov_9fa48("633");
              // Log but don't fail status update due to performance tracking issues
              console.warn(`Failed to record agent status change performance event for ${agentId}:`, error);
            }
          }
        }
      }
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
    if (stryMutAct_9fa48("635")) {
      {}
    } else {
      stryCov_9fa48("635");
      // Security check: authenticate and authorize
      if (stryMutAct_9fa48("638") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("637") ? false : stryMutAct_9fa48("636") ? true : (stryCov_9fa48("636", "637", "638"), this.config.enableSecurity && this.securityManager)) {
        if (stryMutAct_9fa48("639")) {
          {}
        } else {
          stryCov_9fa48("639");
          if (stryMutAct_9fa48("642") ? false : stryMutAct_9fa48("641") ? true : stryMutAct_9fa48("640") ? securityContext : (stryCov_9fa48("640", "641", "642"), !securityContext)) {
            if (stryMutAct_9fa48("643")) {
              {}
            } else {
              stryCov_9fa48("643");
              throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, "Security context required when security is enabled");
            }
          }
          const authorized = await this.securityManager.authorize(securityContext, "read" as any, "agent", agentId);
          if (stryMutAct_9fa48("648") ? false : stryMutAct_9fa48("647") ? true : stryMutAct_9fa48("646") ? authorized : (stryCov_9fa48("646", "647", "648"), !authorized)) {
            if (stryMutAct_9fa48("649")) {
              {}
            } else {
              stryCov_9fa48("649");
              await this.securityManager.logAuditEvent(stryMutAct_9fa48("650") ? {} : (stryCov_9fa48("650"), {
                id: this.generateId(),
                timestamp: new Date(),
                eventType: "agent_query" as any,
                actor: stryMutAct_9fa48("651") ? {} : (stryCov_9fa48("651"), {
                  tenantId: securityContext.tenantId,
                  userId: securityContext.userId,
                  sessionId: securityContext.sessionId
                }),
                resource: stryMutAct_9fa48("652") ? {} : (stryCov_9fa48("652"), {
                  type: "agent",
                  id: agentId
                }),
                action: "read" as any,
                details: stryMutAct_9fa48("654") ? {} : (stryCov_9fa48("654"), {
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
      if (stryMutAct_9fa48("661") ? !profile || this.dbClient : stryMutAct_9fa48("660") ? false : stryMutAct_9fa48("659") ? true : (stryCov_9fa48("659", "660", "661"), (stryMutAct_9fa48("662") ? profile : (stryCov_9fa48("662"), !profile)) && this.dbClient)) {
        if (stryMutAct_9fa48("663")) {
          {}
        } else {
          stryCov_9fa48("663");
          try {
            if (stryMutAct_9fa48("664")) {
              {}
            } else {
              stryCov_9fa48("664");
              const dbProfile = await this.dbClient.getAgent(agentId);
              if (stryMutAct_9fa48("666") ? false : stryMutAct_9fa48("665") ? true : (stryCov_9fa48("665", "666"), dbProfile)) {
                if (stryMutAct_9fa48("667")) {
                  {}
                } else {
                  stryCov_9fa48("667");
                  // Cache in memory for future requests
                  this.agents.set(agentId, dbProfile);
                  profile = dbProfile;
                }
              }
            }
          } catch (error) {
            if (stryMutAct_9fa48("668")) {
              {}
            } else {
              stryCov_9fa48("668");
              throw new RegistryError(RegistryErrorType.DATABASE_ERROR, `Failed to retrieve agent from database: ${error instanceof Error ? error.message : String(error)}`, stryMutAct_9fa48("670") ? {} : (stryCov_9fa48("670"), {
                agentId
              }));
            }
          }
        }
      }
      if (stryMutAct_9fa48("673") ? false : stryMutAct_9fa48("672") ? true : stryMutAct_9fa48("671") ? profile : (stryCov_9fa48("671", "672", "673"), !profile)) {
        if (stryMutAct_9fa48("674")) {
          {}
        } else {
          stryCov_9fa48("674");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, `Agent with ID ${agentId} not found`, stryMutAct_9fa48("676") ? {} : (stryCov_9fa48("676"), {
            agentId
          }));
        }
      }

      // Audit log successful profile access
      if (stryMutAct_9fa48("679") ? this.config.enableSecurity && this.securityManager || securityContext : stryMutAct_9fa48("678") ? false : stryMutAct_9fa48("677") ? true : (stryCov_9fa48("677", "678", "679"), (stryMutAct_9fa48("681") ? this.config.enableSecurity || this.securityManager : stryMutAct_9fa48("680") ? true : (stryCov_9fa48("680", "681"), this.config.enableSecurity && this.securityManager)) && securityContext)) {
        if (stryMutAct_9fa48("682")) {
          {}
        } else {
          stryCov_9fa48("682");
          await this.securityManager.logAuditEvent(stryMutAct_9fa48("683") ? {} : (stryCov_9fa48("683"), {
            id: this.generateId(),
            timestamp: new Date(),
            eventType: "agent_query" as any,
            actor: stryMutAct_9fa48("684") ? {} : (stryCov_9fa48("684"), {
              tenantId: securityContext.tenantId,
              userId: securityContext.userId,
              sessionId: securityContext.sessionId
            }),
            resource: stryMutAct_9fa48("685") ? {} : (stryCov_9fa48("685"), {
              type: "agent",
              id: agentId
            }),
            action: "read" as any,
            details: stryMutAct_9fa48("687") ? {} : (stryCov_9fa48("687"), {
              queryType: "getProfile",
              found: stryMutAct_9fa48("689") ? false : (stryCov_9fa48("689"), true)
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
    if (stryMutAct_9fa48("691")) {
      {}
    } else {
      stryCov_9fa48("691");
      const results: AgentQueryResult[] = [];
      for (const profile of Array.from(this.agents.values())) {
        if (stryMutAct_9fa48("693")) {
          {}
        } else {
          stryCov_9fa48("693");
          // Check task type match
          if (stryMutAct_9fa48("696") ? false : stryMutAct_9fa48("695") ? true : stryMutAct_9fa48("694") ? profile.capabilities.taskTypes.includes(query.taskType) : (stryCov_9fa48("694", "695", "696"), !profile.capabilities.taskTypes.includes(query.taskType))) {
            if (stryMutAct_9fa48("697")) {
              {}
            } else {
              stryCov_9fa48("697");
              continue;
            }
          }

          // Check language requirements if specified
          if (stryMutAct_9fa48("700") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("699") ? false : stryMutAct_9fa48("698") ? true : (stryCov_9fa48("698", "699", "700"), query.languages && (stryMutAct_9fa48("703") ? query.languages.length <= 0 : stryMutAct_9fa48("702") ? query.languages.length >= 0 : stryMutAct_9fa48("701") ? true : (stryCov_9fa48("701", "702", "703"), query.languages.length > 0)))) {
            if (stryMutAct_9fa48("704")) {
              {}
            } else {
              stryCov_9fa48("704");
              const hasAllLanguages = stryMutAct_9fa48("705") ? query.languages.some(lang => profile.capabilities.languages.includes(lang)) : (stryCov_9fa48("705"), query.languages.every(stryMutAct_9fa48("706") ? () => undefined : (stryCov_9fa48("706"), lang => profile.capabilities.languages.includes(lang))));
              if (stryMutAct_9fa48("709") ? false : stryMutAct_9fa48("708") ? true : stryMutAct_9fa48("707") ? hasAllLanguages : (stryCov_9fa48("707", "708", "709"), !hasAllLanguages)) {
                if (stryMutAct_9fa48("710")) {
                  {}
                } else {
                  stryCov_9fa48("710");
                  continue;
                }
              }
            }
          }

          // Check specialization requirements if specified
          if (stryMutAct_9fa48("713") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("712") ? false : stryMutAct_9fa48("711") ? true : (stryCov_9fa48("711", "712", "713"), query.specializations && (stryMutAct_9fa48("716") ? query.specializations.length <= 0 : stryMutAct_9fa48("715") ? query.specializations.length >= 0 : stryMutAct_9fa48("714") ? true : (stryCov_9fa48("714", "715", "716"), query.specializations.length > 0)))) {
            if (stryMutAct_9fa48("717")) {
              {}
            } else {
              stryCov_9fa48("717");
              const hasAllSpecializations = stryMutAct_9fa48("718") ? query.specializations.some(spec => profile.capabilities.specializations.includes(spec)) : (stryCov_9fa48("718"), query.specializations.every(stryMutAct_9fa48("719") ? () => undefined : (stryCov_9fa48("719"), spec => profile.capabilities.specializations.includes(spec))));
              if (stryMutAct_9fa48("722") ? false : stryMutAct_9fa48("721") ? true : stryMutAct_9fa48("720") ? hasAllSpecializations : (stryCov_9fa48("720", "721", "722"), !hasAllSpecializations)) {
                if (stryMutAct_9fa48("723")) {
                  {}
                } else {
                  stryCov_9fa48("723");
                  continue;
                }
              }
            }
          }

          // Check utilization threshold if specified
          if (stryMutAct_9fa48("726") ? query.maxUtilization !== undefined || profile.currentLoad.utilizationPercent > query.maxUtilization : stryMutAct_9fa48("725") ? false : stryMutAct_9fa48("724") ? true : (stryCov_9fa48("724", "725", "726"), (stryMutAct_9fa48("728") ? query.maxUtilization === undefined : stryMutAct_9fa48("727") ? true : (stryCov_9fa48("727", "728"), query.maxUtilization !== undefined)) && (stryMutAct_9fa48("731") ? profile.currentLoad.utilizationPercent <= query.maxUtilization : stryMutAct_9fa48("730") ? profile.currentLoad.utilizationPercent >= query.maxUtilization : stryMutAct_9fa48("729") ? true : (stryCov_9fa48("729", "730", "731"), profile.currentLoad.utilizationPercent > query.maxUtilization)))) {
            if (stryMutAct_9fa48("732")) {
              {}
            } else {
              stryCov_9fa48("732");
              continue;
            }
          }

          // Check minimum success rate if specified
          if (stryMutAct_9fa48("735") ? query.minSuccessRate !== undefined || profile.performanceHistory.successRate < query.minSuccessRate : stryMutAct_9fa48("734") ? false : stryMutAct_9fa48("733") ? true : (stryCov_9fa48("733", "734", "735"), (stryMutAct_9fa48("737") ? query.minSuccessRate === undefined : stryMutAct_9fa48("736") ? true : (stryCov_9fa48("736", "737"), query.minSuccessRate !== undefined)) && (stryMutAct_9fa48("740") ? profile.performanceHistory.successRate >= query.minSuccessRate : stryMutAct_9fa48("739") ? profile.performanceHistory.successRate <= query.minSuccessRate : stryMutAct_9fa48("738") ? true : (stryCov_9fa48("738", "739", "740"), profile.performanceHistory.successRate < query.minSuccessRate)))) {
            if (stryMutAct_9fa48("741")) {
              {}
            } else {
              stryCov_9fa48("741");
              continue;
            }
          }

          // Calculate match score
          const matchScore = this.calculateMatchScore(profile, query);
          const matchReason = this.explainMatchScore(profile, query, matchScore);
          results.push(stryMutAct_9fa48("742") ? {} : (stryCov_9fa48("742"), {
            agent: AgentProfileHelper.cloneProfile(profile),
            matchScore,
            matchReason
          }));
        }
      }

      // Sort by success rate (highest first), then by match score
      return stryMutAct_9fa48("743") ? results : (stryCov_9fa48("743"), results.sort((a, b) => {
        if (stryMutAct_9fa48("744")) {
          {}
        } else {
          stryCov_9fa48("744");
          const successDiff = stryMutAct_9fa48("745") ? b.agent.performanceHistory.successRate + a.agent.performanceHistory.successRate : (stryCov_9fa48("745"), b.agent.performanceHistory.successRate - a.agent.performanceHistory.successRate);
          if (stryMutAct_9fa48("749") ? Math.abs(successDiff) <= 0.01 : stryMutAct_9fa48("748") ? Math.abs(successDiff) >= 0.01 : stryMutAct_9fa48("747") ? false : stryMutAct_9fa48("746") ? true : (stryCov_9fa48("746", "747", "748", "749"), Math.abs(successDiff) > 0.01)) {
            if (stryMutAct_9fa48("750")) {
              {}
            } else {
              stryCov_9fa48("750");
              return successDiff;
            }
          }
          return stryMutAct_9fa48("751") ? b.matchScore + a.matchScore : (stryCov_9fa48("751"), b.matchScore - a.matchScore);
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
    if (stryMutAct_9fa48("752")) {
      {}
    } else {
      stryCov_9fa48("752");
      const profile = this.agents.get(agentId);
      if (stryMutAct_9fa48("755") ? false : stryMutAct_9fa48("754") ? true : stryMutAct_9fa48("753") ? profile : (stryCov_9fa48("753", "754", "755"), !profile)) {
        if (stryMutAct_9fa48("756")) {
          {}
        } else {
          stryCov_9fa48("756");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, `Agent with ID ${agentId} not found`, stryMutAct_9fa48("758") ? {} : (stryCov_9fa48("758"), {
            agentId
          }));
        }
      }
      try {
        if (stryMutAct_9fa48("759")) {
          {}
        } else {
          stryCov_9fa48("759");
          // Compute new running average (atomic operation)
          const newPerformanceHistory = AgentProfileHelper.updatePerformanceHistory(profile.performanceHistory, metrics);

          // Update profile with new performance history
          const updatedProfile: AgentProfile = stryMutAct_9fa48("760") ? {} : (stryCov_9fa48("760"), {
            ...profile,
            performanceHistory: newPerformanceHistory,
            lastActiveAt: new Date().toISOString()
          });

          // Atomically update in registry
          this.agents.set(agentId, updatedProfile);

          // Record performance metrics to database if enabled
          if (stryMutAct_9fa48("762") ? false : stryMutAct_9fa48("761") ? true : (stryCov_9fa48("761", "762"), this.dbClient)) {
            if (stryMutAct_9fa48("763")) {
              {}
            } else {
              stryCov_9fa48("763");
              try {
                if (stryMutAct_9fa48("764")) {
                  {}
                } else {
                  stryCov_9fa48("764");
                  await this.dbClient.recordPerformance(agentId, metrics);
                }
              } catch (error) {
                if (stryMutAct_9fa48("765")) {
                  {}
                } else {
                  stryCov_9fa48("765");
                  // Log database error but don't fail the operation
                  console.error(`Failed to record performance to database for agent ${agentId}:`, error);
                }
              }
            }
          }
          return AgentProfileHelper.cloneProfile(updatedProfile);
        }
      } catch (error) {
        if (stryMutAct_9fa48("767")) {
          {}
        } else {
          stryCov_9fa48("767");
          throw new RegistryError(RegistryErrorType.UPDATE_FAILED, `Failed to update performance for agent ${agentId}: ${(error as Error).message}`, stryMutAct_9fa48("769") ? {} : (stryCov_9fa48("769"), {
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
    if (stryMutAct_9fa48("770")) {
      {}
    } else {
      stryCov_9fa48("770");
      const profile = this.agents.get(agentId);
      if (stryMutAct_9fa48("773") ? false : stryMutAct_9fa48("772") ? true : stryMutAct_9fa48("771") ? profile : (stryCov_9fa48("771", "772", "773"), !profile)) {
        if (stryMutAct_9fa48("774")) {
          {}
        } else {
          stryCov_9fa48("774");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, `Agent with ID ${agentId} not found`, stryMutAct_9fa48("776") ? {} : (stryCov_9fa48("776"), {
            agentId
          }));
        }
      }
      const utilizationPercent = stryMutAct_9fa48("777") ? activeTasks / this.maxConcurrentTasksPerAgent / 100 : (stryCov_9fa48("777"), (stryMutAct_9fa48("778") ? activeTasks * this.maxConcurrentTasksPerAgent : (stryCov_9fa48("778"), activeTasks / this.maxConcurrentTasksPerAgent)) * 100);
      const updatedProfile: AgentProfile = stryMutAct_9fa48("779") ? {} : (stryCov_9fa48("779"), {
        ...profile,
        currentLoad: stryMutAct_9fa48("780") ? {} : (stryCov_9fa48("780"), {
          activeTasks,
          queuedTasks,
          utilizationPercent: stryMutAct_9fa48("781") ? Math.max(100, utilizationPercent) : (stryCov_9fa48("781"), Math.min(100, utilizationPercent))
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
    if (stryMutAct_9fa48("782")) {
      {}
    } else {
      stryCov_9fa48("782");
      const allAgents = Array.from(this.agents.values());
      const activeAgents = stryMutAct_9fa48("783") ? allAgents : (stryCov_9fa48("783"), allAgents.filter(stryMutAct_9fa48("784") ? () => undefined : (stryCov_9fa48("784"), a => stryMutAct_9fa48("788") ? a.currentLoad.activeTasks <= 0 : stryMutAct_9fa48("787") ? a.currentLoad.activeTasks >= 0 : stryMutAct_9fa48("786") ? false : stryMutAct_9fa48("785") ? true : (stryCov_9fa48("785", "786", "787", "788"), a.currentLoad.activeTasks > 0))));
      const idleAgents = stryMutAct_9fa48("789") ? allAgents : (stryCov_9fa48("789"), allAgents.filter(stryMutAct_9fa48("790") ? () => undefined : (stryCov_9fa48("790"), a => stryMutAct_9fa48("793") ? a.currentLoad.activeTasks !== 0 : stryMutAct_9fa48("792") ? false : stryMutAct_9fa48("791") ? true : (stryCov_9fa48("791", "792", "793"), a.currentLoad.activeTasks === 0))));
      const totalUtilization = allAgents.reduce(stryMutAct_9fa48("794") ? () => undefined : (stryCov_9fa48("794"), (sum, a) => stryMutAct_9fa48("795") ? sum - a.currentLoad.utilizationPercent : (stryCov_9fa48("795"), sum + a.currentLoad.utilizationPercent)), 0);
      const averageUtilization = (stryMutAct_9fa48("799") ? allAgents.length <= 0 : stryMutAct_9fa48("798") ? allAgents.length >= 0 : stryMutAct_9fa48("797") ? false : stryMutAct_9fa48("796") ? true : (stryCov_9fa48("796", "797", "798", "799"), allAgents.length > 0)) ? stryMutAct_9fa48("800") ? totalUtilization * allAgents.length : (stryCov_9fa48("800"), totalUtilization / allAgents.length) : 0;
      const totalSuccessRate = allAgents.reduce(stryMutAct_9fa48("801") ? () => undefined : (stryCov_9fa48("801"), (sum, a) => stryMutAct_9fa48("802") ? sum - a.performanceHistory.successRate : (stryCov_9fa48("802"), sum + a.performanceHistory.successRate)), 0);
      const averageSuccessRate = (stryMutAct_9fa48("806") ? allAgents.length <= 0 : stryMutAct_9fa48("805") ? allAgents.length >= 0 : stryMutAct_9fa48("804") ? false : stryMutAct_9fa48("803") ? true : (stryCov_9fa48("803", "804", "805", "806"), allAgents.length > 0)) ? stryMutAct_9fa48("807") ? totalSuccessRate * allAgents.length : (stryCov_9fa48("807"), totalSuccessRate / allAgents.length) : 0;
      return stryMutAct_9fa48("808") ? {} : (stryCov_9fa48("808"), {
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
    if (stryMutAct_9fa48("809")) {
      {}
    } else {
      stryCov_9fa48("809");
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
    if (stryMutAct_9fa48("810")) {
      {}
    } else {
      stryCov_9fa48("810");
      let score = 0.0;
      let weights = 0.0;

      // Task type match (required, so always contributes)
      stryMutAct_9fa48("811") ? score -= 0.3 : (stryCov_9fa48("811"), score += 0.3);
      stryMutAct_9fa48("812") ? weights -= 0.3 : (stryCov_9fa48("812"), weights += 0.3);

      // Language matches (if specified)
      if (stryMutAct_9fa48("815") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("814") ? false : stryMutAct_9fa48("813") ? true : (stryCov_9fa48("813", "814", "815"), query.languages && (stryMutAct_9fa48("818") ? query.languages.length <= 0 : stryMutAct_9fa48("817") ? query.languages.length >= 0 : stryMutAct_9fa48("816") ? true : (stryCov_9fa48("816", "817", "818"), query.languages.length > 0)))) {
        if (stryMutAct_9fa48("819")) {
          {}
        } else {
          stryCov_9fa48("819");
          const matchedLanguages = stryMutAct_9fa48("820") ? query.languages.length : (stryCov_9fa48("820"), query.languages.filter(stryMutAct_9fa48("821") ? () => undefined : (stryCov_9fa48("821"), lang => profile.capabilities.languages.includes(lang))).length);
          stryMutAct_9fa48("822") ? score -= matchedLanguages / query.languages.length * 0.3 : (stryCov_9fa48("822"), score += stryMutAct_9fa48("823") ? matchedLanguages / query.languages.length / 0.3 : (stryCov_9fa48("823"), (stryMutAct_9fa48("824") ? matchedLanguages * query.languages.length : (stryCov_9fa48("824"), matchedLanguages / query.languages.length)) * 0.3));
          stryMutAct_9fa48("825") ? weights -= 0.3 : (stryCov_9fa48("825"), weights += 0.3);
        }
      }

      // Specialization matches (if specified)
      if (stryMutAct_9fa48("828") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("827") ? false : stryMutAct_9fa48("826") ? true : (stryCov_9fa48("826", "827", "828"), query.specializations && (stryMutAct_9fa48("831") ? query.specializations.length <= 0 : stryMutAct_9fa48("830") ? query.specializations.length >= 0 : stryMutAct_9fa48("829") ? true : (stryCov_9fa48("829", "830", "831"), query.specializations.length > 0)))) {
        if (stryMutAct_9fa48("832")) {
          {}
        } else {
          stryCov_9fa48("832");
          const matchedSpecs = stryMutAct_9fa48("833") ? query.specializations.length : (stryCov_9fa48("833"), query.specializations.filter(stryMutAct_9fa48("834") ? () => undefined : (stryCov_9fa48("834"), spec => profile.capabilities.specializations.includes(spec))).length);
          stryMutAct_9fa48("835") ? score -= matchedSpecs / query.specializations.length * 0.2 : (stryCov_9fa48("835"), score += stryMutAct_9fa48("836") ? matchedSpecs / query.specializations.length / 0.2 : (stryCov_9fa48("836"), (stryMutAct_9fa48("837") ? matchedSpecs * query.specializations.length : (stryCov_9fa48("837"), matchedSpecs / query.specializations.length)) * 0.2));
          stryMutAct_9fa48("838") ? weights -= 0.2 : (stryCov_9fa48("838"), weights += 0.2);
        }
      }

      // Performance bonus
      stryMutAct_9fa48("839") ? score -= profile.performanceHistory.successRate * 0.2 : (stryCov_9fa48("839"), score += stryMutAct_9fa48("840") ? profile.performanceHistory.successRate / 0.2 : (stryCov_9fa48("840"), profile.performanceHistory.successRate * 0.2));
      stryMutAct_9fa48("841") ? weights -= 0.2 : (stryCov_9fa48("841"), weights += 0.2);
      return (stryMutAct_9fa48("845") ? weights <= 0 : stryMutAct_9fa48("844") ? weights >= 0 : stryMutAct_9fa48("843") ? false : stryMutAct_9fa48("842") ? true : (stryCov_9fa48("842", "843", "844", "845"), weights > 0)) ? stryMutAct_9fa48("846") ? score * weights : (stryCov_9fa48("846"), score / weights) : 0;
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
    if (stryMutAct_9fa48("847")) {
      {}
    } else {
      stryCov_9fa48("847");
      const reasons: string[] = [];
      reasons.push(`Supports ${query.taskType}`);
      if (stryMutAct_9fa48("852") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("851") ? false : stryMutAct_9fa48("850") ? true : (stryCov_9fa48("850", "851", "852"), query.languages && (stryMutAct_9fa48("855") ? query.languages.length <= 0 : stryMutAct_9fa48("854") ? query.languages.length >= 0 : stryMutAct_9fa48("853") ? true : (stryCov_9fa48("853", "854", "855"), query.languages.length > 0)))) {
        if (stryMutAct_9fa48("856")) {
          {}
        } else {
          stryCov_9fa48("856");
          reasons.push(`Languages: ${query.languages.join(", ")}`);
        }
      }
      if (stryMutAct_9fa48("861") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("860") ? false : stryMutAct_9fa48("859") ? true : (stryCov_9fa48("859", "860", "861"), query.specializations && (stryMutAct_9fa48("864") ? query.specializations.length <= 0 : stryMutAct_9fa48("863") ? query.specializations.length >= 0 : stryMutAct_9fa48("862") ? true : (stryCov_9fa48("862", "863", "864"), query.specializations.length > 0)))) {
        if (stryMutAct_9fa48("865")) {
          {}
        } else {
          stryCov_9fa48("865");
          reasons.push(`Specializations: ${query.specializations.join(", ")}`);
        }
      }
      reasons.push(`${(stryMutAct_9fa48("869") ? profile.performanceHistory.successRate / 100 : (stryCov_9fa48("869"), profile.performanceHistory.successRate * 100)).toFixed(1)}% success rate`);
      reasons.push(`${profile.currentLoad.utilizationPercent.toFixed(0)}% utilized`);
      return reasons.join("; ");
    }
  }

  /**
   * Start automatic cleanup of stale agents.
   */
  private startAutoCleanup(): void {
    if (stryMutAct_9fa48("872")) {
      {}
    } else {
      stryCov_9fa48("872");
      this.cleanupTimer = setInterval(() => {
        if (stryMutAct_9fa48("873")) {
          {}
        } else {
          stryCov_9fa48("873");
          this.cleanupStaleAgents();
        }
      }, this.config.cleanupIntervalMs);
    }
  }

  /**
   * Clean up stale agents (inactive beyond threshold).
   */
  private cleanupStaleAgents(): void {
    if (stryMutAct_9fa48("874")) {
      {}
    } else {
      stryCov_9fa48("874");
      const now = new Date().toISOString();
      const staleAgents: AgentId[] = [];
      const agents = Array.from(this.agents.entries());
      for (const [agentId, profile] of agents) {
        if (stryMutAct_9fa48("876")) {
          {}
        } else {
          stryCov_9fa48("876");
          if (stryMutAct_9fa48("878") ? false : stryMutAct_9fa48("877") ? true : (stryCov_9fa48("877", "878"), AgentProfileHelper.isStale(profile, this.config.staleAgentThresholdMs, now))) {
            if (stryMutAct_9fa48("879")) {
              {}
            } else {
              stryCov_9fa48("879");
              staleAgents.push(agentId);
            }
          }
        }
      }
      for (const agentId of staleAgents) {
        if (stryMutAct_9fa48("880")) {
          {}
        } else {
          stryCov_9fa48("880");
          this.agents.delete(agentId);
        }
      }
    }
  }

  /**
   * Shutdown the registry manager and cleanup resources.
   */
  async shutdown(): Promise<void> {
    if (stryMutAct_9fa48("881")) {
      {}
    } else {
      stryCov_9fa48("881");
      if (stryMutAct_9fa48("883") ? false : stryMutAct_9fa48("882") ? true : (stryCov_9fa48("882", "883"), this.cleanupTimer)) {
        if (stryMutAct_9fa48("884")) {
          {}
        } else {
          stryCov_9fa48("884");
          clearInterval(this.cleanupTimer);
        }
      }
    }
  }

  /**
   * Get the current availability status of an agent.
   *
   * @param profile - Agent profile
   * @returns Availability status string
   */
  private getAgentAvailabilityStatus(profile: AgentProfile): string {
    if (stryMutAct_9fa48("885")) {
      {}
    } else {
      stryCov_9fa48("885");
      // Determine status based on load and activity
      const utilization = profile.currentLoad.utilizationPercent;
      const activeTasks = profile.currentLoad.activeTasks;
      if (stryMutAct_9fa48("888") ? utilization >= 100 && activeTasks >= this.maxConcurrentTasksPerAgent : stryMutAct_9fa48("887") ? false : stryMutAct_9fa48("886") ? true : (stryCov_9fa48("886", "887", "888"), (stryMutAct_9fa48("891") ? utilization < 100 : stryMutAct_9fa48("890") ? utilization > 100 : stryMutAct_9fa48("889") ? false : (stryCov_9fa48("889", "890", "891"), utilization >= 100)) || (stryMutAct_9fa48("894") ? activeTasks < this.maxConcurrentTasksPerAgent : stryMutAct_9fa48("893") ? activeTasks > this.maxConcurrentTasksPerAgent : stryMutAct_9fa48("892") ? false : (stryCov_9fa48("892", "893", "894"), activeTasks >= this.maxConcurrentTasksPerAgent)))) {
        if (stryMutAct_9fa48("895")) {
          {}
        } else {
          stryCov_9fa48("895");
          return "offline";
        }
      } else if (stryMutAct_9fa48("899") ? utilization >= 50 && activeTasks > 0 : stryMutAct_9fa48("898") ? false : stryMutAct_9fa48("897") ? true : (stryCov_9fa48("897", "898", "899"), (stryMutAct_9fa48("902") ? utilization < 50 : stryMutAct_9fa48("901") ? utilization > 50 : stryMutAct_9fa48("900") ? false : (stryCov_9fa48("900", "901", "902"), utilization >= 50)) || (stryMutAct_9fa48("905") ? activeTasks <= 0 : stryMutAct_9fa48("904") ? activeTasks >= 0 : stryMutAct_9fa48("903") ? false : (stryCov_9fa48("903", "904", "905"), activeTasks > 0)))) {
        if (stryMutAct_9fa48("906")) {
          {}
        } else {
          stryCov_9fa48("906");
          return "busy";
        }
      } else {
        if (stryMutAct_9fa48("908")) {
          {}
        } else {
          stryCov_9fa48("908");
          return "available";
        }
      }
    }
  }

  /**
   * Calculate baseline performance metrics for a new agent.
   *
   * @param profile - Agent profile
   * @returns Baseline metrics for performance tracking
   */
  private calculateBaselineMetrics(profile: AgentProfile): {
    latencyMs: number;
    accuracy: number;
    costPerTask: number;
    reliability: number;
  } {
    if (stryMutAct_9fa48("910")) {
      {}
    } else {
      stryCov_9fa48("910");
      // Use model family to estimate baseline performance
      // These are conservative estimates based on typical performance
      const modelFamily = stryMutAct_9fa48("911") ? profile.modelFamily.toUpperCase() : (stryCov_9fa48("911"), profile.modelFamily.toLowerCase());
      let baselineLatency: number;
      let baselineAccuracy: number;
      let baselineCost: number;
      let baselineReliability: number;

      // Estimate based on model capabilities
      if (stryMutAct_9fa48("914") ? modelFamily.includes("gpt-4") && modelFamily.includes("claude-3") : stryMutAct_9fa48("913") ? false : stryMutAct_9fa48("912") ? true : (stryCov_9fa48("912", "913", "914"), modelFamily.includes("gpt-4") || modelFamily.includes("claude-3"))) {
        if (stryMutAct_9fa48("917")) {
          {}
        } else {
          stryCov_9fa48("917");
          baselineLatency = 1500; // 1.5s average response time
          baselineAccuracy = 0.92; // 92% accuracy
          baselineCost = 0.015; // $0.015 per task
          baselineReliability = 0.98; // 98% reliability
        }
      } else if (stryMutAct_9fa48("920") ? modelFamily.includes("gpt-3.5") && modelFamily.includes("claude-2") : stryMutAct_9fa48("919") ? false : stryMutAct_9fa48("918") ? true : (stryCov_9fa48("918", "919", "920"), modelFamily.includes("gpt-3.5") || modelFamily.includes("claude-2"))) {
        if (stryMutAct_9fa48("923")) {
          {}
        } else {
          stryCov_9fa48("923");
          baselineLatency = 1200; // 1.2s average response time
          baselineAccuracy = 0.88; // 88% accuracy
          baselineCost = 0.008; // $0.008 per task
          baselineReliability = 0.95; // 95% reliability
        }
      } else {
        if (stryMutAct_9fa48("924")) {
          {}
        } else {
          stryCov_9fa48("924");
          // Conservative defaults for unknown models
          baselineLatency = 2000; // 2s average response time
          baselineAccuracy = 0.8; // 80% accuracy
          baselineCost = 0.01; // $0.010 per task
          baselineReliability = 0.9; // 90% reliability
        }
      }

      // Adjust based on agent capabilities (more specialized = better performance)
      const capabilityBonus = stryMutAct_9fa48("925") ? Math.max(profile.capabilities.specializations.length * 0.02, 0.1) : (stryCov_9fa48("925"), Math.min(stryMutAct_9fa48("926") ? profile.capabilities.specializations.length / 0.02 : (stryCov_9fa48("926"), profile.capabilities.specializations.length * 0.02), 0.1));
      baselineAccuracy = stryMutAct_9fa48("927") ? Math.max(baselineAccuracy + capabilityBonus, 0.95) : (stryCov_9fa48("927"), Math.min(stryMutAct_9fa48("928") ? baselineAccuracy - capabilityBonus : (stryCov_9fa48("928"), baselineAccuracy + capabilityBonus), 0.95));

      // Language support bonus (more languages = slightly higher cost but better accuracy)
      const languageBonus = stryMutAct_9fa48("929") ? Math.max(profile.capabilities.languages.length * 0.01, 0.05) : (stryCov_9fa48("929"), Math.min(stryMutAct_9fa48("930") ? profile.capabilities.languages.length / 0.01 : (stryCov_9fa48("930"), profile.capabilities.languages.length * 0.01), 0.05));
      baselineAccuracy = stryMutAct_9fa48("931") ? Math.max(baselineAccuracy + languageBonus, 0.95) : (stryCov_9fa48("931"), Math.min(stryMutAct_9fa48("932") ? baselineAccuracy - languageBonus : (stryCov_9fa48("932"), baselineAccuracy + languageBonus), 0.95));
      stryMutAct_9fa48("933") ? baselineCost -= languageBonus * 0.002 : (stryCov_9fa48("933"), baselineCost += stryMutAct_9fa48("934") ? languageBonus / 0.002 : (stryCov_9fa48("934"), languageBonus * 0.002));
      return stryMutAct_9fa48("935") ? {} : (stryCov_9fa48("935"), {
        latencyMs: baselineLatency,
        accuracy: baselineAccuracy,
        costPerTask: baselineCost,
        reliability: baselineReliability
      });
    }
  }

  /**
   * Generate a unique ID for audit events
   */
  private generateId(): string {
    if (stryMutAct_9fa48("936")) {
      {}
    } else {
      stryCov_9fa48("936");
      return `audit_${Date.now()}_${stryMutAct_9fa48("938") ? Math.random().toString(36) : (stryCov_9fa48("938"), Math.random().toString(36).substr(2, 9))}`;
    }
  }
}