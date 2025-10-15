#!/bin/bash

# Fix verification-database.test.ts specific issues
# Run from iterations/v2 directory

set -e

FILE="tests/integration/verification/verification-database.test.ts"

echo "🔧 Fixing verification-database.test.ts..."

# Fix missing commas after VERIFIED_TRUE/VERIFIED_FALSE
echo "🔄 Adding missing commas..."
sed -i '' 's/VerificationVerdict\.VERIFIED_TRUE$/VerificationVerdict.VERIFIED_TRUE,/g' "$FILE"
sed -i '' 's/VerificationVerdict\.VERIFIED_FALSE$/VerificationVerdict.VERIFIED_FALSE,/g' "$FILE"

# Fix VerificationResult property names
echo "🔄 Fixing VerificationResult properties..."
sed -i '' 's/evidence: \[/supportingEvidence: [/g' "$FILE"
sed -i '' 's/methodResults: \[/verificationMethods: [/g' "$FILE"

# Remove timestamp and metadata from VerificationResult (they don't exist)
echo "🔄 Removing invalid properties..."
sed -i '' '/^[[:space:]]*timestamp: new Date(),$/d' "$FILE"
sed -i '' '/^[[:space:]]*processingTimeMs: [0-9]*,$/d' "$FILE"

# Add required reasoning property to VerificationResult
echo "🔄 Adding reasoning property..."
sed -i '' 's/verdict: VerificationVerdict\.\(VERIFIED_TRUE\|VERIFIED_FALSE\|UNVERIFIED\),$/verdict: VerificationVerdict.\1,\
        reasoning: ["Test verification reasoning"],/g' "$FILE"

# Add contradictoryEvidence property
echo "🔄 Adding contradictoryEvidence property..."
sed -i '' 's/supportingEvidence: \[$/supportingEvidence: [],\
        contradictoryEvidence: [/g' "$FILE"

# Fix VerificationMethodResult properties (type -> method)
echo "🔄 Fixing VerificationMethodResult properties..."
sed -i '' 's/type: VerificationType/method: VerificationType/g' "$FILE"

# Remove evidence array from VerificationMethodResult (use evidenceCount instead)
sed -i '' 's/evidence: \[\],/evidenceCount: 0,/g' "$FILE"

# Remove processingTimeMs and metadata from VerificationMethodResult
sed -i '' '/^[[:space:]]*processingTimeMs: [0-9]*,$/d' "$FILE"
sed -i '' '/^[[:space:]]*metadata: {},$/d' "$FILE"

# Add reasoning to VerificationMethodResult
sed -i '' 's/method: VerificationType\.\([A-Z_]*\),$/method: VerificationType.\1,\
            reasoning: ["Method verification reasoning"],/g' "$FILE"

# Fix property access in assertions
echo "🔄 Fixing property access..."
sed -i '' 's/retrieved?\.evidence/retrieved?.supportingEvidence/g' "$FILE"
sed -i '' 's/retrieved?\.methodResults/retrieved?.verificationMethods/g' "$FILE"
sed -i '' 's/result\.methodResults/result.verificationMethods/g' "$FILE"

# Fix fail() calls
echo "🔄 Fixing fail() calls..."
sed -i '' 's/fail(/throw new Error(/g' "$FILE"

# Fix _cached variable name
echo "🔄 Fixing unused variables..."
sed -i '' 's/const cached =/const _cached =/g' "$FILE"

# Fix VerificationVerdict.VERIFIED
echo "🔄 Fixing VERIFIED enum..."
sed -i '' 's/VerificationVerdict\.VERIFIED[^_]/VerificationVerdict.VERIFIED_TRUE/g' "$FILE"

echo "✅ verification-database.test.ts fixed!"


