/**
 * @fileoverview
 * Unit tests for OllamaProvider - local Ollama model integration.
 * Tests generation, health checking, and performance tracking.
 *
 * @author @darianrosebrook
 */

import {
  OllamaProvider,
  OllamaProviderError,
} from "@/models/providers/OllamaProvider";
import type { OllamaModelConfig } from "@/types/model-registry";
import { beforeEach, describe, expect, it } from "@jest/globals";

describe("OllamaProvider", () => {
  let provider: OllamaProvider;
  let mockConfig: OllamaModelConfig;

  beforeEach(() => {
    mockConfig = {
      id: "test-model-123",
      name: "gemma-fast",
      type: "ollama",
      ollamaName: "gemma3:1b",
      version: "1.0.0",
      category: "fast",
      capabilities: ["text-generation", "chat"],
      contextWindow: 8192,
      supportsStreaming: true,
      supportsBatching: false,
      quantization: "4bit",
      tokensPerSec: 130,
      memoryUsageMB: 1024,
      endpoint: "http://localhost:11434",
      createdAt: new Date(),
      updatedAt: new Date(),
      status: "active",
    };

    provider = new OllamaProvider(mockConfig);

    // Mock fetch globally
    (global as any).fetch = jest.fn();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("Configuration", () => {
    it("should initialize with Ollama config", () => {
      expect(provider.getConfig()).toEqual(mockConfig);
      expect(provider.getType()).toBe("ollama");
      expect(provider.getModelId()).toBe("test-model-123");
      expect(provider.getModelName()).toBe("gemma-fast");
      expect(provider.getEndpoint()).toBe("http://localhost:11434");
    });

    it("should use default endpoint if not provided", () => {
      const config = { ...mockConfig, endpoint: undefined };
      const defaultProvider = new OllamaProvider(config);

      expect(defaultProvider.getEndpoint()).toBe("http://localhost:11434");
    });
  });

  describe("Text Generation", () => {
    it("should generate text successfully", async () => {
      const mockResponse = {
        model: "gemma3:1b",
        created_at: "2025-10-13T00:00:00Z",
        response: "Hello! How can I help you today?",
        done: true,
        total_duration: 500000000, // 500ms in nanoseconds
        load_duration: 100000000,
        prompt_eval_count: 5,
        prompt_eval_duration: 200000000,
        eval_count: 8,
        eval_duration: 300000000,
      };

      (global as any).fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await provider.generate({
        prompt: "Hello",
        maxTokens: 100,
        temperature: 0.7,
        requestId: "test-123",
      });

      expect(result.text).toBe("Hello! How can I help you today?");
      expect(result.inputTokens).toBe(5);
      expect(result.outputTokens).toBe(8);
      expect(result.requestId).toBe("test-123");
      expect(result.computeCost).toBeDefined();
      expect(result.computeCost?.modelId).toBe("test-model-123");
      expect(result.computeCost?.operationId).toBe("test-123");
    });

    it("should estimate tokens when not provided by Ollama", async () => {
      const mockResponse = {
        model: "gemma3:1b",
        created_at: "2025-10-13T00:00:00Z",
        response: "Test response",
        done: true,
      };

      (global as any).fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await provider.generate({
        prompt: "Test prompt",
        maxTokens: 100,
      });

      // Should estimate tokens (4 chars per token)
      expect(result.inputTokens).toBeGreaterThan(0);
      expect(result.outputTokens).toBeGreaterThan(0);
    });

    it("should handle streaming requests", async () => {
      const mockResponse = {
        model: "gemma3:1b",
        created_at: "2025-10-13T00:00:00Z",
        response: "Streaming response",
        done: true,
        eval_count: 2,
        eval_duration: 100000000,
      };

      (global as any).fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await provider.generate({
        prompt: "Stream test",
        stream: true,
      });

      expect(result.text).toBe("Streaming response");
    });

    it("should include generation options in request", async () => {
      const mockResponse = {
        model: "gemma3:1b",
        created_at: "2025-10-13T00:00:00Z",
        response: "Response",
        done: true,
      };

      (global as any).fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      await provider.generate({
        prompt: "Test",
        maxTokens: 50,
        temperature: 0.8,
        topP: 0.95,
        stopSequences: ["END"],
      });

      expect((global as any).fetch).toHaveBeenCalledWith(
        "http://localhost:11434/api/generate",
        expect.objectContaining({
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: expect.stringContaining('"temperature":0.8'),
        })
      );
    });

    it("should throw error when context window exceeded", async () => {
      const longPrompt = "a".repeat(40000); // ~10K tokens, exceeds 8K window

      await expect(
        provider.generate({
          prompt: longPrompt,
          maxTokens: 100,
        })
      ).rejects.toThrow(OllamaProviderError);

      await expect(
        provider.generate({
          prompt: longPrompt,
          maxTokens: 100,
        })
      ).rejects.toThrow("exceeds model context window");
    });

    it("should throw error when Ollama API fails", async () => {
      (global as any).fetch.mockResolvedValueOnce({
        ok: false,
        statusText: "Internal Server Error",
      });

      await expect(
        provider.generate({
          prompt: "Test",
        })
      ).rejects.toThrow("Ollama API error");
    });

    it("should throw error when fetch fails", async () => {
      (global as any).fetch.mockRejectedValueOnce(new Error("Network error"));

      await expect(
        provider.generate({
          prompt: "Test",
        })
      ).rejects.toThrow(OllamaProviderError);
    });

    it("should calculate tokens per second", async () => {
      const mockResponse = {
        model: "gemma3:1b",
        created_at: "2025-10-13T00:00:00Z",
        response: "Response",
        done: true,
        eval_count: 100,
        eval_duration: 1000000000, // 1 second
      };

      (global as any).fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await provider.generate({
        prompt: "Test",
      });

      expect(result.tokensPerSecond).toBeGreaterThan(0);
    });
  });

  describe("Health Checking", () => {
    it("should return healthy when Ollama is running and model exists", async () => {
      (global as any).fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          models: [{ name: "gemma3:1b" }],
        }),
      });

      const health = await provider.checkHealth();

      expect(health.healthy).toBe(true);
      expect(health.modelId).toBe("test-model-123");
      expect(health.checkedAt).toBeInstanceOf(Date);
      expect(health.responseTimeMs).toBeGreaterThanOrEqual(0);
    });

    it("should return unhealthy when Ollama is not responding", async () => {
      (global as any).fetch.mockResolvedValueOnce({
        ok: false,
        statusText: "Service Unavailable",
      });

      const health = await provider.checkHealth();

      expect(health.healthy).toBe(false);
      expect(health.error).toContain("Ollama not responding");
    });

    it("should return unhealthy when model not found", async () => {
      (global as any).fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          models: [{ name: "other-model:1b" }],
        }),
      });

      const health = await provider.checkHealth();

      expect(health.healthy).toBe(false);
      expect(health.error).toContain("Model gemma3:1b not found");
    });

    it("should return unhealthy when fetch fails", async () => {
      (global as any).fetch.mockRejectedValueOnce(new Error("Network error"));

      const health = await provider.checkHealth();

      expect(health.healthy).toBe(false);
      expect(health.error).toContain("Network error");
    });
  });

  describe("Model Loading", () => {
    it("should load model and return performance characteristics", async () => {
      const mockResponse = {
        model: "gemma3:1b",
        created_at: "2025-10-13T00:00:00Z",
        response: "Hi",
        done: true,
        eval_count: 1,
        eval_duration: 10000000,
      };

      (global as any).fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const performance = await provider.load();

      expect(performance).toBeDefined();
      expect(performance.avgLatencyMs).toBeGreaterThanOrEqual(0);
      expect(performance.tokensPerSec).toBe(130); // From config
      expect(performance.memoryUsageMB).toBeGreaterThan(0);
      expect(provider.isModelLoaded()).toBe(true);
    });

    it("should throw error when warmup fails", async () => {
      (global as any).fetch.mockRejectedValueOnce(new Error("Load failed"));

      await expect(provider.load()).rejects.toThrow("Failed to load model");
      expect(provider.isModelLoaded()).toBe(false);
    });
  });

  describe("Model Unloading", () => {
    it("should unload model", async () => {
      // First load
      const mockResponse = {
        model: "gemma3:1b",
        created_at: "2025-10-13T00:00:00Z",
        response: "Hi",
        done: true,
      };

      (global as any).fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      await provider.load();
      expect(provider.isModelLoaded()).toBe(true);

      // Then unload
      await provider.unload();
      expect(provider.isModelLoaded()).toBe(false);
    });
  });

  describe("Performance Metrics", () => {
    it("should get performance characteristics", async () => {
      const performance = await provider.getPerformance();

      expect(performance).toBeDefined();
      expect(performance.tokensPerSec).toBe(130);
      expect(performance.memoryUsageMB).toBe(1024);
      expect(performance.avgLatencyMs).toBeGreaterThan(0);
    });
  });

  describe("Request Validation", () => {
    it("should validate request can be handled", () => {
      const valid = provider.canHandle({
        prompt: "Short prompt",
        maxTokens: 100,
      });

      expect(valid).toBe(true);
    });

    it("should reject request exceeding context window", () => {
      const longPrompt = "a".repeat(40000);

      const valid = provider.canHandle({
        prompt: longPrompt,
        maxTokens: 100,
      });

      expect(valid).toBe(false);
    });
  });

  describe("Cost Estimation", () => {
    it("should estimate compute cost for request", () => {
      const cost = provider.estimateCost({
        prompt: "Test prompt with some content",
        maxTokens: 50,
        requestId: "test-cost",
      });

      expect(cost).toBeDefined();
      expect(cost.modelId).toBe("test-model-123");
      expect(cost.operationId).toBe("test-cost");
      expect(cost.inputTokens).toBeGreaterThan(0);
      expect(cost.outputTokens).toBe(50);
    });

    it("should use default output tokens when not specified", () => {
      const cost = provider.estimateCost({
        prompt: "Test",
      });

      expect(cost.outputTokens).toBe(100); // Default
    });
  });
});
