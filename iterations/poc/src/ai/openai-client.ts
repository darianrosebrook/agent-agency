/**
 * OpenAI Client - Enterprise-grade OpenAI API integration
 *
 * @author @darianrosebrook
 * @description OpenAI API client with advanced features and error handling
 */

import { Logger } from "../utils/Logger.js";
import type {
  AIModelClient,
  GenerateRequest,
  GenerateResponse,
} from "./types.js";

export interface OpenAIConfig {
  apiKey: string;
  baseURL?: string;
  organization?: string;
  project?: string;
  model: string;
  timeout?: number;
  maxRetries?: number;
  retryDelay?: number;
}

export class OpenAIClient implements AIModelClient {
  private config: OpenAIConfig;
  private logger: Logger;

  constructor(config: OpenAIConfig, logger?: Logger) {
    this.config = config;
    this.logger = logger || new Logger("OpenAIClient");

    if (!config.apiKey) {
      throw new Error("OpenAI API key is required");
    }
  }

  async generate(request: GenerateRequest): Promise<GenerateResponse> {
    const startTime = Date.now();

    try {
      this.logger.debug("Generating with OpenAI", {
        model: this.config.model,
        promptLength: request.prompt.length,
        hasSystemPrompt: !!request.systemPrompt,
      });

      // Prepare messages for OpenAI chat completions API
      const messages = [];

      // Add system message if provided
      if (request.systemPrompt) {
        messages.push({
          role: "system",
          content: request.systemPrompt,
        });
      }

      // Add user message
      messages.push({
        role: "user",
        content: request.prompt,
      });

      // Prepare request payload
      const payload = {
        model: this.config.model,
        messages,
        temperature: request.config?.temperature ?? 0.7,
        max_tokens: request.config?.maxTokens ?? 1024,
        top_p: request.config?.topP ?? 1.0,
        frequency_penalty: request.config?.frequencyPenalty ?? 0.0,
        presence_penalty: request.config?.presencePenalty ?? 0.0,
        stop: request.config?.stopSequences,
      };

      // Make API request with retry logic
      let lastError: Error | null = null;

      for (
        let attempt = 0;
        attempt <= (this.config.maxRetries ?? 3);
        attempt++
      ) {
        try {
          const response = await this.makeRequest(payload);

          const duration = Date.now() - startTime;
          this.logger.info("OpenAI generation completed", {
            model: this.config.model,
            duration,
            usage: response.usage,
            finishReason: response.choices?.[0]?.finish_reason,
          });

          return {
            text: response.choices?.[0]?.message?.content || "",
            usage: response.usage
              ? {
                  promptTokens: response.usage.prompt_tokens,
                  completionTokens: response.usage.completion_tokens,
                  totalTokens: response.usage.total_tokens,
                }
              : undefined,
            finishReason: response.choices?.[0]?.finish_reason,
          };
        } catch (error) {
          lastError = error as Error;
          this.logger.warn(`OpenAI request attempt ${attempt + 1} failed`, {
            error: lastError.message,
            model: this.config.model,
          });

          // Wait before retry (exponential backoff)
          if (attempt < (this.config.maxRetries ?? 3)) {
            const delay =
              (this.config.retryDelay ?? 1000) * Math.pow(2, attempt);
            await new Promise((resolve) => setTimeout(resolve, delay));
          }
        }
      }

      // All retries failed
      throw lastError || new Error("OpenAI request failed after all retries");
    } catch (error) {
      const duration = Date.now() - startTime;
      this.logger.error("OpenAI generation failed", {
        model: this.config.model,
        error: (error as Error).message,
        duration,
      });
      throw error;
    }
  }

  supportsToolCalling(): boolean {
    // OpenAI models support tool calling for GPT-4 and newer models
    return (
      this.config.model.includes("gpt-4") ||
      this.config.model.includes("gpt-3.5-turbo")
    );
  }

  getModelName(): string {
    return this.config.model;
  }

  async isAvailable(): Promise<boolean> {
    try {
      // Try a simple request to check availability
      const testRequest: GenerateRequest = {
        prompt: "Hello",
        config: { maxTokens: 1 },
      };

      await this.generate(testRequest);
      return true;
    } catch (error) {
      this.logger.warn("OpenAI availability check failed", {
        error: (error as Error).message,
      });
      return false;
    }
  }

  private async makeRequest(payload: any): Promise<any> {
    const url = `${
      this.config.baseURL || "https://api.openai.com/v1"
    }/chat/completions`;

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      Authorization: `Bearer ${this.config.apiKey}`,
    };

    if (this.config.organization) {
      headers["OpenAI-Organization"] = this.config.organization;
    }

    if (this.config.project) {
      headers["OpenAI-Project"] = this.config.project;
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(
      () => controller.abort(),
      this.config.timeout ?? 30000
    );

    try {
      const response = await fetch(url, {
        method: "POST",
        headers,
        body: JSON.stringify(payload),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const errorData = await response.text();
        throw new Error(`OpenAI API error ${response.status}: ${errorData}`);
      }

      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof Error && error.name === "AbortError") {
        throw new Error("OpenAI request timed out");
      }

      throw error;
    }
  }
}
