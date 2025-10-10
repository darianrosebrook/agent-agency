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
    this.name = stryMutAct_9fa48("638") ? "" : (stryCov_9fa48("638"), "SecurityError");
  }
}

/**
 * Agent Registry Security Layer
 *
 * Enforces authentication, authorization, input validation, and multi-tenant isolation.
 */
export class AgentRegistrySecurity {
  private config: SecurityConfig;
  private auditLog: AuditEntry[] = stryMutAct_9fa48("639") ? ["Stryker was here"] : (stryCov_9fa48("639"), []);
  private rateLimits: Map<string, RateLimitEntry> = new Map();
  constructor(config: Partial<SecurityConfig> = {}) {
    if (stryMutAct_9fa48("640")) {
      {}
    } else {
      stryCov_9fa48("640");
      this.config = stryMutAct_9fa48("641") ? {} : (stryCov_9fa48("641"), {
        authenticationEnabled: stryMutAct_9fa48("642") ? false : (stryCov_9fa48("642"), true),
        authorizationEnabled: stryMutAct_9fa48("643") ? false : (stryCov_9fa48("643"), true),
        multiTenantEnabled: stryMutAct_9fa48("644") ? false : (stryCov_9fa48("644"), true),
        auditLoggingEnabled: stryMutAct_9fa48("645") ? false : (stryCov_9fa48("645"), true),
        rateLimitingEnabled: stryMutAct_9fa48("646") ? false : (stryCov_9fa48("646"), true),
        rateLimitPerMinute: 100,
        allowedRegistrationRoles: stryMutAct_9fa48("647") ? [] : (stryCov_9fa48("647"), [stryMutAct_9fa48("648") ? "" : (stryCov_9fa48("648"), "admin"), stryMutAct_9fa48("649") ? "" : (stryCov_9fa48("649"), "agent-manager")]),
        allowedModificationRoles: stryMutAct_9fa48("650") ? [] : (stryCov_9fa48("650"), [stryMutAct_9fa48("651") ? "" : (stryCov_9fa48("651"), "admin"), stryMutAct_9fa48("652") ? "" : (stryCov_9fa48("652"), "agent-manager"), stryMutAct_9fa48("653") ? "" : (stryCov_9fa48("653"), "orchestrator")]),
        allowedDeletionRoles: stryMutAct_9fa48("654") ? [] : (stryCov_9fa48("654"), [stryMutAct_9fa48("655") ? "" : (stryCov_9fa48("655"), "admin")]),
        ...config
      });
    }
  }

