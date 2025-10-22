#!/bin/bash
# Find God Objects in V3 codebase
# Usage: ./find-god-objects.sh [path_to_v3]

V3_PATH=${1:-"/Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3"}
OUTPUT_DIR="/Users/darianrosebrook/Desktop/Projects/agent-agency/docs/audits/v3-codebase-audit-2025-10/metrics"

echo "=== God Object Detection Report ===" > "$OUTPUT_DIR/god-objects.txt"
echo "Generated: $(date)" >> "$OUTPUT_DIR/god-objects.txt"
echo "" >> "$OUTPUT_DIR/god-objects.txt"

echo "Files >1000 LOC:" >> "$OUTPUT_DIR/god-objects.txt"
find "$V3_PATH" -name "*.rs" -path "*/src/*" -exec wc -l {} \; | awk '$1 > 1000' | sort -rn >> "$OUTPUT_DIR/god-objects.txt"

echo "" >> "$OUTPUT_DIR/god-objects.txt"
echo "Files >2000 LOC (Critical God Objects):" >> "$OUTPUT_DIR/god-objects.txt"
find "$V3_PATH" -name "*.rs" -path "*/src/*" -exec wc -l {} \; | awk '$1 > 2000' | sort -rn >> "$OUTPUT_DIR/god-objects.txt"

echo "" >> "$OUTPUT_DIR/god-objects.txt"
echo "Files >3000 LOC (Severe God Objects):" >> "$OUTPUT_DIR/god-objects.txt"
find "$V3_PATH" -name "*.rs" -path "*/src/*" -exec wc -l {} \; | awk '$1 > 3000' | sort -rn >> "$OUTPUT_DIR/god-objects.txt"

echo "God object analysis complete. Results saved to $OUTPUT_DIR/god-objects.txt"

