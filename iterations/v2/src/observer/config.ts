/**
 * Observer configuration loader
 *
 * Parses environment variables and produces the runtime configuration used by
 * the Arbiter observer bridge. Defaults mirror those documented in the feature
 * plan to keep behaviour predictable across environments.
 */

import fs from "fs";
import path from "path";
import { z } from "zod";
import {
  ObserverConfig,
  ObserverPrivacyMode,
  RedactionRule,
} from "./types";

const DEFAULT_DATA_DIR = path.resolve(
  process.cwd(),
  "iterations/v2/data/arbiter-observer"
);

const DEFAULT_SAMPLE_RATES: Record<string, number> = {
  "task.debug": 0.1,
  "task.info": 0.5,
  "task.warn": 1.0,
  "task.error": 1.0,
};

const DEFAULT_REDACTION_RULES: Array<{ name: string; pattern: string }> = [
  {
    name: "jwt",
    pattern:
      "(eyJ[a-zA-Z0-9_-]{10,}\\.[a-zA-Z0-9_-]{10,}\\.[a-zA-Z0-9_-]{10,})",
  },
  {
    name: "bearer-token",
    pattern: "(sk-[A-Za-z0-9]{16,})",
  },
  {
    name: "aws-access-key",
    pattern: "(AKIA[0-9A-Z]{16})",
  },
  {
    name: "email",
    pattern: "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[A-Za-z]{2,}",
  },
];

const observerConfigSchema = z.object({
  bind: z.string().default("127.0.0.1"),
  port: z.number().min(1).max(65535).default(4387),
  socketPath: z.string().nullable().default(null),
  authToken: z.string().optional(),
  allowedOrigins: z.array(z.string()).default(["null", "file://"]),
  dataDir: z.string().default(DEFAULT_DATA_DIR),
  maxClients: z.number().min(1).max(256).default(32),
  flushIntervalMs: z.number().min(10).default(50),
  heartbeatIntervalMs: z.number().min(1000).default(20000),
  maxQueueSize: z.number().min(100).default(10000),
  rotateMB: z.number().min(1).default(256),
  retentionDays: z.number().min(1).default(14),
  privacyMode: z.enum(["standard", "strict"]).default("standard"),
  sampleRates: z.record(z.string(), z.number().min(0).max(1)).default(
    DEFAULT_SAMPLE_RATES
  ),
  redactionRules: z
    .array(
      z.object({
        name: z.string(),
        pattern: z.string(),
        replacement: z.string().optional(),
      })
    )
    .default(DEFAULT_REDACTION_RULES),
});

function parseNumber(
  value: string | undefined,
  fallback: number,
  min?: number,
  max?: number
): number {
  if (!value) return fallback;
  const parsed = Number(value);
  if (Number.isNaN(parsed)) return fallback;
  if (typeof min === "number" && parsed < min) return fallback;
  if (typeof max === "number" && parsed > max) return fallback;
  return parsed;
}

function parseStringArray(value: string | undefined): string[] | undefined {
  if (!value) return undefined;
  return value
    .split(",")
    .map((v) => v.trim())
    .filter(Boolean);
}

function parseSampleRates(value: string | undefined): Record<string, number> {
  if (!value) {
    return DEFAULT_SAMPLE_RATES;
  }
  return value.split(",").reduce<Record<string, number>>((acc, token) => {
    const [key, rawVal] = token.split("=");
    if (!key || !rawVal) return acc;
    const parsed = Number(rawVal);
    if (!Number.isNaN(parsed) && parsed >= 0 && parsed <= 1) {
      acc[key.trim()] = parsed;
    }
    return acc;
  }, {});
}

function parseRedactionRules(
  value: string | undefined
): Array<{ name: string; pattern: string; replacement?: string }> | undefined {
  if (!value) return undefined;
  try {
    const decoded = JSON.parse(value);
    if (Array.isArray(decoded)) {
      return decoded.filter(
        (item) => typeof item?.name === "string" && typeof item?.pattern === "string"
      );
    }
  } catch (error) {
    console.warn("Failed to parse OBSERVER_REDACTION_RULES JSON:", error);
  }
  return undefined;
}

/**
 * Load observer configuration from the environment.
 */
export function loadObserverConfig(): ObserverConfig {
  const raw = observerConfigSchema.parse({
    bind: process.env.OBSERVER_BIND,
    port: parseNumber(process.env.OBSERVER_PORT, 4387, 1, 65535),
    socketPath: process.env.OBSERVER_SOCKET_PATH ?? null,
    authToken: process.env.OBSERVER_AUTH_TOKEN?.trim() || undefined,
    allowedOrigins:
      parseStringArray(process.env.OBSERVER_ALLOWED_ORIGINS) ??
      undefined,
    dataDir: process.env.OBSERVER_DATA_DIR
      ? path.resolve(process.cwd(), process.env.OBSERVER_DATA_DIR)
      : DEFAULT_DATA_DIR,
    maxClients: parseNumber(process.env.OBSERVER_MAX_CLIENTS, 32, 1, 256),
    flushIntervalMs: parseNumber(
      process.env.OBSERVER_FLUSH_INTERVAL_MS,
      50,
      10
    ),
    heartbeatIntervalMs: parseNumber(
      process.env.OBSERVER_HEARTBEAT_INTERVAL_MS,
      20000,
      1000
    ),
    maxQueueSize: parseNumber(
      process.env.OBSERVER_MAX_QUEUE_SIZE,
      10000,
      100
    ),
    rotateMB: parseNumber(process.env.OBSERVER_ROTATE_MB, 256, 1),
    retentionDays: parseNumber(process.env.OBSERVER_RETENTION_DAYS, 14, 1),
    privacyMode: (process.env.OBSERVER_PRIVACY_MODE as ObserverPrivacyMode) ??
      undefined,
    sampleRates: parseSampleRates(process.env.OBSERVER_SAMPLE_RATES),
    redactionRules:
      parseRedactionRules(process.env.OBSERVER_REDACTION_RULES) ??
      DEFAULT_REDACTION_RULES,
  });

  ensureDataDirectory(raw.dataDir);

  return {
    bind: raw.bind,
    port: raw.port,
    socketPath: raw.socketPath || undefined,
    authToken: raw.authToken,
    allowedOrigins: new Set(
      raw.allowedOrigins.length
        ? raw.allowedOrigins
        : ["null", "file://", "urn:app"]
    ),
    dataDir: raw.dataDir,
    maxClients: raw.maxClients,
    flushIntervalMs: raw.flushIntervalMs,
    heartbeatIntervalMs: raw.heartbeatIntervalMs,
    maxQueueSize: raw.maxQueueSize,
    rotateMB: raw.rotateMB,
    retentionDays: raw.retentionDays,
    sampleRates: raw.sampleRates,
    redactionRules: compileRedactionRules(raw.redactionRules),
    privacyMode: raw.privacyMode,
  };
}

function ensureDataDirectory(dir: string): void {
  try {
    fs.mkdirSync(dir, { recursive: true });
  } catch (error) {
    console.warn(`Failed to create observer data directory "${dir}":`, error);
  }
}

function compileRedactionRules(
  rules: Array<{ name: string; pattern: string; replacement?: string }>
): RedactionRule[] {
  const compiled: RedactionRule[] = [];

  for (const rule of rules) {
    try {
      compiled.push({
        name: rule.name,
        pattern: new RegExp(rule.pattern, "g"),
        replacement: rule.replacement,
      });
    } catch (error) {
      console.warn(`Invalid redaction rule "${rule.name}":`, error);
    }
  }

  return compiled;
}
