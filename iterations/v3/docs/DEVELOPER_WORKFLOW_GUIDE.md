# Developer Workflow Guide

**Author:** @darianrosebrook  
**Purpose:** Complete guide for developers working with Agent Agency v3 system

---

## Quick Start

### Prerequisites

- **Rust 1.70+** with Cargo
- **Node.js 18+** with npm/pnpm
- **Git** for version control
- **macOS** (Apple Silicon recommended) or Linux
- **32GB+ RAM** recommended for local model execution

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/agent-agency.git
cd agent-agency/iterations/v3

# Install dependencies
cargo build --workspace
npm install

# Initialize CAWS (Node.js tool)
cd apps/tools/caws
node cli.js init --interactive

# Verify installation
cargo check --workspace
```

## Core Workflows

### 1. Autonomous Task Execution

The primary workflow for running autonomous agents on development tasks.

#### Basic Task Execution

```bash
# Execute a task via API (CLI binary temporarily disabled)
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task": "Add user authentication to the API",
    "scope": {"in": ["src/api/", "tests/api/"]},
    "risk_tier": 2
  }'

# Execute with specific execution mode
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task": "Implement caching layer",
    "execution_mode": "auto"
  }'
```

#### Task Configuration

Create a working specification for complex tasks:

```yaml
# .caws/working-spec.yaml
id: FEAT-001
title: "Add user authentication flow"
risk_tier: 2
mode: feature
change_budget:
  max_files: 25
  max_loc: 1000
blast_radius:
  modules: ["auth", "api"]
  data_migration: false
operational_rollback_slo: "5m"
scope:
  in: ["src/auth/", "tests/auth/", "package.json"]
  out: ["src/billing/", "node_modules/"]
invariants:
  - "System maintains data consistency during rollback"
  - "Authentication state is never stored in localStorage"
  - "All auth tokens expire within 24h"
acceptance:
  - id: "A1"
    given: "User is logged out"
    when: "User submits valid credentials"
    then: "User is logged in and redirected to dashboard"
  - id: "A2"
    given: "User has invalid session token"
    when: "User attempts to access protected route"
    then: "User is redirected to login with error message"
```

#### Execution Modes

**Strict Mode** (Human-in-the-loop):
```bash
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task": "Refactor database layer",
    "execution_mode": "strict",
    "require_approval": true
  }'
```

**Autonomous Mode** (Fully automated):
```bash
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task": "Add unit tests for user service",
    "execution_mode": "auto",
    "max_iterations": 5
  }'
```

**Dry Run Mode** (Planning only):
```bash
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task": "Implement new feature",
    "execution_mode": "dry-run",
    "output_plan": true
  }'
```

### 2. Multi-Agent Orchestration

For complex tasks requiring multiple specialized agents.

#### Worker Pool Management

```bash
# Start worker service
cargo run --bin agent-workers

# Submit orchestrated task via API
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task": "Build full-stack application",
    "coordination_strategy": "sequential",
    "quality_gates": "all"
  }'
```

#### Specialized Agent Execution

```bash
# Submit tasks with specific requirements via API
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task": "Implement React components",
    "required_tools": ["frontend"],
    "metadata": {"framework": "react"}
  }'

# Backend specialist  
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task": "Design API endpoints",
    "required_tools": ["backend"],
    "metadata": {"framework": "rust"}
  }'

# Database specialist
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task": "Design database schema",
    "required_tools": ["database"],
    "metadata": {"engine": "postgresql"}
  }'
```

### 3. Quality Assurance Workflows

#### Automated Testing

```bash
# Run full test suite
cargo test --workspace

# Run with coverage
cargo run --bin test-runner -- \
  --coverage \
  --threshold 80 \
  --mutation-testing

# Run specific test categories
cargo run --bin test-runner -- \
  --category unit \
  --category integration \
  --parallel
```

#### Code Quality Gates

```bash
# Run CAWS validation (Node.js tool)
cd apps/tools/caws
node validate.js .caws/working-spec.yaml

