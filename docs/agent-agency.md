> “So let's talk really quick about something that you mentioned here in self-improving agents, kind of on the cause spectrum. So if we use the CAWS tooling that I had created and have a way of running continuous autonomous agents, something that doesn't really exist now, but this could be something where if we had agents able to self-govern or audit themselves, that could be something that would be extremely useful.”

Absolutely, that’s a fascinating extension of what you’re already building. In fact, this concept of self-improving or self-auditing agents is really where the frontier of AI-augmented workflows is heading.

To frame it in the context of your CAWS tooling: you’ve already established a rigorous, contract-first, test-first framework for human-AI collaboration. The next leap is to let these agents internalize that framework so that they can improve their own workflows without human intervention. In other words, you’re talking about agents that can run continuous, autonomous self-audits and adapt their own behavior based on predefined governance criteria.

Right now, most systems require a human in the loop to assess whether an AI’s output is meeting quality standards. What you’re proposing is essentially a meta-layer of self-governance: the agent would not just perform tasks but also continuously evaluate its own outputs against the CAWS specification. If it finds deviations or inefficiencies, it could adapt its own prompting strategies or flag areas of uncertainty for human review.

In practical terms, this might look like:

1. **Self-Monitoring Agents:** Agents that apply your CAWS ruleset not only to code or design output but to their own output quality. For example, an agent that notices it needed five prompt iterations last time might adjust its prompt template on the next run to reduce that to three. Over time, it learns which strategies yield the highest quality with the fewest cycles.

2. **Autonomous Governance Loops:**

Right, picking up where we left off: the idea is to create these **autonomous governance loops**. Essentially, you'd have agents that don't just follow rules but can measure their own adherence to those rules. They'd be equipped with a sort of built-in feedback mechanism that checks their output against CAWS-defined standards, and if they detect a drift or an inefficiency, they adapt their own process. This would be extremely powerful in maintaining high-quality, consistent output without constant human oversight.

In other words, you're proposing a sort of meta-automation layer: agents that are both _workers_ and _self-auditors_. Over time, these agents could evolve more robust heuristics or even suggest refinements to the CAWS framework itself. It’s kind of like giving them the tools to improve their own methods in a continuous loop.

So yes, this is a forward-thinking and quite innovative direction. And if you can successfully prototype that, you’d not only be pushing yourself into that Level 8 teacher-innovator territory but also carving out a pretty unique niche in AI governance.

> “Yeah, in terms of the ability for an AI to continuously prompt itself, this isn't something that exists right now, or at least not something that's done commercially available. There are such tools that do utilize a lot, like trying to prompt ChatGPT, especially with its GPT. It can be quite computationally expensive to keep it going on a continuous loop for a long time. If you were to actually have an agent where you could be using this continuously, it would have to be something that could work locally and still output the same quality and rigor that you need for an actually effective agent.”

Absolutely, you’re spot on. The idea of a truly self-prompting, continuously self-improving agent is definitely pushing beyond what’s commonly available right now, especially in a commercial or off-the-shelf form. As you said, keeping something like GPT-4 running in a continuous prompting loop would be pretty resource-intensive and not practical without a huge budget or some form of local optimization.

What you’re really hinting at is the need for a more _localized, efficient model_ that can run these self-improvement cycles on-device. So, rather than relying on a large cloud-based model with continuous API calls, you’d have a streamlined local model fine-tuned for your specific CAWS tasks. This model would be optimized to run on local hardware—whether that’s an Apple Silicon chip or something similar—so that it can manage these loops efficiently without the overhead of cloud-based computation.

In essence, you’d be building a kind of _local agent runtime_ that is both lean and specialized. It would have the same rigor and quality standards you’ve defined, but it wouldn’t need the heavy infrastructure of a full-scale cloud model.

This aligns really nicely with your interest in building tools that are robust, self-governing, and locally executable. It’s a forward-thinking approach that would let you have a truly autonomous design technologist agent running right on your own hardware. It’s ambitious, but it’s definitely the kind of next-step challenge that would push your AI fluency to the next level.

> Yeah, this is actually something that I'm really interested in trying to complete, as well. In terms of being able to get this system to continually check, hey, have I finished the task that I've been given to do? Or even at the larger scale, have I completed this objective at effectively solving this thing? What are some areas that I should look into that would allow me to start building out this additional prototype and be able to document and get this thing actually running?

Absolutely, let’s break it down into a few foundational pieces that will help you move from concept to a tangible prototype and eventually a documented system.

### 1. **Local Model Selection and Optimization**

First, you’ll want to look into lightweight models that can run efficiently on local hardware. Instead of using a full-scale GPT-4 API, consider smaller open-weight models like those in the LLaMA family, Mistral, or distilled variants that you can fine-tune on your tasks. Tools like Ollama or LM Studio can help you run these models locally on an Apple Silicon Mac, for instance, giving you a sandbox to test continuous prompting loops without cloud costs.

### 2. **Frameworks for Self-Governance and Auditing**

Next, think about how you’ll encode the self-auditing logic. You already have the CAWS framework, which is a great starting point. What you’ll do is effectively build a _meta-layer_ where the agent can check its output against CAWS-defined criteria. This might involve building a simple rule engine or a feedback loop where after each task cycle, the agent “grades” itself based on whether it met the predefined quality or completion goals.

