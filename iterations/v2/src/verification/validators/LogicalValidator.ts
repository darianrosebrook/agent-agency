/**
 * @fileoverview Logical Validator (ARBITER-007)
 *
 * Validates logical reasoning, argument structure, and inference chains,
 * detecting logical fallacies and invalid arguments.
 *
 * @author @darianrosebrook
 */

import {
  VerificationMethodResult,
  VerificationRequest,
  VerificationType,
  VerificationVerdict,
} from "../../types/verification";

/**
 * Configuration for logical validation
 */
export interface LogicalConfig {
  reasoningEngine: "symbolic" | "simple";
  detectFallacies: boolean;
  strictMode?: boolean;
}

/**
 * Logical argument structure
 */
interface LogicalArgument {
  premises: string[];
  conclusion?: string;
  connectives: LogicalConnective[];
  structure: "deductive" | "inductive" | "abductive" | "unknown";
}

/**
 * Logical connective found in text
 */
interface LogicalConnective {
  type: "if-then" | "and" | "or" | "not" | "if-and-only-if";
  position: number;
  text: string;
}

/**
 * Detected logical fallacy
 */
interface LogicalFallacy {
  type: string;
  description: string;
  location: string;
  severity: "high" | "medium" | "low";
}

/**
 * Logical Validator
 *
 * Validates logical reasoning and detects fallacies in arguments.
 */
export class LogicalValidator {
  private config: LogicalConfig;

  private readonly fallacyPatterns: Array<{
    name: string;
    pattern: RegExp;
    description: string;
    severity: "high" | "medium" | "low";
  }> = [
    {
      name: "Ad Hominem",
      pattern: /\b(stupid|idiot|moron|fool|ignorant)\b/i,
      description: "Attacking the person instead of the argument",
      severity: "high",
    },
    {
      name: "Straw Man",
      pattern:
        /\b(they say|some people say|critics claim)\b.*\b(but actually|in reality|the truth is)\b/i,
      description: "Misrepresenting an argument to make it easier to attack",
      severity: "high",
    },
    {
      name: "False Dichotomy",
      pattern: /\b(either|only two|must be)\b.*\b(or|otherwise)\b/i,
      description: "Presenting only two options when more exist",
      severity: "medium",
    },
    {
      name: "Appeal to Authority",
      pattern:
        /\b(expert|authority|scientist|doctor)\s+(says|claims|believes)\b/i,
      description: "Claiming something is true because an authority says so",
      severity: "low",
    },
    {
      name: "Slippery Slope",
      pattern:
        /\b(if we|if you).*\b(then|eventually|ultimately)\b.*\b(will|would|could)\b/i,
      description:
        "Arguing that one thing will inevitably lead to extreme consequences",
      severity: "medium",
    },
    {
      name: "Circular Reasoning",
      pattern: /\b(because|since)\b.*\b(therefore|thus|so)\b.*\1/i,
      description: "Using the conclusion as a premise",
      severity: "high",
    },
    {
      name: "Hasty Generalization",
      pattern: /\b(all|every|always|never|none)\b.*\b(are|is|was|were)\b/i,
      description: "Drawing broad conclusions from limited evidence",
      severity: "medium",
    },
  ];

  constructor(config: Partial<LogicalConfig> = {}) {
    this.config = {
      reasoningEngine: config.reasoningEngine ?? "symbolic",
      detectFallacies: config.detectFallacies ?? true,
      strictMode: config.strictMode ?? false,
    };
  }

