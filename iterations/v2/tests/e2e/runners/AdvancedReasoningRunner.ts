/**
 * Advanced Reasoning E2E Test Runner
 *
 * @author @darianrosebrook
 * @description Tests requiring deep reasoning, multi-step problem solving, and iterative refinement
 */

import { ModelBasedJudge } from "@/evaluation/ModelBasedJudge";
import type { ArbiterMCPServer } from "@/mcp-server/ArbiterMCPServer";
import type { ModelRegistry } from "@/models/ModelRegistry";
import type { PerformanceTracker } from "@/rl/PerformanceTracker";
import type {
  CriterionResult,
  EvaluationReport,
  GenerationContext,
  IterativeConfig,
  TestResult,
} from "../types/evaluation";
import {
  combineCriteria,
  createProgrammaticCriterion,
  createRequiredElementsCriterion,
} from "../utils/evaluation-helpers";
import { V2EvaluationRunner } from "./V2EvaluationRunner";

/**
 * Advanced reasoning problem types
 */
export type ReasoningProblemType =
  | "algorithm-design"
  | "code-refactoring"
  | "system-design"
  | "bug-analysis"
  | "performance-optimization";

/**
 * Advanced reasoning specification
 */
export interface AdvancedReasoningSpec {
  problemType: ReasoningProblemType;
  input: {
    description: string; // Problem description
    constraints?: string[]; // Constraints to satisfy
    existingCode?: string; // For refactoring/bug-fixing
    requirements: string[]; // Must satisfy these
    testCases?: Array<{
      input: unknown;
      expectedOutput: unknown;
      description: string;
    }>;
  };
  evaluation: {
    correctness: boolean; // Must work correctly
    efficiency?: boolean; // Should be efficient
    quality?: boolean; // Code quality matters
    completeness?: boolean; // All requirements met
  };
}

/**
 * Solution with reasoning
 */
interface ReasoningSolution {
  code: string;
  reasoning: string[];
  approach: string;
  tradeoffs: string[];
}

/**
 * Advanced Reasoning E2E Test Runner
 *
 * Tests the agent's ability to solve complex problems requiring:
 * - Multi-step reasoning
 * - Algorithm design
 * - Code optimization
 * - Bug analysis
 * - System architecture
 */
export class AdvancedReasoningRunner extends V2EvaluationRunner<
  AdvancedReasoningSpec,
  string
