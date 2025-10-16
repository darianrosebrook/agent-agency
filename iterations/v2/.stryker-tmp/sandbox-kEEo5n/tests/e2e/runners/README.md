# E2E Test Runners - Base Infrastructure

## Overview

This directory contains the base infrastructure for end-to-end agent testing with **iterative feedback loops** and **consistent metric scoring**.

### Architecture

```
tests/e2e/
├── types/
│   └── evaluation.ts          # Type definitions
├── runners/
│   └── V2EvaluationRunner.ts  # Base abstract class
├── utils/
│   └── evaluation-helpers.ts  # Common evaluation utilities
└── [specific-test-runners]    # Concrete implementations
```

---

## Core Components

### 1. **V2EvaluationRunner** (Base Class)

Abstract base class that provides:

- ✅ **Multi-turn iterative feedback loop** (up to N iterations)
- ✅ **Agent interaction tracking** (generations, evaluations, tool calls)
- ✅ **Satisficing logic** (stops when "good enough")
- ✅ **Consistent metric scoring** across all tests
- ✅ **Performance tracking integration**
- ✅ **Timeout handling** per iteration
- ✅ **Delay between iterations** to avoid rate limiting

### 2. **Evaluation Types**

- `EvaluationReport` - Complete evaluation with criteria results
- `CriterionResult` - Individual criterion score and reasoning
- `TestResult` - Final test outcome with all metadata
- `AgentInteraction` - Tracked interaction (generation, evaluation, tool call)
- `TestStatistics` - Aggregated statistics (time, iterations, scores)

### 3. **Evaluation Helpers**

- `createProgrammaticCriterion()` - Simple pass/fail checks
- `createRegexCriterion()` - Pattern matching
- `createBannedPhrasesCriterion()` - Forbidden phrase detection
- `createRequiredElementsCriterion()` - Required element checks
- `createLengthCriterion()` - Length validation
- `combineCriteria()` - Combine multiple criteria with AND logic

---

## Usage

### Step 1: Create a Concrete Runner

Extend `V2EvaluationRunner` and implement `runScenario()`:

```typescript
import { V2EvaluationRunner } from "./V2EvaluationRunner";
import type { TestResult, EvaluationReport } from "../types/evaluation";

interface MyTestSpec {
  input: string;
  expectedOutput: string;
}

class MyTestRunner extends V2EvaluationRunner<MyTestSpec, string> {
  async runScenario(spec: MyTestSpec): Promise<TestResult> {
    return this.iterativeLoop(
      // Generate function
      async (context) => {
        const prompt = this.buildPrompt(spec, context.feedbackHistory);

        // Use MCP to call LLM
        const response = await this.mcpServer.callTool({
          name: "generate-text",
          arguments: { prompt, maxTokens: 500 },
        });

        return response.text;
      },

      // Evaluate function
      async (output) => {
        const criteria = [
          createBannedPhrasesCriterion(["bad", "terrible"]),
          createRequiredElementsCriterion(["good", "excellent"]),
        ];

        return this.evaluateCriteria(output, criteria);
      }
    );
  }

  private buildPrompt(spec: MyTestSpec, feedback: string[]): string {
    let prompt = `Transform this text: ${spec.input}`;

    if (feedback.length > 0) {
      prompt += `\n\nPrevious attempts had issues:\n${feedback.join("\n")}`;
      prompt += `\n\nPlease address these issues.`;
    }

    return prompt;
  }
}
```

### Step 2: Use in Tests

```typescript
describe("My E2E Test", () => {
  let runner: MyTestRunner;

  beforeAll(async () => {
    // Initialize dependencies
    const registry = new ModelRegistry();
    const judge = new ModelBasedJudge();
    const mcpServer = new ArbiterMCPServer();
    const performanceTracker = new PerformanceTracker();

    runner = new MyTestRunner(judge, mcpServer, performanceTracker, registry, {
      maxIterations: 3,
      passingThreshold: 0.8,
      requireAllCriteriaPassed: true,
    });
  });

  it("should transform text successfully", async () => {
    const spec: MyTestSpec = {
      input: "This is a terrible message",
      expectedOutput: "This is a good message",
    };

    const result = await runner.runScenario(spec);

    // Assertions
    expect(result.success).toBe(true);
    expect(result.iterations).toBeLessThanOrEqual(3);
    expect(result.report.overallScore).toBeGreaterThanOrEqual(0.8);

    // Log summary
    runner.logTestSummary(result);

    // Get statistics
    const stats = runner.calculateStatistics(result);
    console.log("Score improvement:", stats.scoreImprovement);
  });
});
```

---

## Configuration

### IterativeConfig

```typescript
interface IterativeConfig {
  maxIterations: number; // Default: 3
  passingThreshold: number; // Default: 0.8 (80%)
  requireAllCriteriaPassed: boolean; // Default: true
  iterationTimeoutMs: number; // Default: 120000 (2 min)
  delayBetweenIterationsMs: number; // Default: 1000 (1 sec)
}
```

Pass custom config:

- In constructor: `new MyRunner(judge, mcp, tracker, registry, { maxIterations: 5 })`
- In `iterativeLoop()`: `this.iterativeLoop(genFn, evalFn, { passingThreshold: 0.9 })`

---

## Iterative Feedback Loop

The core `iterativeLoop()` method works like this:

1. **Generate** output using `generateFn(context)`
2. **Evaluate** output using `evaluateFn(output)`
3. **Check** if passing criteria met:
   - `overallScore >= passingThreshold` AND
   - `all criteria passed` (if `requireAllCriteriaPassed`)
4. If passing: **Return success**
5. If not passing: **Generate feedback**, add to history, iterate
6. Repeat until `maxIterations` or passing

