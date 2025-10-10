/**
 * E2E Evaluation Framework
 *
 * @author @darianrosebrook
 * @description Framework for evaluating and critiquing agent outputs in E2E tests
 */

export interface EvaluationCriteria {
  id: string;
  name: string;
  description: string;
  weight: number; // 0-1
  evaluate: (output: any, context?: any) => Promise<EvaluationResult>;
}

export interface EvaluationResult {
  passed: boolean;
  score: number; // 0-1
  message: string;
  details?: any;
  suggestions?: string[];
}

export interface EvaluationReport {
  taskId: string;
  criteria: Array<{
    criteria: EvaluationCriteria;
    result: EvaluationResult;
    weightedScore: number;
  }>;
  overallScore: number;
  overallPassed: boolean;
  summary: string;
  timestamp: string;
  duration: number;
}

export interface TestScenario {
  id: string;
  name: string;
  description: string;
  input: any;
  expectedCriteria: EvaluationCriteria[];
  timeout?: number; // ms
  retries?: number;
  maxIterations?: number;
  mockErrors?: MockErrorScenario[];
}

export interface MockErrorScenario {
  iteration: number;
  error: string;
  feedback: string;
}

export class E2EEvaluator {
  private criteria = new Map<string, EvaluationCriteria>();

  /**
   * Register an evaluation criteria
   */
  registerCriteria(criteria: EvaluationCriteria): void {
    this.criteria.set(criteria.id, criteria);
  }

  /**
   * Get registered criteria
   */
  getCriteria(id: string): EvaluationCriteria | undefined {
    return this.criteria.get(id);
  }

  /**
   * Evaluate output against criteria
   */
  async evaluateOutput(
    taskId: string,
    output: any,
    criteriaIds: string[],
    context?: any
  ): Promise<EvaluationReport> {
    const startTime = Date.now();
    const results: Array<{
      criteria: EvaluationCriteria;
      result: EvaluationResult;
      weightedScore: number;
    }> = [];

    let totalWeight = 0;
    let weightedSum = 0;

    for (const criteriaId of criteriaIds) {
      const criteria = this.criteria.get(criteriaId);
      if (!criteria) {
        throw new Error(`Unknown criteria: ${criteriaId}`);
      }

      const result = await criteria.evaluate(output, context);
      const weightedScore = result.score * criteria.weight;

      results.push({
        criteria,
        result,
        weightedScore,
      });

      totalWeight += criteria.weight;
      weightedSum += weightedScore;
    }

    const overallScore = totalWeight > 0 ? weightedSum / totalWeight : 0;
    const overallPassed = results.every((r) => r.result.passed);

    const duration = Date.now() - startTime;

    return {
      taskId,
      criteria: results,
      overallScore,
      overallPassed,
      summary: this.generateSummary(results, overallScore, overallPassed),
      timestamp: new Date().toISOString(),
      duration,
    };
  }

  /**
   * Generate human-readable summary
   */
  private generateSummary(
    results: Array<{
      criteria: EvaluationCriteria;
      result: EvaluationResult;
      weightedScore: number;
    }>,
    overallScore: number,
    overallPassed: boolean
  ): string {
    const passedCount = results.filter((r) => r.result.passed).length;
    const totalCount = results.length;

    let summary = `Evaluation: ${passedCount}/${totalCount} criteria passed. `;
    summary += `Overall score: ${(overallScore * 100).toFixed(1)}%. `;

    if (overallPassed) {
      summary += "✅ All requirements met.";
    } else {
      summary += "❌ Some requirements not met.";
      const failed = results.filter((r) => !r.result.passed);
      summary += ` Failed: ${failed.map((r) => r.criteria.name).join(", ")}.`;
    }

    return summary;
  }

  /**
   * Run a complete evaluation scenario
   */
  async runScenario(scenario: TestScenario): Promise<{
    report: EvaluationReport;
    output: any;
    executionTime: number;
  }> {
    const startTime = Date.now();

    // This would be implemented by subclasses for specific scenarios
    const output = await this.executeScenario(scenario);

    const report = await this.evaluateOutput(
      scenario.id,
      output,
      scenario.expectedCriteria.map((c) => c.id),
      scenario
    );

    const executionTime = Date.now() - startTime;

    return {
      report,
      output,
      executionTime,
    };
  }

  /**
   * Execute a test scenario (to be implemented by subclasses)
   */
  protected async executeScenario(_scenario: TestScenario): Promise<any> {
    throw new Error("executeScenario must be implemented by subclass");
  }
}

// Pre-defined evaluation criteria for common use cases