  /**
   * Verify logical reasoning in content
   */
  async verify(
    request: VerificationRequest
  ): Promise<VerificationMethodResult> {
    const startTime = Date.now();

    try {
      // Parse logical structure
      const argument = this.parseLogicalStructure(request.content);

      if (
        argument.premises.length === 0 &&
        !argument.conclusion &&
        argument.connectives.length === 0
      ) {
        return {
          method: VerificationType.LOGICAL_VALIDATION,
          verdict: VerificationVerdict.INSUFFICIENT_DATA,
          confidence: 0,
          reasoning: ["No logical argument structure found in content"],
          processingTimeMs: Date.now() - startTime,
          evidenceCount: 0,
        };
      }

      // Detect fallacies
      const fallacies = this.config.detectFallacies
        ? this.detectFallacies(request.content, argument)
        : [];

      // Check argument validity
      const validity = this.checkArgumentValidity(argument);

      // Assess overall logic
      const assessment = this.assessLogic(argument, validity, fallacies);

      return {
        method: VerificationType.LOGICAL_VALIDATION,
        verdict: assessment.verdict,
        confidence: assessment.confidence,
        reasoning: assessment.reasoning,
        processingTimeMs: Date.now() - startTime,
        evidenceCount: argument.premises.length + argument.connectives.length,
      };
    } catch (error) {
      return {
        method: VerificationType.LOGICAL_VALIDATION,
        verdict: VerificationVerdict.ERROR,
        confidence: 0,
        reasoning: [
          `Logical validation failed: ${
            error instanceof Error ? error.message : String(error)
          }`,
        ],
        processingTimeMs: Date.now() - startTime,
        evidenceCount: 0,
      };
    }
  }

  /**
   * Parse logical structure from content
   */
  private parseLogicalStructure(content: string): LogicalArgument {
    const sentences = content
      .split(/[.!?]+/)
      .map((s) => s.trim())
      .filter((s) => s.length > 0);

    const premises: string[] = [];
    let conclusion: string | undefined;
    const connectives: LogicalConnective[] = [];

    for (let i = 0; i < sentences.length; i++) {
      const sentence = sentences[i];

      // Detect conclusion indicators
      if (
        /\b(therefore|thus|hence|consequently|so|it follows that)\b/i.test(
          sentence
        )
      ) {
        conclusion = sentence;
      } else if (
        /\b(because|since|given that|assuming that|if)\b/i.test(sentence)
      ) {
        premises.push(sentence);
      } else if (i < sentences.length - 1) {
        premises.push(sentence);
      } else {
        // Last sentence might be conclusion
        conclusion = sentence;
      }

      // Detect connectives
      const ifThen = sentence.match(/\b(if|when)\b.*\b(then|,)\b/i);
      if (ifThen) {
        connectives.push({
          type: "if-then",
          position: i,
          text: ifThen[0],
        });
      }

      const and = sentence.match(/\b(and|also|furthermore|moreover)\b/i);
      if (and) {
        connectives.push({
          type: "and",
          position: i,
          text: and[0],
        });
      }

      const or = sentence.match(/\b(or|alternatively|either)\b/i);
      if (or) {
        connectives.push({
          type: "or",
          position: i,
          text: or[0],
        });
      }

      const not = sentence.match(/\b(not|no|never|neither)\b/i);
      if (not) {
        connectives.push({
          type: "not",
          position: i,
          text: not[0],
        });
      }
    }

    // Determine argument structure
    let structure: LogicalArgument["structure"] = "unknown";
    if (connectives.some((c) => c.type === "if-then")) {
      structure = "deductive";
    } else if (/\b(probably|likely|suggests|indicates)\b/i.test(content)) {
      structure = "inductive";
    } else if (
      /\b(best explanation|most likely|explains why)\b/i.test(content)
    ) {
      structure = "abductive";
    }

    return {
      premises,
      conclusion,
      connectives,
      structure,
    };
  }

  /**
   * Detect logical fallacies in content
   */
  private detectFallacies(
    content: string,
    argument: LogicalArgument
  ): LogicalFallacy[] {
    const fallacies: LogicalFallacy[] = [];

    for (const pattern of this.fallacyPatterns) {
      if (pattern.pattern.test(content)) {
        fallacies.push({
          type: pattern.name,
          description: pattern.description,
          location: this.findFallacyLocation(content, pattern.pattern),
          severity: pattern.severity,
        });
      }
    }

    // Check for circular reasoning in argument structure
    if (argument.premises.length > 0 && argument.conclusion) {
      const conclusionWords = new Set(
        argument.conclusion.toLowerCase().split(/\s+/)
      );

      for (const premise of argument.premises) {
        const premiseWords = new Set(premise.toLowerCase().split(/\s+/));
        const overlap = [...conclusionWords].filter((w) => premiseWords.has(w));

        if (overlap.length > 3) {
          fallacies.push({
            type: "Potential Circular Reasoning",
            description: "Premise and conclusion share significant wording",
            location: premise.substring(0, 50) + "...",
            severity: "medium",
          });
        }
      }
    }

    return fallacies;
  }

