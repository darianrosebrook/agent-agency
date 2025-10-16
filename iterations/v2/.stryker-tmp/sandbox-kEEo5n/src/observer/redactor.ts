// @ts-nocheck
import crypto from "crypto";
import { ObserverConfig, ObserverPrivacyMode, RedactionRule } from "./types";

export interface RedactionResult {
  text?: string;
  redacted: boolean;
  hash?: string;
}

/**
 * Redactor applies configurable pattern-based sanitisation to text payloads.
 * It supports two privacy modes:
 *  - standard: apply replacements in-line but retain surrounding context
 *  - strict: drop the original text entirely, storing only hashes/flags
 */
export class Redactor {
  private readonly rules: RedactionRule[];
  private readonly privacyMode: ObserverPrivacyMode;

  constructor(config: ObserverConfig) {
    this.rules = config.redactionRules;
    this.privacyMode = config.privacyMode;
  }

  redactText(text: string): RedactionResult {
    if (!text) {
      return { text, redacted: false };
    }

    const hash = this.computeHash(text);

    if (this.privacyMode === "strict") {
      return { text: undefined, redacted: true, hash };
    }

    let redactedText = text;
    let applied = false;

    for (const rule of this.rules) {
      redactedText = redactedText.replace(rule.pattern, () => {
        applied = true;
        return rule.replacement ?? `[REDACTED:${rule.name}]`;
      });
    }

    if (!applied) {
      return { text, redacted: false, hash };
    }

    return {
      text: redactedText,
      redacted: true,
      hash,
    };
  }

  /**
   * Recursively sanitize plain objects/arrays by redacting string values.
   */
  redactObject<T>(value: T): T {
    if (value === null || value === undefined) {
      return value;
    }

    if (typeof value === "string") {
      const result = this.redactText(value);
      return (result.text ?? `[REDACTED]`) as unknown as T;
    }

    if (Array.isArray(value)) {
      return value.map((item) => this.redactObject(item)) as unknown as T;
    }

    if (typeof value === "object") {
      const clone: Record<string, unknown> = {};
      for (const [key, entry] of Object.entries(value as Record<
        string,
        unknown
      >)) {
        clone[key] = this.redactObject(entry);
      }
      return clone as T;
    }

    return value;
  }

  private computeHash(text: string): string {
    return crypto.createHash("sha256").update(text, "utf8").digest("hex");
  }
}