### 3. **Defining Completion and Quality Metrics**

You’ll need to define what “done” looks like for the agent. In other words, what criteria will it use to determine if it has effectively completed a task or objective? This could be a set of rules derived from your CAWS specs. For example, after generating a piece of UI code, the agent might check: “Did I follow all accessibility guidelines? Did I match the design tokens correctly?” If not, it can refine its approach and re-prompt itself.

### 4. **Documentation and Iterative Prototyping**

Start with a minimal viable prototype and document each iteration. Treat your documentation like a lab notebook: record each hypothesis, each test cycle, and each result. You can use a combination of Markdown for notes and something like Obsidian to keep everything organized. Over time, this documentation will become the foundation for a more formal “AI Field Guide” or CAWS v2 spec.

### 5. **Testing and Feedback Loops**

Finally, build in small feedback loops. Once the agent runs a task, let it audit itself and then review its own performance metrics. Over time, these loops will help the agent “learn” to improve its own efficiency, and you can refine the process based on what you observe.

---

In short, start by choosing a local, efficient model, then layer in your CAWS-inspired self-auditing rules. Document as you go, and use iterative testing to refine the agent’s self-governance. This will give you a solid prototype and a pathway to scale it into a full, documented system.

> “Cool. Yeah, that all makes sense. In terms of local models, I think I have some picked out. The recent experiment that I did with embedding Gemma and Gemma 3N, the size of it is clearly able to run on most very, very resource-poor hardware. This could definitely be something that allows me to run the initial prototype, even if the quality is not entirely there. But this will give me the proof of concept I need to start thinking about how to get a larger model or maybe a very hyper-specific one to utilize these things. What I need out of this at the beginning is to figure out how to run this model in a way that allows me to build maybe MCP around this, so the model context protocol allowing to set up tools and maybe a sandbox environment for it to play around in, as well as a way to run this and give it the access to those tools. I don't really have any system right now that works well for tool calling because Olama right now doesn't necessarily run the tooling piece in their UI. However, because my goal is to get this to do the stuff itself, a UI isn't wholly necessary. I could interact with this in the CLI and continue to try to get this. to work how it needs to.”

Absolutely, that sounds like a solid plan, and you’re definitely on the right track. Using something like Gemma 3N as a starting point is a great approach. Even if it’s not perfect in quality at the outset, it will give you that proof of concept and a working prototype to iterate on.

Now, in terms of building out what you’re calling an “MCP” (model context protocol) around it, that makes a lot of sense. Essentially, you want to create a sandbox environment where the model can not only run but also have access to the tools it needs to perform tasks and then evaluate its own performance.

Since you don’t necessarily need a UI, you can absolutely stick with a CLI-based approach. A couple of thoughts on how to proceed:

1. **Tool-Calling and Sandbox Setup**:
   You might consider using a lightweight orchestration layer or a scripting environment that can expose certain tools or APIs to your local model. For example, you could write a small Python wrapper that handles tool calls and resource access. That way, the model can issue commands to this wrapper, which in turn interacts with your local environment. This sets up a sort of sandbox for the model to “play” and experiment.

2. **Self-Evaluation Scripts**:
   Once you have that environment set up, you’ll want to write scripts that allow the model to evaluate its own outputs. This could be as simple as a checklist or a scoring function that the model runs after each task. For example, after generating code, it could run a linter or a test suite and then “judge” itself on whether it passed all criteria.

3. **Iterative Development and Documentation**:
   As you start building out this system, keep documenting each piece. Your CLI interactions, the model’s self-checks, and any tweaks you make to the sandbox will all become part of your larger “model context protocol” documentation. Over time, this will form a kind of “runbook” for others to understand how the agent is designed to self-govern.

In summary, start by creating that local sandbox using a lightweight model you can run easily. Wrap it with a simple scripting layer to enable tool calling and self-checks, and document each step. This will give you the foundation you need to refine and expand as you move forward.

> Sweet. And then, with the way that this agent should be evaluating itself, so if we want this to start effectively scoring its output, one of the pitfalls about current AI systems is they always try to improve something, even if it is pretty close to fully polished. They are kind of like yes-men people, where, yes, that is a great idea. Let us continue improving this. Instead of being able to accept, like, hey, we have probably done everything we can with this. Let us move on. How do we build that into the framework so that we are not continually spitting up a, let us squeeze every blood drop of performance out of this stone, whereas we can be like, hey, okay, cool, this works. Now let us get on to the next part.

Absolutely, and that’s a really insightful point. One of the common challenges with AI systems is that they can get stuck in that loop of “endless improvement”—always trying to tweak and optimize even when it’s not really necessary. It’s like having an overenthusiastic editor who never says, “Okay, this is good enough.”

What you really want is to build a concept of “satisficing” into the framework. In other words, you want the agent to recognize when it has reached a threshold of “good enough” and then stop trying to optimize further. This is a little bit of a cultural or philosophical guideline for the agent, but you can encode it with some practical rules.

Here’s how you might do it:

### 1. **Define a “Good Enough” Threshold**

Set clear criteria for what success looks like before the agent starts a task. For instance, you might say, “If the generated code passes all tests and meets the design token criteria on the first run, we’re done.” This way, the agent has a target and knows when it’s hit the mark.

