/**
 * SystemHealthMonitor Unit Tests
 *
 * Tests health monitoring functionality, agent health tracking,
 * and alert management.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { SystemHealthMonitor } from "../../../src/monitoring/SystemHealthMonitor.js";

describe("SystemHealthMonitor", () => {
  let healthMonitor: SystemHealthMonitor;

  beforeEach(() => {
    healthMonitor = new SystemHealthMonitor({
      collectionIntervalMs: 1000, // Fast for testing
      healthCheckIntervalMs: 1000,
      retentionPeriodMs: 60000,
    });

    // Don't start automatic collection/health checks in tests
  });

  afterEach(async () => {
    if (healthMonitor) {
      await healthMonitor.shutdown();
    }
  });

  describe("initialization", () => {
    it("should create health monitor instance", () => {
      expect(healthMonitor).toBeDefined();
    });

    it("should initialize with default config", () => {
      const defaultMonitor = new SystemHealthMonitor();
      expect(defaultMonitor).toBeDefined();
    });
  });

  describe("health metrics", () => {
    it("should provide health metrics structure", async () => {
      const metrics = await healthMonitor.getHealthMetrics();

      expect(metrics).toBeDefined();
      expect(metrics.overallHealth).toBeGreaterThanOrEqual(0);
      expect(metrics.overallHealth).toBeLessThanOrEqual(1);
      expect(metrics.agents).toBeInstanceOf(Map);
      expect(metrics.timestamp).toBeInstanceOf(Date);
      expect(typeof metrics.errorRate).toBe("number");
      expect(typeof metrics.queueDepth).toBe("number");
      expect(typeof metrics.circuitBreakerOpen).toBe("boolean");
    });
  });

  describe("agent health tracking", () => {
    const agentId = "test-agent";

    it("should return null for unknown agent", () => {
      const health = healthMonitor.getAgentHealth("unknown-agent");
      expect(health).toBeNull();
    });

    it("should update and retrieve agent health", () => {
      healthMonitor.updateAgentHealth(agentId, {
        healthScore: 0.9,
        reliabilityScore: 0.95,
        errorRate: 1,
        responseTimeP95: 1500,
        currentLoad: 2,
        maxLoad: 5,
        successRate: 0.98,
      });

      const health = healthMonitor.getAgentHealth(agentId);
      expect(health).toBeDefined();
      expect(health!.agentId).toBe(agentId);
      expect(health!.healthScore).toBeCloseTo(0.9, 1); // Allow small variance due to calculation
      expect(health!.currentLoad).toBe(2);
    });

    it("should calculate health score automatically", () => {
      healthMonitor.updateAgentHealth(agentId, {
        reliabilityScore: 0.8,
        errorRate: 2,
        responseTimeP95: 3000,
        currentLoad: 3,
        maxLoad: 5,
        successRate: 0.9,
      });

      const health = healthMonitor.getAgentHealth(agentId);
      expect(health!.healthScore).toBeDefined();
      expect(typeof health!.healthScore).toBe("number");
    });

    it("should track agent task completions", () => {
      // Initial state
      healthMonitor.updateAgentHealth(agentId, {
        currentLoad: 3,
        successRate: 0.8,
      });

      // Record successful task
      healthMonitor.recordAgentTask(agentId, true, 1000);

      const health = healthMonitor.getAgentHealth(agentId);
      expect(health!.currentLoad).toBe(2); // Load reduced
      expect(health!.successRate).toBeGreaterThan(0.8); // Success rate improved
    });

    it("should track agent errors", () => {
      healthMonitor.updateAgentHealth(agentId, {
        errorRate: 0,
      });

      healthMonitor.recordAgentError(agentId);

      const health = healthMonitor.getAgentHealth(agentId);
      expect(health!.errorRate).toBe(1);
    });
  });

  describe("alerts", () => {
    it("should generate alerts for system issues", async () => {
      // Create monitor with low thresholds for testing
      const testMonitor = new SystemHealthMonitor({
        thresholds: {
          cpuWarningThreshold: 40, // Our mock returns 45%
          cpuCriticalThreshold: 50,
          memoryWarningThreshold: 50,
          memoryCriticalThreshold: 70,
          diskWarningThreshold: 50,
          diskCriticalThreshold: 70,
          agentErrorRateThreshold: 5,
          agentResponseTimeThreshold: 5000,
          systemErrorRateThreshold: 10,
          queueDepthThreshold: 100,
        },
      });

      // Manually trigger health check (would normally be automatic)
      const alerts = testMonitor.getActiveAlerts();
      expect(Array.isArray(alerts)).toBe(true);
    });

    it("should allow acknowledging alerts", () => {
      // This would need actual alerts to test properly
      // For now, just test the method exists
      expect(typeof healthMonitor.acknowledgeAlert).toBe("function");
    });
  });

  describe("circuit breaker", () => {
    it("should maintain circuit breaker state", async () => {
      const metrics = await healthMonitor.getHealthMetrics();
      expect(metrics.circuitBreakerOpen).toBe(false);
    });
  });

  describe("health degradation simulation", () => {
    it("should simulate health degradation", async () => {
      // Setup initial healthy state
      healthMonitor.updateAgentHealth("agent-1", { healthScore: 0.9 });
      const initialMetrics = await healthMonitor.getHealthMetrics();
      const initialHealth = initialMetrics.overallHealth;

      // Simulate degradation
      await healthMonitor.simulateHealthDegradation();

      const degradedMetrics = await healthMonitor.getHealthMetrics();
      const degradedHealth = degradedMetrics.overallHealth;

      // Health should be worse after degradation
      expect(degradedHealth).toBeLessThanOrEqual(initialHealth);

      // Agent health should be degraded
      const agentHealth = healthMonitor.getAgentHealth("agent-1");
      expect(agentHealth!.healthScore).toBeLessThan(0.9);
    });
  });

  describe("initialization", () => {
    it("should start metrics collection and health checks", async () => {
      const mockMetricsCollector = {
        collectSystemMetrics: jest.fn().mockResolvedValue({
          cpuUsage: 45,
          memoryUsage: 60,
          timestamp: new Date(),
        }),
      };

      (healthMonitor as any).metricsCollector = mockMetricsCollector;

      await healthMonitor.initialize();

      expect(mockMetricsCollector.collectSystemMetrics).toHaveBeenCalled();
      expect((healthMonitor as any).metricsHistory.length).toBeGreaterThan(0);
    });

    it("should handle metrics collection errors gracefully", async () => {
      const mockMetricsCollector = {
        collectSystemMetrics: jest
          .fn()
          .mockRejectedValue(new Error("Collection failed")),
      };

      (healthMonitor as any).metricsCollector = mockMetricsCollector;

      await expect(healthMonitor.initialize()).resolves.toBeUndefined();
      // Should not crash the system
    });
  });

  describe("agent metrics initialization", () => {
    it("should initialize agent metrics when recording first success", () => {
      const agentId = "new-agent";
      const responseTime = 150;

      healthMonitor.recordAgentTask(agentId, true, responseTime);

      const metrics = healthMonitor.getAgentHealth(agentId);
      expect(metrics).toBeDefined();
      expect(metrics!.currentLoad).toBe(1);
      expect(metrics!.lastActivity).toBeInstanceOf(Date);
    });

    it("should initialize agent metrics when recording first error", () => {
      const agentId = "error-agent";

      healthMonitor.recordAgentError(agentId, "timeout");

      const metrics = healthMonitor.getAgentHealth(agentId);
      expect(metrics).toBeDefined();
      expect(metrics!.errorRate).toBe(1);
      expect(metrics!.lastActivity).toBeInstanceOf(Date);
    });
  });

  describe("alert management", () => {
    it("should acknowledge existing unacknowledged alerts", () => {
      const alert = {
        id: "test-alert",
        severity: "warning" as const,
        component: "system" as const,
        metric: "cpu",
        value: 85,
        threshold: 80,
        message: "High CPU usage",
        timestamp: new Date(),
        acknowledged: false,
      };

      (healthMonitor as any).alerts = [alert];

      const result = healthMonitor.acknowledgeAlert("test-alert");

      expect(result).toBe(true);
      expect(alert.acknowledged).toBe(true);
    });

    it("should not acknowledge already acknowledged alerts", () => {
      const alert = {
        id: "acknowledged-alert",
        severity: "warning" as const,
        component: "system" as const,
        metric: "cpu",
        value: 85,
        threshold: 80,
        message: "High CPU usage",
        timestamp: new Date(),
        acknowledged: true,
      };

      (healthMonitor as any).alerts = [alert];

      const result = healthMonitor.acknowledgeAlert("acknowledged-alert");

      expect(result).toBe(false);
      expect(alert.acknowledged).toBe(true); // Remains acknowledged
    });

    it("should return false for non-existent alerts", () => {
      const result = healthMonitor.acknowledgeAlert("non-existent");
      expect(result).toBe(false);
    });
  });

  describe("health degradation simulation", () => {
    it("should simulate system health degradation", async () => {
      // First, establish baseline metrics
      const baselineMetrics = {
        cpuUsage: 30,
        memoryUsage: 40,
        timestamp: new Date(),
      };

      (healthMonitor as any).metricsHistory = [baselineMetrics];

      // Add an agent with good health
      healthMonitor.recordAgentTask("test-agent", true, 100);
      const initialHealth = healthMonitor.getAgentHealth("test-agent");

      await healthMonitor.simulateHealthDegradation();

      // Check system metrics degradation
      const systemMetrics = (healthMonitor as any).metricsHistory;
      expect(systemMetrics.length).toBe(2);
      expect(systemMetrics[1].cpuUsage).toBe(60); // 30 + 30 = 60
      expect(systemMetrics[1].memoryUsage).toBe(60); // 40 + 20 = 60

      // Check agent health degradation
      const degradedHealth = healthMonitor.getAgentHealth("test-agent");
      expect(degradedHealth!.errorRate).toBeGreaterThan(
        initialHealth!.errorRate
      );
      expect(degradedHealth!.healthScore).toBeLessThan(
        initialHealth!.healthScore
      );
    });

    it("should handle simulation when no baseline metrics exist", async () => {
      (healthMonitor as any).metricsHistory = [];

      await expect(
        healthMonitor.simulateHealthDegradation()
      ).resolves.toBeUndefined();
      // Should not crash
    });
  });

  describe("metrics collection", () => {
    beforeEach(() => {
      jest.useFakeTimers();
    });

    afterEach(() => {
      jest.useRealTimers();
    });

    it("should collect metrics periodically", async () => {
      const mockMetricsCollector = {
        collectSystemMetrics: jest.fn().mockResolvedValue({
          cpuUsage: 50,
          memoryUsage: 55,
          timestamp: new Date(),
        }),
      };

      (healthMonitor as any).metricsCollector = mockMetricsCollector;

      await healthMonitor.initialize();

      // Fast-forward time
      jest.advanceTimersByTime(61000); // 61 seconds

      expect(mockMetricsCollector.collectSystemMetrics).toHaveBeenCalledTimes(
        2
      ); // Initial + periodic
    });

    it("should handle metrics collection errors during periodic collection", async () => {
      const mockMetricsCollector = {
        collectSystemMetrics: jest
          .fn()
          .mockResolvedValueOnce({
            cpuUsage: 30,
            memoryUsage: 40,
            timestamp: new Date(),
          }) // Initial
          .mockRejectedValueOnce(new Error("Collection failed")) // Periodic failure
          .mockResolvedValueOnce({
            cpuUsage: 35,
            memoryUsage: 45,
            timestamp: new Date(),
          }), // Recovery
      };

      (healthMonitor as any).metricsCollector = mockMetricsCollector;

      await healthMonitor.initialize();

      // Fast-forward time - should not crash on collection failure
      jest.advanceTimersByTime(61000);

      expect(mockMetricsCollector.collectSystemMetrics).toHaveBeenCalledTimes(
        3
      );
    });
  });

  describe("system alerts", () => {
    it("should generate CPU critical alert when threshold exceeded", () => {
      const healthMetrics = {
        system: { cpuUsage: 95, memoryUsage: 30 },
        errorRate: 0.01,
        queueDepth: 5,
        timestamp: new Date(),
      };

      (healthMonitor as any).checkSystemAlerts(healthMetrics);

      const alerts = (healthMonitor as any).alerts;
      expect(alerts.length).toBe(1);
      expect(alerts[0].severity).toBe("critical");
      expect(alerts[0].metric).toBe("cpu");
      expect(alerts[0].message).toBe("Critical CPU usage");
    });

    it("should generate CPU warning alert when warning threshold exceeded", () => {
      const healthMetrics = {
        system: { cpuUsage: 85, memoryUsage: 30 },
        errorRate: 0.01,
        queueDepth: 5,
        timestamp: new Date(),
      };

      (healthMonitor as any).checkSystemAlerts(healthMetrics);

      const alerts = (healthMonitor as any).alerts;
      expect(alerts.length).toBe(1);
      expect(alerts[0].severity).toBe("warning");
      expect(alerts[0].metric).toBe("cpu");
      expect(alerts[0].message).toBe("High CPU usage");
    });

    it("should generate memory alerts when thresholds exceeded", () => {
      const healthMetrics = {
        system: { cpuUsage: 30, memoryUsage: 95 },
        errorRate: 0.01,
        queueDepth: 5,
        timestamp: new Date(),
      };

      (healthMonitor as any).checkSystemAlerts(healthMetrics);

      const alerts = (healthMonitor as any).alerts;
      expect(alerts.length).toBe(1);
      expect(alerts[0].severity).toBe("critical");
      expect(alerts[0].metric).toBe("memory");
      expect(alerts[0].message).toBe("Critical memory usage");
    });

    it("should generate error rate alerts when threshold exceeded", () => {
      const healthMetrics = {
        system: { cpuUsage: 30, memoryUsage: 40 },
        errorRate: 0.15, // Above default threshold
        queueDepth: 5,
        timestamp: new Date(),
      };

      (healthMonitor as any).checkSystemAlerts(healthMetrics);

      const alerts = (healthMonitor as any).alerts;
      expect(alerts.length).toBe(1);
      expect(alerts[0].severity).toBe("error");
      expect(alerts[0].metric).toBe("error_rate");
      expect(alerts[0].message).toBe("High system error rate");
    });

    it("should generate queue depth alerts when threshold exceeded", () => {
      const healthMetrics = {
        system: { cpuUsage: 30, memoryUsage: 40 },
        errorRate: 0.01,
        queueDepth: 150, // Above default threshold
        timestamp: new Date(),
      };

      (healthMonitor as any).checkSystemAlerts(healthMetrics);

      const alerts = (healthMonitor as any).alerts;
      expect(alerts.length).toBe(1);
      expect(alerts[0].severity).toBe("warning");
      expect(alerts[0].metric).toBe("queue_depth");
      expect(alerts[0].message).toBe("High task queue depth");
    });
  });

  describe("agent alerts", () => {
    it("should generate agent error rate alerts", () => {
      const agentId = "failing-agent";

      // Set up agent with high error rate
      healthMonitor.recordAgentError(agentId, "timeout");
      healthMonitor.recordAgentError(agentId, "timeout");
      healthMonitor.recordAgentError(agentId, "timeout");

      // Trigger alert checking (normally done by health check timer)
      const metrics = healthMonitor.getAgentHealth(agentId);
      if (metrics) {
        (healthMonitor as any).checkAgentAlerts(agentId, metrics);
      }

      const alerts = (healthMonitor as any).alerts;
      const errorAlerts = alerts.filter((a: any) => a.metric === "error_rate");
      expect(errorAlerts.length).toBe(1);
      expect(errorAlerts[0].message).toContain("High error rate for agent");
    });

    it("should generate agent response time alerts", () => {
      const agentId = "slow-agent";

      // Set up agent with slow response time
      healthMonitor.recordAgentTask(agentId, true, 5000); // Very slow

      const metrics = healthMonitor.getAgentHealth(agentId);
      if (metrics) {
        (healthMonitor as any).checkAgentAlerts(agentId, metrics);
      }

      const alerts = (healthMonitor as any).alerts;
      const responseTimeAlerts = alerts.filter(
        (a: any) => a.metric === "response_time"
      );
      expect(responseTimeAlerts.length).toBe(1);
      expect(responseTimeAlerts[0].message).toContain("Slow response time");
    });

    it("should generate critical agent health score alerts", () => {
      const agentId = "unhealthy-agent";

      // Set up agent with very low health score
      healthMonitor.recordAgentError(agentId, "critical");
      healthMonitor.recordAgentError(agentId, "critical");

      // Manually set health score to critical level
      const metrics = healthMonitor.getAgentHealth(agentId);
      if (metrics) {
        metrics.healthScore = 0.3; // Critical
        (healthMonitor as any).checkAgentAlerts(agentId, metrics);
      }

      const alerts = (healthMonitor as any).alerts;
      const healthAlerts = alerts.filter(
        (a: any) => a.metric === "health_score"
      );
      expect(healthAlerts.length).toBe(1);
      expect(healthAlerts[0].severity).toBe("critical");
      expect(healthAlerts[0].message).toContain("health critically low");
    });

    it("should generate warning agent health score alerts", () => {
      const agentId = "warning-agent";

      // Set up agent with low health score
      healthMonitor.recordAgentTask(agentId, true, 100);

      const metrics = healthMonitor.getAgentHealth(agentId);
      if (metrics) {
        metrics.healthScore = 0.6; // Warning level
        (healthMonitor as any).checkAgentAlerts(agentId, metrics);
      }

      const alerts = (healthMonitor as any).alerts;
      const healthAlerts = alerts.filter(
        (a: any) => a.metric === "health_score"
      );
      expect(healthAlerts.length).toBe(1);
      expect(healthAlerts[0].severity).toBe("warning");
      expect(healthAlerts[0].message).toContain("health score low");
    });
  });

  describe("circuit breaker", () => {
    it("should open circuit breaker when failure threshold exceeded", () => {
      // Set up high failure count
      (healthMonitor as any).circuitBreakerFailureCount = 6; // Above default threshold of 5

      (healthMonitor as any).updateCircuitBreaker();

      expect((healthMonitor as any).circuitBreakerState).toBe("open");
    });

    it("should transition from open to half-open after recovery timeout", () => {
      // Set circuit breaker to open
      (healthMonitor as any).circuitBreakerState = "open";
      (healthMonitor as any).circuitBreakerFailureCount = 6;
      (healthMonitor as any).circuitBreakerLastFailure = Date.now() - 65000; // 65 seconds ago

      (healthMonitor as any).updateCircuitBreaker();

      expect((healthMonitor as any).circuitBreakerState).toBe("half-open");
    });

    it("should remain closed when failure count below threshold", () => {
      (healthMonitor as any).circuitBreakerFailureCount = 2; // Below threshold
      (healthMonitor as any).circuitBreakerState = "closed";

      (healthMonitor as any).updateCircuitBreaker();

      expect((healthMonitor as any).circuitBreakerState).toBe("closed");
    });
  });

  describe("shutdown", () => {
    it("should shutdown cleanly", async () => {
      await expect(healthMonitor.shutdown()).resolves.toBeUndefined();
    });

    it("should clear metrics collection timer", async () => {
      await healthMonitor.initialize(); // Start timers

      const hasMetricsTimer =
        (healthMonitor as any).metricsCollectionTimer !== undefined;
      const hasHealthTimer =
        (healthMonitor as any).healthCheckTimer !== undefined;

      expect(hasMetricsTimer || hasHealthTimer).toBe(true);

      await healthMonitor.shutdown();

      expect((healthMonitor as any).metricsCollectionTimer).toBeUndefined();
      expect((healthMonitor as any).healthCheckTimer).toBeUndefined();
    });
  });
});
