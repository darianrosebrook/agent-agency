# ARBITER-003: Integration Assessment & Revised Plan

**Date**: October 11, 2025  
**Status**: Phase 1 Complete - Pivoting to Integration Strategy  
**Decision**: Adopt **Option B - Import CAWS modules, extend with arbiter logic**

---

## 📊 What We Built (Phase 1 Assessment)

### ✅ Assets Created

| Component            | Status      | Quality   | Integration Strategy                 |
| -------------------- | ----------- | --------- | ------------------------------------ |
| **Type Definitions** | ✅ Complete | Excellent | **Keep & Extend** - Good foundation  |
| **SpecValidator**    | ✅ Complete | Good      | **Replace** - Use CAWS CLI directly  |
| **BudgetValidator**  | ✅ Complete | Good      | **Replace** - Use CAWS policy system |
| **PolicyLoader**     | ✅ Complete | Basic     | **Replace** - Use CAWS native loader |
| **WaiverManager**    | ✅ Complete | Basic     | **Replace** - Use CAWS waiver system |
| **Tests (45+)**      | ✅ Complete | Excellent | **Adapt** - Reuse test patterns      |

### ⚠️ Critical Gaps Identified

1. **No Policy-First Architecture** - Budgets treated as per-spec fields
2. **No MCP Integration** - Can't expose to orchestrator agents
3. **No Real-Time Monitoring** - Batch validation only
4. **No Iterative Guidance** - Pass/fail only, no step-by-step help
5. **Simplified Provenance** - Missing AI attribution and analytics
6. **Reimplementation vs Integration** - Built from scratch instead of wrapping CAWS

---

## 🎯 Revised Integration Strategy: Option B

### Core Principle

**"Treat CAWS as infrastructure, not inspiration"**

Instead of adapting CAWS CLI code, we'll:

1. **Import CAWS as dependency** (`@paths.design/caws-cli`, `@paths.design/caws-mcp-server`)
2. **Wrap CAWS validation** with arbiter-specific orchestration logic
3. **Extend with arbiter features** (task assignment, worker monitoring, multi-agent coordination)
4. **Preserve Phase 1 work** where it adds value (types, test patterns, integration glue)

---

## 📦 Phase 1 Work: Keep, Refactor, or Replace?

### KEEP: Type Definitions (Minimal Changes)

**What We Built**:

```typescript
// src/caws-validator/types/validation-types.ts
export interface CAWSValidationResult {
  /* ... */
}
export interface BudgetCompliance {
  /* ... */
}
export interface QualityGateResult {
  /* ... */
}
```

**Why Keep**:

- Provides TypeScript type safety for CAWS JS modules
- Acts as adapter layer between CAWS and V2
- Already comprehensive and well-structured

**Refactor Needed**:

```typescript
// Add CAWS CLI integration types
import type { CAWSCLIResult } from "@paths.design/caws-cli";

export interface ArbiterValidationResult extends CAWSValidationResult {
  // Arbiter-specific extensions
  taskId: string;
  assignedAgent: string;
  orchestrationMetadata: OrchestrationMetadata;
}
```

### REPLACE: Validators with CAWS Wrappers

**Current Implementation** (349 lines):

```typescript
// src/caws-validator/validation/SpecValidator.ts
export class SpecValidator {
  validateWorkingSpec(spec: WorkingSpec): SpecValidationResult {
    // 300+ lines of reimplemented validation
  }
}
```

**New Implementation** (50 lines):

```typescript
// src/caws-validator/adapters/CAWSValidationAdapter.ts
import { validateCommand } from "@paths.design/caws-cli";

export class CAWSValidationAdapter {
  async validateWorkingSpec(
    spec: WorkingSpec,
    projectRoot: string
  ): Promise<ArbiterValidationResult> {
    // Write spec to .caws/working-spec.yaml
    await this.writeSpecFile(spec, projectRoot);

    // Use CAWS CLI directly
    const cawsResult = await validateCommand(
      path.join(projectRoot, ".caws/working-spec.yaml"),
      { autoFix: false, quiet: false }
    );

    // Enrich with arbiter metadata
    return this.enrichWithArbiterContext(cawsResult, spec);
  }
}
```

### KEEP & ENHANCE: Test Patterns

**What We Built**: 45+ comprehensive test cases

**Value**: Test patterns are excellent and can be adapted

**Strategy**:

