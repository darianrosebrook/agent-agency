/**
 * @fileoverview Security Layer for Agent Registry (ARBITER-001)
 *
 * Provides authentication, authorization, input validation, and multi-tenant isolation
 * for the agent registry system.
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
import { AgentId, AgentProfile, AgentQuery, PerformanceMetrics } from "../types/agent-registry";

/**
 * Security Context for operations
 */
export interface SecurityContext {
  /** Tenant identifier for multi-tenant isolation */
  tenantId: string;

  /** User identifier */
  userId: string;

  /** User roles */
  roles: string[];

  /** Request timestamp */
  requestedAt: Date;

  /** Request identifier for audit logging */
  requestId: string;

  /** IP address for rate limiting */
  ipAddress?: string;
}

/**
 * Security Configuration
 */
export interface SecurityConfig {
  /** Enable authentication */
  authenticationEnabled: boolean;

  /** Enable authorization */
  authorizationEnabled: boolean;

  /** Enable multi-tenant isolation */
  multiTenantEnabled: boolean;

  /** Enable audit logging */
  auditLoggingEnabled: boolean;

  /** Enable rate limiting */
  rateLimitingEnabled: boolean;

  /** Rate limit: requests per minute */
  rateLimitPerMinute: number;

  /** Allowed roles for agent registration */
  allowedRegistrationRoles: string[];

  /** Allowed roles for agent modification */
  allowedModificationRoles: string[];

  /** Allowed roles for agent deletion */
  allowedDeletionRoles: string[];
}

/**
 * Security Audit Entry
 */
export interface AuditEntry {
  id: string;
  tenantId: string;
  userId: string;
  operation: string;
  resource: string;
  resourceId: string;
  timestamp: Date;
  success: boolean;
  errorMessage?: string;
  metadata: Record<string, any>;
}

/**
 * Rate Limit Tracker
 */
interface RateLimitEntry {
  count: number;
  resetAt: Date;
}

/**
 * Security Error
 */
export class SecurityError extends Error {
  constructor(message: string, public code: string, public context?: SecurityContext) {
    super(message);
    this.name = stryMutAct_9fa48("370") ? "" : (stryCov_9fa48("370"), "SecurityError");
  }
}

/**
 * Agent Registry Security Layer
 *
 * Enforces authentication, authorization, input validation, and multi-tenant isolation.
 */
export class AgentRegistrySecurity {
  private config: SecurityConfig;
  private auditLog: AuditEntry[] = stryMutAct_9fa48("371") ? ["Stryker was here"] : (stryCov_9fa48("371"), []);
  private rateLimits: Map<string, RateLimitEntry> = new Map();
  constructor(config: Partial<SecurityConfig> = {}) {
    if (stryMutAct_9fa48("372")) {
      {}
    } else {
      stryCov_9fa48("372");
      this.config = stryMutAct_9fa48("373") ? {} : (stryCov_9fa48("373"), {
        authenticationEnabled: stryMutAct_9fa48("374") ? false : (stryCov_9fa48("374"), true),
        authorizationEnabled: stryMutAct_9fa48("375") ? false : (stryCov_9fa48("375"), true),
        multiTenantEnabled: stryMutAct_9fa48("376") ? false : (stryCov_9fa48("376"), true),
        auditLoggingEnabled: stryMutAct_9fa48("377") ? false : (stryCov_9fa48("377"), true),
        rateLimitingEnabled: stryMutAct_9fa48("378") ? false : (stryCov_9fa48("378"), true),
        rateLimitPerMinute: 100,
        allowedRegistrationRoles: stryMutAct_9fa48("379") ? [] : (stryCov_9fa48("379"), [stryMutAct_9fa48("380") ? "" : (stryCov_9fa48("380"), "admin"), stryMutAct_9fa48("381") ? "" : (stryCov_9fa48("381"), "agent-manager")]),
        allowedModificationRoles: stryMutAct_9fa48("382") ? [] : (stryCov_9fa48("382"), [stryMutAct_9fa48("383") ? "" : (stryCov_9fa48("383"), "admin"), stryMutAct_9fa48("384") ? "" : (stryCov_9fa48("384"), "agent-manager"), stryMutAct_9fa48("385") ? "" : (stryCov_9fa48("385"), "orchestrator")]),
        allowedDeletionRoles: stryMutAct_9fa48("386") ? [] : (stryCov_9fa48("386"), [stryMutAct_9fa48("387") ? "" : (stryCov_9fa48("387"), "admin")]),
        ...config
      });
    }
  }

  /**
   * Authenticate request and create security context
   */
  authenticateRequest(token: string, requestId: string, ipAddress?: string): SecurityContext {
    if (stryMutAct_9fa48("388")) {
      {}
    } else {
      stryCov_9fa48("388");
      if (stryMutAct_9fa48("391") ? false : stryMutAct_9fa48("390") ? true : stryMutAct_9fa48("389") ? this.config.authenticationEnabled : (stryCov_9fa48("389", "390", "391"), !this.config.authenticationEnabled)) {
        if (stryMutAct_9fa48("392")) {
          {}
        } else {
          stryCov_9fa48("392");
          return this.createAnonymousContext(requestId);
        }
      }

      // TODO: Implement actual token validation (JWT, OAuth, etc.)
      // For now, parse a simple token format: "tenant:user:roles"
      const parts = Buffer.from(token, stryMutAct_9fa48("393") ? "" : (stryCov_9fa48("393"), "base64")).toString(stryMutAct_9fa48("394") ? "" : (stryCov_9fa48("394"), "utf8")).split(stryMutAct_9fa48("395") ? "" : (stryCov_9fa48("395"), ":"));
      if (stryMutAct_9fa48("399") ? parts.length >= 3 : stryMutAct_9fa48("398") ? parts.length <= 3 : stryMutAct_9fa48("397") ? false : stryMutAct_9fa48("396") ? true : (stryCov_9fa48("396", "397", "398", "399"), parts.length < 3)) {
        if (stryMutAct_9fa48("400")) {
          {}
        } else {
          stryCov_9fa48("400");
          throw new SecurityError(stryMutAct_9fa48("401") ? "" : (stryCov_9fa48("401"), "Invalid authentication token"), stryMutAct_9fa48("402") ? "" : (stryCov_9fa48("402"), "INVALID_TOKEN"));
        }
      }
      return stryMutAct_9fa48("403") ? {} : (stryCov_9fa48("403"), {
        tenantId: parts[0],
        userId: parts[1],
        roles: parts[2].split(stryMutAct_9fa48("404") ? "" : (stryCov_9fa48("404"), ",")),
        requestedAt: new Date(),
        requestId,
        ipAddress
      });
    }
  }

