# Benchmark Data Schema

**Author**: @darianrosebrook

---

## Executive Summary

This document defines the complete data schema for benchmark data collection, ensuring consistent structure across the arbiter orchestration and RL training pipeline.

**Design Principle**: Every field serves either immediate orchestration needs or future RL training requirements.

---

## Core Data Types

### BenchmarkDataPoint (Complete Structure)

```typescript
/**
 * Complete benchmark data point capturing all information from task execution
 *
 * Used by:
 * - Arbiter for routing decisions (performance history)
 * - RL training for supervised and reinforcement learning
 * - Analytics for system monitoring and improvement
 */
interface BenchmarkDataPoint {
  // ============================================
  // IDENTITY & CONTEXT
  // ============================================

  /** Unique identifier for this data point */
  id: string;

  /** When this task was executed */
  timestamp: Date;

  /** Anonymized tenant identifier (hashed) */
  tenantId: string;

  // ============================================
  // TASK CONTEXT
  // ============================================

  task: {
    /** Task identifier */
    id: string;

    /** Task classification */
    type: "code-editing" | "research" | "data-analysis" | "design" | "planning";

    /** Complexity assessment */
    complexity: "trivial" | "standard" | "complex";

    /** Required capabilities */
    requirements: string[]; // ["TypeScript", "AST analysis", "Testing"]

    /** Task description (anonymized) */
    description: string;

    /** Working spec reference */
    cawsSpecId?: string;
  };

  // ============================================
  // ROUTING CONTEXT
  // ============================================

  routing: {
    /** Which agent was selected */
    selectedAgent: {
      id: string;
      name: string;
      modelFamily: string; // "GPT-4", "Claude-3", "Gemma-2", etc.
    };

    /** Routing strategy used */
    strategy:
      | "multi-armed-bandit"
      | "capability-match"
      | "load-balance"
      | "fallback";

    /** Confidence in routing decision */
    confidence: number; // 0-1

    /** Why this agent was chosen */
    rationale: string;

    /** Other agents that were considered */
    alternativesConsidered: Array<{
      agentId: string;
      score: number;
      reason: string;
    }>;

    /** Historical context at time of routing */
    historicalPerformance: {
      agentSuccessRate: number; // 0-1
      agentAvgQuality: number; // 0-1
      agentTaskCount: number;
    };
  };

  // ============================================
  // EXECUTION METRICS
  // ============================================

  execution: {
    /** Did the task complete successfully */
    success: boolean;

    /** If failed, why */
    failureReason?: string;

    /** Partial success indicator */
    partialSuccess: boolean;

    /** Total execution time */
    latencyMs: number;

    /** Token consumption */
    tokensUsed: number;
    thinkingTokens: number;
    outputTokens: number;

    /** Tool usage */
    toolCallCount: number;
    toolCallSuccessRate: number;
    toolsUsed: Array<{
      toolName: string;
      callCount: number;
      successRate: number;
      avgLatency: number;
    }>;

    /** Efficiency metrics */
    tokenEfficiency: number; // Task value / tokens used
    timeToFirstAction: number; // Latency before first tool call
  };

  // ============================================
  // EVALUATION METRICS
  // ============================================

  evaluation: {
    /** CAWS compliance */
    cawsCompliant: boolean;
    budgetCompliant: boolean;

    /** Quality gates */
    qualityGates: Array<{
      name: string;
      passed: boolean;
      score: number;
      threshold: number;
      mandatory: boolean;
    }>;

    /** Rubric scores (for RL training) */
    rubricScores: {
      format: number; // 0-1: JSON/schema compliance
      tool: number; // 0-1: Tool choice appropriateness
      task: number; // 0-1: Task completion correctness
      minimal: number; // 0-1: Code minimality
      cost: number; // 0-1: Token/time efficiency
      safety: number; // 0-1: Security and permissions
    };

    /** Minimal-diff analysis */
    minimalDiffMetrics: {
      astSimilarity: number; // 0-1
      fileTouchCount: number;
      lineChangeRatio: number;
      scaffoldingPenalty: number; // 0-1
      rewardMultiplier: number; // 0.1-1.0
    };

    /** Overall evaluation */
    overallScore: number; // 0-1
    passedAllMandatory: boolean;
  };

  // ============================================
  // RL TRAINING LABELS
  // ============================================

  rlLabels: {
    /** Turn-by-turn data for multi-turn RL */
    turns: Array<{
      turnNumber: number;
      toolCalled: string;
      informationGain: number; // 0-1
      formatCorrectness: number; // 0-1
      taskProgress: number; // 0-1
      safetyScore: number; // 0-1
      turnReward: number;
    }>;

    /** Final outcome label */
    finalOutcome: "success" | "failure" | "partial";

    /** Total trajectory reward */
    totalReward: number;

    /** Advantage (computed during RL training) */
    advantage?: number;
  };

  // ============================================
  // PROVENANCE & AUDIT
  // ============================================

  provenance: {
    /** Was this AI-assisted */
    aiAssisted: boolean;

    /** Human review occurred */
    humanReviewed: boolean;

    /** Git commit reference */
    commitHash?: string;

    /** CAWS spec version */
    cawsVersion: string;
  };

  // ============================================
  // CAWS GOVERNANCE DATA
  // ============================================

  caws: {
    /** CAWS working spec reference */
    specId: string;

    /** CAWS verdict ID from validation */
    verdictId: string;

    /** Waivers used for this task */
    waiversUsed: string[]; // e.g., ["WV-BUDGET-001"]

    /** Quality gates passed */
    gatesPassed: string[]; // e.g., ["tests-pass", "lint-clean", "coverage"]

    /** CAWS compliance scores */
    scores: {
      evidenceCompleteness: number; // 0-1
      budgetAdherence: number; // 0-1
      gateIntegrity: number; // 0-1
      provenanceClarity: number; // 0-1
    };

    /** Arbiter signature for audit trail */
    arbiterSignature: string;

    /** Budget usage details */
    budgetUsage: {
      filesUsed: number;
      filesLimit: number;
      locUsed: number;
      locLimit: number;
      budgetCompliant: boolean;
    };
  };
}
```

