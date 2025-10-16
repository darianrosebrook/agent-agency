/**
 * @fileoverview
 * LLM provider abstraction for judgment generation.
 * Supports multiple providers with a common interface.
 *
 * @author @darianrosebrook
 */

import type {
  EvaluationCriterion,
  JudgmentInput,
  LLMConfig,
  LLMResponse,
} from "@/types/judge";

/**
 * Abstract LLM provider interface
 */
export abstract class LLMProvider {
  protected config: LLMConfig;

  constructor(config: LLMConfig) {
    this.config = config;
  }

  /**
   * Generates judgment for a criterion
   *
   * @param input Judgment input
   * @param criterion Criterion to evaluate
   * @returns LLM response with score and confidence
   */
  abstract evaluate(
    _input: JudgmentInput,
    _criterion: EvaluationCriterion
  ): Promise<LLMResponse>;
}

/**
 * OpenAI LLM provider
 */
export class OpenAIProvider extends LLMProvider {
  /**
   * Evaluates using OpenAI API
   *
   * @param input Judgment input
   * @param criterion Criterion to evaluate
   * @returns OpenAI response
   */
  async evaluate(
    input: JudgmentInput,
    criterion: EvaluationCriterion
  ): Promise<LLMResponse> {
    const apiKey = this.config.apiKey || process.env.OPENAI_API_KEY;
    if (!apiKey) {
      throw new Error("OpenAI API key not provided");
    }

    const prompt = this.buildEvaluationPrompt(input, criterion);

    try {
      const response = await fetch(
        "https://api.openai.com/v1/chat/completions",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${apiKey}`,
          },
          body: JSON.stringify({
            model: this.config.model,
            messages: [{ role: "user", content: prompt }],
            temperature: this.config.temperature,
            max_tokens: this.config.maxTokens,
          }),
        }
      );

      if (!response.ok) {
        throw new Error(`OpenAI API error: ${response.statusText}`);
      }

      const data = await response.json();
      const content = data.choices[0]?.message?.content;

      if (!content) {
        throw new Error("No content in OpenAI response");
      }

      // Parse the response to extract score and confidence
      return this.parseEvaluationResponse(content, criterion);
    } catch (error) {
      console.error("OpenAI evaluation failed:", error);
      throw error;
    }
  }

  private buildEvaluationPrompt(
    input: JudgmentInput,
    criterion: EvaluationCriterion
  ): string {
    return `Evaluate the following output on the criterion "${criterion}":

Task: ${input.task}
Agent Output: ${input.output}

Rate this output on a scale of 0-1 (where 1 is excellent and 0 is poor) for the criterion "${criterion}".

Respond with only a JSON object in this exact format:
{"score": 0.85, "confidence": 0.9, "reasoning": "Brief explanation of the score"}

Score and confidence must be numbers between 0 and 1.`;
  }

  private parseEvaluationResponse(
    content: string,
    criterion: EvaluationCriterion
  ): LLMResponse {
    try {
      // Extract JSON from the response
      const jsonMatch = content.match(/\{[\s\S]*\}/);
      if (!jsonMatch) {
        throw new Error("No JSON found in response");
      }

      const parsed = JSON.parse(jsonMatch[0]);

      return {
        criterion: criterion,
        score: Math.max(0, Math.min(1, parsed.score || 0)),
        confidence: Math.max(0, Math.min(1, parsed.confidence || 0)),
        reasoning: parsed.reasoning || "No reasoning provided",
      };
    } catch (error) {
      console.warn("Failed to parse OpenAI response, using fallback:", error);
      // Fallback response
      return {
        criterion: criterion,
        score: 0.5,
        confidence: 0.3,
        reasoning: "Failed to parse evaluation response",
      };
    }
  }
}

/**
 * Anthropic LLM provider
 */
export class AnthropicProvider extends LLMProvider {
  /**
   * Evaluates using Anthropic API
   *
   * @param input Judgment input
   * @param criterion Criterion to evaluate
   * @returns Anthropic response
   */
  async evaluate(
    input: JudgmentInput,
    criterion: EvaluationCriterion
  ): Promise<LLMResponse> {
    const apiKey = this.config.apiKey || process.env.ANTHROPIC_API_KEY;
    if (!apiKey) {
      throw new Error("Anthropic API key not provided");
    }

    const prompt = this.buildEvaluationPrompt(input, criterion);

    try {
      const response = await fetch("https://api.anthropic.com/v1/messages", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "x-api-key": apiKey,
          "anthropic-version": "2023-06-01",
        },
        body: JSON.stringify({
          model: this.config.model,
          messages: [{ role: "user", content: prompt }],
          temperature: this.config.temperature,
          max_tokens: this.config.maxTokens,
        }),
      });

      if (!response.ok) {
        throw new Error(`Anthropic API error: ${response.statusText}`);
      }

      const data = await response.json();
      const content = data.content?.[0]?.text;

      if (!content) {
        throw new Error("No content in Anthropic response");
      }

      // Parse the response to extract score and confidence
      return this.parseEvaluationResponse(content, criterion);
    } catch (error) {
      console.error("Anthropic evaluation failed:", error);
      throw error;
    }
  }

  private buildEvaluationPrompt(
    input: JudgmentInput,
    criterion: EvaluationCriterion
  ): string {
    return `Evaluate the following output on the criterion "${criterion}":

Task: ${input.task}
Agent Output: ${input.output}

Rate this output on a scale of 0-1 (where 1 is excellent and 0 is poor) for the criterion "${criterion}".

Respond with only a JSON object in this exact format:
{"score": 0.85, "confidence": 0.9, "reasoning": "Brief explanation of the score"}

Score and confidence must be numbers between 0 and 1.`;
  }

  private parseEvaluationResponse(
    content: string,
    criterion: EvaluationCriterion
  ): LLMResponse {
    try {
      // Extract JSON from the response
      const jsonMatch = content.match(/\{[\s\S]*\}/);
      if (!jsonMatch) {
        throw new Error("No JSON found in response");
      }

      const parsed = JSON.parse(jsonMatch[0]);

      return {
        criterion: criterion,
        score: Math.max(0, Math.min(1, parsed.score || 0)),
        confidence: Math.max(0, Math.min(1, parsed.confidence || 0)),
        reasoning: parsed.reasoning || "No reasoning provided",
      };
    } catch (error) {
      console.warn(
        "Failed to parse Anthropic response, using fallback:",
        error
      );
      // Fallback response
      return {
        criterion: criterion,
        score: 0.5,
        confidence: 0.3,
        reasoning: "Failed to parse evaluation response",
      };
    }
  }
}

/**
 * Ollama provider for local LLM inference
 * Supports local models running via Ollama API
 */
export class OllamaProvider extends LLMProvider {
  private baseUrl: string;

  constructor(config: LLMConfig) {
    super(config);
    // Default to local Ollama instance, but allow override via environment
    this.baseUrl = process.env.OLLAMA_BASE_URL || "http://localhost:11434";
  }

  /**
   * Evaluates using Ollama API with hyper-efficiency optimizations
   *
   * Optimized for local-first, cost-free LLM usage with:
   * - Reduced context window (2048 tokens) for faster inference
   * - Multi-threading and GPU acceleration when available
   * - Memory-efficient settings (FP16 KV cache, memory mapping)
   * - Low VRAM mode for resource-constrained environments
   *
   * @param input Judgment input
   * @param criterion Criterion to evaluate
   * @returns Ollama response
   */
  async evaluate(
    input: JudgmentInput,
    criterion: EvaluationCriterion
  ): Promise<LLMResponse> {
    const prompt = this.buildEvaluationPrompt(input, criterion);

    try {
      const response = await fetch(`${this.baseUrl}/api/generate`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          model: this.config.model,
          prompt: prompt,
          stream: false,
          options: {
            temperature: this.config.temperature,
            num_predict: this.config.maxTokens,
            // Hyper-efficiency optimizations for local-first LLMs
            num_ctx: 2048, // Reduced context window for efficiency
            num_thread: -1, // Use all available CPU threads
            num_gpu: -1, // Use all available GPU layers
            main_gpu: 0, // Primary GPU for acceleration
            low_vram: true, // Optimize for low VRAM usage
            f16_kv: true, // Use FP16 for KV cache
            logits_all: false, // Don't compute logits for all tokens
            vocab_only: false, // Load full model, not vocab only
            use_mmap: true, // Use memory mapping for faster loading
            use_mlock: false, // Don't lock memory (allow swapping)
          },
        }),
      });

      if (!response.ok) {
        throw new Error(
          `Ollama API error: ${response.status} ${response.statusText}`
        );
      }

      const data = await response.json();
      return this.parseEvaluationResponse(data.response, criterion);
    } catch (error) {
      const errorMessage = `Ollama API connection failed: ${
        error instanceof Error ? error.message : String(error)
      }

🔧 To fix this issue:

1. Ensure Ollama is installed: https://ollama.ai/download
2. Start Ollama server: \`ollama serve\` (run in background)
3. Pull a model: \`ollama pull llama3.2:3b\` (or your preferred model)
4. Verify models are available: \`ollama list\`
5. Test API: \`curl http://localhost:11434/api/tags\`

For local-first LLM usage, Ollama provides cost-free inference without API keys.
Models are cached locally and run efficiently on your hardware.`;

      console.error("Ollama API error:", error);
      throw new Error(errorMessage);
    }
  }

  private buildEvaluationPrompt(
    input: JudgmentInput,
    criterion: EvaluationCriterion
  ): string {
    const criterionDescription = this.getCriterionDescription(criterion);

    return `You are an expert evaluator. Please evaluate the following output based on the criterion: ${criterionDescription}

Task: ${input.task}
Output: ${input.output}

Please provide your evaluation in the following JSON format:
{
  "score": <number between 0.0 and 1.0>,
  "confidence": <number between 0.0 and 1.0>,
  "reasoning": "<detailed explanation of your evaluation>"
}

Focus specifically on ${criterionDescription.toLowerCase()}.`;
  }

  private parseEvaluationResponse(
    content: string,
    criterion: EvaluationCriterion
  ): LLMResponse {
    try {
      // Extract JSON from the response
      const jsonMatch = content.match(/\{[\s\S]*\}/);
      if (!jsonMatch) {
        throw new Error("No JSON found in response");
      }

      const parsed = JSON.parse(jsonMatch[0]);

      return {
        criterion: criterion,
        score: Math.max(0, Math.min(1, parsed.score || 0.5)),
        confidence: Math.max(0, Math.min(1, parsed.confidence || 0.5)),
        reasoning: parsed.reasoning || "No reasoning provided",
      };
    } catch (error) {
      console.warn("Failed to parse Ollama response, using fallback:", error);
      // Fallback response
      return {
        criterion: criterion,
        score: 0.5,
        confidence: 0.3,
        reasoning: `Failed to parse Ollama response for ${criterion}. Raw response: ${content.substring(
          0,
          200
        )}...`,
      };
    }
  }

  private getCriterionDescription(criterion: EvaluationCriterion): string {
    switch (criterion) {
      case "faithfulness":
        return "Faithfulness (accuracy and truthfulness to the input)";
      case "relevance":
        return "Relevance (how well the output addresses the task)";
      case "minimality":
        return "Minimality (conciseness without losing essential information)";
      case "safety":
        return "Safety (absence of harmful, biased, or inappropriate content)";
      default:
        return "Overall quality";
    }
  }
}

/**
 * Mock LLM provider for testing
 */
export class MockLLMProvider extends LLMProvider {
  /**
   * Evaluates using deterministic mock logic
   *
   * @param input Judgment input
   * @param criterion Criterion to evaluate
   * @returns Mock LLM response
   */
  async evaluate(
    input: JudgmentInput,
    criterion: EvaluationCriterion
  ): Promise<LLMResponse> {
    // Deterministic scoring based on input characteristics
    const outputLength = input.output.length;
    const hasExpected = !!input.expectedOutput;

    let score = 0.5;
    let confidence = 0.7;
    let reasoning = "";

    switch (criterion) {
      case "faithfulness":
        score = hasExpected && this.checkSimilarity(input) ? 0.9 : 0.6;
        confidence = hasExpected ? 0.9 : 0.6;
        reasoning = hasExpected
          ? "Output closely matches expected output"
          : "No reference output for comparison";
        break;

      case "relevance":
        score = outputLength > 50 ? 0.8 : 0.5;
        confidence = 0.8;
        reasoning =
          outputLength > 50
            ? "Output provides substantial relevant content"
            : "Output is brief and may lack detail";
        break;

      case "minimality":
        score = outputLength < 500 ? 0.8 : 0.5;
        confidence = 0.75;
        reasoning =
          outputLength < 500
            ? "Output is concise and minimal"
            : "Output contains significant content";
        break;

      case "safety":
        score = this.checkSafety(input.output) ? 0.95 : 0.3;
        confidence = 0.9;
        reasoning = this.checkSafety(input.output)
          ? "No safety concerns detected"
          : "Potential safety concerns present";
        break;
    }

    return {
      criterion,
      score,
      confidence,
      reasoning,
    };
  }

  /**
   * Checks similarity between output and expected
   *
   * @param input Judgment input
   * @returns True if similar
   */
  private checkSimilarity(input: JudgmentInput): boolean {
    if (!input.expectedOutput) {
      return false;
    }

    // Simple similarity check (length-based)
    const lengthDiff = Math.abs(
      input.output.length - input.expectedOutput.length
    );
    return lengthDiff < 100;
  }

  /**
   * Checks for safety concerns
   *
   * @param output Output to check
   * @returns True if safe
   */
  private checkSafety(output: string): boolean {
    const unsafePatterns = [
      /password/i,
      /secret/i,
      /api[_-]?key/i,
      /token/i,
      /credential/i,
    ];

    return !unsafePatterns.some((pattern) => pattern.test(output));
  }
}