  /**
   * Authorize registration operation
   */
  authorizeRegistration(context: SecurityContext): void {
    if (stryMutAct_9fa48("405")) {
      {}
    } else {
      stryCov_9fa48("405");
      if (stryMutAct_9fa48("408") ? false : stryMutAct_9fa48("407") ? true : stryMutAct_9fa48("406") ? this.config.authorizationEnabled : (stryCov_9fa48("406", "407", "408"), !this.config.authorizationEnabled)) {
        if (stryMutAct_9fa48("409")) {
          {}
        } else {
          stryCov_9fa48("409");
          return;
        }
      }
      const hasPermission = stryMutAct_9fa48("410") ? context.roles.every(role => this.config.allowedRegistrationRoles.includes(role)) : (stryCov_9fa48("410"), context.roles.some(stryMutAct_9fa48("411") ? () => undefined : (stryCov_9fa48("411"), role => this.config.allowedRegistrationRoles.includes(role))));
      if (stryMutAct_9fa48("414") ? false : stryMutAct_9fa48("413") ? true : stryMutAct_9fa48("412") ? hasPermission : (stryCov_9fa48("412", "413", "414"), !hasPermission)) {
        if (stryMutAct_9fa48("415")) {
          {}
        } else {
          stryCov_9fa48("415");
          this.logAuditEntry(stryMutAct_9fa48("416") ? {} : (stryCov_9fa48("416"), {
            ...this.createAuditEntry(context, stryMutAct_9fa48("417") ? "" : (stryCov_9fa48("417"), "register_agent"), stryMutAct_9fa48("418") ? "" : (stryCov_9fa48("418"), "agent"), stryMutAct_9fa48("419") ? "" : (stryCov_9fa48("419"), "unknown")),
            success: stryMutAct_9fa48("420") ? true : (stryCov_9fa48("420"), false),
            errorMessage: stryMutAct_9fa48("421") ? "" : (stryCov_9fa48("421"), "Insufficient permissions")
          }));
          throw new SecurityError(stryMutAct_9fa48("422") ? "" : (stryCov_9fa48("422"), "Insufficient permissions to register agents"), stryMutAct_9fa48("423") ? "" : (stryCov_9fa48("423"), "UNAUTHORIZED"), context);
        }
      }
    }
  }

  /**
   * Authorize modification operation
   */
  authorizeModification(context: SecurityContext): void {
    if (stryMutAct_9fa48("424")) {
      {}
    } else {
      stryCov_9fa48("424");
      if (stryMutAct_9fa48("427") ? false : stryMutAct_9fa48("426") ? true : stryMutAct_9fa48("425") ? this.config.authorizationEnabled : (stryCov_9fa48("425", "426", "427"), !this.config.authorizationEnabled)) {
        if (stryMutAct_9fa48("428")) {
          {}
        } else {
          stryCov_9fa48("428");
          return;
        }
      }
      const hasPermission = stryMutAct_9fa48("429") ? context.roles.every(role => this.config.allowedModificationRoles.includes(role)) : (stryCov_9fa48("429"), context.roles.some(stryMutAct_9fa48("430") ? () => undefined : (stryCov_9fa48("430"), role => this.config.allowedModificationRoles.includes(role))));
      if (stryMutAct_9fa48("433") ? false : stryMutAct_9fa48("432") ? true : stryMutAct_9fa48("431") ? hasPermission : (stryCov_9fa48("431", "432", "433"), !hasPermission)) {
        if (stryMutAct_9fa48("434")) {
          {}
        } else {
          stryCov_9fa48("434");
          throw new SecurityError(stryMutAct_9fa48("435") ? "" : (stryCov_9fa48("435"), "Insufficient permissions to modify agents"), stryMutAct_9fa48("436") ? "" : (stryCov_9fa48("436"), "UNAUTHORIZED"), context);
        }
      }
    }
  }

  /**
   * Authorize deletion operation
   */
  authorizeDeletion(context: SecurityContext): void {
    if (stryMutAct_9fa48("437")) {
      {}
    } else {
      stryCov_9fa48("437");
      if (stryMutAct_9fa48("440") ? false : stryMutAct_9fa48("439") ? true : stryMutAct_9fa48("438") ? this.config.authorizationEnabled : (stryCov_9fa48("438", "439", "440"), !this.config.authorizationEnabled)) {
        if (stryMutAct_9fa48("441")) {
          {}
        } else {
          stryCov_9fa48("441");
          return;
        }
      }
      const hasPermission = stryMutAct_9fa48("442") ? context.roles.every(role => this.config.allowedDeletionRoles.includes(role)) : (stryCov_9fa48("442"), context.roles.some(stryMutAct_9fa48("443") ? () => undefined : (stryCov_9fa48("443"), role => this.config.allowedDeletionRoles.includes(role))));
      if (stryMutAct_9fa48("446") ? false : stryMutAct_9fa48("445") ? true : stryMutAct_9fa48("444") ? hasPermission : (stryCov_9fa48("444", "445", "446"), !hasPermission)) {
        if (stryMutAct_9fa48("447")) {
          {}
        } else {
          stryCov_9fa48("447");
          throw new SecurityError(stryMutAct_9fa48("448") ? "" : (stryCov_9fa48("448"), "Insufficient permissions to delete agents"), stryMutAct_9fa48("449") ? "" : (stryCov_9fa48("449"), "UNAUTHORIZED"), context);
        }
      }
    }
  }