**Note**: These CAWS governance fields ensure every benchmark record is traceable to a specific CAWS verdict, making benchmark data inherently auditable and suitable for compliance-aware RL training.

---

## Supporting Types

### TaskContext

```typescript
interface TaskContext {
  id: string;
  type: "code-editing" | "research" | "data-analysis" | "design" | "planning";
  complexity: "trivial" | "standard" | "complex";
  requirements: string[];
  description: string;
  cawsSpecId?: string;
}
```

### RoutingContext

```typescript
interface RoutingContext {
  selectedAgent: {
    id: string;
    name: string;
    modelFamily: string;
  };
  strategy:
    | "multi-armed-bandit"
    | "capability-match"
    | "load-balance"
    | "fallback";
  confidence: number;
  rationale: string;
  alternativesConsidered: AlternativeAgent[];
  historicalPerformance: {
    agentSuccessRate: number;
    agentAvgQuality: number;
    agentTaskCount: number;
  };
}

interface AlternativeAgent {
  agentId: string;
  score: number;
  reason: string;
}
```

### ExecutionMetrics

```typescript
interface ExecutionMetrics {
  success: boolean;
  failureReason?: string;
  partialSuccess: boolean;
  latencyMs: number;
  tokensUsed: number;
  thinkingTokens: number;
  outputTokens: number;
  toolCallCount: number;
  toolCallSuccessRate: number;
  toolsUsed: ToolUsageMetric[];
  tokenEfficiency: number;
  timeToFirstAction: number;
}

interface ToolUsageMetric {
  toolName: string;
  callCount: number;
  successRate: number;
  avgLatency: number;
}
```

### EvaluationMetrics

```typescript
interface EvaluationMetrics {
  cawsCompliant: boolean;
  budgetCompliant: boolean;
  qualityGates: QualityGateResult[];
  rubricScores: RubricScores;
  minimalDiffMetrics: MinimalDiffMetrics;
  overallScore: number;
  passedAllMandatory: boolean;
}

interface QualityGateResult {
  name: string;
  passed: boolean;
  score: number;
  threshold: number;
  mandatory: boolean;
}

interface RubricScores {
  format: number; // 0-1
  tool: number; // 0-1
  task: number; // 0-1
  minimal: number; // 0-1
  cost: number; // 0-1
  safety: number; // 0-1
}

interface MinimalDiffMetrics {
  astSimilarity: number; // 0-1
  fileTouchCount: number;
  lineChangeRatio: number;
  scaffoldingPenalty: number; // 0-1
  rewardMultiplier: number; // 0.1-1.0
}
```

### RLTrainingLabels

```typescript
interface RLTrainingLabels {
  turns: TurnLevelData[];
  finalOutcome: "success" | "failure" | "partial";
  totalReward: number;
  advantage?: number; // Computed during training
}

interface TurnLevelData {
  turnNumber: number;
  toolCalled: string;
  informationGain: number; // 0-1
  formatCorrectness: number; // 0-1
  taskProgress: number; // 0-1
  safetyScore: number; // 0-1
  turnReward: number;
}
```

---

## Data Collection Example

