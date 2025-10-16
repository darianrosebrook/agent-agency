/**
 * @fileoverview Task intake processor for Arbiter orchestration.
 *
 * Performs streaming-safe validation, binary detection, UTF-8 chunking,
 * and normalization before tasks enter the orchestration pipeline.
 */

import { ValidationUtils } from "../Validation";
import { Task, TaskType } from "../../types/arbiter-orchestration";

export interface TaskIntakeIssue {
  code: string;
  message: string;
  field?: string;
  value?: unknown;
}

export interface TaskIntakeResult {
  status: "accepted" | "rejected";
  sanitizedTask?: Task;
  chunks: string[];
  errors: TaskIntakeIssue[];
  warnings: TaskIntakeIssue[];
  metadata: {
    rawSizeBytes: number;
    contentType?: string;
    encoding?: string;
    chunkSizeBytes: number;
    chunkCount: number;
  };
}

export interface TaskIntakeEnvelope {
  payload: string | Buffer | Record<string, any>;
  metadata?: {
    contentType?: string;
    encoding?: BufferEncoding;
    priorityHint?: "low" | "normal" | "high" | "urgent";
    surface?: string;
  };
}

export interface TaskIntakeConfig {
  chunkSizeBytes?: number;
  maxDescriptionBytes?: number;
  chunkOverlapBytes?: number;
  requiredFields?: string[];
  defaults?: {
    priority?: number;
    timeoutMs?: number;
    attempts?: number;
    maxAttempts?: number;
    budget?: {
      maxFiles?: number;
      maxLoc?: number;
    };
  };
  binaryDetection?: {
    enabled?: boolean;
    sampleBytes?: number;
    nonTextThreshold?: number;
  };
}

type ResolvedBinaryDetection = {
  enabled: boolean;
  sampleBytes: number;
  nonTextThreshold: number;
};

type ResolvedDefaults = {
  priority: number;
  timeoutMs: number;
  attempts: number;
  maxAttempts: number;
  budget: {
    maxFiles: number;
    maxLoc: number;
  };
};

type ResolvedTaskIntakeConfig = {
  chunkSizeBytes: number;
  maxDescriptionBytes: number;
  chunkOverlapBytes: number;
  requiredFields: string[];
  defaults: ResolvedDefaults;
  binaryDetection: ResolvedBinaryDetection;
};

const DEFAULT_CONFIG: ResolvedTaskIntakeConfig = {
  chunkSizeBytes: 5 * 1024, // 5 KiB
  maxDescriptionBytes: 20 * 1024, // 20 KiB guard rail
  chunkOverlapBytes: 0,
  requiredFields: ["id", "type", "description"],
  defaults: {
    priority: 5,
    timeoutMs: 5 * 60 * 1000,
    attempts: 0,
    maxAttempts: 3,
    budget: {
      maxFiles: 10,
      maxLoc: 500,
    },
  },
  binaryDetection: {
    enabled: true,
    sampleBytes: 2048,
    nonTextThreshold: 0.3,
  },
};

function resolveConfig(config?: TaskIntakeConfig): ResolvedTaskIntakeConfig {
  return {
    chunkSizeBytes: config?.chunkSizeBytes ?? DEFAULT_CONFIG.chunkSizeBytes,
    maxDescriptionBytes:
      config?.maxDescriptionBytes ?? DEFAULT_CONFIG.maxDescriptionBytes,
    chunkOverlapBytes:
      config?.chunkOverlapBytes ?? DEFAULT_CONFIG.chunkOverlapBytes,
    requiredFields: config?.requiredFields ?? DEFAULT_CONFIG.requiredFields,
    defaults: {
      priority: config?.defaults?.priority ?? DEFAULT_CONFIG.defaults.priority,
      timeoutMs:
        config?.defaults?.timeoutMs ?? DEFAULT_CONFIG.defaults.timeoutMs,
      attempts: config?.defaults?.attempts ?? DEFAULT_CONFIG.defaults.attempts,
      maxAttempts:
        config?.defaults?.maxAttempts ?? DEFAULT_CONFIG.defaults.maxAttempts,
      budget: {
        maxFiles:
          config?.defaults?.budget?.maxFiles ??
          DEFAULT_CONFIG.defaults.budget.maxFiles,
        maxLoc:
          config?.defaults?.budget?.maxLoc ??
          DEFAULT_CONFIG.defaults.budget.maxLoc,
      },
    },
    binaryDetection: {
      enabled:
        config?.binaryDetection?.enabled ??
        DEFAULT_CONFIG.binaryDetection.enabled,
      sampleBytes:
        config?.binaryDetection?.sampleBytes ??
        DEFAULT_CONFIG.binaryDetection.sampleBytes,
      nonTextThreshold:
        config?.binaryDetection?.nonTextThreshold ??
        DEFAULT_CONFIG.binaryDetection.nonTextThreshold,
    },
  };
}

