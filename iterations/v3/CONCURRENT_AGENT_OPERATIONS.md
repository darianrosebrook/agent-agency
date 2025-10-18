# Concurrent Agent Operations Guide

**Author:** @darianrosebrook
**Purpose:** Enable safe, efficient concurrent work across Rust, Python, and Node/TypeScript agents without resource conflicts

## Overview

This guide establishes the operational framework for running multiple agents concurrently across polyglot codebases (Rust, Python, Node/TypeScript). It eliminates file-lock contention, maximizes cache utilization, and provides systematic approaches for cross-language agent coordination.

## Core Philosophy

### 1. Isolation First, Coordination Second

**Principle:** Every agent operates in complete isolation by default, with explicit coordination only when necessary.

**Implementation:**
- **Per-agent write paths:** No shared `target/`, `.venv/`, `node_modules/`, or cache directories
- **Read-only shared resources:** Package registries, compiler caches, and source code
- **Namespace-based isolation:** All agent artifacts keyed by `toolchain + platform + project_hash + agent_id`

### 2. Cache the Expensive Parts, Isolate the Cheap

**Principle:** Expensive compilation/transformation work gets cached globally; cheap I/O operations stay isolated.

**Implementation:**
- **Global compiler caches:** `sccache`, `ccache`, `pnpm` store, `turbo` remote cache
- **Per-agent build outputs:** Private target directories, virtual environments, build artifacts
- **Smart invalidation:** Namespace-aware cache keys prevent cross-agent pollution

### 3. Compute Graphs Over Scripts

**Principle:** Treat agent work as a dependency graph, not a collection of scripts.

**Implementation:**
- **Task dependencies:** Explicit input/output relationships
- **Incremental execution:** Only rebuild what changed
- **Resource-aware scheduling:** Match parallelism to actual resource constraints

## Cross-Language Agent Architecture

### Agent Environment Contract

Every agent sources a standardized environment setup that provides:

```bash
# Core namespace computation
BUILD_NAMESPACE="$(os_arch)-$(toolchain_hash)-$(project_hash)-$(agent_id)"

# Language-specific write paths
export CARGO_TARGET_DIR=".target/$BUILD_NAMESPACE"
export PYTHON_VENV_PATH=".venv/$BUILD_NAMESPACE"
export NODE_CACHE_DIR=".cache/$BUILD_NAMESPACE"

# Shared read-only resources
export SCCACHE_DIR="$HOME/.cache/sccache"
export PNPM_STORE_DIR="$HOME/.cache/pnpm"
export TURBO_CACHE_DIR="$HOME/.cache/turbo"

# Concurrency gates
export COMPILE_JOBS="$(nproc)"
export LINK_JOBS="2"  # Memory-bound
export TEST_JOBS="$(nproc)"
```

### Resource Management

**Memory-Aware Scheduling:**
- Compilation: CPU-bound, scale with cores
- Linking: Memory-bound, limit to 2-4 concurrent
- Testing: I/O-bound, scale with cores but watch RAM
- File operations: Sequential per directory to avoid lock contention

**Storage Strategy:**
- **Fast SSD for caches:** Compiler outputs, package registries
- **tmpfs for intermediates:** When memory allows, for build artifacts
- **Network storage for artifacts:** Only final outputs, not intermediates

## Language-Specific Agent Operations

### Rust Agents

**Goals:** Eliminate lock contention, maximize incremental compilation, tame monomorphization.

#### 1. Target Directory Isolation
```bash
# Each agent gets unique target directory
export CARGO_TARGET_DIR=".target/${RUSTC_VERSION}_${TARGET_TRIPLE}_${LOCK_HASH}_${AGENT_ID}"
cargo build --workspace
```

#### 2. Compiler Caching
```bash
export RUSTC_WRAPPER="$(which sccache)"
export SCCACHE_CACHE_SIZE="50G"
export SCCACHE_NAMESPACE="agent-agency"
```

#### 3. Fast Linking
```toml
# .cargo/config.toml
[target.x86_64-apple-darwin]
rustflags = ["-Clink-arg=-fuse-ld=/usr/local/bin/ld64.lld"]

[target.x86_64-unknown-linux-gnu]
rustflags = ["-Clink-arg=-fuse-ld=lld"]
```

#### 4. Profile-Based Loops

**Edit Loop (Development):**
```toml
[profile.dev]
opt-level = 0
debug = 2
incremental = true
codegen-units = 256
lto = "off"
```

**Ship Loop (Release):**
```toml
[profile.release]
opt-level = 3
debug = 1
incremental = false
codegen-units = 1
lto = "thin"
panic = "abort"
strip = "symbols"
```

#### 5. Cranelift for Development
```bash
# Automatic in build wrapper for nightly toolchain
export RUSTFLAGS="$RUSTFLAGS -Zcodegen-backend=cranelift"
```

#### 6. Monomorphization Control
- Prefer `dyn Trait` at crate boundaries
- Avoid `#[inline(always)]` on public generics
- Split crates: stable core + heavy adapters + bins