```typescript
// tests/integration/caws-integration.test.ts
describe("CAWS CLI Integration", () => {
  it("should validate working spec via CAWS CLI", async () => {
    const adapter = new CAWSValidationAdapter();
    const spec = createValidSpec();

    const result = await adapter.validateWorkingSpec(spec, testProjectRoot);

    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
    // Reuse our existing test assertions
  });
});
```

---

## 🏗️ Revised Architecture

### Layer 1: CAWS Foundation (Import as Dependency)

```typescript
// package.json
{
  "dependencies": {
    "@paths.design/caws-cli": "^3.4.0",
    "@paths.design/caws-mcp-server": "^1.0.0"
  }
}
```

### Layer 2: Adapter Layer (What We Keep from Phase 1)

```typescript
// src/caws-integration/
├── adapters/
│   ├── CAWSValidationAdapter.ts      // Wrap CAWS CLI validation
│   ├── CAWSPolicyAdapter.ts          // Policy loading with caching
│   └── CAWSProvenanceAdapter.ts      // Provenance tracking
├── types/
│   ├── arbiter-caws-types.ts         // Extended types (keep Phase 1 work)
│   └── mcp-tool-types.ts             // MCP tool schemas
└── utils/
    ├── spec-file-manager.ts          // Convert spec ↔ YAML
    └── caws-process-manager.ts       // Execute CAWS CLI commands
```

### Layer 3: Arbiter Extensions (New Capabilities)

```typescript
// src/arbiter/
├── orchestration/
│   ├── TaskValidator.ts              // Validate before assignment
│   ├── BudgetAllocator.ts            // Multi-agent budget allocation
│   └── WorkerMonitor.ts              // Real-time worker tracking
├── mcp/
│   ├── ArbiterMCPServer.ts           // MCP server for orchestrator
│   └── tools/
│       ├── arbiter-validate.ts       // arbiter_validate() tool
│       ├── arbiter-assign.ts         // arbiter_assign_task() tool
│       └── arbiter-monitor.ts        // arbiter_monitor_progress() tool
└── guidance/
    └── IterativeGuidance.ts          // Step-by-step agent guidance
```

---

## 🔄 Migration Path

### Week 1: Foundation Integration

#### Day 1-2: Add CAWS Dependencies

```bash
cd /Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v2
npm install @paths.design/caws-cli@^3.4.0
npm install @paths.design/caws-mcp-server@^1.0.0
```

**Deliverable**: CAWS CLI callable from V2 codebase

#### Day 3-4: Build Adapter Layer

```typescript
// src/caws-integration/adapters/CAWSValidationAdapter.ts
export class CAWSValidationAdapter {
  private cawsCLI: typeof import("@paths.design/caws-cli");

  async validateSpec(spec: WorkingSpec): Promise<ArbiterValidationResult> {
    // Convert WorkingSpec → YAML
    const yamlPath = await this.writeSpecFile(spec);

    // Call CAWS CLI
    const result = await this.cawsCLI.validateCommand(yamlPath, {
      autoFix: false,
      suggestions: true,
      checkBudget: true,
    });

    // Convert CAWS result → ArbiterValidationResult
    return this.toArbiterResult(result, spec);
  }
}
```

**Deliverable**: Working adapter that calls CAWS CLI

#### Day 5: Test Integration

```typescript
// tests/integration/caws-validation-adapter.test.ts
describe("CAWSValidationAdapter", () => {
  it("should validate via CAWS CLI", async () => {
    const adapter = new CAWSValidationAdapter();
    const result = await adapter.validateSpec(createValidSpec());

    expect(result.valid).toBe(true);
    expect(result.cawsVersion).toBe("3.4.0");
  });
});
```

**Deliverable**: 20+ integration tests passing

### Week 2: MCP Integration

#### Day 1-2: Build MCP Server

```typescript
// src/arbiter/mcp/ArbiterMCPServer.ts
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

export class ArbiterMCPServer extends Server {
  constructor(private cawsAdapter: CAWSValidationAdapter) {
    super(
      {
        name: "arbiter-mcp-server",
        version: "2.0.0",
      },
      {
        capabilities: {
          tools: {},
          resources: {},
        },
      }
    );

    this.registerTools();
  }

  private registerTools() {
    this.tool(
      "arbiter_validate",
      "Validate working spec and enforce quality gates",
      {
        spec: { type: "object", description: "Working specification" },
      },
      async ({ spec }) => {
        const result = await this.cawsAdapter.validateSpec(spec);
        return {
          content: [{ type: "text", text: JSON.stringify(result) }],
        };
      }
    );
  }
}
```

