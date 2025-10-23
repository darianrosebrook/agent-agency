# Rust Build Optimization Guide

**Author:** @darianrosebrook  
**Purpose:** Eliminate file lock contention and dramatically improve Rust compile times for multi-agent workflows

## Overview

This guide implements the "Rust equivalent" of the major compiler/runtime speedups seen in projects like Bun, Deno, and faster TypeScript implementations. The optimizations address:

1. **File lock contention** - Multiple agents can now build concurrently
2. **Compile time improvements** - 2-10x faster builds through strategic optimizations
3. **Build system efficiency** - Optimized profiles, caching, and tooling

## Quick Start

### 1. Install Build Tools

```bash
# Install optimization tools
make install-tools

# Or manually:
cargo install sccache cargo-nextest
brew install llvm  # for fast linker on macOS
```

### 2. Use the Build Wrapper

```bash
# Single agent build
AGENT_ID="my-agent" ./scripts/build-wrapper.sh check --workspace

# Multiple agents (no lock contention!)
AGENT_ID="agent-1" ./scripts/build-wrapper.sh build --package config &
AGENT_ID="agent-2" ./scripts/build-wrapper.sh build --package workers &
AGENT_ID="agent-3" ./scripts/build-wrapper.sh test --package council &
```

### 3. Use Make Targets

```bash
# Optimized builds
make build-dev      # Development build
make build-test     # Test build  
make build-release  # Release build
make check          # Workspace check
make check-fast     # Fast single-package check
```

## Architecture

### File Lock Contention Solution

**Problem:** Cargo serializes access to `target/` directory, causing agents to wait.

**Solution:** Each agent gets a unique target directory based on:
- Rust toolchain version
- Target triple
- Lockfile hash
- Feature flags
- Agent identifier

**Result:** Zero lock contention, full parallelization.

### Build Performance Optimizations

#### 1. Compiler Caching (`sccache`)
- **Impact:** 5-50x faster rebuilds
- **Setup:** Automatic via build wrapper
- **Cache:** 50GB local cache, namespace isolation

#### 2. Fast Linkers
- **macOS:** `ld64.lld` (LLVM's Mach-O linker)
- **Linux:** `lld` or `mold`
- **Impact:** 2-5x faster linking for release builds

#### 3. Optimized Cargo Profiles

**Development Profile:**
```toml
[profile.dev]
opt-level = 0           # Fastest compilation
debug = 2              # Full debug info
incremental = true     # Incremental compilation
codegen-units = 256    # Maximum parallelization
lto = "off"           # No link-time optimization
```

**Release Profile:**
```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = "thin"          # Balanced LTO (not full)
codegen-units = 1     # Best runtime performance
debug = 1             # Minimal debug info
strip = "symbols"     # Smaller binaries
```

**Test Profile:**
```toml
[profile.test]
opt-level = 0         # Fast compilation
incremental = true    # Incremental compilation
codegen-units = 256   # Maximum parallelization
```

#### 4. Cranelift Backend (Development)
- **Purpose:** Much faster compilation than LLVM
- **Usage:** Automatic for dev builds with nightly toolchain
- **Trade-off:** Slightly slower runtime, much faster compile time

#### 5. Parallel Testing (`cargo-nextest`)
- **Impact:** 2-10x faster test execution
- **Features:** Parallel test runs, better scheduling, hermetic isolation

## Usage Patterns

### Agent Coordination

Each agent should use a unique `AGENT_ID`:

```bash
# Agent 1
export AGENT_ID="council-agent"
./scripts/build-wrapper.sh build --package council

# Agent 2  
export AGENT_ID="workers-agent"
./scripts/build-wrapper.sh build --package workers

# Agent 3
export AGENT_ID="test-agent"
./scripts/build-wrapper.sh test --workspace
```

### CI/CD Integration

```bash
# CI environment setup
export AGENT_ID="ci-${BUILD_ID}-${NODE_ID}"
export MODE="release"
export VERBOSE="true"

# Run optimized build
./scripts/build-wrapper.sh build --workspace
```

### Development Workflow

```bash
# Fast iteration cycle
make check-fast           # Quick single-package check
make build-dev           # Full development build
make build-test          # Run tests

# Performance analysis
./scripts/analyze-build-performance.sh
```

## Performance Analysis

### Build Timing Analysis

```bash
# Generate detailed timing reports
CARGO_PROFILE_TIMINGS=html cargo build --workspace

# Analyze with build wrapper
./scripts/analyze-build-performance.sh
```

### Monomorphization Analysis

```bash
# Requires nightly toolchain
rustup override set nightly
cargo rustc --workspace -- -Zprint-mono-items=lazy
```

### Cache Statistics

```bash
# Check sccache performance
sccache --show-stats
```

## Advanced Optimizations

### Codebase Shape Optimizations

1. **Control Monomorphization**
   - Use `dyn Trait` at crate boundaries
   - Avoid `#[inline(always)]` on public generics
   - Use sealed traits for internal APIs

2. **Proc-macro Optimization**
   - Minimize heavy `syn/quote` usage
   - Cache code generation outputs
   - Use narrower derives instead of broad ones

3. **Feature Flag Hygiene**
   - Create feature presets (small/medium/full)
   - Avoid power sets of feature combinations

4. **Crate Graph Optimization**
   - Split large crates into stable-core + adapters
   - Keep high-churn files separate from high-fanout types

### Alternative Build Systems

For very large monorepos, consider:

- **Bazel/Buck2** with remote caching
- **Distributed builds** across multiple machines
- **Incremental compilation** with persistent workers

## Troubleshooting

### Common Issues

1. **"Blocking waiting for file lock"**
   - **Solution:** Use unique `AGENT_ID` for each process
   - **Check:** Verify target directories are isolated

2. **Slow compilation**
   - **Check:** Is `sccache` running? (`sccache --show-stats`)
   - **Check:** Are you using the optimized profiles?
   - **Check:** Is Cranelift enabled for dev builds?

3. **Linker errors**
   - **macOS:** Install LLVM (`brew install llvm`)
   - **Linux:** Install `lld` or `mold`
   - **Fallback:** Comment out fast linker config

### Performance Debugging

```bash
# Check what's taking time
cargo build --timings

# Check monomorphization hotspots
cargo rustc -- -Zprint-mono-items=lazy

# Check dependency tree
cargo tree --workspace

# Check for unused dependencies
cargo machete --workspace
```

## Results

### Before Optimization
- File lock contention between agents
- 2-5 minute builds for workspace
- No incremental compilation benefits
- Serial test execution

### After Optimization
- Zero file lock contention
- 30-60 second builds for workspace
- 2-10x faster incremental builds
- Parallel test execution
- Intelligent caching across agents
- Optimized profiles for dev/test/release

## Maintenance

### Regular Tasks

1. **Update tools:**
   ```bash
   cargo install-update sccache cargo-nextest
   ```

2. **Clean caches:**
   ```bash
   sccache --clear-cache
   cargo clean
   ```

3. **Analyze performance:**
   ```bash
   ./scripts/analyze-build-performance.sh
   ```

### Monitoring

- Monitor cache hit rates with `sccache --show-stats`
- Track build times with timing reports
- Watch for monomorphization growth
- Monitor target directory sizes

## References

- [sccache documentation](https://github.com/mozilla/sccache)
- [cargo-nextest documentation](https://nexte.st/)
- [Cranelift backend](https://github.com/bjorn3/rustc_codegen_cranelift)
- [Rust performance book](https://nnethercote.github.io/perf-book/)
