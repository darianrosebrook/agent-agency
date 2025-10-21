#!/bin/bash
# Test restoring an orchestration file
# This tests a more complex component

set -e

echo '🧪 Orchestration File Recovery Test'
echo '==================================='
echo ''

# Test with an orchestration file
TEST_FILE="iterations/v3/orchestration/src/planning/agent.rs"
RECOVERY_PATH="recovered_work/51e85205/WINB.rs"

echo "📄 Testing file: $TEST_FILE"
echo "📦 Source: $RECOVERY_PATH"
echo ""

# Create directory structure
echo "📁 Creating directory structure..."
mkdir -p "$(dirname "$TEST_FILE")"

# Check if target file already exists
if [ -f "$TEST_FILE" ]; then
    echo "⚠️  Target file already exists, creating backup..."
    cp "$TEST_FILE" "${TEST_FILE}.backup.$(date +%s)"
    echo "✅ Backup created"
fi

# Restore the file
echo "🔄 Restoring file..."
cp "$RECOVERY_PATH" "$TEST_FILE"

# Verify the restoration
if [ -f "$TEST_FILE" ]; then
    echo "✅ File restored successfully!"
    echo "📊 File size: $(wc -c < "$TEST_FILE") bytes"
    echo "📝 First few lines:"
    head -5 "$TEST_FILE"
    echo ""
    echo "🔍 Checking if it's valid Rust code..."
    if grep -q "use " "$TEST_FILE" && grep -q "fn " "$TEST_FILE"; then
        echo "✅ Appears to be valid Rust code"
    else
        echo "⚠️  May not be valid Rust code"
    fi
    echo ""
    echo "🎉 Orchestration file test PASSED!"
else
    echo "❌ File restoration failed!"
    exit 1
fi

echo ""
echo "🔍 Next steps:"
echo "  1. Review the restored file: cat $TEST_FILE"
echo "  2. Test compilation: cd iterations/v3 && cargo check --package orchestration"
echo "  3. If satisfied, run full recovery: bash recovery_output/recover.sh"
