# POC E2E Test Mapping to V2 System

**Date**: October 13, 2025  
**Purpose**: Map POC's self-prompting agent E2E tests to V2 architecture  
**Status**: Design & Planning

---

## Overview

The POC has three main E2E test scenarios focused on **agent self-evaluation and iterative improvement**:

1. **Text Transformation** - Agent rewrites content and self-evaluates completion
2. **Code Generation** - Agent produces code, runs quality gates, fixes issues
3. **Design Token Application** - Agent uses semantic tokens, scans for hardcoded values

**V2 has the infrastructure to implement these tests with:**

- ✅ Real Ollama inference (ARBITER-017)
- ✅ ModelBasedJudge for evaluations (RL-003)
- ✅ Performance tracking (ARBITER-004)
- ✅ MCP server integration
- ✅ Task orchestration

---

## POC Test Architecture

### Key Components from POC:

#### 1. **E2EEvaluationRunner** (`evaluation-runner.ts`)

```typescript
class E2EEvaluationRunner {
  private client: MCPClient;
  private evaluator: E2EEvaluator;
  private interactions: AgentInteraction[];

  async runScenario(scenario: TestScenario): Promise<E2ETestResult> {
    // Multi-turn feedback loop
    while (iteration < maxIterations) {
      const output = await this.executeScenario(scenario, feedback);
      const report = await this.evaluateScenarioOutput(scenario, output);

      if (report.overallPassed) break;

      const feedback = this.generateFeedback(scenario, report, output);
      feedbackHistory.push(feedback);
    }
  }
}
```

**Features:**

- Multi-turn feedback loop (max 3 iterations)
- Tracks agent interactions (tool calls, evaluations)
- Generates feedback for next iteration
- Evaluates output against criteria
- Satisficing logic (stops when "good enough")

#### 2. **E2EEvaluator** (`evaluation-framework.ts`)

```typescript
class E2EEvaluator {
  async evaluate(
    output: any,
    criteria: EvaluationCriterion[]
  ): Promise<EvaluationReport> {
    const results = await Promise.all(
      criteria.map((c) => c.evaluate(output, context))
    );

    return {
      overallScore: average(results.map((r) => r.score)),
      overallPassed: results.every((r) => r.passed),
      criteria: results,
    };
  }
}
```

**Evaluation Criteria Structure:**

```typescript
interface EvaluationCriterion {
  id: string;
  name: string;
  description: string;
  evaluate: (output: any, context: any) => Promise<EvaluationResult>;
  threshold: number; // e.g., 0.7 for passing
}
```

#### 3. **Test Scenarios** (from tests)

**Text Transformation Criteria:**

- `formal-language`: No casual phrases, professional tone
- `no-banned-phrases`: Specific phrases must be removed
- `required-elements`: Must contain specific terms
- `clarity-score`: Readability and structure

**Code Generation Criteria:**

- `compiles`: TypeScript compilation succeeds
- `lints-clean`: Zero linting errors
- `tests-pass`: All unit tests pass
- `type-safe`: No `any` types, proper typing

**Design Token Criteria:**

- `no-hardcoded-colors`: No hex/rgb values
- `semantic-tokens`: Uses design token variables
- `no-hardcoded-spacing`: No raw px values
- `token-consistency`: Consistent token usage

---

## V2 System Capabilities

### What We Have:

#### 1. **ModelRegistryLLMProvider** (NEW - Real LLM)

- ✅ Real Ollama inference
- ✅ Criterion-specific prompting
- ✅ Structured JSON output
- ✅ Performance/cost tracking
- ✅ Provider pooling

**Can Be Used For:**

- Generating agent responses
- Self-evaluation of outputs
- Iterative improvements based on feedback

#### 2. **ModelBasedJudge** (RL-003)

```typescript
class ModelBasedJudge {
  async evaluate(
    input: JudgmentInput,
    criterion: EvaluationCriterion
  ): Promise<JudgmentResult> {
    const llmResponse = await this.llmProvider.evaluate(input, criterion);
    return {
      overallScore: llmResponse.score,
      assessments: [llmResponse],
      evaluationTimeMs: latency,
    };
  }
}
```