# Run linting and formatting
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Run type checking
cargo check --workspace
```

#### Performance Testing

```bash
# Run performance benchmarks
cargo run --bin benchmark -- \
  --suite "api-performance" \
  --duration 60s \
  --concurrent-users 100

# Run load testing
cargo run --bin load-tester -- \
  --scenario "user-registration" \
  --ramp-up 30s \
  --sustained-load 5m
```

### 4. Monitoring and Observability

#### Real-time Monitoring

```bash
# Start monitoring dashboard
cargo run --bin dashboard -- \
  --port 8080 \
  --metrics \
  --logs \
  --traces

# Monitor specific agents
cargo run --bin monitor -- \
  --agent-id "agent-001" \
  --metrics "cpu,memory,throughput" \
  --alerts
```

#### System Health Checks

```bash
# Comprehensive health check
cargo run --bin health-check -- \
  --components "all" \
  --detailed \
  --output json

# Check specific components
cargo run --bin health-check -- \
  --components "database,redis,models" \
  --timeout 30s
```

#### Log Analysis

```bash
# Analyze agent logs
cargo run --bin log-analyzer -- \
  --input "logs/agent-*.log" \
  --analysis "performance,errors,patterns" \
  --output "analysis-report.json"

# Monitor real-time logs
cargo run --bin log-monitor -- \
  --follow \
  --filter "ERROR,WARN" \
  --format structured
```

## Advanced Workflows

### 1. Custom Model Integration

#### Local Model Setup

```bash
# Download and configure local model (if model-manager binary exists)
# Note: Verify binary availability before use
# Alternative: Use API endpoints or manual model configuration

# Test model performance
cargo run --bin model-benchmark -- \
  --model "llama3.1:8b" \
  --benchmark "inference-speed" \
  --iterations 100
```

#### Model Hot-Swapping

```bash
# Switch models during execution
cargo run --bin model-switcher -- \
  --from "gpt-4" \
  --to "llama3.1:8b" \
  --preserve-context

# A/B test models
cargo run --bin model-comparator -- \
  --model-a "gpt-4" \
  --model-b "llama3.1:8b" \
  --task "code-generation" \
  --iterations 50
```

### 2. Workspace Management

#### Workspace Detection

```bash
# Detect and configure workspace
cargo run --bin workspace-detector -- \
  --auto-detect \
  --configure \
  --safety-mode "strict"

# Validate workspace setup
cargo run --bin workspace-validator -- \
  --check-permissions \
  --check-git \
  --check-dependencies
```

#### File Operations

```bash
# Safe file operations with allow lists
cargo run --bin file-manager -- \
  --operation "edit" \
  --path "src/main.rs" \
  --allow-list "src/**" \
  --backup

# Batch file operations
cargo run --bin file-manager -- \
  --operation "batch-edit" \
  --pattern "*.rs" \
  --transform "add-documentation" \
  --dry-run
```

### 3. Integration Testing

#### End-to-End Testing

```bash
# Run full E2E test suite
cargo run --bin e2e-tester -- \
  --scenarios "all" \
  --browser "chrome" \
  --parallel 4

# Test specific user journeys
cargo run --bin e2e-tester -- \
  --scenario "user-registration-flow" \
  --scenario "payment-processing" \
  --headless
```

#### API Testing

```bash
# Test API endpoints
cargo run --bin api-tester -- \
  --base-url "http://localhost:3000" \
  --endpoints "all" \
  --authentication "bearer-token" \
  --rate-limit 100
```

## Configuration Management

### Environment Configuration

```bash
# Development environment
export AGENT_AGENCY_ENV=development
export AGENT_AGENCY_LOG_LEVEL=debug
export AGENT_AGENCY_MODEL=coreml:mistral-7b

