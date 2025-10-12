/**
 * Integration tests for CAWSPolicyAdapter
 *
 * Tests policy loading, caching, budget derivation, and waiver application.
 *
 * @author @darianrosebrook
 */

import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import * as fs from "fs/promises";
import * as path from "path";
import { CAWSPolicyAdapter } from "../../../src/caws-integration/adapters/CAWSPolicyAdapter";
import type { WorkingSpec } from "../../../src/types/caws-types";

describe("CAWSPolicyAdapter Integration Tests", () => {
  const fixturesDir = path.join(__dirname, "../../fixtures/caws-integration");
  const tempDir = path.join(__dirname, "../../temp/policy-adapter-tests");
  let adapter: CAWSPolicyAdapter;

  // Sample working spec for tests
  const sampleSpec: WorkingSpec = {
    id: "TEST-002",
    title: "Test Specification",
    risk_tier: 2,
    mode: "feature",
    blast_radius: {
      modules: ["src/test"],
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

  beforeEach(async () => {
    // Create temp directory
    await fs.mkdir(tempDir, { recursive: true });

    adapter = new CAWSPolicyAdapter({
      projectRoot: tempDir,
      enableCaching: true,
      cacheTTL: 300000,
    });
  });

  afterEach(async () => {
    // Clean up temp directory
    try {
      await fs.rm(tempDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  });

  describe("Policy Loading", () => {
    it("should load policy from fixture directory", async () => {
      const fixtureAdapter = new CAWSPolicyAdapter({
        projectRoot: fixturesDir,
        enableCaching: false,
      });

      const result = await fixtureAdapter.loadPolicy();

      expect(result.success).toBe(true);
      expect(result.data).toBeDefined();
      expect(result.data?.version).toBe("3.1.0");
      expect(result.data?.risk_tiers).toBeDefined();
    });

    it("should load default policy when policy.yaml doesn't exist", async () => {
      const result = await adapter.loadPolicy();

      expect(result.success).toBe(true);
      expect(result.data).toBeDefined();
      expect(result.data?.version).toBe("3.1.0");
      expect(result.data?.risk_tiers).toBeDefined();
    });

    it("should cache policy on first load", async () => {
      // First load
      const result1 = await adapter.loadPolicy();
      expect(result1.success).toBe(true);

      // Check cache status
      const cacheStatus = adapter.getCacheStatus();
      expect(cacheStatus.cached).toBe(true);
      expect(cacheStatus.age).toBeLessThan(100);
    });

    it("should return cached policy on second load", async () => {
      // First load (from disk)
      const result1 = await adapter.loadPolicy();

      // Second load (from cache)
      const result2 = await adapter.loadPolicy();

      expect(result2.success).toBe(true);
      expect(result2.durationMs).toBeLessThanOrEqual(result1.durationMs + 1); // Cache should be at least as fast
    });

    it("should clear cache on request", async () => {
      // Load and cache
      await adapter.loadPolicy();

      // Clear cache
      adapter.clearCache();

      // Check cache status
      const cacheStatus = adapter.getCacheStatus();
      expect(cacheStatus.cached).toBe(false);
    });

    it("should reload policy bypassing cache", async () => {
      // Load and cache
      await adapter.loadPolicy();

      // Reload (bypass cache)
      const result = await adapter.reloadPolicy();

      expect(result.success).toBe(true);

      // Should have new cached version
      const cacheStatus = adapter.getCacheStatus();
      expect(cacheStatus.cached).toBe(true);
      expect(cacheStatus.age).toBeLessThan(100); // Recently cached
    });
  });

  describe("Budget Derivation", () => {
    it("should derive baseline budget for Tier 1", async () => {
      const spec: WorkingSpec = { ...sampleSpec, risk_tier: 1 };

      const result = await adapter.deriveBudget({
        spec,
        projectRoot: tempDir,
        applyWaivers: false,
      });

      expect(result.success).toBe(true);
      expect(result.data?.baseline.max_files).toBe(10);
      expect(result.data?.baseline.max_loc).toBe(250);
      expect(result.data?.effective).toEqual(result.data?.baseline);
    });

    it("should derive baseline budget for Tier 2", async () => {
      const spec: WorkingSpec = { ...sampleSpec, risk_tier: 2 };

      const result = await adapter.deriveBudget({
        spec,
        projectRoot: tempDir,
        applyWaivers: false,
      });

      expect(result.success).toBe(true);
      expect(result.data?.baseline.max_files).toBe(100);
      expect(result.data?.baseline.max_loc).toBe(10000);
    });

    it("should derive baseline budget for Tier 3", async () => {
      const spec: WorkingSpec = { ...sampleSpec, risk_tier: 3 };

      const result = await adapter.deriveBudget({
        spec,
        projectRoot: tempDir,
        applyWaivers: false,
      });

      expect(result.success).toBe(true);
      expect(result.data?.baseline.max_files).toBe(500);
      expect(result.data?.baseline.max_loc).toBe(40000);
    });

    it("should return error for invalid risk tier", async () => {
      const spec: WorkingSpec = { ...sampleSpec, risk_tier: 5 as any };

      const result = await adapter.deriveBudget({
        spec,
        projectRoot: tempDir,
        applyWaivers: false,
      });

      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
      expect(result.error?.message).toContain("not defined in policy");
    });

    it("should include policy version in result", async () => {
      const result = await adapter.deriveBudget({
        spec: sampleSpec,
        projectRoot: tempDir,
        applyWaivers: false,
      });

      expect(result.success).toBe(true);
      expect(result.data?.policyVersion).toBe("3.1.0");
    });

    it("should include derivation timestamp", async () => {
      const result = await adapter.deriveBudget({
        spec: sampleSpec,
        projectRoot: tempDir,
        applyWaivers: false,
      });

      expect(result.success).toBe(true);
      expect(result.data?.derivedAt).toBeDefined();

      // Verify it's a valid ISO timestamp
      const timestamp = new Date(result.data!.derivedAt);
      expect(timestamp.toString()).not.toBe("Invalid Date");
    });
  });

  describe("Waiver Application", () => {
    beforeEach(async () => {
      // Create waivers directory with test waiver
      const waiversDir = path.join(tempDir, ".caws", "waivers");
      await fs.mkdir(waiversDir, { recursive: true });

      const waiver = {
        id: "WV-TEST-001",
        status: "active",
        gates: ["budget_limit"],
        expires_at: "2026-12-31T23:59:59Z",
        delta: {
          max_files: 20,
          max_loc: 1000,
        },
      };

      await fs.writeFile(
        path.join(waiversDir, "WV-TEST-001.yaml"),
        `id: ${waiver.id}
status: ${waiver.status}
gates:
  - ${waiver.gates[0]}
expires_at: "${waiver.expires_at}"
delta:
  max_files: ${waiver.delta.max_files}
  max_loc: ${waiver.delta.max_loc}
`,
        "utf-8"
      );
    });

    it("should apply waiver to budget", async () => {
      const spec: WorkingSpec = {
        ...sampleSpec,
        waiver_ids: ["WV-TEST-001"],
      };

      const result = await adapter.deriveBudget({
        spec,
        projectRoot: tempDir,
        applyWaivers: true,
      });

      expect(result.success).toBe(true);
      expect(result.data?.baseline.max_files).toBe(100);
      expect(result.data?.effective.max_files).toBe(120); // 100 + 20
      expect(result.data?.baseline.max_loc).toBe(10000);
      expect(result.data?.effective.max_loc).toBe(11000); // 10000 + 1000
      expect(result.data?.waiversApplied).toContain("WV-TEST-001");
    });

    it("should not apply waivers when disabled", async () => {
      const spec: WorkingSpec = {
        ...sampleSpec,
        waiver_ids: ["WV-TEST-001"],
      };

      const result = await adapter.deriveBudget({
        spec,
        projectRoot: tempDir,
        applyWaivers: false,
      });

      expect(result.success).toBe(true);
      expect(result.data?.effective).toEqual(result.data?.baseline);
      expect(result.data?.waiversApplied).toHaveLength(0);
    });

    it("should skip invalid waivers", async () => {
      const spec: WorkingSpec = {
        ...sampleSpec,
        waiver_ids: ["WV-NONEXISTENT"],
      };

      const result = await adapter.deriveBudget({
        spec,
        projectRoot: tempDir,
        applyWaivers: true,
      });

      expect(result.success).toBe(true);
      expect(result.data?.effective).toEqual(result.data?.baseline);
      expect(result.data?.waiversApplied).toHaveLength(0);
    });

    it("should apply multiple waivers additively", async () => {
      // Create second waiver
      const waiversDir = path.join(tempDir, ".caws", "waivers");
      await fs.writeFile(
        path.join(waiversDir, "WV-TEST-002.yaml"),
        `id: WV-TEST-002
status: active
gates:
  - budget_limit
expires_at: "2026-12-31T23:59:59Z"
delta:
  max_files: 10
  max_loc: 500
`,
        "utf-8"
      );

      const spec: WorkingSpec = {
        ...sampleSpec,
        waiver_ids: ["WV-TEST-001", "WV-TEST-002"],
      };

      const result = await adapter.deriveBudget({
        spec,
        projectRoot: tempDir,
        applyWaivers: true,
      });

      expect(result.success).toBe(true);
      expect(result.data?.effective.max_files).toBe(130); // 100 + 20 + 10
      expect(result.data?.effective.max_loc).toBe(11500); // 10000 + 1000 + 500
      expect(result.data?.waiversApplied).toHaveLength(2);
    });
  });

  describe("Integration with Fixtures", () => {
    it("should load fixture policy and derive budget", async () => {
      const fixtureAdapter = new CAWSPolicyAdapter({
        projectRoot: fixturesDir,
        enableCaching: false,
      });

      const result = await fixtureAdapter.deriveBudget({
        spec: sampleSpec,
        projectRoot: fixturesDir,
        applyWaivers: false,
      });

      expect(result.success).toBe(true);
      expect(result.data?.baseline.max_files).toBe(100);
      expect(result.data?.policyVersion).toBe("3.1.0");
    });

    it("should load and apply fixture waiver", async () => {
      const fixtureAdapter = new CAWSPolicyAdapter({
        projectRoot: fixturesDir,
        enableCaching: false,
      });

      const spec: WorkingSpec = {
        ...sampleSpec,
        waiver_ids: ["WV-0001"],
      };

      const result = await fixtureAdapter.deriveBudget({
        spec,
        projectRoot: fixturesDir,
        applyWaivers: true,
      });

      expect(result.success).toBe(true);
      expect(result.data?.effective.max_files).toBe(115); // 100 + 15
      expect(result.data?.effective.max_loc).toBe(10500); // 10000 + 500
      expect(result.data?.waiversApplied).toContain("WV-0001");
    });
  });

  describe("Cache Behavior", () => {
    it("should respect cache TTL", async () => {
      // Create adapter with very short TTL
      const shortCacheAdapter = new CAWSPolicyAdapter({
        projectRoot: tempDir,
        enableCaching: true,
        cacheTTL: 100, // 100ms
      });

      // Load policy
      await shortCacheAdapter.loadPolicy();

      // Wait for cache to expire
      await new Promise((resolve) => setTimeout(resolve, 150));

      // Load again - should reload from disk
      const result = await shortCacheAdapter.loadPolicy();

      expect(result.success).toBe(true);

      // Cache should be fresh again
      const cacheStatus = shortCacheAdapter.getCacheStatus();
      expect(cacheStatus.age).toBeLessThan(50);
    });

    it("should work without caching", async () => {
      const noCacheAdapter = new CAWSPolicyAdapter({
        projectRoot: tempDir,
        enableCaching: false,
      });

      const result1 = await noCacheAdapter.loadPolicy();
      const result2 = await noCacheAdapter.loadPolicy();

      expect(result1.success).toBe(true);
      expect(result2.success).toBe(true);

      // No cache should exist
      const cacheStatus = noCacheAdapter.getCacheStatus();
      expect(cacheStatus.cached).toBe(false);
    });
  });

  describe("Error Handling", () => {
    it("should handle corrupted policy file gracefully", async () => {
      // Create corrupted policy file
      const policyPath = path.join(tempDir, ".caws", "policy.yaml");
      await fs.mkdir(path.dirname(policyPath), { recursive: true });
      await fs.writeFile(policyPath, "invalid: yaml: structure:", "utf-8");

      const result = await adapter.loadPolicy();

      // Should fall back to default policy or return error
      expect(result).toBeDefined();
    });

    it("should provide detailed error information", async () => {
      const spec: WorkingSpec = { ...sampleSpec, risk_tier: 999 as any };

      const result = await adapter.deriveBudget({
        spec,
        projectRoot: tempDir,
        applyWaivers: false,
      });

      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
      expect(result.error?.code).toBe("INVALID_RISK_TIER");
      expect(result.error?.message).toBeDefined();
    });
  });
});
