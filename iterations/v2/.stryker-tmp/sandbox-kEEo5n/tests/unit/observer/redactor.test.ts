// @ts-nocheck
import { describe, expect, it } from "@jest/globals";
import { Redactor } from "@/observer/redactor";
import { ObserverConfig, RedactionRule } from "@/observer/types";

const baseRules: RedactionRule[] = [
  { name: "token", pattern: /sk-[A-Za-z0-9]{10,}/g },
  { name: "email", pattern: /[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g },
];

function buildConfig(overrides: Partial<ObserverConfig> = {}): ObserverConfig {
  return {
    bind: "127.0.0.1",
    port: 4387,
    socketPath: null,
    authToken: undefined,
    allowedOrigins: new Set(["null", "file://"]),
    dataDir: "/tmp",
    maxClients: 10,
    flushIntervalMs: 50,
    heartbeatIntervalMs: 20000,
    maxQueueSize: 1000,
    rotateMB: 256,
    retentionDays: 14,
    sampleRates: { "*": 1 },
    redactionRules: baseRules,
    privacyMode: "standard",
    ...overrides,
  };
}

describe("Redactor", () => {
  it("redacts configured patterns in standard mode", () => {
    const redactor = new Redactor(buildConfig());
    const input =
      "Issue reported by user john.doe@example.com using token sk-ABC123456789XYZ";
    const result = redactor.redactText(input);

    expect(result.redacted).toBe(true);
    expect(result.text).not.toContain("john.doe@example.com");
    expect(result.text).not.toContain("sk-ABC123456789XYZ");
    expect(result.text).toContain("[REDACTED:email]");
    expect(result.text).toContain("[REDACTED:token]");
    expect(result.hash).toMatch(/^[a-f0-9]{64}$/);
  });

  it("removes sensitive text in strict mode", () => {
    const redactor = new Redactor(
      buildConfig({
        privacyMode: "strict",
      })
    );
    const result = redactor.redactText("secret instruction");
    expect(result.redacted).toBe(true);
    expect(result.text).toBeUndefined();
    expect(result.hash).toMatch(/^[a-f0-9]{64}$/);
  });

  it("sanitizes nested metadata objects", () => {
    const redactor = new Redactor(buildConfig());
    const sanitized = redactor.redactObject({
      owner: "jane@example.com",
      nested: {
        token: "sk-DEF123456789LMN",
        comment: "harmless",
      },
    });

    expect(sanitized.owner).toContain("[REDACTED:email]");
    expect(sanitized.nested.token).toContain("[REDACTED:token]");
    expect(sanitized.nested.comment).toBe("harmless");
  });
});
