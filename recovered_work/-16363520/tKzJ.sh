#!/bin/bash

# Agent Agency V3 - Edge Case Testing Results Analyzer
# Displays historical performance and trends

set -e

echo "🧪 Agent Agency V3 - Edge Case Testing Results Analysis"
echo "═══════════════════════════════════════════════════════════"
echo

# Check if history file exists
if [ ! -f "test-results-history.jsonl" ]; then
    echo "❌ No test results history found. Run edge case tests first."
    exit 1
fi

# Get latest run
LATEST_RUN=$(tail -n 1 test-results-history.jsonl)
TOTAL_RUNS=$(wc -l < test-results-history.jsonl)

echo "📊 Test Results Summary (Latest of $TOTAL_RUNS runs)"
echo "─────────────────────────────────────────────────────"

# Parse and display key metrics
SUCCESS_RATE=$(echo "$LATEST_RUN" | jq -r '.success_rate * 100' | xargs printf "%.1f")
ETHICAL_RATE=$(echo "$LATEST_RUN" | jq -r '.ethical_safety_rate * 100' | xargs printf "%.1f")
TECH_ACCURACY=$(echo "$LATEST_RUN" | jq -r '.technical_accuracy * 100' | xargs printf "%.1f")
AVG_QUALITY=$(echo "$LATEST_RUN" | jq -r '.average_quality_score * 100' | xargs printf "%.1f")
TIMESTAMP=$(echo "$LATEST_RUN" | jq -r '.timestamp')
VERSION=$(echo "$LATEST_RUN" | jq -r '.version')

echo "📅 Date: $TIMESTAMP"
echo "🏷️  Version: $VERSION"
echo "✅ Overall Success Rate: ${SUCCESS_RATE}%"
echo "🛡️  Ethical Safety Rate: ${ETHICAL_RATE}%"
echo "🔧 Technical Accuracy: ${TECH_ACCURACY}%"
echo "📊 Average Quality Score: ${AVG_QUALITY}%"
echo

echo "📋 Category Performance Breakdown"
echo "──────────────────────────────────"

# Display category results
echo "$LATEST_RUN" | jq -r '
.categories |
to_entries[] |
@text "\(.key | gsub("_"; " ") | ascii_upcase): \(.value.tests) tests, \(.value.approved)/\(.value.executed) approved/executed (\(.value.success_rate * 100 | floor)%)"
' | while read line; do
    echo "• $line"
done

echo
echo "🔍 Key Findings"
echo "───────────────"

echo "$LATEST_RUN" | jq -r '.key_findings[]' | while read finding; do
    echo "✅ $finding"
done

echo
echo "💡 Improvement Recommendations"
echo "──────────────────────────────"

echo "$LATEST_RUN" | jq -r '.improvement_areas[]' | while read area; do
    echo "🔄 $area"
done

echo
echo "$LATEST_RUN" | jq -r '.recommendations[]' | while read rec; do
    echo "💡 $rec"
done

echo
echo "📈 Historical Trends"
echo "────────────────────"

if [ "$TOTAL_RUNS" -gt 1 ]; then
    echo "Showing trends across all $TOTAL_RUNS test runs:"
    echo

    # Calculate trends
    FIRST_SUCCESS=$(head -n 1 test-results-history.jsonl | jq -r '.success_rate * 100')
    LATEST_SUCCESS=$SUCCESS_RATE
    IMPROVEMENT=$(echo "scale=1; $LATEST_SUCCESS - $FIRST_SUCCESS" | bc)

    FIRST_ETHICAL=$(head -n 1 test-results-history.jsonl | jq -r '.ethical_safety_rate * 100')
    LATEST_ETHICAL=$ETHICAL_RATE
    ETHICAL_CHANGE=$(echo "scale=1; $LATEST_ETHICAL - $FIRST_ETHICAL" | bc)

    echo "📈 Success Rate: ${FIRST_SUCCESS}% → ${LATEST_SUCCESS}% (${IMPROVEMENT}% change)"
    echo "🛡️  Ethical Safety: ${FIRST_ETHICAL}% → ${LATEST_ETHICAL}% (${ETHICAL_CHANGE}% change)"

    # Show quality score trend
    FIRST_QUALITY=$(head -n 1 test-results-history.jsonl | jq -r '.average_quality_score * 100')
    LATEST_QUALITY=$AVG_QUALITY
    QUALITY_CHANGE=$(echo "scale=1; $LATEST_QUALITY - $FIRST_QUALITY" | bc)

    echo "📊 Quality Score: ${FIRST_QUALITY}% → ${LATEST_QUALITY}% (${QUALITY_CHANGE}% change)"
else
    echo "📝 This is the first test run. More runs needed for trend analysis."
fi

echo
echo "🎯 Next Steps"
echo "─────────────"

if (( $(echo "$SUCCESS_RATE < 95" | bc -l) )); then
    echo "🔄 Continue improving technical accuracy and domain expertise detection"
fi

if (( $(echo "$ETHICAL_RATE < 100" | bc -l) )); then
    echo "🛡️  Address any ethical safety concerns"
fi

if (( $(echo "$TECH_ACCURACY < 80" | bc -l) )); then
    echo "🔧 Focus on technical feasibility assessment improvements"
fi

echo "📋 Run full edge case suite: ./run-edge-case-tests.sh"
echo "📊 View detailed results: cat test-results-history.jsonl | jq"
echo "📖 Read documentation: EDGE_CASE_TESTING_DOCUMENTATION.md"