**Supports:**

- Faithfulness, Relevance, Minimality, Safety
- Custom LLM providers (including ModelRegistryLLMProvider)
- Confidence scoring
- Reasoning extraction

#### 3. **ArbiterMCPServer**

- ✅ Tool execution
- ✅ Resource management
- ✅ Task routing
- ✅ Session management

#### 4. **Performance Tracking** (ARBITER-004)

- ✅ Metric collection
- ✅ RL training data
- ✅ Task execution tracking

#### 5. **Orchestration Components**

- `IterativeGuidance`: Generates next-step guidance
- `TaskPlanner`: Breaks tasks into steps
- `ConstitutionalRuntimeOrchestrator`: Policy enforcement

---

## Mapping POC Tests to V2

### Test 1: Text Transformation E2E

**POC Implementation:**

```typescript
const scenario = {
  input: {
    text: "Hey team, this is a really casual message...",
    bannedPhrases: ["hey team", "really casual"],
    requiredElements: ["professional", "stakeholders"],
  },
  expectedCriteria: TEXT_TRANSFORMATION_CRITERIA,
  maxIterations: 3,
};

const result = await runner.runScenario(scenario);
```

**V2 Implementation Strategy:**

```typescript
// 1. Create V2EvaluationRunner
class V2EvaluationRunner {
  constructor(
    private judge: ModelBasedJudge,
    private mcpServer: ArbiterMCPServer,
    private performanceTracker: PerformanceTracker
  ) {}

  async runTextTransformation(
    spec: TextTransformationSpec
  ): Promise<TestResult> {
    let iteration = 0;
    const maxIterations = 3;
    const feedbackHistory: string[] = [];

    while (iteration < maxIterations) {
      // Use MCP to call agent for text generation
      const agentResponse = await this.mcpServer.callTool({
        name: "generate-text",
        arguments: {
          prompt: this.buildPrompt(spec, feedbackHistory),
          maxTokens: 500,
        },
      });

      // Evaluate using ModelBasedJudge with custom criteria
      const judgments = await Promise.all([
        this.evaluateFormalLanguage(agentResponse.text, spec),
        this.evaluateBannedPhrases(agentResponse.text, spec),
        this.evaluateRequiredElements(agentResponse.text, spec),
      ]);

      const overallScore =
        judgments.reduce((sum, j) => sum + j.score, 0) / judgments.length;

      // Satisficing: stop if good enough
      if (overallScore >= 0.8 && judgments.every((j) => j.passed)) {
        return {
          success: true,
          output: agentResponse.text,
          iterations: iteration + 1,
        };
      }

      // Generate feedback for next iteration
      const feedback = this.generateFeedback(judgments);
      feedbackHistory.push(feedback);
      iteration++;
    }

    return { success: false, output: null, iterations };
  }

  private async evaluateFormalLanguage(text: string, spec: any) {
    return this.judge.evaluate(
      {
        task: "Evaluate if text is professional and formal",
        output: text,
        expectedOutput: "Professional, stakeholder-appropriate language",
      },
      EvaluationCriterion.RELEVANCE
    );
  }

  private async evaluateBannedPhrases(text: string, spec: any) {
    const hasBannedPhrases = spec.bannedPhrases.some((phrase) =>
      text.toLowerCase().includes(phrase.toLowerCase())
    );

    return {
      score: hasBannedPhrases ? 0 : 1,
      passed: !hasBannedPhrases,
      criterion: "no-banned-phrases",
      reasoning: hasBannedPhrases
        ? `Found banned phrases: ${spec.bannedPhrases
            .filter((p) => text.includes(p))
            .join(", ")}`
        : "No banned phrases found",
    };
  }
}
```

**Required V2 Components:**

