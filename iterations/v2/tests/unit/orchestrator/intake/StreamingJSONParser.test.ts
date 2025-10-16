/**
 * @fileoverview Tests for StreamingJSONParser component.
 */

import {
  StreamingJSONParser,
  createStreamingChunks,
  shouldUseStreamingParsing,
} from "../../../../src/orchestrator/intake/StreamingJSONParser";

describe("StreamingJSONParser", () => {
  let parser: StreamingJSONParser;

  beforeEach(() => {
    parser = new StreamingJSONParser({
      maxChunkSize: 1024,
      maxTotalSize: 200000, // 200KB - allow for larger test objects
      timeoutMs: 5000,
    });
  });

  afterEach(() => {
    parser.destroy();
  });

  describe("parseLargeString", () => {
    it("should parse small JSON strings successfully", async () => {
      const jsonString = '{"id": "test", "value": 42}';
      const result = await parser.parseLargeString(jsonString);

      expect(result.success).toBe(true);
      expect(result.data).toEqual({ id: "test", value: 42 });
      expect(result.bytesProcessed).toBeGreaterThan(0);
      expect(result.chunksProcessed).toBeGreaterThan(0);
      expect(result.parseTimeMs).toBeGreaterThanOrEqual(0);
    });

    it("should parse large JSON objects with streaming", async () => {
      // Create a large JSON object
      const largeObject = {
        id: "large-test",
        data: new Array(1000).fill(0).map((_, i) => ({
          index: i,
          value: `item-${i}`,
          metadata: { created: new Date().toISOString(), version: 1 },
        })),
      };

      const jsonString = JSON.stringify(largeObject);
      const result = await parser.parseLargeString(jsonString);
      expect(result.success).toBe(true);
      expect(result.data).toEqual(largeObject);
      expect(result.bytesProcessed).toBe(Buffer.byteLength(jsonString, "utf8"));
      expect(result.chunksProcessed).toBeGreaterThan(1); // Should be chunked
    });

    it("should handle malformed JSON gracefully", async () => {
      const malformedJson = '{"id": "test", "value": 42,}'; // Trailing comma
      const result = await parser.parseLargeString(malformedJson);

      expect(result.success).toBe(false);
      expect(result.error).toContain("JSON parse failed");
      expect(result.data).toBeUndefined();
    });

    it("should respect size limits", async () => {
      // Create JSON that exceeds maxTotalSize (200KB)
      const oversizedData = new Array(300000).fill("x").join(""); // 300KB of data
      const oversizedJson = JSON.stringify({ data: oversizedData });

      const result = await parser.parseLargeString(oversizedJson);

      expect(result.success).toBe(false);
      expect(result.error).toContain("exceeds maximum allowed size");
    });

    it("should handle timeout scenarios", async () => {
      // Create a parser with very short timeout
      const fastParser = new StreamingJSONParser({
        maxChunkSize: 1, // Force many chunks
        maxTotalSize: 10240,
        timeoutMs: 1, // Very short timeout
      });

      const jsonString = JSON.stringify({ data: "test" });
      const result = await fastParser.parseLargeString(jsonString);

      // Should either succeed or timeout
      expect(result.success || result.error?.includes("timeout")).toBe(true);

      fastParser.destroy();
    });
  });

  describe("parseStreaming", () => {
    it("should parse chunks incrementally", async () => {
      const originalData = { id: "streaming-test", items: [1, 2, 3, 4, 5] };
      const jsonString = JSON.stringify(originalData);
      const chunks = createStreamingChunks(jsonString, 10); // Small chunks

      const chunkEvents = [];
      parser.on("chunkProcessed", (chunk) => {
        chunkEvents.push(chunk);
      });

      const result = await parser.parseStreaming(chunks);

      expect(result.success).toBe(true);
      expect(result.data).toEqual(originalData);
      expect(chunkEvents).toHaveLength(chunks.length);
      expect(chunkEvents[chunkEvents.length - 1].isComplete).toBe(true);
    });

    it("should emit progress events", async () => {
      const jsonString = '{"test": "data"}';
      const chunks = ["{", '"', "test", '": "', "data", '"}'];

      const events: string[] = [];
      parser.on("chunkProcessed", () => events.push("chunk"));
      parser.on("parseComplete", () => events.push("complete"));

      await parser.parseStreaming(chunks);

      expect(events).toContain("chunk");
      expect(events).toContain("complete");
    });

    it("should handle empty chunks gracefully", async () => {
      const chunks = ["", '{"id": "test"}', ""];
      const result = await parser.parseStreaming(chunks);

      expect(result.success).toBe(true);
      expect(result.data).toEqual({ id: "test" });
    });
  });

  describe("validation", () => {
    it("should detect common JSON errors", async () => {
      const errorCases = [
        '{"trailing": "comma",}', // Trailing comma
        '{"unquoted": key}', // Unquoted key
        '{"invalid": value}', // Unquoted value
        '{"invalid": : "value"}', // Double colon
      ];

      for (const errorCase of errorCases) {
        const result = await parser.parseLargeString(errorCase);
        expect(result.success).toBe(false);
        expect(result.error).toBeDefined();
      }
    });

    it("should validate chunk sizes", async () => {
      const oversizedChunk = "x".repeat(2000); // Exceeds maxChunkSize
      const result = await parser.parseStreaming([oversizedChunk]);

      expect(result.success).toBe(false);
      expect(result.error).toContain("exceeds maximum");
    });
  });

  describe("getStats", () => {
    it("should provide accurate statistics", async () => {
      const jsonString = '{"id": "stats-test"}';

      // Get initial stats
      const initialStats = parser.getStats();
      expect(initialStats.bytesProcessed).toBe(0);
      expect(initialStats.chunksProcessed).toBe(0);
      expect(initialStats.bufferSize).toBe(0);

      // Parse and get updated stats
      await parser.parseLargeString(jsonString);
      const finalStats = parser.getStats();

      expect(finalStats.bytesProcessed).toBeGreaterThan(0);
      expect(finalStats.chunksProcessed).toBeGreaterThan(0);
      expect(finalStats.bufferSize).toBe(0); // Buffer should be cleared after parsing
      expect(finalStats.elapsedMs).toBeGreaterThanOrEqual(0);
    });
  });

  describe("destroy", () => {
    it("should clean up resources properly", () => {
      const jsonString = '{"id": "cleanup-test"}';

      // Start parsing but don't await
      const parsePromise = parser.parseLargeString(jsonString);

      // Destroy immediately
      parser.destroy();

      // Parser should still complete or handle cleanup gracefully
      return expect(parsePromise).resolves.toBeDefined();
    });
  });
});

