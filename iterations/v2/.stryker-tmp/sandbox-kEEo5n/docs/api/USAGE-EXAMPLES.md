# ARBITER v2 Usage Examples

This document provides practical examples of using ARBITER v2 components in real development scenarios.

## Table of Contents

- [Quick Start](#quick-start)
- [Basic Usage](#basic-usage)
- [Advanced Scenarios](#advanced-scenarios)
- [Integration Patterns](#integration-patterns)
- [Troubleshooting](#troubleshooting)

## Quick Start

### 1. Project Setup

```bash
# Create a new project
mkdir my-arbiter-project
cd my-arbiter-project
npm init -y

# Install ARBITER
npm install @paths.design/caws-cli chokidar js-yaml

# Create project structure
mkdir -p .caws src tests
```

### 2. Basic Configuration

Create `.caws/policy.yaml`:

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

Create `.caws/working-spec.yaml`:

```yaml
id: "QUICK-START-001"
title: "Quick Start Example"
risk_tier: 2
mode: "feature"
blast_radius:
  modules: ["example"]
  data_migration: false
operational_rollback_slo: "30m"
scope:
  in: ["src/", "tests/"]
  out: ["node_modules/"]
invariants:
  - "Example must be functional"
acceptance:
  - id: "A1"
    given: "User runs the example"
    when: "They follow the instructions"
    then: "It should work as expected"
non_functional:
  perf:
    api_p95_ms: 100
```

### 3. First ARBITER Script

Create `arbiter-example.js`:

```javascript
import { CAWSValidationAdapter } from "./node_modules/@paths.design/arbiter-v2/src/caws-integration/adapters/CAWSValidationAdapter.js";
import { BudgetMonitor } from "./node_modules/@paths.design/arbiter-v2/src/monitoring/BudgetMonitor.js";
import { SpecFileManager } from "./node_modules/@paths.design/arbiter-v2/src/caws-integration/utils/spec-file-manager.js";

async function quickStart() {
  console.log("🚀 ARBITER Quick Start Example\n");

  const projectRoot = process.cwd();

  // Load working spec
  const specManager = new SpecFileManager({ projectRoot });
  const spec = await specManager.readSpecFile();
  console.log(`📋 Loaded spec: ${spec.title}`);

  // Validate spec
  const validator = new CAWSValidationAdapter({ projectRoot });
  const validation = await validator.validateExistingSpec();

  if (validation.success && validation.data?.passed) {
    console.log("✅ Spec validation passed");
  } else {
    console.log("❌ Spec validation failed:", validation.error?.message);
    return;
  }

  // Start budget monitoring
  const monitor = new BudgetMonitor({
    projectRoot,
    spec,
    useFileWatching: true,
  });

  monitor.on("alert", (alert) => {
    console.log(`🚨 Budget Alert: ${alert.message}`);
  });

  await monitor.start();
  console.log("📊 Budget monitoring started");

  // Simulate some work
  console.log("🔧 Simulating development work...");
  await new Promise((resolve) => setTimeout(resolve, 2000));

  // Check status
  const status = monitor.getStatus();
  console.log(
    `📈 Current budget: ${status.currentUsage.filesChanged} files, ${status.currentUsage.linesChanged} LOC`
  );

  // Cleanup
  await monitor.stop();
  console.log("✅ Example completed successfully");
}

quickStart().catch(console.error);
```

Run it:

```bash
node arbiter-example.js
```

## Basic Usage

### Working with Specs

```javascript
import { SpecFileManager } from "./src/caws-integration/utils/spec-file-manager.js";

// Initialize
const specManager = new SpecFileManager({
  projectRoot: "/path/to/project",
  useTemporaryFiles: false,
});

// Create a new spec
const newSpec = {
  id: "EXAMPLE-001",
  title: "Example Feature",
  risk_tier: 2,
  mode: "feature",
  blast_radius: { modules: ["example"], data_migration: false },
  operational_rollback_slo: "30m",
  scope: { in: ["src/"], out: ["node_modules/"] },
  invariants: ["Must work correctly"],
  acceptance: [
    {
      id: "A1",
      given: "User uses feature",
      when: "They perform action",
      then: "Expected result occurs",
    },
  ],
};

// Save to file
await specManager.writeSpecFile(newSpec);

// Read from file
const loadedSpec = await specManager.readSpecFile();
console.log("Spec ID:", loadedSpec.id);
```

### Validation

```javascript
import { CAWSValidationAdapter } from "./src/caws-integration/adapters/CAWSValidationAdapter.js";

const validator = new CAWSValidationAdapter({
  projectRoot: "/path/to/project",
  arbiterVersion: "2.0.0",
});

// Validate spec file
const result = await validator.validateExistingSpec();

if (result.success) {
  console.log("Validation result:", result.data);
  if (result.data?.passed) {
    console.log("✅ Spec is valid");
  } else {
    console.log("❌ Spec has issues");
    if (result.data?.remediation) {
      console.log("Suggested fixes:", result.data.remediation);
    }
  }
} else {
  console.error("Validation failed:", result.error?.message);
}

// Validate spec object directly
const directResult = await validator.validateSpec({
  spec: myWorkingSpec,
  projectRoot: "/path/to/project",
  options: {
    autoFix: true,
    suggestions: true,
  },
});
```

### Budget Monitoring

```javascript
import { BudgetMonitor } from "./src/monitoring/BudgetMonitor.js";

const monitor = new BudgetMonitor({
  projectRoot: "/path/to/project",
  spec: workingSpec,
  useFileWatching: true,
  thresholds: {
    warning: 0.5, // Alert at 50%
    critical: 0.8, // Alert at 80%
    exceeded: 0.95, // Alert at 95%
  },
});

// Handle alerts
monitor.on("alert", (alert) => {
  const emoji =
    {
      warning: "⚠️",
      critical: "🚨",
      exceeded: "💥",
    }[alert.severity] || "❓";

  console.log(`${emoji} Budget Alert: ${alert.message}`);
  console.log(
    `   Current: ${alert.currentUsage.filesChanged}/${alert.currentUsage.maxFiles} files`
  );
  console.log(
    `   Usage: ${(alert.currentUsage.filesPercentage * 100).toFixed(1)}%`
  );
});

// Handle status updates
monitor.on("status-update", (status) => {
  console.log(
    `📊 Budget Status: ${status.filesPercentage}% files, ${status.locPercentage}% LOC`
  );
});

// Start monitoring
await monitor.start();

// Do some work...
await simulateDevelopmentWork();

// Check current status
const status = monitor.getStatus();
console.log("Current budget usage:", {
  files: `${status.currentUsage.filesChanged}/${status.currentUsage.maxFiles}`,
  loc: `${status.currentUsage.linesChanged}/${status.currentUsage.maxLoc}`,
  filesPercent: `${(status.currentUsage.filesPercentage * 100).toFixed(1)}%`,
  locPercent: `${(status.currentUsage.locPercentage * 100).toFixed(1)}%`,
});

// Stop monitoring
await monitor.stop();
```

### Iterative Guidance

```javascript
import { IterativeGuidance } from "./src/guidance/IterativeGuidance.js";

const guidance = new IterativeGuidance(
  {
    spec: workingSpec,
    projectRoot: "/path/to/project",
  },
  {
    phase: "implementation",
    teamSize: 2,
    experienceLevel: "senior",
    timePressure: "medium",
  }
);

// Analyze current progress
const analysis = await guidance.analyzeProgress();

console.log("📊 Progress Analysis:");
console.log(
  `Overall Progress: ${(analysis.summary?.overallProgress || 0) * 100}%`
);

if (analysis.summary?.acceptanceCriteria) {
  console.log("Acceptance Criteria:");
  analysis.summary.acceptanceCriteria.forEach((criterion) => {
    const status = criterion.status.toUpperCase();
    console.log(`  ${criterion.id}: ${status}`);
  });
}

if (analysis.summary?.nextSteps) {
  console.log("Next Steps:");
  analysis.summary.nextSteps.slice(0, 3).forEach((step) => {
    console.log(`  ${step.priority}: ${step.description}`);
  });
}

// Get detailed guidance for next step
if (analysis.summary?.nextSteps?.length > 0) {
  const stepGuidance = await guidance.getStepGuidance(0);
  console.log(`\n🎯 Current Step: ${stepGuidance.step?.title}`);
  console.log(`Description: ${stepGuidance.step?.description}`);
  console.log(`Estimated effort: ${stepGuidance.effort?.hours}h`);
}

// Get recommendations
const recommendations = await guidance.getRecommendations();
console.log("\n💡 Recommendations:");
recommendations.slice(0, 3).forEach((rec) => {
  console.log(`  ${rec.type}: ${rec.message}`);
});
```

### Provenance Tracking

```javascript
import { ProvenanceTracker } from "./src/provenance/ProvenanceTracker.js";

const tracker = new ProvenanceTracker({
  projectRoot: "/path/to/project",
  spec: workingSpec,
  enableAIAttribution: true,
});

// Record various types of changes
await tracker.recordEntry(
  "commit",
  workingSpec.id,
  { type: "human", identifier: "developer-1" },
  {
    type: "implemented",
    description: "Implemented user authentication",
    details: { feature: "auth", complexity: "moderate" },
  },
  {
    affectedFiles: [
      { path: "src/auth/login.ts", changeType: "added", linesChanged: 45 },
      {
        path: "tests/auth/login.test.ts",
        changeType: "added",
        linesChanged: 30,
      },
    ],
  }
);

await tracker.recordEntry(
  "validation",
  workingSpec.id,
  { type: "ai", identifier: "arbiter-validator" },
  {
    type: "validated",
    description: "CAWS validation completed",
    details: { passed: true, duration: 150 },
  }
);

// Get provenance chain
const chain = await tracker.getProvenanceChain(workingSpec.id);
console.log(`Total provenance entries: ${chain?.entries.length}`);

if (chain?.entries) {
  console.log("Recent entries:");
  chain.entries.slice(-3).forEach((entry) => {
    console.log(`  ${entry.action.type}: ${entry.action.description}`);
    console.log(`    By: ${entry.actor.type}/${entry.actor.identifier}`);
  });
}

// Generate reports
const summaryReport = await tracker.generateReport(workingSpec.id, "summary");
const complianceReport = await tracker.generateReport(
  workingSpec.id,
  "compliance"
);

console.log("Compliance Status:", {
  cawsCompliant: complianceReport.compliance.cawsCompliant,
  totalEntries: complianceReport.provenanceChain.entries.length,
  aiContributions: complianceReport.aiStats.total,
});

// Get AI attribution statistics
const aiStats = await tracker.getAIAttributionStats();
console.log("AI Attribution:", {
  total: aiStats.total,
  byTool: aiStats.byTool,
  percentage: `${((aiStats.total / (chain?.entries.length || 1)) * 100).toFixed(
    1
  )}%`,
});
```

## Advanced Scenarios

### Multi-Agent Collaboration

```javascript
import { ArbiterMCPServer } from "./src/mcp-server/ArbiterMCPServer.js";

class CollaborativeDevelopment {
  constructor(projectRoot, workingSpec) {
    this.projectRoot = projectRoot;
    this.spec = workingSpec;
    this.mcpServer = new ArbiterMCPServer(projectRoot);
    this.activeAgents = new Map();
  }

  async initialize() {
    await this.mcpServer.initialize();
    console.log("🤝 Collaborative development session started");
  }

  async assignTaskToAgent(agentId, taskDescription, acceptanceCriteria) {
    // Use MCP to assign task
    const assignResult = await this.mcpServer.handleAssignTask({
      spec: this.spec,
      availableAgents: [agentId],
      strategy: "capability",
      priority: "high",
      orchestrationContext: {
        taskId: `task-${Date.now()}`,
        agentId,
      },
    });

    if (assignResult.success) {
      this.activeAgents.set(agentId, {
        taskId: assignResult.data?.taskId,
        assignedAt: new Date(),
        status: "active",
      });

      console.log(`✅ Task assigned to ${agentId}: ${taskDescription}`);
      return assignResult.data?.taskId;
    } else {
      console.error(`❌ Failed to assign task to ${agentId}`);
      return null;
    }
  }

  async monitorAgentProgress(agentId) {
    const agentInfo = this.activeAgents.get(agentId);
    if (!agentInfo) return null;

    const progressResult = await this.mcpServer.handleMonitorProgress({
      taskId: agentInfo.taskId,
      detailed: true,
      thresholds: { warning: 0.7, critical: 0.9 },
    });

    if (progressResult.success) {
      const progress = progressResult.data;
      console.log(`📊 ${agentId} Progress:`, {
        overallProgress: progress?.overallProgress,
        budgetUsage: progress?.budgetUsage,
        blockers: progress?.blockers?.length || 0,
      });

      return progress;
    }

    return null;
  }

  async generateFinalVerdict(taskId) {
    const verdictResult = await this.mcpServer.handleGenerateVerdict({
      taskId,
      spec: this.spec,
      criteria: ["budget-compliance", "acceptance-criteria", "quality-gates"],
    });

    if (verdictResult.success) {
      const verdict = verdictResult.data?.verdict;
      console.log(`🎯 Final Verdict for ${taskId}: ${verdict}`);

      if (verdict === "approved") {
        console.log("✅ Task completed successfully");
      } else if (verdict === "conditional") {
        console.log("⚠️ Task completed with conditions");
      } else {
        console.log("❌ Task needs revision");
      }

      return verdict;
    }

    return null;
  }

  async endSession() {
    // Generate final reports for all active agents
    for (const [agentId, agentInfo] of this.activeAgents) {
      if (agentInfo.status === "active") {
        await this.generateFinalVerdict(agentInfo.taskId);
      }
    }

    console.log("👋 Collaborative development session ended");
  }
}

// Usage
const collaboration = new CollaborativeDevelopment(
  "/path/to/project",
  workingSpec
);
await collaboration.initialize();

// Assign tasks to multiple agents
await collaboration.assignTaskToAgent(
  "cursor-composer",
  "Implement authentication",
  ["A1", "A2"]
);
await collaboration.assignTaskToAgent(
  "copilot-agent",
  "Implement user management",
  ["A3", "A4"]
);

// Monitor progress
setInterval(async () => {
  await collaboration.monitorAgentProgress("cursor-composer");
  await collaboration.monitorAgentProgress("copilot-agent");
}, 30000); // Every 30 seconds

// End session after some time
setTimeout(async () => {
  await collaboration.endSession();
}, 10 * 60 * 1000); // After 10 minutes
```

### Continuous Integration Pipeline

```javascript
import { CAWSValidationAdapter } from "./src/caws-integration/adapters/CAWSValidationAdapter.js";
import { BudgetMonitor } from "./src/monitoring/BudgetMonitor.js";
import { ProvenanceTracker } from "./src/provenance/ProvenanceTracker.js";

class CIPipeline {
  constructor(projectRoot, workingSpec) {
    this.projectRoot = projectRoot;
    this.spec = workingSpec;
    this.validator = new CAWSValidationAdapter({ projectRoot });
    this.monitor = new BudgetMonitor({ projectRoot, spec: workingSpec });
    this.tracker = new ProvenanceTracker({ projectRoot, spec: workingSpec });
  }

  async runValidationStage() {
    console.log("🔍 CI: Validation Stage");

    const validation = await this.validator.validateExistingSpec();

    await this.tracker.recordEntry(
      "validation",
      this.spec.id,
      { type: "ai", identifier: "ci-pipeline" },
      {
        type: "validated",
        description: "CI validation completed",
        details: {
          passed: validation.success && validation.data?.passed,
          duration: validation.durationMs,
        },
      }
    );

    if (!validation.success || !validation.data?.passed) {
      console.error("❌ CI: Validation failed");
      throw new Error("Spec validation failed");
    }

    console.log("✅ CI: Validation passed");
    return validation;
  }

  async runBudgetCheckStage() {
    console.log("📊 CI: Budget Check Stage");

    await this.monitor.start();

    // Simulate CI running tests and builds
    await this.simulateCIWork();

    const status = this.monitor.getStatus();

    await this.tracker.recordEntry(
      "quality_gate",
      this.spec.id,
      { type: "ai", identifier: "ci-pipeline" },
      {
        type: "evaluated",
        description: "Budget compliance checked",
        details: {
          filesUsed: status.currentUsage.filesChanged,
          filesLimit: status.currentUsage.maxFiles,
          locUsed: status.currentUsage.linesChanged,
          locLimit: status.currentUsage.maxLoc,
          compliant:
            status.currentUsage.filesPercentage <= 1.0 &&
            status.currentUsage.locPercentage <= 1.0,
        },
      }
    );

    await this.monitor.stop();

    if (
      status.currentUsage.filesPercentage > 1.0 ||
      status.currentUsage.locPercentage > 1.0
    ) {
      console.error("❌ CI: Budget exceeded");
      throw new Error("Budget compliance failed");
    }

    console.log("✅ CI: Budget check passed");
    return status;
  }

  async runTestStage() {
    console.log("🧪 CI: Test Stage");

    // Run actual tests (would integrate with Jest, etc.)
    const testResults = await this.runTests();

    await this.tracker.recordEntry(
      "validation",
      this.spec.id,
      { type: "ai", identifier: "ci-pipeline" },
      {
        type: "tested",
        description: "Test suite completed",
        details: {
          passed: testResults.passed,
          totalTests: testResults.total,
          failedTests: testResults.failed,
          coverage: testResults.coverage,
        },
      }
    );

    if (!testResults.passed) {
      console.error("❌ CI: Tests failed");
      throw new Error("Test suite failed");
    }

    console.log("✅ CI: Tests passed");
    return testResults;
  }

  async runDeploymentStage() {
    console.log("🚀 CI: Deployment Stage");

    // Simulate deployment
    const deployResult = await this.deployToStaging();

    await this.tracker.recordEntry(
      "commit",
      this.spec.id,
      { type: "ai", identifier: "ci-pipeline" },
      {
        type: "deployed",
        description: "Deployed to staging environment",
        details: {
          environment: "staging",
          success: deployResult.success,
          url: deployResult.url,
        },
      }
    );

    if (!deployResult.success) {
      console.error("❌ CI: Deployment failed");
      throw new Error("Deployment failed");
    }

    console.log("✅ CI: Deployment completed");
    return deployResult;
  }

  async runFullPipeline() {
    try {
      console.log("🚀 Starting CI Pipeline for:", this.spec.title);

      await this.runValidationStage();
      await this.runBudgetCheckStage();
      await this.runTestStage();
      await this.runDeploymentStage();

      // Generate final report
      const report = await this.tracker.generateReport(
        this.spec.id,
        "compliance"
      );

      console.log("🎉 CI Pipeline completed successfully!");
      console.log("📋 Final Report:", {
        compliant: report.compliance.cawsCompliant,
        totalEntries: report.provenanceChain.entries.length,
        aiContributions: report.aiStats.total,
      });

      return report;
    } catch (error) {
      console.error("💥 CI Pipeline failed:", error.message);

      // Record failure
      await this.tracker.recordEntry(
        "quality_gate",
        this.spec.id,
        { type: "ai", identifier: "ci-pipeline" },
        {
          type: "failed",
          description: `CI Pipeline failed: ${error.message}`,
          details: { error: error.message, stage: error.stage },
        }
      );

      throw error;
    }
  }

  // Helper methods (would be implemented based on actual CI setup)
  async simulateCIWork() {
    // Simulate CI workload
    return new Promise((resolve) => setTimeout(resolve, 1000));
  }

  async runTests() {
    // Would integrate with actual test runner
    return {
      passed: true,
      total: 42,
      failed: 0,
      coverage: 0.87,
    };
  }

  async deployToStaging() {
    // Would integrate with actual deployment system
    return {
      success: true,
      url: "https://staging.example.com",
    };
  }
}

// Usage in CI
const pipeline = new CIPipeline("/path/to/project", workingSpec);
await pipeline.runFullPipeline();
```

### Real-time Development Dashboard

```javascript
import { BudgetMonitor } from "./src/monitoring/BudgetMonitor.js";
import { IterativeGuidance } from "./src/guidance/IterativeGuidance.js";
import { ProvenanceTracker } from "./src/provenance/ProvenanceTracker.js";

class DevelopmentDashboard {
  constructor(projectRoot, workingSpec) {
    this.projectRoot = projectRoot;
    this.spec = workingSpec;
    this.monitor = new BudgetMonitor({ projectRoot, spec: workingSpec });
    this.guidance = new IterativeGuidance({ spec: workingSpec, projectRoot });
    this.tracker = new ProvenanceTracker({ projectRoot, spec: workingSpec });
    this.dashboardData = {};
  }

  async initialize() {
    await this.monitor.start();
    await this.updateDashboard();
    console.log("📊 Development Dashboard initialized");
  }

  async updateDashboard() {
    // Gather all dashboard data
    const [budgetStatus, progressAnalysis, provenanceChain, aiStats] =
      await Promise.all([
        this.monitor.getStatus(),
        this.guidance.analyzeProgress(),
        this.tracker.getProvenanceChain(this.spec.id),
        this.tracker.getAIAttributionStats(),
      ]);

    this.dashboardData = {
      timestamp: new Date().toISOString(),
      budget: {
        files: {
          used: budgetStatus.currentUsage.filesChanged,
          limit: budgetStatus.currentUsage.maxFiles,
          percentage: budgetStatus.currentUsage.filesPercentage,
        },
        loc: {
          used: budgetStatus.currentUsage.linesChanged,
          limit: budgetStatus.currentUsage.maxLoc,
          percentage: budgetStatus.currentUsage.locPercentage,
        },
        alerts: budgetStatus.alerts || [],
      },
      progress: {
        overall: progressAnalysis.summary?.overallProgress || 0,
        acceptanceCriteria: progressAnalysis.summary?.acceptanceCriteria || [],
        nextSteps: progressAnalysis.summary?.nextSteps?.slice(0, 5) || [],
        blockers: progressAnalysis.summary?.blockers || [],
      },
      provenance: {
        totalEntries: provenanceChain?.entries.length || 0,
        recentEntries: provenanceChain?.entries.slice(-10) || [],
        aiContributions: aiStats.total,
        aiPercentage:
          (aiStats.total / (provenanceChain?.entries.length || 1)) * 100,
      },
    };

    return this.dashboardData;
  }

  displayDashboard() {
    const data = this.dashboardData;
    console.clear();
    console.log("🚀 ARBITER Development Dashboard");
    console.log("================================");
    console.log(`Updated: ${new Date(data.timestamp).toLocaleTimeString()}`);
    console.log("");

    // Budget Section
    console.log("📊 Budget Usage:");
    console.log(
      `  Files: ${data.budget.files.used}/${data.budget.files.limit} (${(
        data.budget.files.percentage * 100
      ).toFixed(1)}%)`
    );
    console.log(
      `  LOC:   ${data.budget.loc.used}/${data.budget.loc.limit} (${(
        data.budget.loc.percentage * 100
      ).toFixed(1)}%)`
    );

    if (data.budget.alerts.length > 0) {
      console.log("  🚨 Alerts:");
      data.budget.alerts.forEach((alert) => {
        console.log(`    ${alert.severity}: ${alert.message}`);
      });
    }
    console.log("");

    // Progress Section
    console.log("📈 Progress:");
    console.log(`  Overall: ${(data.progress.overall * 100).toFixed(1)}%`);

    const completed = data.progress.acceptanceCriteria.filter(
      (c) => c.status === "completed"
    ).length;
    const total = data.progress.acceptanceCriteria.length;
    console.log(`  Acceptance Criteria: ${completed}/${total} completed`);

    if (data.progress.nextSteps.length > 0) {
      console.log("  Next Steps:");
      data.progress.nextSteps.forEach((step) => {
        console.log(`    ${step.priority}: ${step.description}`);
      });
    }

    if (data.progress.blockers.length > 0) {
      console.log("  🚫 Blockers:");
      data.progress.blockers.forEach((blocker) => {
        console.log(`    ${blocker.description}`);
      });
    }
    console.log("");

    // Provenance Section
    console.log("📋 Activity:");
    console.log(`  Total Entries: ${data.provenance.totalEntries}`);
    console.log(
      `  AI Contributions: ${
        data.provenance.aiContributions
      } (${data.provenance.aiPercentage.toFixed(1)}%)`
    );

    if (data.provenance.recentEntries.length > 0) {
      console.log("  Recent Activity:");
      data.provenance.recentEntries.slice(-5).forEach((entry) => {
        const time = new Date(entry.timestamp).toLocaleTimeString();
        console.log(
          `    ${time} - ${entry.action.type}: ${entry.action.description}`
        );
      });
    }
  }

  async startRealTimeUpdates(intervalMs = 5000) {
    this.updateInterval = setInterval(async () => {
      await this.updateDashboard();
      this.displayDashboard();
    }, intervalMs);
  }

  async stopRealTimeUpdates() {
    if (this.updateInterval) {
      clearInterval(this.updateInterval);
      this.updateInterval = null;
    }
    await this.monitor.stop();
  }

  async getDashboardData() {
    return this.dashboardData;
  }
}

// Usage
const dashboard = new DevelopmentDashboard("/path/to/project", workingSpec);
await dashboard.initialize();

// Start real-time updates
await dashboard.startRealTimeUpdates(3000); // Update every 3 seconds

// Dashboard will continuously update in terminal
// Press Ctrl+C to stop
process.on("SIGINT", async () => {
  await dashboard.stopRealTimeUpdates();
  console.log("📊 Dashboard stopped");
  process.exit(0);
});
```

## Integration Patterns

### Express.js Middleware

```javascript
import express from "express";
import { BudgetMonitor } from "./src/monitoring/BudgetMonitor.js";
import { ProvenanceTracker } from "./src/provenance/ProvenanceTracker.js";

const app = express();

// ARBITER middleware
app.use(async (req, res, next) => {
  const startTime = Date.now();

  // Add ARBITER context to request
  req.arbiter = {
    startTime,
    projectRoot: process.cwd(),
    userId: req.headers["x-user-id"] || "anonymous",
  };

  // Continue with request
  res.on("finish", async () => {
    const duration = Date.now() - startTime;

    // Record API usage in provenance
    if (req.arbiter.tracker) {
      await req.arbiter.tracker.recordEntry(
        "validation",
        req.arbiter.specId || "api-request",
        { type: "human", identifier: req.arbiter.userId },
        {
          type: "api_call",
          description: `${req.method} ${req.path}`,
          details: {
            method: req.method,
            path: req.path,
            statusCode: res.statusCode,
            duration,
            userAgent: req.headers["user-agent"],
          },
        }
      );
    }
  });

  next();
});

// Initialize ARBITER for API routes
async function initializeArbiterForAPI(projectRoot, spec) {
  const monitor = new BudgetMonitor({ projectRoot, spec });
  const tracker = new ProvenanceTracker({ projectRoot, spec });

  await monitor.start();

  // Make available to middleware
  app.locals.arbiter = { monitor, tracker, spec: spec.id };

  return { monitor, tracker };
}

// Example protected route
app.post("/api/features", async (req, res) => {
  try {
    const { monitor, tracker } = app.locals.arbiter;

    // Check budget before allowing new feature
    const status = monitor.getStatus();
    if (status.currentUsage.filesPercentage > 0.9) {
      return res.status(429).json({
        error: "Budget limit exceeded",
        currentUsage: status.currentUsage,
      });
    }

    // Process feature creation
    const feature = await createFeature(req.body);

    // Record in provenance
    await tracker.recordEntry(
      "commit",
      app.locals.arbiter.spec,
      { type: "human", identifier: req.arbiter.userId },
      {
        type: "implemented",
        description: `Created feature: ${feature.name}`,
        details: feature,
      }
    );

    res.json(feature);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// Budget status endpoint
app.get("/api/budget", (req, res) => {
  const { monitor } = app.locals.arbiter;
  const status = monitor.getStatus();
  res.json(status);
});

// Guidance endpoint
app.get("/api/guidance", async (req, res) => {
  const { tracker } = app.locals.arbiter;

  // Would need IterativeGuidance instance
  const guidance = new IterativeGuidance({
    spec: await loadSpecFromSomewhere(),
    projectRoot: process.cwd(),
  });

  const analysis = await guidance.analyzeProgress();
  res.json(analysis.summary);
});

export { app, initializeArbiterForAPI };
```

### VS Code Extension Integration

```typescript
import * as vscode from "vscode";
import { CAWSValidationAdapter } from "./src/caws-integration/adapters/CAWSValidationAdapter.js";
import { BudgetMonitor } from "./src/monitoring/BudgetMonitor.js";
import { IterativeGuidance } from "./src/guidance/IterativeGuidance.js";

export class ArbiterVscodeExtension {
  private validator: CAWSValidationAdapter | null = null;
  private monitor: BudgetMonitor | null = null;
  private guidance: IterativeGuidance | null = null;
  private statusBarItem: vscode.StatusBarItem;

  constructor() {
    this.statusBarItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Right,
      100
    );
    this.statusBarItem.command = "arbiter.showDashboard";
  }

  async activate(context: vscode.ExtensionContext) {
    console.log("ARBITER VS Code Extension activated");

    // Register commands
    context.subscriptions.push(
      vscode.commands.registerCommand(
        "arbiter.validate",
        this.validateSpec.bind(this)
      ),
      vscode.commands.registerCommand(
        "arbiter.showGuidance",
        this.showGuidance.bind(this)
      ),
      vscode.commands.registerCommand(
        "arbiter.showBudget",
        this.showBudget.bind(this)
      ),
      vscode.commands.registerCommand(
        "arbiter.showDashboard",
        this.showDashboard.bind(this)
      )
    );

    // Initialize ARBITER components
    await this.initializeArbiter();

    // Start status bar
    this.statusBarItem.show();
    this.updateStatusBar();

    // Watch for file changes
    const watcher = vscode.workspace.createFileSystemWatcher(
      "**/*.{ts,tsx,js,jsx}"
    );
    watcher.onDidChange(() => this.updateStatusBar());
    watcher.onDidCreate(() => this.updateStatusBar());
    watcher.onDidDelete(() => this.updateStatusBar());

    context.subscriptions.push(watcher);
  }

  private async initializeArbiter() {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) return;

    const projectRoot = workspaceFolder.uri.fsPath;

    try {
      // Load working spec
      const specPath = vscode.Uri.joinPath(
        workspaceFolder.uri,
        ".caws",
        "working-spec.yaml"
      );
      const specContent = await vscode.workspace.fs.readFile(specPath);
      const spec = YAML.parse(specContent.toString());

      // Initialize components
      this.validator = new CAWSValidationAdapter({ projectRoot });
      this.monitor = new BudgetMonitor({ projectRoot, spec });
      this.guidance = new IterativeGuidance({ spec, projectRoot });

      await this.monitor.start();

      // Set up event handlers
      this.monitor.on("alert", (alert) => {
        const message = `ARBITER Budget Alert: ${alert.message}`;
        vscode.window.showWarningMessage(message);
      });
    } catch (error) {
      vscode.window.showErrorMessage(
        `ARBITER initialization failed: ${error.message}`
      );
    }
  }

  private async validateSpec() {
    if (!this.validator) {
      vscode.window.showErrorMessage("ARBITER not initialized");
      return;
    }

    try {
      const result = await this.validator.validateExistingSpec();

      if (result.success && result.data?.passed) {
        vscode.window.showInformationMessage("✅ Spec validation passed");
      } else {
        const message =
          result.data?.remediation?.join("\n") || "Validation failed";
        vscode.window.showErrorMessage(
          `❌ Spec validation failed:\n${message}`
        );
      }
    } catch (error) {
      vscode.window.showErrorMessage(`Validation error: ${error.message}`);
    }
  }

  private async showGuidance() {
    if (!this.guidance) {
      vscode.window.showErrorMessage("ARBITER not initialized");
      return;
    }

    try {
      const analysis = await this.guidance.analyzeProgress();
      const panel = vscode.window.createWebviewPanel(
        "arbiterGuidance",
        "ARBITER Guidance",
        vscode.ViewColumn.One,
        {}
      );

      const html = this.generateGuidanceHtml(analysis);
      panel.webview.html = html;
    } catch (error) {
      vscode.window.showErrorMessage(`Guidance error: ${error.message}`);
    }
  }

  private async showBudget() {
    if (!this.monitor) {
      vscode.window.showErrorMessage("ARBITER not initialized");
      return;
    }

    const status = this.monitor.getStatus();
    const message = `Budget Usage:
Files: ${status.currentUsage.filesChanged}/${status.currentUsage.maxFiles} (${(
      status.currentUsage.filesPercentage * 100
    ).toFixed(1)}%)
LOC: ${status.currentUsage.linesChanged}/${status.currentUsage.maxLoc} (${(
      status.currentUsage.locPercentage * 100
    ).toFixed(1)}%)`;

    vscode.window.showInformationMessage(message);
  }

  private async showDashboard() {
    if (!this.monitor || !this.guidance) {
      vscode.window.showErrorMessage("ARBITER not initialized");
      return;
    }

    const [budgetStatus, progressAnalysis] = await Promise.all([
      this.monitor.getStatus(),
      this.guidance.analyzeProgress(),
    ]);

    const panel = vscode.window.createWebviewPanel(
      "arbiterDashboard",
      "ARBITER Dashboard",
      vscode.ViewColumn.One,
      { enableScripts: true }
    );

    const html = this.generateDashboardHtml(budgetStatus, progressAnalysis);
    panel.webview.html = html;
  }

  private updateStatusBar() {
    if (!this.monitor) return;

    const status = this.monitor.getStatus();
    const filesPercent = (status.currentUsage.filesPercentage * 100).toFixed(0);
    const locPercent = (status.currentUsage.locPercentage * 100).toFixed(0);

    this.statusBarItem.text = `$(check) ARBITER: ${filesPercent}%/${locPercent}%`;
    this.statusBarItem.tooltip = `Files: ${status.currentUsage.filesChanged}/${status.currentUsage.maxFiles}\nLOC: ${status.currentUsage.linesChanged}/${status.currentUsage.maxLoc}`;
  }

  private generateGuidanceHtml(analysis: any): string {
    // Generate HTML for guidance display
    return `
      <!DOCTYPE html>
      <html>
        <head>
          <title>ARBITER Guidance</title>
          <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            .progress { font-size: 24px; font-weight: bold; margin: 10px 0; }
            .step { margin: 10px 0; padding: 10px; border-left: 4px solid #007acc; }
            .priority-critical { border-left-color: #f48771; }
            .priority-high { border-left-color: #ffa500; }
            .priority-medium { border-left-color: #ffd700; }
          </style>
        </head>
        <body>
          <h1>ARBITER Development Guidance</h1>
          <div class="progress">
            Overall Progress: ${(
              analysis.summary?.overallProgress * 100 || 0
            ).toFixed(1)}%
          </div>

          <h2>Next Steps</h2>
          ${
            analysis.summary?.nextSteps
              ?.map(
                (step) => `
            <div class="step priority-${step.priority}">
              <strong>${step.priority.toUpperCase()}</strong>: ${
                  step.description
                }
            </div>
          `
              )
              .join("") || "<p>No next steps available</p>"
          }
        </body>
      </html>
    `;
  }

  private generateDashboardHtml(
    budgetStatus: any,
    progressAnalysis: any
  ): string {
    // Generate HTML for dashboard display
    return `
      <!DOCTYPE html>
      <html>
        <head>
          <title>ARBITER Dashboard</title>
          <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            .metric { display: inline-block; margin: 10px; padding: 20px; border: 1px solid #ccc; border-radius: 5px; }
            .metric h3 { margin: 0 0 10px 0; }
            .metric .value { font-size: 24px; font-weight: bold; }
            .progress-bar { width: 100%; height: 20px; background: #eee; border-radius: 10px; overflow: hidden; }
            .progress-fill { height: 100%; background: #007acc; }
          </style>
        </head>
        <body>
          <h1>ARBITER Development Dashboard</h1>

          <div class="metric">
            <h3>Files Budget</h3>
            <div class="value">${budgetStatus.currentUsage.filesChanged}/${
      budgetStatus.currentUsage.maxFiles
    }</div>
            <div class="progress-bar">
              <div class="progress-fill" style="width: ${
                budgetStatus.currentUsage.filesPercentage * 100
              }%"></div>
            </div>
          </div>

          <div class="metric">
            <h3>LOC Budget</h3>
            <div class="value">${budgetStatus.currentUsage.linesChanged}/${
      budgetStatus.currentUsage.maxLoc
    }</div>
            <div class="progress-bar">
              <div class="progress-fill" style="width: ${
                budgetStatus.currentUsage.locPercentage * 100
              }%"></div>
            </div>
          </div>

          <div class="metric">
            <h3>Overall Progress</h3>
            <div class="value">${(
              progressAnalysis.summary?.overallProgress * 100 || 0
            ).toFixed(1)}%</div>
            <div class="progress-bar">
              <div class="progress-fill" style="width: ${
                progressAnalysis.summary?.overallProgress * 100 || 0
              }%"></div>
            </div>
          </div>
        </body>
      </html>
    `;
  }
}
```

## Troubleshooting

### Common Issues and Solutions

#### "CAWS CLI not found"

```bash
# Ensure CAWS CLI is installed
npm list @paths.design/caws-cli

# If not installed
npm install @paths.design/caws-cli

# Check PATH
which caws
```

#### "Policy file not found"

```bash
# Check if .caws directory exists
ls -la .caws/

# Create policy file if missing
mkdir -p .caws
cat > .caws/policy.yaml << 'EOF'
version: "1.0.0"
risk_tiers:
  2:
    max_files: 20
    max_loc: 500
    coverage_threshold: 0.80
    mutation_threshold: 0.70
    contracts_required: true
    manual_review_required: false
EOF
```

#### "Budget monitor not detecting changes"

```bash
# Check if file watching is enabled
const monitor = new BudgetMonitor({
  projectRoot: '/path/to/project',
  spec: workingSpec,
  useFileWatching: true,  // Ensure this is true
});

// Test file watching
monitor.on('file-changed', (change) => {
  console.log('File changed:', change);
});

// Manually trigger change
fs.writeFileSync('test.txt', 'test');
```

#### "Guidance analysis fails"

```bash
# Check project structure
find . -name "*.ts" -o -name "*.js" | head -10

# Ensure acceptance criteria have proper IDs
const spec = await specManager.readSpecFile();
spec.acceptance.forEach(criterion => {
  console.log(`Criterion: ${criterion.id} - ${criterion.given}`);
});
```

#### "Provenance chain corrupted"

```bash
# Clear and restart provenance tracking
rm -f .caws/provenance.json
const tracker = new ProvenanceTracker({ projectRoot, spec });
await tracker.initialize(); // If available
```

#### "MCP server connection refused"

```bash
# Check if server is running
netstat -tlnp | grep :3000

# Start MCP server
const server = new ArbiterMCPServer('/path/to/project');
await server.initialize();
await server.start(3000);
```

#### Performance Issues

**High memory usage:**

```typescript
// Enable streaming for large datasets
const guidance = new IterativeGuidance({
  spec: workingSpec,
  projectRoot,
  enableStreaming: true,
  batchSize: 10,
});

// Use pagination for large provenance chains
const chain = await tracker.getProvenanceChain(specId, {
  limit: 100,
  offset: 0,
});
```

**Slow file watching:**

```typescript
const monitor = new BudgetMonitor({
  projectRoot,
  spec,
  pollingInterval: 2000, // Increase polling interval
  ignorePatterns: [
    // Add more ignore patterns
    "**/*.log",
    "**/tmp/**",
    "**/cache/**",
  ],
});
```

---

_These examples demonstrate ARBITER v2 usage patterns. For the latest documentation, visit the project repository._
