# Auto-Generated Status Reports

This directory contains **evidence-based status reports** generated from actual code analysis.

## Purpose

These reports are automatically generated to provide accurate, up-to-date status information based on actual code state rather than manual claims that become outdated.

## Reports

- **`STATUS_REPORT_*.md`** - Comprehensive status reports with:
  - API endpoint coverage (actual implementations)
  - TODO/PLACEHOLDER density analysis
  - Test infrastructure status
  - CoreML integration status
  - Compilation status

## Generation

Generate a new report:
```bash
python3 scripts/v3/analysis/generate_status_report.py \
  --v3-root iterations/v3 \
  --output-dir iterations/v3/docs/status-generated
```

Update `CURRENT_STATUS_AND_NEXT_STEPS.md` with evidence section:
```bash
python3 scripts/v3/analysis/generate_status_report.py \
  --v3-root iterations/v3 \
  --output-dir iterations/v3/docs/status-generated \
  --update-current
```

## Manual Status Documents

Manual status documents in `iterations/v3/docs/` may be outdated:
- `CURRENT_STATUS_AND_NEXT_STEPS.md` - May contain stale claims
- `API_GAP_ANALYSIS.md` - May claim endpoints are missing when they exist
- `CRITICAL_BLOCKING_TODOS.md` - May claim issues are resolved when they're not

**Always check the latest auto-generated report for accurate status.**

## Evidence-Based Claims

All claims in these reports are:
- ✅ Verified from actual code analysis
- ✅ Generated automatically (no manual drift)
- ✅ Timestamped for freshness tracking
- ✅ Include evidence (file paths, line numbers, counts)

## Integration with V4

V4 should:
- Generate status reports automatically in CI/CD
- Link status claims to evidence (test results, code analysis)
- Expire manual status documents after 30 days
- Use code metrics for status (coverage, test results, compilation success)