- ✅ `ModelRegistryLLMProvider` - For agent text generation
- ✅ `ModelBasedJudge` - For evaluation
- ✅ `ArbiterMCPServer` - For tool execution
- ⚠️ **NEW**: `V2EvaluationRunner` - Multi-turn feedback orchestration
- ⚠️ **NEW**: Custom evaluation criteria adapters

---

### Test 2: Code Generation E2E

**POC Implementation:**

```typescript
const scenario = {
  input: {
    componentSpec: "React button with primary/secondary variants",
    requirements: ["TypeScript", "Styled Components", "Tests"],
  },
  expectedCriteria: CODE_GENERATION_CRITERIA,
  qualityGates: ["compile", "lint", "test", "typecheck"],
};
```

**V2 Implementation Strategy:**

```typescript
class V2CodeGenerationRunner extends V2EvaluationRunner {
  async runCodeGeneration(spec: CodeGenerationSpec): Promise<TestResult> {
    let iteration = 0;
    const maxIterations = 3;
    const feedbackHistory: string[] = [];

    while (iteration < maxIterations) {
      // Generate code using LLM
      const code = await this.generateCode(spec, feedbackHistory);

      // Run quality gates
      const gateResults = await this.runQualityGates(code);

      // Evaluate with ModelBasedJudge
      const judgments = await Promise.all([
        this.evaluateCodeQuality(code, spec),
        this.evaluateTestCoverage(code, gateResults),
        this.evaluateTypeSafety(code, gateResults),
      ]);

      // Check if all gates passed
      if (
        gateResults.every((g) => g.passed) &&
        judgments.every((j) => j.score >= 0.8)
      ) {
        return { success: true, output: code, iterations: iteration + 1 };
      }

      // Generate feedback from failures
      const feedback = this.generateCodeFeedback(gateResults, judgments);
      feedbackHistory.push(feedback);
      iteration++;
    }

    return { success: false, output: null, iterations };
  }

  private async runQualityGates(code: string) {
    // Use MCP to execute quality checks
    const gates = [
      { name: "compile", cmd: "tsc --noEmit" },
      { name: "lint", cmd: "eslint" },
      { name: "test", cmd: "jest" },
      { name: "typecheck", cmd: "tsc --strict" },
    ];

    return Promise.all(
      gates.map(async (gate) => {
        const result = await this.mcpServer.callTool({
          name: "execute-command",
          arguments: { command: gate.cmd },
        });

        return {
          gate: gate.name,
          passed: result.exitCode === 0,
          output: result.stdout,
          errors: result.stderr,
        };
      })
    );
  }
}
```

**Required V2 Components:**

- ✅ `ModelRegistryLLMProvider` - For code generation
- ✅ `ModelBasedJudge` - For code quality evaluation
- ✅ `ArbiterMCPServer` - For running quality gates (lint, test, typecheck)
- ✅ `TerminalSessionManager` - For command execution
- ⚠️ **NEW**: `V2CodeGenerationRunner` - Specialized for code workflows
- ⚠️ **NEW**: Quality gate integration

---

### Test 3: Design Token Application E2E

**POC Implementation:**

```typescript
const scenario = {
  input: {
    componentSpec: "Card component with spacing and colors",
    designTokens: {
      colors: { primary: "var(--color-primary)", ... },
      spacing: { sm: "var(--spacing-sm)", ... }
    }
  },
  expectedCriteria: DESIGN_TOKEN_CRITERIA
};
```

**V2 Implementation Strategy:**

