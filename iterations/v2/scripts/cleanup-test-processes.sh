#!/bin/bash

# Cleanup Test Processes Script
# @author @darianrosebrook
#
# This script helps clean up hanging Jest and test processes

echo "🔍 Checking for test processes..."

# Count current test processes
JEST_COUNT=$(ps aux | grep -E "jest.*--coverage|jest.*--detectOpenHandles" | grep -v grep | wc -l | tr -d ' ')
NPM_TEST_COUNT=$(ps aux | grep -E "npm.*test" | grep -v grep | wc -l | tr -d ' ')
STRYKER_COUNT=$(ps aux | grep stryker | grep -v grep | wc -l | tr -d ' ')

echo "📊 Current process counts:"
echo "  Jest processes: $JEST_COUNT"
echo "  npm test processes: $NPM_TEST_COUNT"
echo "  Stryker processes: $STRYKER_COUNT"

if [ "$JEST_COUNT" -gt 10 ] || [ "$NPM_TEST_COUNT" -gt 10 ] || [ "$STRYKER_COUNT" -gt 5 ]; then
    echo "⚠️  High number of test processes detected. Cleaning up..."
    
    # Kill Jest processes
    if [ "$JEST_COUNT" -gt 0 ]; then
        echo "🧹 Killing Jest processes..."
        pkill -f "jest --coverage" 2>/dev/null || true
        pkill -f "jest --detectOpenHandles" 2>/dev/null || true
    fi
    
    # Kill npm test processes
    if [ "$NPM_TEST_COUNT" -gt 0 ]; then
        echo "🧹 Killing npm test processes..."
        pkill -f "npm run test" 2>/dev/null || true
    fi
    
    # Kill Stryker processes
    if [ "$STRYKER_COUNT" -gt 0 ]; then
        echo "🧹 Killing Stryker processes..."
        pkill -f "stryker" 2>/dev/null || true
    fi
    
    # Wait a moment for processes to terminate
    sleep 2
    
    # Check final counts
    FINAL_JEST=$(ps aux | grep -E "jest.*--coverage|jest.*--detectOpenHandles" | grep -v grep | wc -l | tr -d ' ')
    FINAL_NPM=$(ps aux | grep -E "npm.*test" | grep -v grep | wc -l | tr -d ' ')
    FINAL_STRYKER=$(ps aux | grep stryker | grep -v grep | wc -l | tr -d ' ')
    
    echo "✅ Cleanup complete. Final counts:"
    echo "  Jest processes: $FINAL_JEST"
    echo "  npm test processes: $FINAL_NPM"
    echo "  Stryker processes: $FINAL_STRYKER"
    
    if [ "$FINAL_JEST" -gt 5 ] || [ "$FINAL_NPM" -gt 5 ] || [ "$FINAL_STRYKER" -gt 2 ]; then
        echo "⚠️  Some processes may still be running. You may need to kill them manually."
        echo "💡 Try: pkill -f 'jest' && pkill -f 'npm.*test' && pkill -f 'stryker'"
    fi
else
    echo "✅ Test process counts are normal."
fi

echo ""
echo "💡 To prevent this issue in the future:"
echo "   - Use 'npm run test:unit' instead of 'npm run test:coverage' for development"
echo "   - Run tests with limited workers: 'jest --maxWorkers=2'"
echo "   - Use this script regularly: './scripts/cleanup-test-processes.sh'"

