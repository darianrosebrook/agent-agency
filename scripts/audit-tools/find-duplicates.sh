#!/bin/bash
# Find Duplication in V3 codebase
# Usage: ./find-duplicates.sh [path_to_v3]

V3_PATH=${1:-"/Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3"}
OUTPUT_DIR="/Users/darianrosebrook/Desktop/Projects/agent-agency/docs/audits/v3-codebase-audit-2025-10/metrics"

echo "=== Duplication Detection Report ===" > "$OUTPUT_DIR/duplication-report.txt"
echo "Generated: $(date)" >> "$OUTPUT_DIR/duplication-report.txt"
echo "" >> "$OUTPUT_DIR/duplication-report.txt"

echo "Duplicate filenames across crates:" >> "$OUTPUT_DIR/duplication-report.txt"
find "$V3_PATH" -name "*.rs" -path "*/src/*" | xargs basename | sort | uniq -d >> "$OUTPUT_DIR/duplication-report.txt"

echo "" >> "$OUTPUT_DIR/duplication-report.txt"
echo "Duplicate struct names:" >> "$OUTPUT_DIR/duplication-report.txt"
cd "$V3_PATH" && rg "^pub struct (\w+)" -r '$1' --no-filename | sort | uniq -d >> "$OUTPUT_DIR/duplication-report.txt"

echo "" >> "$OUTPUT_DIR/duplication-report.txt"
echo "Duplicate trait names:" >> "$OUTPUT_DIR/duplication-report.txt"
cd "$V3_PATH" && rg "^pub trait (\w+)" -r '$1' --no-filename | sort | uniq -d >> "$OUTPUT_DIR/duplication-report.txt"

echo "Duplication analysis complete. Results saved to $OUTPUT_DIR/duplication-report.txt"