  /**
   * Check rate limit
   */
  checkRateLimit(context: SecurityContext): void {
    if (stryMutAct_9fa48("450")) {
      {}
    } else {
      stryCov_9fa48("450");
      if (stryMutAct_9fa48("453") ? false : stryMutAct_9fa48("452") ? true : stryMutAct_9fa48("451") ? this.config.rateLimitingEnabled : (stryCov_9fa48("451", "452", "453"), !this.config.rateLimitingEnabled)) {
        if (stryMutAct_9fa48("454")) {
          {}
        } else {
          stryCov_9fa48("454");
          return;
        }
      }
      const key = stryMutAct_9fa48("455") ? `` : (stryCov_9fa48("455"), `${context.tenantId}:${context.userId}`);
      const now = new Date();
      const limit = this.rateLimits.get(key);
      if (stryMutAct_9fa48("458") ? limit || limit.resetAt > now : stryMutAct_9fa48("457") ? false : stryMutAct_9fa48("456") ? true : (stryCov_9fa48("456", "457", "458"), limit && (stryMutAct_9fa48("461") ? limit.resetAt <= now : stryMutAct_9fa48("460") ? limit.resetAt >= now : stryMutAct_9fa48("459") ? true : (stryCov_9fa48("459", "460", "461"), limit.resetAt > now)))) {
        if (stryMutAct_9fa48("462")) {
          {}
        } else {
          stryCov_9fa48("462");
          stryMutAct_9fa48("463") ? limit.count-- : (stryCov_9fa48("463"), limit.count++);
          if (stryMutAct_9fa48("467") ? limit.count <= this.config.rateLimitPerMinute : stryMutAct_9fa48("466") ? limit.count >= this.config.rateLimitPerMinute : stryMutAct_9fa48("465") ? false : stryMutAct_9fa48("464") ? true : (stryCov_9fa48("464", "465", "466", "467"), limit.count > this.config.rateLimitPerMinute)) {
            if (stryMutAct_9fa48("468")) {
              {}
            } else {
              stryCov_9fa48("468");
              throw new SecurityError(stryMutAct_9fa48("469") ? `` : (stryCov_9fa48("469"), `Rate limit exceeded: ${this.config.rateLimitPerMinute} requests per minute`), stryMutAct_9fa48("470") ? "" : (stryCov_9fa48("470"), "RATE_LIMIT_EXCEEDED"), context);
            }
          }
        }
      } else {
        if (stryMutAct_9fa48("471")) {
          {}
        } else {
          stryCov_9fa48("471");
          this.rateLimits.set(key, stryMutAct_9fa48("472") ? {} : (stryCov_9fa48("472"), {
            count: 1,
            resetAt: new Date(stryMutAct_9fa48("473") ? now.getTime() - 60000 : (stryCov_9fa48("473"), now.getTime() + 60000)) // 1 minute from now
          }));
        }
      }
    }
  }

  /**
   * Validate agent profile data
   */
  validateAgentProfile(agent: Partial<AgentProfile>): void {
    if (stryMutAct_9fa48("474")) {
      {}
    } else {
      stryCov_9fa48("474");
      // ID validation
      if (stryMutAct_9fa48("477") ? !agent.id && typeof agent.id !== "string" : stryMutAct_9fa48("476") ? false : stryMutAct_9fa48("475") ? true : (stryCov_9fa48("475", "476", "477"), (stryMutAct_9fa48("478") ? agent.id : (stryCov_9fa48("478"), !agent.id)) || (stryMutAct_9fa48("480") ? typeof agent.id === "string" : stryMutAct_9fa48("479") ? false : (stryCov_9fa48("479", "480"), typeof agent.id !== (stryMutAct_9fa48("481") ? "" : (stryCov_9fa48("481"), "string")))))) {
        if (stryMutAct_9fa48("482")) {
          {}
        } else {
          stryCov_9fa48("482");
          throw new SecurityError(stryMutAct_9fa48("483") ? "" : (stryCov_9fa48("483"), "Agent ID is required and must be a string"), stryMutAct_9fa48("484") ? "" : (stryCov_9fa48("484"), "INVALID_INPUT"));
        }
      }
      if (stryMutAct_9fa48("488") ? agent.id.length <= 100 : stryMutAct_9fa48("487") ? agent.id.length >= 100 : stryMutAct_9fa48("486") ? false : stryMutAct_9fa48("485") ? true : (stryCov_9fa48("485", "486", "487", "488"), agent.id.length > 100)) {
        if (stryMutAct_9fa48("489")) {
          {}
        } else {
          stryCov_9fa48("489");
          throw new SecurityError(stryMutAct_9fa48("490") ? "" : (stryCov_9fa48("490"), "Agent ID too long (max 100 characters)"), stryMutAct_9fa48("491") ? "" : (stryCov_9fa48("491"), "INVALID_INPUT"));
        }
      }

      // Name validation
      if (stryMutAct_9fa48("494") ? !agent.name && typeof agent.name !== "string" : stryMutAct_9fa48("493") ? false : stryMutAct_9fa48("492") ? true : (stryCov_9fa48("492", "493", "494"), (stryMutAct_9fa48("495") ? agent.name : (stryCov_9fa48("495"), !agent.name)) || (stryMutAct_9fa48("497") ? typeof agent.name === "string" : stryMutAct_9fa48("496") ? false : (stryCov_9fa48("496", "497"), typeof agent.name !== (stryMutAct_9fa48("498") ? "" : (stryCov_9fa48("498"), "string")))))) {
        if (stryMutAct_9fa48("499")) {
          {}
        } else {
          stryCov_9fa48("499");
          throw new SecurityError(stryMutAct_9fa48("500") ? "" : (stryCov_9fa48("500"), "Agent name is required and must be a string"), stryMutAct_9fa48("501") ? "" : (stryCov_9fa48("501"), "INVALID_INPUT"));
        }
      }
      if (stryMutAct_9fa48("505") ? agent.name.length <= 200 : stryMutAct_9fa48("504") ? agent.name.length >= 200 : stryMutAct_9fa48("503") ? false : stryMutAct_9fa48("502") ? true : (stryCov_9fa48("502", "503", "504", "505"), agent.name.length > 200)) {
        if (stryMutAct_9fa48("506")) {
          {}
        } else {
          stryCov_9fa48("506");
          throw new SecurityError(stryMutAct_9fa48("507") ? "" : (stryCov_9fa48("507"), "Agent name too long (max 200 characters)"), stryMutAct_9fa48("508") ? "" : (stryCov_9fa48("508"), "INVALID_INPUT"));
        }
      }

      // Sanitize string inputs
      agent.id = this.sanitizeString(agent.id);
      agent.name = this.sanitizeString(agent.name);

      // Validate capabilities
      if (stryMutAct_9fa48("510") ? false : stryMutAct_9fa48("509") ? true : (stryCov_9fa48("509", "510"), agent.capabilities)) {
        if (stryMutAct_9fa48("511")) {
          {}
        } else {
          stryCov_9fa48("511");
          if (stryMutAct_9fa48("515") ? agent.capabilities.taskTypes.length <= 50 : stryMutAct_9fa48("514") ? agent.capabilities.taskTypes.length >= 50 : stryMutAct_9fa48("513") ? false : stryMutAct_9fa48("512") ? true : (stryCov_9fa48("512", "513", "514", "515"), agent.capabilities.taskTypes.length > 50)) {
            if (stryMutAct_9fa48("516")) {
              {}
            } else {
              stryCov_9fa48("516");
              throw new SecurityError(stryMutAct_9fa48("517") ? "" : (stryCov_9fa48("517"), "Too many task types (max 50)"), stryMutAct_9fa48("518") ? "" : (stryCov_9fa48("518"), "INVALID_INPUT"));
            }
          }
          if (stryMutAct_9fa48("522") ? agent.capabilities.languages.length <= 50 : stryMutAct_9fa48("521") ? agent.capabilities.languages.length >= 50 : stryMutAct_9fa48("520") ? false : stryMutAct_9fa48("519") ? true : (stryCov_9fa48("519", "520", "521", "522"), agent.capabilities.languages.length > 50)) {
            if (stryMutAct_9fa48("523")) {
              {}
            } else {
              stryCov_9fa48("523");
              throw new SecurityError(stryMutAct_9fa48("524") ? "" : (stryCov_9fa48("524"), "Too many languages (max 50)"), stryMutAct_9fa48("525") ? "" : (stryCov_9fa48("525"), "INVALID_INPUT"));
            }
          }
          if (stryMutAct_9fa48("529") ? agent.capabilities.specializations.length <= 50 : stryMutAct_9fa48("528") ? agent.capabilities.specializations.length >= 50 : stryMutAct_9fa48("527") ? false : stryMutAct_9fa48("526") ? true : (stryCov_9fa48("526", "527", "528", "529"), agent.capabilities.specializations.length > 50)) {
            if (stryMutAct_9fa48("530")) {
              {}
            } else {
              stryCov_9fa48("530");
              throw new SecurityError(stryMutAct_9fa48("531") ? "" : (stryCov_9fa48("531"), "Too many specializations (max 50)"), stryMutAct_9fa48("532") ? "" : (stryCov_9fa48("532"), "INVALID_INPUT"));
            }
          }
        }
      }
    }
  }

