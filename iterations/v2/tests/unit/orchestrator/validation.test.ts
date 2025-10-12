/**
 * @fileoverview Tests for Validation utilities (ARBITER-005)
 *
 * @author @darianrosebrook
 */

import {
  VerificationPriority,
  ValidationUtils,
  validateTask,
} from "../../../src/orchestrator/Validation";
import { TaskType } from "../../../src/types/arbiter-orchestration";

describe("ValidationUtils", () => {
  describe("validateTask", () => {
    it("should pass validation for a valid task", () => {
      const validTask = {
        id: "task-123",
        description: "Test task",
        type: "code-editing" as TaskType,
        priority: 5,
        timeoutMs: 30000,
        attempts: 0,
        maxAttempts: 3,
        requiredCapabilities: {},
        budget: {
          maxFiles: 10,
          maxLoc: 100,
        },
        createdAt: new Date(),
        metadata: {},
      };

      const result = ValidationUtils.validateTask(validTask);
      expect(result.isValid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("should fail validation for missing required fields", () => {
      const invalidTask = {
        description: "Test task",
        // Missing id, type, priority, etc.
      };

      const result = ValidationUtils.validateTask(invalidTask);
      expect(result.isValid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it("should fail validation for invalid task type", () => {
      const invalidTask = {
        id: "task-123",
        description: "Test task",
        type: "invalid-type" as any,
        priority: 5,
        timeoutMs: 30000,
        attempts: 0,
        maxAttempts: 3,
        requiredCapabilities: {},
        budget: {
          maxFiles: 10,
          maxLoc: 100,
        },
        createdAt: new Date(),
        metadata: {},
      };

      const result = ValidationUtils.validateTask(invalidTask);
      expect(result.isValid).toBe(false);
      expect(result.errors.some((e) => e.code === "INVALID_TYPE")).toBe(true);
    });

    it("should fail validation for invalid priority range", () => {
      const invalidTask = {
        id: "task-123",
        description: "Test task",
        type: "code-editing" as TaskType,
        priority: 15, // Invalid: should be 1-10
        timeoutMs: 30000,
        attempts: 0,
        maxAttempts: 3,
        requiredCapabilities: {},
        budget: {
          maxFiles: 10,
          maxLoc: 100,
        },
        createdAt: new Date(),
        metadata: {},
      };

      const result = ValidationUtils.validateTask(invalidTask);
      expect(result.isValid).toBe(false);
      expect(result.errors.some((e) => e.code === "INVALID_PRIORITY")).toBe(
        true
      );
    });
  });

  describe("validateTaskQueueConfig", () => {
    it("should pass validation for valid config", () => {
      const validConfig = {
        maxCapacity: 1000,
        defaultTimeoutMs: 30000,
        maxRetries: 3,
        priorityMode: "priority" as const,
        persistenceEnabled: false,
      };

      const result = ValidationUtils.validateTaskQueueConfig(validConfig);
      expect(result.isValid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("should fail validation for invalid capacity", () => {
      const invalidConfig = {
        maxCapacity: -1, // Invalid
        defaultTimeoutMs: 30000,
        maxRetries: 3,
        priorityMode: "priority" as const,
        persistenceEnabled: false,
      };

      const result = ValidationUtils.validateTaskQueueConfig(invalidConfig);
      expect(result.isValid).toBe(false);
      expect(result.errors.some((e) => e.code === "INVALID_CAPACITY")).toBe(
        true
      );
    });
  });
});

describe("Guard Functions", () => {
  describe("validateTask guard", () => {
    it("should not throw for valid task", () => {
      const validTask = {
        id: "task-123",
        description: "Test task",
        type: "code-editing" as TaskType,
        priority: 5,
        timeoutMs: 30000,
        attempts: 0,
        maxAttempts: 3,
        requiredCapabilities: {},
        budget: {
          maxFiles: 10,
          maxLoc: 100,
        },
        createdAt: new Date(),
        metadata: {},
      };

      expect(() => validateTask(validTask)).not.toThrow();
    });

    it("should throw for invalid task", () => {
      const invalidTask = {
        description: "Test task",
        // Missing required fields
      };

      expect(() => validateTask(invalidTask)).toThrow();
    });
  });
});