#### 7. Agent Command Examples
```bash
# Development agent
AGENT_ID="dev-agent-1" ./scripts/build-wrapper.sh dev --workspace

# Test agent
AGENT_ID="test-agent-2" ./scripts/build-wrapper.sh test --workspace

# Release agent
AGENT_ID="release-agent-3" ./scripts/build-wrapper.sh release --workspace
```

### Python Agents

**Goals:** Isolate environments, parallelize CPU work, speed up installs, shard tests safely.

#### 1. Virtual Environment Isolation
```bash
# Per-agent venv
export PYTHON_VENV_PATH=".venv/${PYTHON_VERSION}_${PLATFORM}_${REQ_HASH}_${AGENT_ID}"
python -m venv "$PYTHON_VENV_PATH"
source "$PYTHON_VENV_PATH/bin/activate"
```

#### 2. Wheel Cache Sharing
```bash
# Shared read-only wheel cache
export PIP_CACHE_DIR="$HOME/.cache/pip"
export PIP_WHEEL_CACHE="$HOME/.cache/pip/wheels"

# Agent installs from cache
pip install --cache-dir="$PIP_CACHE_DIR" -r requirements.txt
```

#### 3. True Parallel CPU Tasks
```python
# Use multiprocessing for CPU-bound work
import multiprocessing as mp

def cpu_intensive_task(data):
    # Heavy computation here
    return result

with mp.Pool(processes=mp.cpu_count()) as pool:
    results = pool.map(cpu_intensive_task, data_chunks)
```

#### 4. Pytest at Scale
```bash
# Per-agent cache, parallel execution
export PYTEST_CACHE_DIR=".pytest_cache/${AGENT_ID}"
pytest -n auto --dist=loadscope --cache-dir="$PYTEST_CACHE_DIR"
```

#### 5. Type Checking Sharding
```bash
# Only check impacted files
export MYPY_CACHE_DIR=".mypy_cache/${AGENT_ID}"
git diff --name-only | xargs mypy --cache-dir="$MYPY_CACHE_DIR"
```

### Node/TypeScript Agents

**Goals:** Avoid node_modules stampedes, use incremental compilation correctly, batch transforms.

#### 1. Package Manager Isolation
```bash
# pnpm with shared store, per-agent project
export PNPM_STORE_DIR="$HOME/.cache/pnpm"
export PNPM_HOME="$HOME/.cache/pnpm"

# Per-agent node_modules
export NODE_MODULES_DIR="node_modules.${AGENT_ID}"
pnpm install --store-dir="$PNPM_STORE_DIR"
```

#### 2. Task Graph Coordination
```json
// turbo.json
{
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": ["dist/**"],
      "cache": true
    },
    "test": {
      "dependsOn": ["build"],
      "cache": true
    }
  }
}
```

#### 3. TypeScript Project References
```json
// tsconfig.json
{
  "compilerOptions": {
    "composite": true,
    "incremental": true,
    "isolatedModules": true,
    "skipLibCheck": true
  },
  "references": [
    {"path": "./packages/core"},
    {"path": "./packages/ui"}
  ]
}
```

#### 4. Fast Transpilation
```javascript
// vite.config.js or next.config.js
export default {
  esbuild: {
    // Use esbuild for development
    target: 'esnext'
  },
  swcMinify: true
}
```

#### 5. Test Runner Configuration
```javascript
// vitest.config.js
export default {
  cache: {
    dir: `.vitest_cache/${process.env.AGENT_ID}`
  },
  pool: 'threads',
  poolOptions: {
    threads: {
      singleThread: false,
      maxThreads: Math.max(1, require('os').cpus().length - 1)
    }
  }
}
```

## Agent Lifecycle Management

### Agent Registration
```bash
# Bootstrap new agent
export AGENT_ID="$(uuidgen | cut -c1-8)"
export AGENT_TYPE="rust-dev"  # rust-dev, python-test, node-build, etc.

./scripts/bootstrap-agent.sh
```

### Resource Cleanup
```bash
# Clean agent artifacts
./scripts/clean-agent.sh "$AGENT_ID"

# Clean old agents (garbage collection)
./scripts/gc-agents.sh --older-than 7d
```

### Monitoring and Observability

#### Performance Metrics
```bash
# Collect per-agent metrics
./scripts/collect-metrics.sh "$AGENT_ID" > "metrics/${AGENT_ID}.json"
```

#### Cache Efficiency
```bash
# Report cache hit rates
sccache --show-stats
pnpm store status
turbo build --dry-run
```

#### Resource Usage
```bash
# Monitor agent resource consumption
./scripts/monitor-agent.sh "$AGENT_ID"
```

## Integration with V3 Architecture