  /**
   * Find location of fallacy in content
   */
  private findFallacyLocation(content: string, pattern: RegExp): string {
    const match = content.match(pattern);
    if (!match) {
      return "unknown";
    }

    const index = content.indexOf(match[0]);
    const start = Math.max(0, index - 20);
    const end = Math.min(content.length, index + match[0].length + 20);

    return "..." + content.substring(start, end) + "...";
  }

  /**
   * Check argument validity
   */
  private checkArgumentValidity(argument: LogicalArgument): {
    valid: boolean;
    issues: string[];
  } {
    const issues: string[] = [];

    // Check for basic structural issues
    if (argument.premises.length === 0) {
      issues.push("No clear premises identified");
    }

    if (!argument.conclusion) {
      issues.push("No clear conclusion identified");
    }

    // Check deductive reasoning patterns
    if (argument.structure === "deductive") {
      const hasIfThen = argument.connectives.some((c) => c.type === "if-then");
      if (!hasIfThen) {
        issues.push("Deductive structure lacks clear if-then relationship");
      }
    }

    // Check for sufficient premises
    if (argument.premises.length < 2 && argument.structure === "deductive") {
      issues.push(
        "Deductive argument typically requires at least two premises"
      );
    }

    const valid = issues.length === 0;

    return { valid, issues };
  }

  /**
   * Assess overall logical quality
   */
  private assessLogic(
    argument: LogicalArgument,
    validity: { valid: boolean; issues: string[] },
    fallacies: LogicalFallacy[]
  ): {
    verdict: VerificationVerdict;
    confidence: number;
    reasoning: string[];
  } {
    const reasoning: string[] = [];

    // Report argument structure
    reasoning.push(
      `Argument structure: ${argument.structure} (${argument.premises.length} premises)`
    );

    if (argument.connectives.length > 0) {
      const connectiveTypes = Array.from(
        new Set(argument.connectives.map((c) => c.type))
      );
      reasoning.push(`Logical connectives: ${connectiveTypes.join(", ")}`);
    }

    // No fallacies and valid structure
    if (fallacies.length === 0 && validity.valid) {
      reasoning.push("No logical fallacies detected");
      reasoning.push("Argument structure is logically valid");

      return {
        verdict: VerificationVerdict.VERIFIED_TRUE,
        confidence: 0.75,
        reasoning,
      };
    }

    // Valid structure but has fallacies
    if (validity.valid && fallacies.length > 0) {
      reasoning.push(
        `Detected ${fallacies.length} potential fallac${
          fallacies.length === 1 ? "y" : "ies"
        }:`
      );

      for (const fallacy of fallacies.slice(0, 3)) {
        reasoning.push(
          `${fallacy.severity.toUpperCase()}: ${fallacy.type} - ${
            fallacy.description
          }`
        );
      }

      const highSeverityCount = fallacies.filter(
        (f) => f.severity === "high"
      ).length;

      if (highSeverityCount > 0) {
        return {
          verdict: VerificationVerdict.VERIFIED_FALSE,
          confidence: 0.7,
          reasoning,
        };
      }

      return {
        verdict: VerificationVerdict.PARTIALLY_TRUE,
        confidence: 0.5,
        reasoning,
      };
    }

    // Invalid structure
    if (!validity.valid) {
      reasoning.push("Logical structure issues:");
      reasoning.push(...validity.issues);

      if (fallacies.length > 0) {
        reasoning.push(
          `Also found ${fallacies.length} fallac${
            fallacies.length === 1 ? "y" : "ies"
          }`
        );
      }

      return {
        verdict: VerificationVerdict.UNVERIFIED,
        confidence: 0.3,
        reasoning,
      };
    }

    return {
      verdict: VerificationVerdict.UNVERIFIED,
      confidence: 0.4,
      reasoning,
    };
  }
}