export const TEXT_TRANSFORMATION_CRITERIA: EvaluationCriteria[] = [
  {
    id: "formal-language",
    name: "Formal Language",
    description: "Output uses professional, formal language without slang",
    weight: 0.3,
    evaluate: async (output: string) => {
      const informalWords = [
        "hey",
        "really casual",
        "got some",
        "let's make it work",
      ];
      const hasInformal = informalWords.some((word) =>
        output.toLowerCase().includes(word.toLowerCase())
      );

      const formalIndicators = [
        "professional",
        "stakeholders",
        "formal",
        "structured",
      ];
      const hasFormal = formalIndicators.some((word) =>
        output.toLowerCase().includes(word.toLowerCase())
      );

      const passed = !hasInformal && hasFormal;
      const score = passed ? 1 : hasFormal ? 0.5 : 0;

      return {
        passed,
        score,
        message: passed
          ? "Uses appropriate formal language"
          : "Contains informal language or lacks formal tone",
        suggestions: passed
          ? []
          : [
              "Replace casual phrases with professional alternatives",
              "Use complete sentences and proper structure",
            ],
      };
    },
  },
  {
    id: "content-structure",
    name: "Content Structure",
    description:
      "Output is well-structured with clear paragraphs and logical flow",
    weight: 0.2,
    evaluate: async (output: string) => {
      const sentences = output
        .split(/[.!?]+/)
        .filter((s) => s.trim().length > 0);
      const hasMultipleSentences = sentences.length >= 3;

      const paragraphs = output
        .split(/\n\s*\n/)
        .filter((p) => p.trim().length > 0);
      const hasParagraphs = paragraphs.length >= 2;

      const passed = hasMultipleSentences && hasParagraphs;
      const score = passed
        ? 1
        : hasMultipleSentences || hasParagraphs
        ? 0.5
        : 0;

      return {
        passed,
        score,
        message: passed
          ? "Well-structured content with clear organization"
          : "Content lacks proper structure or organization",
        suggestions: passed
          ? []
          : [
              "Add paragraph breaks for better readability",
              "Ensure logical flow between ideas",
            ],
      };
    },
  },
  {
    id: "content-length",
    name: "Content Length",
    description: "Output length is appropriate (not too short or verbose)",
    weight: 0.15,
    evaluate: async (output: string) => {
      const wordCount = output.split(/\s+/).length;
      const charCount = output.length;

      // Appropriate length: 50-200 words, 300-1200 characters
      const wordAppropriate = wordCount >= 50 && wordCount <= 200;
      const charAppropriate = charCount >= 300 && charCount <= 1200;

      const passed = wordAppropriate && charAppropriate;
      let score = 0;

      if (passed) {
        score = 1;
      } else if (wordAppropriate || charAppropriate) {
        score = 0.5;
      }

      return {
        passed,
        score,
        message: passed
          ? "Appropriate content length"
          : `Content length issue: ${wordCount} words, ${charCount} characters`,
        suggestions: passed
          ? []
          : [
              "Aim for 50-200 words for optimal readability",
              "Ensure content is comprehensive but concise",
            ],
      };
    },
  },
  {
    id: "no-banned-phrases",
    name: "No Banned Phrases",
    description: "Output does not contain specified banned phrases",
    weight: 0.2,
    evaluate: async (
      output: string,
      context?: { bannedPhrases?: string[] }
    ) => {
      const bannedPhrases = context?.bannedPhrases || [
        "hey team",
        "really casual",
        "let's make it work",
      ];
      const foundBanned = bannedPhrases.filter((phrase) =>
        output.toLowerCase().includes(phrase.toLowerCase())
      );

      const passed = foundBanned.length === 0;
      const score = passed ? 1 : Math.max(0, 1 - foundBanned.length * 0.3);

      return {
        passed,
        score,
        message: passed
          ? "No banned phrases found"
          : `Found banned phrases: ${foundBanned.join(", ")}`,
        suggestions: passed
          ? []
          : ["Replace banned phrases with professional alternatives"],
      };
    },
  },
  {
    id: "required-elements",
    name: "Required Elements",
    description: "Output includes all required elements",
    weight: 0.15,
    evaluate: async (
      output: string,
      context?: { requiredElements?: string[] }
    ) => {
      const requiredElements = context?.requiredElements || [
        "professional",
        "stakeholders",
      ];
      const missingElements = requiredElements.filter(
        (element) => !output.toLowerCase().includes(element.toLowerCase())
      );

      const passed = missingElements.length === 0;
      const score = passed ? 1 : Math.max(0, 1 - missingElements.length * 0.4);

      return {
        passed,
        score,
        message: passed
          ? "All required elements present"
          : `Missing required elements: ${missingElements.join(", ")}`,
        suggestions: passed
          ? []
          : ["Include all specified required elements in the output"],
      };
    },
  },
];