export class TaskIntakeProcessor {
  private readonly config: ResolvedTaskIntakeConfig;

  constructor(config?: TaskIntakeConfig) {
    this.config = resolveConfig(config);
  }

  /**
   * Process a raw task payload and return normalization details.
   */
  process(envelope: TaskIntakeEnvelope): TaskIntakeResult {
    const errors: TaskIntakeIssue[] = [];
    const warnings: TaskIntakeIssue[] = [];

    if (!envelope || envelope.payload === undefined || envelope.payload === null) {
      return this.rejectResult(errors, warnings, {
        code: "EMPTY_PAYLOAD",
        message: "Task payload is required",
      });
    }

    const encoding = envelope.metadata?.encoding ?? "utf8";
    const contentType =
      envelope.metadata?.contentType ?? "application/json; charset=utf-8";

    let rawBuffer: Buffer;
    if (Buffer.isBuffer(envelope.payload)) {
      rawBuffer = envelope.payload;
    } else if (typeof envelope.payload === "string") {
      rawBuffer = Buffer.from(envelope.payload, encoding);
    } else if (typeof envelope.payload === "object") {
      rawBuffer = Buffer.from(JSON.stringify(envelope.payload), encoding);
    } else {
      return this.rejectResult(
        errors,
        warnings,
        {
          code: "UNSUPPORTED_PAYLOAD_TYPE",
          message: `Unsupported payload type: ${typeof envelope.payload}`,
        },
        { rawSizeBytes: 0, contentType, encoding }
      );
    }

    const rawSizeBytes = rawBuffer.length;

    if (rawSizeBytes === 0) {
      return this.rejectResult(
        errors,
        warnings,
        {
          code: "EMPTY_PAYLOAD",
          message: "Task payload is empty",
        },
        { rawSizeBytes, contentType, encoding }
      );
    }

    if (this.config.binaryDetection.enabled) {
      if (this.isLikelyBinary(rawBuffer, contentType)) {
        return this.rejectResult(
          errors,
          warnings,
          {
            code: "BINARY_PAYLOAD",
            message: "Binary payloads are not supported for task ingestion",
          },
          { rawSizeBytes, contentType, encoding }
        );
      }
    }

    let rawText: string | undefined;
    if (Buffer.isBuffer(envelope.payload) || typeof envelope.payload === "string") {
      rawText = rawBuffer.toString(encoding);
    } else {
      rawText = JSON.stringify(envelope.payload);
    }

    if (rawText && rawText.trim().length === 0) {
      return this.rejectResult(
        errors,
        warnings,
        {
          code: "EMPTY_PAYLOAD",
          message: "Task payload is empty",
        },
        { rawSizeBytes, contentType, encoding }
      );
    }

    let parsedTask: Record<string, any> | null = null;

    if (typeof envelope.payload === "object" && !Buffer.isBuffer(envelope.payload)) {
      parsedTask = envelope.payload as Record<string, any>;
    } else {
      try {
        parsedTask = rawText ? JSON.parse(rawText) : null;
      } catch (error) {
        return this.rejectResult(
          errors,
          warnings,
          {
            code: "MALFORMED_JSON",
            message: "Task payload is not valid JSON",
            value: (error as Error).message,
          },
          { rawSizeBytes, contentType, encoding }
        );
      }
    }

    if (!parsedTask || typeof parsedTask !== "object") {
      return this.rejectResult(
        errors,
        warnings,
        {
          code: "INVALID_PAYLOAD",
          message: "Task payload must deserialize to an object",
        },
        { rawSizeBytes, contentType, encoding }
      );
    }

    // Required field checks
    for (const field of this.config.requiredFields) {
      const value = (parsedTask as Record<string, any>)[field];
      if (
        value === undefined ||
        value === null ||
        (typeof value === "string" && value.trim().length === 0)
      ) {
        errors.push({
          code: "MISSING_REQUIRED_FIELD",
          message: `Field '${field}' is required`,
          field,
        });
      }
    }

    if (errors.length > 0) {
      return this.rejectResult(
        errors,
        warnings,
        undefined,
        { rawSizeBytes, contentType, encoding }
      );
    }

    if (typeof parsedTask.description !== "string") {
      errors.push({
        code: "INVALID_DESCRIPTION",
        message: "Task description must be a string",
        field: "description",
      });
      return this.rejectResult(
        errors,
        warnings,
        undefined,
        { rawSizeBytes, contentType, encoding }
      );
    }

    const sanitizedTask = this.normalizeTask(
      parsedTask,
      envelope.metadata ?? {},
      warnings
    );

    const validation = ValidationUtils.validateTask(sanitizedTask);
    if (!validation.isValid) {
      for (const error of validation.errors) {
        errors.push({
          code: "VALIDATION_ERROR",
          message: error.message,
          field: error.field,
          value: error.value,
        });
      }
    }

    for (const warn of validation.warnings) {
      warnings.push({
        code: `VALIDATION_WARNING_${warn.code}`,
        message: warn.message,
        field: warn.field,
        value: warn.value,
      });
    }

    if (errors.length > 0) {
      return this.rejectResult(
        errors,
        warnings,
        undefined,
        { rawSizeBytes, contentType, encoding }
      );
    }

    const descriptionBytes = Buffer.byteLength(
      sanitizedTask.description,
      "utf8"
    );

    if (descriptionBytes > this.config.maxDescriptionBytes) {
      warnings.push({
        code: "DESCRIPTION_OVERSIZED",
        message: `Description exceeds ${this.config.maxDescriptionBytes} bytes`,
        field: "description",
        value: descriptionBytes,
      });
    }

    const chunks = this.chunkUtf8(sanitizedTask.description);
    if (chunks.length > 1) {
      warnings.push({
        code: "DESCRIPTION_CHUNKED",
        message: `Description chunked into ${chunks.length} parts`,
        field: "description",
        value: {
          chunkCount: chunks.length,
          chunkSizeBytes: this.config.chunkSizeBytes,
        },
      });
    }

    return {
      status: "accepted",
      sanitizedTask,
      chunks,
      errors,
      warnings,
      metadata: {
        rawSizeBytes,
        contentType,
        encoding,
        chunkSizeBytes: this.config.chunkSizeBytes,
        chunkCount: chunks.length,
      },
    };
  }