  /**
   * Validate performance metrics
   */
  validatePerformanceMetrics(metrics: PerformanceMetrics): void {
    if (stryMutAct_9fa48("533")) {
      {}
    } else {
      stryCov_9fa48("533");
      if (stryMutAct_9fa48("536") ? metrics.qualityScore < 0 && metrics.qualityScore > 1 : stryMutAct_9fa48("535") ? false : stryMutAct_9fa48("534") ? true : (stryCov_9fa48("534", "535", "536"), (stryMutAct_9fa48("539") ? metrics.qualityScore >= 0 : stryMutAct_9fa48("538") ? metrics.qualityScore <= 0 : stryMutAct_9fa48("537") ? false : (stryCov_9fa48("537", "538", "539"), metrics.qualityScore < 0)) || (stryMutAct_9fa48("542") ? metrics.qualityScore <= 1 : stryMutAct_9fa48("541") ? metrics.qualityScore >= 1 : stryMutAct_9fa48("540") ? false : (stryCov_9fa48("540", "541", "542"), metrics.qualityScore > 1)))) {
        if (stryMutAct_9fa48("543")) {
          {}
        } else {
          stryCov_9fa48("543");
          throw new SecurityError(stryMutAct_9fa48("544") ? "" : (stryCov_9fa48("544"), "Quality score must be between 0 and 1"), stryMutAct_9fa48("545") ? "" : (stryCov_9fa48("545"), "INVALID_INPUT"));
        }
      }
      if (stryMutAct_9fa48("548") ? metrics.latencyMs < 0 && metrics.latencyMs > 300000 : stryMutAct_9fa48("547") ? false : stryMutAct_9fa48("546") ? true : (stryCov_9fa48("546", "547", "548"), (stryMutAct_9fa48("551") ? metrics.latencyMs >= 0 : stryMutAct_9fa48("550") ? metrics.latencyMs <= 0 : stryMutAct_9fa48("549") ? false : (stryCov_9fa48("549", "550", "551"), metrics.latencyMs < 0)) || (stryMutAct_9fa48("554") ? metrics.latencyMs <= 300000 : stryMutAct_9fa48("553") ? metrics.latencyMs >= 300000 : stryMutAct_9fa48("552") ? false : (stryCov_9fa48("552", "553", "554"), metrics.latencyMs > 300000)))) {
        if (stryMutAct_9fa48("555")) {
          {}
        } else {
          stryCov_9fa48("555");
          throw new SecurityError(stryMutAct_9fa48("556") ? "" : (stryCov_9fa48("556"), "Latency must be between 0 and 300000ms (5 minutes)"), stryMutAct_9fa48("557") ? "" : (stryCov_9fa48("557"), "INVALID_INPUT"));
        }
      }
      if (stryMutAct_9fa48("560") ? metrics.tokensUsed !== undefined || metrics.tokensUsed < 0 || metrics.tokensUsed > 1000000 : stryMutAct_9fa48("559") ? false : stryMutAct_9fa48("558") ? true : (stryCov_9fa48("558", "559", "560"), (stryMutAct_9fa48("562") ? metrics.tokensUsed === undefined : stryMutAct_9fa48("561") ? true : (stryCov_9fa48("561", "562"), metrics.tokensUsed !== undefined)) && (stryMutAct_9fa48("564") ? metrics.tokensUsed < 0 && metrics.tokensUsed > 1000000 : stryMutAct_9fa48("563") ? true : (stryCov_9fa48("563", "564"), (stryMutAct_9fa48("567") ? metrics.tokensUsed >= 0 : stryMutAct_9fa48("566") ? metrics.tokensUsed <= 0 : stryMutAct_9fa48("565") ? false : (stryCov_9fa48("565", "566", "567"), metrics.tokensUsed < 0)) || (stryMutAct_9fa48("570") ? metrics.tokensUsed <= 1000000 : stryMutAct_9fa48("569") ? metrics.tokensUsed >= 1000000 : stryMutAct_9fa48("568") ? false : (stryCov_9fa48("568", "569", "570"), metrics.tokensUsed > 1000000)))))) {
        if (stryMutAct_9fa48("571")) {
          {}
        } else {
          stryCov_9fa48("571");
          throw new SecurityError(stryMutAct_9fa48("572") ? "" : (stryCov_9fa48("572"), "Tokens used must be between 0 and 1,000,000"), stryMutAct_9fa48("573") ? "" : (stryCov_9fa48("573"), "INVALID_INPUT"));
        }
      }
    }
  }

