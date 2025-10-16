/**
 * LLM Configuration Utilities
 *
 * Provides default LLM configuration for the evaluation system.
 *
 * @author @darianrosebrook
 */

import type { LLMConfig } from "../types/judge";

/**
 * Load default LLM configuration
 */
export function loadLLMConfig(): LLMConfig {
  return {
    provider: "mock",
    model: "gpt-3.5-turbo",
    temperature: 0.1,
    maxTokens: 1000,
  };
}

/**
 * Validate LLM configuration
 */
export function validateLLMConfig(config: LLMConfig): boolean {
  return (
    config.provider !== undefined &&
    config.model !== undefined &&
    config.temperature >= 0 &&
    config.temperature <= 2 &&
    config.maxTokens > 0
  );
}