```typescript
// Complete data point creation
const dataPoint: BenchmarkDataPoint = {
  id: crypto.randomUUID(),
  timestamp: new Date(),
  tenantId: hashTenantId(task.tenantId),

  task: {
    id: task.id,
    type: "code-editing",
    complexity: "standard",
    requirements: ["TypeScript", "AST analysis"],
    description: anonymize(task.description),
    cawsSpecId: "FEAT-001"
  },

  routing: {
    selectedAgent: {
      id: agent.id,
      name: agent.name,
      modelFamily: "GPT-4"
    },
    strategy: "multi-armed-bandit",
    confidence: 0.85,
    rationale: "Best success rate for AST tasks (85%)",
    alternativesConsidered: [...],
    historicalPerformance: {
      agentSuccessRate: 0.85,
      agentAvgQuality: 0.88,
      agentTaskCount: 127
    }
  },

  execution: {
    success: true,
    partialSuccess: false,
    latencyMs: 12500,
    tokensUsed: 8500,
    thinkingTokens: 2000,
    outputTokens: 6500,
    toolCallCount: 8,
    toolCallSuccessRate: 1.0,
    toolsUsed: [
      { toolName: "read_file", callCount: 3, successRate: 1.0, avgLatency: 150 },
      { toolName: "search_replace", callCount: 4, successRate: 1.0, avgLatency: 200 },
      { toolName: "run_tests", callCount: 1, successRate: 1.0, avgLatency: 3000 }
    ],
    tokenEfficiency: 0.92,
    timeToFirstAction: 850
  },

  evaluation: {
    cawsCompliant: true,
    budgetCompliant: true,
    qualityGates: [
      { name: "tests-pass", passed: true, score: 1.0, threshold: 1.0, mandatory: true },
      { name: "lint-clean", passed: true, score: 1.0, threshold: 1.0, mandatory: true },
      { name: "coverage", passed: true, score: 0.92, threshold: 0.8, mandatory: true }
    ],
    rubricScores: {
      format: 0.95,
      tool: 0.88,
      task: 0.92,
      minimal: 0.85,
      cost: 0.87,
      safety: 1.0
    },
    minimalDiffMetrics: {
      astSimilarity: 0.92,
      fileTouchCount: 2,
      lineChangeRatio: 0.08,
      scaffoldingPenalty: 0.1,
      rewardMultiplier: 0.88
    },
    overallScore: 0.90,
    passedAllMandatory: true
  },

  rlLabels: {
    turns: [
      { turnNumber: 1, toolCalled: "read_file", informationGain: 0.9, formatCorrectness: 1.0, taskProgress: 0.1, safetyScore: 1.0, turnReward: 0.75 },
      { turnNumber: 2, toolCalled: "search_replace", informationGain: 0.8, formatCorrectness: 1.0, taskProgress: 0.5, safetyScore: 1.0, turnReward: 0.82 },
      // ... more turns
    ],
    finalOutcome: "success",
    totalReward: 0.90
  },

  provenance: {
    aiAssisted: true,
    humanReviewed: false,
    commitHash: "abc123...",
    cawsVersion: "3.1.0"
  }
};
```

---

## Database Implementation

### Time-Series Storage (TimescaleDB)

```sql
-- Routing decisions
CREATE TABLE routing_decisions (
  time TIMESTAMPTZ NOT NULL,
  task_id UUID,
  agent_id UUID,
  strategy VARCHAR(50),
  confidence DECIMAL(3,2),
  success_rate DECIMAL(5,4),
  avg_quality DECIMAL(3,2),
  task_count INTEGER,
  metadata JSONB
);

SELECT create_hypertable('routing_decisions', 'time');
CREATE INDEX idx_routing_agent ON routing_decisions(agent_id, time DESC);
CREATE INDEX idx_routing_task_type ON routing_decisions((metadata->>'taskType'), time DESC);

-- Execution metrics
CREATE TABLE execution_metrics (
  time TIMESTAMPTZ NOT NULL,
  task_id UUID,
  agent_id UUID,
  success BOOLEAN,
  latency_ms INTEGER,
  tokens_used INTEGER,
  thinking_tokens INTEGER,
  tool_call_count INTEGER,
  quality_score DECIMAL(3,2),
  metadata JSONB
);

SELECT create_hypertable('execution_metrics', 'time');
CREATE INDEX idx_execution_agent ON execution_metrics(agent_id, time DESC);

-- Evaluation outcomes
CREATE TABLE evaluation_outcomes (
  time TIMESTAMPTZ NOT NULL,
  task_id UUID,
  agent_id UUID,
  caws_compliant BOOLEAN,
  overall_score DECIMAL(3,2),
  rubric_scores JSONB,
  minimal_diff_metrics JSONB,
  metadata JSONB
);

SELECT create_hypertable('evaluation_outcomes', 'time');
CREATE INDEX idx_evaluation_task ON evaluation_outcomes(task_id, time DESC);
```

### Document Storage (PostgreSQL JSONB)

