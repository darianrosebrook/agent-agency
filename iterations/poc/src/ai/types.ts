/**
 * AI Client Types
 *
 * @author @darianrosebrook
 * @description Type definitions for AI model integrations
 */

export interface ModelConfig {
  temperature?: number;
  maxTokens?: number;
  topP?: number;
  frequencyPenalty?: number;
  presencePenalty?: number;
  stopSequences?: string[];
}

export interface GenerateRequest {
  prompt: string;
  config?: ModelConfig;
  systemPrompt?: string;
}

export interface GenerateResponse {
  text: string;
  usage?: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
  finishReason?: string;
}

export interface AIModelClient {
  generate(request: GenerateRequest): Promise<GenerateResponse>;
  supportsToolCalling(): boolean;
  getModelName(): string;
  isAvailable(): Promise<boolean>;
}