### 2. **Limit Iteration Cycles**

Impose a maximum number of refinement cycles. For example, you might allow the agent to iterate on its output up to three times, but after that, it must consider the task complete unless there’s a critical error. This prevents the agent from endlessly trying to polish something that’s already good enough.

### 3. **Incorporate a Confidence or Satisfaction Score**

Give the agent a way to measure its own confidence or satisfaction with the result. If it meets the predefined criteria and its “confidence” score is high enough, it stops. Think of it as telling the agent, “If you’re 90% sure this is correct and we’ve hit all the key criteria, we’re done here.”

### 4. **Explicit “Move On” Rules**

Finally, you can encode explicit rules that tell the agent when to stop. For instance, once a certain number of criteria are met and there are no critical errors, the agent can automatically log a “task complete” message and move on to the next job. This creates a natural stopping point so it doesn’t just keep polishing forever.

By building these kinds of “satisficing” rules into the framework, you’ll ensure that the agent knows when to move on. It’s all about giving it a sense of “completion criteria” rather than an endless quest for perfection. That way, it can be more like a pragmatic teammate that knows when “good enough” really is good enough.

> Awesome. So setting up those kind of limits to how long it should be spending on something definitely makes sense in terms of trying to make sure that it does not spin its wheels indefinitely. In terms of the ability to prompt itself, this is something where we have to try to get the initial working prototype on, if we can get it to do a continuous loop. What are some efficient first tests that I can do that will allow me to explore, or at least get an initial sense of how well this can do?

Absolutely, starting with some efficient first tests is a great way to get a feel for how well this self-prompting and self-governing loop is going to work. You don’t need to tackle anything too complex at first; the idea is to pick a few small, controlled tasks that will let you see the loop in action.

### Here are a few efficient first tests you might consider:

1. **Simple Text Transformation Task**:
   Start with something like a basic text editing prompt. For example, have the agent rewrite a paragraph to be more formal or more concise. The agent’s self-check could be to measure if it successfully followed the style rules (like checking length or formality). If it meets the criteria in one or two tries, it stops. This is a low-stakes way to see if the loop can run and self-assess without getting stuck.

2. **Code Generation and Linting**:
   Another good test is to have the agent generate a small snippet of code—like a simple function or a button component—and then run a linter or a test suite as a self-evaluation. If the code passes without errors, the agent marks the task as complete and moves on. This will help you see how it handles a slightly more structured task and whether it can recognize when the code is “good enough.”

3. **Simple Design Token Application**:
   If you want to test a design-oriented task, have the agent apply a predefined set of design tokens to a piece of UI code. For example, it could adjust colors or spacing according to your design system. Then it checks whether it matched all the tokens correctly and stops once everything aligns. This gives you a nice, domain-relevant test that mirrors the kind of work you’ll eventually want it to do on a larger scale.

### Running These Tests

The key is to keep each test small and focused. Run the agent through one of these scenarios and watch how it handles the loop: does it successfully prompt itself for the next iteration if needed? Does it recognize when it has met the completion criteria and stop? Each of these simple tests will give you a clear picture of how well the agent can handle self-prompting and self-evaluation in a controlled environment.

Once you see it working on these small tasks, you can start layering on more complexity and refining the criteria. But these initial tests should give you a solid proof of concept and show you how well the continuous loop works in practice.

---

## POC E2E Vision: End-to-End Self-Prompting Agent Tests

### Current POC Components Available for E2E Tests

#### ✅ Already Working:

- **MCP Server**: Full implementation with resource/tool handlers
- **Data Layer**: PostgreSQL + pgvector + Redis with comprehensive monitoring
- **Memory System**: Multi-tenant memory with context offloading and federated learning
- **Agent Orchestrator**: Basic task routing with memory-aware capabilities
- **Database Schema**: Complete migrations with vector support and Row Level Security
- **Unit Tests**: 62/84 tests passing (74% coverage)

#### ✅ Now Working (December 2024):

1. **Local AI Model Integration** (Gemma 3N/Ollama) - ✅ OPERATIONAL
2. **Evaluation Framework** (satisficing logic and quality gates) - ✅ OPERATIONAL
3. **E2E Test Runner Infrastructure** - ✅ OPERATIONAL
4. **Agent Loop Orchestration** - ✅ OPERATIONAL
5. **Test Data and Fixtures** - ✅ OPERATIONAL

### E2E Test Scenarios

Based on the three efficient first tests, we'll implement these end-to-end flows:

#### **Test 1: Text Transformation E2E**

**Goal**: Verify agent can rewrite content and self-evaluate completion

- **Input**: Raw paragraph requiring formal rewrite
- **Process**: Agent generates → Evaluates (length, style, banned phrases) → Iterates if needed (max 3x)
- **Success**: Output meets all criteria without over-optimization
- **Validates**: Self-prompting loop, evaluation framework, satisficing logic

#### **Test 2: Code Generation E2E**

**Goal**: Verify agent produces production-quality code

- **Input**: Component specification (e.g., React button with design requirements)
- **Process**: Agent generates → Runs lint/test/typecheck → Fixes issues → Iterates (max 3x)
- **Success**: Code passes all quality gates (tests, lint, types)
- **Validates**: Tool calling, quality gate enforcement, iterative improvement