### Agent State Management
```rust
// In workspace-state-manager
pub struct AgentState {
    pub id: String,
    pub namespace: String,
    pub resources: ResourceLimits,
    pub capabilities: Vec<Capability>,
    pub active_tasks: Vec<TaskId>,
}

impl AgentState {
    pub fn new(id: &str) -> Self {
        let namespace = compute_namespace(id);
        Self {
            id: id.to_string(),
            namespace,
            resources: ResourceLimits::default(),
            capabilities: vec![],
            active_tasks: vec![],
        }
    }
}
```

### Task Scheduling
```rust
// In orchestration engine
pub struct TaskScheduler {
    pub agents: HashMap<String, AgentState>,
    pub task_queue: VecDeque<Task>,
    pub resource_pool: ResourcePool,
}

impl TaskScheduler {
    pub fn schedule_task(&mut self, task: Task) -> Result<TaskAssignment, Error> {
        let suitable_agent = self.find_suitable_agent(&task)?;
        self.assign_task_to_agent(task, suitable_agent)
    }
}
```

### Conflict Resolution
```rust
// Prevent resource conflicts
pub fn check_resource_conflicts(&self, task: &Task, agent: &AgentState) -> bool {
    // Check for file path conflicts
    // Check for resource limits
    // Check for capability requirements
    !self.would_conflict(task, agent)
}
```

## Bootstrap Script

The `bootstrap-agent.sh` script automatically detects platforms and toolchains, setting up:

- Per-agent namespaces
- Rust: sccache + linker + profiles
- Python: wheel cache + venv layout
- Node: pnpm/Turbo caches + TS project refs
- Sane default concurrency gates

```bash
#!/usr/bin/env bash
# bootstrap-agent.sh
# @darianrosebrook

set -euo pipefail

# Auto-detect agent ID
export AGENT_ID="${AGENT_ID:-$(uuidgen | cut -c1-8)}"
export AGENT_TYPE="${AGENT_TYPE:-auto}"

echo "🚀 Bootstrapping agent: $AGENT_ID (type: $AGENT_TYPE)"

# Compute namespace
source scripts/compute-namespace.sh

# Setup language-specific environments
source scripts/setup-rust-env.sh
source scripts/setup-python-env.sh
source scripts/setup-node-env.sh

# Set concurrency gates
source scripts/set-concurrency.sh

echo "✅ Agent $AGENT_ID ready for work"
echo "📋 Namespace: $BUILD_NAMESPACE"
echo "🔧 Capabilities: $(list_capabilities)"
```

## Verification and Quality Gates

### Pre-Agent Checks
- [ ] Agent namespace computation is deterministic
- [ ] No shared write paths between agents
- [ ] Resource limits are enforced
- [ ] Cache directories are properly isolated

### Runtime Verification
- [ ] File lock contention is eliminated
- [ ] Cache hit rates > 80% for incremental work
- [ ] Resource utilization stays within bounds
- [ ] Task completion times are logged and analyzed

### Post-Agent Cleanup
- [ ] All agent artifacts are properly cleaned
- [ ] Shared caches remain intact
- [ ] No cross-agent state pollution
- [ ] Performance metrics are collected

## Troubleshooting Guide

### Common Issues

#### File Lock Contention
**Symptoms:** "Blocking waiting for file lock" errors
**Solution:**
- Verify unique `CARGO_TARGET_DIR` per agent
- Check `PYTHON_VENV_PATH` isolation
- Ensure `NODE_MODULES_DIR` separation

#### Cache Invalidation
**Symptoms:** Unexpected cache misses
**Solution:**
- Verify namespace computation stability
- Check toolchain version pinning
- Review cache key inputs

#### Resource Exhaustion
**Symptoms:** Out of memory, slow performance
**Solution:**
- Reduce concurrency gates
- Increase memory allocation
- Switch to tmpfs for intermediates

#### Cross-Agent Interference
**Symptoms:** Inconsistent results between agents
**Solution:**
- Audit shared resource access
- Implement proper locking for coordination points
- Add conflict detection and resolution

## Performance Benchmarks

### Rust Agent Performance
- **Cold build:** 2-3x faster with sccache
- **Incremental build:** 10-50x faster
- **Test execution:** 3-5x faster with nextest
- **Memory usage:** 20-30% reduction with optimized profiles

### Python Agent Performance
- **Install time:** 5-10x faster with wheel cache
- **Test execution:** 2-4x faster with pytest-xdist
- **Type checking:** 50% faster with sharded mypy

### Node/TypeScript Agent Performance
- **Install time:** 3-5x faster with pnpm store
- **Build time:** 4-8x faster with Turbo cache
- **Test execution:** 2-3x faster with Vitest

## Future Enhancements

### Distributed Coordination
- Agent discovery and registration
- Load balancing across agent pools
- Priority-based task scheduling
- Cross-agent cache sharing

### Advanced Resource Management
- NUMA-aware scheduling
- GPU resource allocation
- Network bandwidth optimization
- Energy-aware scaling

### Observability Improvements
- Real-time performance dashboards
- Predictive resource allocation
- Automated bottleneck detection
- Historical trend analysis