**Deliverable**: MCP server exposing arbiter tools

#### Day 3-4: Implement Core Tools

```typescript
// arbiter_validate - Validate before assignment
// arbiter_assign_task - Assign with CAWS constraints
// arbiter_monitor_progress - Real-time monitoring
// arbiter_generate_verdict - Final judgment
```

**Deliverable**: 4 core MCP tools

#### Day 5: Test MCP Integration

```typescript
// tests/integration/arbiter-mcp-server.test.ts
describe("ArbiterMCPServer", () => {
  it("should expose arbiter_validate tool", async () => {
    const server = new ArbiterMCPServer(adapter);
    const tools = await server.listTools();

    expect(tools.tools.some((t) => t.name === "arbiter_validate")).toBe(true);
  });
});
```

**Deliverable**: MCP integration tests passing

### Week 3: Real-Time Monitoring

#### Day 1-3: Build BudgetMonitor

```typescript
// src/arbiter/monitoring/BudgetMonitor.ts
import chokidar from "chokidar";

export class BudgetMonitor extends EventEmitter {
  async watchProject(spec: WorkingSpec) {
    const watcher = chokidar.watch(spec.scope.in);

    watcher.on("change", async (filePath) => {
      const stats = await this.analyzeChanges();
      const usage = this.calculateUsage(stats, spec);

      if (usage.overall > 80) {
        this.emit("budget:warning", {
          type: "approaching_limit",
          usage: usage.overall,
          remaining: {
            files: stats.maxFiles - stats.filesChanged,
            loc: stats.maxLoc - stats.linesChanged,
          },
        });
      }
    });
  }
}
```

**Deliverable**: Real-time budget monitoring

#### Day 4-5: Build IterativeGuidance

```typescript
// src/arbiter/guidance/IterativeGuidance.ts
export class IterativeGuidance {
  async getNextSteps(
    spec: WorkingSpec,
    currentState: ProjectState
  ): Promise<GuidanceResult> {
    // Calculate progress
    const progress = this.calculateProgress(spec, currentState);

    // Identify gaps
    const gaps = this.identifyGaps(spec, currentState);

    // Generate recommendations
    return {
      currentProgress: progress,
      nextSteps: this.generateNextSteps(gaps),
      estimatedRemaining: this.estimateRemainingWork(gaps),
    };
  }
}
```

**Deliverable**: Iterative guidance system

### Week 4: Provenance & Polish

#### Day 1-2: Enhanced Provenance

```typescript
// src/arbiter/provenance/ProvenanceTracker.ts
export class ProvenanceTracker {
  async recordTaskExecution(
    task: Task,
    result: TaskResult
  ): Promise<ProvenanceEntry> {
    return {
      taskId: task.id,
      arbiterVersion: "2.0.0",
      workerAgent: result.assignedAgent,
      aiTool: result.aiTool, // 'cursor-composer', 'gpt-4', etc.
      qualityMetrics: {
        coverage: result.coverage,
        mutationScore: result.mutationScore,
        lintErrors: result.lintErrors,
      },
      budgetCompliance: {
        filesUsed: result.filesChanged,
        locUsed: result.linesChanged,
        withinBudget: result.budgetCompliant,
      },
      verdict: result.verdict,
      humanInterventions: result.humanInterventions,
      timestamp: new Date().toISOString(),
    };
  }
}
```

**Deliverable**: AI-aware provenance tracking

#### Day 3-5: Integration Testing & Documentation

- End-to-end orchestrator tests
- Performance benchmarking
- API documentation
- Usage examples

---

## 📊 What Changes from Original Plan

### ❌ Remove (Replaced by CAWS)

- `src/caws-validator/validation/SpecValidator.ts` (405 lines) → Use CAWS CLI
- `src/caws-validator/validation/BudgetValidator.ts` (249 lines) → Use CAWS CLI
- `src/caws-validator/utils/policy-loader.ts` (103 lines) → Use CAWS CLI
- `src/caws-validator/waivers/WaiverManager.ts` (141 lines) → Use CAWS CLI

**Total Removed**: ~900 lines (replaced with ~100 line adapter)

