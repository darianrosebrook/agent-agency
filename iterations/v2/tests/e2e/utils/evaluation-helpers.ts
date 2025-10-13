/**
 * E2E Evaluation Helper Utilities
 *
 * @author @darianrosebrook
 * @description Common utilities for building evaluation criteria and processing results
 */

import type {
  CriterionResult,
  EvaluationCriterion,
  EvaluationFunction,
} from "../types/evaluation";

/**
 * Create a simple programmatic evaluation criterion
 *
 * @param id Unique identifier
 * @param name Human-readable name
 * @param description What this criterion evaluates
 * @param checkFn Function that returns true if criterion passes
 * @param threshold Minimum score to pass (0-1)
 * @param weight Optional weight for overall score
 * @returns Evaluation criterion
 */
export function createProgrammaticCriterion(
  id: string,
  name: string,
  description: string,
  checkFn: (
    output: unknown,
    context: Record<string, unknown>
  ) => boolean | { passed: boolean; reasoning: string },
  threshold: number = 1.0,
  weight?: number
): EvaluationCriterion {
  const evaluate: EvaluationFunction = async (output, context) => {
    const result = checkFn(output, context);

    if (typeof result === "boolean") {
      return {
        id,
        name,
        score: result ? 1 : 0,
        passed: result,
        threshold,
        reasoning: result ? `${name} check passed` : `${name} check failed`,
      };
    }

    return {
      id,
      name,
      score: result.passed ? 1 : 0,
      passed: result.passed,
      threshold,
      reasoning: result.reasoning,
    };
  };

  return {
    id,
    name,
    description,
    evaluate,
    threshold,
    weight,
  };
}

/**
 * Create a regex-based text matching criterion
 *
 * @param id Unique identifier
 * @param name Human-readable name
 * @param pattern Regex pattern to match
 * @param shouldMatch Whether pattern should be present (true) or absent (false)
 * @returns Evaluation criterion
 */
export function createRegexCriterion(
  id: string,
  name: string,
  pattern: RegExp,
  shouldMatch: boolean = true
): EvaluationCriterion {
  return createProgrammaticCriterion(
    id,
    name,
    `Text ${shouldMatch ? "must" : "must not"} match pattern: ${pattern}`,
    (output) => {
      if (typeof output !== "string") {
        return {
          passed: false,
          reasoning: "Output is not a string",
        };
      }

      const matches = pattern.test(output);
      const passed = shouldMatch ? matches : !matches;

      return {
        passed,
        reasoning: passed
          ? `Pattern ${shouldMatch ? "found" : "not found"} as expected`
          : `Pattern ${shouldMatch ? "not found" : "found"} (unexpected)`,
      };
    }
  );
}

/**
 * Create a criterion that checks for banned phrases
 *
 * @param bannedPhrases Array of phrases that must not appear
 * @param caseSensitive Whether comparison is case-sensitive
 * @returns Evaluation criterion
 */
export function createBannedPhrasesCriterion(
  bannedPhrases: string[],
  caseSensitive: boolean = false
): EvaluationCriterion {
  return createProgrammaticCriterion(
    "no-banned-phrases",
    "No Banned Phrases",
    `Output must not contain: ${bannedPhrases.join(", ")}`,
    (output) => {
      if (typeof output !== "string") {
        return {
          passed: false,
          reasoning: "Output is not a string",
        };
      }

      const text = caseSensitive ? output : output.toLowerCase();
      const phrases = caseSensitive
        ? bannedPhrases
        : bannedPhrases.map((p) => p.toLowerCase());

      const found = phrases.filter((phrase) => text.includes(phrase));

      return {
        passed: found.length === 0,
        reasoning:
          found.length === 0
            ? "No banned phrases found"
            : `Found banned phrases: ${found.join(", ")}`,
      };
    }
  );
}

/**
 * Create a criterion that checks for required elements
 *
 * @param requiredElements Array of elements that must appear
 * @param caseSensitive Whether comparison is case-sensitive
 * @returns Evaluation criterion
 */
export function createRequiredElementsCriterion(
  requiredElements: string[],
  caseSensitive: boolean = false
): EvaluationCriterion {
  return createProgrammaticCriterion(
    "required-elements",
    "Required Elements Present",
    `Output must contain: ${requiredElements.join(", ")}`,
    (output) => {
      if (typeof output !== "string") {
        return {
          passed: false,
          reasoning: "Output is not a string",
        };
      }

      const text = caseSensitive ? output : output.toLowerCase();
      const elements = caseSensitive
        ? requiredElements
        : requiredElements.map((e) => e.toLowerCase());

      const missing = elements.filter((element) => !text.includes(element));

      return {
        passed: missing.length === 0,
        reasoning:
          missing.length === 0
            ? "All required elements present"
            : `Missing required elements: ${missing.join(", ")}`,
      };
    }
  );
}

