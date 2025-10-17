/**
 * @fileoverview EmbeddingService Unit Tests
 *
 * Tests for EmbeddingService using Ollama embeddinggemma integration.
 * Mocks Ollama API responses to test embedding generation and caching.
 *
 * @author @darianrosebrook
 */

import { jest } from "@jest/globals";
import { EmbeddingService } from "../../src/embeddings/EmbeddingService";
import { EmbeddingError } from "../../src/embeddings/types";

// Mock fetch for Ollama API calls
const mockFetch = jest.fn() as jest.MockedFunction<typeof fetch>;
global.fetch = mockFetch;

describe("EmbeddingService", () => {
  let embeddingService: EmbeddingService;

  beforeEach(() => {
    jest.clearAllMocks();
    embeddingService = new EmbeddingService({
      ollamaEndpoint: "http://localhost:11434",
      cacheSize: 10,
      timeout: 5000,
    });
  });

  afterEach(() => {
    embeddingService.clearCache();
  });

  describe("generateEmbedding", () => {
    it("should generate embedding for valid text", async () => {
      const mockEmbedding = new Array(768).fill(0).map(() => Math.random());
      const mockResponse = {
        embedding: mockEmbedding,
        model: "embeddinggemma",
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await embeddingService.generateEmbedding("test text");

      expect(result).toEqual(mockEmbedding);
      expect(result.length).toBe(768);
      expect(mockFetch).toHaveBeenCalledWith(
        "http://localhost:11434/api/embeddings",
        expect.objectContaining({
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            model: "embeddinggemma",
            prompt: "test text",
          }),
        })
      );
    });

    it("should cache embeddings and avoid duplicate API calls", async () => {
      const mockEmbedding = new Array(768).fill(0.5);
      const mockResponse = {
        embedding: mockEmbedding,
        model: "embeddinggemma",
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      // First call should hit API
      const result1 = await embeddingService.generateEmbedding("test text");
      expect(result1).toEqual(mockEmbedding);
      expect(mockFetch).toHaveBeenCalledTimes(1);

      // Second call should use cache
      const result2 = await embeddingService.generateEmbedding("test text");
      expect(result2).toEqual(mockEmbedding);
      expect(mockFetch).toHaveBeenCalledTimes(1); // Still 1 call
    });

    it("should reject empty text", async () => {
      await expect(embeddingService.generateEmbedding("")).rejects.toThrow(
        EmbeddingError
      );
      await expect(embeddingService.generateEmbedding("   ")).rejects.toThrow(
        EmbeddingError
      );
    });

    it("should handle API errors", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: "Internal Server Error",
        text: async () => "Server error",
      } as Response);

      await expect(embeddingService.generateEmbedding("test")).rejects.toThrow(
        "Ollama API error: 500 Internal Server Error - Server error"
      );
    });

    it("should handle timeout", async () => {
      mockFetch.mockImplementationOnce(() => new Promise(() => {})); // Never resolves

      const timeoutService = new EmbeddingService({
        ollamaEndpoint: "http://localhost:11434",
        timeout: 100, // Very short timeout
      });

      await expect(timeoutService.generateEmbedding("test")).rejects.toThrow(
        "Embedding generation timeout after 100ms"
      );
    });

    it("should validate embedding dimensions", async () => {
      const invalidEmbedding = new Array(512).fill(0.1); // Wrong dimension
      const mockResponse = {
        embedding: invalidEmbedding,
        model: "embeddinggemma",
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      await expect(embeddingService.generateEmbedding("test")).rejects.toThrow(
        "Expected embedding dimension 768, got 512"
      );
    });

    it("should validate embedding values", async () => {
      const invalidEmbedding = new Array(768).fill(NaN);
      const mockResponse = {
        embedding: invalidEmbedding,
        model: "embeddinggemma",
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      await expect(embeddingService.generateEmbedding("test")).rejects.toThrow(
        "Invalid embedding value at index 0: NaN"
      );
    });
  });

  describe("generateBatch", () => {
    it("should generate embeddings for multiple texts", async () => {
      const mockEmbeddings = [
        new Array(768).fill(0.1),
        new Array(768).fill(0.2),
      ];

      // Mock sequential API calls
      mockEmbeddings.forEach((embedding, _index) => {
        mockFetch.mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            embedding,
            model: "embeddinggemma",
          }),
        } as Response);
      });

      const texts = ["text 1", "text 2"];
      const results = await embeddingService.generateBatch(texts);

      expect(results).toHaveLength(2);
      expect(results[0]).toEqual(mockEmbeddings[0]);
      expect(results[1]).toEqual(mockEmbeddings[1]);
      expect(mockFetch).toHaveBeenCalledTimes(2);
    });

    it("should reject empty text array", async () => {
      await expect(embeddingService.generateBatch([])).rejects.toThrow(
        EmbeddingError
      );
    });

    it("should reject too many texts", async () => {
      const tooManyTexts = new Array(101).fill("text");
      await expect(
        embeddingService.generateBatch(tooManyTexts)
      ).rejects.toThrow("Batch size cannot exceed 100 texts");
    });

    it("should use cache for batch operations", async () => {
      const mockEmbedding = new Array(768).fill(0.5);
      const mockResponse = {
        embedding: mockEmbedding,
        model: "embeddinggemma",
      };

      // First batch call
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const texts1 = ["text 1", "text 2"];
      await embeddingService.generateBatch(texts1);
      expect(mockFetch).toHaveBeenCalledTimes(2);

      // Second batch call with some cached texts
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const texts2 = ["text 1", "text 3"]; // text 1 is cached, text 3 is new
      await embeddingService.generateBatch(texts2);
      expect(mockFetch).toHaveBeenCalledTimes(3); // Only 1 new call
    });
  });

  describe("isAvailable", () => {
    it("should return true when Ollama is available", async () => {
      const mockTagsResponse = {
        models: [{ name: "embeddinggemma" }, { name: "gemma3n:e2b" }],
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockTagsResponse,
      } as Response);

      const available = await embeddingService.isAvailable();
      expect(available).toBe(true);
    });

    it("should return false when Ollama is not available", async () => {
      mockFetch.mockRejectedValueOnce(new Error("Connection refused"));

      const available = await embeddingService.isAvailable();
      expect(available).toBe(false);
    });

    it("should return false when embedding model is not available", async () => {
      const mockTagsResponse = {
        models: [
          { name: "gemma3n:e2b" },
          // No embeddinggemma
        ],
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockTagsResponse,
      } as Response);

      const available = await embeddingService.isAvailable();
      expect(available).toBe(false);
    });
  });

  describe("cache management", () => {
    it("should respect cache size limits", async () => {
      const smallCacheService = new EmbeddingService({
        ollamaEndpoint: "http://localhost:11434",
        cacheSize: 2, // Very small cache
      });

      const mockResponse = {
        embedding: new Array(768).fill(0.1),
        model: "embeddinggemma",
      };

      // Fill cache beyond limit
      for (let i = 0; i < 5; i++) {
        mockFetch.mockResolvedValueOnce({
          ok: true,
          json: async () => mockResponse,
        } as Response);

        await smallCacheService.generateEmbedding(`text ${i}`);
      }

      const stats = smallCacheService.getCacheStats();
      expect(stats.size).toBeLessThanOrEqual(2);
      expect(stats.maxSize).toBe(2);
    });

    it("should provide cache statistics", () => {
      const stats = embeddingService.getCacheStats();
      expect(stats).toHaveProperty("size");
      expect(stats).toHaveProperty("maxSize");
      expect(stats).toHaveProperty("hitRate");
    });

    it("should clear cache", async () => {
      const mockEmbedding = new Array(768).fill(0.1);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ embedding: mockEmbedding }),
      } as Response);

      await embeddingService.generateEmbedding("test");
      expect(embeddingService.getCacheStats().size).toBe(1);

      embeddingService.clearCache();
      expect(embeddingService.getCacheStats().size).toBe(0);
    });
  });

  describe("error handling", () => {
    it("should wrap non-EmbeddingError exceptions", async () => {
      mockFetch.mockRejectedValueOnce(new Error("Network error"));

      await expect(embeddingService.generateEmbedding("test")).rejects.toThrow(
        "Failed to generate embedding: Network error"
      );
    });

    it("should handle invalid JSON response", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => {
          throw new Error("Invalid JSON");
        },
      } as Response);

      await expect(embeddingService.generateEmbedding("test")).rejects.toThrow(
        "Failed to generate embedding: Invalid JSON"
      );
    });
  });
});