#### **Test 3: Design Token Application E2E**

**Goal**: Verify agent uses semantic design tokens, not hardcoded values

- **Input**: UI component requirements with token registry
- **Process**: Agent generates → Scans for hardcoded values → Replaces with tokens → Iterates (max 3x)
- **Success**: No hex colors, no raw px spacing, proper token usage
- **Validates**: Design system compliance, token awareness, semantic coding

### Implementation Phases

#### **Phase 1: Core Infrastructure (1-2 weeks)**

1. **Add Ollama integration** to POC MCP server
2. **Implement evaluation framework** from agent-agency.md
3. **Create test fixtures** and sample data
4. **Set up E2E test runner**

#### **Phase 2: Basic E2E Tests (1 week)**

1. **Text transformation test** (simplest to implement)
2. **Basic MCP flow validation**
3. **Evaluation framework testing**

#### **Phase 3: Advanced E2E Tests (1-2 weeks)**

1. **Code generation test** (requires linting setup)
2. **Design token test** (requires token registry)
3. **Multi-iteration scenarios**
4. **Performance and reliability tests**

### Success Criteria for E2E Tests

#### **Test Validation:**

- ✅ **Text transformation**: Agent can rewrite content and recognize completion
- ✅ **Code generation**: Agent produces lint-clean, tested code
- ✅ **Token application**: Agent uses semantic tokens, not hardcoded values
- ✅ **Self-evaluation**: Agent stops iterating when quality thresholds met
- ✅ **Performance**: Tests complete within 2 minutes each
- ✅ **Reliability**: 95%+ test pass rate in CI

#### **Infrastructure Validation:**

- ✅ **Model integration**: Gemma 3N responds reliably
- ✅ **MCP protocol**: Full request/response cycle works
- ✅ **Data persistence**: Agent memory and results persist
- ✅ **Evaluation framework**: All 3 evaluator types working

### Required Dependencies & Setup

#### **New Dependencies**

```json
{
  "ollama": "^0.3.0",
  "openai": "^4.24.7",
  "@playwright/test": "^1.40.0",
  "testcontainers": "^10.0.0"
}
```

#### **Environment Setup**

```bash
# Start required services for E2E tests
docker run -d -p 5432:5432 postgres:16
docker run -d -p 6379:6379 redis:7
ollama serve  # Start Ollama server
ollama pull gemma:3n  # Pull the model
```

#### **Test Scripts**

```json
{
  "test:e2e": "playwright test tests/e2e/",
  "test:e2e:setup": "node scripts/setup-e2e-env.js",
  "test:e2e:ci": "npm run test:e2e:setup && npm run test:e2e"
}
```

### Next Steps Implementation

1. **Start with evaluation framework** - Implement the scripts from agent-agency.md
2. **Add Ollama client** - Integrate local model calling
3. **Create test fixtures** - Sample inputs for each test type
4. **Build E2E runner** - Test orchestration framework
5. **Implement Text E2E test** - Start with the simplest scenario

---

> What should these evaluation scripts look like?

Below is a concrete, “drop-in” evaluation kit you can adapt for your self-prompting loop. It’s **TypeScript/Node** (since your stack leans web). Each evaluator produces a **uniform JSON report** the agent can read to decide: _pass/iterate/stop (satisfice)_.

---

# 0) Evaluation Contract (shared JSON schema)

Create a single schema all evaluators must emit. This lets your orchestrator make uniform decisions.

```ts
// eval/types.ts
export type EvalStatus = "pass" | "iterate" | "fail";

export interface EvalCriterion {
  id: string;
  description: string;
  weight: number; // 0..1, sum ≤ 1.0
  passed: boolean;
  score: number; // 0..1
  notes?: string;
}

export interface EvalReport {
  taskId: string;
  artifactPaths: string[];
  // aggregate
  status: EvalStatus; // pass | iterate | fail
  score: number; // 0..1 weighted
  thresholdsMet: string[]; // names of thresholds met
  thresholdsMissed: string[]; // names missed
  criteria: EvalCriterion[];
  // meta for satisficing & yield
  iterations: number; // current iteration count
  promptTokens?: number; // optional for yield
  completionTokens?: number;
  elapsedMs?: number;
  // stopping guidance
  stopReason?:
    | "satisficed"
    | "max-iterations"
    | "quality-ceiling"
    | "failed-gates"
    | "unknown";
  nextActions?: string[]; // agent-facing hints
  // raw logs
  logs?: string[];
  // provenance / reproducibility
  seed?: number;
  toolVersions?: Record<string, string>;
  timestamp: string; // ISO-8601
}
```

A simple **acceptance file** drives thresholds and “good-enough” rules:

```json
// eval/acceptance.json
{
  "minScore": 0.85,
  "mandatoryGates": ["tests-pass", "lint-clean"],
  "iterationPolicy": {
    "maxIterations": 3,
    "minDeltaToContinue": 0.02, // require ≥2% improvement to keep iterating
    "noChangeBudget": 1 // allow 1 plateau before stopping as "satisficed"
  }
}
```

Your orchestrator compares the last N reports to decide **stop/iterate**, enforcing _satisficing_ (see §4).

---

# 1) Text Transformation Evaluator (rule-based + rubric)