### ✅ Keep (Enhanced)

- `src/caws-validator/types/validation-types.ts` → Extend with arbiter types
- Test patterns (45+ tests) → Adapt for integration testing
- Type definitions → Add CAWS CLI type wrappers

**Total Kept**: ~650 lines (types) + test patterns

### ➕ Add (New Capabilities)

- MCP server integration (~300 lines)
- Real-time monitoring (~200 lines)
- Iterative guidance (~250 lines)
- Enhanced provenance (~200 lines)
- CAWS adapter layer (~150 lines)

**Total Added**: ~1,100 lines (net: +200 lines vs original plan)

---

## 🎯 Success Metrics (Revised)

### Integration Quality

- [ ] CAWS CLI callable from V2 codebase
- [ ] Policy.yaml loaded and enforced
- [ ] Waivers apply correctly
- [ ] Quality gates execute via CAWS
- [ ] MCP server exposes arbiter tools

### Arbiter Extensions

- [ ] Real-time budget monitoring working
- [ ] Iterative guidance provides actionable steps
- [ ] Provenance tracks AI attribution
- [ ] Multi-agent coordination supported

### Performance

- [ ] Validation completes in <2s (via CAWS)
- [ ] Budget monitoring overhead <5%
- [ ] MCP tool latency <100ms

### Test Coverage

- [ ] 20+ integration tests with CAWS CLI
- [ ] 10+ MCP tool tests
- [ ] 5+ end-to-end orchestration tests
- [ ] 80%+ coverage on arbiter extensions

---

## 💡 Key Benefits of Integration Approach

### 1. **Faster Delivery**

- Week 1: Working integration (vs Week 3 with reimplementation)
- 4 weeks total (vs 6-8 weeks reimplementing)

### 2. **Battle-Tested Validation**

- CAWS CLI has 3+ years of production hardening
- Comprehensive edge case handling
- Known bug fixes and optimizations

### 3. **Ecosystem Compatibility**

- Works with existing CAWS projects
- Compatible with CAWS VS Code extension
- Can leverage CAWS community tools

### 4. **Focus on Innovation**

- Spend time on arbiter orchestration, not validation
- Build multi-agent coordination features
- Enhance AI-agent collaboration patterns

### 5. **Maintenance Burden**

- CAWS CLI maintained by @paths.design
- Security updates handled upstream
- Bug fixes flow downstream automatically

---

## 🚀 Immediate Next Steps

### 1. Install CAWS Dependencies (Today)

```bash
npm install @paths.design/caws-cli@^3.4.0
npm install @paths.design/caws-mcp-server@^1.0.0
npm install chokidar  # For file watching
npm install yaml      # For spec serialization
```

### 2. Create Adapter Skeleton (Today)

```typescript
// src/caws-integration/adapters/CAWSValidationAdapter.ts
export class CAWSValidationAdapter {
  async validateSpec(spec: WorkingSpec): Promise<ArbiterValidationResult> {
    // TODO: Implement CAWS CLI integration
    throw new Error("Not implemented");
  }
}
```

### 3. Write Integration Test (Today)

```typescript
// tests/integration/caws-adapter.test.ts
describe("CAWSValidationAdapter", () => {
  it("should call CAWS CLI", async () => {
    const adapter = new CAWSValidationAdapter();
    const spec = createValidSpec();

    const result = await adapter.validateSpec(spec);

    expect(result).toBeDefined();
  });
});
```

### 4. Update TODO List (Today)

- Mark Phase 1 validators as "deprecated"
- Add new tasks for integration approach
- Update timeline to 4 weeks

---

## 📝 Decision Log

**Date**: October 11, 2025  
**Decision**: Pivot from reimplementation to integration (Option B)  
**Rationale**:

- Phase 1 audit revealed CAWS CLI has production features we missed
- Integration is faster and more maintainable
- Allows focus on arbiter-specific orchestration features
- Preserves valuable work (types, test patterns)

**Impact**:

- ~900 lines of reimplementation work deprecated
- Timeline reduced from 6-8 weeks to 4 weeks
- Risk reduced (using battle-tested CAWS foundation)
- Innovation capacity increased (focus on orchestration)

**Approval**: Pending team review

---

**Status**: Ready for implementation  
**Next Review**: End of Week 1 (integration working)  
**Success Criteria**: CAWS CLI successfully validates specs from V2
