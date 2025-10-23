# Security Hardening Audit - ARBITER v2

**Date**: October 19, 2025
**Status**: In Development → Production Hardening Phase
**Severity Levels**: Critical | High | Medium | Low

---

## Executive Summary

ARBITER v2 has a solid security foundation with properly implemented:
- Input validation and sanitization framework
- Command injection prevention 
- XSS/SQL injection protection
- Authentication framework with JWT
- Audit logging and monitoring
- Constitutional violation detection

**However**, there are **6 critical hardening items** that must be addressed before production deployment.

---

## CRITICAL ISSUES

### Issue 1: Default JWT Secret in Production
**Severity**: CRITICAL
**File**: `src/security/AgentRegistrySecurity.ts:105`
**Current Code**:
```typescript
jwtSecret: process.env.JWT_SECRET || "default-jwt-secret-change-in-production",
```

**Risk**: Any attacker can forge JWT tokens using the default secret
**Impact**: Complete authentication bypass
**Fix Required**:
```typescript
// In production, JWT_SECRET MUST be provided
if (!process.env.JWT_SECRET && process.env.NODE_ENV === 'production') {
  throw new Error('JWT_SECRET environment variable is required in production');
}
const jwtSecret = process.env.JWT_SECRET;
if (!jwtSecret || jwtSecret === 'default-jwt-secret-change-in-production') {
  throw new Error('Invalid or missing JWT_SECRET. Please set a strong secret in production.');
}
```

**Verification Command**:
```bash
# Should FAIL in production
NODE_ENV=production npm start  # Will error: JWT_SECRET required
# Should PASS
NODE_ENV=production JWT_SECRET="$(openssl rand -base64 32)" npm start
```

---

### Issue 2: Mock Fallbacks in Production Code
**Severity**: CRITICAL
**Files**:
- `src/orchestrator/ArbiterOrchestrator.ts:1228, 1255, 1288, 1322, 1655`
- `src/security/AgentRegistrySecurity.ts:164-165`
- `src/orchestrator/TaskOrchestrator.ts:527`

**Examples**:
```typescript
// AgentRegistrySecurity.ts
if (!this.config.enableJwtValidation) {
  console.warn("JWT validation disabled, using mock authentication");
  return this.createMockSecurityContext(token);  // INSECURE IN PRODUCTION
}

// ArbiterOrchestrator.ts
if (!this.components.agentRegistry.getAgent) {
  // Fallback to mock implementation
  console.log("Agent registry not available, using mock");  // INSECURE
  return mockAgentProfile;
}
```

**Risk**: Attackers can bypass security by disabling components
**Impact**: Authentication bypass, privilege escalation
**Fix Required**:
```typescript
// PRODUCTION: No fallback to mock. Throw error instead.
if (process.env.NODE_ENV === 'production') {
  if (!this.components.agentRegistry?.getAgent) {
    throw new Error('Agent registry is required in production');
  }
}

// DEVELOPMENT: Keep warning but explicitly guard production
if (process.env.NODE_ENV !== 'production' && !this.config.enableJwtValidation) {
  console.warn('⚠️ JWT validation disabled - DEVELOPMENT ONLY');
  return this.createMockSecurityContext(token);
}
```

**Verification Command**:
```bash
# Audit mock references
grep -r "mock\|fallback\|development only" iterations/v2/src/security/ iterations/v2/src/orchestrator/ | grep -i "authentication\|registry\|token"
```

---

### Issue 3: Insecure Password Handling in Database Config
**Severity**: CRITICAL
**File**: `src/config/AppConfig.ts:135`
**Current Code**:
```typescript
password: process.env.DB_PASSWORD,  // Could be exposed in logs
```

**Risk**: Database password might be exposed in error logs, stack traces, or debug output
**Impact**: Database compromise, data breach
**Fix Required**:
```typescript
// AUDIT: Ensure DB password never appears in logs
import { MaskingLogger } from '@/observability/MaskingLogger';

// When logging, use:
private logger = new MaskingLogger('AppConfig', {
  maskPatterns: [/password|secret|key|token/i],
});

// In config loading:
private loadConfig(): AppConfig {
  const raw = {
    database: {
      // ... other config
      password: process.env.DB_PASSWORD,  // Never log this
    }
  };
  
  // NEVER do this:
  // console.log('Loaded config:', raw);  // INSECURE
  
  // DO this instead:
  this.logger.debug('Database config loaded', { 
    host: raw.database.host, 
    port: raw.database.port 
    // DON'T include password
  });
}
```

