# ARBITER v2 API Reference

**ARBITER (AI Runtime for Business Efficiency and Task Execution & Regulation)** is a comprehensive orchestration system that integrates CAWS (Coding-Agent Working Standard) constitutional authority with AI agent coordination.

## Table of Contents

- [Overview](#overview)
- [Core Components](#core-components)
  - [CAWS Integration Layer](#caws-integration-layer)
  - [MCP Server](#mcp-server)
  - [Budget Monitoring](#budget-monitoring)
  - [Iterative Guidance](#iterative-guidance)
  - [Provenance Tracking](#provenance-tracking)
- [API Reference](#api-reference)
- [Configuration](#configuration)
- [Usage Examples](#usage-examples)
- [Integration Guides](#integration-guides)

## Overview

ARBITER v2 provides a complete AI orchestration platform with:

- **Constitutional Authority**: CAWS-based governance and quality gates
- **Real-time Monitoring**: Budget tracking and threshold alerts
- **Intelligent Guidance**: Progress analysis and next-step recommendations
- **Complete Provenance**: AI attribution and audit trails
- **MCP Integration**: Model Context Protocol for agent communication

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    ARBITER Orchestrator                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │ CAWS        │ │ MCP Server  │ │ Budget      │           │
│  │ Integration │ │             │ │ Monitor     │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
│  ┌─────────────┐ ┌─────────────┐                           │
│  │ Iterative   │ │ Provenance  │                           │
│  │ Guidance    │ │ Tracker     │                           │
│  └─────────────┘ └─────────────┘                           │
└─────────────────────────────────────────────────────────────┘
          │              │              │
          └──────────────┼──────────────┘
                         │
              ┌─────────────┐
              │ Working     │
              │ Spec &      │
              │ Policies    │
              └─────────────┘
```

## Core Components

### CAWS Integration Layer

Provides constitutional authority and validation through CAWS CLI integration.

#### CAWSValidationAdapter

Validates working specifications against CAWS policies.

```typescript
import { CAWSValidationAdapter } from "./src/caws-integration/adapters/CAWSValidationAdapter.js";

const validator = new CAWSValidationAdapter({
  projectRoot: "/path/to/project",
  arbiterVersion: "2.0.0",
});

// Validate existing spec file
const result = await validator.validateExistingSpec();
if (result.success && result.data?.passed) {
  console.log("✅ Spec validation passed");
}
```

#### CAWSPolicyAdapter

Loads and caches CAWS policies with budget derivation.

```typescript
import { CAWSPolicyAdapter } from "./src/caws-integration/adapters/CAWSPolicyAdapter.js";

const policyAdapter = new CAWSPolicyAdapter({
  projectRoot: "/path/to/project",
  enableCaching: true,
});

// Load policy
const policyResult = await policyAdapter.loadPolicy();

// Derive budget for working spec
const budgetResult = await policyAdapter.deriveBudget({
  spec: workingSpec,
  projectRoot: "/path/to/project",
  applyWaivers: true,
});
```

#### SpecFileManager

Handles YAML serialization/deserialization of working specs.

```typescript
import { SpecFileManager } from "./src/caws-integration/utils/spec-file-manager.js";

const specManager = new SpecFileManager({
  projectRoot: "/path/to/project",
});

// Write spec to file
await specManager.writeSpecFile(workingSpec);

// Read spec from file
const loadedSpec = await specManager.readSpecFile();
```

### MCP Server

Model Context Protocol server exposing ARBITER tools to AI agents.

#### ArbiterMCPServer

Extends MCP Server with orchestration tools.

```typescript
import { ArbiterMCPServer } from "./src/mcp-server/ArbiterMCPServer.js";

const mcpServer = new ArbiterMCPServer("/path/to/project");
await mcpServer.initialize();

// Server exposes tools:
// - arbiter_validate: Validate working specs
// - arbiter_assign_task: Assign tasks to agents
// - arbiter_monitor_progress: Monitor task progress
// - arbiter_generate_verdict: Generate quality verdicts
```

**Available MCP Tools:**

1. **arbiter_validate**

   - Validates working specs against CAWS policies
   - Returns validation results and remediation suggestions

2. **arbiter_assign_task**

   - Assigns tasks to available agents based on capability matching
   - Supports different assignment strategies (capability, performance, round-robin)

3. **arbiter_monitor_progress**

   - Monitors task progress with real-time budget tracking
   - Provides threshold alerts and progress summaries

4. **arbiter_generate_verdict**
   - Generates final quality verdicts for completed tasks
   - Evaluates against multiple criteria (budget, acceptance, quality gates)

### Budget Monitoring

Real-time budget tracking with file system monitoring.

#### BudgetMonitor

Monitors project budget usage with configurable thresholds.

```typescript
import { BudgetMonitor } from "./src/monitoring/BudgetMonitor.js";

const monitor = new BudgetMonitor({
  projectRoot: "/path/to/project",
  spec: workingSpec,
  useFileWatching: true,
  thresholds: {
    warning: 0.5, // 50%
    critical: 0.8, // 80%
    exceeded: 0.95, // 95%
  },
});

// Start monitoring
await monitor.start();

// Get current status
const status = monitor.getStatus();
console.log(
  `Files: ${status.currentUsage.filesChanged}/${status.currentUsage.maxFiles}`
);
console.log(
  `LOC: ${status.currentUsage.linesChanged}/${status.currentUsage.maxLoc}`
);

// Listen for alerts
monitor.on("alert", (alert) => {
  console.log(`🚨 Budget Alert: ${alert.severity} - ${alert.message}`);
});

// Stop monitoring
await monitor.stop();
```

**Budget Status Properties:**

- `filesChanged`: Number of files modified
- `maxFiles`: Maximum allowed files (from policy)
- `linesChanged`: Total lines of code changed
- `maxLoc`: Maximum allowed lines of code
- `filesPercentage`: Current usage as percentage
- `locPercentage`: LOC usage as percentage

**Alert Severities:**

- `warning`: 50% usage threshold
- `critical`: 80% usage threshold
- `exceeded`: 95% usage threshold

### Iterative Guidance

Intelligent progress tracking and gap analysis.

#### IterativeGuidance

Analyzes working spec progress and provides actionable guidance.

```typescript
import { IterativeGuidance } from "./src/guidance/IterativeGuidance.js";

const guidance = new IterativeGuidance(
  {
    spec: workingSpec,
    projectRoot: "/path/to/project",
  },
  {
    phase: "implementation",
    teamSize: 3,
    experienceLevel: "senior",
    timePressure: "medium",
  }
);

// Analyze current progress
const analysis = await guidance.analyzeProgress();
console.log(
  `Overall Progress: ${(analysis.summary?.overallProgress || 0) * 100}%`
);

// Get next steps
const nextSteps = analysis.summary?.nextSteps || [];
nextSteps.forEach((step) => {
  console.log(`📋 ${step.priority}: ${step.description}`);
});

// Get step-by-step guidance
const stepGuidance = await guidance.getStepGuidance(0);
console.log(`Current Step: ${stepGuidance.step?.title}`);

// Get recommendations
const recommendations = await guidance.getRecommendations();
recommendations.forEach((rec) => {
  console.log(`💡 ${rec.type}: ${rec.message}`);
});
```

**Guidance Capabilities:**

- **Progress Analysis**: Tracks completion of acceptance criteria
- **Gap Identification**: Identifies missing implementations and tests
- **Next Steps**: Provides prioritized actionable tasks
- **Work Estimation**: Estimates time and complexity for remaining work
- **Risk Assessment**: Evaluates project risks and blockers

### Provenance Tracking

Complete audit trails with AI attribution.

#### ProvenanceTracker

Tracks all changes with detailed provenance information.

```typescript
import { ProvenanceTracker } from "./src/provenance/ProvenanceTracker.js";

const tracker = new ProvenanceTracker({
  projectRoot: "/path/to/project",
  spec: workingSpec,
  enableAIAttribution: true,
  cawsIntegration: { enabled: false },
});

// Record a change
await tracker.recordEntry(
  "commit",
  specId,
  { type: "human", identifier: "developer-1" },
  {
    type: "implemented",
    description: "Added user authentication logic",
    details: { feature: "auth", complexity: "moderate" },
  },
  {
    affectedFiles: [
      { path: "src/auth/login.ts", changeType: "added", linesChanged: 50 },
      {
        path: "tests/auth/login.test.ts",
        changeType: "added",
        linesChanged: 30,
      },
    ],
  }
);

// Get provenance chain
const chain = await tracker.getProvenanceChain(specId);
console.log(`Total entries: ${chain?.entries.length}`);

// Generate compliance report
const report = await tracker.generateReport(specId, "compliance");
console.log(`CAWS Compliant: ${report.compliance.cawsCompliant}`);

// Get AI attribution stats
const aiStats = await tracker.getAIAttributionStats();
console.log(`AI Contributions: ${aiStats.total} entries`);
```

**Provenance Entry Types:**

- `commit`: Code changes and implementations
- `validation`: CAWS validation runs
- `quality_gate`: Quality gate evaluations
- `human_review`: Human review activities

**AI Attribution Tracking:**

- Tracks which AI tools contributed to changes
- Measures AI vs human contribution ratios
- Supports provenance chain integrity verification

## API Reference

### Common Types

#### WorkingSpec

```typescript
interface WorkingSpec {
  id: string;
  title: string;
  risk_tier: 1 | 2 | 3;
  mode: "feature" | "fix" | "refactor" | "chore";
  blast_radius: {
    modules: string[];
    data_migration: boolean;
  };
  operational_rollback_slo: string; // e.g., "30m", "2h"
  scope: {
    in: string[]; // Paths included in scope
    out: string[]; // Paths excluded from scope
  };
  invariants: string[];
  acceptance: Array<{
    id: string;
    given: string;
    when: string;
    then: string;
  }>;
  non_functional?: {
    perf?: { api_p95_ms: number };
    security?: string[];
  };
  contracts?: Array<{
    type: "openapi" | "graphql";
    path: string;
  }>;
}
```

#### AdapterOperationResult<T>

```typescript
interface AdapterOperationResult<T = any> {
  success: boolean;
  data?: T;
  error?: {
    message: string;
    code?: string;
    details?: any;
  };
  durationMs?: number;
}
```

### Error Handling

All ARBITER APIs follow consistent error handling patterns:

```typescript
// Synchronous operations
try {
  const result = component.operation(params);
  if (!result.success) {
    console.error("Operation failed:", result.error?.message);
    return;
  }
  // Use result.data
} catch (error) {
  console.error("Unexpected error:", error);
}

// Asynchronous operations
try {
  const result = await component.asyncOperation(params);
  if (!result.success) {
    console.error("Async operation failed:", result.error?.message);
    return;
  }
  // Use result.data
} catch (error) {
  console.error("Unexpected error:", error);
}
```

## Configuration

### Environment Variables

```bash
# CAWS Integration
CAWS_CLI_PATH=/path/to/caws-cli
CAWS_POLICY_CACHE_TTL=300000

# Budget Monitoring
BUDGET_MONITOR_INTERVAL=1000
BUDGET_WARNING_THRESHOLD=0.5
BUDGET_CRITICAL_THRESHOLD=0.8

# Provenance Tracking
PROVENANCE_AUTO_SAVE=true
PROVENANCE_COMPRESSION=true

# MCP Server
MCP_SERVER_PORT=3000
MCP_DEBUG_LOGGING=false
```

### Configuration Files

#### .caws/policy.yaml

```yaml
version: "1.0.0"
risk_tiers:
  1:
    max_files: 15
    max_loc: 300
    coverage_threshold: 0.90
    mutation_threshold: 0.70
    contracts_required: true
    manual_review_required: true
  2:
    max_files: 20
    max_loc: 500
    coverage_threshold: 0.80
    mutation_threshold: 0.70
    contracts_required: true
    manual_review_required: false
  3:
    max_files: 25
    max_loc: 750
    coverage_threshold: 0.70
    mutation_threshold: 0.30
    contracts_required: false
    manual_review_required: false
```

#### .caws/working-spec.yaml

```yaml
id: "FEAT-001"
title: "User Authentication Feature"
risk_tier: 2
mode: "feature"
blast_radius:
  modules: ["auth", "user"]
  data_migration: false
operational_rollback_slo: "30m"
scope:
  in: ["src/auth/", "tests/auth/"]
  out: ["node_modules/"]
invariants:
  - "Passwords must be hashed"
  - "Sessions expire after 24h"
acceptance:
  - id: "A1"
    given: "A user is on the login page"
    when: "They enter valid credentials"
    then: "They should be redirected to dashboard"
non_functional:
  perf:
    api_p95_ms: 250
  security:
    - "input-validation"
contracts:
  - type: "openapi"
    path: "docs/api/auth.yaml"
```

## Usage Examples

### Complete Orchestration Workflow

```typescript
import { CAWSValidationAdapter } from "./src/caws-integration/adapters/CAWSValidationAdapter.js";
import { SpecFileManager } from "./src/caws-integration/utils/spec-file-manager.js";
import { BudgetMonitor } from "./src/monitoring/BudgetMonitor.js";
import { IterativeGuidance } from "./src/guidance/IterativeGuidance.js";
import { ProvenanceTracker } from "./src/provenance/ProvenanceTracker.js";

async function runCompleteWorkflow(projectRoot: string, spec: WorkingSpec) {
  // 1. Initialize components
  const specManager = new SpecFileManager({ projectRoot });
  const validator = new CAWSValidationAdapter({ projectRoot });
  const budgetMonitor = new BudgetMonitor({ projectRoot, spec });
  const guidance = new IterativeGuidance({ spec, projectRoot });
  const provenance = new ProvenanceTracker({ projectRoot, spec });

  // 2. Start monitoring
  await budgetMonitor.start();

  // 3. Validate spec
  await specManager.writeSpecFile(spec);
  const validation = await validator.validateExistingSpec();

  await provenance.recordEntry(
    "validation",
    spec.id,
    { type: "ai", identifier: "arbiter-validator" },
    { type: "validated", description: "CAWS validation completed" }
  );

  // 4. Get initial guidance
  const analysis = await guidance.analyzeProgress();
  console.log(
    `Initial progress: ${(analysis.summary?.overallProgress || 0) * 100}%`
  );

  // 5. Simulate development work
  // (In real usage, this would be done by AI agents)

  // 6. Monitor progress
  const status = budgetMonitor.getStatus();
  console.log(
    `Budget usage: ${status.currentUsage.filesPercentage}% files, ${status.currentUsage.locPercentage}% LOC`
  );

  // 7. Generate final guidance
  const finalAnalysis = await guidance.analyzeProgress();
  const recommendations = await guidance.getRecommendations();

  // 8. Generate provenance report
  const report = await provenance.generateReport(spec.id, "compliance");

  // 9. Cleanup
  await budgetMonitor.stop();

  return {
    validation,
    progress: finalAnalysis.summary?.overallProgress,
    recommendations,
    provenanceReport: report,
  };
}
```

### MCP Server Integration

```typescript
import { ArbiterMCPServer } from "./src/mcp-server/ArbiterMCPServer.js";

async function setupMCPServer(projectRoot: string) {
  const server = new ArbiterMCPServer(projectRoot);

  // The server automatically registers all ARBITER tools
  // Tools available:
  // - arbiter_validate
  // - arbiter_assign_task
  // - arbiter_monitor_progress
  // - arbiter_generate_verdict

  return server;
}

// Example MCP tool usage (from AI agent)
async function validateSpecWithMCP(
  server: ArbiterMCPServer,
  spec: WorkingSpec
) {
  const result = await server.handleArbiterValidate({
    spec,
    projectRoot: "/path/to/project",
    autoFix: true,
    suggestions: true,
  });

  if (result.success) {
    console.log("Validation result:", result.data);
  } else {
    console.error("Validation failed:", result.error);
  }
}
```

### Real-time Budget Monitoring

```typescript
import { BudgetMonitor } from "./src/monitoring/BudgetMonitor.js";

async function monitorProjectBudget(projectRoot: string, spec: WorkingSpec) {
  const monitor = new BudgetMonitor({
    projectRoot,
    spec,
    useFileWatching: true,
    pollingInterval: 2000, // Check every 2 seconds
  });

  // Set up alert handlers
  monitor.on("alert", (alert) => {
    switch (alert.severity) {
      case "warning":
        console.warn(`⚠️  Budget warning: ${alert.message}`);
        break;
      case "critical":
        console.error(`🚨 Budget critical: ${alert.message}`);
        break;
      case "exceeded":
        console.error(`💥 Budget exceeded: ${alert.message}`);
        break;
    }
  });

  monitor.on("status-update", (status) => {
    console.log(
      `📊 Budget: ${status.filesPercentage}% files, ${status.locPercentage}% LOC`
    );
  });

  await monitor.start();

  // Monitor for 5 minutes
  await new Promise((resolve) => setTimeout(resolve, 5 * 60 * 1000));

  const finalStatus = monitor.getStatus();
  console.log("Final budget status:", finalStatus);

  await monitor.stop();
}
```

## Integration Guides

### Integrating with Existing Projects

1. **Install Dependencies**

   ```bash
   npm install @paths.design/caws-cli chokidar js-yaml
   ```

2. **Setup Project Structure**

   ```
   your-project/
   ├── .caws/
   │   ├── policy.yaml
   │   └── working-spec.yaml
   ├── src/
   ├── tests/
   └── package.json
   ```

3. **Initialize ARBITER Components**

   ```typescript
   import {
     CAWSValidationAdapter,
     BudgetMonitor,
     IterativeGuidance,
   } from "arbiter-v2";

   // Initialize with your project
   const validator = new CAWSValidationAdapter({ projectRoot: process.cwd() });
   const monitor = new BudgetMonitor({
     projectRoot: process.cwd(),
     spec: yourSpec,
   });
   const guidance = new IterativeGuidance({
     spec: yourSpec,
     projectRoot: process.cwd(),
   });
   ```

4. **Configure Policies**
   Create `.caws/policy.yaml` with your risk tiers and budget limits.

### CI/CD Integration

```yaml
# .github/workflows/arbiter-validation.yml
name: ARBITER Validation
on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: "18"

      - name: Install dependencies
        run: npm ci

      - name: Run ARBITER validation
        run: npm run arbiter:validate

      - name: Check budget compliance
        run: npm run arbiter:check-budget

      - name: Generate provenance report
        run: npm run arbiter:provenance-report
```

### Custom MCP Tool Development

```typescript
import { ArbiterMCPServer } from "./src/mcp-server/ArbiterMCPServer.js";

class CustomArbiterMCPServer extends ArbiterMCPServer {
  // Add custom tools
  async handleCustomAnalysis(
    args: CustomAnalysisArgs
  ): Promise<MCPToolResponse> {
    // Implement custom analysis logic
    const result = await this.performCustomAnalysis(args);

    return {
      content: [
        {
          type: "text",
          text: JSON.stringify(result, null, 2),
        },
      ],
    };
  }
}
```

## Performance Considerations

### Optimization Tips

1. **Enable Caching**: Use policy and validation result caching
2. **Batch Operations**: Group file operations to reduce I/O overhead
3. **Debounced Monitoring**: Use debouncing for file watching events
4. **Selective Watching**: Only watch relevant file patterns

### Memory Management

- BudgetMonitor maintains file change history - monitor memory usage for large projects
- ProvenanceTracker accumulates entries - consider periodic archiving for long-running projects
- Guidance analysis scans file system - cache results for repeated analysis

### Scalability

- File watching scales well for projects up to 10k files
- Validation performance degrades with very large specs (>100 acceptance criteria)
- Provenance tracking is optimized for high-frequency entry recording

## Troubleshooting

### Common Issues

**Budget Monitor Not Detecting Changes**

- Ensure `useFileWatching: true` and chokidar is installed
- Check file paths are within monitored scope
- Verify file permissions allow watching

**CAWS Validation Fails**

- Ensure `.caws/policy.yaml` exists and is valid
- Check CAWS CLI is accessible
- Verify working spec format matches expected schema

**MCP Server Connection Issues**

- Confirm server is initialized before tool calls
- Check project root path is correct
- Verify all required dependencies are installed

**Guidance Analysis Incomplete**

- Ensure all acceptance criteria have proper IDs
- Check file system permissions for analysis
- Verify project structure matches scope definitions

**Provenance Chain Corruption**

- Check disk space and permissions
- Verify JSON serialization is working
- Consider enabling compression for large chains

### Debug Logging

Enable debug logging for detailed troubleshooting:

```typescript
import { setLogLevel } from "arbiter-v2";

// Enable debug logging
setLogLevel("debug");

// All components will now output detailed logs
const validator = new CAWSValidationAdapter({
  projectRoot: "/path/to/project",
  debug: true,
});
```

---

_This API reference covers ARBITER v2.0.0. For the latest documentation, visit the project repository._