```typescript
class V2DesignTokenRunner extends V2EvaluationRunner {
  async runDesignTokenApplication(spec: DesignTokenSpec): Promise<TestResult> {
    let iteration = 0;
    const maxIterations = 3;
    const feedbackHistory: string[] = [];

    while (iteration < maxIterations) {
      // Generate code with token awareness
      const code = await this.generateCodeWithTokens(spec, feedbackHistory);

      // Scan for hardcoded values
      const violations = this.scanForViolations(code, spec.designTokens);

      // Evaluate with ModelBasedJudge
      const judgments = await Promise.all([
        this.evaluateTokenUsage(code, spec),
        this.evaluateSemanticConsistency(code, spec),
      ]);

      if (violations.length === 0 && judgments.every((j) => j.score >= 0.9)) {
        return { success: true, output: code, iterations: iteration + 1 };
      }

      // Generate feedback about violations
      const feedback = this.generateTokenFeedback(violations, judgments);
      feedbackHistory.push(feedback);
      iteration++;
    }

    return { success: false, output: null, iterations };
  }

  private scanForViolations(code: string, tokens: DesignTokens) {
    const violations: Violation[] = [];

    // Scan for hardcoded colors (hex, rgb)
    const colorPattern = /#[0-9a-f]{3,6}|rgb\(|rgba\(/gi;
    const colorMatches = code.match(colorPattern);
    if (colorMatches) {
      violations.push({
        type: "hardcoded-color",
        matches: colorMatches,
        suggestion: "Use design token variables like var(--color-primary)",
      });
    }

    // Scan for hardcoded spacing (px values)
    const spacingPattern = /\d+px/g;
    const spacingMatches = code.match(spacingPattern);
    if (spacingMatches) {
      violations.push({
        type: "hardcoded-spacing",
        matches: spacingMatches,
        suggestion: "Use design token variables like var(--spacing-md)",
      });
    }

    return violations;
  }
}
```

**Required V2 Components:**

- ✅ `ModelRegistryLLMProvider` - For code generation with token awareness
- ✅ `ModelBasedJudge` - For semantic consistency evaluation
- ⚠️ **NEW**: `V2DesignTokenRunner` - Token-specific workflows
- ⚠️ **NEW**: Token violation scanner (regex-based)

---

## Implementation Plan for V2

### Phase 1: Core E2E Infrastructure (Week 1)

#### 1.1 Create Base E2E Runner

**File**: `tests/e2e/runners/V2EvaluationRunner.ts`

```typescript
export abstract class V2EvaluationRunner {
  constructor(
    protected judge: ModelBasedJudge,
    protected mcpServer: ArbiterMCPServer,
    protected performanceTracker: PerformanceTracker,
    protected registry: ModelRegistry
  ) {}

  abstract runScenario(spec: any): Promise<TestResult>;

  protected async iterativeLoop(
    maxIterations: number,
    generateFn: (feedback: string[]) => Promise<string>,
    evaluateFn: (output: string) => Promise<EvaluationReport>,
    passingThreshold: number = 0.8
  ): Promise<TestResult> {
    // Generic iterative feedback loop
  }

  protected generateFeedback(report: EvaluationReport, output: string): string {
    // Extract actionable feedback from failed criteria
  }
}
```

#### 1.2 Create Evaluation Report Types

**File**: `tests/e2e/types/evaluation.ts`

```typescript
export interface EvaluationReport {
  overallScore: number;
  overallPassed: boolean;
  criteria: CriterionResult[];
  executionTime: number;
  metadata: Record<string, any>;
}

export interface CriterionResult {
  id: string;
  name: string;
  score: number;
  passed: boolean;
  threshold: number;
  reasoning: string;
}

export interface TestResult {
  success: boolean;
  output: any;
  iterations: number;
  feedbackHistory: string[];
  report: EvaluationReport;
  agentInteractions: AgentInteraction[];
}
```

### Phase 2: Implement Text Transformation Test (Week 1-2)

#### 2.1 Create Text Transformation Runner

**File**: `tests/e2e/runners/TextTransformationRunner.ts`