describe("Utility Functions", () => {
  describe("shouldUseStreamingParsing", () => {
    it("should return true for large strings", () => {
      const largeString = "x".repeat(6000); // 6KB
      expect(shouldUseStreamingParsing(largeString)).toBe(true);
    });

    it("should return false for small strings", () => {
      const smallString = '{"id": "test"}'; // < 5KB
      expect(shouldUseStreamingParsing(smallString)).toBe(false);
    });

    it("should respect custom threshold", () => {
      const mediumString = "x".repeat(3000); // 3KB
      expect(shouldUseStreamingParsing(mediumString, 2000)).toBe(true);
      expect(shouldUseStreamingParsing(mediumString, 4000)).toBe(false);
    });

    it("should handle Buffer input", () => {
      const largeBuffer = Buffer.alloc(6000, "x");
      expect(shouldUseStreamingParsing(largeBuffer)).toBe(true);
    });
  });

  describe("createStreamingChunks", () => {
    it("should split string into chunks", () => {
      const data = "abcdefghijklmnopqrstuvwxyz";
      const chunks = createStreamingChunks(data, 5);

      expect(chunks).toEqual([
        "abcde",
        "fghij",
        "klmno",
        "pqrst",
        "uvwxy",
        "z",
      ]);
    });

    it("should handle empty string", () => {
      const chunks = createStreamingChunks("", 10);
      expect(chunks).toEqual([]);
    });

    it("should handle string smaller than chunk size", () => {
      const chunks = createStreamingChunks("abc", 10);
      expect(chunks).toEqual(["abc"]);
    });

    it("should use default chunk size", () => {
      const data = "x".repeat(20000);
      const chunks = createStreamingChunks(data);

      expect(chunks.length).toBeGreaterThan(1);
      expect(chunks.every((chunk) => chunk.length <= 8192)).toBe(true);
    });
  });
});

describe("Integration with TaskIntakeProcessor", () => {
  it("should handle real-world task payloads", async () => {
    const parser = new StreamingJSONParser();

    const realTask = {
      id: "integration-test",
      type: "analysis",
      description: "Test task for streaming parser integration",
      priority: 5,
      timeoutMs: 300000,
      metadata: {
        source: "test-suite",
        version: "1.0.0",
        tags: ["integration", "streaming", "json"],
        config: {
          chunkSize: 8192,
          maxRetries: 3,
          features: ["validation", "streaming", "chunking"],
        },
      },
      payload: {
        data: new Array(500).fill(0).map((_, i) => ({
          id: i,
          value: `item-${i}`,
          timestamp: new Date().toISOString(),
          metadata: {
            processed: false,
            retries: 0,
            errors: [],
          },
        })),
      },
    };

    const jsonString = JSON.stringify(realTask);
    const result = await parser.parseLargeString(jsonString);

    expect(result.success).toBe(true);
    expect(result.data).toEqual(realTask);
    expect(result.bytesProcessed).toBe(Buffer.byteLength(jsonString, "utf8"));

    parser.destroy();
  });
});