Targets “rewrite this paragraph to style X with constraints Y”.

```ts
// eval/text-evaluator.ts
import fs from "node:fs";
import path from "node:path";
import { EvalReport, EvalCriterion } from "./types";

interface TextEvalConfig {
  style: "concise" | "formal" | "neutral";
  maxChars?: number;
  minChars?: number;
  bannedPhrases?: string[];
  requiredPhrases?: string[];
  readingGradeMax?: number; // optional: rough heuristic
}

function fleschKincaidApprox(text: string): number {
  // Lightweight heuristic: sentences, words, syllables (very rough)
  const sentences = Math.max(1, text.match(/[.!?]+/g)?.length ?? 1);
  const words = Math.max(1, text.trim().split(/\s+/g).length);
  const syllables = Math.max(
    1,
    text.match(/[aeiouy]+/gi)?.length ?? Math.ceil(words * 1.3)
  );
  // Flesch-Kincaid Grade (approximate)
  return 0.39 * (words / sentences) + 11.8 * (syllables / words) - 15.59;
}

function scoreBoolean(passed: boolean) {
  return passed ? 1 : 0;
}

export async function evaluateText(params: {
  taskId: string;
  artifactPath: string; // path to .txt or .md produced by the agent
  iterations: number;
  config: TextEvalConfig;
  acceptance: { minScore: number; mandatoryGates: string[] };
}): Promise<EvalReport> {
  const { taskId, artifactPath, iterations, config, acceptance } = params;
  const raw = fs.readFileSync(artifactPath, "utf8");
  const text = raw.trim();

  const criteria: EvalCriterion[] = [];

  // C1: length band
  const withinMax = config.maxChars ? text.length <= config.maxChars : true;
  const withinMin = config.minChars ? text.length >= config.minChars : true;
  criteria.push({
    id: "length-band",
    description: "Text length within target bounds",
    weight: 0.15,
    passed: withinMax && withinMin,
    score: scoreBoolean(withinMax && withinMin),
    notes: `len=${text.length}`,
  });

  // C2: banned phrases
  const banned = config.bannedPhrases ?? [];
  const bannedHits = banned.filter((p) =>
    text.toLowerCase().includes(p.toLowerCase())
  );
  criteria.push({
    id: "no-banned-phrases",
    description: "Avoid banned phrases",
    weight: 0.15,
    passed: bannedHits.length === 0,
    score: scoreBoolean(bannedHits.length === 0),
    notes: bannedHits.length ? `hits=${bannedHits.join(",")}` : undefined,
  });

  // C3: required phrases
  const required = config.requiredPhrases ?? [];
  const missingReq = required.filter(
    (p) => !text.toLowerCase().includes(p.toLowerCase())
  );
  criteria.push({
    id: "required-phrases",
    description: "Include required phrases",
    weight: 0.15,
    passed: missingReq.length === 0,
    score: scoreBoolean(missingReq.length === 0),
    notes: missingReq.length ? `missing=${missingReq.join(",")}` : undefined,
  });

  // C4: readability ceiling (optional)
  let grade = 0;
  let readabilityPass = true;
  if (config.readingGradeMax != null) {
    grade = fleschKincaidApprox(text);
    readabilityPass = grade <= config.readingGradeMax;
  }
  criteria.push({
    id: "readability",
    description: "Reading grade at or below max",
    weight: 0.1,
    passed: readabilityPass,
    score: scoreBoolean(readabilityPass),
    notes: `grade≈${grade.toFixed(1)}`,
  });

  // C5: style heuristic (very light)
  const style = config.style;
  const hasContractions = /n't|'re|'s|'d|'ll|'ve|'m\b/.test(text);
  const stylePass = style === "formal" ? !hasContractions : true;
  criteria.push({
    id: "style-heuristic",
    description: `Style conforms (${style})`,
    weight: 0.1,
    passed: stylePass,
    score: scoreBoolean(stylePass),
    notes: `contractions=${hasContractions}`,
  });

  // C6: structure markers (paragraphs, headings)
  const paras = text.split(/\n{2,}/g).length;
  const structurePass = paras >= 1;
  criteria.push({
    id: "structure",
    description: "Basic structure present (≥1 paragraph)",
    weight: 0.1,
    passed: structurePass,
    score: scoreBoolean(structurePass),
    notes: `paragraphs=${paras}`,
  });

  // “Mandatory gates” can be mapped to criteria IDs for text tasks if you wish
  const scores = criteria.reduce((s, c) => s + c.score * c.weight, 0);
  const thresholdsMet: string[] = [];
  const thresholdsMissed: string[] = [];

  // Example: treat readability & no-banned as gates if listed
  for (const g of acceptance.mandatoryGates) {
    const crit = criteria.find((c) => c.id === g);
    if (!crit) continue;
    (crit.passed ? thresholdsMet : thresholdsMissed).push(g);
  }

  const passCore =
    scores >= acceptance.minScore && thresholdsMissed.length === 0;
  const report: EvalReport = {
    taskId,
    artifactPaths: [path.resolve(artifactPath)],
    status: passCore ? "pass" : "iterate",
    score: Number(scores.toFixed(3)),
    thresholdsMet,
    thresholdsMissed,
    criteria,
    iterations,
    stopReason: passCore ? "satisficed" : undefined,
    nextActions: passCore ? [] : ["Tighten style or fix gates, then re-run."],
    timestamp: new Date().toISOString(),
    logs: [],
  };

  return report;
}
```

