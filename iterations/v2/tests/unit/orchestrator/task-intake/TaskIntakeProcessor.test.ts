/**
 * @fileoverview Unit tests for TaskIntakeProcessor (Phase 1 ingestion hardening)
 *
 * Validates streaming validation, chunking, and guard rails for Arbiter task intake.
 */

import fc from "fast-check";
import {
  TaskIntakeIssue,
  TaskIntakeProcessor,
} from "../../../../src/orchestrator/intake/TaskIntakeProcessor";

const BASE_TASK = {
  id: "task-edge-001",
  type: "analysis",
  surface: "cli",
  description: "baseline description",
  priority: 5,
  timeoutMs: 60000,
  attempts: 0,
  maxAttempts: 3,
  requiredCapabilities: {},
  budget: {
    maxFiles: 10,
    maxLoc: 200,
  },
  metadata: {
    requester: "unit-test",
  },
};

function makeEnvelope(overrides: Partial<typeof BASE_TASK> = {}) {
  const payload = {
    ...BASE_TASK,
    ...overrides,
    metadata: {
      ...(BASE_TASK.metadata || {}),
      ...(overrides.metadata || {}),
    },
  };

  return {
    payload: JSON.stringify(payload),
    metadata: {
      contentType: "application/json",
      encoding: "utf8" as BufferEncoding,
    },
  };
}

describe("TaskIntakeProcessor", () => {
  it("rejects empty payloads with actionable error", () => {
    const processor = new TaskIntakeProcessor();
    const result = processor.process({ payload: "" });

    expect(result.status).toBe("rejected");
    expect(result.errors.map((e) => e.code)).toContain("EMPTY_PAYLOAD");
    expect(result.metadata.rawSizeBytes).toBe(0);
  });

  it("rejects malformed JSON inputs", () => {
    const processor = new TaskIntakeProcessor();
    const result = processor.process({
      payload: "{ this is not valid json",
      metadata: { contentType: "application/json" },
    });

    expect(result.status).toBe("rejected");
    expect(result.errors.map((e) => e.code)).toContain("MALFORMED_JSON");
  });

  it("rejects tasks missing required fields", () => {
    const processor = new TaskIntakeProcessor();
    const result = processor.process(
      makeEnvelope({
        id: "",
        surface: "",
      })
    );

    expect(result.status).toBe("rejected");
    expect(result.errors.some((e) => e.code === "MISSING_REQUIRED_FIELD")).toBe(
      true
    );
  });

  it("chunks oversized descriptions while preserving original text", () => {
    const processor = new TaskIntakeProcessor({
      chunkSizeBytes: 1024,
      maxDescriptionBytes: 4096,
    });

    const longDescription = "Chunk me!".repeat(800); // > 1024 bytes
    const envelope = makeEnvelope({ description: longDescription });
    const result = processor.process(envelope);

    expect(result.status).toBe("accepted");
    expect(result.chunks.length).toBeGreaterThan(1);
    expect(
      result.chunks.every(
        (chunk) => Buffer.byteLength(chunk, "utf8") <= 1024
      )
    ).toBe(true);
    expect(result.chunks.join("")).toBe(longDescription);
    expect(
      result.warnings.some((warning) => warning.code === "DESCRIPTION_CHUNKED")
    ).toBe(true);
  });

  it("preserves multibyte characters across chunk boundaries", () => {
    const processor = new TaskIntakeProcessor({
      chunkSizeBytes: 512,
      maxDescriptionBytes: 2048,
    });

    const unicodeDescription = "🤖 CAWS テスト ".repeat(200); // force multiple chunks with multibyte chars
    const envelope = makeEnvelope({ description: unicodeDescription });
    const result = processor.process(envelope);

    expect(result.status).toBe("accepted");
    expect(result.chunks.length).toBeGreaterThan(1);
    expect(result.chunks.join("")).toBe(unicodeDescription);
  });

  it("rejects binary payloads to protect ingestion pipeline", () => {
    const processor = new TaskIntakeProcessor();
    const binaryBuffer = Buffer.from([0x00, 0x01, 0x02, 0x03, 0xff, 0x10]);

    const result = processor.process({
      payload: binaryBuffer,
      metadata: { contentType: "application/octet-stream" },
    });

    expect(result.status).toBe("rejected");
    expect(result.errors.map((e) => e.code)).toContain("BINARY_PAYLOAD");
  });

  it("preserves arbitrary unicode content under property-based fuzzing", () => {
    const processor = new TaskIntakeProcessor({
      chunkSizeBytes: 256,
      maxDescriptionBytes: 8192,
    });

    fc.assert(
      fc.property(
        fc.unicodeString({ minLength: 1, maxLength: 1024 }),
        (unicodeDescription) => {
          const envelope = makeEnvelope({ description: unicodeDescription });
          const result = processor.process(envelope);

          expect(result.status).toBe("accepted");
          expect(result.chunks.join("")).toBe(unicodeDescription);
          expect(
            result.chunks.every(
              (chunk) => Buffer.byteLength(chunk, "utf8") <= 256
            )
          ).toBe(true);
        }
      ),
      { numRuns: 50 }
    );
  });

  it("detects binary payloads via fuzzed corpora", () => {
    const processor = new TaskIntakeProcessor();

    fc.assert(
      fc.property(
        fc
          .array(fc.integer({ min: 0, max: 255 }), { minLength: 32, maxLength: 2048 })
          .map((bytes) => {
            bytes[0] = 0x00;
            return Buffer.from(bytes);
          }),
        (buffer) => {
          const result = processor.process({
            payload: buffer,
            metadata: { contentType: "application/octet-stream" },
          });

          expect(result.status).toBe("rejected");
          expect(
            result.errors.some((issue) => issue.code === "BINARY_PAYLOAD")
          ).toBe(true);
        }
      ),
      { numRuns: 25 }
    );
  });
});