export const CODE_GENERATION_CRITERIA: EvaluationCriteria[] = [
  {
    id: "syntax-valid",
    name: "Syntax Valid",
    description: "Generated code has valid syntax",
    weight: 0.25,
    evaluate: async (output: string) => {
      // Basic syntax check - this would be enhanced with actual language parsing
      const hasBalancedBraces =
        (output.match(/\{/g) || []).length ===
        (output.match(/\}/g) || []).length;
      const hasBalancedParens =
        (output.match(/\(/g) || []).length ===
        (output.match(/\)/g) || []).length;
      const hasBalancedBrackets =
        (output.match(/\[/g) || []).length ===
        (output.match(/\]/g) || []).length;

      const passed =
        hasBalancedBraces && hasBalancedParens && hasBalancedBrackets;
      const score = passed ? 1 : 0.3; // Partial credit for mostly correct syntax

      return {
        passed,
        score,
        message: passed
          ? "Code syntax is valid"
          : "Code has syntax errors (unbalanced braces/parens/brackets)",
        suggestions: passed
          ? []
          : ["Check for missing or extra braces, parentheses, or brackets"],
      };
    },
  },
  {
    id: "lint-clean",
    name: "Lint Clean",
    description: "Code passes linting rules",
    weight: 0.2,
    evaluate: async (output: string) => {
      // Basic lint checks - would be enhanced with actual linter
      const hasSemicolons = output.includes(";");
      const hasConsistentQuotes =
        !output.includes("'") || !output.includes('"') || true; // Simplified
      const noConsoleLog = !output.includes("console.log");

      const passed = hasSemicolons && hasConsistentQuotes && noConsoleLog;
      const score = passed ? 1 : 0.5;

      return {
        passed,
        score,
        message: passed
          ? "Code passes basic linting rules"
          : "Code has linting issues",
        suggestions: passed
          ? []
          : [
              "Add semicolons where needed",
              "Remove console.log statements",
              "Use consistent quote style",
            ],
      };
    },
  },
  {
    id: "functional-correct",
    name: "Functional Correct",
    description: "Code implements the required functionality",
    weight: 0.3,
    evaluate: async (
      output: string,
      context?: { expectedFunctionality?: string[] }
    ) => {
      const expectedFeatures = context?.expectedFunctionality || [
        "component",
        "props",
        "render",
      ];

      let score = 0;
      const foundFeatures: string[] = [];

      for (const feature of expectedFeatures) {
        if (output.toLowerCase().includes(feature.toLowerCase())) {
          score += 1 / expectedFeatures.length;
          foundFeatures.push(feature);
        }
      }

      const passed = score >= 0.8; // 80% of expected features

      return {
        passed,
        score,
        message: passed
          ? `Functionality implemented: ${foundFeatures.join(", ")}`
          : `Missing functionality. Found: ${foundFeatures.join(
              ", "
            )}, Missing: ${expectedFeatures
              .filter((f) => !foundFeatures.includes(f))
              .join(", ")}`,
        suggestions: passed ? [] : ["Implement all required functionality"],
      };
    },
  },
  {
    id: "type-safe",
    name: "Type Safe",
    description: "Code uses proper TypeScript types",
    weight: 0.15,
    evaluate: async (output: string) => {
      const hasTypes =
        output.includes(": ") ||
        output.includes("interface") ||
        output.includes("type");
      const hasAnyType = output.includes(": any");

      const passed = hasTypes && !hasAnyType;
      const score = passed ? 1 : hasTypes ? 0.7 : 0.3;

      return {
        passed,
        score,
        message: passed
          ? "Code uses proper TypeScript types"
          : "Code lacks proper type annotations",
        suggestions: passed
          ? []
          : [
              "Add proper TypeScript type annotations",
              "Avoid 'any' type usage",
            ],
      };
    },
  },
  {
    id: "follows-patterns",
    name: "Follows Patterns",
    description: "Code follows established patterns and conventions",
    weight: 0.1,
    evaluate: async (output: string) => {
      // Basic pattern checks
      const hasProperNaming = /^[A-Z]/.test(output.trim()); // Starts with capital (component)
      const hasExport = output.includes("export");

      const passed = hasProperNaming && hasExport;
      const score = passed ? 1 : 0.5;

      return {
        passed,
        score,
        message: passed
          ? "Code follows established patterns"
          : "Code does not follow expected patterns",
        suggestions: passed
          ? []
          : ["Use proper naming conventions", "Ensure proper exports"],
      };
    },
  },
];