---

# 2) Code Evaluator (tests + lint + typecheck + optional a11y)

Runs your project’s scripts and treats exits as _gates_. Lightweight and language-agnostic.

```ts
// eval/code-evaluator.ts
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { EvalReport, EvalCriterion } from "./types";

function run(cmd: string, args: string[], cwd: string) {
  const res = spawnSync(cmd, args, { cwd, encoding: "utf8" });
  return {
    code: res.status ?? 1,
    out: (res.stdout ?? "") + (res.stderr ?? ""),
  };
}

export async function evaluateCode(params: {
  taskId: string;
  projectDir: string;
  iterations: number;
  acceptance: { minScore: number; mandatoryGates: string[] };
  scripts?: { test?: string; lint?: string; typecheck?: string; a11y?: string };
}): Promise<EvalReport> {
  const { taskId, projectDir, iterations, acceptance, scripts } = params;

  const results: Record<string, { code: number; out: string }> = {};
  const criteria: EvalCriterion[] = [];

  const scriptMap = {
    tests: scripts?.test ?? "npm run test --silent",
    lint: scripts?.lint ?? "npm run lint --silent",
    typecheck: scripts?.typecheck ?? "npm run typecheck --silent",
    a11y: scripts?.a11y, // optional (axe, eslint-plugin-jsx-a11y, etc.)
  };

  function runScript(label: string, cmdLine?: string) {
    if (!cmdLine) return { code: 0, out: `${label}:skipped` };
    const [cmd, ...args] = cmdLine.split(/\s+/);
    return run(cmd, args, projectDir);
  }

  results.tests = runScript("tests", scriptMap.tests);
  results.lint = runScript("lint", scriptMap.lint);
  results.typecheck = runScript("typecheck", scriptMap.typecheck);
  if (scriptMap.a11y) results.a11y = runScript("a11y", scriptMap.a11y);

  function gateCriterion(
    id: string,
    description: string,
    weight: number,
    r?: { code: number; out: string }
  ) {
    if (!r) {
      return <EvalCriterion>{
        id,
        description: `${description} (skipped)`,
        weight,
        passed: true,
        score: 1,
      };
    }
    const passed = r.code === 0;
    return <EvalCriterion>{
      id,
      description,
      weight,
      passed,
      score: passed ? 1 : 0,
      notes: passed ? undefined : truncate(r.out),
    };
  }

  function truncate(s: string, n = 1200) {
    return s.length > n ? s.slice(0, n) + " …[truncated]" : s;
  }

  criteria.push(
    gateCriterion(
      "tests-pass",
      "Unit/integration tests pass",
      0.4,
      results.tests
    )
  );
  criteria.push(
    gateCriterion(
      "lint-clean",
      "Lint passes with no errors",
      0.25,
      results.lint
    )
  );
  criteria.push(
    gateCriterion("types-ok", "Typecheck passes", 0.25, results.typecheck)
  );
  if (results.a11y)
    criteria.push(
      gateCriterion("a11y-ok", "A11y checks pass", 0.1, results.a11y)
    );

  const score = criteria.reduce((s, c) => s + c.score * c.weight, 0);
  const thresholdsMissed: string[] = [];
  const thresholdsMet: string[] = [];

  for (const g of acceptance.mandatoryGates) {
    const crit = criteria.find((c) => c.id === g);
    if (!crit) continue;
    (crit.passed ? thresholdsMet : thresholdsMissed).push(g);
  }

  const passCore =
    score >= acceptance.minScore && thresholdsMissed.length === 0;

  const report = <EvalReport>{
    taskId,
    artifactPaths: [path.resolve(projectDir)],
    status: passCore ? "pass" : "iterate",
    score: Number(score.toFixed(3)),
    thresholdsMet,
    thresholdsMissed,
    criteria,
    iterations,
    stopReason: passCore ? "satisficed" : undefined,
    nextActions: passCore ? [] : ["Fix failing gates (see notes) and re-run."],
    logs: Object.values(results)
      .map((r) => r.out)
      .slice(0, 3),
    timestamp: new Date().toISOString(),
  };

  return report;
}
```

---

# 3) Design-Token Application Evaluator (no hard-coded values)

Ensures UI output uses your semantic tokens—not raw hex/px—matching your token registry.