```typescript
export class TextTransformationRunner extends V2EvaluationRunner {
  async runScenario(spec: TextTransformationSpec): Promise<TestResult> {
    return this.iterativeLoop(
      3, // max iterations
      (feedback) => this.generateText(spec, feedback),
      (output) => this.evaluateText(output, spec),
      0.8 // passing threshold
    );
  }

  private async generateText(
    spec: TextTransformationSpec,
    feedback: string[]
  ): Promise<string> {
    const prompt = this.buildTextPrompt(spec, feedback);

    // Use ModelRegistryLLMProvider through MCP
    const response = await this.mcpServer.callTool({
      name: "model-inference",
      arguments: {
        modelId: "gemma:2b",
        prompt,
        maxTokens: 500,
        temperature: 0.7,
      },
    });

    return response.text;
  }

  private async evaluateText(
    text: string,
    spec: TextTransformationSpec
  ): Promise<EvaluationReport> {
    const criteria: CriterionResult[] = [];

    // Evaluate formal language (using ModelBasedJudge)
    const formalityJudgment = await this.judge.evaluate(
      {
        task: "Is this text professional and formal?",
        output: text,
        context: { originalSpec: spec.input.text },
      },
      EvaluationCriterion.RELEVANCE
    );
    criteria.push({
      id: "formal-language",
      name: "Professional Language",
      score: formalityJudgment.overallScore,
      passed: formalityJudgment.overallScore >= 0.8,
      threshold: 0.8,
      reasoning: formalityJudgment.assessments[0].reasoning,
    });

    // Check banned phrases (programmatic)
    const bannedPhrasesFound = spec.input.bannedPhrases.filter((phrase) =>
      text.toLowerCase().includes(phrase.toLowerCase())
    );
    criteria.push({
      id: "no-banned-phrases",
      name: "No Banned Phrases",
      score: bannedPhrasesFound.length === 0 ? 1 : 0,
      passed: bannedPhrasesFound.length === 0,
      threshold: 1.0,
      reasoning:
        bannedPhrasesFound.length > 0
          ? `Found banned phrases: ${bannedPhrasesFound.join(", ")}`
          : "No banned phrases detected",
    });

    // Check required elements (programmatic)
    const missingElements = spec.input.requiredElements.filter(
      (element) => !text.toLowerCase().includes(element.toLowerCase())
    );
    criteria.push({
      id: "required-elements",
      name: "Required Elements Present",
      score:
        missingElements.length === 0
          ? 1
          : (spec.input.requiredElements.length - missingElements.length) /
            spec.input.requiredElements.length,
      passed: missingElements.length === 0,
      threshold: 1.0,
      reasoning:
        missingElements.length > 0
          ? `Missing required elements: ${missingElements.join(", ")}`
          : "All required elements present",
    });

    const overallScore =
      criteria.reduce((sum, c) => sum + c.score, 0) / criteria.length;

    return {
      overallScore,
      overallPassed: criteria.every((c) => c.passed),
      criteria,
      executionTime: Date.now(),
      metadata: {},
    };
  }
}
```

#### 2.2 Create Test File

**File**: `tests/e2e/text-transformation.test.ts`

```typescript
describe("Text Transformation E2E - V2", () => {
  let runner: TextTransformationRunner;

  beforeAll(async () => {
    const registry = new ModelRegistry();
    await registry.registerOllamaModel("gemma-2b", "gemma:2b", "1.0.0");

    const selector = new LocalModelSelector(registry);
    const costTracker = new ComputeCostTracker();

    const llmProvider = new ModelRegistryLLMProvider(
      {
        provider: "model-registry",
        model: "gemma-2b",
        taskType: "text-generation",
        qualityThreshold: 0.7,
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        temperature: 0.7,
        maxTokens: 500,
      },
      registry,
      selector,
      costTracker
    );

    const judge = new ModelBasedJudge({}, llmProvider);
    const mcpServer = new ArbiterMCPServer({});
    const performanceTracker = new PerformanceTracker();

    runner = new TextTransformationRunner(
      judge,
      mcpServer,
      performanceTracker,
      registry
    );
  });

  it("should transform casual text to professional language", async () => {
    const spec: TextTransformationSpec = {
      input: {
        text: "Hey team, this is a really casual message that needs to be made more professional.",
        bannedPhrases: ["hey team", "really casual"],
        requiredElements: ["professional", "team"],
      },
    };

    const result = await runner.runScenario(spec);

    expect(result.success).toBe(true);
    expect(result.iterations).toBeLessThanOrEqual(3);
    expect(result.report.overallScore).toBeGreaterThanOrEqual(0.8);
    expect(result.report.overallPassed).toBe(true);

    // Verify no banned phrases
    const bannedPhrasesResult = result.report.criteria.find(
      (c) => c.id === "no-banned-phrases"
    );
    expect(bannedPhrasesResult?.passed).toBe(true);

    // Verify required elements
    const requiredElementsResult = result.report.criteria.find(
      (c) => c.id === "required-elements"
    );
    expect(requiredElementsResult?.passed).toBe(true);
  });
});
```