  /**
   * Authenticate request and create security context
   */
  authenticateRequest(token: string, requestId: string, ipAddress?: string): SecurityContext {
    if (stryMutAct_9fa48("656")) {
      {}
    } else {
      stryCov_9fa48("656");
      if (stryMutAct_9fa48("659") ? false : stryMutAct_9fa48("658") ? true : stryMutAct_9fa48("657") ? this.config.authenticationEnabled : (stryCov_9fa48("657", "658", "659"), !this.config.authenticationEnabled)) {
        if (stryMutAct_9fa48("660")) {
          {}
        } else {
          stryCov_9fa48("660");
          return this.createAnonymousContext(requestId);
        }
      }

      // TODO: Implement actual token validation (JWT, OAuth, etc.)
      // For now, parse a simple token format: "tenant:user:roles"
      const parts = Buffer.from(token, stryMutAct_9fa48("661") ? "" : (stryCov_9fa48("661"), "base64")).toString(stryMutAct_9fa48("662") ? "" : (stryCov_9fa48("662"), "utf8")).split(stryMutAct_9fa48("663") ? "" : (stryCov_9fa48("663"), ":"));
      if (stryMutAct_9fa48("667") ? parts.length >= 3 : stryMutAct_9fa48("666") ? parts.length <= 3 : stryMutAct_9fa48("665") ? false : stryMutAct_9fa48("664") ? true : (stryCov_9fa48("664", "665", "666", "667"), parts.length < 3)) {
        if (stryMutAct_9fa48("668")) {
          {}
        } else {
          stryCov_9fa48("668");
          throw new SecurityError(stryMutAct_9fa48("669") ? "" : (stryCov_9fa48("669"), "Invalid authentication token"), stryMutAct_9fa48("670") ? "" : (stryCov_9fa48("670"), "INVALID_TOKEN"));
        }
      }
      return stryMutAct_9fa48("671") ? {} : (stryCov_9fa48("671"), {
        tenantId: parts[0],
        userId: parts[1],
        roles: parts[2].split(stryMutAct_9fa48("672") ? "" : (stryCov_9fa48("672"), ",")),
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
    if (stryMutAct_9fa48("673")) {
      {}
    } else {
      stryCov_9fa48("673");
      if (stryMutAct_9fa48("676") ? false : stryMutAct_9fa48("675") ? true : stryMutAct_9fa48("674") ? this.config.authorizationEnabled : (stryCov_9fa48("674", "675", "676"), !this.config.authorizationEnabled)) {
        if (stryMutAct_9fa48("677")) {
          {}
        } else {
          stryCov_9fa48("677");
          return;
        }
      }
      const hasPermission = stryMutAct_9fa48("678") ? context.roles.every(role => this.config.allowedRegistrationRoles.includes(role)) : (stryCov_9fa48("678"), context.roles.some(stryMutAct_9fa48("679") ? () => undefined : (stryCov_9fa48("679"), role => this.config.allowedRegistrationRoles.includes(role))));
      if (stryMutAct_9fa48("682") ? false : stryMutAct_9fa48("681") ? true : stryMutAct_9fa48("680") ? hasPermission : (stryCov_9fa48("680", "681", "682"), !hasPermission)) {
        if (stryMutAct_9fa48("683")) {
          {}
        } else {
          stryCov_9fa48("683");
          this.logAuditEntry(stryMutAct_9fa48("684") ? {} : (stryCov_9fa48("684"), {
            ...this.createAuditEntry(context, stryMutAct_9fa48("685") ? "" : (stryCov_9fa48("685"), "register_agent"), stryMutAct_9fa48("686") ? "" : (stryCov_9fa48("686"), "agent"), stryMutAct_9fa48("687") ? "" : (stryCov_9fa48("687"), "unknown")),
            success: stryMutAct_9fa48("688") ? true : (stryCov_9fa48("688"), false),
            errorMessage: stryMutAct_9fa48("689") ? "" : (stryCov_9fa48("689"), "Insufficient permissions")
          }));
          throw new SecurityError(stryMutAct_9fa48("690") ? "" : (stryCov_9fa48("690"), "Insufficient permissions to register agents"), stryMutAct_9fa48("691") ? "" : (stryCov_9fa48("691"), "UNAUTHORIZED"), context);
        }
      }
    }
  }

  /**
   * Authorize modification operation
   */
  authorizeModification(context: SecurityContext): void {
    if (stryMutAct_9fa48("692")) {
      {}
    } else {
      stryCov_9fa48("692");
      if (stryMutAct_9fa48("695") ? false : stryMutAct_9fa48("694") ? true : stryMutAct_9fa48("693") ? this.config.authorizationEnabled : (stryCov_9fa48("693", "694", "695"), !this.config.authorizationEnabled)) {
        if (stryMutAct_9fa48("696")) {
          {}
        } else {
          stryCov_9fa48("696");
          return;
        }
      }
      const hasPermission = stryMutAct_9fa48("697") ? context.roles.every(role => this.config.allowedModificationRoles.includes(role)) : (stryCov_9fa48("697"), context.roles.some(stryMutAct_9fa48("698") ? () => undefined : (stryCov_9fa48("698"), role => this.config.allowedModificationRoles.includes(role))));
      if (stryMutAct_9fa48("701") ? false : stryMutAct_9fa48("700") ? true : stryMutAct_9fa48("699") ? hasPermission : (stryCov_9fa48("699", "700", "701"), !hasPermission)) {
        if (stryMutAct_9fa48("702")) {
          {}
        } else {
          stryCov_9fa48("702");
          throw new SecurityError(stryMutAct_9fa48("703") ? "" : (stryCov_9fa48("703"), "Insufficient permissions to modify agents"), stryMutAct_9fa48("704") ? "" : (stryCov_9fa48("704"), "UNAUTHORIZED"), context);
        }
      }
    }
  }

  /**
   * Authorize deletion operation
   */
  authorizeDeletion(context: SecurityContext): void {
    if (stryMutAct_9fa48("705")) {
      {}
    } else {
      stryCov_9fa48("705");
      if (stryMutAct_9fa48("708") ? false : stryMutAct_9fa48("707") ? true : stryMutAct_9fa48("706") ? this.config.authorizationEnabled : (stryCov_9fa48("706", "707", "708"), !this.config.authorizationEnabled)) {
        if (stryMutAct_9fa48("709")) {
          {}
        } else {
          stryCov_9fa48("709");
          return;
        }
      }
      const hasPermission = stryMutAct_9fa48("710") ? context.roles.every(role => this.config.allowedDeletionRoles.includes(role)) : (stryCov_9fa48("710"), context.roles.some(stryMutAct_9fa48("711") ? () => undefined : (stryCov_9fa48("711"), role => this.config.allowedDeletionRoles.includes(role))));
      if (stryMutAct_9fa48("714") ? false : stryMutAct_9fa48("713") ? true : stryMutAct_9fa48("712") ? hasPermission : (stryCov_9fa48("712", "713", "714"), !hasPermission)) {
        if (stryMutAct_9fa48("715")) {
          {}
        } else {
          stryCov_9fa48("715");
          throw new SecurityError(stryMutAct_9fa48("716") ? "" : (stryCov_9fa48("716"), "Insufficient permissions to delete agents"), stryMutAct_9fa48("717") ? "" : (stryCov_9fa48("717"), "UNAUTHORIZED"), context);
        }
      }
    }
  }

  /**
   * Check rate limit
   */
  checkRateLimit(context: SecurityContext): void {
    if (stryMutAct_9fa48("718")) {
      {}
    } else {
      stryCov_9fa48("718");
      if (stryMutAct_9fa48("721") ? false : stryMutAct_9fa48("720") ? true : stryMutAct_9fa48("719") ? this.config.rateLimitingEnabled : (stryCov_9fa48("719", "720", "721"), !this.config.rateLimitingEnabled)) {
        if (stryMutAct_9fa48("722")) {
          {}
        } else {
          stryCov_9fa48("722");
          return;
        }
      }
      const key = stryMutAct_9fa48("723") ? `` : (stryCov_9fa48("723"), `${context.tenantId}:${context.userId}`);
      const now = new Date();
      const limit = this.rateLimits.get(key);
      if (stryMutAct_9fa48("726") ? limit || limit.resetAt > now : stryMutAct_9fa48("725") ? false : stryMutAct_9fa48("724") ? true : (stryCov_9fa48("724", "725", "726"), limit && (stryMutAct_9fa48("729") ? limit.resetAt <= now : stryMutAct_9fa48("728") ? limit.resetAt >= now : stryMutAct_9fa48("727") ? true : (stryCov_9fa48("727", "728", "729"), limit.resetAt > now)))) {
        if (stryMutAct_9fa48("730")) {
          {}
        } else {
          stryCov_9fa48("730");
          stryMutAct_9fa48("731") ? limit.count-- : (stryCov_9fa48("731"), limit.count++);
          if (stryMutAct_9fa48("735") ? limit.count <= this.config.rateLimitPerMinute : stryMutAct_9fa48("734") ? limit.count >= this.config.rateLimitPerMinute : stryMutAct_9fa48("733") ? false : stryMutAct_9fa48("732") ? true : (stryCov_9fa48("732", "733", "734", "735"), limit.count > this.config.rateLimitPerMinute)) {
            if (stryMutAct_9fa48("736")) {
              {}
            } else {
              stryCov_9fa48("736");
              throw new SecurityError(stryMutAct_9fa48("737") ? `` : (stryCov_9fa48("737"), `Rate limit exceeded: ${this.config.rateLimitPerMinute} requests per minute`), stryMutAct_9fa48("738") ? "" : (stryCov_9fa48("738"), "RATE_LIMIT_EXCEEDED"), context);
            }
          }
        }
      } else {
        if (stryMutAct_9fa48("739")) {
          {}
        } else {
          stryCov_9fa48("739");
          this.rateLimits.set(key, stryMutAct_9fa48("740") ? {} : (stryCov_9fa48("740"), {
            count: 1,
            resetAt: new Date(stryMutAct_9fa48("741") ? now.getTime() - 60000 : (stryCov_9fa48("741"), now.getTime() + 60000)) // 1 minute from now
          }));
        }
      }
    }
  }

  /**
   * Validate agent profile data
   */
  validateAgentProfile(agent: Partial<AgentProfile>): void {
    if (stryMutAct_9fa48("742")) {
      {}
    } else {
      stryCov_9fa48("742");
      // ID validation
      if (stryMutAct_9fa48("745") ? !agent.id && typeof agent.id !== "string" : stryMutAct_9fa48("744") ? false : stryMutAct_9fa48("743") ? true : (stryCov_9fa48("743", "744", "745"), (stryMutAct_9fa48("746") ? agent.id : (stryCov_9fa48("746"), !agent.id)) || (stryMutAct_9fa48("748") ? typeof agent.id === "string" : stryMutAct_9fa48("747") ? false : (stryCov_9fa48("747", "748"), typeof agent.id !== (stryMutAct_9fa48("749") ? "" : (stryCov_9fa48("749"), "string")))))) {
        if (stryMutAct_9fa48("750")) {
          {}
        } else {
          stryCov_9fa48("750");
          throw new SecurityError(stryMutAct_9fa48("751") ? "" : (stryCov_9fa48("751"), "Agent ID is required and must be a string"), stryMutAct_9fa48("752") ? "" : (stryCov_9fa48("752"), "INVALID_INPUT"));
        }
      }
      if (stryMutAct_9fa48("756") ? agent.id.length <= 100 : stryMutAct_9fa48("755") ? agent.id.length >= 100 : stryMutAct_9fa48("754") ? false : stryMutAct_9fa48("753") ? true : (stryCov_9fa48("753", "754", "755", "756"), agent.id.length > 100)) {
        if (stryMutAct_9fa48("757")) {
          {}
        } else {
          stryCov_9fa48("757");
          throw new SecurityError(stryMutAct_9fa48("758") ? "" : (stryCov_9fa48("758"), "Agent ID too long (max 100 characters)"), stryMutAct_9fa48("759") ? "" : (stryCov_9fa48("759"), "INVALID_INPUT"));
        }
      }

      // Name validation
      if (stryMutAct_9fa48("762") ? !agent.name && typeof agent.name !== "string" : stryMutAct_9fa48("761") ? false : stryMutAct_9fa48("760") ? true : (stryCov_9fa48("760", "761", "762"), (stryMutAct_9fa48("763") ? agent.name : (stryCov_9fa48("763"), !agent.name)) || (stryMutAct_9fa48("765") ? typeof agent.name === "string" : stryMutAct_9fa48("764") ? false : (stryCov_9fa48("764", "765"), typeof agent.name !== (stryMutAct_9fa48("766") ? "" : (stryCov_9fa48("766"), "string")))))) {
        if (stryMutAct_9fa48("767")) {
          {}
        } else {
          stryCov_9fa48("767");
          throw new SecurityError(stryMutAct_9fa48("768") ? "" : (stryCov_9fa48("768"), "Agent name is required and must be a string"), stryMutAct_9fa48("769") ? "" : (stryCov_9fa48("769"), "INVALID_INPUT"));
        }
      }
      if (stryMutAct_9fa48("773") ? agent.name.length <= 200 : stryMutAct_9fa48("772") ? agent.name.length >= 200 : stryMutAct_9fa48("771") ? false : stryMutAct_9fa48("770") ? true : (stryCov_9fa48("770", "771", "772", "773"), agent.name.length > 200)) {
        if (stryMutAct_9fa48("774")) {
          {}
        } else {
          stryCov_9fa48("774");
          throw new SecurityError(stryMutAct_9fa48("775") ? "" : (stryCov_9fa48("775"), "Agent name too long (max 200 characters)"), stryMutAct_9fa48("776") ? "" : (stryCov_9fa48("776"), "INVALID_INPUT"));
        }
      }

      // Sanitize string inputs
      agent.id = this.sanitizeString(agent.id);
      agent.name = this.sanitizeString(agent.name);

      // Validate capabilities
      if (stryMutAct_9fa48("778") ? false : stryMutAct_9fa48("777") ? true : (stryCov_9fa48("777", "778"), agent.capabilities)) {
        if (stryMutAct_9fa48("779")) {
          {}
        } else {
          stryCov_9fa48("779");
          if (stryMutAct_9fa48("783") ? agent.capabilities.taskTypes.length <= 50 : stryMutAct_9fa48("782") ? agent.capabilities.taskTypes.length >= 50 : stryMutAct_9fa48("781") ? false : stryMutAct_9fa48("780") ? true : (stryCov_9fa48("780", "781", "782", "783"), agent.capabilities.taskTypes.length > 50)) {
            if (stryMutAct_9fa48("784")) {
              {}
            } else {
              stryCov_9fa48("784");
              throw new SecurityError(stryMutAct_9fa48("785") ? "" : (stryCov_9fa48("785"), "Too many task types (max 50)"), stryMutAct_9fa48("786") ? "" : (stryCov_9fa48("786"), "INVALID_INPUT"));
            }
          }
          if (stryMutAct_9fa48("790") ? agent.capabilities.languages.length <= 50 : stryMutAct_9fa48("789") ? agent.capabilities.languages.length >= 50 : stryMutAct_9fa48("788") ? false : stryMutAct_9fa48("787") ? true : (stryCov_9fa48("787", "788", "789", "790"), agent.capabilities.languages.length > 50)) {
            if (stryMutAct_9fa48("791")) {
              {}
            } else {
              stryCov_9fa48("791");
              throw new SecurityError(stryMutAct_9fa48("792") ? "" : (stryCov_9fa48("792"), "Too many languages (max 50)"), stryMutAct_9fa48("793") ? "" : (stryCov_9fa48("793"), "INVALID_INPUT"));
            }
          }
          if (stryMutAct_9fa48("797") ? agent.capabilities.specializations.length <= 50 : stryMutAct_9fa48("796") ? agent.capabilities.specializations.length >= 50 : stryMutAct_9fa48("795") ? false : stryMutAct_9fa48("794") ? true : (stryCov_9fa48("794", "795", "796", "797"), agent.capabilities.specializations.length > 50)) {
            if (stryMutAct_9fa48("798")) {
              {}
            } else {
              stryCov_9fa48("798");
              throw new SecurityError(stryMutAct_9fa48("799") ? "" : (stryCov_9fa48("799"), "Too many specializations (max 50)"), stryMutAct_9fa48("800") ? "" : (stryCov_9fa48("800"), "INVALID_INPUT"));
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
    if (stryMutAct_9fa48("801")) {
      {}
    } else {
      stryCov_9fa48("801");
      if (stryMutAct_9fa48("804") ? metrics.qualityScore < 0 && metrics.qualityScore > 1 : stryMutAct_9fa48("803") ? false : stryMutAct_9fa48("802") ? true : (stryCov_9fa48("802", "803", "804"), (stryMutAct_9fa48("807") ? metrics.qualityScore >= 0 : stryMutAct_9fa48("806") ? metrics.qualityScore <= 0 : stryMutAct_9fa48("805") ? false : (stryCov_9fa48("805", "806", "807"), metrics.qualityScore < 0)) || (stryMutAct_9fa48("810") ? metrics.qualityScore <= 1 : stryMutAct_9fa48("809") ? metrics.qualityScore >= 1 : stryMutAct_9fa48("808") ? false : (stryCov_9fa48("808", "809", "810"), metrics.qualityScore > 1)))) {
        if (stryMutAct_9fa48("811")) {
          {}
        } else {
          stryCov_9fa48("811");
          throw new SecurityError(stryMutAct_9fa48("812") ? "" : (stryCov_9fa48("812"), "Quality score must be between 0 and 1"), stryMutAct_9fa48("813") ? "" : (stryCov_9fa48("813"), "INVALID_INPUT"));
        }
      }
      if (stryMutAct_9fa48("816") ? metrics.latencyMs < 0 && metrics.latencyMs > 300000 : stryMutAct_9fa48("815") ? false : stryMutAct_9fa48("814") ? true : (stryCov_9fa48("814", "815", "816"), (stryMutAct_9fa48("819") ? metrics.latencyMs >= 0 : stryMutAct_9fa48("818") ? metrics.latencyMs <= 0 : stryMutAct_9fa48("817") ? false : (stryCov_9fa48("817", "818", "819"), metrics.latencyMs < 0)) || (stryMutAct_9fa48("822") ? metrics.latencyMs <= 300000 : stryMutAct_9fa48("821") ? metrics.latencyMs >= 300000 : stryMutAct_9fa48("820") ? false : (stryCov_9fa48("820", "821", "822"), metrics.latencyMs > 300000)))) {
        if (stryMutAct_9fa48("823")) {
          {}
        } else {
          stryCov_9fa48("823");
          throw new SecurityError(stryMutAct_9fa48("824") ? "" : (stryCov_9fa48("824"), "Latency must be between 0 and 300000ms (5 minutes)"), stryMutAct_9fa48("825") ? "" : (stryCov_9fa48("825"), "INVALID_INPUT"));
        }
      }
      if (stryMutAct_9fa48("828") ? metrics.tokensUsed !== undefined || metrics.tokensUsed < 0 || metrics.tokensUsed > 1000000 : stryMutAct_9fa48("827") ? false : stryMutAct_9fa48("826") ? true : (stryCov_9fa48("826", "827", "828"), (stryMutAct_9fa48("830") ? metrics.tokensUsed === undefined : stryMutAct_9fa48("829") ? true : (stryCov_9fa48("829", "830"), metrics.tokensUsed !== undefined)) && (stryMutAct_9fa48("832") ? metrics.tokensUsed < 0 && metrics.tokensUsed > 1000000 : stryMutAct_9fa48("831") ? true : (stryCov_9fa48("831", "832"), (stryMutAct_9fa48("835") ? metrics.tokensUsed >= 0 : stryMutAct_9fa48("834") ? metrics.tokensUsed <= 0 : stryMutAct_9fa48("833") ? false : (stryCov_9fa48("833", "834", "835"), metrics.tokensUsed < 0)) || (stryMutAct_9fa48("838") ? metrics.tokensUsed <= 1000000 : stryMutAct_9fa48("837") ? metrics.tokensUsed >= 1000000 : stryMutAct_9fa48("836") ? false : (stryCov_9fa48("836", "837", "838"), metrics.tokensUsed > 1000000)))))) {
        if (stryMutAct_9fa48("839")) {
          {}
        } else {
          stryCov_9fa48("839");
          throw new SecurityError(stryMutAct_9fa48("840") ? "" : (stryCov_9fa48("840"), "Tokens used must be between 0 and 1,000,000"), stryMutAct_9fa48("841") ? "" : (stryCov_9fa48("841"), "INVALID_INPUT"));
        }
      }
    }
  }

  /**
   * Validate query parameters
   */
  validateQuery(query: AgentQuery): void {
    if (stryMutAct_9fa48("842")) {
      {}
    } else {
      stryCov_9fa48("842");
      if (stryMutAct_9fa48("845") ? query.maxUtilization === undefined : stryMutAct_9fa48("844") ? false : stryMutAct_9fa48("843") ? true : (stryCov_9fa48("843", "844", "845"), query.maxUtilization !== undefined)) {
        if (stryMutAct_9fa48("846")) {
          {}
        } else {
          stryCov_9fa48("846");
          if (stryMutAct_9fa48("849") ? query.maxUtilization < 0 && query.maxUtilization > 100 : stryMutAct_9fa48("848") ? false : stryMutAct_9fa48("847") ? true : (stryCov_9fa48("847", "848", "849"), (stryMutAct_9fa48("852") ? query.maxUtilization >= 0 : stryMutAct_9fa48("851") ? query.maxUtilization <= 0 : stryMutAct_9fa48("850") ? false : (stryCov_9fa48("850", "851", "852"), query.maxUtilization < 0)) || (stryMutAct_9fa48("855") ? query.maxUtilization <= 100 : stryMutAct_9fa48("854") ? query.maxUtilization >= 100 : stryMutAct_9fa48("853") ? false : (stryCov_9fa48("853", "854", "855"), query.maxUtilization > 100)))) {
            if (stryMutAct_9fa48("856")) {
              {}
            } else {
              stryCov_9fa48("856");
              throw new SecurityError(stryMutAct_9fa48("857") ? "" : (stryCov_9fa48("857"), "Max utilization must be between 0 and 100"), stryMutAct_9fa48("858") ? "" : (stryCov_9fa48("858"), "INVALID_INPUT"));
            }
          }
        }
      }
      if (stryMutAct_9fa48("861") ? query.minSuccessRate === undefined : stryMutAct_9fa48("860") ? false : stryMutAct_9fa48("859") ? true : (stryCov_9fa48("859", "860", "861"), query.minSuccessRate !== undefined)) {
        if (stryMutAct_9fa48("862")) {
          {}
        } else {
          stryCov_9fa48("862");
          if (stryMutAct_9fa48("865") ? query.minSuccessRate < 0 && query.minSuccessRate > 1 : stryMutAct_9fa48("864") ? false : stryMutAct_9fa48("863") ? true : (stryCov_9fa48("863", "864", "865"), (stryMutAct_9fa48("868") ? query.minSuccessRate >= 0 : stryMutAct_9fa48("867") ? query.minSuccessRate <= 0 : stryMutAct_9fa48("866") ? false : (stryCov_9fa48("866", "867", "868"), query.minSuccessRate < 0)) || (stryMutAct_9fa48("871") ? query.minSuccessRate <= 1 : stryMutAct_9fa48("870") ? query.minSuccessRate >= 1 : stryMutAct_9fa48("869") ? false : (stryCov_9fa48("869", "870", "871"), query.minSuccessRate > 1)))) {
            if (stryMutAct_9fa48("872")) {
              {}
            } else {
              stryCov_9fa48("872");
              throw new SecurityError(stryMutAct_9fa48("873") ? "" : (stryCov_9fa48("873"), "Min success rate must be between 0 and 1"), stryMutAct_9fa48("874") ? "" : (stryCov_9fa48("874"), "INVALID_INPUT"));
            }
          }
        }
      }
      if (stryMutAct_9fa48("877") ? query.languages || query.languages.length > 20 : stryMutAct_9fa48("876") ? false : stryMutAct_9fa48("875") ? true : (stryCov_9fa48("875", "876", "877"), query.languages && (stryMutAct_9fa48("880") ? query.languages.length <= 20 : stryMutAct_9fa48("879") ? query.languages.length >= 20 : stryMutAct_9fa48("878") ? true : (stryCov_9fa48("878", "879", "880"), query.languages.length > 20)))) {
        if (stryMutAct_9fa48("881")) {
          {}
        } else {
          stryCov_9fa48("881");
          throw new SecurityError(stryMutAct_9fa48("882") ? "" : (stryCov_9fa48("882"), "Too many languages in query (max 20)"), stryMutAct_9fa48("883") ? "" : (stryCov_9fa48("883"), "INVALID_INPUT"));
        }
      }
    }
  }

  /**
   * Enforce tenant isolation
   */
  scopeToTenant(agentId: AgentId, context: SecurityContext): AgentId {
    if (stryMutAct_9fa48("884")) {
      {}
    } else {
      stryCov_9fa48("884");
      if (stryMutAct_9fa48("887") ? false : stryMutAct_9fa48("886") ? true : stryMutAct_9fa48("885") ? this.config.multiTenantEnabled : (stryCov_9fa48("885", "886", "887"), !this.config.multiTenantEnabled)) {
        if (stryMutAct_9fa48("888")) {
          {}
        } else {
          stryCov_9fa48("888");
          return agentId;
        }
      }

      // Ensure agent ID is scoped to tenant
      const scopedId = stryMutAct_9fa48("889") ? `` : (stryCov_9fa48("889"), `${context.tenantId}:${agentId}`);
      return scopedId as AgentId;
    }
  }

  /**
   * Extract agent ID from scoped ID
   */
  unscopeAgentId(scopedId: AgentId): string {
    if (stryMutAct_9fa48("890")) {
      {}
    } else {
      stryCov_9fa48("890");
      if (stryMutAct_9fa48("893") ? false : stryMutAct_9fa48("892") ? true : stryMutAct_9fa48("891") ? this.config.multiTenantEnabled : (stryCov_9fa48("891", "892", "893"), !this.config.multiTenantEnabled)) {
        if (stryMutAct_9fa48("894")) {
          {}
        } else {
          stryCov_9fa48("894");
          return scopedId;
        }
      }
      const parts = scopedId.split(stryMutAct_9fa48("895") ? "" : (stryCov_9fa48("895"), ":"));
      return (stryMutAct_9fa48("899") ? parts.length <= 1 : stryMutAct_9fa48("898") ? parts.length >= 1 : stryMutAct_9fa48("897") ? false : stryMutAct_9fa48("896") ? true : (stryCov_9fa48("896", "897", "898", "899"), parts.length > 1)) ? parts[1] : scopedId;
    }
  }

  /**
   * Verify agent belongs to tenant
   */
  verifyTenantOwnership(agentId: AgentId, context: SecurityContext): void {
    if (stryMutAct_9fa48("900")) {
      {}
    } else {
      stryCov_9fa48("900");
      if (stryMutAct_9fa48("903") ? false : stryMutAct_9fa48("902") ? true : stryMutAct_9fa48("901") ? this.config.multiTenantEnabled : (stryCov_9fa48("901", "902", "903"), !this.config.multiTenantEnabled)) {
        if (stryMutAct_9fa48("904")) {
          {}
        } else {
          stryCov_9fa48("904");
          return;
        }
      }
      if (stryMutAct_9fa48("907") ? false : stryMutAct_9fa48("906") ? true : stryMutAct_9fa48("905") ? agentId.startsWith(`${context.tenantId}:`) : (stryCov_9fa48("905", "906", "907"), !(stryMutAct_9fa48("908") ? agentId.endsWith(`${context.tenantId}:`) : (stryCov_9fa48("908"), agentId.startsWith(stryMutAct_9fa48("909") ? `` : (stryCov_9fa48("909"), `${context.tenantId}:`)))))) {
        if (stryMutAct_9fa48("910")) {
          {}
        } else {
          stryCov_9fa48("910");
          throw new SecurityError(stryMutAct_9fa48("911") ? "" : (stryCov_9fa48("911"), "Agent does not belong to your tenant"), stryMutAct_9fa48("912") ? "" : (stryCov_9fa48("912"), "UNAUTHORIZED"), context);
        }
      }
    }
  }

  /**
   * Log audit entry
   */
  logAuditEntry(entry: AuditEntry): void {
    if (stryMutAct_9fa48("913")) {
      {}
    } else {
      stryCov_9fa48("913");
      if (stryMutAct_9fa48("916") ? false : stryMutAct_9fa48("915") ? true : stryMutAct_9fa48("914") ? this.config.auditLoggingEnabled : (stryCov_9fa48("914", "915", "916"), !this.config.auditLoggingEnabled)) {
        if (stryMutAct_9fa48("917")) {
          {}
        } else {
          stryCov_9fa48("917");
          return;
        }
      }
      this.auditLog.push(entry);

      // TODO: Persist to database or external audit system
      console.log(stryMutAct_9fa48("918") ? `` : (stryCov_9fa48("918"), `[AUDIT] ${entry.operation} on ${entry.resource}:${entry.resourceId} by ${entry.userId} - ${entry.success ? stryMutAct_9fa48("919") ? "" : (stryCov_9fa48("919"), "SUCCESS") : stryMutAct_9fa48("920") ? "" : (stryCov_9fa48("920"), "FAILURE")}`));
    }
  }

  /**
   * Get audit log
   */
  getAuditLog(tenantId?: string, limit: number = 100): AuditEntry[] {
    if (stryMutAct_9fa48("921")) {
      {}
    } else {
      stryCov_9fa48("921");
      let filtered = this.auditLog;
      if (stryMutAct_9fa48("923") ? false : stryMutAct_9fa48("922") ? true : (stryCov_9fa48("922", "923"), tenantId)) {
        if (stryMutAct_9fa48("924")) {
          {}
        } else {
          stryCov_9fa48("924");
          filtered = stryMutAct_9fa48("925") ? filtered : (stryCov_9fa48("925"), filtered.filter(stryMutAct_9fa48("926") ? () => undefined : (stryCov_9fa48("926"), entry => stryMutAct_9fa48("929") ? entry.tenantId !== tenantId : stryMutAct_9fa48("928") ? false : stryMutAct_9fa48("927") ? true : (stryCov_9fa48("927", "928", "929"), entry.tenantId === tenantId))));
        }
      }
      return stryMutAct_9fa48("930") ? filtered : (stryCov_9fa48("930"), filtered.slice(stryMutAct_9fa48("931") ? +limit : (stryCov_9fa48("931"), -limit)));
    }
  }

  /**
   * Create audit entry
   */
  private createAuditEntry(context: SecurityContext, operation: string, resource: string, resourceId: string): Omit<AuditEntry, "success"> {
    if (stryMutAct_9fa48("932")) {
      {}
    } else {
      stryCov_9fa48("932");
      return stryMutAct_9fa48("933") ? {} : (stryCov_9fa48("933"), {
        id: stryMutAct_9fa48("934") ? `` : (stryCov_9fa48("934"), `audit-${Date.now()}-${stryMutAct_9fa48("935") ? Math.random().toString(36) : (stryCov_9fa48("935"), Math.random().toString(36).substr(2, 9))}`),
        tenantId: context.tenantId,
        userId: context.userId,
        operation,
        resource,
        resourceId,
        timestamp: new Date(),
        metadata: stryMutAct_9fa48("936") ? {} : (stryCov_9fa48("936"), {
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
    if (stryMutAct_9fa48("937")) {
      {}
    } else {
      stryCov_9fa48("937");
      return stryMutAct_9fa48("938") ? {} : (stryCov_9fa48("938"), {
        tenantId: stryMutAct_9fa48("939") ? "" : (stryCov_9fa48("939"), "default"),
        userId: stryMutAct_9fa48("940") ? "" : (stryCov_9fa48("940"), "anonymous"),
        roles: stryMutAct_9fa48("941") ? [] : (stryCov_9fa48("941"), [stryMutAct_9fa48("942") ? "" : (stryCov_9fa48("942"), "public")]),
        requestedAt: new Date(),
        requestId
      });
    }
  }

  /**
   * Sanitize string input to prevent injection attacks
   */
  private sanitizeString(input: string): string {
    if (stryMutAct_9fa48("943")) {
      {}
    } else {
      stryCov_9fa48("943");
      // Remove null bytes
      let sanitized = input.replace(/\0/g, stryMutAct_9fa48("944") ? "Stryker was here!" : (stryCov_9fa48("944"), ""));

      // Trim whitespace
      sanitized = stryMutAct_9fa48("945") ? sanitized : (stryCov_9fa48("945"), sanitized.trim());

      // Remove control characters
      // eslint-disable-next-line no-control-regex
      sanitized = sanitized.replace(stryMutAct_9fa48("946") ? /[^\x00-\x1F\x7F]/g : (stryCov_9fa48("946"), /[\x00-\x1F\x7F]/g), stryMutAct_9fa48("947") ? "Stryker was here!" : (stryCov_9fa48("947"), ""));
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
    if (stryMutAct_9fa48("948")) {
      {}
    } else {
      stryCov_9fa48("948");
      this.security = new AgentRegistrySecurity(securityConfig);
    }
  }

  /**
   * Register agent with security checks
   */
  async registerAgent(agent: AgentProfile, context: SecurityContext): Promise<AgentProfile> {
    if (stryMutAct_9fa48("949")) {
      {}
    } else {
      stryCov_9fa48("949");
      // Rate limiting
      this.security.checkRateLimit(context);

      // Authorization
      this.security.authorizeRegistration(context);

      // Input validation
      this.security.validateAgentProfile(agent);

      // Tenant scoping
      const scopedAgent = stryMutAct_9fa48("950") ? {} : (stryCov_9fa48("950"), {
        ...agent,
        id: this.security.scopeToTenant(agent.id, context)
      });
      try {
        if (stryMutAct_9fa48("951")) {
          {}
        } else {
          stryCov_9fa48("951");
          const result = await this.registry.registerAgent(scopedAgent);

          // Audit log
          this.security.logAuditEntry(stryMutAct_9fa48("952") ? {} : (stryCov_9fa48("952"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("953") ? "" : (stryCov_9fa48("953"), "register_agent"), stryMutAct_9fa48("954") ? "" : (stryCov_9fa48("954"), "agent"), agent.id),
            success: stryMutAct_9fa48("955") ? false : (stryCov_9fa48("955"), true)
          }));
          return result;
        }
      } catch (error) {
        if (stryMutAct_9fa48("956")) {
          {}
        } else {
          stryCov_9fa48("956");
          // Audit log failure
          this.security.logAuditEntry(stryMutAct_9fa48("957") ? {} : (stryCov_9fa48("957"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("958") ? "" : (stryCov_9fa48("958"), "register_agent"), stryMutAct_9fa48("959") ? "" : (stryCov_9fa48("959"), "agent"), agent.id),
            success: stryMutAct_9fa48("960") ? true : (stryCov_9fa48("960"), false),
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
    if (stryMutAct_9fa48("961")) {
      {}
    } else {
      stryCov_9fa48("961");
      // Rate limiting
      this.security.checkRateLimit(context);

      // Tenant scoping
      const scopedId = this.security.scopeToTenant(agentId, context);
      const result = await this.registry.getProfile(scopedId);

      // Audit log
      this.security.logAuditEntry(stryMutAct_9fa48("962") ? {} : (stryCov_9fa48("962"), {
        ...this.createBaseAuditEntry(context, stryMutAct_9fa48("963") ? "" : (stryCov_9fa48("963"), "get_agent"), stryMutAct_9fa48("964") ? "" : (stryCov_9fa48("964"), "agent"), agentId),
        success: stryMutAct_9fa48("965") ? false : (stryCov_9fa48("965"), true)
      }));
      return result;
    }
  }

  /**
   * Query agents with tenant isolation
   */
  async queryAgents(query: AgentQuery, context: SecurityContext): Promise<AgentProfile[]> {
    if (stryMutAct_9fa48("966")) {
      {}
    } else {
      stryCov_9fa48("966");
      // Rate limiting
      this.security.checkRateLimit(context);

      // Input validation
      this.security.validateQuery(query);

      // Get agents (registry will handle capability filtering)
      const results = await this.registry.getAgentsByCapability(query);

      // Filter by tenant
      const tenantResults = stryMutAct_9fa48("967") ? results : (stryCov_9fa48("967"), results.filter(stryMutAct_9fa48("968") ? () => undefined : (stryCov_9fa48("968"), (result: any) => stryMutAct_9fa48("969") ? result.agent.id.endsWith(`${context.tenantId}:`) : (stryCov_9fa48("969"), result.agent.id.startsWith(stryMutAct_9fa48("970") ? `` : (stryCov_9fa48("970"), `${context.tenantId}:`))))));

      // Audit log
      this.security.logAuditEntry(stryMutAct_9fa48("971") ? {} : (stryCov_9fa48("971"), {
        ...this.createBaseAuditEntry(context, stryMutAct_9fa48("972") ? "" : (stryCov_9fa48("972"), "query_agents"), stryMutAct_9fa48("973") ? "" : (stryCov_9fa48("973"), "agent"), stryMutAct_9fa48("974") ? "" : (stryCov_9fa48("974"), "query")),
        success: stryMutAct_9fa48("975") ? false : (stryCov_9fa48("975"), true),
        metadata: stryMutAct_9fa48("976") ? {} : (stryCov_9fa48("976"), {
          requestId: context.requestId,
          roles: context.roles,
          ipAddress: context.ipAddress,
          queryParams: query,
          resultCount: tenantResults.length
        })
      }));
      return tenantResults.map(stryMutAct_9fa48("977") ? () => undefined : (stryCov_9fa48("977"), (r: any) => r.agent));
    }
  }

  /**
   * Update performance with security checks
   */
  async updatePerformance(agentId: AgentId, metrics: PerformanceMetrics, context: SecurityContext): Promise<AgentProfile> {
    if (stryMutAct_9fa48("978")) {
      {}
    } else {
      stryCov_9fa48("978");
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
        if (stryMutAct_9fa48("979")) {
          {}
        } else {
          stryCov_9fa48("979");
          const result = await this.registry.updatePerformance(scopedId, metrics);

          // Audit log
          this.security.logAuditEntry(stryMutAct_9fa48("980") ? {} : (stryCov_9fa48("980"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("981") ? "" : (stryCov_9fa48("981"), "update_performance"), stryMutAct_9fa48("982") ? "" : (stryCov_9fa48("982"), "agent"), agentId),
            success: stryMutAct_9fa48("983") ? false : (stryCov_9fa48("983"), true),
            metadata: stryMutAct_9fa48("984") ? {} : (stryCov_9fa48("984"), {
              requestId: context.requestId,
              roles: context.roles,
              ipAddress: context.ipAddress,
              metrics
            })
          }));
          return result;
        }
      } catch (error) {
        if (stryMutAct_9fa48("985")) {
          {}
        } else {
          stryCov_9fa48("985");
          // Audit log failure
          this.security.logAuditEntry(stryMutAct_9fa48("986") ? {} : (stryCov_9fa48("986"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("987") ? "" : (stryCov_9fa48("987"), "update_performance"), stryMutAct_9fa48("988") ? "" : (stryCov_9fa48("988"), "agent"), agentId),
            success: stryMutAct_9fa48("989") ? true : (stryCov_9fa48("989"), false),
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
    if (stryMutAct_9fa48("990")) {
      {}
    } else {
      stryCov_9fa48("990");
      // Rate limiting
      this.security.checkRateLimit(context);

      // Authorization
      this.security.authorizeDeletion(context);

      // Tenant verification
      const scopedId = this.security.scopeToTenant(agentId, context);
      this.security.verifyTenantOwnership(scopedId, context);
      try {
        if (stryMutAct_9fa48("991")) {
          {}
        } else {
          stryCov_9fa48("991");
          const result = await this.registry.unregisterAgent(scopedId);

          // Audit log
          this.security.logAuditEntry(stryMutAct_9fa48("992") ? {} : (stryCov_9fa48("992"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("993") ? "" : (stryCov_9fa48("993"), "unregister_agent"), stryMutAct_9fa48("994") ? "" : (stryCov_9fa48("994"), "agent"), agentId),
            success: stryMutAct_9fa48("995") ? false : (stryCov_9fa48("995"), true)
          }));
          return result;
        }
      } catch (error) {
        if (stryMutAct_9fa48("996")) {
          {}
        } else {
          stryCov_9fa48("996");
          // Audit log failure
          this.security.logAuditEntry(stryMutAct_9fa48("997") ? {} : (stryCov_9fa48("997"), {
            ...this.createBaseAuditEntry(context, stryMutAct_9fa48("998") ? "" : (stryCov_9fa48("998"), "unregister_agent"), stryMutAct_9fa48("999") ? "" : (stryCov_9fa48("999"), "agent"), agentId),
            success: stryMutAct_9fa48("1000") ? true : (stryCov_9fa48("1000"), false),
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
    if (stryMutAct_9fa48("1001")) {
      {}
    } else {
      stryCov_9fa48("1001");
      return this.security.getAuditLog(context.tenantId, limit);
    }
  }

  /**
   * Create base audit entry
   */
  private createBaseAuditEntry(context: SecurityContext, operation: string, resource: string, resourceId: string): Omit<AuditEntry, "success"> {
    if (stryMutAct_9fa48("1002")) {
      {}
    } else {
      stryCov_9fa48("1002");
      return stryMutAct_9fa48("1003") ? {} : (stryCov_9fa48("1003"), {
        id: stryMutAct_9fa48("1004") ? `` : (stryCov_9fa48("1004"), `audit-${Date.now()}-${stryMutAct_9fa48("1005") ? Math.random().toString(36) : (stryCov_9fa48("1005"), Math.random().toString(36).substr(2, 9))}`),
        tenantId: context.tenantId,
        userId: context.userId,
        operation,
        resource,
        resourceId,
        timestamp: new Date(),
        metadata: stryMutAct_9fa48("1006") ? {} : (stryCov_9fa48("1006"), {
          requestId: context.requestId,
          roles: context.roles,
          ipAddress: context.ipAddress
        })
      });
    }
  }
}