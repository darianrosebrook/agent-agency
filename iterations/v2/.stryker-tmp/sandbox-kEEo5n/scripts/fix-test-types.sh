#!/bin/bash

# Script to fix common type errors in verification tests
# Run from iterations/v2 directory

set -e

echo "🔧 Starting test type fixes..."

TEST_DIR="tests"

# Add imports for VerificationPriority where needed
echo "📦 Adding VerificationPriority import..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/import {$/import {\n  VerificationPriority,/g' {} \;

# Fix priority values - MEDIUM
echo "🔄 Fixing priority: MEDIUM..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/priority: "medium"/priority: VerificationPriority.MEDIUM/g' {} \;

# Fix priority values - HIGH
echo "🔄 Fixing priority: HIGH..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/priority: "high"/priority: VerificationPriority.HIGH/g' {} \;

# Fix priority values - LOW
echo "🔄 Fixing priority: LOW..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/priority: "low"/priority: VerificationPriority.LOW/g' {} \;

# Fix priority values - CRITICAL
echo "🔄 Fixing priority: CRITICAL..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/priority: "critical"/priority: VerificationPriority.CRITICAL/g' {} \;

# Fix verdict values - VERIFIED -> VERIFIED_TRUE
echo "🔄 Fixing verdict: VERIFIED -> VERIFIED_TRUE..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/VerificationVerdict\.VERIFIED[^_]/VerificationVerdict.VERIFIED_TRUE/g' {} \;

# Fix verdict values - REFUTED -> VERIFIED_FALSE
echo "🔄 Fixing verdict: REFUTED -> VERIFIED_FALSE..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/VerificationVerdict\.REFUTED/VerificationVerdict.VERIFIED_FALSE/g' {} \;

# Fix verdict values - UNVERIFIABLE -> UNVERIFIED
echo "🔄 Fixing verdict: UNVERIFIABLE -> UNVERIFIED..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/VerificationVerdict\.UNVERIFIABLE/VerificationVerdict.UNVERIFIED/g' {} \;

# Fix QueryType.CONCEPTUAL -> QueryType.EXPLANATORY
echo "🔄 Fixing QueryType.CONCEPTUAL -> QueryType.EXPLANATORY..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/QueryType\.CONCEPTUAL/QueryType.EXPLANATORY/g' {} \;

# Fix fail() -> throw new Error()
echo "🔄 Fixing fail() calls..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/fail(\(.*\))/throw new Error(\1)/g' {} \;

# Fix unused variables - prefix with underscore
echo "🔄 Fixing unused variables..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/const cached =/const _cached =/g' {} \;

echo "✅ Bulk replacements complete!"
echo ""
echo "⚠️  Manual fixes still needed:"
echo "  - VerificationResult structure (evidence -> supportingEvidence, etc.)"
echo "  - VerificationMethodResult structure (type -> method, etc.)"
echo "  - KnowledgeQuery metadata (add requesterId, createdAt)"
echo "  - Property access (result.evidence -> result.supportingEvidence)"
echo "  - ArbiterOrchestratorConfig structure issues"
echo ""
echo "Run 'npm run lint' to see remaining errors"

