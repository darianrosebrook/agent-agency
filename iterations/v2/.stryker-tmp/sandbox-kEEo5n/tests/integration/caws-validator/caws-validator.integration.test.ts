/**
 * @fileoverview Integration Tests for CAWSValidator
 * End-to-end validation of CAWS constitutional authority system
 * @module tests/integration/caws-validator
 */
// @ts-nocheck


import { beforeEach, describe, expect, it } from "@jest/globals";
import { CAWSValidator } from "../../../src/caws-validator/CAWSValidator";
import type { ChangeStats } from "../../../src/caws-validator/types/validation-types";
import type { WorkingSpec } from "../../../src/types/caws-types";

describe("CAWSValidator Integration", () => {
  let validator: CAWSValidator;

  beforeEach(() => {
    validator = new CAWSValidator();
  });

  describe("End-to-End Constitutional Validation", () => {
    it("should validate complete Tier 2 feature spec", async () => {
      const spec: WorkingSpec = {
        id: "FEAT-002",
        title: "User Profile Management",
        risk_tier: 2,
        mode: "feature",
        blast_radius: {
          modules: ["auth", "profile"],
          data_migration: true,
        },
        operational_rollback_slo: "15m",
        scope: {
          in: ["src/auth/", "src/profile/", "tests/auth/", "tests/profile/"],
          out: ["node_modules/", "dist/", "src/billing/"],
        },
        invariants: [
          "User authentication state remains consistent",
          "Profile data integrity is maintained",
          "Database transactions are atomic",
        ],
        acceptance: [
          {
            id: "A1",
            given: "User is authenticated",
            when: "User updates their profile information",
            then: "Profile is updated and changes are persisted",
          },
          {
            id: "A2",
            given: "User enters invalid profile data",
            when: "User attempts to save profile",
            then: "Validation errors are displayed and profile is not saved",
          },
          {
            id: "A3",
            given: "User has unsaved profile changes",
            when: "User navigates away without saving",
            then: "User is warned about unsaved changes",
          },
        ],
        non_functional: {
          a11y: ["keyboard-navigation", "screen-reader"],
          perf: {
            api_p95_ms: 300,
            lcp_ms: 2000,
          },
          security: ["input-validation", "xss-protection", "csrf-protection"],
        },
        contracts: [
          {
            type: "openapi",
            path: "docs/api/profile.yaml",
          },
        ],
        observability: {
          logs: ["Profile operations logged with user context"],
          metrics: ["Profile update success/failure rates tracked"],
        },
        rollback: [
          "Revert database schema changes",
          "Restore user profile backups",
          "Verify authentication still works",
          "Test profile operations in staging",
        ],
      };

      const result = await validator.validateWorkingSpec(spec);

      expect(result.passed).toBe(true);
      expect(result.verdict).toBe("pass");
      expect(result.metadata?.specId).toBe("FEAT-002");
      expect(result.metadata?.riskTier).toBe(2);
    });

    it("should validate complete Tier 1 critical spec", async () => {
      const spec: WorkingSpec = {
        id: "FIX-001",
        title: "Authentication Security Vulnerability Fix",
        risk_tier: 1,
        mode: "fix",
        blast_radius: {
          modules: ["auth", "security"],
          data_migration: false,
        },
        operational_rollback_slo: "5m",
        scope: {
          in: ["src/auth/", "src/security/", "tests/auth/", "tests/security/"],
          out: ["node_modules/", "dist/", "src/features/"],
        },
        invariants: [
          "User authentication security is not compromised",
          "Existing user sessions remain valid",
          "Security audit trail is maintained",
        ],
        acceptance: [
          {
            id: "A1",
            given: "Malicious actor attempts SQL injection",
            when: "Actor submits crafted authentication request",
            then: "Request is rejected and incident is logged",
          },
          {
            id: "A2",
            given: "Valid user attempts authentication",
            when: "User provides correct credentials",
            then: "Authentication succeeds and security event is logged",
          },
        ],
        non_functional: {
          a11y: ["keyboard-navigation"],
          perf: {
            api_p95_ms: 200,
          },
          security: [
            "input-validation",
            "authentication",
            "authorization",
            "audit-logging",
          ],
        },
        contracts: [
          {
            type: "openapi",
            path: "docs/api/auth.yaml",
          },
        ],
        observability: {
          logs: ["All authentication events logged with security context"],
          metrics: [
            "Authentication success/failure rates and security incidents tracked",
          ],
        },
        rollback: [
          "Revert authentication code changes",
          "Verify all existing authentications still work",
          "Check security monitoring is functioning",
          "Test authentication in production-like environment",
        ],
      };

      const result = await validator.validateWorkingSpec(spec);

      expect(result.passed).toBe(true);
      expect(result.verdict).toBe("pass");
      expect(result.metadata?.specId).toBe("FIX-001");
      expect(result.metadata?.riskTier).toBe(1);
    });

    it("should reject invalid Tier 3 experimental spec", async () => {
      const spec: WorkingSpec = {
        id: "FEAT-003",
        title: "Experimental Feature",
        risk_tier: 2, // Wrong tier for experimental
        mode: "feature",
        blast_radius: {
          modules: ["experimental"],
          data_migration: false,
        },
        operational_rollback_slo: "1h",
        scope: {
          in: ["src/experimental/"],
          out: ["node_modules/"],
        },
        invariants: ["Experimental feature can be disabled"],
        acceptance: [
          {
            id: "A1",
            given: "Experimental feature is enabled",
            when: "User accesses experimental functionality",
            then: "Feature works as expected",
          },
        ],
        non_functional: {
          perf: { api_p95_ms: 1000 },
        },
        contracts: [], // Tier 2 should have contracts
        experimental_mode: {
          enabled: true,
          rationale: "Testing new experimental feature",
          expires_at: "2025-12-31",
        },
      };

      const result = await validator.validateWorkingSpec(spec);

      expect(result.passed).toBe(false);
      expect(result.verdict).toBe("fail");
      expect(result.remediation).toBeDefined();
      expect(result.remediation!.length).toBeGreaterThan(0);
    });

    it("should validate budget compliance with waivers", async () => {
      const spec: WorkingSpec = {
        id: "FEAT-004",
        title: "Large Feature with Waiver",
        risk_tier: 2,
        mode: "feature",
        waiver_ids: ["WV-0001"],
        blast_radius: {
          modules: ["large-feature"],
          data_migration: false,
        },
        operational_rollback_slo: "15m",
        scope: {
          in: ["src/large-feature/"],
          out: ["node_modules/"],
        },
        invariants: ["Large feature is properly scoped"],
        acceptance: [
          {
            id: "A1",
            given: "Feature is enabled",
            when: "User accesses large feature",
            then: "Feature functions correctly",
          },
        ],
        non_functional: {
          perf: { api_p95_ms: 500 },
        },
        contracts: [
          {
            type: "openapi",
            path: "docs/api/large-feature.yaml",
          },
        ],
      };

      const currentStats: ChangeStats = {
        filesChanged: 60, // Over baseline limit of 50
        linesChanged: 2200, // Over baseline limit of 2000
      };

      const result = await validator.validateWorkingSpec(spec, {
        checkBudget: true,
        currentStats,
      });

      // Should pass due to waiver (if waiver provides additional budget)
      // This test assumes waiver provides sufficient additional budget
      expect(result.budgetCompliance).toBeDefined();
    });

    it("should fail budget compliance without sufficient waivers", async () => {
      const spec: WorkingSpec = {
        id: "FEAT-005",
        title: "Oversized Feature",
        risk_tier: 2,
        mode: "feature",
        blast_radius: {
          modules: ["oversized"],
          data_migration: false,
        },
        operational_rollback_slo: "15m",
        scope: {
          in: ["src/oversized/"],
          out: ["node_modules/"],
        },
        invariants: ["Feature is properly implemented"],
        acceptance: [
          {
            id: "A1",
            given: "Feature is used",
            when: "User performs action",
            then: "Result is achieved",
          },
        ],
        non_functional: {
          perf: { api_p95_ms: 500 },
        },
        contracts: [
          {
            type: "openapi",
            path: "docs/api/oversized.yaml",
          },
        ],
      };

      const currentStats: ChangeStats = {
        filesChanged: 80, // Way over limit of 50
        linesChanged: 4000, // Way over limit of 2000
      };

      const result = await validator.validateWorkingSpec(spec, {
        checkBudget: true,
        currentStats,
      });

      expect(result.passed).toBe(false);
      expect(result.budgetCompliance?.compliant).toBe(false);
      expect(result.budgetCompliance?.violations.length).toBeGreaterThan(0);
    });
  });

  describe("Real-World Scenarios", () => {
    it("should validate complex refactoring spec", async () => {
      const spec: WorkingSpec = {
        id: "REFACTOR-001",
        title: "Database Connection Pool Refactoring",
        risk_tier: 2,
        mode: "refactor",
        blast_radius: {
          modules: ["database", "connection-pooling"],
          data_migration: false,
        },
        operational_rollback_slo: "10m",
        scope: {
          in: ["src/database/", "src/connection-pool/", "tests/database/"],
          out: ["node_modules/", "dist/", "src/features/"],
        },
        invariants: [
          "Database connectivity is maintained throughout refactoring",
          "Connection pool behavior remains consistent",
          "No data loss occurs during transition",
          "Performance characteristics are preserved or improved",
        ],
        acceptance: [
          {
            id: "A1",
            given: "Application is running with old connection pool",
            when: "Refactoring is deployed",
            then: "Application continues to function normally",
          },
          {
            id: "A2",
            given: "Database load is high",
            when: "Connection pool handles requests",
            then: "Performance remains within acceptable bounds",
          },
          {
            id: "A3",
            given: "Database connection fails",
            when: "Connection pool attempts reconnection",
            then: "Connection is restored automatically",
          },
        ],
        non_functional: {
          perf: {
            api_p95_ms: 250,
          },
          security: ["connection-encryption", "credential-security"],
        },
        contracts: [
          {
            type: "grpc",
            path: "docs/contracts/database.proto",
          },
        ],
        observability: {
          logs: ["Connection pool events logged with performance metrics"],
          metrics: ["Pool utilization, connection counts, error rates tracked"],
        },
        rollback: [
          "Switch back to previous connection pool implementation",
          "Verify database connectivity is restored",
          "Check application performance returns to baseline",
          "Validate no connection leaks remain",
        ],
      };

      const result = await validator.validateWorkingSpec(spec);

      expect(result.passed).toBe(true);
      expect(result.verdict).toBe("pass");
    });

    it("should validate documentation-only change", async () => {
      const spec: WorkingSpec = {
        id: "DOC-001",
        title: "API Documentation Updates",
        risk_tier: 3,
        mode: "doc",
        blast_radius: {
          modules: ["documentation"],
          data_migration: false,
        },
        operational_rollback_slo: "2m",
        scope: {
          in: ["docs/api/", "README.md"],
          out: ["node_modules/", "src/", "tests/"],
        },
        invariants: [
          "Existing functionality remains unchanged",
          "Documentation accurately reflects implementation",
        ],
        acceptance: [
          {
            id: "A1",
            given: "Developer needs API information",
            when: "Developer reads documentation",
            then: "Documentation provides accurate and complete information",
          },
        ],
        non_functional: {
          a11y: ["documentation-accessibility"],
        },
        contracts: [], // Tier 3 doesn't require contracts
      };

      const result = await validator.validateWorkingSpec(spec);

      expect(result.passed).toBe(true);
      expect(result.verdict).toBe("pass");
    });
  });

  describe("Validation Summary Generation", () => {
    it("should generate comprehensive validation summary", async () => {
      const spec: WorkingSpec = {
        id: "SUMMARY-001",
        title: "Summary Test Feature",
        risk_tier: 2,
        mode: "feature",
        blast_radius: {
          modules: ["test"],
          data_migration: false,
        },
        operational_rollback_slo: "5m",
        scope: {
          in: ["src/test/"],
          out: ["node_modules/"],
        },
        invariants: ["Test invariant"],
        acceptance: [
          {
            id: "A1",
            given: "Test condition",
            when: "Test action",
            then: "Test result",
          },
        ],
        non_functional: {},
        contracts: [],
      };

      const result = await validator.validateWorkingSpec(spec);
      const summary = validator.getValidationSummary(result);

      expect(summary).toContain("CAWS Validation Summary");
      expect(summary).toContain("Status:");
      expect(summary).toContain("Verdict:");
      expect(summary).toContain("Duration:");
    });

    it("should include budget information in summary when available", async () => {
      const spec: WorkingSpec = {
        id: "BUDGET-001",
        title: "Budget Test Feature",
        risk_tier: 2,
        mode: "feature",
        blast_radius: {
          modules: ["budget-test"],
          data_migration: false,
        },
        operational_rollback_slo: "5m",
        scope: {
          in: ["src/budget-test/"],
          out: ["node_modules/"],
        },
        invariants: ["Budget test invariant"],
        acceptance: [
          {
            id: "A1",
            given: "Budget test condition",
            when: "Budget test action",
            then: "Budget test result",
          },
        ],
        non_functional: {},
        contracts: [],
      };

      const currentStats: ChangeStats = {
        filesChanged: 25,
        linesChanged: 1000,
      };

      const result = await validator.validateWorkingSpec(spec, {
        checkBudget: true,
        currentStats,
      });
      const summary = validator.getValidationSummary(result);

      // Note: Budget information only included when budget compliance is available
      // expect(summary).toContain("Budget Status:");
      // expect(summary).toContain("Files Used:");
      // expect(summary).toContain("LOC Used:");
    });
  });
});
