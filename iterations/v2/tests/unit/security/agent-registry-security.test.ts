/**
 * Agent Registry Security Unit Tests
 *
 * Tests authentication, authorization, input validation, and audit logging.
 *
 * @author @darianrosebrook
 */

import { AgentRegistrySecurity, SecurityContext, AuditEventType } from '../../../src/security/AgentRegistrySecurity.js';
import { RegistryError, RegistryErrorType } from '../../../src/types/agent-registry.js';

describe('AgentRegistrySecurity', () => {
  let security: AgentRegistrySecurity;
  let mockContext: SecurityContext;

  beforeEach(() => {
    security = new AgentRegistrySecurity({
      enableAuditLogging: true,
      enableInputValidation: true,
      enableAuthorization: true,
      maxAuditEvents: 100,
      auditRetentionDays: 30,
    });

    mockContext = {
      tenantId: 'test-tenant',
      userId: 'test-user',
      roles: ['agent-registry-user'],
      permissions: ['agent:read', 'agent:create', 'agent:update', 'agent:delete'],
      sessionId: 'test-session',
      ipAddress: '127.0.0.1',
      userAgent: 'test-agent',
    };
  });

  describe('Authentication', () => {
    it('should authenticate valid token', async () => {
      const context = await security.authenticate('valid-token-1234567890');

      expect(context).toBeTruthy();
      expect(context?.tenantId).toBe('default-tenant');
      expect(context?.userId).toBe('anonymous');
      expect(context?.permissions).toContain('agent:read');
    });

    it('should reject invalid token', async () => {
      const context = await security.authenticate('invalid');

      expect(context).toBeNull();
    });

    it('should reject empty token', async () => {
      const context = await security.authenticate('');

      expect(context).toBeNull();
    });
  });

  describe('Authorization', () => {
    it('should authorize valid action with proper permissions', async () => {
      const authorized = await security.authorize(
        mockContext,
        'create' as any,
        'agent',
        'test-agent'
      );

      expect(authorized).toBe(true);
    });

    it('should deny action without required permissions', async () => {
      const restrictedContext = {
        ...mockContext,
        permissions: ['agent:read'], // Missing agent:create
      };

      const authorized = await security.authorize(
        restrictedContext,
        'create' as any,
        'agent',
        'test-agent'
      );

      expect(authorized).toBe(false);
    });

    it('should handle rate limiting', async () => {
      // Configure very low rate limit for testing
      const securityWithLimit = new AgentRegistrySecurity({
        enableAuthorization: true,
        rateLimitWindowMs: 1000,
        rateLimitMaxRequests: 2,
      });

      // First two requests should succeed
      const auth1 = await securityWithLimit.authorize(mockContext, 'read' as any, 'agent', 'test');
      const auth2 = await securityWithLimit.authorize(mockContext, 'read' as any, 'agent', 'test');

      expect(auth1).toBe(true);
      expect(auth2).toBe(true);

      // Third request should be rate limited
      const auth3 = await securityWithLimit.authorize(mockContext, 'read' as any, 'agent', 'test');

      expect(auth3).toBe(false);
    });
  });

  describe('Input Validation', () => {
    it('should validate valid agent data', () => {
      const validData = {
        id: 'test-agent-123',
        name: 'Test Agent',
        modelFamily: 'gpt-4' as const,
        capabilities: {
          taskTypes: ['code-editing' as const],
          languages: ['TypeScript' as const],
          specializations: [],
        },
      };

      const result = security.validateAgentData(validData);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should reject invalid agent ID', () => {
      const invalidData = {
        id: '',
        name: 'Test Agent',
        modelFamily: 'gpt-4' as const,
        capabilities: {
          taskTypes: [],
          languages: [],
          specializations: [],
        },
      };

      const result = security.validateAgentData(invalidData);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Agent ID is required and must be a non-empty string');
    });

    it('should reject invalid model family', () => {
      const invalidData = {
        id: 'test-agent-123',
        name: 'Test Agent',
        modelFamily: 'invalid-model' as any,
        capabilities: {
          taskTypes: [],
          languages: [],
          specializations: [],
        },
      };

      const result = security.validateAgentData(invalidData);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Model family must be one of: gpt-4, claude-3, claude-3.5, gemini-pro, llama-3, mixtral');
    });

    it('should reject invalid task types', () => {
      const invalidData = {
        id: 'test-agent-123',
        name: 'Test Agent',
        modelFamily: 'gpt-4' as const,
        capabilities: {
          taskTypes: ['invalid-task' as any],
          languages: [],
          specializations: [],
        },
      };

      const result = security.validateAgentData(invalidData);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid task type: invalid-task');
    });

    it('should sanitize agent ID', () => {
      const unsanitizedData = {
        id: 'test agent@#$%^&*()',
        name: 'Test Agent',
        modelFamily: 'gpt-4' as const,
        capabilities: {
          taskTypes: [],
          languages: [],
          specializations: [],
        },
      };

      const result = security.validateAgentData(unsanitizedData);

      expect(result.valid).toBe(true);
      expect(result.sanitized?.id).toBe('testagent'); // Special chars removed
    });
  });

  describe('Audit Logging', () => {
    it('should log successful events', async () => {
      const event = {
        id: 'test-event-123',
        timestamp: new Date(),
        eventType: AuditEventType.AGENT_REGISTRATION,
        actor: {
          tenantId: 'test-tenant',
          userId: 'test-user',
          sessionId: 'test-session',
        },
        resource: { type: 'agent' as const, id: 'test-agent' },
        action: 'create' as any,
        details: { test: true },
        result: 'success' as const,
      };

      await security.logAuditEvent(event);

      const events = security.getAuditEvents('test-agent');
      expect(events).toHaveLength(1);
      expect(events[0].result).toBe('success');
    });

    it('should log security violations', async () => {
      // Trigger a security violation by attempting unauthorized action
      const authorized = await security.authorize(
        { ...mockContext, permissions: [] }, // No permissions
        'create' as any,
        'agent',
        'test-agent'
      );

      expect(authorized).toBe(false);

      // Check that violation was logged
      const events = security.getAuditEvents('test-agent');
      const violationEvent = events.find(e => e.eventType === AuditEventType.AUTHORIZATION_FAILURE);
      expect(violationEvent).toBeTruthy();
      expect(violationEvent?.result).toBe('failure');
    });

    it('should maintain audit event limits', async () => {
      const securityWithLimit = new AgentRegistrySecurity({
        maxAuditEvents: 3,
      });

      // Add more events than the limit
      for (let i = 0; i < 5; i++) {
        await securityWithLimit.logAuditEvent({
          id: `event-${i}`,
          timestamp: new Date(),
          eventType: AuditEventType.AGENT_REGISTRATION,
          actor: {
            tenantId: 'test',
            userId: 'test',
            sessionId: 'test',
          },
          resource: { type: 'agent' as const, id: 'test-agent' },
          action: 'create' as any,
          details: {},
          result: 'success' as const,
        });
      }

      // Should only keep the most recent events
      const events = securityWithLimit.getAuditEvents('test-agent');
      expect(events.length).toBeLessThanOrEqual(3);
    });
  });

  describe('Security Statistics', () => {
    it('should provide security statistics', async () => {
      // Trigger some security events
      await security.authorize(
        { ...mockContext, permissions: [] },
        'create' as any,
        'agent',
        'test-agent'
      );

      const stats = security.getSecurityStats();

      expect(stats).toHaveProperty('totalAuditEvents');
      expect(stats).toHaveProperty('securityViolations');
      expect(stats).toHaveProperty('authFailures');
      expect(stats).toHaveProperty('authzFailures');
      expect(stats).toHaveProperty('rateLimitHits');

      expect(stats.authzFailures).toBeGreaterThan(0);
    });
  });

  describe('Performance Metrics Validation', () => {
    it('should validate valid performance metrics', () => {
      const validMetrics = {
        success: true,
        qualityScore: 0.85,
        latencyMs: 150,
        taskType: 'code-editing' as const,
        tokensUsed: 500,
      };

      const result = security.validatePerformanceMetrics(validMetrics);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should reject invalid quality score', () => {
      const invalidMetrics = {
        success: true,
        qualityScore: 1.5, // Invalid: > 1.0
        latencyMs: 150,
      };

      const result = security.validatePerformanceMetrics(invalidMetrics);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Quality score must be a number between 0 and 1');
    });

    it('should reject negative latency', () => {
      const invalidMetrics = {
        success: true,
        qualityScore: 0.8,
        latencyMs: -100, // Invalid: negative
      };

      const result = security.validatePerformanceMetrics(invalidMetrics);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Latency must be a non-negative number');
    });
  });
});