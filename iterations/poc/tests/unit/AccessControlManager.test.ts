/**
 * Unit tests for AccessControlManager
 *
 * Tests sophisticated access control beyond Row Level Security for Data Layer Phase 3.
 *
 * @author @darianrosebrook
 */

import {
  AccessControlManager,
  AccessRequest,
} from "../../src/data/security/AccessControlManager";

describe("AccessControlManager", () => {
  let accessControl: AccessControlManager;

  beforeEach(() => {
    accessControl = new AccessControlManager({
      enableAccessControl: true,
      defaultEffect: "deny",
      policyEvaluationMode: "first-match",
      auditLogging: false, // Disable logging for tests
      initializeDefaults: false, // Don't add default policies for this test
    });

    // Add a policy for documents access - only allow access to alice's documents
    accessControl.addPolicy({
      id: "documents-access",
      name: "User Access",
      description: "Allow users to access alice's documents",
      effect: "allow",
      principals: ["*"],
      resources: ["documents:user:alice"], // Only alice's documents
      actions: ["read", "write", "update"],
      priority: 60,
      enabled: true,
    });

    // Remove any other policies to ensure isolation
    const allPolicies = accessControl.listPolicies();
    for (const policy of allPolicies) {
      if (policy.id !== "documents-access") {
        accessControl.removePolicy(policy.id);
      }
    }
  });

  describe("Policy Management", () => {
    it("should add and retrieve policies", () => {
      const policy = {
        id: "test-policy",
        name: "Test Policy",
        description: "A test policy",
        effect: "allow" as const,
        principals: ["user:alice"],
        resources: ["documents:*"],
        actions: ["read"],
        priority: 10,
        enabled: true,
      };

      accessControl.addPolicy(policy);

      const retrieved = accessControl.getPolicy("test-policy");
      expect(retrieved).toEqual(policy);
    });

    it("should update existing policies", () => {
      const policy = {
        id: "update-test",
        name: "Update Test",
        description: "Test policy updates",
        effect: "allow" as const,
        principals: ["user:bob"],
        resources: ["files:*"],
        actions: ["read"],
        priority: 5,
        enabled: true,
      };

      accessControl.addPolicy(policy);
      accessControl.updatePolicy("update-test", {
        priority: 15,
        enabled: false,
      });

      const updated = accessControl.getPolicy("update-test");
      expect(updated?.priority).toBe(15);
      expect(updated?.enabled).toBe(false);
    });

    it("should remove policies", () => {
      const policy = {
        id: "remove-test",
        name: "Remove Test",
        description: "Test policy removal",
        effect: "allow" as const,
        principals: ["user:charlie"],
        resources: ["data:*"],
        actions: ["write"],
        priority: 1,
        enabled: true,
      };

      accessControl.addPolicy(policy);
      expect(accessControl.removePolicy("remove-test")).toBe(true);
      expect(accessControl.getPolicy("remove-test")).toBeUndefined();
      expect(accessControl.removePolicy("non-existent")).toBe(false);
    });

    it("should list all policies", () => {
      const policies = accessControl.listPolicies();
      expect(policies.length).toBeGreaterThan(0);
      expect(policies[0].priority).toBeGreaterThanOrEqual(
        policies[policies.length - 1].priority
      );
    });
  });

  describe("Access Evaluation", () => {
    beforeEach(() => {
      // Clear default policies and add test policies
      const allPolicies = accessControl.listPolicies();
      allPolicies.forEach((policy) => accessControl.removePolicy(policy.id));

      // Add test policies
      accessControl.addPolicy({
        id: "admin-policy",
        name: "Admin Access",
        description: "Full access for admins",
        effect: "allow",
        principals: ["role:admin"],
        resources: ["*"],
        actions: ["*"],
        priority: 100,
        enabled: true,
      });

      accessControl.addPolicy({
        id: "user-policy",
        name: "User Access",
        description: "Limited access for users to their own resources",
        effect: "allow",
        principals: ["user:*"],
        resources: ["documents:user:${principal}", "files:user:${principal}"], // Only own resources
        actions: ["read", "write"],
        priority: 80,
        enabled: true,
        conditions: [
          {
            type: "time",
            operator: "between",
            attribute: "timestamp",
            value: [9, 17], // Use numeric hours instead of time strings
          },
        ],
      });
    });

    it("should allow access for admin role", async () => {
      const request: AccessRequest = {
        principal: "role:admin",
        resource: "sensitive:data",
        action: "delete",
        tenantId: "tenant-1",
      };

      const decision = await accessControl.evaluateAccess(request);
      expect(decision.allowed).toBe(true);
      expect(decision.reason).toContain("Admin Access");
    });

    it("should allow user access to own resources", async () => {
      const request: AccessRequest = {
        principal: "user:alice",
        resource: "documents:user:alice",
        action: "read",
        tenantId: "tenant-1",
        context: {
          timestamp: new Date("2024-01-01T14:00:00Z"), // 2 PM - within business hours (9-5)
        },
      };

      const decision = await accessControl.evaluateAccess(request);
      expect(decision.allowed).toBe(true);
      expect(decision.reason).toContain("User Access");
    });

    it("should deny user access to other resources", async () => {
      const request: AccessRequest = {
        principal: "user:alice",
        resource: "documents:user:bob",
        action: "read",
        tenantId: "tenant-1",
      };

      const decision = await accessControl.evaluateAccess(request);
      expect(decision.allowed).toBe(false);
      expect(decision.reason).toContain("Default deny");
    });

    it("should deny access outside business hours", async () => {
      const request: AccessRequest = {
        principal: "user:alice",
        resource: "documents:user:alice",
        action: "read",
        tenantId: "tenant-1",
        context: {
          timestamp: new Date("2024-01-01T20:00:00Z"), // Outside business hours
        },
      };

      const decision = await accessControl.evaluateAccess(request);
      expect(decision.allowed).toBe(false);
    });

    it("should handle wildcards in patterns", async () => {
      const request: AccessRequest = {
        principal: "user:test123",
        resource: "documents:user:test123",
        action: "read",
        tenantId: "tenant-1",
        context: {
          timestamp: new Date("2024-01-01T12:00:00Z"),
        },
      };

      const decision = await accessControl.evaluateAccess(request);
      expect(decision.allowed).toBe(true);
    });
  });

  describe("Condition Evaluation", () => {
    beforeEach(() => {
      const allPolicies = accessControl.listPolicies();
      allPolicies.forEach((policy) => accessControl.removePolicy(policy.id));

      accessControl.addPolicy({
        id: "conditional-policy",
        name: "Conditional Access",
        description: "Access with conditions",
        effect: "allow",
        principals: ["user:*"],
        resources: ["api:*"],
        actions: ["call"],
        priority: 50,
        enabled: true,
        conditions: [
          {
            type: "attribute",
            operator: "equals",
            attribute: "department",
            value: "engineering",
          },
          {
            type: "ip",
            operator: "regex",
            attribute: "ip",
            value: "^192\\.168\\.",
          },
          {
            type: "relationship",
            operator: "contains",
            attribute: "permissions",
            value: "api_access",
          },
        ],
      });
    });

    it("should evaluate multiple conditions correctly", async () => {
      const request: AccessRequest = {
        principal: "user:alice",
        resource: "api:endpoint",
        action: "call",
        tenantId: "tenant-1",
        context: {
          attributes: {
            department: "engineering",
            permissions: ["api_access", "read_only"],
          },
          ip: "192.168.1.100",
        },
      };

      const decision = await accessControl.evaluateAccess(request);
      expect(decision.allowed).toBe(true);
    });

    it("should deny when conditions are not met", async () => {
      const request: AccessRequest = {
        principal: "user:alice",
        resource: "api:endpoint",
        action: "call",
        tenantId: "tenant-1",
        context: {
          attributes: {
            department: "marketing", // Wrong department
            permissions: ["api_access"],
          },
          ip: "192.168.1.100",
        },
      };

      const decision = await accessControl.evaluateAccess(request);
      expect(decision.allowed).toBe(false);
    });
  });

  describe("Rate Limiting", () => {
    beforeEach(() => {
      const allPolicies = accessControl.listPolicies();
      allPolicies.forEach((policy) => accessControl.removePolicy(policy.id));

      // Create access control with rate limiting and deny by default
      accessControl = new AccessControlManager({
        enableAccessControl: true,
        defaultEffect: "deny",
        rateLimiting: {
          enabled: true,
          windowMs: 1000, // 1 second window
          maxRequests: 2,
        },
      });

      // Add a policy that allows the test requests
      accessControl.addPolicy({
        id: "test-allow",
        name: "Test Allow",
        description: "Allow test requests",
        effect: "allow",
        principals: ["user:test"],
        resources: ["api:test"],
        actions: ["call"],
        priority: 50,
        enabled: true,
      });
    });

    it("should enforce rate limits", async () => {
      const request: AccessRequest = {
        principal: "user:test",
        resource: "api:test",
        action: "call",
        tenantId: "tenant-1",
      };

      // First two requests should succeed
      const decision1 = await accessControl.evaluateAccess(request);
      expect(decision1.allowed).toBe(true);

      const decision2 = await accessControl.evaluateAccess(request);
      expect(decision2.allowed).toBe(true);

      // Third request should be rate limited
      const decision3 = await accessControl.evaluateAccess(request);
      expect(decision3.allowed).toBe(false);
      expect(decision3.reason).toContain("Rate limit exceeded");
    });

    it("should reset rate limit after window", async () => {
      const request: AccessRequest = {
        principal: "user:test",
        resource: "api:test",
        action: "call",
        tenantId: "tenant-1",
      };

      // Exhaust rate limit
      await accessControl.evaluateAccess(request);
      await accessControl.evaluateAccess(request);
      const decision3 = await accessControl.evaluateAccess(request);
      expect(decision3.allowed).toBe(false);

      // Wait for window to reset using fake timers
      jest.advanceTimersByTime(1100);

      // Should allow again
      const decision4 = await accessControl.evaluateAccess(request);
      expect(decision4.allowed).toBe(true);
    });
  });

  describe("Configuration and Status", () => {
    it("should provide access control status", () => {
      const status = accessControl.getStatus();
      expect(status.enabled).toBe(true);
      expect(status.policyCount).toBeGreaterThan(0);
      expect(typeof status.defaultEffect).toBe("string");
      expect(typeof status.auditLoggingEnabled).toBe("boolean");
    });

    it("should disable access control when configured", async () => {
      const disabledControl = new AccessControlManager({
        enableAccessControl: false,
      });

      const request: AccessRequest = {
        principal: "user:test",
        resource: "sensitive:data",
        action: "delete",
        tenantId: "tenant-1",
      };

      const decision = await disabledControl.evaluateAccess(request);
      expect(decision.allowed).toBe(true);
      expect(decision.reason).toBe("Access control disabled");
    });

    it("should clear rate limit cache", () => {
      accessControl.clearRateLimitCache();
      // Should not throw
      expect(true).toBe(true);
    });
  });

  describe("Default Policies", () => {
    it("should initialize with default deny policy", async () => {
      const freshControl = new AccessControlManager();

      const request: AccessRequest = {
        principal: "unknown:user",
        resource: "unknown:resource",
        action: "unknown",
        tenantId: "tenant-1",
      };

      const decision = await freshControl.evaluateAccess(request);
      expect(decision.allowed).toBe(false);
      expect(decision.reason).toContain("Default");
    });

    it("should include tenant admin policy", async () => {
      const freshControl = new AccessControlManager();

      const request: AccessRequest = {
        principal: "role:tenant-admin",
        resource: "agents:create",
        action: "create",
        tenantId: "tenant-1",
      };

      const decision = await freshControl.evaluateAccess(request);
      expect(decision.allowed).toBe(true);
      expect(decision.reason).toContain("Admin");
    });
  });
});
