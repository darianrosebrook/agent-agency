#!/usr/bin/env bash
# TODO Assessment Module for V3 Readiness Framework
# Analyzes TODOs and identifies blockers in training/conversion paths
# @author: @darianrosebrook

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
V3_DIR="$ROOT_DIR/iterations/v3"
CONFIG_FILE="$SCRIPT_DIR/config.yaml"
OUTPUT_DIR="$ROOT_DIR/artifacts"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Load config values
get_config() {
    local key="$1"
    grep "^${key}:" "$CONFIG_FILE" | cut -d: -f2 | tr -d ' "'
}

get_config_list() {
    local section="$1"
    grep -A 20 "^${section}:" "$CONFIG_FILE" | grep "^-" | sed 's/^- //' | tr '\n' ' '
}

CONFIDENCE_THRESHOLD=$(grep -A 2 "todo_config:" "$CONFIG_FILE" | grep "confidence_threshold:" | cut -d: -f2 | awk '{print $1}')
TRAINING_CRATES=$(grep -A 5 "critical_paths:" "$CONFIG_FILE" | grep -A 3 "training:" | grep "^-" | sed 's/^- //' | tr '\n' ' ')
CONVERSION_CRATES=$(grep -A 10 "critical_paths:" "$CONFIG_FILE" | grep -A 3 "conversion:" | grep "^-" | sed 's/^- //' | tr '\n' ' ')
INFERENCE_CRATES=$(grep -A 15 "critical_paths:" "$CONFIG_FILE" | grep -A 3 "inference:" | grep "^-" | sed 's/^- //' | tr '\n' ' ')

cd "$ROOT_DIR"

echo -e "${BLUE}[todo-assessment] Starting TODO assessment...${NC}"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check if todo_analyzer.py exists
TODO_ANALYZER="$ROOT_DIR/scripts/v3/analysis/todo_analyzer.py"
if [ ! -f "$TODO_ANALYZER" ]; then
    echo -e "${RED}[todo-assessment] TODO analyzer not found: $TODO_ANALYZER${NC}"
    exit 1
fi

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}[todo-assessment] python3 not found${NC}"
    exit 1
fi

# Initialize results JSON
RESULTS_FILE="$OUTPUT_DIR/todo-results.json"
cat > "$RESULTS_FILE" <<EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "confidence_threshold": $CONFIDENCE_THRESHOLD,
  "summary": {
    "total_todos": 0,
    "high_confidence": 0,
    "medium_confidence": 0,
    "low_confidence": 0,
    "blocking_todos": 0,
    "non_blocking_todos": 0
  },
  "by_path": {
    "training": {
      "total": 0,
      "blocking": 0,
      "todos": []
    },
    "conversion": {
      "total": 0,
      "blocking": 0,
      "todos": []
    },
    "inference": {
      "total": 0,
      "blocking": 0,
      "todos": []
    }
  },
  "by_crate": {},
  "blocking_in_critical_paths": []
}
EOF

# Run TODO analyzer
echo -e "${BLUE}[todo-assessment] Running TODO analyzer...${NC}"
TODO_ANALYZER_OUTPUT="$OUTPUT_DIR/todo-analyzer-output.json"
TODO_ANALYZER_LOG="$OUTPUT_DIR/todo-analyzer.log"

if timeout 300 python3 "$TODO_ANALYZER" \
    --root "$ROOT_DIR" \
    --v3-only \
    --min-confidence "$CONFIDENCE_THRESHOLD" \
    --output-json "$TODO_ANALYZER_OUTPUT" \
    2>&1 | tee "$TODO_ANALYZER_LOG"; then
    echo -e "${GREEN}[todo-assessment] TODO analysis completed${NC}"
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo -e "${YELLOW}[todo-assessment] TODO analyzer timed out after 5 minutes, using partial results${NC}"
    else
        echo -e "${YELLOW}[todo-assessment] TODO analyzer completed with warnings (exit code: $EXIT_CODE)${NC}"
    fi
    # Continue even if analyzer had issues - we may have partial results
fi