```sql
-- Complete benchmark documents
CREATE TABLE benchmark_data (
  id UUID PRIMARY KEY,
  timestamp TIMESTAMPTZ NOT NULL,
  tenant_id VARCHAR(64), -- Hashed

  -- Full document
  data JSONB NOT NULL,

  -- Indexed fields for fast queries
  task_type VARCHAR(50),
  complexity VARCHAR(20),
  agent_id UUID,
  success BOOLEAN,
  quality_score DECIMAL(3,2),

  -- RL training flags
  rl_ready BOOLEAN DEFAULT false,
  rl_exported_at TIMESTAMPTZ,
  rl_batch_id UUID,

  -- Metadata
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_benchmark_agent ON benchmark_data(agent_id, timestamp DESC);
CREATE INDEX idx_benchmark_task_type ON benchmark_data(task_type, timestamp DESC);
CREATE INDEX idx_benchmark_rl_ready ON benchmark_data(rl_ready, timestamp DESC) WHERE rl_ready = true;
CREATE INDEX idx_benchmark_quality ON benchmark_data(quality_score DESC);
```

### Agent Statistics (Aggregated)

```sql
-- Fast lookups for routing decisions
CREATE TABLE agent_statistics (
  agent_id UUID,
  task_type VARCHAR(50),
  time_bucket TIMESTAMPTZ, -- Hourly or daily buckets

  -- Aggregated metrics
  total_tasks INTEGER,
  successful_tasks INTEGER,
  success_rate DECIMAL(5,4),
  avg_quality DECIMAL(3,2),
  avg_latency INTEGER,
  avg_tokens INTEGER,

  PRIMARY KEY (agent_id, task_type, time_bucket)
);

CREATE INDEX idx_agent_stats_recent ON agent_statistics(agent_id, time_bucket DESC);
```

---

## Data Access Patterns

### Fast Routing Queries

```sql
-- Get recent agent performance for routing decision
SELECT
  agent_id,
  success_rate,
  avg_quality,
  avg_latency,
  total_tasks
FROM agent_statistics
WHERE agent_id = $1
  AND task_type = $2
  AND time_bucket >= NOW() - INTERVAL '7 days'
ORDER BY time_bucket DESC
LIMIT 1;
```

### RL Training Batch Export

```sql
-- Export quality data for RL training
SELECT data
FROM benchmark_data
WHERE rl_ready = true
  AND rl_exported_at IS NULL
  AND quality_score >= 0.7
  AND timestamp >= NOW() - INTERVAL '90 days'
ORDER BY timestamp DESC
LIMIT 5000;
```

### Analytics Queries

```sql
-- System-wide performance trends
SELECT
  task_type,
  DATE_TRUNC('day', time_bucket) as day,
  AVG(success_rate) as avg_success,
  AVG(avg_quality) as avg_quality,
  SUM(total_tasks) as total_tasks
FROM agent_statistics
WHERE time_bucket >= NOW() - INTERVAL '30 days'
GROUP BY task_type, day
ORDER BY day DESC;
```

---

## Data Validation Schema

### JSON Schema for Validation

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": [
    "id",
    "timestamp",
    "task",
    "routing",
    "execution",
    "evaluation",
    "rlLabels"
  ],
  "properties": {
    "id": { "type": "string", "format": "uuid" },
    "timestamp": { "type": "string", "format": "date-time" },
    "tenantId": { "type": "string", "pattern": "^[a-f0-9]{64}$" },
    "task": {
      "type": "object",
      "required": ["id", "type", "complexity"],
      "properties": {
        "type": {
          "enum": [
            "code-editing",
            "research",
            "data-analysis",
            "design",
            "planning"
          ]
        },
        "complexity": { "enum": ["trivial", "standard", "complex"] }
      }
    },
    "evaluation": {
      "type": "object",
      "properties": {
        "rubricScores": {
          "type": "object",
          "properties": {
            "format": { "type": "number", "minimum": 0, "maximum": 1 },
            "tool": { "type": "number", "minimum": 0, "maximum": 1 },
            "task": { "type": "number", "minimum": 0, "maximum": 1 },
            "minimal": { "type": "number", "minimum": 0, "maximum": 1 },
            "cost": { "type": "number", "minimum": 0, "maximum": 1 },
            "safety": { "type": "number", "minimum": 0, "maximum": 1 }
          },
          "required": ["format", "tool", "task", "minimal", "cost", "safety"]
        }
      }
    }
  }
}
```

---

## Success Criteria

**Schema Compliance**:

- ✅ 100% of data points match schema
- ✅ All required fields present
- ✅ Type validation passes

**Data Integrity**:

- ✅ No duplicate data points
- ✅ Referential integrity maintained
- ✅ Timestamps chronologically correct

**Performance**:

- ✅ Insert latency <10ms
- ✅ Query latency <100ms
- ✅ Storage efficiency >80%

---

**This schema provides the structured foundation for both operational orchestration and long-term RL training—every field is intentional and serves a specific purpose.**
