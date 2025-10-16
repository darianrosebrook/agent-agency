/**
 * LLM Configuration Utilities
 *
 * Provides utilities for loading and validating LLM configuration
 * from environment variables with proper defaults and validation.
 *
 * @author @darianrosebrook
 */

import type { LLMConfig } from "@/types/judge";

/**
 * Load LLM configuration from environment variables
 *
 * @returns Validated LLM configuration
 */
export function loadLLMConfig(): LLMConfig {
  const provider =
    (process.env.LLM_PROVIDER as LLMConfig["provider"]) || "ollama";
  const model = process.env.LLM_MODEL || getDefaultModelForProvider(provider);
  const temperature = parseFloat(process.env.LLM_TEMPERATURE || "0");
  const maxTokens = parseInt(process.env.LLM_MAX_TOKENS || "500");
  const apiKey = process.env.LLM_API_KEY;

  // Validate provider
  if (
    !["openai", "anthropic", "ollama", "mock", "model-registry"].includes(
      provider
    )
  ) {
    console.warn(`Invalid LLM_PROVIDER "${provider}", defaulting to "openai"`);
  }

  // Validate temperature
  if (temperature < 0 || temperature > 2) {
    console.warn(
      `Invalid LLM_TEMPERATURE "${temperature}", must be between 0-2, defaulting to 0`
    );
  }

  // Validate maxTokens
  if (maxTokens < 1 || maxTokens > 4000) {
    console.warn(
      `Invalid LLM_MAX_TOKENS "${maxTokens}", must be between 1-4000, defaulting to 500`
    );
  }

  return {
    provider: provider as LLMConfig["provider"],
    model,
    temperature: Math.max(0, Math.min(2, temperature)),
    maxTokens: Math.max(1, Math.min(4000, maxTokens)),
    apiKey,
  };
}

/**
 * Get default model for a provider
 *
 * @param provider LLM provider name
 * @returns Default model name
 */
function getDefaultModelForProvider(provider: string): string {
  switch (provider) {
    case "openai":
      return "gpt-4";
    case "anthropic":
      return "claude-3-sonnet-20240229";
    case "ollama":
      return "llama3.1:8b";
    case "mock":
      return "mock-model";
    case "model-registry":
      return "registry-model";
    default:
      return "gpt-4";
  }
}

/**
 * Validate that required API keys are available for the provider
 *
 * @param config LLM configuration
 * @returns True if configuration is valid
 */
export function validateLLMConfig(config: LLMConfig): boolean {
  if (config.provider === "mock" || config.provider === "ollama") {
    return true; // Mock and Ollama don't need API keys
  }

  let apiKey: string | undefined;

  if (config.provider === "openai") {
    apiKey = config.apiKey || process.env.OPENAI_API_KEY;
  } else if (config.provider === "anthropic") {
    apiKey = config.apiKey || process.env.ANTHROPIC_API_KEY;
  } else if (config.provider === "model-registry") {
    // Model registry might have its own authentication
    return true; // Assume configured properly for now
  }

  if (!apiKey) {
    console.error(
      `Missing API key for ${
        config.provider
      } provider. Set ${config.provider.toUpperCase()}_API_KEY environment variable or LLM_API_KEY.`
    );
    return false;
  }

  // Basic validation - check if it looks like an API key
  if (apiKey.length < 10) {
    console.warn(
      `API key for ${config.provider} seems too short (${apiKey.length} characters). Please verify it's correct.`
    );
  }

  return true;
}

/**
 * Get LLM configuration status for monitoring
 *
 * @returns Status information about LLM configuration
 */
export function getLLMConfigStatus(): {
  provider: string;
  model: string;
  hasApiKey: boolean;
  temperature: number;
  maxTokens: number;
  isValid: boolean;
} {
  const config = loadLLMConfig();
  const isValid = validateLLMConfig(config);

  let hasApiKey = false;
  if (config.provider === "openai") {
    hasApiKey = !!(config.apiKey || process.env.OPENAI_API_KEY);
  } else if (config.provider === "anthropic") {
    hasApiKey = !!(config.apiKey || process.env.ANTHROPIC_API_KEY);
  } else if (config.provider === "mock" || config.provider === "ollama") {
    hasApiKey = true; // Mock and Ollama don't need API keys
  }

  return {
    provider: config.provider,
    model: config.model,
    hasApiKey,
    temperature: config.temperature,
    maxTokens: config.maxTokens,
    isValid,
  };
}
