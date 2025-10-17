# Mutation Testing Guide

Mutation testing helps surface logic that is insufficiently exercised by the test
suite. For V3 we standardise on [`cargo-mutants`](https://mutants.rs) for Rust
crates and expect Tier 1/2 components to track their mutation score alongside
coverage.

## Prerequisites

```bash
cargo install cargo-mutants
```

The tool installs as a Cargo subcommand and downloads any missing build
dependencies during the first run.

## Running Mutants

### Full MCP crate sweep

```bash
make mutants-mcp
```

- Runs `cargo mutants` against `agent-agency-mcp`
- Uses a 900 second timeout and disables shuffling for reproducibility
- Fails fast if the baseline (unmutated `cargo test`) fails

### List or scope mutants

```bash
# Inspect potential mutants without running them
make mutants-mcp-list

# Limit to specific files or glob patterns
make mutants-mcp-files FILES='mcp-integration/src/caws_integration.rs'
```

Scoping is useful while strengthening tests for a module or when iterating on
surviving mutants.

## Interpreting Results

- **Killed** mutants indicate tests caught the injected change.
- **Survived** mutants mean no failing test – add or strengthen tests.
- **Timeout / unviable** mutants often signal long-running tests; consider
  tightening baseline run time or adding exclusions via `.cargo/mutants.toml`.

Capture the final mutation score in CI and enforce Tier thresholds:

| Tier | Minimum mutation score |
| ---- | ---------------------- |
| T1   | ≥ 70%                  |
| T2   | ≥ 50%                  |
| T3   | ≥ 30%                  |

## CI Integration Tips

- Add a nightly or gated workflow step invoking `make mutants-mcp`.
- Store reports (e.g. `target/mutants/latest/report.json`) as artifacts for review.
- For long-running suites, shard with `--shard 1/4`, `--shard 2/4`, etc.

## Exclusions

Use `.cargo/mutants.toml` to exclude generated code, unsafe FFI shims, or areas
that are intentionally difficult to mutate without false positives. Keep
exclusions narrow and document the rationale.

```toml
[mutants]
exclude = [
  "generated_schema.rs",
  "*/ffi/*",
]
```

## Next Steps

1. Run `make mutants-mcp` and note survivors.
2. Strengthen or add tests targeting those code paths.
3. Commit updated tests and, if necessary, adjust `.cargo/mutants.toml`.
4. Add a CI job to ensure mutation thresholds stay within tier requirements.
