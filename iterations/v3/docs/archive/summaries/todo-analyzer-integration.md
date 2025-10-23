# TODO Analyzer CI/CD Integration

## Overview

The CAWS system now includes automated hidden TODO detection as part of the quality gates. This prevents the accumulation of hidden technical debt by catching TODO comments that should be properly structured or addressed.

## How It Works

### Pre-commit Hook (Warning Mode)

- **Trigger**: Runs automatically on `git commit`
- **Behavior**: Warns about hidden TODOs but allows the commit to proceed
- **Purpose**: Early feedback to developers

### CI Checks (Blocking Mode)

- **Pull Request**: Warns about hidden TODOs in changed files
- **Push**: Blocks the push if hidden TODOs are found
- **Purpose**: Quality gate enforcement

## Installation

### Local Pre-commit Hook

```bash
# Install the CAWS git hooks
cd iterations/v3
make install-hooks
# or
npm run install-hooks
```

This sets up:
- Pre-commit hook that warns about hidden TODOs
- Automatic checking of staged files

### CI Integration

The TODO analyzer is automatically integrated into the GitHub Actions workflow:

- `pre-commit-check`: Warns on PRs (doesn't block)
- `push-block-check`: Blocks pushes with hidden TODOs

## Usage

### Manual Checking

```bash
# Check entire codebase
cd iterations/v3
python3 scripts/todo_analyzer.py

# Check specific files
python3 scripts/todo_analyzer.py --files src/main.rs src/lib.rs

# CI mode (blocks on errors)
python3 scripts/todo_analyzer.py --ci-mode --files src/main.rs

# Warning mode (never blocks)
python3 scripts/todo_analyzer.py --warn-only --files src/main.rs
```

### Package Scripts

```bash
# Install hooks
npm run install-hooks

# Run TODO analyzer
npm run todo-check -- --files src/*.rs
```

## Configuration

### Confidence Thresholds

- **Default**: 0.7 (70% confidence)
- **High confidence**: ≥0.9
- **Medium confidence**: ≥0.6
- **Low confidence**: <0.6

### Bypass Options

```bash
# Skip TODO check in commit message
git commit -m "[skip-todo-check] Urgent fix"

# Bypass hooks entirely
git commit --no-verify -m "Emergency commit"
```

## What Gets Detected

The analyzer looks for hidden TODO patterns like:

```rust
// TODO: add error handling here
// FIXME: this needs to be fixed
// HACK: temporary workaround
// NOTE: this should be improved
```

But filters out legitimate technical terms and documentation.

## Integration Benefits

### Quality Assurance
- Prevents accumulation of hidden technical debt
- Enforces TODO comment standards
- Catches incomplete implementations

### Developer Experience
- Early feedback during development
- Clear guidance on TODO standards
- Non-blocking warnings for flexibility

### CI/CD Pipeline
- Automated quality gates
- Consistent standards across team
- Integration with existing CAWS tooling

## Troubleshooting

### Hook Not Running

```bash
# Check if hooks are installed
git config core.hooksPath

# Reinstall hooks
cd iterations/v3 && make install-hooks
```

### False Positives

If legitimate comments are flagged as hidden TODOs:

1. Check the confidence score
2. Review the matched patterns
3. Consider adjusting the comment text
4. Use `--disable-code-stub-scan` if needed

### CI Failures

- Review the CI logs for specific files with issues
- Address the TODOs or use proper formatting
- Bypass with `[skip-todo-check]` only for emergencies

## Advanced Configuration

### Custom Patterns

The analyzer can be extended with custom TODO patterns by modifying `scripts/todo_analyzer.py`.

### Language Support

Currently supports:
- Rust
- Python
- JavaScript/TypeScript
- Go
- Java
- C/C++

### Integration with Other Tools

The TODO analyzer integrates with:
- CAWS quality gates
- Coverage reporting
- Mutation testing
- Schema validation
