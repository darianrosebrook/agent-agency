/**
 * @fileoverview Predefined Chaos Scenarios for Testing
 *
 * Common chaos engineering scenarios for testing arbiter resilience
 * against various failure modes and edge cases.
 *
 * @author @darianrosebrook
 */

import { ChaosScenario } from "./ChaosTestingHarness";

export const CHAOS_SCENARIOS: ChaosScenario[] = [
  // Worker Failure Scenarios
  {
    id: "worker-crash-random",
    name: "Random Worker Crash",
    description: "Randomly crashes a worker during task execution",
    probability: 0.05, // 5% chance
    duration: 30000, // 30 seconds
    recoveryTime: 60000, // 1 minute to recover
    conditions: [
      {
        type: "task_load",
        operator: ">",
        value: 0.7, // Only when task load is high
      },
    ],
  },

  {
    id: "worker-timeout-high-load",
    name: "Worker Timeout Under High Load",
    description: "Workers timeout when system load is high",
    probability: 0.1, // 10% chance
    duration: 45000, // 45 seconds
    recoveryTime: 30000, // 30 seconds to recover
    conditions: [
      {
        type: "task_load",
        operator: ">",
        value: 0.8,
      },
      {
        type: "system_load",
        operator: ">",
        value: 0.9,
      },
    ],
  },

  // Network Degradation Scenarios
  {
    id: "network-latency-spike",
    name: "Network Latency Spike",
    description:
      "Simulates network latency spikes affecting worker communication",
    probability: 0.08, // 8% chance
    duration: 60000, // 1 minute
    recoveryTime: 45000, // 45 seconds to recover
    conditions: [
      {
        type: "worker_saturation",
        operator: ">",
        value: 5, // When we have more than 5 workers
      },
    ],
  },

  {
    id: "network-partition",
    name: "Network Partition",
    description: "Simulates network partition isolating some workers",
    probability: 0.02, // 2% chance (rare but critical)
    duration: 120000, // 2 minutes
    recoveryTime: 90000, // 1.5 minutes to recover
    conditions: [
      {
        type: "worker_saturation",
        operator: ">",
        value: 3, // Need at least 3 workers to partition
      },
    ],
  },

  // Resource Exhaustion Scenarios
  {
    id: "memory-exhaustion",
    name: "Memory Exhaustion",
    description: "Simulates memory exhaustion causing worker failures",
    probability: 0.03, // 3% chance
    duration: 90000, // 1.5 minutes
    recoveryTime: 120000, // 2 minutes to recover
    conditions: [
      {
        type: "system_load",
        operator: ">",
        value: 0.85,
      },
    ],
  },

  {
    id: "cpu-exhaustion",
    name: "CPU Exhaustion",
    description: "Simulates CPU exhaustion causing worker degradation",
    probability: 0.06, // 6% chance
    duration: 75000, // 1.25 minutes
    recoveryTime: 60000, // 1 minute to recover
    conditions: [
      {
        type: "system_load",
        operator: ">",
        value: 0.9,
      },
    ],
  },

  // Time-based Scenarios
  {
    id: "peak-hour-degradation",
    name: "Peak Hour Degradation",
    description: "Simulates degradation during peak hours",
    probability: 0.12, // 12% chance
    duration: 180000, // 3 minutes
    recoveryTime: 120000, // 2 minutes to recover
    conditions: [
      {
        type: "time_of_day",
        operator: ">=",
        value: 9, // 9 AM
      },
      {
        type: "time_of_day",
        operator: "<=",
        value: 17, // 5 PM
      },
    ],
  },

  {
    id: "off-hours-resource-constraint",
    name: "Off-Hours Resource Constraint",
    description: "Simulates resource constraints during off-hours",
    probability: 0.04, // 4% chance
    duration: 150000, // 2.5 minutes
    recoveryTime: 90000, // 1.5 minutes to recover
    conditions: [
      {
        type: "time_of_day",
        operator: "<",
        value: 6, // Before 6 AM
      },
    ],
  },

  // Cascading Failure Scenarios
  {
    id: "cascading-worker-failure",
    name: "Cascading Worker Failure",
    description: "Simulates cascading failures when one worker fails",
    probability: 0.01, // 1% chance (rare but severe)
    duration: 300000, // 5 minutes
    recoveryTime: 240000, // 4 minutes to recover
    conditions: [
      {
        type: "worker_saturation",
        operator: ">",
        value: 4, // Need multiple workers for cascade
      },
      {
        type: "task_load",
        operator: ">",
        value: 0.75,
      },
    ],
  },

  // Database/Storage Scenarios
  {
    id: "database-connection-pool-exhaustion",
    name: "Database Connection Pool Exhaustion",
    description: "Simulates database connection pool exhaustion",
    probability: 0.03, // 3% chance
    duration: 120000, // 2 minutes
    recoveryTime: 90000, // 1.5 minutes to recover
    conditions: [
      {
        type: "task_load",
        operator: ">",
        value: 0.8,
      },
    ],
  },

  {
    id: "storage-io-spike",
    name: "Storage I/O Spike",
    description: "Simulates storage I/O spikes affecting persistence",
    probability: 0.05, // 5% chance
    duration: 90000, // 1.5 minutes
    recoveryTime: 60000, // 1 minute to recover
    conditions: [
      {
        type: "system_load",
        operator: ">",
        value: 0.7,
      },
    ],
  },

  // API/External Service Scenarios
  {
    id: "external-api-timeout",
    name: "External API Timeout",
    description: "Simulates external API timeouts",
    probability: 0.07, // 7% chance
    duration: 60000, // 1 minute
    recoveryTime: 45000, // 45 seconds to recover
    conditions: [
      {
        type: "task_load",
        operator: ">",
        value: 0.6,
      },
    ],
  },

  {
    id: "rate-limit-exceeded",
    name: "Rate Limit Exceeded",
    description: "Simulates rate limit exceeded on external services",
    probability: 0.04, // 4% chance
    duration: 180000, // 3 minutes
    recoveryTime: 120000, // 2 minutes to recover
    conditions: [
      {
        type: "task_load",
        operator: ">",
        value: 0.9,
      },
    ],
  },

  // Edge Case Scenarios
  {
    id: "malformed-task-payload",
    name: "Malformed Task Payload",
    description: "Simulates malformed task payloads causing worker errors",
    probability: 0.02, // 2% chance
    duration: 30000, // 30 seconds
    recoveryTime: 15000, // 15 seconds to recover
  },

  {
    id: "task-timeout-extreme",
    name: "Extreme Task Timeout",
    description: "Simulates extreme task timeouts",
    probability: 0.01, // 1% chance
    duration: 600000, // 10 minutes
    recoveryTime: 300000, // 5 minutes to recover
    conditions: [
      {
        type: "task_load",
        operator: ">",
        value: 0.95,
      },
    ],
  },
];

