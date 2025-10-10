/**
 * Ollama AI Client
 *
 * @author @darianrosebrook
 * @description Ollama integration for local AI model inference
 */

// Dynamic import for ES module compatibility
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
  private client?: any;

  constructor(config: OllamaConfig, logger?: Logger) {
    this.config = {
      host: "http://localhost:11434",
      timeout: 30000,
      ...config,
    };
    this.logger = logger || new Logger("OllamaClient");
  }

  private async ensureClient(): Promise<void> {
    if (!this.client) {
      const { Ollama } = await import("ollama");
      this.client = new Ollama();
    }
  }

  async generate(request: GenerateRequest): Promise<GenerateResponse> {
    await this.ensureClient();
    const startTime = Date.now();

    try {
      // Prepare the prompt with system message if provided
      let fullPrompt = request.prompt;
      if (request.systemPrompt) {
        fullPrompt = `${request.systemPrompt}\n\n${request.prompt}`;
      }

      // Call Ollama API using the generate method
      const generator = await this.client.generate(
        this.config.model,
        fullPrompt,
        {
          num_predict: request.config?.maxTokens ?? 1024,
        } as any
      );

      let fullResponse = "";
      let finalResult: any = {};

      // Collect all generated text from the async generator
      for await (const part of generator) {
        if (typeof part === "string") {
          fullResponse += part;
        } else {
          finalResult = part;
        }
      }

      const duration = Date.now() - startTime;
      this.logger.debug(`Ollama generation completed in ${duration}ms`, {
        model: this.config.model,
        responseLength: fullResponse.length,
      });

      return {
        text: fullResponse,
        usage: {
          promptTokens: finalResult.prompt_eval_count || 0,
          completionTokens: finalResult.eval_count || 0,
          totalTokens:
            (finalResult.prompt_eval_count || 0) +
            (finalResult.eval_count || 0),
        },
        finishReason: finalResult.done ? "completed" : "unknown",
      };
    } catch (error) {
      throw new Error(
        `Ollama generation failed: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
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
    await this.ensureClient();
    try {
      // Try to list models to check if Ollama is running
      await this.client.tags();
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
      const response = await this.client.tags();
      return (response as any).models?.map((model: any) => model.name) || [];
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
      await this.client.pull({ model: modelName } as any);
      this.logger.info(`Successfully pulled model: ${modelName}`);
    } catch (error) {
      this.logger.error(`Failed to pull model ${modelName}`, error);
      throw error;
    }
  }
}