### Phase 3: Implement Code Generation & Design Token Tests (Week 2-3)

Follow similar pattern for:

- `CodeGenerationRunner.ts`
- `DesignTokenRunner.ts`
- `code-generation.test.ts`
- `design-token-application.test.ts`

---

## Benefits of V2 Implementation

### 1. Real LLM Integration ✅

- **POC**: Used mock responses
- **V2**: Uses real Ollama models via `ModelRegistryLLMProvider`
- **Impact**: Validates actual agent capabilities, not simulated behavior

### 2. Production-Grade Components ✅

- **POC**: E2E framework was test-only
- **V2**: Uses production `ModelBasedJudge`, `ArbiterMCPServer`, performance tracking
- **Impact**: E2E tests validate actual system components

### 3. Hardware Optimization ✅

- **POC**: No hardware awareness
- **V2**: Can leverage Apple Silicon, GPU acceleration via `LocalModelSelector`
- **Impact**: Tests reflect real deployment performance

### 4. Cost & Performance Tracking ✅

- **POC**: No cost tracking
- **V2**: `ComputeCostTracker` monitors resource usage
- **Impact**: E2E tests provide performance benchmarks

### 5. Learning Preservation ✅

- **POC**: No learning between iterations
- **V2**: `PerformanceTrackerBridge` + `ModelRegistry` track model performance
- **Impact**: Tests can validate learning improvements

---

## Next Steps

### Immediate (This Week):

1. ✅ Map POC tests to V2 components (this document)
2. ⚠️ Create `V2EvaluationRunner` base class
3. ⚠️ Implement `TextTransformationRunner`
4. ⚠️ Write first E2E test with real Ollama

### Short Term (Next Week):

1. ⚠️ Implement `CodeGenerationRunner`
2. ⚠️ Add quality gate integration (lint, test, typecheck)
3. ⚠️ Create `DesignTokenRunner`
4. ⚠️ Add token violation scanner

### Medium Term (Next 2 Weeks):

1. ⚠️ Add multi-agent scenarios (debate/critique)
2. ⚠️ Performance benchmarking suite
3. ⚠️ CI/CD integration
4. ⚠️ Comprehensive test fixtures

---

## Success Criteria

### Test Coverage:

- ✅ Text transformation E2E
- ✅ Code generation E2E
- ✅ Design token application E2E
- ✅ 95%+ test pass rate in CI
- ✅ Tests complete within 2 minutes each

### System Validation:

- ✅ Real Ollama models respond reliably
- ✅ ModelBasedJudge evaluations are accurate
- ✅ Multi-turn feedback improves outputs
- ✅ Satisficing logic prevents over-optimization
- ✅ Performance/cost metrics are tracked

### Quality Gates:

- ✅ All quality gates execute correctly
- ✅ Code generation produces lint-clean code
- ✅ Design tokens are used correctly
- ✅ Agent self-evaluates completion

---

## Conclusion

**We have all the V2 components needed to implement POC-style E2E tests with real LLMs.**

The POC provides excellent test scenarios and patterns. V2 adds:

- Real model inference (not mocked)
- Production-grade evaluation (ModelBasedJudge)
- Cost/performance tracking
- Hardware optimization
- Learning preservation

**Next: Implement `TextTransformationRunner` as the first concrete E2E test with real Ollama integration.**