  private rejectResult(
    errors: TaskIntakeIssue[],
    warnings: TaskIntakeIssue[],
    newError?: TaskIntakeIssue,
    metadataOverrides?: Partial<TaskIntakeResult["metadata"]>
  ): TaskIntakeResult {
    if (newError) {
      errors.push(newError);
    }

    const metadata = {
      rawSizeBytes: metadataOverrides?.rawSizeBytes ?? 0,
      contentType: metadataOverrides?.contentType,
      encoding: metadataOverrides?.encoding,
      chunkSizeBytes: this.config.chunkSizeBytes,
      chunkCount: 0,
    };

    return {
      status: "rejected",
      chunks: [],
      errors,
      warnings,
      metadata,
    };
  }

  private normalizeTask(
    raw: Record<string, any>,
    metadata: TaskIntakeEnvelope["metadata"],
    warnings: TaskIntakeIssue[]
  ): Task {
    const description =
      typeof raw.description === "string" ? raw.description : String(raw.description ?? "");

    const normalizedMetadata =
      raw.metadata && typeof raw.metadata === "object"
        ? { ...raw.metadata }
        : {};

    if (!normalizedMetadata.surface && raw.surface) {
      normalizedMetadata.surface = raw.surface;
    }

    if (!normalizedMetadata.surface && metadata?.surface) {
      normalizedMetadata.surface = metadata.surface;
    }

    if (!normalizedMetadata.surface) {
      warnings.push({
        code: "SURFACE_DEFAULTED",
        message: "Surface not provided; defaulting to 'unknown'",
        field: "surface",
      });
      normalizedMetadata.surface = "unknown";
    }

    const createdAt = this.normalizeDate(raw.createdAt, warnings);

    const normalizedTask: Task = {
      id: String(raw.id ?? "").trim(),
      description,
      type: this.normalizeTaskType(raw.type),
      requiredCapabilities:
        (raw.requiredCapabilities &&
          typeof raw.requiredCapabilities === "object" &&
          !Array.isArray(raw.requiredCapabilities)
          ? raw.requiredCapabilities
          : {}) ?? {},
      priority: this.normalizePriority(raw.priority),
      timeoutMs: this.normalizePositiveNumber(
        raw.timeoutMs,
        this.config.defaults.timeoutMs
      ),
      budget: {
        maxFiles: this.normalizePositiveNumber(
          raw.budget?.maxFiles,
          this.config.defaults.budget.maxFiles
        ),
        maxLoc: this.normalizePositiveNumber(
          raw.budget?.maxLoc,
          this.config.defaults.budget.maxLoc
        ),
      },
      createdAt,
      metadata: normalizedMetadata,
      attempts: this.normalizeNonNegativeNumber(
        raw.attempts,
        this.config.defaults.attempts
      ),
      maxAttempts: this.normalizePositiveNumber(
        raw.maxAttempts,
        this.config.defaults.maxAttempts
      ),
    };

    if (raw.payload) {
      normalizedTask.payload = raw.payload;
    }

    return normalizedTask;
  }