# Parse TODO analyzer results
if [ ! -f "$TODO_ANALYZER_OUTPUT" ]; then
    echo -e "${YELLOW}[todo-assessment] TODO analyzer output not found, creating empty results${NC}"
    # Create empty output file so we can continue
    echo '{"summary": {"total_hidden_todos": 0, "high_confidence_todos": 0, "medium_confidence_todos": 0, "low_confidence_todos": 0}, "files": {}}' > "$TODO_ANALYZER_OUTPUT"
fi

echo -e "${BLUE}[todo-assessment] Parsing TODO analysis results...${NC}"

# Extract summary from analyzer output
SUMMARY=$(jq -r '.summary' "$TODO_ANALYZER_OUTPUT")
TOTAL_TODOS=$(echo "$SUMMARY" | jq -r '.total_hidden_todos // 0')
HIGH_CONF=$(echo "$SUMMARY" | jq -r '.high_confidence_todos // 0')
MEDIUM_CONF=$(echo "$SUMMARY" | jq -r '.medium_confidence_todos // 0')
LOW_CONF=$(echo "$SUMMARY" | jq -r '.low_confidence_todos // 0')

# Update summary in results
tmp_file=$(mktemp)
jq \
    --argjson total "$TOTAL_TODOS" \
    --argjson high "$HIGH_CONF" \
    --argjson medium "$MEDIUM_CONF" \
    --argjson low "$LOW_CONF" \
    '.summary = {
        total_todos: $total,
        high_confidence: $high,
        medium_confidence: $medium,
        low_confidence: $low,
        blocking_todos: 0,
        non_blocking_todos: 0
    }' "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

# Process files and categorize TODOs by path
BLOCKING_IN_CRITICAL=()

# Function to check if file belongs to a crate
file_belongs_to_crate() {
    local file="$1"
    local crate="$2"
    echo "$file" | grep -q "iterations/v3/$crate"
}

# Function to check if TODO is blocking
is_blocking_todo() {
    local todo_data="$1"
    local comment=$(echo "$todo_data" | jq -r '.comment // ""')
    
    # Check for explicit blocking markers
    if echo "$comment" | grep -qiE "BLOCKING.*Yes|blocking.*true|critical|must|required"; then
        return 0
    fi
    
    # Check confidence and priority
    local confidence=$(echo "$todo_data" | jq -r '.confidence_score // 0')
    if (( $(echo "$confidence >= 0.9" | bc -l) )); then
        # High confidence TODOs in critical paths are considered blocking
        return 0
    fi
    
    return 1
}

# Process each file with TODOs
jq -r '.files | to_entries[] | "\(.key)|\(.value | tojson)"' "$TODO_ANALYZER_OUTPUT" | while IFS='|' read -r file_path file_data; do
    # Extract crate name
    crate_name=""
    if [[ "$file_path" =~ iterations/v3/([^/]+) ]]; then
        crate_name="${BASH_REMATCH[1]}"
    fi
    
    # Initialize crate entry if needed
    if [ -n "$crate_name" ]; then
        crate_exists=$(jq -r ".by_crate[\"$crate_name\"] // empty" "$RESULTS_FILE")
        if [ -z "$crate_exists" ]; then
            tmp_file=$(mktemp)
            jq --arg crate "$crate_name" '.by_crate[$crate] = {total: 0, blocking: 0, todos: []}' "$RESULTS_FILE" > "$tmp_file"
            mv "$tmp_file" "$RESULTS_FILE"
        fi
    fi
    
    # Process TODOs in this file
    echo "$file_data" | jq -r '.hidden_todos | to_entries[] | "\(.key)|\(.value | tojson)"' | while IFS='|' read -r line_num todo_data; do
        blocking=0
        if is_blocking_todo "$todo_data"; then
            blocking=1
        fi
        
        # Categorize by path
        path_category=""
        if [ -n "$crate_name" ]; then
            for training_crate in $TRAINING_CRATES; do
                if [ "$crate_name" = "$training_crate" ]; then
                    path_category="training"
                    break
                fi
            done
            
            if [ -z "$path_category" ]; then
                for conversion_crate in $CONVERSION_CRATES; do
                    if [ "$crate_name" = "$conversion_crate" ]; then
                        path_category="conversion"
                        break
                    fi
                done
            fi
            
            if [ -z "$path_category" ]; then
                for inference_crate in $INFERENCE_CRATES; do
                    if [ "$crate_name" = "$inference_crate" ]; then
                        path_category="inference"
                        break
                    fi
                done
            fi
        fi
        
        # Create TODO entry
        todo_entry=$(jq -n \
            --arg file "$file_path" \
            --arg line "$line_num" \
            --argjson blocking "$blocking" \
            --arg category "$path_category" \
            '{
                file: $file,
                line: $line,
                blocking: ($blocking == 1),
                path_category: $category
            } + ($todo_data | fromjson)')
        
        # Update path-specific counts
        if [ -n "$path_category" ]; then
            tmp_file=$(mktemp)
            jq \
                --arg category "$path_category" \
                --argjson todo "$todo_entry" \
                --argjson blocking "$blocking" \
                '.by_path[$category].total += 1 |
                 .by_path[$category].blocking += ($blocking == 1) |
                 .by_path[$category].todos += [$todo]' \
                "$RESULTS_FILE" > "$tmp_file"
            mv "$tmp_file" "$RESULTS_FILE"
            
            # Track blocking TODOs in critical paths
            if [ "$blocking" -eq 1 ]; then
                BLOCKING_IN_CRITICAL+=("$file_path:$line_num")
            fi
        fi
        
        # Update crate counts
        if [ -n "$crate_name" ]; then
            tmp_file=$(mktemp)
            jq \
                --arg crate "$crate_name" \
                --argjson todo "$todo_entry" \
                --argjson blocking "$blocking" \
                '.by_crate[$crate].total += 1 |
                 .by_crate[$crate].blocking += ($blocking == 1) |
                 .by_crate[$crate].todos += [$todo]' \
                "$RESULTS_FILE" > "$tmp_file"
            mv "$tmp_file" "$RESULTS_FILE"
        fi
    done