  /**
   * Validate query parameters
   */
  validateQuery(query: AgentQuery): void {
    if (stryMutAct_9fa48("574")) {
      {}
    } else {
      stryCov_9fa48("574");
      if (stryMutAct_9fa48("577") ? query.maxUtilization === undefined : stryMutAct_9fa48("576") ? false : stryMutAct_9fa48("575") ? true : (stryCov_9fa48("575", "576", "577"), query.maxUtilization !== undefined)) {
        if (stryMutAct_9fa48("578")) {
          {}
        } else {
          stryCov_9fa48("578");
          if (stryMutAct_9fa48("581") ? query.maxUtilization < 0 && query.maxUtilization > 100 : stryMutAct_9fa48("580") ? false : stryMutAct_9fa48("579") ? true : (stryCov_9fa48("579", "580", "581"), (stryMutAct_9fa48("584") ? query.maxUtilization >= 0 : stryMutAct_9fa48("583") ? query.maxUtilization <= 0 : stryMutAct_9fa48("582") ? false : (stryCov_9fa48("582", "583", "584"), query.maxUtilization < 0)) || (stryMutAct_9fa48("587") ? query.maxUtilization <= 100 : stryMutAct_9fa48("586") ? query.maxUtilization >= 100 : stryMutAct_9fa48("585") ? false : (stryCov_9fa48("585", "586", "587"), query.maxUtilization > 100)))) {
            if (stryMutAct_9fa48("588")) {
              {}
            } else {
              stryCov_9fa48("588");
              throw new SecurityError(stryMutAct_9fa48("589") ? "" : (stryCov_9fa48("589"), "Max utilization must be between 0 and 100"), stryMutAct_9fa48("590") ? "" : (stryCov_9fa48("590"), "INVALID_INPUT"));
            }
          }
        }
      }
      if (stryMutAct_9fa48("593") ? query.minSuccessRate === undefined : stryMutAct_9fa48("592") ? false : stryMutAct_9fa48("591") ? true : (stryCov_9fa48("591", "592", "593"), query.minSuccessRate !== undefined)) {
        if (stryMutAct_9fa48("594")) {
          {}
        } else {
          stryCov_9fa48("594");
          if (stryMutAct_9fa48("597") ? query.minSuccessRate < 0 && query.minSuccessRate > 1 : stryMutAct_9fa48("596") ? false : stryMutAct_9fa48("595") ? true : (stryCov_9fa48("595", "596", "597"), (stryMutAct_9fa48("600") ? query.minSuccessRate >= 0 : stryMutAct_9fa48("599") ? query.minSuccessRate <= 0 : stryMutAct_9fa48("598") ? false : (stryCov_9fa48("598", "599", "600"), query.minSuccessRate < 0)) || (stryMutAct_9fa48("603") ? query.minSuccessRate <= 1 : stryMutAct_9fa48("602") ? query.minSuccessRate >= 1 : stryMutAct_9fa48("601") ? false : (stryCov_9fa48("601", "602", "603"), query.minSuccessRate > 1)))) {
            if (stryMutAct_9fa48("604")) {
              {}
            } else {
              stryCov_9fa48("604");
              throw new SecurityError(stryMutAct_9fa48("605") ? "" : (stryCov_9fa48("605"), "Min success rate must be between 0 and 1"), stryMutAct_9fa48("606") ? "" : (stryCov_9fa48("606"), "INVALID_INPUT"));
            }
          }
        }
      }
      if (stryMutAct_9fa48("609") ? query.languages || query.languages.length > 20 : stryMutAct_9fa48("608") ? false : stryMutAct_9fa48("607") ? true : (stryCov_9fa48("607", "608", "609"), query.languages && (stryMutAct_9fa48("612") ? query.languages.length <= 20 : stryMutAct_9fa48("611") ? query.languages.length >= 20 : stryMutAct_9fa48("610") ? true : (stryCov_9fa48("610", "611", "612"), query.languages.length > 20)))) {
        if (stryMutAct_9fa48("613")) {
          {}
        } else {
          stryCov_9fa48("613");
          throw new SecurityError(stryMutAct_9fa48("614") ? "" : (stryCov_9fa48("614"), "Too many languages in query (max 20)"), stryMutAct_9fa48("615") ? "" : (stryCov_9fa48("615"), "INVALID_INPUT"));
        }
      }
    }
  }

  /**
   * Enforce tenant isolation
   */
  scopeToTenant(agentId: AgentId, context: SecurityContext): AgentId {
    if (stryMutAct_9fa48("616")) {
      {}
    } else {
      stryCov_9fa48("616");
      if (stryMutAct_9fa48("619") ? false : stryMutAct_9fa48("618") ? true : stryMutAct_9fa48("617") ? this.config.multiTenantEnabled : (stryCov_9fa48("617", "618", "619"), !this.config.multiTenantEnabled)) {
        if (stryMutAct_9fa48("620")) {
          {}
        } else {
          stryCov_9fa48("620");
          return agentId;
        }
      }

      // Ensure agent ID is scoped to tenant
      const scopedId = stryMutAct_9fa48("621") ? `` : (stryCov_9fa48("621"), `${context.tenantId}:${agentId}`);
      return scopedId as AgentId;
    }
  }