> {
  constructor(
    protected readonly judge: ModelBasedJudge,
    protected readonly mcpServer: ArbiterMCPServer,
    protected readonly performanceTracker: PerformanceTracker,
    protected readonly registry: ModelRegistry,
    iterativeConfig?: Partial<IterativeConfig>
  ) {
    // Advanced reasoning needs more iterations and time
    const advancedConfig: Partial<IterativeConfig> = {
      maxIterations: 5, // Allow more thinking
      passingThreshold: 0.85, // Higher bar
      requireAllCriteriaPassed: false,
      iterationTimeoutMs: 60000, // 1 minute per iteration
      delayBetweenIterationsMs: 1000,
      ...iterativeConfig,
    };

    super(judge, mcpServer, performanceTracker, registry, advancedConfig);
  }

  /**
   * Run an advanced reasoning scenario
   */
  async runScenario(spec: AdvancedReasoningSpec): Promise<TestResult> {
    console.log("\n🧠 Advanced Reasoning E2E Test");
    console.log("================================");
    console.log(`Problem Type: ${spec.problemType}`);
    console.log(
      `Description: "${spec.input.description.substring(0, 100)}..."`
    );
    console.log(`Requirements: ${spec.input.requirements.length} items`);
    if (spec.input.testCases) {
      console.log(`Test Cases: ${spec.input.testCases.length}`);
    }
    console.log("================================");

    return this.iterativeLoop(
      // Generate function
      async (context: GenerationContext) => {
        return this.solveProblem(spec, context);
      },

      // Evaluate function
      async (output: string) => {
        return this.evaluateSolution(output, spec);
      }
    );
  }

  /**
   * Solve the problem with reasoning
   */
  private async solveProblem(
    spec: AdvancedReasoningSpec,
    context: GenerationContext
  ): Promise<string> {
    console.log(
      `\n🤔 Reasoning about ${spec.problemType} (iteration ${context.iteration})...`
    );

    // Build reasoning prompt
    const prompt = this.buildReasoningPrompt(spec, context);

    // For testing, use problem-specific mock solutions
    const solution = this.mockAdvancedReasoning(spec, context);

    console.log(`✅ Generated solution (${solution.split("\n").length} lines)`);
    console.log(`   Approach: ${this.extractApproach(solution)}`);

    return solution;
  }

  /**
   * Mock advanced reasoning for testing
   */
  private mockAdvancedReasoning(
    spec: AdvancedReasoningSpec,
    context: GenerationContext
  ): string {
    const { problemType, input } = spec;

    switch (problemType) {
      case "algorithm-design":
        return this.mockAlgorithmDesign(input.description, context);
      case "code-refactoring":
        return this.mockCodeRefactoring(input.existingCode || "", context);
      case "system-design":
        return this.mockSystemDesign(input.description, context);
      case "bug-analysis":
        return this.mockBugAnalysis(input.existingCode || "", context);
      case "performance-optimization":
        return this.mockPerformanceOptimization(
          input.existingCode || "",
          context
        );
      default:
        return this.mockGenericSolution(input.description);
    }
  }

  /**
   * Mock algorithm design (e.g., LRU Cache)
   */
  private mockAlgorithmDesign(
    description: string,
    context: GenerationContext
  ): string {
    // Check if it's an LRU cache problem
    if (/lru.*cache/i.test(description)) {
      // First iteration: basic implementation
      if (context.iteration === 1) {
        return `/**
 * LRU Cache Implementation
 * 
 * Approach: Use a Map for O(1) access and maintain insertion order
 * 
 * Reasoning:
 * 1. Map maintains insertion order in JavaScript
 * 2. Get operation: Move accessed item to end (most recent)
 * 3. Put operation: Add to end, remove oldest if over capacity
 * 
 * Time Complexity: O(1) for get and put
 * Space Complexity: O(capacity)
 */
class LRUCache {
  private cache: Map<number, number>;
  private capacity: number;

  constructor(capacity: number) {
    this.cache = new Map();
    this.capacity = capacity;
  }

  get(key: number): number {
    if (!this.cache.has(key)) {
      return -1;
    }

    // Move to end (most recent)
    const value = this.cache.get(key)!;
    this.cache.delete(key);
    this.cache.set(key, value);

    return value;
  }

  put(key: number, value: number): void {
    // If key exists, remove it first
    if (this.cache.has(key)) {
      this.cache.delete(key);
    }

    // Add to end
    this.cache.set(key, value);

    // Remove oldest if over capacity
    if (this.cache.size > this.capacity) {
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
  }
}

export default LRUCache;
`;
      }

      // Subsequent iterations: add improvements
      return `/**
 * LRU Cache Implementation (Improved)
 * 
 * Approach: Doubly-linked list + HashMap for true O(1) operations
 * 
 * Reasoning:
 * 1. HashMap for O(1) key lookup
 * 2. Doubly-linked list for O(1) add/remove
 * 3. Most recent at tail, least recent at head
 * 
 * Improvements from previous iteration:
 * - More explicit node management
 * - Better memory efficiency
 * - Clearer separation of concerns
 * 
 * Time Complexity: O(1) for all operations
 * Space Complexity: O(capacity)
 */
class ListNode {
  key: number;
  value: number;
  prev: ListNode | null = null;
  next: ListNode | null = null;

  constructor(key: number, value: number) {
    this.key = key;
    this.value = value;
  }
}

class LRUCache {
  private cache: Map<number, ListNode>;
  private capacity: number;
  private head: ListNode;
  private tail: ListNode;

  constructor(capacity: number) {
    this.cache = new Map();
    this.capacity = capacity;

    // Dummy head and tail nodes
    this.head = new ListNode(-1, -1);
    this.tail = new ListNode(-1, -1);
    this.head.next = this.tail;
    this.tail.prev = this.head;
  }

  private addToTail(node: ListNode): void {
    node.prev = this.tail.prev;
    node.next = this.tail;
    this.tail.prev!.next = node;
    this.tail.prev = node;
  }

  private removeNode(node: ListNode): void {
    node.prev!.next = node.next;
    node.next!.prev = node.prev;
  }

  get(key: number): number {
    if (!this.cache.has(key)) {
      return -1;
    }

    const node = this.cache.get(key)!;
    this.removeNode(node);
    this.addToTail(node);

    return node.value;
  }

  put(key: number, value: number): void {
    if (this.cache.has(key)) {
      const node = this.cache.get(key)!;
      node.value = value;
      this.removeNode(node);
      this.addToTail(node);
      return;
    }

    const newNode = new ListNode(key, value);
    this.cache.set(key, newNode);
    this.addToTail(newNode);

    if (this.cache.size > this.capacity) {
      const lru = this.head.next!;
      this.removeNode(lru);
      this.cache.delete(lru.key);
    }
  }
}

export default LRUCache;
`;
    }

    // Default algorithm
    return `/**
 * Algorithm Implementation
 * 
 * Reasoning: ${description}
 */
export function solve(input: unknown): unknown {
  // Implementation based on requirements
  return null;
}
`;
  }

  /**
   * Mock code refactoring
   */
  private mockCodeRefactoring(
    existingCode: string,
    context: GenerationContext
  ): string {
    if (context.iteration === 1) {
      return `/**
 * Refactored Code
 * 
 * Improvements:
 * 1. Extract complex logic into named functions
 * 2. Add type annotations
 * 3. Remove code duplication
 * 4. Improve variable names
 */
${existingCode.replace(/function\s+(\w+)/g, "function $1Refactored")}

// Refactoring notes:
// - Extracted helper functions
// - Improved readability
// - Added error handling
`;
    }

    return `/**
 * Refactored Code (Further Improved)
 * 
 * Additional Improvements:
 * 1. Applied SOLID principles
 * 2. Added comprehensive error handling
 * 3. Optimized performance
 * 4. Added JSDoc comments
 * 5. Improved testability
 */
${existingCode.replace(/function\s+(\w+)/g, "function optimized$1")}
`;
  }

  /**
   * Mock system design
   */
  private mockSystemDesign(
    description: string,
    context: GenerationContext
  ): string {
    return `/**
 * System Design: ${description}
 * 
 * Architecture Overview:
 * - Component-based design
 * - Clear separation of concerns
 * - Scalable and maintainable
 * 
 * Components:
 * 1. Data Layer: Repository pattern for data access
 * 2. Business Logic: Service layer for core operations
 * 3. API Layer: RESTful endpoints
 * 4. Presentation: React components
 * 
 * Reasoning:
 * - Layered architecture enables independent scaling
 * - Repository pattern abstracts data source
 * - Service layer centralizes business rules
 * - Clear boundaries between layers
 */

// Data Layer
interface Repository<T> {
  find(id: string): Promise<T | null>;
  save(entity: T): Promise<void>;
  delete(id: string): Promise<void>;
}

// Business Logic Layer
interface Service {
  execute(input: unknown): Promise<unknown>;
}

// API Layer
interface APIEndpoint {
  path: string;
  method: "GET" | "POST" | "PUT" | "DELETE";
  handler: (req: Request) => Promise<Response>;
}

// Implementation would follow...
export { Repository, Service, APIEndpoint };
`;
  }

  /**
   * Mock bug analysis
   */
  private mockBugAnalysis(
    existingCode: string,
    context: GenerationContext
  ): string {
    return `/**
 * Bug Analysis and Fix
 * 
 * Root Cause:
 * - Off-by-one error in loop condition
 * - Missing null check
 * - Incorrect variable scope
 * 
 * Fix Applied:
 * 1. Corrected loop boundary
 * 2. Added null safety checks
 * 3. Fixed variable scoping
 */

${existingCode
  .replace(/i <=/g, "i <")
  .replace(/if \(/g, "if (item && ")
  .replace(/var /g, "const ")}

// Testing:
// - Added unit tests for edge cases
// - Verified fix resolves original issue
// - No regressions introduced
`;
  }

  /**
   * Mock performance optimization
   */
  private mockPerformanceOptimization(
    existingCode: string,
    context: GenerationContext
  ): string {
    return `/**
 * Performance Optimization
 * 
 * Original Time Complexity: O(n²)
 * Optimized Time Complexity: O(n)
 * 
 * Optimization Techniques:
 * 1. Replaced nested loops with hash map
 * 2. Memoized expensive computations
 * 3. Used early returns to avoid unnecessary work
 * 4. Batch operations where possible
 * 
 * Benchmarks:
 * - Original: 1000ms for 10k items
 * - Optimized: 50ms for 10k items
 * - 20x performance improvement
 */

${existingCode
  .replace(/for.*for/gs, "// Optimized with single pass using Map")
  .replace(/function/g, "function optimized")}

// Memoization cache
const cache = new Map();

// Performance monitoring
console.time("optimization");
// ... code ...
console.timeEnd("optimization");
`;
  }

  /**
   * Mock generic solution
   */
  private mockGenericSolution(description: string): string {
    return `/**
 * Solution for: ${description}
 * 
 * Approach: Step-by-step problem solving
 */
export function solve(): void {
  // Implementation
}
`;
  }

  /**
   * Extract approach from solution
   */
  private extractApproach(solution: string): string {
    const match = solution.match(/Approach: (.+)/);
    return match ? match[1] : "Multi-step problem solving";
  }

  /**
   * Build reasoning prompt
   */
  private buildReasoningPrompt(
    spec: AdvancedReasoningSpec,
    context: GenerationContext
  ): string {
    let prompt = `Solve this ${spec.problemType} problem:\n\n`;
    prompt += `${spec.input.description}\n\n`;

    if (spec.input.constraints && spec.input.constraints.length > 0) {
      prompt += `Constraints:\n`;
      spec.input.constraints.forEach((c) => (prompt += `- ${c}\n`));
      prompt += `\n`;
    }

    prompt += `Requirements:\n`;
    spec.input.requirements.forEach((r) => (prompt += `- ${r}\n`));

    if (spec.input.existingCode) {
      prompt += `\nExisting Code:\n${spec.input.existingCode}\n`;
    }

    if (context.previousOutput) {
      prompt += `\nPrevious Attempt:\n${context.previousOutput}\n\n`;
      if (context.feedbackHistory.length > 0) {
        prompt += `Feedback:\n${
          context.feedbackHistory[context.feedbackHistory.length - 1]
        }\n`;
      }
    }

    prompt += `\nThink step-by-step and provide your reasoning.`;

    return prompt;
  }

  /**
   * Evaluate solution
   */
  private async evaluateSolution(
    solution: string,
    spec: AdvancedReasoningSpec
  ): Promise<EvaluationReport> {
    const startTime = Date.now();
    console.log("\n📊 Evaluating solution...");

    const criteria: CriterionResult[] = [];

    // 1. Correctness (all requirements met)
    const correctnessResult = await this.checkCorrectness(solution, spec);
    criteria.push(correctnessResult);
    console.log(
      `   ${correctnessResult.passed ? "✅" : "❌"} Correctness: ${(
        correctnessResult.score * 100
      ).toFixed(0)}%`
    );

    // 2. Completeness (all requirements addressed)
    const completenessResult = await this.checkCompleteness(solution, spec);
    criteria.push(completenessResult);
    console.log(
      `   ${completenessResult.passed ? "✅" : "❌"} Completeness: ${(
        completenessResult.score * 100
      ).toFixed(0)}%`
    );

    // 3. Reasoning quality (has explanation)
    const reasoningResult = await this.checkReasoning(solution, spec);
    criteria.push(reasoningResult);
    console.log(
      `   ${reasoningResult.passed ? "✅" : "❌"} Reasoning: ${(
        reasoningResult.score * 100
      ).toFixed(0)}%`
    );

    // 4. Code quality (if applicable)
    if (spec.evaluation.quality) {
      const qualityResult = await this.checkQuality(solution, spec);
      criteria.push(qualityResult);
      console.log(
        `   ${qualityResult.passed ? "✅" : "❌"} Quality: ${(
          qualityResult.score * 100
        ).toFixed(0)}%`
      );
    }

    const overallScore =
      criteria.reduce((sum, c) => sum + c.score, 0) / criteria.length;
    const overallPassed = criteria.every((c) => c.passed);

    console.log(
      `\n   Overall: ${(overallScore * 100).toFixed(1)}% (${
        criteria.filter((c) => c.passed).length
      }/${criteria.length} passed)`
    );

    const executionTime = Date.now() - startTime;

    return {
      criteria,
      overallScore,
      overallPassed,
      executionTime,
      metadata: {
        problemType: spec.problemType,
        requirements: spec.input.requirements,
      },
    };
  }

  /**
   * Check correctness
   */
  private async checkCorrectness(
    solution: string,
    spec: AdvancedReasoningSpec
  ): Promise<CriterionResult> {
    const requiredCriterion = createRequiredElementsCriterion(
      spec.input.requirements,
      false
    );

    return requiredCriterion.evaluate(solution, {});
  }

  /**
   * Check completeness
   */
  private async checkCompleteness(
    solution: string,
    spec: AdvancedReasoningSpec
  ): Promise<CriterionResult> {
    const criterion = createProgrammaticCriterion(
      "completeness",
      "Completeness",
      "Solution addresses all requirements",
      (output: unknown) => {
        if (typeof output !== "string") return false;

        // Check for key solution elements
        const hasImplementation = output.length > 100;
        const hasExports = /export/i.test(output);
        const hasLogic = /function|class|const.*=/i.test(output);

        return hasImplementation && hasExports && hasLogic;
      },
      0.8
    );

    return criterion.evaluate(solution, {});
  }

  /**
   * Check reasoning quality
   */
  private async checkReasoning(
    solution: string,
    spec: AdvancedReasoningSpec
  ): Promise<CriterionResult> {
    const criterion = createProgrammaticCriterion(
      "reasoning",
      "Reasoning Quality",
      "Solution includes clear reasoning and approach",
      (output: unknown) => {
        if (typeof output !== "string") return false;

        // Check for reasoning elements
        const hasReasoning = /Reasoning:|Approach:|Why:|Because:/i.test(output);
        const hasComments = /\/\*\*[\s\S]*?\*\//g.test(output);
        const hasExplanation = /Complexity:|Time:|Space:/i.test(output);

        return hasReasoning || hasComments || hasExplanation;
      },
      0.7
    );

    return criterion.evaluate(solution, {});
  }

  /**
   * Check code quality
   */
  private async checkQuality(
    solution: string,
    spec: AdvancedReasoningSpec
  ): Promise<CriterionResult> {
    const qualityCriteria = [];

    qualityCriteria.push(
      createProgrammaticCriterion(
        "has-types",
        "Type Annotations",
        "Has TypeScript types",
        (output) =>
          typeof output === "string" &&
          /:\s*(string|number|boolean|void|Promise)/i.test(output),
        0.8
      )
    );

    qualityCriteria.push(
      createProgrammaticCriterion(
        "has-error-handling",
        "Error Handling",
        "Includes error handling",
        (output) =>
          typeof output === "string" && /(try|catch|throw|Error)/i.test(output),
        0.6
      )
    );

    const combined = combineCriteria(
      "code-quality",
      "Code Quality",
      qualityCriteria
    );

    return combined.evaluate(solution, {});
  }

  /**
   * Generate domain-specific feedback
   */
  protected generateFeedback(
    report: import("../types/evaluation").EvaluationReport,
    output: string
  ): string {
    const failedCriteria = report.criteria.filter((c) => !c.passed);

    const feedback = failedCriteria
      .map((criterion, index) => {
        const num = index + 1;
        const name = criterion.name;
        const score = (criterion.score * 100).toFixed(1);
        const threshold = (criterion.threshold * 100).toFixed(0);

        let suggestion = "";

        if (criterion.id.includes("correctness")) {
          suggestion = "Ensure all requirements are addressed in the solution";
        } else if (criterion.id.includes("completeness")) {
          suggestion =
            "Add missing implementation details and ensure solution is complete";
        } else if (criterion.id.includes("reasoning")) {
          suggestion =
            "Add clear explanations of your approach, reasoning, and complexity analysis";
        } else if (criterion.id.includes("quality")) {
          suggestion =
            "Improve code quality: add types, error handling, and documentation";
        }

        return `${num}. ${name} (Score: ${score}%, Required: ${threshold}%)\n   Suggestion: ${suggestion}`;
      })
      .join("\n\n");

    return `The solution needs improvement in ${failedCriteria.length} area${
      failedCriteria.length > 1 ? "s" : ""
    }:\n${feedback}\n\nThink deeper about the problem and refine your approach.`;
  }
}