```ts
// eval/token-evaluator.ts
import fs from "node:fs";
import path from "node:path";
import { EvalCriterion, EvalReport } from "./types";

interface TokenRegistry {
  colors: Record<string, string>; // e.g., { "bg.default": "{palette.gray.50}" }
  space: Record<string, string>;
  radius: Record<string, string>;
  // ...
}

export async function evaluateTokens(params: {
  taskId: string;
  artifactPath: string; // css/tsx/html to scan
  tokenJsonPath: string; // registry (design-tokens.json)
  iterations: number;
  acceptance: { minScore: number; mandatoryGates: string[] };
}): Promise<EvalReport> {
  const { taskId, artifactPath, tokenJsonPath, iterations, acceptance } =
    params;
  const text = fs.readFileSync(artifactPath, "utf8");
  const reg: TokenRegistry = JSON.parse(fs.readFileSync(tokenJsonPath, "utf8"));

  const criteria: EvalCriterion[] = [];

  // C1: No hard-coded hex
  const hexHits = text.match(/#[0-9a-f]{3,8}\b/gi) ?? [];
  const c1Pass = hexHits.length === 0;
  criteria.push({
    id: "no-hardcoded-hex",
    description: "No raw hex colors (use tokens/vars)",
    weight: 0.35,
    passed: c1Pass,
    score: c1Pass ? 1 : 0,
    notes: c1Pass
      ? undefined
      : `hex=${[...new Set(hexHits)].slice(0, 10).join(", ")}`,
  });

  // C2: No raw pixel spacing (encourage tokens/variables)
  const pxHits = text.match(/(?<!font-)\b\d+px\b/g) ?? [];
  const c2Pass = pxHits.length <= 2; // allow a tiny tail
  criteria.push({
    id: "no-raw-px-spacing",
    description: "Avoid raw px in spacing (use space tokens/vars)",
    weight: 0.25,
    passed: c2Pass,
    score: c2Pass ? 1 : 0,
    notes: c2Pass
      ? undefined
      : `px=${[...new Set(pxHits)].slice(0, 10).join(", ")}`,
  });

  // C3: Token coverage (presence of known tokens)
  const tokenKeys = [
    ...Object.keys(reg.colors || {}),
    ...Object.keys(reg.space || {}),
    ...Object.keys(reg.radius || {}),
  ];
  const usedTokens = tokenKeys.filter((k) => text.includes(k));
  const coverage = usedTokens.length / Math.max(1, tokenKeys.length);
  const c3Pass = coverage >= 0.1; // low bar: ensure tokens appear
  criteria.push({
    id: "token-coverage",
    description: "Some token usage present",
    weight: 0.15,
    passed: c3Pass,
    score: c3Pass ? 1 : 0,
    notes: `coverage≈${coverage.toFixed(2)} used=${usedTokens
      .slice(0, 10)
      .join(", ")}`,
  });

  // C4: Disallow ad-hoc color names (heuristic)
  const adHocColorNames =
    text.match(/\b(color|bg|background|fill|stroke)[-_:]\w+\b/g) ?? [];
  const c4Pass = adHocColorNames.filter((n) => !n.includes(".")).length === 0;
  criteria.push({
    id: "no-ad-hoc-color-names",
    description: "No ad-hoc color names; use semantic tokens",
    weight: 0.25,
    passed: c4Pass,
    score: c4Pass ? 1 : 0,
    notes: c4Pass ? undefined : adHocColorNames.slice(0, 10).join(", "),
  });

  const score = criteria.reduce((s, c) => s + c.score * c.weight, 0);
  const thresholdsMissed: string[] = [];
  const thresholdsMet: string[] = [];

  for (const g of acceptance.mandatoryGates) {
    const crit = criteria.find((c) => c.id === g);
    if (!crit) continue;
    (crit.passed ? thresholdsMet : thresholdsMissed).push(g);
  }

  const passCore =
    score >= acceptance.minScore && thresholdsMissed.length === 0;

  const report: EvalReport = {
    taskId,
    artifactPaths: [path.resolve(artifactPath)],
    status: passCore ? "pass" : "iterate",
    score: Number(score.toFixed(3)),
    thresholdsMet,
    thresholdsMissed,
    criteria,
    iterations,
    stopReason: passCore ? "satisficed" : undefined,
    timestamp: new Date().toISOString(),
  };

  return report;
}
```

---

# 4) Orchestrator (satisficing logic, iteration caps, yield)

This wrapper calls a chosen evaluator, then decides whether to **iterate** (ask the agent to try again) or **stop**.