  /**
   * Extract agent ID from scoped ID
   */
  unscopeAgentId(scopedId: AgentId): string {
    if (stryMutAct_9fa48("622")) {
      {}
    } else {
      stryCov_9fa48("622");
      if (stryMutAct_9fa48("625") ? false : stryMutAct_9fa48("624") ? true : stryMutAct_9fa48("623") ? this.config.multiTenantEnabled : (stryCov_9fa48("623", "624", "625"), !this.config.multiTenantEnabled)) {
        if (stryMutAct_9fa48("626")) {
          {}
        } else {
          stryCov_9fa48("626");
          return scopedId;
        }
      }
      const parts = scopedId.split(stryMutAct_9fa48("627") ? "" : (stryCov_9fa48("627"), ":"));
      return (stryMutAct_9fa48("631") ? parts.length <= 1 : stryMutAct_9fa48("630") ? parts.length >= 1 : stryMutAct_9fa48("629") ? false : stryMutAct_9fa48("628") ? true : (stryCov_9fa48("628", "629", "630", "631"), parts.length > 1)) ? parts[1] : scopedId;
    }
  }

  /**
   * Verify agent belongs to tenant
   */
  verifyTenantOwnership(agentId: AgentId, context: SecurityContext): void {
    if (stryMutAct_9fa48("632")) {
      {}
    } else {
      stryCov_9fa48("632");
      if (stryMutAct_9fa48("635") ? false : stryMutAct_9fa48("634") ? true : stryMutAct_9fa48("633") ? this.config.multiTenantEnabled : (stryCov_9fa48("633", "634", "635"), !this.config.multiTenantEnabled)) {
        if (stryMutAct_9fa48("636")) {
          {}
        } else {
          stryCov_9fa48("636");
          return;
        }
      }
      if (stryMutAct_9fa48("639") ? false : stryMutAct_9fa48("638") ? true : stryMutAct_9fa48("637") ? agentId.startsWith(`${context.tenantId}:`) : (stryCov_9fa48("637", "638", "639"), !(stryMutAct_9fa48("640") ? agentId.endsWith(`${context.tenantId}:`) : (stryCov_9fa48("640"), agentId.startsWith(stryMutAct_9fa48("641") ? `` : (stryCov_9fa48("641"), `${context.tenantId}:`)))))) {
        if (stryMutAct_9fa48("642")) {
          {}
        } else {
          stryCov_9fa48("642");
          throw new SecurityError(stryMutAct_9fa48("643") ? "" : (stryCov_9fa48("643"), "Agent does not belong to your tenant"), stryMutAct_9fa48("644") ? "" : (stryCov_9fa48("644"), "UNAUTHORIZED"), context);
        }
      }
    }
  }

  /**
   * Log audit entry
   */
  logAuditEntry(entry: AuditEntry): void {
    if (stryMutAct_9fa48("645")) {
      {}
    } else {
      stryCov_9fa48("645");
      if (stryMutAct_9fa48("648") ? false : stryMutAct_9fa48("647") ? true : stryMutAct_9fa48("646") ? this.config.auditLoggingEnabled : (stryCov_9fa48("646", "647", "648"), !this.config.auditLoggingEnabled)) {
        if (stryMutAct_9fa48("649")) {
          {}
        } else {
          stryCov_9fa48("649");
          return;
        }
      }
      this.auditLog.push(entry);

      // TODO: Persist to database or external audit system
      console.log(stryMutAct_9fa48("650") ? `` : (stryCov_9fa48("650"), `[AUDIT] ${entry.operation} on ${entry.resource}:${entry.resourceId} by ${entry.userId} - ${entry.success ? stryMutAct_9fa48("651") ? "" : (stryCov_9fa48("651"), "SUCCESS") : stryMutAct_9fa48("652") ? "" : (stryCov_9fa48("652"), "FAILURE")}`));
    }
  }

  /**
   * Get audit log
   */
  getAuditLog(tenantId?: string, limit: number = 100): AuditEntry[] {
    if (stryMutAct_9fa48("653")) {
      {}
    } else {
      stryCov_9fa48("653");
      let filtered = this.auditLog;
      if (stryMutAct_9fa48("655") ? false : stryMutAct_9fa48("654") ? true : (stryCov_9fa48("654", "655"), tenantId)) {
        if (stryMutAct_9fa48("656")) {
          {}
        } else {
          stryCov_9fa48("656");
          filtered = stryMutAct_9fa48("657") ? filtered : (stryCov_9fa48("657"), filtered.filter(stryMutAct_9fa48("658") ? () => undefined : (stryCov_9fa48("658"), entry => stryMutAct_9fa48("661") ? entry.tenantId !== tenantId : stryMutAct_9fa48("660") ? false : stryMutAct_9fa48("659") ? true : (stryCov_9fa48("659", "660", "661"), entry.tenantId === tenantId))));
        }
      }
      return stryMutAct_9fa48("662") ? filtered : (stryCov_9fa48("662"), filtered.slice(stryMutAct_9fa48("663") ? +limit : (stryCov_9fa48("663"), -limit)));
    }
  }

  /**
   * Create audit entry
   */
  private createAuditEntry(context: SecurityContext, operation: string, resource: string, resourceId: string): Omit<AuditEntry, "success"> {
    if (stryMutAct_9fa48("664")) {
      {}
    } else {
      stryCov_9fa48("664");
      return stryMutAct_9fa48("665") ? {} : (stryCov_9fa48("665"), {
        id: stryMutAct_9fa48("666") ? `` : (stryCov_9fa48("666"), `audit-${Date.now()}-${stryMutAct_9fa48("667") ? Math.random().toString(36) : (stryCov_9fa48("667"), Math.random().toString(36).substr(2, 9))}`),
        tenantId: context.tenantId,
        userId: context.userId,
        operation,
        resource,
        resourceId,
        timestamp: new Date(),
        metadata: stryMutAct_9fa48("668") ? {} : (stryCov_9fa48("668"), {
          requestId: context.requestId,
          roles: context.roles,
          ipAddress: context.ipAddress
        })
      });
    }
  }

