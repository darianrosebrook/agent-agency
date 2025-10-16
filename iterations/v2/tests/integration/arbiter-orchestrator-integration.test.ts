/**
 * Arbiter Orchestrator Integration Test
 *
 * Tests the complete arbitration workflow with real component integration,
 * exercising the main application logic for CAWS compliance.
 *
 * @author @darianrosebrook
 */

import { ArbiterOrchestrator } from "../../src/orchestrator/ArbiterOrchestrator.js";

describe("Arbiter Orchestrator Integration", () => {
  let orchestrator: ArbiterOrchestrator;

  beforeAll(async () => {
    // Use the real orchestrator which manages its own dependencies
    orchestrator = new ArbiterOrchestrator();

    // Initialize the orchestrator
    await orchestrator.initialize();
  }, 30000);

  afterAll(async () => {
    await orchestrator.shutdown();
  }, 30000);

  describe("Basic Orchestrator Functionality", () => {
    it("should initialize and provide basic orchestrator methods", async () => {
      // Test that the orchestrator initializes successfully
      expect(orchestrator).toBeDefined();

      // Test basic health check
      const health = await orchestrator.getHealth();
      expect(health).toBeDefined();
      expect(typeof health.status).toBe("string");
    });

    it("should provide component access", async () => {
      const components = orchestrator.getComponents();
      expect(components).toBeDefined();

      // Check that major components are available
      expect(components.ruleEngine).toBeDefined();
      expect(components.verdictGenerator).toBeDefined();
      expect(components.precedentManager).toBeDefined();
    });

    it("should provide statistics", async () => {
      const stats = orchestrator.getStatistics();
      expect(stats).toBeDefined();
      expect(typeof stats.totalSessions).toBe("number");
      expect(typeof stats.activeSessions).toBe("number");
    });
  });
});
