#!/bin/bash
# Single File Recovery Test
# Test restoring just one file to verify the process works

set -e

echo '🧪 Single File Recovery Test'
echo '============================'
echo ''

# Test file details from our analysis
TEST_FILE=".caws/EXEC_SUMMARY.md"
RECOVERY_PATH="recovered_work/13c18ee1/O9Rr.md"

echo "📄 Testing file: $TEST_FILE"
echo "📦 Source: $RECOVERY_PATH"
echo ""

# Create directory if it doesn't exist
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
    head -3 "$TEST_FILE"
    echo ""
    echo "🎉 Single file test PASSED!"
else
    echo "❌ File restoration failed!"
    exit 1
fi

echo ""
echo "🔍 Next steps:"
echo "  1. Review the restored file: cat $TEST_FILE"
echo "  2. If satisfied, run full recovery: bash recovery_output/recover.sh"
echo "  3. Or test another single file first"
