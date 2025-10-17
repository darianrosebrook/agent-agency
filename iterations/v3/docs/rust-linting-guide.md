# Rust Linting and Code Quality Guide

**Author:** @darianrosebrook

## Overview

This project uses a comprehensive Rust linting and code quality system to maintain high code standards across the entire workspace.

## Available Tools

### 1. **cargo check** - Basic Compilation
```bash
cargo check --workspace
```
- Checks for compilation errors
- Fastest way to verify code compiles
- Run this frequently during development

### 2. **cargo clippy** - Advanced Linting
```bash
cargo clippy --workspace --all-targets --all-features
```
- Hundreds of linting rules
- Catches common mistakes and performance issues
- Configured via `.clippy.toml`

### 3. **cargo fmt** - Code Formatting
```bash
cargo fmt --all
```
- Consistent code formatting
- Configured via `rustfmt.toml`
- Check formatting: `cargo fmt --all -- --check`

### 4. **cargo test** - Testing
```bash
cargo test --workspace
```
- Unit and integration tests
- Essential for code quality

## Convenience Scripts

### Quick Linting Check
```bash
./scripts/lint.sh
```
Runs all linting checks in sequence:
- ✅ Basic compilation check
- ✅ Clippy linting
- ✅ Formatting check
- ✅ Unit tests
- ✅ Security audit (if available)
- ✅ Dependency check (if available)

### Auto-Fix Common Issues
```bash
./scripts/fix.sh
```
Automatically fixes:
- ✅ Auto-fixable issues with `cargo fix`
- ✅ Code formatting
- ✅ Unused imports (with nightly toolchain)

## Configuration Files

### `.clippy.toml` - Clippy Configuration
- **Allows:** Some complexity for domain-specific code
- **Denies:** Critical issues (unused code, unreachable patterns)
- **Warns:** All clippy lints for comprehensive coverage

### `rustfmt.toml` - Formatting Configuration
- **Max width:** 100 characters
- **Tab spaces:** 4
- **Import organization:** Crate-level granularity
- **Comment formatting:** Enabled

## Current Project Status

### ✅ What's Working
- Basic compilation (with warnings)
- All modules compile individually
- Comprehensive linting configuration

### ❌ Issues to Fix
1. **Compilation Errors (9 total):**
   - Database module: Missing trait methods, type mismatches
   - Council module: Missing fields, undefined variables

2. **Warnings (50+ total):**
   - Unused variables and imports
   - Dead code (unused structs/methods)
   - Unreachable patterns
   - Lifetime syntax issues

## Recommended Workflow

### Daily Development
1. **Before coding:** `cargo check --workspace`
2. **During coding:** Run `cargo check` on your module
3. **Before commit:** `./scripts/lint.sh`
4. **Auto-fix issues:** `./scripts/fix.sh`

### Before Major Commits
1. Run full linting: `./scripts/lint.sh`
2. Fix any remaining issues manually
3. Ensure all tests pass
4. Format code: `cargo fmt --all`

## Priority Fixes

### High Priority (Compilation Errors)
1. **Database module errors:**
   - Fix missing `validate_worker_update` method
   - Fix type mismatches in migrations
   - Fix missing fields in structs

2. **Council module errors:**
   - Fix undefined variables (`evaluations`, `debates`)
   - Fix missing fields in `ConsensusResult`
   - Fix struct field mismatches

### Medium Priority (Warnings)
1. **Unused code cleanup:**
   - Remove or use unused struct fields
   - Remove unused methods
   - Clean up unused imports

2. **Code quality improvements:**
   - Fix unreachable patterns
   - Fix lifetime syntax issues
   - Add proper error handling

## Additional Tools (Optional)

### Security Audit
```bash
cargo install cargo-audit
cargo audit
```

### Dependency Updates
```bash
cargo install cargo-outdated
cargo outdated --workspace
```

### Nightly Features (for advanced fixes)
```bash
rustup toolchain install nightly
cargo +nightly fix --workspace --clippy
```

## IDE Integration

### VS Code
- Install "rust-analyzer" extension
- Configure to run clippy on save
- Enable format on save

### Other IDEs
- Most Rust IDEs support clippy integration
- Configure to use project's `.clippy.toml`

## Best Practices

1. **Run linting frequently** - Don't let issues accumulate
2. **Fix warnings promptly** - They often indicate real problems
3. **Use auto-fix when possible** - `./scripts/fix.sh` handles many issues
4. **Review clippy suggestions** - They often improve code quality
5. **Keep dependencies updated** - Use `cargo audit` and `cargo outdated`

## Troubleshooting

### Common Issues
- **Build lock:** Kill existing cargo processes if stuck
- **Toolchain issues:** Ensure rustup components are installed
- **Permission errors:** Make scripts executable with `chmod +x`

### Getting Help
- Run `cargo clippy --help` for clippy options
- Run `cargo fmt --help` for formatting options
- Check Rust documentation for specific error messages