**Verification**: 
```bash
# Audit for password logging
grep -r "password\|secret" iterations/v2/src --include="*.ts" | grep -i "log\|console\|error\|debug"
```

---

### Issue 4: Missing Rate Limiting on Authentication
**Severity**: CRITICAL
**File**: `src/security/AgentRegistrySecurity.ts`
**Issue**: Rate limiting exists but isn't enforced on token validation

**Risk**: Brute force attacks on JWT tokens or authentication endpoints
**Impact**: Account takeover through password guessing or token prediction
**Fix Required**:
```typescript
// Add rate limiting to authenticate method
async authenticate(token: string): Promise<SecurityContext | null> {
  const rateLimitKey = `auth:${token.substring(0, 20)}`;
  
  // Check rate limit
  if (!this.checkRateLimit(rateLimitKey)) {
    this.logAuditEvent({
      eventType: AuditEventType.RATE_LIMIT_EXCEEDED,
      details: { reason: 'Authentication rate limit exceeded' }
    });
    throw new Error('Too many authentication attempts. Please try again later.');
  }
  
  try {
    // Existing authentication logic
    const decoded = await this.validateJwtToken(token);
    // ... rest of authentication
  } catch (error) {
    this.incrementRateLimit(rateLimitKey);
    throw error;
  }
}
```

---

### Issue 5: No HTTPS Enforcement in Production
**Severity**: CRITICAL
**Files**: 
- `src/observer/auth.ts`
- `src/mcp/arbiter-mcp-server.ts`
- API endpoints throughout

**Issue**: No enforcement of HTTPS in production
**Risk**: Man-in-the-middle attacks, credential interception, data eavesdropping
**Impact**: Complete compromise of authentication and data confidentiality

**Fix Required**:
```typescript
// In main server setup
if (process.env.NODE_ENV === 'production') {
  // MUST use HTTPS
  const https = require('https');
  const fs = require('fs');
  
  const options = {
    key: fs.readFileSync(process.env.TLS_KEY_PATH),
    cert: fs.readFileSync(process.env.TLS_CERT_PATH),
  };
  
  https.createServer(options, app).listen(process.env.PORT || 443);
} else {
  // Development: HTTP is acceptable
  const http = require('http');
  http.createServer(app).listen(process.env.PORT || 3000);
}

// Add HSTS header
app.use((req, res, next) => {
  if (process.env.NODE_ENV === 'production') {
    res.setHeader('Strict-Transport-Security', 'max-age=31536000; includeSubDomains; preload');
  }
  next();
});
```

---

### Issue 6: Insufficient Input Validation in Task Execution
**Severity**: CRITICAL
**File**: `src/orchestrator/TaskOrchestrator.ts`
**Issue**: Task scripts executed without comprehensive validation

**Risk**: Code injection, arbitrary code execution
**Impact**: Complete system compromise, data theft, malware execution

**Fix Required**:
```typescript
// Add comprehensive task validation
async executeTask(task: Task): Promise<void> {
  // Validate task structure
  if (!this.isValidTask(task)) {
    throw new Error('Invalid task structure');
  }
  
  // Validate script/code payload
  if (task.type === 'script') {
    const scriptValidation = await this.validateScript(task.script);
    if (!scriptValidation.valid) {
      this.logSecurityViolation({
        type: 'INVALID_SCRIPT',
        taskId: task.id,
        reason: scriptValidation.reason,
        details: scriptValidation.details
      });
      throw new Error(`Script validation failed: ${scriptValidation.reason}`);
    }
  }
  
  // Validate payload size
  const payloadSize = JSON.stringify(task).length;
  if (payloadSize > this.config.maxPayloadSizeBytes) {
    throw new Error(`Task payload exceeds maximum size`);
  }
  
  // Continue with execution...
}

private isValidTask(task: any): task is Task {
  return (
    task && 
    typeof task === 'object' &&
    typeof task.id === 'string' &&
    typeof task.type === 'string' &&
    ['script', 'function', 'workflow'].includes(task.type)
  );
}
```

---

## HIGH PRIORITY ISSUES

### Issue 7: Missing CORS and CSRF Protection
**Severity**: HIGH
**Files**: `src/mcp/arbiter-mcp-server.ts`, API endpoints

**Current State**: No CORS/CSRF headers visible
**Fix Required**:
```typescript
// Add CORS middleware
import cors from 'cors';

app.use(cors({
  origin: process.env.ALLOWED_ORIGINS?.split(',') || ['http://localhost:3000'],
  credentials: true,
  maxAge: 86400,
}));

// Add CSRF protection
import csrf from 'csurf';
const csrfProtection = csrf({ cookie: false });
app.post('/api/*', csrfProtection, (req, res, next) => {
  // Handle CSRF token validation
  next();
});
```