  /**
   * Create anonymous context for non-authenticated mode
   */
  private createAnonymousContext(requestId: string): SecurityContext {
    if (stryMutAct_9fa48("669")) {
      {}
    } else {
      stryCov_9fa48("669");
      return stryMutAct_9fa48("670") ? {} : (stryCov_9fa48("670"), {
        tenantId: stryMutAct_9fa48("671") ? "" : (stryCov_9fa48("671"), "default"),
        userId: stryMutAct_9fa48("672") ? "" : (stryCov_9fa48("672"), "anonymous"),
        roles: stryMutAct_9fa48("673") ? [] : (stryCov_9fa48("673"), [stryMutAct_9fa48("674") ? "" : (stryCov_9fa48("674"), "public")]),
        requestedAt: new Date(),
        requestId
      });
    }
  }

  /**
   * Sanitize string input to prevent injection attacks
   */
  private sanitizeString(input: string): string {
    if (stryMutAct_9fa48("675")) {
      {}
    } else {
      stryCov_9fa48("675");
      // Remove null bytes
      let sanitized = input.replace(/\0/g, stryMutAct_9fa48("676") ? "Stryker was here!" : (stryCov_9fa48("676"), ""));

      // Trim whitespace
      sanitized = stryMutAct_9fa48("677") ? sanitized : (stryCov_9fa48("677"), sanitized.trim());

      // Remove control characters
      // eslint-disable-next-line no-control-regex
      sanitized = sanitized.replace(stryMutAct_9fa48("678") ? /[^\x00-\x1F\x7F]/g : (stryCov_9fa48("678"), /[\x00-\x1F\x7F]/g), stryMutAct_9fa48("679") ? "Stryker was here!" : (stryCov_9fa48("679"), ""));
      return sanitized;
    }
  }
}

/**
 * Secure Agent Registry Wrapper
 *
 * Wraps AgentRegistryManager with security enforcement.
 */
export class SecureAgentRegistry {
  private security: AgentRegistrySecurity;
  constructor(private registry: any,
  // AgentRegistryManager
  securityConfig?: Partial<SecurityConfig>) {
    if (stryMutAct_9fa48("680")) {
      {}
    } else {
      stryCov_9fa48("680");
      this.security = new AgentRegistrySecurity(securityConfig);
    }
  }