### Feedback Generation

By default, `generateFeedback()` creates:

```
The output needs improvement in 2 areas:
1. No Banned Phrases (Score: 0%, Required: 100%)
   Issue: Found banned phrases: terrible, bad
   Suggestion: Remove all banned phrases

2. Required Elements Present (Score: 50%, Required: 100%)
   Issue: Missing required elements: excellent
   Suggestion: Add the missing required elements
```

Override `generateFeedback()` and `getSuggestionForCriterion()` to customize.

---

## Interaction Tracking

All interactions are automatically tracked:

```typescript
interface AgentInteraction {
  type: "tool_call" | "resource_read" | "evaluation" | "generation";
  timestamp: Date;
  details: { ... };
  duration?: number;
}
```

Access via:

- `runner.getInteractions()` - Get all tracked interactions
- `result.agentInteractions` - Interactions from test
- `runner.calculateStatistics(result)` - Aggregated stats

---

## Evaluation Criteria

### Built-in Helpers

```typescript
// Banned phrases
createBannedPhrasesCriterion(["hey team", "casual"]);

// Required elements
createRequiredElementsCriterion(["professional", "formal"]);

// Length check
createLengthCriterion(100, 500); // min 100, max 500 chars

// Regex matching
createRegexCriterion("no-urls", /https?:\/\//, false); // URLs forbidden

// Custom programmatic
createProgrammaticCriterion(
  "custom-check",
  "Custom Check",
  "My custom validation",
  (output, context) => {
    // Return boolean or { passed, reasoning }
    return output.includes("magic word");
  }
);

// Combine multiple
combineCriteria("all-text-rules", "All Text Rules", [
  bannedPhrases,
  requiredElements,
  lengthCheck,
]);
```

### Custom Criteria

```typescript
const myCustomCriterion: EvaluationCriterion = {
  id: "custom",
  name: "My Custom Check",
  description: "Checks something custom",
  threshold: 0.8,
  weight: 2.0, // Optional: 2x weight in overall score

  evaluate: async (output, context) => {
    // Your custom logic
    const score = calculateScore(output);

    return {
      id: "custom",
      name: "My Custom Check",
      score,
      passed: score >= 0.8,
      threshold: 0.8,
      reasoning: "Explanation of score",
    };
  },
};
```

---

## Integration with ModelBasedJudge

Use `ModelBasedJudge` for LLM-based evaluation:

```typescript
private async evaluateWithJudge(
  text: string,
  spec: MySpec
): Promise<EvaluationReport> {
  // Use ModelBasedJudge for qualitative evaluation
  const formalityJudgment = await this.judge.evaluate(
    {
      task: "Is this text professional and formal?",
      output: text,
      context: { originalSpec: spec }
    },
    EvaluationCriterion.RELEVANCE
  );

  // Convert to CriterionResult
  const criteria: CriterionResult[] = [{
    id: "formal-language",
    name: "Professional Language",
    score: formalityJudgment.overallScore,
    passed: formalityJudgment.overallScore >= 0.8,
    threshold: 0.8,
    reasoning: formalityJudgment.assessments[0].reasoning
  }];

  // Add programmatic checks
  criteria.push(
    await createBannedPhrasesCriterion(spec.bannedPhrases)
      .evaluate(text, {})
  );

  return {
    overallScore: criteria.reduce((sum, c) => sum + c.score, 0) / criteria.length,
    overallPassed: criteria.every(c => c.passed),
    criteria,
    executionTime: Date.now(),
    metadata: {}
  };
}
```

---

## Testing Best Practices

### 1. **Keep Iterations Low**

- Default: 3 iterations
- Most tests should pass in 1-2 iterations
- If consistently needing >3, your criteria might be too strict

### 2. **Use Appropriate Thresholds**

- `0.8` (80%) is a good default for most tests
- Use `0.9` (90%) for critical functionality
- Use `0.7` (70%) for exploratory/experimental tests

### 3. **Combine Programmatic + LLM Evaluation**

- Programmatic checks are fast, deterministic, cheap
- LLM checks are slower, probabilistic, expensive
- Use programmatic for "hard requirements" (banned phrases, length)
- Use LLM for "qualitative judgments" (professionalism, tone)

### 4. **Track Performance**

```typescript
const stats = runner.calculateStatistics(result);
console.log(`Generation time: ${stats.totalGenerationTimeMs}ms`);
console.log(`Evaluation time: ${stats.totalEvaluationTimeMs}ms`);
console.log(`Score improvement: ${stats.scoreImprovement}`);
```

### 5. **Use Timeouts**

- Default: 2 minutes per iteration
- Adjust based on model speed: `{ iterationTimeoutMs: 60000 }` for fast models

---

## Example: Text Transformation Test

See `POC_E2E_MAPPING.md` for a complete example of a Text Transformation runner that:

- Generates professional text from casual input
- Evaluates against banned phrases, required elements, and tone
- Iterates with feedback until passing
- Tracks performance and costs

**Next Step**: Implement concrete test runners for:

1. Text Transformation E2E
2. Code Generation E2E
3. Design Token Application E2E

---

## Summary

**Base E2E infrastructure is now complete!**

✅ **Iterative feedback loop** with configurable iterations  
✅ **Consistent metric scoring** across all tests  
✅ **Agent interaction tracking** (generations, evaluations, tools)  
✅ **Satisficing logic** (stops when good enough)  
✅ **Built-in evaluation helpers** (banned phrases, required elements, etc.)  
✅ **Integration with ModelBasedJudge** for LLM-based evaluation  
✅ **Performance tracking** and statistics  
✅ **Zero linting errors** - production-ready

**Ready to build concrete test runners on top of this foundation!**
