#!/bin/bash
# Pattern Detection Script for V3 Documentation Reality Audit
# Detects stubs, placeholders, mocks, and incomplete implementations

set -euo pipefail

V3_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTPUT_DIR="${V3_ROOT}/docs-status/audit-reports"
RESULTS_FILE="${OUTPUT_DIR}/pattern-detection-results.json"

mkdir -p "${OUTPUT_DIR}"

echo "{" > "${RESULTS_FILE}"
echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"," >> "${RESULTS_FILE}"
echo "  \"patterns\": {" >> "${RESULTS_FILE}"

# Stub implementations
echo "    \"stubs\": {" >> "${RESULTS_FILE}"
STUB_COUNT=$(rg -n "struct Stub|class Stub|impl.*Stub|_stub\(|stub_|Stub.*::new" "${V3_ROOT}/src" 2>/dev/null | wc -l || echo "0")
echo "      \"count\": ${STUB_COUNT}," >> "${RESULTS_FILE}"
echo "      \"matches\": [" >> "${RESULTS_FILE}"
rg -n "struct Stub|class Stub|impl.*Stub|_stub\(|stub_|Stub.*::new" "${V3_ROOT}/src" 2>/dev/null | head -50 | while IFS= read -r line; do
  file=$(echo "$line" | cut -d: -f1)
  line_num=$(echo "$line" | cut -d: -f2)
  content=$(echo "$line" | cut -d: -f3- | sed 's/"/\\"/g')
  echo "        {\"file\": \"${file}\", \"line\": ${line_num}, \"content\": \"${content}\"}," >> "${RESULTS_FILE}"
done || true
echo "      ]" >> "${RESULTS_FILE}"
echo "    }," >> "${RESULTS_FILE}"

# Placeholder implementations
echo "    \"placeholders\": {" >> "${RESULTS_FILE}"
PLACEHOLDER_COUNT=$(rg -n "PLACEHOLDER|placeholder.*not.*implemented|return.*placeholder" "${V3_ROOT}/src" 2>/dev/null | wc -l || echo "0")
echo "      \"count\": ${PLACEHOLDER_COUNT}," >> "${RESULTS_FILE}"
echo "      \"matches\": [" >> "${RESULTS_FILE}"
rg -n "PLACEHOLDER|placeholder.*not.*implemented|return.*placeholder" "${V3_ROOT}/src" 2>/dev/null | head -50 | while IFS= read -r line; do
  file=$(echo "$line" | cut -d: -f1)
  line_num=$(echo "$line" | cut -d: -f2)
  content=$(echo "$line" | cut -d: -f3- | sed 's/"/\\"/g')
  echo "        {\"file\": \"${file}\", \"line\": ${line_num}, \"content\": \"${content}\"}," >> "${RESULTS_FILE}"
done || true
echo "      ]" >> "${RESULTS_FILE}"
echo "    }," >> "${RESULTS_FILE}"

# Mock data
echo "    \"mocks\": {" >> "${RESULTS_FILE}"
MOCK_COUNT=$(rg -n "MOCK_DATA|mock.*data|fake.*data|hardcoded|test.*value|dummy.*value" "${V3_ROOT}/src" 2>/dev/null | wc -l || echo "0")
echo "      \"count\": ${MOCK_COUNT}," >> "${RESULTS_FILE}"
echo "      \"matches\": [" >> "${RESULTS_FILE}"
rg -n "MOCK_DATA|mock.*data|fake.*data|hardcoded|test.*value|dummy.*value" "${V3_ROOT}/src" 2>/dev/null | head -50 | while IFS= read -r line; do
  file=$(echo "$line" | cut -d: -f1)
  line_num=$(echo "$line" | cut -d: -f2)
  content=$(echo "$line" | cut -d: -f3- | sed 's/"/\\"/g')
  echo "        {\"file\": \"${file}\", \"line\": ${line_num}, \"content\": \"${content}\"}," >> "${RESULTS_FILE}"
done || true
echo "      ]" >> "${RESULTS_FILE}"
echo "    }," >> "${RESULTS_FILE}"

# Incomplete implementations
echo "    \"incomplete\": {" >> "${RESULTS_FILE}"
INCOMPLETE_COUNT=$(rg -n "not.*implemented|NotImplemented|unimplemented" "${V3_ROOT}/src" --glob "!tests/**" --glob "!examples/**" 2>/dev/null | wc -l || echo "0")
echo "      \"count\": ${INCOMPLETE_COUNT}," >> "${RESULTS_FILE}"
echo "      \"matches\": [" >> "${RESULTS_FILE}"
rg -n "not.*implemented|NotImplemented|unimplemented" "${V3_ROOT}/src" --glob "!tests/**" --glob "!examples/**" 2>/dev/null | head -50 | while IFS= read -r line; do
  file=$(echo "$line" | cut -d: -f1)
  line_num=$(echo "$line" | cut -d: -f2)
  content=$(echo "$line" | cut -d: -f3- | sed 's/"/\\"/g")
  echo "        {\"file\": \"${file}\", \"line\": ${line_num}, \"content\": \"${content}\"}," >> "${RESULTS_FILE}"
done || true
echo "      ]" >> "${RESULTS_FILE}"
echo "    }" >> "${RESULTS_FILE}"

echo "  }" >> "${RESULTS_FILE}"
echo "}" >> "${RESULTS_FILE}"

echo "Pattern detection complete. Results saved to ${RESULTS_FILE}"