done

# Update blocking counts in summary
TOTAL_BLOCKING=$(jq '[.by_path[].blocking] | add' "$RESULTS_FILE")
TOTAL_NON_BLOCKING=$((TOTAL_TODOS - TOTAL_BLOCKING))

tmp_file=$(mktemp)
jq \
    --argjson blocking "$TOTAL_BLOCKING" \
    --argjson non_blocking "$TOTAL_NON_BLOCKING" \
    '.summary.blocking_todos = $blocking | .summary.non_blocking_todos = $non_blocking' \
    "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

# Update blocking_in_critical_paths array
tmp_file=$(mktemp)
jq \
    --argjson blocking_list "$(printf '%s\n' "${BLOCKING_IN_CRITICAL[@]}" | jq -R . | jq -s .)" \
    '.blocking_in_critical_paths = $blocking_list' \
    "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

# Print summary
echo -e "${BLUE}[todo-assessment] TODO Assessment Summary:${NC}"
echo -e "  Total TODOs: $TOTAL_TODOS"
echo -e "  High Confidence: $HIGH_CONF"
echo -e "  Medium Confidence: $MEDIUM_CONF"
echo -e "  Low Confidence: $LOW_CONF"
echo -e "  Blocking TODOs: $TOTAL_BLOCKING"
echo -e "  Non-Blocking TODOs: $TOTAL_NON_BLOCKING"

# Print path-specific breakdown
for path in training conversion inference; do
    path_total=$(jq -r ".by_path.$path.total" "$RESULTS_FILE")
    path_blocking=$(jq -r ".by_path.$path.blocking" "$RESULTS_FILE")
    if [ "$path_total" -gt 0 ]; then
        echo -e "  $path path: $path_total TODOs ($path_blocking blocking)"
    fi
done

# Print blocking TODOs in critical paths
BLOCKING_COUNT=$(jq -r '.blocking_in_critical_paths | length' "$RESULTS_FILE")
if [ "$BLOCKING_COUNT" -gt 0 ]; then
    echo -e "${YELLOW}  Blocking TODOs in critical paths: $BLOCKING_COUNT${NC}"
    jq -r '.blocking_in_critical_paths[]' "$RESULTS_FILE" | head -10 | while read -r blocking_todo; do
        echo -e "    - $blocking_todo"
    done
    if [ "$BLOCKING_COUNT" -gt 10 ]; then
        echo -e "    ... and $((BLOCKING_COUNT - 10)) more"
    fi
fi

echo -e "${GREEN}[todo-assessment] Results saved to $RESULTS_FILE${NC}"

