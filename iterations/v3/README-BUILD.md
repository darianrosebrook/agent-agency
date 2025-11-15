# Building on M1 Max MacBook Pro

**Target Hardware**: M1 Max 64GB MacBook Pro  
**Architecture**: ARM64 (aarch64-apple-darwin)

## Quick Start

1. **Run the setup script** (one-time setup):
   ```bash
   bash scripts/v3/setup/setup-m1-build-env.sh
   ```

2. **Source the environment** (every shell session):
   ```bash
   source .env.build
   ```

3. **Build using the wrapper script** (recommended):
   ```bash
   bash scripts/v3/build-with-env.sh test --workspace --all-features
   ```

## Environment Requirements

### Required Components

- ✅ **ARM64 Python** (via Homebrew: `brew install python@3.13`)
- ✅ **Xcode Command Line Tools** (`xcode-select --install`)
- ✅ **libtorch-cpu** (already in project root)
- ✅ **C++17 compatible compiler** (Apple Clang 17+)

### Critical Environment Variables

These are automatically set by `.env.build`:

```bash
LIBTORCH=/path/to/libtorch-cpu          # CPU-only libtorch
LIBTORCH_CXX11_ABI=0                     # Disable C++11 ABI
CMAKE_PREFIX_PATH=/path/to/libtorch-cpu  # CMake search path
DYLD_LIBRARY_PATH=/path/to/libtorch-cpu/lib  # macOS library path
CXXFLAGS="-std=c++17 -stdlib=libc++"    # C++17 for torch-sys
CXX=clang++                              # C++ compiler
CC=clang                                 # C compiler
```

## Why These Settings?

### C++17 Requirement

`torch-sys` requires C++17, but doesn't always detect it correctly. Setting `CXXFLAGS` explicitly ensures the compiler uses C++17.

### libtorch-cpu vs libtorch

- **libtorch-cpu**: CPU-only version, no CUDA dependencies, smaller footprint
- **libtorch**: Full version with CUDA support (not needed on macOS)

Using `libtorch-cpu` avoids CUDA-related compilation issues.

### ARM64 Python

CoreML and torch integrations require ARM64 Python to match the M1 architecture. x86_64 Python (via Rosetta) can cause linking issues.

## Troubleshooting

### torch-sys C++17 Errors

**Symptom**: `error: C++17 or later compatible compiler is required`

**Fix**: Ensure `CXXFLAGS` is set:
```bash
export CXXFLAGS="-std=c++17 -stdlib=libc++"
export CXX="clang++"
```

### CoreML Linker Errors

**Symptom**: `ld: library not found for -lCoreMLBridge`

**Fix**: Ensure Swift bridge is built:
```bash
cd models/languages/swift/coreml-bridge
swift build -c release
```

### Wrong Python Architecture

**Symptom**: Linking errors or architecture mismatches

**Check**:
```bash
python3 -c "import platform; print(platform.machine())"
# Should output: arm64
```

**Fix**: Install ARM64 Python via Homebrew:
```bash
brew install python@3.13
```

## Permanent Setup

Add to your `~/.zshrc` or `~/.bashrc`:

```bash
# Agent Agency M1 Max Build Environment
if [ -f "$HOME/Desktop/Projects/agent-agency/.env.build" ]; then
    source "$HOME/Desktop/Projects/agent-agency/.env.build"
fi
```

Or use the build wrapper script which automatically sources the environment:

```bash
# Instead of: cargo test
bash scripts/v3/build-with-env.sh test --workspace --all-features
```

## Verification

Test that everything is configured correctly:

```bash
# 1. Check environment
env | grep -E "(LIBTORCH|CXX|PYTHON)"

# 2. Verify Python architecture
python3 -c "import platform; print(f'Python: {platform.machine()}')"

# 3. Test compilation
cd iterations/v3
cargo check --workspace --all-features
```

## Related Documentation

- `docs/libtorch-integration.md` - Detailed libtorch setup guide
- `scripts/v3/setup/setup-m1-build-env.sh` - Automated setup script
- `scripts/v3/build-with-env.sh` - Build wrapper with environment