  private normalizeTaskType(value: unknown): TaskType {
    const allowedTypes: TaskType[] = [
      "analysis",
      "research",
      "validation",
      "general",
      "code-editing",
      "code-review",
      "script-execution",
    ];

    if (typeof value === "string") {
      const normalized = value.trim() as TaskType;
      if (allowedTypes.includes(normalized)) {
        return normalized;
      }
    }

    return "analysis";
  }

  private normalizePriority(value: unknown): number {
    if (typeof value === "number" && value >= 1 && value <= 10) {
      return Math.floor(value);
    }
    return this.config.defaults.priority;
  }

  private normalizePositiveNumber(
    value: unknown,
    fallback: number
  ): number {
    if (typeof value === "number" && value > 0) {
      return value;
    }
    return fallback;
  }

  private normalizeNonNegativeNumber(
    value: unknown,
    fallback: number
  ): number {
    if (typeof value === "number" && value >= 0) {
      return value;
    }
    return fallback;
  }

  private normalizeDate(
    value: unknown,
    warnings: TaskIntakeIssue[]
  ): Date {
    if (value instanceof Date && !isNaN(value.getTime())) {
      return value;
    }

    if (typeof value === "string") {
      const parsed = new Date(value);
      if (!isNaN(parsed.getTime())) {
        return parsed;
      }
      warnings.push({
        code: "CREATED_AT_NORMALIZED",
        message: "createdAt string could not be parsed; defaulting to now",
        field: "createdAt",
        value,
      });
    }

    if (typeof value === "number") {
      const parsed = new Date(value);
      if (!isNaN(parsed.getTime())) {
        return parsed;
      }
      warnings.push({
        code: "CREATED_AT_NORMALIZED",
        message: "createdAt number could not be parsed; defaulting to now",
        field: "createdAt",
        value,
      });
    }

    return new Date();
  }

  private chunkUtf8(text: string): string[] {
    if (text.length === 0) {
      return [""];
    }

    const encoder = new TextEncoder();
    const decoder = new TextDecoder("utf-8", { fatal: false });
    const chunkSize = this.config.chunkSizeBytes;
    const chunks: string[] = [];
    let currentBytes: number[] = [];
    let currentSize = 0;

    for (const char of text) {
      const encoded = Array.from(encoder.encode(char));
      if (encoded.length > chunkSize) {
        if (currentBytes.length > 0) {
          chunks.push(decoder.decode(new Uint8Array(currentBytes)));
          currentBytes = [];
          currentSize = 0;
        }
        chunks.push(char);
        continue;
      }

      if (currentSize + encoded.length > chunkSize && currentBytes.length > 0) {
        chunks.push(decoder.decode(new Uint8Array(currentBytes)));
        currentBytes = [];
        currentSize = 0;
      }

      currentBytes.push(...encoded);
      currentSize += encoded.length;
    }

    if (currentBytes.length > 0) {
      chunks.push(decoder.decode(new Uint8Array(currentBytes)));
    }

    return chunks.length > 0 ? chunks : [""];
  }

  private isLikelyBinary(buffer: Buffer, contentType?: string): boolean {
    if (buffer.length === 0) {
      return false;
    }

    if (contentType) {
      const lowered = contentType.toLowerCase();
      if (
        lowered.includes("json") ||
        lowered.startsWith("text/") ||
        lowered.includes("xml") ||
        lowered.includes("yaml") ||
        lowered.includes("javascript")
      ) {
        return false;
      }
    }

    const sampleSize = Math.min(
      buffer.length,
      this.config.binaryDetection.sampleBytes
    );

    let nonTextCount = 0;
    for (let i = 0; i < sampleSize; i += 1) {
      const byte = buffer[i];
      if (byte === 0) {
        return true;
      }
      if (
        (byte < 7 || (byte > 14 && byte < 32)) &&
        byte !== 9 &&
        byte !== 10 &&
        byte !== 13
      ) {
        nonTextCount += 1;
      }
    }

    return nonTextCount / sampleSize > this.config.binaryDetection.nonTextThreshold;
  }
}
