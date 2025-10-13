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
    input: JudgmentInput,
    criterion: EvaluationCriterion
  ): Promise<LLMResponse>;
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
