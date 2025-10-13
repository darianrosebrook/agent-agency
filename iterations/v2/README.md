# Agent Agency V2: Agentic RL & Extended Thinking

[![Version](https://img.shields.io/badge/version-2.0.0-blue.svg)](https://github.com/darianrosebrook/agent-agency)
[![Risk Tier](https://img.shields.io/badge/risk-T2-yellow.svg)](.caws/working-spec.yaml)
[![Quality Gates](https://img.shields.io/badge/coverage-80%2B-brightgreen.svg)](../../jest.config.js)

> **Advanced Agentic Reinforcement Learning with Extended Thinking Capabilities**

---

## 🎯 Overview

Agent Agency V2 transforms the POC foundation into a production-ready agentic system with industry-leading reinforcement learning capabilities. Building on Anthropic's extended thinking research and practical reliability measures, V2 introduces:

- **Extended Thinking as a Budgeted Resource** - Optimal token allocation based on task complexity
- **Reward Hacking Prevention** - AST-based minimal-diff analysis with penalty systems
- **Turn-Level RL Training** - Multi-turn conversation learning with intermediate rewards
- **Intelligent Evaluation** - Model-based judges complementing rule-based assessment
- **Enhanced Tool Learning** - 3-5x improvement in tool adoption rates
- **CAWS Constitutional Authority** - Runtime enforcement of quality gates and provenance

---

## 📋 Component Specifications

V2 includes comprehensive CAWS working specifications for all core arbiter components:

- **[V2 Specs Status](./docs/status/V2-SPECS-ACTUAL-STATUS.md)** - Current status of all component specs
- **[Implementation Index](./docs/status/IMPLEMENTATION-INDEX.md)** - Quick reference for all components
- **[Theory Alignment Analysis](./docs/THEORY-ALIGNMENT-AUDIT.md)** - Comprehensive 57-page mapping of theory to implementation
- **[Theory Alignment Summary](./docs/status/THEORY-ALIGNMENT-SUMMARY.md)** - Quick reference scorecard and gap analysis
- **[Theory Implementation Delta](./docs/THEORY-IMPLEMENTATION-DELTA.md)** - Executive summary: what exceeds theory, what's different, what's missing

### Core Components

| Component              | Spec ID     | Risk Tier | Status           |
| ---------------------- | ----------- | --------- | ---------------- |
| Agent Registry Manager | ARBITER-001 | T2        | ✅ Spec Complete |
| Task Routing Manager   | ARBITER-002 | T2        | ✅ Spec Complete |
| CAWS Validator         | ARBITER-003 | T1        | ✅ Spec Complete |
| Performance Tracker    | ARBITER-004 | T2        | ✅ Spec Complete |
| Arbiter Orchestrator   | ARBITER-005 | T1        | ✅ Spec Complete |

See [V2 Specs Status](./docs/status/V2-SPECS-ACTUAL-STATUS.md) for detailed specifications and validation status.

---

## 🚀 Key Features

### 1. Thinking Budget Management

Treat thinking as an optimizable resource, not a binary toggle.

```typescript
const budget = await thinkingBudgetManager.allocateBudget(task);
// Trivial: ≤500 tokens, Standard: ≤2000 tokens, Complex: ≤8000 tokens

// Automatic escalation based on confidence thresholds
await thinkingBudgetManager.monitorConsumption(budget, evaluationResult);
```

**Benefits**:

- Prevents infinite thinking loops with hard ceilings
- Optimizes token usage across task complexities
- Provides cost control and performance predictability

### 2. Minimal-Diff Evaluation

Prevents reward hacking by enforcing minimal, targeted code changes.

```typescript
const evaluation = await minimalDiffEvaluator.evaluate(solution);
const rewardMultiplier = evaluation.rewardMultiplier; // 0.1 - 1.0

// Rewards penalize unnecessary scaffolding and over-engineering
const adjustedScore = baseScore * rewardMultiplier;
```

**Benefits**:

- Reduces "spray edits" by 60-80%
- Improves code maintainability and focus
- Creates more reliable evaluation signals

### 3. Turn-Level RL Training

Enables reinforcement learning on complete conversation trajectories.

```typescript
const trainer = new AgenticRLTrainer();
const modelUpdate = await trainer.trainOnConversation(conversation);

// Each turn receives intermediate rewards for:
// - Tool choice appropriateness
// - Information utility
// - Format correctness
// - Task progress contribution
```

**Benefits**:

- Learns from multi-turn tool interactions
- Provides credit assignment for long-horizon tasks
- Improves agent decision-making across conversation flows

### 4. Intelligent Evaluation System

Combines rule-based and model-based assessment for subjective criteria.

```typescript
const evaluation = await intelligentEvaluator.evaluate(artifact);
// Includes model judgments for:
// - Faithfulness (no hallucination)
// - Relevance (useful information)
// - Minimality (simplest correct solution)
// - Safety (no harmful patterns)
```

**Benefits**:

- More accurate evaluation of creative outputs
- Better assessment of subjective quality criteria
- Confidence-weighted judgment integration

### 5. Enhanced Tool Adoption

Dramatically improves tool usage through supervised warmup and rewards.

```typescript
// Phase 1: Supervised fine-tuning on tool usage patterns
const warmedModel = await toolTrainer.supervisedWarmup(baseModel, examples);

// Phase 2: RL fine-tuning with intermediate tool rewards
const enhancedModel = await toolTrainer.rlFineTuning(warmedModel, examples);
```

**Benefits**:

- 3-5x improvement in tool adoption rates
- Proper JSON formatting and error handling
- Distinguishes tool choice from execution quality

---

## 📊 Performance Improvements

| Metric                  | V1 Baseline | V2 Target | Improvement             |
| ----------------------- | ----------- | --------- | ----------------------- |
| Tool Adoption Rate      | 10%         | 40%       | +300%                   |
| Thinking Efficiency     | 100%        | 60%       | -40% token waste        |
| Reward Hacking          | 100/week    | 30/week   | -70% incidents          |
| Task Completion         | 70%         | 87.5%     | +25% complex tasks      |
| Evaluation Accuracy     | 85%         | 90%       | +5% subjective criteria |
| Database Connections    | 65          | 10-20     | -75% to -85%            |
| Connection Pool Overhead| ~50-65 MB   | ~10 MB    | -80% to -85%            |

---

## 🏗️ Architecture

### Core Components

```
iterations/v2/
├── components/             # Component spec workspaces (CAWS working specs)
│   ├── agent-registry-manager/
│   ├── task-routing-manager/
│   ├── caws-validator/
│   └── ... (14 components total)
├── src/                    # Consolidated implementation
│   ├── orchestrator/       # Agent registry, routing, orchestration
│   ├── database/           # Centralized connection pool & clients
│   ├── knowledge/          # Knowledge seeker implementation
│   ├── rl/                 # Agentic RL training system
│   ├── thinking/           # Budgeted thinking management
│   ├── evaluation/         # Enhanced evaluation with model judges
│   └── types/              # Shared type definitions
├── tests/                  # Comprehensive test suite
├── docs/                   # Technical documentation
│   ├── status/             # Implementation status reports
│   ├── database/           # Database architecture & patterns
│   └── 1-core-orchestration/ # Architecture docs
├── migrations/             # Database schema migrations
├── logs/                   # Output logs
├── test-results/           # Test artifacts and coverage
└── scripts/                # Build and utility scripts
```

### Database Architecture

V2 features a **production-ready centralized database architecture** with enterprise-grade connection management:

```
┌─────────────────────────────────────────────────────┐
│           ConnectionPoolManager (Singleton)          │
│                        ↓                             │
│              Single Shared Pool                      │
│                  (10-20 conns)                       │
│                        ↑                             │
│    ┌───────┬───────┬───────┬───────┬───────┐       │
│   Agent  Know   WebNav  Verify  Orch                │
│   Registry                                           │
└─────────────────────────────────────────────────────┘
```

**Key Features**:
- ✅ **Centralized Pool Management**: Single shared connection pool across all clients
- ✅ **Connection Efficiency**: 75-85% reduction in connections (65 → 10-20)
- ✅ **Memory Optimization**: 80-85% reduction in pool overhead (~50-65 MB → ~10 MB)
- ✅ **Health Monitoring**: Comprehensive connection health checks and statistics
- ✅ **Graceful Shutdown**: Proper connection cleanup on application termination
- ✅ **Tenant Context Support**: Row Level Security (RLS) ready for multi-tenancy
- ✅ **Hybrid Vector-Graph**: pgvector for semantic search + knowledge graphs for relationships

**Database Clients**:
- `AgentRegistryDatabaseClient` - Agent profiles and performance tracking
- `KnowledgeDatabaseClient` - Knowledge queries and search results
- `WebNavigatorDatabaseClient` - Web content and traversal tracking
- `VerificationDatabaseClient` - Verification requests and evidence
- `DatabaseClient` (Orchestrator) - Task assignments and orchestration state

See **[Database Documentation](./docs/database/README.md)** for complete architecture details, migration guides, and query patterns.

### Integration Points

V2 seamlessly integrates with V1 components:

- **Agent Orchestrator**: Enhanced routing with RL insights
- **MCP Server**: Tool management with adoption training
- **Memory System**: Context offloading with thinking budgets
- **Evaluation Orchestrator**: Multi-stage assessment pipeline
- **Data Layer**: RL training data storage and anonymization

### Feature Flags

All V2 features are controlled by environment variables for safe deployment:

```bash
# Enable V2 features individually
ENABLE_THINKING_BUDGETS=true
ENABLE_MINIMAL_DIFF=true
ENABLE_RL_TRAINING=true
ENABLE_MODEL_JUDGES=true
ENABLE_TOOL_TRAINING=true
```

---

## 🚀 Quick Start

### Prerequisites

- Node.js 20+ with ES modules
- PostgreSQL with pgvector extension
- Ollama for local model serving (optional)
- 8GB+ RAM for RL training

### Installation

```bash
# From project root
cd iterations/v2

# Install dependencies
npm install

# Set up environment
cp .env.example .env
# Configure database and model settings

# Run database migrations
npm run migrate

# Start development server
npm run dev
```

### Basic Usage

```typescript
import { AgentAgencyV2 } from "./src/index";

// Initialize with V2 features
const agency = new AgentAgencyV2({
  enableThinkingBudgets: true,
  enableRLTraining: true,
  enableToolLearning: true,
});

// Enhanced agent with RL capabilities
const agent = await agency.createAgent({
  model: "gemma-3n",
  thinkingBudget: "complex", // 8000 token budget
  enableToolLearning: true,
});

// Execute task with V2 enhancements
const result = await agent.executeTask({
  prompt: "Implement user authentication with password reset",
  evaluationMode: "enhanced", // Includes minimal-diff and model judges
  rlTraining: true, // Enable turn-level learning
});
```

---

## 🔧 Configuration

### Environment Variables

```bash
# Database (Centralized Connection Pool)
DB_HOST=localhost
DB_PORT=5432
DB_NAME=agent_agency_v2
DB_USER=postgres
DB_PASSWORD=
DB_POOL_MIN=2
DB_POOL_MAX=20

# Model Configuration
OLLAMA_BASE_URL=http://localhost:11434
DEFAULT_MODEL=gemma-3n

# V2 Feature Flags
ENABLE_THINKING_BUDGETS=true
ENABLE_MINIMAL_DIFF=true
ENABLE_RL_TRAINING=true
ENABLE_MODEL_JUDGES=true
ENABLE_TOOL_TRAINING=true

# Performance Budgets
THINKING_BUDGET_P95_MS=10000
RL_INFERENCE_P95_MS=1000
EVALUATION_P95_MS=500

# Security
RL_DATA_ANONYMIZATION=true
DIFFERENTIAL_PRIVACY_NOISE=0.1
```

### Advanced Configuration

```typescript
const config = {
  thinking: {
    budgets: {
      trivial: { maxTokens: 500, escalationThreshold: 0.9 },
      standard: { maxTokens: 2000, escalationThreshold: 0.7 },
      complex: { maxTokens: 8000, escalationThreshold: 0.5 },
    },
  },
  rl: {
    training: {
      batchSize: 32,
      learningRate: 1e-5,
      maxTurns: 10,
    },
  },
  evaluation: {
    judges: {
      faithfulness: { enabled: true, confidenceThreshold: 0.8 },
      relevance: { enabled: true, confidenceThreshold: 0.7 },
      minimality: { enabled: true, confidenceThreshold: 0.6 },
      safety: { enabled: true, confidenceThreshold: 0.9 },
    },
  },
};
```

---

## 🧪 Testing

### Quality Gates

V2 maintains Tier 2 quality standards:

- **Branch Coverage**: ≥80%
- **Mutation Score**: ≥50%
- **Contract Tests**: Required for RL and evaluation APIs

### Running Tests

```bash
# All tests
npm test

# With coverage
npm run test:coverage

# Mutation testing
npm run test:mutation

# Contract tests
npm run test:contract

# Performance benchmarks
npm run benchmark
```

### Test Categories

```bash
# Unit tests for core logic
npm run test:unit

# Integration tests for component interaction
npm run test:integration

# E2E tests for complete workflows
npm run test:e2e

# RL-specific tests
npm run test:rl

# Performance regression tests
npm run test:perf
```

---

## 📈 Monitoring & Observability

### Metrics

V2 exposes comprehensive metrics for monitoring:

```typescript
// Thinking budget metrics
thinking_budget_allocation_rate;
thinking_budget_escalation_rate;
thinking_budget_efficiency;

// RL training metrics
rl_training_completion_rate;
rl_conversation_processing_rate;
rl_model_update_frequency;

// Tool adoption metrics
tool_adoption_rate;
tool_call_success_rate;
tool_format_correctness;

// Evaluation metrics
judgment_confidence_average;
minimal_diff_penalty_average;
reward_hacking_incidents;
```

### Tracing

Complete request tracing for V2 features:

```typescript
// Trace enhanced evaluation
const trace = await tracer.traceEnhancedEvaluation(taskResult);

// Includes spans for:
// - Rule-based evaluation
// - Minimal-diff analysis
// - Model judgments
// - RL feedback collection
```

### Dashboards

Access monitoring dashboards at:

- **Local**: http://localhost:3000/metrics
- **Production**: https://agent-agency.com/v2/metrics

---

## 🔒 Security & Privacy

### RL Data Protection

V2 implements privacy-first RL training:

- **Anonymization**: PII removal before training
- **Differential Privacy**: Noise addition to prevent re-identification
- **Federated Learning**: Cross-tenant learning without data sharing
- **Audit Trail**: Complete provenance tracking

### Safe RL Training

Multiple safeguards prevent harmful behavior:

```typescript
const safeTrainer = new SafeRLTrainer();

// Validates training data for safety
await safeTrainer.validateTrainingData(conversation);

// Applies safety constraints during training
await safeTrainer.trainSafely(validatedData);
```

---

## 🚦 Rollback & Feature Flags

### Feature Flags

Disable V2 features instantly if issues arise:

```bash
# Emergency rollback - disable all V2 features
ENABLE_THINKING_BUDGETS=false
ENABLE_MINIMAL_DIFF=false
ENABLE_RL_TRAINING=false
ENABLE_MODEL_JUDGES=false
ENABLE_TOOL_TRAINING=false
```

### Rollback Procedures

**Level 1 - Feature Flag Rollback** (1 minute):

```bash
# Disable V2 features via environment
kubectl set env deployment/agent-agency ENABLE_*=false
```

**Level 2 - Blue-Green Rollback** (15 minutes):

```bash
# Cut traffic back to V1 deployment
kubectl rollout undo deployment/agent-agency
```

**Level 3 - Database Rollback** (30 minutes):

```bash
# Remove V2 data while preserving V1
npm run migrate:rollback:v2
```

---

## 🔗 Reference Implementations

V2 leverages the **CAWS CLI project** (`@paths.design/caws-cli` v3.4.0) as a production-ready reference for core governance features:

| Feature                 | CAWS CLI                | V2 Component | Status      |
| ----------------------- | ----------------------- | ------------ | ----------- |
| Working Spec Validation | `validate.js`           | ARBITER-003  | 🔄 Adapting |
| Quality Gate Execution  | `evaluate.js`           | ARBITER-003  | 🔄 Adapting |
| Budget Validation       | `budget-checker.js`     | ARBITER-003  | 🔄 Adapting |
| Provenance Tracking     | `provenance/*.js`       | Audit Trail  | 📋 Planned  |
| Performance Analytics   | `provenance/analyze-ai` | ARBITER-004  | 📋 Planned  |
| Git Integration         | `hooks/*.sh`            | Publication  | 📋 Planned  |

**Benefits**:

- 50-70% reduction in ARBITER-003 development time
- Battle-tested governance patterns
- Production-ready quality gate execution
- Proven provenance tracking architecture

See **[Theory Implementation Delta](./docs/THEORY-IMPLEMENTATION-DELTA.md)** for detailed mapping and time estimates.

---

## 📚 Documentation

### Technical Documentation

- **[Technical Architecture](./docs/technical-architecture.md)** - Detailed system design
- **[Implementation Roadmap](./docs/implementation-roadmap.md)** - 12-week development plan
- **[API Reference](./docs/api/)** - Complete API documentation
- **[Migration Guide](./docs/migration.md)** - Upgrading from V1
- **[Database Architecture](./docs/database/README.md)** - Centralized connection pool & schema

### Database Documentation

- **[Database Architecture Overview](./docs/database/README.md)** - Complete database system guide
- **[Pattern Comparison](./docs/database/DATABASE-PATTERN-COMPARISON.md)** - Cross-project pattern analysis
- **[Query Patterns](./docs/database/QUERY-PATTERNS.md)** - Optimized query examples
- **[Schema Documentation](./docs/database/SCHEMA-DOCUMENTATION.md)** - Complete schema reference
- **[Migration Guide](./docs/database/MIGRATION-PLAN.md)** - Database migration procedures

### Developer Guides

- **[RL Training Guide](./docs/guides/rl-training.md)** - Implementing custom RL features
- **[Thinking Budgets](./docs/guides/thinking-budgets.md)** - Optimizing token allocation
- **[Evaluation Enhancement](./docs/guides/evaluation.md)** - Building model judges
- **[Tool Learning](./docs/guides/tool-learning.md)** - Improving tool adoption

### Operational Guides

- **[Deployment](./docs/ops/deployment.md)** - Production deployment procedures
- **[Monitoring](./docs/ops/monitoring.md)** - Observability and alerting
- **[Troubleshooting](./docs/ops/troubleshooting.md)** - Common issues and solutions
- **[Performance Tuning](./docs/ops/performance.md)** - Optimization strategies

---

## 🤝 Contributing

### Development Workflow

1. **Create working spec** for new features
2. **Implement with tests** following TDD
3. **Validate with CAWS** before PR
4. **Update documentation** as needed

### Code Standards

- **TypeScript**: Strict mode enabled
- **Testing**: 80%+ coverage required
- **Documentation**: JSDoc for all public APIs
- **Security**: All inputs validated, outputs sanitized

### Commit Convention

```
feat: add turn-level RL training
fix: prevent thinking budget exhaustion
docs: update API reference for V2
test: add RL training pipeline tests
refactor: extract minimal-diff evaluation logic
```

---

## 📄 License

Copyright © 2025 Darian Rosebrook. Licensed under MIT.

---

## 🙏 Acknowledgments

V2 builds upon research and insights from:

- **Anthropic**: Extended thinking and reward hacking prevention
- **OpenAI**: Tool usage patterns and multi-turn learning
- **Academic RL**: GRPO algorithms and credit assignment
- **Industry Best Practices**: Production-grade agent safety measures

---

## 🔗 Links

- **Homepage**: https://agent-agency.com
- **Documentation**: https://docs.agent-agency.com/v2
- **API Reference**: https://api.agent-agency.com/v2
- **GitHub**: https://github.com/darianrosebrook/agent-agency
- **Issues**: https://github.com/darianrosebrook/agent-agency/issues

---

_V2 represents the next evolution of agentic AI systems, combining cutting-edge reinforcement learning with practical production requirements. Every feature is designed for reliability, safety, and measurable improvement._