# Production environment
export AGENT_AGENCY_ENV=production
export AGENT_AGENCY_LOG_LEVEL=info
export AGENT_AGENCY_MODEL=openai:gpt-4
export AGENT_AGENCY_SAFETY_MODE=strict
```

### CAWS Configuration

```yaml
# .caws/config.yaml
caws:
  version: "3.1.0"
  environment: "development"
  
  quality_gates:
    coverage_threshold: 80
    mutation_score_threshold: 70
    linting_errors_max: 0
    
  safety:
    allow_list:
      - "src/**"
      - "tests/**"
      - "docs/**"
    budget_limits:
      max_files: 100
      max_loc: 10000
      
  models:
    default: "coreml:mistral-7b"
    fallback: "openai:gpt-4"
    local_optimization: true
```

### Agent Configuration

```yaml
# .caws/agent-config.yaml
agents:
  default:
    max_iterations: 5
    timeout: 300s
    safety_mode: "strict"
    
  specialized:
    frontend:
      framework: "react"
      tools: ["npm", "webpack", "jest"]
    backend:
      framework: "rust"
      tools: ["cargo", "clippy", "cargo-test"]
    database:
      engine: "postgresql"
      tools: ["psql", "migrate", "pg_dump"]
```

## Troubleshooting

### Common Issues

#### Agent Execution Failures

```bash
# Check agent status via API
curl http://localhost:8080/api/v1/tasks/agent-001/status

# View agent logs via API
curl http://localhost:8080/api/v1/tasks/agent-001/logs?tail=100

# Restart failed agent via API
curl -X POST http://localhost:8080/api/v1/tasks/agent-001/restart
```

#### Model Loading Issues

```bash
# Check model status via API
curl http://localhost:8080/api/v1/models/llama3.1:8b/status

# Test model connectivity (if model-tester binary exists)
# Note: Verify binary availability before use

# Reload model via API (if endpoint exists)
curl -X POST http://localhost:8080/api/v1/models/llama3.1:8b/reload
```

#### Workspace Issues

```bash
# Check workspace permissions
cargo run --bin workspace-checker -- --check-permissions

# Reset workspace state
cargo run --bin workspace-reset -- --confirm

# Validate workspace configuration
cargo run --bin workspace-validator -- --full-check
```

### Debugging Tools

#### System Diagnostics

```bash
# Full system diagnostic
cargo run --bin system-diagnostic -- \
  --components "all" \
  --output "diagnostic-report.json" \
  --verbose

# Performance profiling
cargo run --bin profiler -- \
  --duration 60s \
  --output "profile.json" \
  --components "cpu,memory,io"
```

#### Agent Debugging

```bash
# Debug specific agent
cargo run --bin agent-debugger -- \
  --agent-id "agent-001" \
  --breakpoints "task-start,task-end" \
  --step-through

# Trace agent execution
cargo run --bin agent-tracer -- \
  --agent-id "agent-001" \
  --output "trace.json" \
  --detailed
```

## Best Practices

### Development Workflow

1. **Start with Dry Run:** Always test with `--execution-mode dry-run` first
2. **Use Strict Mode:** Begin with human approval for safety
3. **Monitor Progress:** Keep dashboard open during execution
4. **Review Changes:** Always review agent-generated code
5. **Test Thoroughly:** Run full test suite after agent work

### Safety Guidelines

1. **Allow Lists:** Always configure strict allow lists
2. **Budget Limits:** Set appropriate file and LOC limits
3. **Backup Strategy:** Ensure rollback capability
4. **Quality Gates:** Enforce testing and linting
5. **Human Oversight:** Maintain human review for critical changes

### Performance Optimization

1. **Model Selection:** Choose appropriate models for tasks
2. **Parallel Execution:** Use worker pools for large tasks
3. **Caching:** Enable model and workspace caching
4. **Resource Monitoring:** Monitor CPU, memory, and disk usage
5. **Cleanup:** Regular cleanup of temporary files and caches

---

This guide provides the foundation for developers to effectively use the Agent Agency v3 system while maintaining safety, quality, and performance standards.

