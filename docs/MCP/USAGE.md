# MCP Tool Ecosystem Usage Guide

## Overview

**✅ FULLY IMPLEMENTED** - The Model Context Protocol (MCP) integration provides a comprehensive tool ecosystem enabling external AI models and autonomous agents to leverage Agent Agency's sophisticated internal capabilities. This guide covers the 13 specialized tools across 7 categories for governance, verification, reasoning, and workflow management.

## Quick Start

### Prerequisites

- Rust 1.75+
- Local AI model (Gemma, LM Studio, Ollama)
- Agent Agency V3 iteration built and running

### Installation

```bash
# From the project root
cd iterations/v3

# Build the MCP server
cargo build --release

# Start the MCP server (stdio mode for local AI models)
cargo run --bin mcp-server
```

### Basic Usage with Local AI

1. **Start the MCP Server:**

   ```bash
   cd iterations/v3
   cargo run --bin mcp-server
   ```

2. **Connect with Local AI Model:**
   Configure your local AI model to connect to the MCP server using stdio transport.

3. **Tool Discovery:**
   ```json
   // Available tools (13 total)
   {
     "tools": [
       {"name": "caws_policy_validator", "description": "Validate CAWS compliance"},
       {"name": "logic_validator", "description": "Validate logical reasoning"},
       {"name": "progress_tracker", "description": "Track workflow progress"},
       // ... all 13 tools
     ]
   }
   ```

4. **Available Tool Categories:**
   - **Policy Tools (3)**: `caws_policy_validator`, `waiver_auditor`, `budget_verifier`
   - **Conflict Resolution Tools (3)**: `debate_orchestrator`, `consensus_builder`, `evidence_synthesizer`
   - **Evidence Collection Tools (3)**: `claim_extractor`, `fact_verifier`, `source_validator`
   - **Governance Tools (3)**: `audit_logger`, `provenance_tracker`, `compliance_reporter`
   - **Quality Gate Tools (3)**: `code_analyzer`, `test_executor`, `performance_validator`
   - **Reasoning Tools (2)**: `logic_validator`, `inference_engine`
   - **Workflow Tools (2)**: `progress_tracker`, `resource_allocator`

## Tool Usage Examples

### Policy & Governance Tools

#### `caws_policy_validator`

Validates task compliance with CAWS governance rules.

```typescript
// Validate CAWS compliance
const result = await mcp.callTool('caws_policy_validator', {
  task_description: "Implement user authentication with JWT tokens",
  risk_tier: "tier_2",
  scope_boundaries: ["authentication", "security", "api"]
});

console.log(result);
// {
//   "validation_id": "val_abc123",
//   "compliance_score": 0.92,
//   "risk_assessment": {...},
//   "policy_violations": [],
//   "recommendations": [...]
// }
```

#### `waiver_auditor`

Audits waiver requests for governance exceptions.

```typescript
const result = await mcp.callTool('waiver_auditor', {
  waiver_request: {
    justification: "Performance optimization requires direct DB access",
    risk_level: "medium",
    impact_scope: ["database", "performance"]
  },
  risk_assessment: { severity: "medium", probability: 0.3 },
  justification_criteria: ["business_necessity", "performance_critical"]
});
```

### Evidence Collection Tools

#### `fact_verifier`

Performs multi-modal fact verification with council arbitration.

```typescript
const result = await mcp.callTool('fact_verifier', {
  claims: [
    "The Earth orbits the Sun",
    "Water boils at 100°C at sea level"
  ],
  verification_sources: ["scientific_databases", "peer_reviewed_sources"],
  council_tier: "tier_2",
  confidence_threshold: 0.85
});
```

#### `source_validator`

Assesses credibility and security of information sources.

```typescript
const result = await mcp.callTool('source_validator', {
  source_url: "https://example.com/article",
  content_type: "news_article",
  validation_criteria: ["credibility", "timeliness", "security"],
  security_checks: ["malware_scan", "phishing_detection"]
});
```

### Reasoning Tools

#### `logic_validator`

Validates logical consistency and detects reasoning fallacies.

```typescript
const result = await mcp.callTool('logic_validator', {
  reasoning_content: "All men are mortal. Socrates is a man. Therefore, Socrates is mortal.",
  validation_criteria: ["consistency", "soundness", "completeness"],
  strictness_level: "moderate",
  domain_context: "philosophical_logic"
});
```

#### `inference_engine`

Performs probabilistic reasoning across multiple inference methods.

```typescript
const result = await mcp.callTool('inference_engine', {
  premises: [
    "Most software projects exceed their budget",
    "This project is similar to typical software projects"
  ],
  inference_goal: "This project will exceed its budget",
  inference_method: "probabilistic",
  domain_knowledge: {
    expert_rules: ["Software projects often face scope creep"],
    historical_data: [/* project statistics */]
  },
  uncertainty_threshold: 0.7
});
```

### Quality Assurance Tools

#### `code_analyzer`

Performs comprehensive code analysis across multiple dimensions.

```typescript
const result = await mcp.callTool('code_analyzer', {
  code_path: "./src/main.rs",
  analysis_types: ["lint", "type_check", "complexity", "security"],
  include_security_scan: true,
  performance_benchmarks: ["memory_usage", "cpu_efficiency"],
  quality_thresholds: {
    complexity_score: 0.8,
    security_score: 0.9,
    maintainability_index: 0.75
  }
});
```

#### `test_executor`

Executes comprehensive test suites with coverage analysis.

```typescript
const result = await mcp.callTool('test_executor', {
  test_path: "./tests/",
  test_types: ["unit", "integration", "e2e", "performance"],
  include_coverage: true,
  coverage_thresholds: {
    line_coverage: 0.8,
    branch_coverage: 0.9,
    function_coverage: 0.85
  },
  timeout_seconds: 300,
  parallel_execution: true
});
```

