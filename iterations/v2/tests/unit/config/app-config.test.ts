/**
 * Application Configuration Tests
 *
 * @author @darianrosebrook
 */

import { describe, it, expect, beforeEach, afterEach } from "@jest/globals";
import { ConfigManager } from "../../../src/config/AppConfig";

describe("ConfigManager", () => {
  let originalEnv: NodeJS.ProcessEnv;

  beforeEach(() => {
    // Save original environment
    originalEnv = { ...process.env };
  });

  afterEach(() => {
    // Restore original environment
    process.env = originalEnv;
  });

  describe("default configuration", () => {
    it("should load default configuration", () => {
      const config = ConfigManager.getInstance().get();

      // In test environment, NODE_ENV is set to "test"
      expect(["development", "test"]).toContain(config.env);
      expect(config.server.port).toBe(3000);
      expect(config.server.host).toBe("localhost");
      expect(config.registry.maxAgents).toBe(1000);
      expect(config.routing.maxRoutingTimeMs).toBe(100);
    });

    it("should provide configuration sections", () => {
      const manager = ConfigManager.getInstance();

      const serverConfig = manager.getSection("server");
      expect(serverConfig.port).toBe(3000);

      const registryConfig = manager.getSection("registry");
      expect(registryConfig.maxAgents).toBe(1000);
    });
  });

  describe("environment variable overrides", () => {
    it("should override from environment variables", () => {
      process.env.NODE_ENV = "production";
      process.env.PORT = "8080";
      process.env.MAX_AGENTS = "5000";
      process.env.LOG_LEVEL = "warn";

      const manager = new (ConfigManager as any)();
      const config = manager.get();

      expect(config.env).toBe("production");
      expect(config.server.port).toBe(8080);
      expect(config.registry.maxAgents).toBe(5000);
      expect(config.observability.logLevel).toBe("warn");
    });

    it("should parse boolean environment variables", () => {
      process.env.TRACING_ENABLED = "false";
      process.env.CACHE_ENABLED = "true";

      const manager = new (ConfigManager as any)();
      const config = manager.get();

      expect(config.observability.tracingEnabled).toBe(false);
      expect(config.registry.cacheEnabled).toBe(true);
    });

    it("should handle numeric environment variables", () => {
      process.env.MAX_ROUTING_TIME_MS = "200";
      process.env.FAILURE_THRESHOLD = "10";

      const manager = new (ConfigManager as any)();
      const config = manager.get();

      expect(config.routing.maxRoutingTimeMs).toBe(200);
      expect(config.resilience.failureThreshold).toBe(10);
    });

    it("should use defaults for invalid numeric values", () => {
      process.env.PORT = "invalid";
      process.env.MAX_AGENTS = "not-a-number";

      const manager = new (ConfigManager as any)();
      const config = manager.get();

      expect(config.server.port).toBe(3000); // Default
      expect(config.registry.maxAgents).toBe(1000); // Default
    });
  });

  describe("configuration validation", () => {
    it("should validate port range", () => {
      process.env.PORT = "70000"; // Out of range

      expect(() => new (ConfigManager as any)()).toThrow();
    });

    it("should validate exploration rate range", () => {
      process.env.EXPLORATION_RATE = "1.5"; // Out of range

      expect(() => new (ConfigManager as any)()).toThrow();
    });

    it("should validate log level enum", () => {
      process.env.LOG_LEVEL = "invalid";

      expect(() => new (ConfigManager as any)()).toThrow();
    });
  });

  describe("configuration reload", () => {
    it("should reload configuration from environment", () => {
      const manager = ConfigManager.getInstance();
      const originalPort = manager.get().server.port;

      // Change environment
      process.env.PORT = "9000";
      manager.reload();

      const newPort = manager.get().server.port;
      expect(newPort).not.toBe(originalPort);
      expect(newPort).toBe(9000);
    });
  });

  describe("singleton pattern", () => {
    it("should return same instance", () => {
      const instance1 = ConfigManager.getInstance();
      const instance2 = ConfigManager.getInstance();

      expect(instance1).toBe(instance2);
    });
  });
});