---

### Issue 8: No Secrets Rotation Policy
**Severity**: HIGH
**Files**: JWT secret, API keys, database credentials

**Current State**: Secrets are static
**Fix Required**:
```bash
# Add to deployment checklist
1. Implement secrets rotation every 90 days
2. Use HashiCorp Vault or AWS Secrets Manager
3. Add automated rotation jobs
4. Log all secret access events

# Example configuration
SECRETS_ROTATION_ENABLED=true
SECRETS_ROTATION_INTERVAL_DAYS=90
VAULT_ADDR=https://vault.production.example.com
```

---

### Issue 9: Incomplete Audit Logging
**Severity**: HIGH
**Files**: Multiple security operations

**Current State**: Audit logging exists but not comprehensive
**Missing Events**:
- API authentication failures (logged but needs review)
- Configuration changes
- Permission escalations
- Sensitive data access

**Fix Required**: Add to all sensitive operations
```typescript
private async logSecurityEvent(event: {
  type: string;
  actor: string;
  resource: string;
  action: string;
  result: 'success' | 'failure';
  details: any;
  severity: 'low' | 'medium' | 'high' | 'critical';
}): Promise<void> {
  // Log to protected audit database
  await this.auditLogger.log({
    ...event,
    timestamp: new Date(),
    ipAddress: this.currentRequest?.ip,
    userAgent: this.currentRequest?.headers['user-agent'],
  });
}
```

---

### Issue 10: Missing Encryption at Rest
**Severity**: HIGH
**Files**: Database storage, artifact storage

**Current State**: No evidence of encryption for sensitive data at rest
**Fix Required**:
```typescript
// Enable encryption for sensitive fields
import { encrypt, decrypt } from '@/security/Encryption';

class AgentRegistry {
  async storeAgent(agent: AgentProfile): Promise<void> {
    const encrypted = {
      ...agent,
      apiKey: await encrypt(agent.apiKey),
      credentials: await encrypt(JSON.stringify(agent.credentials)),
    };
    await this.db.insert(encrypted);
  }
  
  async retrieveAgent(agentId: string): Promise<AgentProfile> {
    const encrypted = await this.db.get(agentId);
    return {
      ...encrypted,
      apiKey: await decrypt(encrypted.apiKey),
      credentials: JSON.parse(await decrypt(encrypted.credentials)),
    };
  }
}
```

---

## MEDIUM PRIORITY ISSUES

### Issue 11: Insufficient Command Injection Prevention
**Severity**: MEDIUM
**File**: `src/security/CommandValidator.ts`
**Status**: Partially addressed but needs verification

**Current Protection**:
- Blocks shell metacharacters
- Prevents command substitution patterns
- Validates against dangerous commands

**Recommended Additional Controls**:
```typescript
// Add process limits
const execSync = require('child_process').execSync;

function executeCommand(command: string, args: string[]): void {
  const options = {
    maxBuffer: 1024 * 1024,  // 1MB max output
    timeout: 30000,           // 30 second timeout
    stdio: 'pipe',            // No direct console access
    uid: process.getuid?.() + 1,  // Run as different user if possible
    gid: process.getgid?.() + 1,
  };
  
  execSync(command, { ...options, argv0: args[0] });
}
```

---

### Issue 12: Missing API Rate Limiting
**Severity**: MEDIUM
**Files**: All API endpoints

**Current State**: Per-agent rate limiting exists, but global API limits missing
**Fix Required**:
```typescript
import rateLimit from 'express-rate-limit';

const apiLimiter = rateLimit({
  windowMs: 60 * 1000,        // 1 minute
  max: 1000,                  // Max requests per window
  message: 'Too many requests, please try again later',
  standardHeaders: true,      // Return rate limit info in headers
  legacyHeaders: false,       // Disable X-RateLimit-* headers
});

app.use('/api/', apiLimiter);
```

---

### Issue 13: Missing Content Security Policy (CSP)
**Severity**: MEDIUM
**Files**: MCP server, Observer endpoints

**Current State**: No CSP headers
**Fix Required**:
```typescript
app.use((req, res, next) => {
  if (process.env.NODE_ENV === 'production') {
    res.setHeader(
      'Content-Security-Policy',
      "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
    );
  }
  next();
});
```

---

## LOW PRIORITY ISSUES

