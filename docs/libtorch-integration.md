# PyTorch Integration Guide

**Document Version**: 2.0.0
**Last Updated**: November 15, 2025
**Status**: LibTorch removed - Use PyTorch via Python instead

---

## Current Status

**LibTorch and torch-sys have been removed** from the project to eliminate compilation issues.

### Why LibTorch Was Removed

1. **Version Conflicts**: `torch-sys` version incompatibilities with libtorch headers
2. **Unused Dependency**: `rust-bert` (which pulled in torch-sys) was optional and not used
3. **Better Alternative**: PyTorch via Python provides better ARM64 support and avoids C++ compilation issues

## Recommended Approach: PyTorch via Python

If PyTorch functionality is needed, use **PyO3** to call Python's PyTorch from Rust:

### Benefits

- ✅ **ARM64 Native**: Uses ARM64 Python PyTorch (already installed via Homebrew)
- ✅ **No C++ Compilation**: Avoids libtorch C++ build issues
- ✅ **Better Performance**: Python PyTorch optimized for Apple Silicon
- ✅ **Easier Maintenance**: Standard Python package management

### Implementation Pattern

```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;

fn call_pytorch() -> PyResult<()> {
    Python::with_gil(|py| {
        // Import PyTorch
        let torch = py.import("torch")?;

        // Call PyTorch functions
        let tensor = torch.call_method1("tensor", (vec![1.0, 2.0, 3.0],))?;

        Ok(())
    })
}
```

## Historical Context (Deprecated)

The following sections document the previous libtorch integration approach, which is no longer used.

### Required Components

- **libtorch-cpu**: CPU-only PyTorch library (already present in project root)
- **Environment Variables**: Specific configuration for C++ compilation
- **Dynamic Library Path**: macOS library resolution

## Step-by-Step Fix

### 1. Verify libtorch-cpu Installation

Ensure you have the CPU-only version in your project root:

```bash
ls -la libtorch-cpu/
# Should show: bin/, include/, lib/, share/ directories
```

### 2. Set Environment Variables

**Critical**: These must be set BEFORE running `cargo check` or `cargo build`.

```bash
# Point to CPU-only libtorch
export LIBTORCH=/Users/darianrosebrook/Desktop/Projects/agent-agency/libtorch-cpu

# Disable C++11 ABI (required for compatibility)
export LIBTORCH_CXX11_ABI=0

# Set CMake prefix path
export CMAKE_PREFIX_PATH=/Users/darianrosebrook/Desktop/Projects/agent-agency/libtorch-cpu

# macOS dynamic library resolution
export DYLD_LIBRARY_PATH=/Users/darianrosebrook/Desktop/Projects/agent-agency/libtorch-cpu/lib:$DYLD_LIBRARY_PATH
```

### 3. Verify Environment

```bash
echo "LIBTORCH: $LIBTORCH"
echo "LIBTORCH_CXX11_ABI: $LIBTORCH_CXX11_ABI"
echo "CMAKE_PREFIX_PATH: $CMAKE_PREFIX_PATH"
echo "DYLD_LIBRARY_PATH: $DYLD_LIBRARY_PATH"
```

Expected output:

```
LIBTORCH: /Users/darianrosebrook/Desktop/Projects/agent-agency/libtorch-cpu
LIBTORCH_CXX11_ABI: 0
CMAKE_PREFIX_PATH: /Users/darianrosebrook/Desktop/Projects/agent-agency/libtorch-cpu
DYLD_LIBRARY_PATH: /Users/darianrosebrook/Desktop/Projects/agent-agency/libtorch-cpu/lib:
```

### 4. Test the Fix

Build the problematic crate:

```bash
cargo check -p agent-model-management --lib
```

Success indicators:

- No C++ compilation errors
- Clean Rust compilation
- Warnings about "the following packages contain code that will be rejected by a future version of Rust" are OK

## Troubleshooting

### Environment Variables Not Set

**Symptom**: Same C++ errors persist

**Solution**: Double-check all environment variables are exported and have correct values:

```bash
env | grep -E "(LIBTORCH|CMAKE|DYLD)"
```

### Wrong libtorch Version

**Symptom**: Still using CUDA or full libtorch

**Check**: Ensure `LIBTORCH` points to `libtorch-cpu`, not `libtorch`:

```bash
ls -la $LIBTORCH/lib/ | grep -E "(cuda|gpu)" || echo "CPU-only confirmed"
```

### macOS Library Path Issues

**Symptom**: Runtime linking errors

**Solution**: Verify `DYLD_LIBRARY_PATH` includes the libtorch library directory:

```bash
otool -L $LIBTORCH/lib/libtorch.dylib 2>/dev/null || echo "Check DYLD_LIBRARY_PATH"
```

## Integration with Build Scripts

For permanent integration, add environment setup to your build scripts:

### Shell Script (build.sh)

```bash
#!/bin/bash
set -e

# LibTorch environment setup
export LIBTORCH="$(pwd)/libtorch-cpu"
export LIBTORCH_CXX11_ABI=0
export CMAKE_PREFIX_PATH="$LIBTORCH"
export DYLD_LIBRARY_PATH="$LIBTORCH/lib:$DYLD_LIBRARY_PATH"

echo "Building with LibTorch: $LIBTORCH"
cargo build "$@"
```

### CI/CD Integration

Add to your CI pipeline:

```yaml
# GitHub Actions example
- name: Setup LibTorch
  run: |
    echo "LIBTORCH=${{ github.workspace }}/libtorch-cpu" >> $GITHUB_ENV
    echo "LIBTORCH_CXX11_ABI=0" >> $GITHUB_ENV
    echo "CMAKE_PREFIX_PATH=${{ github.workspace }}/libtorch-cpu" >> $GITHUB_ENV
    echo "DYLD_LIBRARY_PATH=${{ github.workspace }}/libtorch-cpu/lib:$DYLD_LIBRARY_PATH" >> $GITHUB_ENV
```

## Technical Details

### Why CPU-Only?

- **Compatibility**: Avoids CUDA-related compilation issues on macOS
- **Size**: Significantly smaller footprint (~500MB vs ~2GB+)
- **Dependencies**: No GPU driver requirements
- **Cross-platform**: Works consistently across different macOS environments

### Environment Variable Explanations

| Variable             | Purpose                                 | Value                   | Required |
| -------------------- | --------------------------------------- | ----------------------- | -------- |
| `LIBTORCH`           | Root directory of libtorch installation | `/path/to/libtorch-cpu` | Yes      |
| `LIBTORCH_CXX11_ABI` | C++ ABI version compatibility           | `0`                     | Yes      |
| `CMAKE_PREFIX_PATH`  | CMake find_package() search path        | Same as LIBTORCH        | Yes      |
| `DYLD_LIBRARY_PATH`  | macOS dynamic library search path       | `$LIBTORCH/lib`         | Yes      |

### File Structure Verification

Correct `libtorch-cpu` structure:

```
libtorch-cpu/
├── bin/
│   └── torch_shm_manager
├── include/
│   ├── ATen/
│   ├── c10/
│   ├── torch/
│   └── torchvision/
├── lib/
│   ├── libc10.dylib
│   ├── libtorch.dylib
│   ├── libtorch_cpu.dylib
│   └── pkgconfig/
└── share/
    └── cmake/
```

## Common Pitfalls

### 1. Using Full LibTorch Instead of CPU

```bash
# WRONG - Uses CUDA version
export LIBTORCH=/path/to/libtorch

# RIGHT - Uses CPU-only
export LIBTORCH=/path/to/libtorch-cpu
```

### 2. Missing DYLD_LIBRARY_PATH

macOS won't find the dynamic libraries without this path set.

### 3. Environment Variables Not Exported

Variables must be `export`ed to be visible to subprocesses:

```bash
# WRONG - Not exported
LIBTORCH=/path/to/libtorch

# RIGHT - Exported
export LIBTORCH=/path/to/libtorch
```

### 4. Path Resolution Issues

Always use absolute paths:

```bash
# WRONG - Relative path
export LIBTORCH=./libtorch-cpu

# RIGHT - Absolute path
export LIBTORCH=/full/path/to/libtorch-cpu
```

## Verification Commands

### Check LibTorch Installation

```bash
# Verify library exists
ls -la $LIBTORCH/lib/libtorch.dylib

# Check C++ headers
ls -la $LIBTORCH/include/torch/

# Verify no CUDA components
find $LIBTORCH -name "*cuda*" -o -name "*gpu*" | wc -l  # Should be 0
```

### Check Environment

```bash
# All variables set
env | grep -E "^(LIBTORCH|CMAKE_PREFIX_PATH|DYLD_LIBRARY_PATH|LIBTORCH_CXX11_ABI)="

# Paths exist
test -d "$LIBTORCH" && echo "LIBTORCH directory exists" || echo "LIBTORCH missing"
test -d "$LIBTORCH/lib" && echo "lib directory exists" || echo "lib missing"
```

### Test Compilation

```bash
# Quick test
cargo check -p agent-model-management --lib --quiet

# Full workspace test
cargo check --workspace --exclude agent-model-management --quiet
```

## References

- [PyTorch C++ Distribution](https://pytorch.org/cppdist/)
- [torch-sys Crate Documentation](https://docs.rs/torch-sys/)
- [Rust CUDA Issues](https://github.com/LaurentMazare/tch-rs/issues)
- [macOS LibTorch Setup](https://github.com/LaurentMazare/tch-rs/blob/main/examples/macOS-setup.md)

---

## Quick Reference

**One-liner setup** (add to your shell profile or build script):

```bash
# LibTorch configuration
export LIBTORCH="$(pwd)/libtorch-cpu"
export LIBTORCH_CXX11_ABI=0
export CMAKE_PREFIX_PATH="$(pwd)/libtorch-cpu"
export DYLD_LIBRARY_PATH="$(pwd)/libtorch-cpu/lib:$DYLD_LIBRARY_PATH"

# C++17 flags (required for torch-sys)
export CXXFLAGS="-std=c++17 -stdlib=libc++"
export CXX="clang++"
export CC="clang"
```

**Or use the automated setup script**:

```bash
bash scripts/v3/setup/setup-m1-build-env.sh
source .env.build
```

**Test command**:

```bash
cargo check -p agent-model-management --lib
```

**Success indicators**:

- ✅ No C++ compilation errors
- ✅ Rust code compiles cleanly
- ✅ No "ToolExecError" messages

---

_This documentation was created to prevent future agents from encountering the same libtorch integration issues. Always verify your environment variables are correctly set before building torch-dependent crates._