  /**
   * Register agent with security checks
   */
  async registerAgent(agent: AgentProfile, context: SecurityContext): Promise<AgentProfile> {
    if (stryMutAct_9fa48("681")) {
      {}
    } else {
      stryCov_9fa48("681");
      // Rate limiting
      this.security.checkRateLimit(context);

      // Authorization
      this.security.authorizeRegistration(context);

      // Input validation
      this.security.validateAgentProfile(agent);

      // Tenant scoping
      const scopedAgent = stryMutAct_9fa48("682") ? {} : (stryCov_9fa48("682"), {
        ...agent,
        id: this.security.scopeToTenant(agent.id, context)
      });
      try {
        if (stryMutAct_9fa48("683")) {
          {}
        } else {
          stryCov_9fa48("683");
          const result = await this.registry.registerAgent(scopedAgent);

          // Audit log
          this.security.logAuditEntry(stryMutAct_9fa48("684") ? {} : (stryCov_9fa48("684"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("685") ? "" : (stryCov_9fa48("685"), "register_agent"), stryMutAct_9fa48("686") ? "" : (stryCov_9fa48("686"), "agent"), agent.id),
            success: stryMutAct_9fa48("687") ? false : (stryCov_9fa48("687"), true)
          }));
          return result;
        }
      } catch (error) {
        if (stryMutAct_9fa48("688")) {
          {}
        } else {
          stryCov_9fa48("688");
          // Audit log failure
          this.security.logAuditEntry(stryMutAct_9fa48("689") ? {} : (stryCov_9fa48("689"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("690") ? "" : (stryCov_9fa48("690"), "register_agent"), stryMutAct_9fa48("691") ? "" : (stryCov_9fa48("691"), "agent"), agent.id),
            success: stryMutAct_9fa48("692") ? true : (stryCov_9fa48("692"), false),
            errorMessage: error instanceof Error ? error.message : String(error)
          }));
          throw error;
        }
      }
    }
  }

  /**
   * Get agent with tenant isolation
   */
  async getAgent(agentId: AgentId, context: SecurityContext): Promise<AgentProfile | null> {
    if (stryMutAct_9fa48("693")) {
      {}
    } else {
      stryCov_9fa48("693");
      // Rate limiting
      this.security.checkRateLimit(context);

      // Tenant scoping
      const scopedId = this.security.scopeToTenant(agentId, context);
      const result = await this.registry.getProfile(scopedId);

      // Audit log
      this.security.logAuditEntry(stryMutAct_9fa48("694") ? {} : (stryCov_9fa48("694"), {
        ...this.createBaseAuditEntry(context, stryMutAct_9fa48("695") ? "" : (stryCov_9fa48("695"), "get_agent"), stryMutAct_9fa48("696") ? "" : (stryCov_9fa48("696"), "agent"), agentId),
        success: stryMutAct_9fa48("697") ? false : (stryCov_9fa48("697"), true)
      }));
      return result;
    }
  }

  /**
   * Query agents with tenant isolation
   */
  async queryAgents(query: AgentQuery, context: SecurityContext): Promise<AgentProfile[]> {
    if (stryMutAct_9fa48("698")) {
      {}
    } else {
      stryCov_9fa48("698");
      // Rate limiting
      this.security.checkRateLimit(context);

      // Input validation
      this.security.validateQuery(query);

      // Get agents (registry will handle capability filtering)
      const results = await this.registry.getAgentsByCapability(query);

      // Filter by tenant
      const tenantResults = stryMutAct_9fa48("699") ? results : (stryCov_9fa48("699"), results.filter(stryMutAct_9fa48("700") ? () => undefined : (stryCov_9fa48("700"), (result: any) => stryMutAct_9fa48("701") ? result.agent.id.endsWith(`${context.tenantId}:`) : (stryCov_9fa48("701"), result.agent.id.startsWith(stryMutAct_9fa48("702") ? `` : (stryCov_9fa48("702"), `${context.tenantId}:`))))));

      // Audit log
      this.security.logAuditEntry(stryMutAct_9fa48("703") ? {} : (stryCov_9fa48("703"), {
        ...this.createBaseAuditEntry(context, stryMutAct_9fa48("704") ? "" : (stryCov_9fa48("704"), "query_agents"), stryMutAct_9fa48("705") ? "" : (stryCov_9fa48("705"), "agent"), stryMutAct_9fa48("706") ? "" : (stryCov_9fa48("706"), "query")),
        success: stryMutAct_9fa48("707") ? false : (stryCov_9fa48("707"), true),
        metadata: stryMutAct_9fa48("708") ? {} : (stryCov_9fa48("708"), {
          requestId: context.requestId,
          roles: context.roles,
          ipAddress: context.ipAddress,
          queryParams: query,
          resultCount: tenantResults.length
        })
      }));
      return tenantResults.map(stryMutAct_9fa48("709") ? () => undefined : (stryCov_9fa48("709"), (r: any) => r.agent));
    }
  }

  /**
   * Update performance with security checks
   */
  async updatePerformance(agentId: AgentId, metrics: PerformanceMetrics, context: SecurityContext): Promise<AgentProfile> {
    if (stryMutAct_9fa48("710")) {
      {}
    } else {
      stryCov_9fa48("710");
      // Rate limiting
      this.security.checkRateLimit(context);

      // Authorization
      this.security.authorizeModification(context);

      // Input validation
      this.security.validatePerformanceMetrics(metrics);

      // Tenant verification
      const scopedId = this.security.scopeToTenant(agentId, context);
      this.security.verifyTenantOwnership(scopedId, context);
      try {
        if (stryMutAct_9fa48("711")) {
          {}
        } else {
          stryCov_9fa48("711");
          const result = await this.registry.updatePerformance(scopedId, metrics);

          // Audit log
          this.security.logAuditEntry(stryMutAct_9fa48("712") ? {} : (stryCov_9fa48("712"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("713") ? "" : (stryCov_9fa48("713"), "update_performance"), stryMutAct_9fa48("714") ? "" : (stryCov_9fa48("714"), "agent"), agentId),
            success: stryMutAct_9fa48("715") ? false : (stryCov_9fa48("715"), true),
            metadata: stryMutAct_9fa48("716") ? {} : (stryCov_9fa48("716"), {
              requestId: context.requestId,
              roles: context.roles,
              ipAddress: context.ipAddress,
              metrics
            })
          }));
          return result;
        }
      } catch (error) {
        if (stryMutAct_9fa48("717")) {
          {}
        } else {
          stryCov_9fa48("717");
          // Audit log failure
          this.security.logAuditEntry(stryMutAct_9fa48("718") ? {} : (stryCov_9fa48("718"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("719") ? "" : (stryCov_9fa48("719"), "update_performance"), stryMutAct_9fa48("720") ? "" : (stryCov_9fa48("720"), "agent"), agentId),
            success: stryMutAct_9fa48("721") ? true : (stryCov_9fa48("721"), false),
            errorMessage: error instanceof Error ? error.message : String(error)
          }));
          throw error;
        }
      }
    }
  }

  /**
   * Unregister agent with security checks
   */
  async unregisterAgent(agentId: AgentId, context: SecurityContext): Promise<boolean> {
    if (stryMutAct_9fa48("722")) {
      {}
    } else {
      stryCov_9fa48("722");
      // Rate limiting
      this.security.checkRateLimit(context);

      // Authorization
      this.security.authorizeDeletion(context);

      // Tenant verification
      const scopedId = this.security.scopeToTenant(agentId, context);
      this.security.verifyTenantOwnership(scopedId, context);
      try {
        if (stryMutAct_9fa48("723")) {
          {}
        } else {
          stryCov_9fa48("723");
          const result = await this.registry.unregisterAgent(scopedId);

          // Audit log
          this.security.logAuditEntry(stryMutAct_9fa48("724") ? {} : (stryCov_9fa48("724"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("725") ? "" : (stryCov_9fa48("725"), "unregister_agent"), stryMutAct_9fa48("726") ? "" : (stryCov_9fa48("726"), "agent"), agentId),
            success: stryMutAct_9fa48("727") ? false : (stryCov_9fa48("727"), true)
          }));
          return result;
        }
      } catch (error) {
        if (stryMutAct_9fa48("728")) {
          {}
        } else {
          stryCov_9fa48("728");
          // Audit log failure
          this.security.logAuditEntry(stryMutAct_9fa48("729") ? {} : (stryCov_9fa48("729"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("730") ? "" : (stryCov_9fa48("730"), "unregister_agent"), stryMutAct_9fa48("731") ? "" : (stryCov_9fa48("731"), "agent"), agentId),
            success: stryMutAct_9fa48("732") ? true : (stryCov_9fa48("732"), false),
            errorMessage: error instanceof Error ? error.message : String(error)
          }));
          throw error;
        }
      }
    }
  }

  /**
   * Get audit log for tenant
   */
  getAuditLog(context: SecurityContext, limit?: number): AuditEntry[] {
    if (stryMutAct_9fa48("733")) {
      {}
    } else {
      stryCov_9fa48("733");
      return this.security.getAuditLog(context.tenantId, limit);
    }
  }

  /**
   * Create base audit entry
   */
  private createBaseAuditEntry(context: SecurityContext, operation: string, resource: string, resourceId: string): Omit<AuditEntry, "success"> {
    if (stryMutAct_9fa48("734")) {
      {}
    } else {
      stryCov_9fa48("734");
      return stryMutAct_9fa48("735") ? {} : (stryCov_9fa48("735"), {
        id: stryMutAct_9fa48("736") ? `` : (stryCov_9fa48("736"), `audit-${Date.now()}-${stryMutAct_9fa48("737") ? Math.random().toString(36) : (stryCov_9fa48("737"), Math.random().toString(36).substr(2, 9))}`),
        tenantId: context.tenantId,
        userId: context.userId,
        operation,
        resource,
        resourceId,
        timestamp: new Date(),
        metadata: stryMutAct_9fa48("738") ? {} : (stryCov_9fa48("738"), {
          requestId: context.requestId,
          roles: context.roles,
          ipAddress: context.ipAddress
        })
      });
    }
  }
}