/**
 * Get scenarios by category
 */
export function getScenariosByCategory(
  category:
    | "worker"
    | "network"
    | "resource"
    | "time"
    | "cascading"
    | "storage"
    | "api"
    | "edge"
): ChaosScenario[] {
  return CHAOS_SCENARIOS.filter((scenario) => {
    const name = scenario.name.toLowerCase();
    switch (category) {
      case "worker":
        return (
          name.includes("worker") ||
          name.includes("crash") ||
          name.includes("timeout")
        );
      case "network":
        return (
          name.includes("network") ||
          name.includes("latency") ||
          name.includes("partition")
        );
      case "resource":
        return (
          name.includes("memory") ||
          name.includes("cpu") ||
          name.includes("exhaustion")
        );
      case "time":
        return (
          name.includes("peak") ||
          name.includes("hour") ||
          name.includes("off-hours")
        );
      case "cascading":
        return name.includes("cascading");
      case "storage":
        return (
          name.includes("database") ||
          name.includes("storage") ||
          name.includes("connection")
        );
      case "api":
        return name.includes("api") || name.includes("rate limit");
      case "edge":
        return name.includes("malformed") || name.includes("extreme");
      default:
        return false;
    }
  });
}

/**
 * Get scenarios by severity level
 */
export function getScenariosBySeverity(
  severity: "low" | "medium" | "high" | "critical"
): ChaosScenario[] {
  return CHAOS_SCENARIOS.filter((scenario) => {
    // Map probability to severity
    if (scenario.probability >= 0.1) return severity === "low";
    if (scenario.probability >= 0.05) return severity === "medium";
    if (scenario.probability >= 0.02) return severity === "high";
    return severity === "critical";
  });
}

/**
 * Get scenarios suitable for specific testing phases
 */
export function getScenariosForPhase(
  phase: "unit" | "integration" | "load" | "chaos" | "production"
): ChaosScenario[] {
  switch (phase) {
    case "unit":
      return CHAOS_SCENARIOS.filter(
        (s) => s.probability >= 0.05 && s.duration <= 60000
      );
    case "integration":
      return CHAOS_SCENARIOS.filter(
        (s) => s.probability >= 0.03 && s.duration <= 120000
      );
    case "load":
      return CHAOS_SCENARIOS.filter((s) =>
        s.conditions?.some(
          (c) => c.type === "task_load" || c.type === "system_load"
        )
      );
    case "chaos":
      return CHAOS_SCENARIOS; // All scenarios for chaos engineering
    case "production":
      return CHAOS_SCENARIOS.filter(
        (s) => s.probability <= 0.02 && s.severity !== "critical"
      );
    default:
      return [];
  }
}