### Workflow Management Tools

#### `progress_tracker`

Tracks workflow progress with milestone analysis and predictions.

```typescript
const result = await mcp.callTool('progress_tracker', {
  workflow_id: "feature_development_sprint",
  workflow_type: "code_development",
  include_milestones: true,
  include_predictions: true,
  current_metrics: {
    completed_tasks: 8,
    total_tasks: 12,
    time_elapsed_ms: 7200000, // 2 hours
    current_phase: "implementation"
  }
});
```

#### `resource_allocator`

Performs adaptive resource allocation with optimization.

```typescript
const result = await mcp.callTool('resource_allocator', {
  task_id: "ml_training_job",
  task_requirements: {
    cpu_cores: 8,
    memory_gb: 32.0,
    gpu_memory_gb: 16.0,
    estimated_duration_hours: 24.0
  },
  optimization_criteria: ["performance", "cost_efficiency"],
  priority_level: "high",
  allocation_constraints: {
    max_concurrent_tasks: 5,
    available_gpu_memory: 48.0
  }
});
```

## Integration with AI Models

### Connecting to Local AI Models

#### LM Studio Integration

```typescript
// Connect MCP server to LM Studio
import { MCPClient } from '@modelcontextprotocol/sdk';

const client = new MCPClient({
  transport: {
    type: 'stdio',
    command: 'cargo',
    args: ['run', '--bin', 'mcp-server'],
    cwd: '/path/to/agent-agency/iterations/v3'
  }
});

// Initialize connection
await client.connect();

// Discover tools
const tools = await client.listTools();
console.log(`Available tools: ${tools.tools.length}`); // 13 tools

// Use tools in your LM Studio workflows
const result = await client.callTool('logic_validator', {
  reasoning_content: userReasoning,
  validation_criteria: ['consistency', 'soundness']
});
```

#### Ollama Integration

```bash
# Start MCP server
cd iterations/v3
cargo run --bin mcp-server &

# Configure Ollama to use MCP tools
# In your Ollama configuration or custom integration
```

### Autonomous Agent Workflows

#### Policy-Governed Development

```typescript
async function autonomousDevelopment(taskDescription: string) {
  // 1. Validate CAWS compliance first
  const policyCheck = await mcp.callTool('caws_policy_validator', {
    task_description: taskDescription,
    risk_tier: "tier_2"
  });

  if (policyCheck.compliance_score < 0.8) {
    throw new Error("Task violates CAWS policies");
  }

  // 2. Implement solution
  const implementation = await generateCode(taskDescription);

  // 3. Analyze code quality
  const analysis = await mcp.callTool('code_analyzer', {
    code_path: implementation.filePath,
    analysis_types: ["lint", "security", "complexity"]
  });

  // 4. Run tests
  const testResults = await mcp.callTool('test_executor', {
    test_path: "./tests/",
    test_types: ["unit", "integration"]
  });

  // 5. Validate performance
  const performance = await mcp.callTool('performance_validator', {
    test_scenario: "production_load",
    duration_minutes: 30
  });

  return {
    implementation,
    quality: analysis,
    tests: testResults,
    performance
  };
}
```

#### Evidence-Based Reasoning

```typescript
async function evidenceBasedDecision(query: string) {
  // 1. Extract claims from query
  const claims = await mcp.callTool('claim_extractor', {
    content: query,
    content_type: "user_query"
  });

  // 2. Verify facts
  const factCheck = await mcp.callTool('fact_verifier', {
    claims: claims.claims,
    verification_sources: ["web_search", "academic_databases"]
  });

  // 3. Validate source credibility
  const sourceValidation = await mcp.callTool('source_validator', {
    source_url: factCheck.primary_source,
    validation_criteria: ["credibility", "timeliness"]
  });

  // 4. Perform logical inference
  const inference = await mcp.callTool('inference_engine', {
    premises: factCheck.verified_claims,
    inference_goal: "evidence_based_conclusion",
    inference_method: "probabilistic"
  });

  return {
    claims: claims.claims,
    verification: factCheck,
    source_credibility: sourceValidation,
    conclusion: inference
  };
}
```

## Summary

### Tool Categories Overview

| Category | Purpose | Key Capabilities | Enterprise Integration |
|----------|---------|------------------|----------------------|
| **Policy** | Governance & Compliance | CAWS validation, waiver audit, budget verification | Claim extraction pipeline |
| **Conflict Resolution** | Decision Making | Multi-model arbitration, consensus building, evidence synthesis | Council arbitration system |
| **Evidence Collection** | Verification & Validation | Fact verification, source validation, claim extraction | Multi-modal verification |
| **Governance** | Audit & Provenance | Audit logging, provenance tracking, compliance reporting | Provenance service |
| **Quality Gate** | Code Quality & Testing | Code analysis, test execution, performance validation | Quality gates, testing infrastructure |
| **Reasoning** | Logic & Inference | Logic validation, probabilistic inference | Reflexive learning algorithms |
| **Workflow** | Project Management | Progress tracking, resource allocation | Progress tracking, resource management |

### Production Deployment

```bash
# Build and deploy MCP server
cd iterations/v3
cargo build --release

# Start in production mode
./target/release/mcp-server --config production.yaml

# Monitor with built-in metrics
curl http://localhost:9090/metrics
```

### Enterprise Integration Benefits

- **Robust Systems**: All tools leverage battle-tested enterprise components
- **Comprehensive Coverage**: 13 tools across 7 categories for complete workflow support
- **External AI Access**: Standardized MCP protocol for seamless AI model integration
- **Governance Compliance**: Built-in CAWS compliance and provenance tracking
- **Performance Optimized**: Efficient async processing for high-throughput operations
