# M1 Max MacBook Pro Build Setup - Permanent Solution

**Target**: M1 Max 64GB MacBook Pro  
**Architecture**: ARM64 (aarch64-apple-darwin)  
**Last Updated**: $(date)

## Problem Statement

CoreML and torch-sys compilation issues repeatedly occur because:

1. Environment variables aren't set permanently
2. C++17 flags aren't configured for torch-sys
3. LIBTORCH points to wrong location
4. Python architecture mismatches

## Permanent Solution

### One-Time Setup

Run the automated setup script:

```bash
bash scripts/v3/setup/setup-m1-build-env.sh
```

This creates `.env.build` with all required environment variables.

### Every Shell Session

Source the environment file:

```bash
source .env.build
```

Or add to your `~/.zshrc`:

```bash
# Agent Agency M1 Max Build Environment
if [ -f "$HOME/Desktop/Projects/agent-agency/.env.build" ]; then
    source "$HOME/Desktop/Projects/agent-agency/.env.build"
fi
```

### Build Commands

**Option 1: Use the build wrapper** (recommended - automatically sources environment):

```bash
bash scripts/v3/build-with-env.sh test --workspace --all-features
bash scripts/v3/build-with-env.sh check --workspace
```

**Option 2: Source environment manually**:

```bash
source .env.build
cd iterations/v3
cargo test --workspace --all-features
```

## What Gets Configured

### Environment Variables

- `LIBTORCH` → Points to `libtorch-cpu` (CPU-only, no CUDA)
- `LIBTORCH_CXX11_ABI=0` → Disables C++11 ABI for compatibility
- `CXXFLAGS="-std=c++17 -stdlib=libc++"` → **Critical for torch-sys**
- `CXX=clang++` → C++ compiler
- `CC=clang` → C compiler
- `CMAKE_PREFIX_PATH` → CMake search path
- `DYLD_LIBRARY_PATH` → macOS dynamic library path

### Cargo Configuration

`.cargo/config.toml` is updated with:

- C++17 environment variables for build scripts
- Target architecture: `aarch64-apple-darwin`
- Optimized build flags

## Verification

Before building, verify environment:

```bash
bash scripts/v3/setup/verify-build-env.sh
```

Expected output:

```
✅ LIBTORCH=/path/to/libtorch-cpu
✅ LIBTORCH_CXX11_ABI=0
✅ CXXFLAGS=-std=c++17 -stdlib=libc++
✅ Python is ARM64
✅ libtorch.dylib found
```

## PyTorch Integration (Updated)

### Current Status: LibTorch Removed

**torch-sys and libtorch have been removed** from the project. The C++17 configuration is still useful for other C++ dependencies (like CoreML Swift bridge), but PyTorch is now accessed via Python.

### If PyTorch Functionality is Needed

Use **PyO3** to call Python's PyTorch instead of libtorch:

1. **Install PyTorch in Python** (if not already installed):

   ```bash
   pip3 install torch
   ```

2. **Add PyO3 to Cargo.toml**:

   ```toml
   pyo3 = { version = "0.20", features = ["auto-initialize"] }
   ```

3. **Call PyTorch from Rust**:

   ```rust
   use pyo3::prelude::*;

   Python::with_gil(|py| {
       let torch = py.import("torch")?;
       // Use PyTorch...
   });
   ```

### Benefits of Python PyTorch

- ✅ **ARM64 Optimized**: Uses Apple Silicon optimized PyTorch
- ✅ **No C++ Compilation**: Avoids libtorch build issues
- ✅ **Standard Installation**: Standard Python package management
- ✅ **Better Compatibility**: Works seamlessly with ARM64 Python

## Why This Fixes CoreML

### Swift Bridge Requirements

CoreML requires:

- Swift runtime libraries linked correctly
- Proper rpath configuration for macOS
- ARM64 architecture matching

The `system-acceleration/build.rs` script handles this automatically when:

- `LIBTORCH` is set correctly
- C++ compiler is configured
- Xcode Command Line Tools are installed

## Troubleshooting

### torch-sys Still Fails

1. **Check CXXFLAGS is set**:

   ```bash
   echo $CXXFLAGS
   # Should show: -std=c++17 -stdlib=libc++
   ```

2. **Verify C++ compiler**:

   ```bash
   $CXX --version
   # Should show: Apple clang version 17+
   ```

3. **Test C++17 compilation**:
   ```bash
   $CXX $CXXFLAGS -x c++ - -o /dev/null <<< "int main() { return 0; }"
   ```

### CoreML Linker Errors

1. **Check Swift bridge is built**:

   ```bash
   ls -la models/languages/swift/coreml-bridge/.build/*/release/libCoreMLBridge.a
   ```

2. **Rebuild Swift bridge**:
   ```bash
   cd models/languages/swift/coreml-bridge
   swift build -c release
   ```

### Python Architecture Mismatch

**Check**:

```bash
python3 -c "import platform; print(platform.machine())"
# Should output: arm64
```

**Fix**: Install ARM64 Python:

```bash
brew install python@3.13
# Verify it's ARM64
/opt/homebrew/bin/python3 -c "import platform; print(platform.machine())"
```

## Integration with CI/CD

For CI/CD pipelines, set environment variables:

```yaml
# GitHub Actions example
env:
  LIBTORCH: ${{ github.workspace }}/libtorch-cpu
  LIBTORCH_CXX11_ABI: 0
  CXXFLAGS: "-std=c++17 -stdlib=libc++"
  CXX: clang++
  CC: clang
  CMAKE_PREFIX_PATH: ${{ github.workspace }}/libtorch-cpu
  DYLD_LIBRARY_PATH: ${{ github.workspace }}/libtorch-cpu/lib
```

## Files Created

- `.env.build` - Environment variables (git-ignored, generated)
- `scripts/v3/setup/setup-m1-build-env.sh` - Setup script
- `scripts/v3/setup/verify-build-env.sh` - Verification script
- `scripts/v3/build-with-env.sh` - Build wrapper
- `iterations/v3/.cargo/config.toml` - Cargo configuration (updated)
- `iterations/v3/README-BUILD.md` - Build documentation

## Quick Reference

**Setup** (one-time):

```bash
bash scripts/v3/setup/setup-m1-build-env.sh
```

**Verify** (before building):

```bash
bash scripts/v3/setup/verify-build-env.sh
```

**Build** (use wrapper):

```bash
bash scripts/v3/build-with-env.sh test --workspace --all-features
```

**Or manually**:

```bash
source .env.build
cd iterations/v3
cargo test --workspace --all-features
```

## Prevention

To prevent these issues from recurring:

1. **Always use the build wrapper** for cargo commands
2. **Source .env.build** at the start of shell sessions
3. **Run verify-build-env.sh** before important builds
4. **Document any new dependencies** that require special configuration

## Related Documentation

- `docs/libtorch-integration.md` - Detailed libtorch setup
- `iterations/v3/README-BUILD.md` - Build instructions
- `scripts/v3/setup/setup-m1-build-env.sh` - Setup script source
