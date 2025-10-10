/**
 * AI Integration Module
 *
 * @author @darianrosebrook
 * @description AI model integrations for agent operations
 */

export { MultiModelOrchestrator } from "./multi-model-orchestrator";
export type {
  ModelCapability,
  ModelSelectionCriteria,
  OrchestratorConfig,
} from "./multi-model-orchestrator";
export { OllamaClient } from "./ollama-client";
export type { OllamaConfig } from "./ollama-client";
export { OpenAIClient } from "./openai-client";
export type { OpenAIConfig } from "./openai-client";
export type {
  AIModelClient,
  GenerateRequest,
  GenerateResponse,
  ModelConfig,
} from "./types";
