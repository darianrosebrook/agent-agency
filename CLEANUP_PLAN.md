# Root & Iterations Cleanup Plan

## Files to Remove (Temporary/Generated)

### Root Level
- **Log files**: `*.log` (cargo-check-output.log, cargo_check_output.log)
- **Analysis JSON**: cargo-check-*.json, hidden-todos-*.json, todo-analysis-*.json, benchmark_results.json
- **Report files**: duplication-full-report.txt, todo-analysis-summary.md, todo.md, missing.md
- **Temporary scripts**: analyze_errors.py, fix-serde-imports.py, fix_critical_todos.py, remove-jsonschema.py

### iterations/v3
- **Status/Report MD files**: *_STATUS.md, *_REPORT.md, *_PROGRESS.md, *_COMPLETE.md, BUILD_ERRORS_*.md, WORKER*.md
- **Log files**: *.log
- **Analysis JSON**: cargo-check-*.json, build_errors_*.json, untracked_errors.json
- **Temporary scripts**: fix_*.sh, fix_*.py
- **Temporary directories**: `true/` (build artifacts)

## Files to Keep
- README.md files
- CHANGELOG.md
- Configuration files (package.json, tsconfig.json, Cargo.toml, etc.)
- Core documentation in docs/
- Integration plans (V3_INTEGRATION_PLAN.md, CYCLIC_DEPENDENCY_FIX.md, INTEGRATION_COMPLETE.md)

## Organization Strategy
1. Move temporary analysis files to `docs-status/` or delete if truly temporary
2. Ensure .gitignore covers all temporary file patterns
3. Archive important status reports to docs-status/ if needed