### Issue 14: Add Security Headers
**Severity**: LOW
**Files**: All HTTP responses

**Recommended Headers**:
```typescript
// Add helmet.js or manually set headers
app.use((req, res, next) => {
  // Prevent clickjacking
  res.setHeader('X-Frame-Options', 'DENY');
  
  // Prevent MIME sniffing
  res.setHeader('X-Content-Type-Options', 'nosniff');
  
  // Enable XSS protection
  res.setHeader('X-XSS-Protection', '1; mode=block');
  
  // Referrer policy
  res.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin');
  
  next();
});
```

---

### Issue 15: Improve Error Messages
**Severity**: LOW
**Files**: Error handlers throughout

**Current State**: Error messages sometimes too verbose
**Fix Required**:
```typescript
// Don't expose internals to clients
function handleError(error: Error, res: Response): void {
  if (process.env.NODE_ENV === 'production') {
    // Generic error message for production
    res.status(500).json({ error: 'Internal server error' });
  } else {
    // Detailed error for development
    res.status(500).json({ 
      error: error.message, 
      stack: error.stack 
    });
  }
}
```

---

## Production Hardening Checklist

### Phase 1: CRITICAL (Must complete before production)
- [ ] Fix default JWT secret (Issue 1) - **BLOCKING**
- [ ] Remove mock fallbacks in production (Issue 2) - **BLOCKING**
- [ ] Mask database passwords in logs (Issue 3) - **BLOCKING**
- [ ] Add authentication rate limiting (Issue 4) - **BLOCKING**
- [ ] Enforce HTTPS (Issue 5) - **BLOCKING**
- [ ] Validate task payloads (Issue 6) - **BLOCKING**

**Estimated Time**: 4-6 hours
**Verification**: `npm run security:audit`

### Phase 2: HIGH PRIORITY (Must complete for MVP+1)
- [ ] Add CORS/CSRF protection (Issue 7) - 1-2 hours
- [ ] Implement secrets rotation (Issue 8) - 2-3 hours
- [ ] Complete audit logging (Issue 9) - 2-3 hours
- [ ] Enable encryption at rest (Issue 10) - 3-4 hours

**Estimated Time**: 8-12 hours
**Verification**: `npm run security:comprehensive`

### Phase 3: MEDIUM PRIORITY (During optimization)
- [ ] Enhance command injection prevention (Issue 11) - 1-2 hours
- [ ] Add global API rate limiting (Issue 12) - 1-2 hours
- [ ] Implement CSP headers (Issue 13) - 1 hour

**Estimated Time**: 3-5 hours

### Phase 4: LOW PRIORITY (Nice to have)
- [ ] Add security headers (Issue 14) - 1 hour
- [ ] Improve error handling (Issue 15) - 1 hour

---

## Verification Commands

```bash
# Check for hardcoded secrets
npm run security:check-secrets

# Audit production safety
npm run security:production-audit

# Check for mock implementations in production
npm run security:check-mocks

# Validate configuration
npm run security:validate-config

# Run full security suite
npm run security:full-audit
```

---

## Security Scorecard

| Category | Status | Score | Required for |
|----------|--------|-------|--------------|
| Authentication | Partial | 60% | MVP |
| Authorization | Good | 80% | MVP |
| Input Validation | Partial | 70% | MVP |
| Encryption | Missing | 20% | MVP+1 |
| Audit Logging | Partial | 65% | MVP+1 |
| Error Handling | Good | 75% | MVP |
| **Overall** | **DEVELOPMENT** | **62%** | **MVP+1** |

---

## Timeline

**This Week (Critical)**:
- Issues 1-6 (4-6 hours)
- Target: Security score 85%+

**Next Week (High Priority)**:
- Issues 7-10 (8-12 hours)
- Target: Security score 90%+

**Week 3 (Medium)**:
- Issues 11-13 (3-5 hours)
- Target: Security score 95%+

---

## Deployment Gate

**Do NOT deploy to production until:**
- [ ] All CRITICAL issues fixed (Phase 1)
- [ ] JWT secret properly configured
- [ ] No mock implementations in production code
- [ ] Database password masked in all logs
- [ ] HTTPS enforced
- [ ] Input validation comprehensive
- [ ] Security team audit passed
- [ ] Penetration test results reviewed

---

## Questions?

For security questions:
1. Check `docs/security/` directory
2. Review `src/security/` source files
3. Contact security team
4. Escalate critical issues immediately

---

**Document Owner**: @darianrosebrook
**Last Updated**: October 19, 2025
**Next Review**: Before MVP release