export const DESIGN_TOKEN_CRITERIA: EvaluationCriteria[] = [
  {
    id: "no-hardcoded-colors",
    name: "No Hardcoded Colors",
    description: "No hex colors, rgb(), or color names in output",
    weight: 0.25,
    evaluate: async (output: string) => {
      const colorPatterns = [
        /#[0-9a-fA-F]{3,6}/g, // hex colors
        /rgb\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*\)/g, // rgb()
        /\b(red|blue|green|yellow|black|white|gray|grey)\b/gi, // color names
      ];

      let hasHardcodedColors = false;
      for (const pattern of colorPatterns) {
        if (pattern.test(output)) {
          hasHardcodedColors = true;
          break;
        }
      }

      const passed = !hasHardcodedColors;
      const score = passed ? 1 : 0;

      return {
        passed,
        score,
        message: passed
          ? "No hardcoded colors found"
          : "Found hardcoded colors in output",
        suggestions: passed
          ? []
          : ["Replace hardcoded colors with design token references"],
      };
    },
  },
  {
    id: "no-hardcoded-spacing",
    name: "No Hardcoded Spacing",
    description: "No hardcoded px, rem, em values",
    weight: 0.25,
    evaluate: async (output: string) => {
      const spacingPatterns = [
        /\d+px\b/g, // px values
        /\d+\.\d+rem\b/g, // rem values
        /\d+\.\d+em\b/g, // em values
      ];

      let hasHardcodedSpacing = false;
      for (const pattern of spacingPatterns) {
        if (pattern.test(output)) {
          hasHardcodedSpacing = true;
          break;
        }
      }

      const passed = !hasHardcodedSpacing;
      const score = passed ? 1 : 0;

      return {
        passed,
        score,
        message: passed
          ? "No hardcoded spacing values found"
          : "Found hardcoded spacing values in output",
        suggestions: passed
          ? []
          : ["Replace hardcoded spacing with design token references"],
      };
    },
  },
  {
    id: "uses-design-tokens",
    name: "Uses Design Tokens",
    description: "Output references design tokens correctly",
    weight: 0.3,
    evaluate: async (
      output: string,
      context?: { availableTokens?: Record<string, any> }
    ) => {
      const tokens = context?.availableTokens || {
        colors: { "bg.default": "#ffffff", "text.primary": "#212529" },
        space: { sm: "0.5rem", md: "1rem" },
        radius: { sm: "0.25rem" },
      };

      // Flatten token paths
      const tokenPaths: string[] = [];
      const flattenTokens = (obj: any, prefix = "") => {
        for (const [key, value] of Object.entries(obj)) {
          if (typeof value === "object" && value !== null) {
            flattenTokens(value, prefix ? `${prefix}.${key}` : key);
          } else {
            tokenPaths.push(prefix ? `${prefix}.${key}` : key);
          }
        }
      };
      flattenTokens(tokens);

      // Check if output uses token references
      const usesTokens = tokenPaths.some(
        (tokenPath) =>
          output.includes(`tokens.${tokenPath}`) ||
          output.includes(`theme.${tokenPath}`) ||
          output.includes(tokenPath.replace(".", ""))
      );

      const passed = usesTokens;
      const score = passed ? 1 : 0;

      return {
        passed,
        score,
        message: passed
          ? "Output correctly uses design token references"
          : "Output does not use design token references",
        suggestions: passed
          ? []
          : [
              "Replace hardcoded values with design token references",
              `Use tokens like: ${tokenPaths.slice(0, 3).join(", ")}`,
            ],
      };
    },
  },
  {
    id: "semantic-token-usage",
    name: "Semantic Token Usage",
    description: "Uses appropriate semantic tokens for the context",
    weight: 0.2,
    evaluate: async (output: string) => {
      // Check for semantic usage patterns
      const hasSemanticColors =
        output.includes("bg.") ||
        output.includes("text.") ||
        output.includes("border.");
      const hasSemanticSpacing =
        output.includes("space.") || output.includes("spacing.");

      const passed = hasSemanticColors && hasSemanticSpacing;
      const score = passed
        ? 1
        : hasSemanticColors || hasSemanticSpacing
        ? 0.5
        : 0;

      return {
        passed,
        score,
        message: passed
          ? "Uses semantic tokens appropriately"
          : "Does not use semantic tokens for context",
        suggestions: passed
          ? []
          : [
              "Use semantic color tokens (bg.*, text.*, etc.)",
              "Use semantic spacing tokens (space.*, etc.)",
            ],
      };
    },
  },
];