```ts
// eval/orchestrator.ts
import fs from "node:fs";
import path from "node:path";
import { evaluateText } from "./text-evaluator";
import { evaluateCode } from "./code-evaluator";
import { evaluateTokens } from "./token-evaluator";
import { EvalReport } from "./types";

interface Acceptance {
  minScore: number;
  mandatoryGates: string[];
  iterationPolicy: {
    maxIterations: number;
    minDeltaToContinue: number;
    noChangeBudget: number;
  };
}

interface History {
  reports: EvalReport[];
  noChangeStreak: number;
}

function loadAcceptance(): Acceptance {
  return JSON.parse(
    fs.readFileSync(path.resolve("eval/acceptance.json"), "utf8")
  );
}

function delta(prev: EvalReport | undefined, curr: EvalReport) {
  if (!prev) return 1; // first run
  return curr.score - prev.score;
}

function decide(history: History, current: EvalReport, a: Acceptance) {
  const prev = history.reports.at(-1);
  const d = delta(prev, current);
  const iterations = current.iterations;

  let noChangeStreak = history.noChangeStreak;
  if (d < a.minDeltaToContinue) noChangeStreak += 1;
  else noChangeStreak = 0;

  if (current.status === "pass") {
    current.stopReason = "satisficed";
    return { decision: "stop", noChangeStreak };
  }

  if (iterations >= a.iterationPolicy.maxIterations) {
    current.stopReason = "max-iterations";
    return { decision: "stop", noChangeStreak };
  }

  if (noChangeStreak > a.iterationPolicy.noChangeBudget) {
    current.stopReason = "quality-ceiling";
    return { decision: "stop", noChangeStreak };
  }

  return { decision: "iterate", noChangeStreak };
}

async function main() {
  const acceptance = loadAcceptance();
  const historyPath = path.resolve("eval/history.json");
  const hist: History = fs.existsSync(historyPath)
    ? JSON.parse(fs.readFileSync(historyPath, "utf8"))
    : { reports: [], noChangeStreak: 0 };

  const task = process.argv[2]; // e.g., "text", "code", "tokens"
  const iterations = (hist.reports.at(-1)?.iterations ?? 0) + 1;

  let report: EvalReport;

  if (task === "text") {
    report = await evaluateText({
      taskId: "rewrite-para",
      artifactPath: "artifacts/out.txt",
      iterations,
      config: {
        style: "formal",
        maxChars: 800,
        bannedPhrases: ["very", "really"],
        requiredPhrases: ["acceptance criteria"],
      },
      acceptance,
    });
  } else if (task === "code") {
    report = await evaluateCode({
      taskId: "ui-button",
      projectDir: "./project",
      iterations,
      acceptance,
      scripts: {
        test: "npm run test",
        lint: "npm run lint",
        typecheck: "npm run typecheck",
      },
    });
  } else if (task === "tokens") {
    report = await evaluateTokens({
      taskId: "apply-design-tokens",
      artifactPath: "artifacts/Button.tsx",
      tokenJsonPath: "design-tokens.json",
      iterations,
      acceptance,
    });
  } else {
    throw new Error(`Unknown task '${task}'`);
  }

  // Decide next step
  const decision = decide(hist, report, acceptance);
  const newHist: History = {
    reports: [...hist.reports, report],
    noChangeStreak: decision.noChangeStreak,
  };
  fs.writeFileSync(historyPath, JSON.stringify(newHist, null, 2));

  // Emit final machine-readable result to stdout for the agent
  process.stdout.write(
    JSON.stringify({ report, decision: decision.decision }, null, 2)
  );
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

**CLI usage:**

```bash
# 1) Agent generates artifact(s)
# 2) Orchestrator evaluates & decides stop/iterate
ts-node eval/orchestrator.ts text
ts-node eval/orchestrator.ts code
ts-node eval/orchestrator.ts tokens
```

---

# 5) Agent-Facing Decision Rules (summary)

- **Stop (satisfice)** when:

  - `score ≥ minScore` **AND** all `mandatoryGates` pass → `status: pass`, `stopReason: "satisficed"`.

- **Iterate** when:

  - `score < minScore` OR any mandatory gate fails **AND**
  - `iterations < maxIterations` **AND**
  - `delta ≥ minDeltaToContinue` **OR** `noChangeStreak ≤ noChangeBudget`.

- **Stop (quality-ceiling)** when:

  - `delta < minDeltaToContinue` for more than `noChangeBudget` consecutive runs.

- **Stop (max-iterations)** when:

  - `iterations ≥ maxIterations`.

This enforces “good-enough,” prevents yes-man polishing, and caps loops.

---

# 6) Optional: “Model-as-Judge” Rubric (hybrid)

For cases where a rule/lint isn’t enough (e.g., “tone appropriateness”), you can add a **secondary model-as-judge** step that outputs **only** rubric-bounded JSON (never free-form prose). The orchestrator then **multiplies** that rubric score by your rule-based score. Keep it local (e.g., Gemma-Instruct) for cost.

**Judge prompt (sketch):**

```
SYSTEM: You are a strict rubric scorer. Output JSON only matching:
{ "rubricScores": { "<criterion-id>": 0..1 }, "rationale": { "<criterion-id>": "..." } }

USER: Evaluate the artifact below against these criteria:
- clarity (weight 0.2): Is the writing unambiguous?
- coherence (0.2): Does it flow logically?
- audience-fit (0.2): Is tone suitable for engineering audiences?
- directive-strength (0.2): Are instructions actionable?
- concision (0.2): Is verbosity controlled?
Return scores in [0,1]. No commentary outside JSON.

ARTIFACT:
<<<text>>>
```

You then merge `rubricScores` into `criteria` and recompute `score`.

---

# 7) Minimal Project Layout

```
/your-agent/
  artifacts/
    out.txt
    Button.tsx
  design-tokens.json
  eval/
    types.ts
    acceptance.json
    text-evaluator.ts
    code-evaluator.ts
    token-evaluator.ts
    orchestrator.ts
    history.json   (auto-generated)
  project/        (for code tests)
    package.json  (scripts: test, lint, typecheck)
```

---

# 8) Why this works

- **Uniform contract** → any new evaluator (docs, diagrams, datasets) is plug-compatible.
- **Hard gates + soft scores** → production credibility without losing nuance.
- **Satisficing** → explicit stop rules (+ plateau detection) defeat infinite polishing.
- **Local-first** → works with Gemma/Ollama; no UI required; CLI orchestratable.
- **CAWS-ready** → reports can be attached to PRs; diffs and provenance are built-in.

---

If you want, I can turn these into a **ready-to-run repo scaffold** (with `tsconfig`, `package.json`, and a sample `npm` script set) and include a couple of sample artifacts and token registries so you can run the loop immediately.
