/**
 * Ollama AI Client
 *
 * @author @darianrosebrook
 * @description Ollama integration for local AI model inference
 */

import * as ollama from "ollama";
import { Logger } from "../utils/Logger";
import { AIModelClient, GenerateRequest, GenerateResponse } from "./types";

export interface OllamaConfig {
  model: string; // e.g., 'gemma:3n', 'llama2:7b'
  host?: string; // default: 'http://localhost:11434'
  timeout?: number; // request timeout in ms
}

export class OllamaClient implements AIModelClient {
  private config: OllamaConfig;
  private logger: Logger;

  constructor(config: OllamaConfig, logger?: Logger) {
    this.config = {
      host: "http://localhost:11434",
      timeout: 30000,
      ...config,
    };
    this.logger = logger || new Logger("OllamaClient");
  }

  async generate(request: GenerateRequest): Promise<GenerateResponse> {
    try {
      const startTime = Date.now();

      // Prepare the messages array for Ollama
      const messages: Array<{
        role: "system" | "user" | "assistant";
        content: string;
      }> = [];

      // Add system prompt if provided
      if (request.systemPrompt) {
        messages.push({
          role: "system",
          content: request.systemPrompt,
        });
      }

      // Add user prompt
      messages.push({
        role: "user",
        content: request.prompt,
      });

      // Call Ollama API
      const response = await ollama.chat({
        model: this.config.model,
        messages,
        options: {
          temperature: request.config?.temperature ?? 0.7,
          num_predict: request.config?.maxTokens ?? 1024,
          top_p: request.config?.topP ?? 0.9,
          frequency_penalty: request.config?.frequencyPenalty ?? 0,
          presence_penalty: request.config?.presencePenalty ?? 0,
          stop: request.config?.stopSequences,
          timeout: this.config.timeout,
        },
        host: this.config.host,
      });

      const duration = Date.now() - startTime;
      this.logger.debug(`Ollama generation completed in ${duration}ms`, {
        model: this.config.model,
        promptTokens: response.prompt_eval_count,
        completionTokens: response.eval_count,
      });

      return {
        text: response.message.content,
        usage: {
          promptTokens: response.prompt_eval_count || 0,
          completionTokens: response.eval_count || 0,
          totalTokens:
            (response.prompt_eval_count || 0) + (response.eval_count || 0),
        },
        finishReason: response.done_reason || "stop",
      };
    } catch (error) {
      this.logger.error("Ollama generation failed", error);
      throw new Error(`Ollama generation failed: ${(error as Error).message}`);
    }
  }

  supportsToolCalling(): boolean {
    // Ollama models don't currently support tool calling
    // This could change with future model support
    return false;
  }

  getModelName(): string {
    return this.config.model;
  }

  async isAvailable(): Promise<boolean> {
    try {
      // Try to list models to check if Ollama is running
      await ollama.list({ host: this.config.host });
      return true;
    } catch (error) {
      this.logger.warn("Ollama service not available", error);
      return false;
    }
  }

  /**
   * List available models
   */
  async listModels(): Promise<string[]> {
    try {
      const response = await ollama.list({ host: this.config.host });
      return response.models.map((model) => model.name);
    } catch (error) {
      this.logger.error("Failed to list Ollama models", error);
      return [];
    }
  }

  /**
   * Pull a model if not available
   */
  async pullModel(modelName: string): Promise<void> {
    try {
      this.logger.info(`Pulling Ollama model: ${modelName}`);
      await ollama.pull({
        model: modelName,
        host: this.config.host,
      });
      this.logger.info(`Successfully pulled model: ${modelName}`);
    } catch (error) {
      this.logger.error(`Failed to pull model ${modelName}`, error);
      throw error;
    }
  }
}