/**
 * Create a criterion that checks minimum length
 *
 * @param minLength Minimum length required
 * @param maxLength Optional maximum length
 * @returns Evaluation criterion
 */
export function createLengthCriterion(
  minLength: number,
  maxLength?: number
): EvaluationCriterion {
  const description = maxLength
    ? `Output must be between ${minLength} and ${maxLength} characters`
    : `Output must be at least ${minLength} characters`;

  return createProgrammaticCriterion(
    "length-check",
    "Length Check",
    description,
    (output) => {
      if (typeof output !== "string") {
        return {
          passed: false,
          reasoning: "Output is not a string",
        };
      }

      const length = output.length;

      if (length < minLength) {
        return {
          passed: false,
          reasoning: `Output is too short: ${length} chars (min: ${minLength})`,
        };
      }

      if (maxLength && length > maxLength) {
        return {
          passed: false,
          reasoning: `Output is too long: ${length} chars (max: ${maxLength})`,
        };
      }

      return {
        passed: true,
        reasoning: `Length is appropriate: ${length} chars`,
      };
    }
  );
}

/**
 * Combine multiple criteria with AND logic
 *
 * All criteria must pass for the combined criterion to pass
 *
 * @param id Unique identifier for combined criterion
 * @param name Human-readable name
 * @param criteria Array of criteria to combine
 * @returns Combined evaluation criterion
 */
export function combineCriteria(
  id: string,
  name: string,
  criteria: EvaluationCriterion[]
): EvaluationCriterion {
  const evaluate: EvaluationFunction = async (output, context) => {
    const results = await Promise.all(
      criteria.map((c) => c.evaluate(output, context))
    );

    const allPassed = results.every((r) => r.passed);
    const averageScore =
      results.reduce((sum, r) => sum + r.score, 0) / results.length;

    const reasoning = allPassed
      ? `All ${criteria.length} sub-criteria passed`
      : `Failed ${results.filter((r) => !r.passed).length}/${
          criteria.length
        } sub-criteria: ${results
          .filter((r) => !r.passed)
          .map((r) => r.reasoning)
          .join("; ")}`;

    return {
      id,
      name,
      score: averageScore,
      passed: allPassed,
      threshold: 1.0,
      reasoning,
      metadata: {
        subCriteria: results,
      },
    };
  };

  return {
    id,
    name,
    description: `Combined criterion: ${criteria
      .map((c) => c.name)
      .join(", ")}`,
    evaluate,
    threshold: 1.0,
  };
}

/**
 * Helper to format evaluation results as a table
 */
export function formatCriteriaTable(criteria: CriterionResult[]): string {
  const rows = criteria.map((c) => {
    const status = c.passed ? "✅" : "❌";
    const score = `${(c.score * 100).toFixed(1)}%`;
    const threshold = `${(c.threshold * 100).toFixed(0)}%`;
    return `${status} | ${c.name.padEnd(30)} | ${score.padEnd(
      6
    )} | ${threshold.padEnd(6)} | ${c.reasoning}`;
  });

  const header = "   | Criterion".padEnd(32) + " | Score  | Min    | Reasoning";
  const separator = "-".repeat(100);

  return [header, separator, ...rows].join("\n");
}

/**
 * Helper to extract failed criteria from a report
 */
export function getFailedCriteria(
  criteria: CriterionResult[]
): CriterionResult[] {
  return criteria.filter((c) => !c.passed);
}

/**
 * Helper to calculate overall statistics for criteria
 */
export function calculateCriteriaStats(criteria: CriterionResult[]): {
  totalCount: number;
  passedCount: number;
  failedCount: number;
  passRate: number;
  averageScore: number;
  lowestScore: number;
  highestScore: number;
} {
  const totalCount = criteria.length;
  const passedCount = criteria.filter((c) => c.passed).length;
  const failedCount = totalCount - passedCount;
  const passRate = totalCount > 0 ? passedCount / totalCount : 0;

  const scores = criteria.map((c) => c.score);
  const averageScore =
    scores.length > 0
      ? scores.reduce((sum, s) => sum + s, 0) / scores.length
      : 0;
  const lowestScore = scores.length > 0 ? Math.min(...scores) : 0;
  const highestScore = scores.length > 0 ? Math.max(...scores) : 0;

  return {
    totalCount,
    passedCount,
    failedCount,
    passRate,
    averageScore,
    lowestScore,
    highestScore,
  };
}
