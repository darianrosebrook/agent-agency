# Test Type Alignment Guide

**Date**: October 12, 2025  
**Errors to Fix**: 236 TypeScript errors  
**Files Affected**: 7 test files

---

## Summary of Type Mismatches

The test files were written assuming certain type structures that don't match the actual type definitions. Here's what needs to be fixed:

---

## 1. VerificationPriority - Use Enum Values

**Current (WRONG)**:

```typescript
priority: "high"; // String literal
priority: "medium";
priority: "low";
priority: "critical";
```

**Correct**:

```typescript
import { VerificationPriority } from "@/types/verification";

priority: VerificationPriority.HIGH;
priority: VerificationPriority.MEDIUM;
priority: VerificationPriority.LOW;
priority: VerificationPriority.CRITICAL;
```

**Files to Fix**: All 7 test files  
**Occurrences**: ~100 instances

---

## 2. VerificationVerdict - Wrong Enum Values

**Current (WRONG)**:

```typescript
VerificationVerdict.VERIFIED; // Doesn't exist
VerificationVerdict.REFUTED; // Doesn't exist
VerificationVerdict.UNVERIFIABLE; // Doesn't exist (UNVERIFIED exists)
```

**Actual Enum Values**:

```typescript
export enum VerificationVerdict {
  VERIFIED_TRUE = "verified_true",
  VERIFIED_FALSE = "verified_false",
  PARTIALLY_TRUE = "partially_true",
  UNVERIFIED = "unverified",
  CONTRADICTORY = "contradictory",
  INSUFFICIENT_DATA = "insufficient_data",
  MIXED = "mixed",
  ERROR = "error",
}
```

**Mapping**:

- `VERIFIED` → `VERIFIED_TRUE`
- `REFUTED` → `VERIFIED_FALSE`
- `UNVERIFIABLE` → `UNVERIFIED`

**Files to Fix**: All 7 test files  
**Occurrences**: ~80 instances

---

## 3. VerificationResult - Wrong Structure

**Current (WRONG)**:

```typescript
const result: VerificationResult = {
  requestId: "test-1",
  verdict: VerificationVerdict.VERIFIED,
  confidence: 0.85,
  evidence: [], // Wrong property name
  methodResults: [], // Wrong property name
  processingTimeMs: 200,
  timestamp: new Date(), // Doesn't exist in type
  metadata: {}, // Doesn't exist in type
};
```

**Actual Type Definition**:

```typescript
export interface VerificationResult {
  requestId: string;
  verdict: VerificationVerdict;
  confidence: number;
  reasoning: string[]; // Required
  supportingEvidence: Evidence[]; // Not "evidence"
  contradictoryEvidence: Evidence[]; // Required
  verificationMethods: VerificationMethodResult[]; // Not "methodResults"
  processingTimeMs: number;
  error?: string;
}
```

**Correct Usage**:

```typescript
const result: VerificationResult = {
  requestId: "test-1",
  verdict: VerificationVerdict.VERIFIED_TRUE,
  confidence: 0.85,
  reasoning: ["Verified through cross-reference"],
  supportingEvidence: [
    {
      source: "https://example.com",
      content: "Supporting content",
      relevance: 0.9,
      credibility: 0.8,
      supporting: true,
      verificationDate: new Date(),
      metadata: {},
    },
  ],
  contradictoryEvidence: [],
  verificationMethods: [
    {
      method: VerificationType.FACT_CHECKING,
      verdict: VerificationVerdict.VERIFIED_TRUE,
      confidence: 0.85,
      reasoning: "Fact check passed",
      processingTimeMs: 150,
      evidenceCount: 1,
    },
  ],
  processingTimeMs: 200,
};
```

**Files to Fix**:

- `verification-database.test.ts` (multiple instances)

---

## 4. VerificationMethodResult - Wrong Structure

**Current (WRONG)**:

```typescript
{
  type: VerificationType.FACT_CHECKING,  // Wrong property name
  verdict: VerificationVerdict.VERIFIED,
  confidence: 0.85,
  evidence: [],  // Doesn't exist
  processingTimeMs: 150,
  metadata: {}  // Doesn't exist
}
```

**Actual Type Definition**:

```typescript
export interface VerificationMethodResult {
  method: VerificationType; // Not "type"
  verdict: VerificationVerdict;
  confidence: number;
  reasoning: string | string[]; // Required
  processingTimeMs: number;
  evidenceCount: number; // Not "evidence" array
}
```

**Correct Usage**:

```typescript
{
  method: VerificationType.FACT_CHECKING,
  verdict: VerificationVerdict.VERIFIED_TRUE,
  confidence: 0.85,
  reasoning: ["Verified through fact-checking API"],
  processingTimeMs: 150,
  evidenceCount: 3
}
```

**Files to Fix**: All validator test files

---

## 5. Evidence - Missing Required Field

**Current (WRONG)**:

```typescript
{
  source: "https://example.com",
  content: "Evidence content",
  relevance: 0.9,
  credibility: 0.8,
  supporting: true,
  metadata: {}
  // Missing verificationDate
}
```

**Correct**:

```typescript
{
  source: "https://example.com",
  content: "Evidence content",
  relevance: 0.9,
  credibility: 0.8,
  supporting: true,
  verificationDate: new Date(),  // Required
  metadata: {}
}
```

---

## 6. KnowledgeQuery Metadata - Wrong Structure

**Current (WRONG)**:

```typescript
metadata: {
  priority: 8,
  tags: ["test"]
}
```

**Actual Type**:

```typescript
metadata: {
  requesterId: string;  // Required
  priority: number;
  createdAt: Date;  // Required
  tags?: string[];
}
```

**Correct**:

```typescript
metadata: {
  requesterId: "test-requester",
  priority: 8,
  createdAt: new Date(),
  tags: ["test"]
}
```

**Files to Fix**: `knowledge-seeker-verification.test.ts`

---

## 7. Undefined Global Functions

**Issue**: Using `fail()` which is not defined in the test environment

**Current (WRONG)**:

```typescript
try {
  // ... test code
  fail("Should have thrown error");
} catch (error) {
  // ...
}
```

**Correct**:

```typescript
try {
  // ... test code
  throw new Error("Should have thrown error");
} catch (error) {
  // ...
}
```

**OR** use Jest's `expect().toThrow()`:

```typescript
expect(() => {
  // ... test code
}).toThrow("Expected error message");
```

**Files to Fix**:

- `verification-database.test.ts` (2 instances)
- `orchestrator-verification.test.ts` (1 instance)

---

## 8. Unused Variables

**Issue**: Variables assigned but never used

**Example**:

```typescript
const cached = await dbClient.getCachedResult(cacheKey);
// cached is never used after this
```

**Fix**: Either use the variable or prefix with underscore:

```typescript
const _cached = await dbClient.getCachedResult(cacheKey);
```

**OR** remove if not needed

---

## 9. QueryType.CONCEPTUAL Doesn't Exist

**Issue**: Using `QueryType.CONCEPTUAL` which doesn't exist

**Actual QueryType Enum**:

```typescript
export enum QueryType {
  FACTUAL = "factual",
  EXPLANATORY = "explanatory",
  COMPARATIVE = "comparative",
  TREND = "trend",
  TECHNICAL = "technical",
}
```

**Fix**: Replace `QueryType.CONCEPTUAL` with `QueryType.EXPLANATORY` or `QueryType.FACTUAL` depending on context

**Files to Fix**: `knowledge-seeker-verification.test.ts` (2 instances)

---

## 10. Missing timeoutMs in KnowledgeQuery

**Issue**: Tests create KnowledgeQuery without required `timeoutMs` field

**Fix**: Add `timeoutMs` to all query objects:

```typescript
const query: KnowledgeQuery = {
  id: "test-query",
  query: "Test query",
  queryType: QueryType.FACTUAL,
  maxResults: 5,
  relevanceThreshold: 0.5,
  timeoutMs: 30000, // Add this
  metadata: {
    requesterId: "test",
    priority: 7,
    createdAt: new Date(),
    tags: ["test"],
  },
};
```

---

## 11. ArbiterOrchestratorConfig Issues

**Issue**: Config object has incorrect properties for some nested configs

**Problems**:

- `orchestrator.name` doesn't exist
- `reasoningEffort` expects complex object, not string
- `eagerness` expects complex object, not number
- `toolBudget.maxToolCalls` wrong structure
- `contextGathering` has wrong properties
- `selfReflection.reflectionTriggers` doesn't exist

**Fix**: Use proper config structure or create minimal test config that matches actual types

---

## Systematic Fix Approach

### Step 1: Create Type-Safe Test Helpers

Create a file `iterations/v2/tests/helpers/verification-helpers.ts`:

```typescript
import {
  VerificationRequest,
  VerificationResult,
  VerificationVerdict,
  VerificationPriority,
  VerificationType,
  Evidence,
  VerificationMethodResult,
} from "@/types/verification";

export function createTestRequest(
  overrides: Partial<VerificationRequest> = {}
): VerificationRequest {
  return {
    id: "test-request",
    content: "Test content",
    source: "https://example.com",
    context: "Test context",
    priority: VerificationPriority.MEDIUM,
    verificationTypes: [VerificationType.FACT_CHECKING],
    metadata: {},
    ...overrides,
  };
}

export function createTestResult(
  overrides: Partial<VerificationResult> = {}
): VerificationResult {
  return {
    requestId: "test-request",
    verdict: VerificationVerdict.VERIFIED_TRUE,
    confidence: 0.8,
    reasoning: ["Test reasoning"],
    supportingEvidence: [],
    contradictoryEvidence: [],
    verificationMethods: [],
    processingTimeMs: 100,
    ...overrides,
  };
}

export function createTestEvidence(
  overrides: Partial<Evidence> = {}
): Evidence {
  return {
    source: "https://example.com",
    content: "Test evidence",
    relevance: 0.8,
    credibility: 0.7,
    supporting: true,
    verificationDate: new Date(),
    metadata: {},
    ...overrides,
  };
}

export function createTestMethodResult(
  overrides: Partial<VerificationMethodResult> = {}
): VerificationMethodResult {
  return {
    method: VerificationType.FACT_CHECKING,
    verdict: VerificationVerdict.VERIFIED_TRUE,
    confidence: 0.8,
    reasoning: "Test reasoning",
    processingTimeMs: 50,
    evidenceCount: 1,
    ...overrides,
  };
}
```

### Step 2: Use Helpers in Tests

```typescript
import {
  createTestRequest,
  createTestResult,
} from "../../helpers/verification-helpers";

it("should save and retrieve verification request", async () => {
  const request = createTestRequest({
    id: "specific-test-id",
    priority: VerificationPriority.HIGH,
  });

  await dbClient.saveRequest(request);
  // ... rest of test
});
```

### Step 3: Find and Replace

Use these patterns for bulk fixes:

**Priority fixes**:

```bash
# Find all string priorities
rg 'priority: "(high|medium|low|critical)"' iterations/v2/tests/

# Replace with enum (manual or script)
priority: "high" → priority: VerificationPriority.HIGH
priority: "medium" → priority: VerificationPriority.MEDIUM
priority: "low" → priority: VerificationPriority.LOW
priority: "critical" → priority: VerificationPriority.CRITICAL
```

**Verdict fixes**:

```bash
# Find all VERIFIED references
rg 'VerificationVerdict.VERIFIED[^_]' iterations/v2/tests/

# Replace
VERIFIED → VERIFIED_TRUE
REFUTED → VERIFIED_FALSE
UNVERIFIABLE → UNVERIFIED
```

---

## Test Execution Plan

After fixing types:

1. **Run TypeScript compilation**:

   ```bash
   cd iterations/v2
   npx tsc --noEmit
   ```

2. **Run linter**:

   ```bash
   npm run lint
   ```

3. **Run tests** (after fixing):

   ```bash
   npm test -- tests/integration/verification/
   npm test -- tests/unit/verification/
   npm test -- tests/integration/orchestrator/orchestrator-verification
   npm test -- tests/integration/knowledge/knowledge-seeker-verification
   ```

4. **Check coverage**:
   ```bash
   npm run test:coverage
   ```

---

## Priority Order for Fixes

1. **HIGH**: Create test helpers (Step 1 above) - Makes everything easier
2. **HIGH**: Fix VerificationPriority enum usage (~100 instances)
3. **HIGH**: Fix VerificationVerdict enum values (~80 instances)
4. **MEDIUM**: Fix VerificationResult structure (database tests)
5. **MEDIUM**: Fix VerificationMethodResult structure (validator tests)
6. **MEDIUM**: Fix KnowledgeQuery metadata (knowledge seeker tests)
7. **LOW**: Fix undefined `fail()` calls (3 instances)
8. **LOW**: Fix unused variables (2 instances)
9. **LOW**: Fix orchestrator config issues (1 file)

---

## Estimated Time

- Creating helpers: 15 minutes
- Bulk find/replace for enums: 30 minutes
- Structural fixes: 1-2 hours
- Testing and iteration: 1 hour
- **Total**: 2.5-3.5 hours

---

## Success Criteria

- [ ] Zero TypeScript compilation errors
- [ ] Zero linting errors
- [ ] All tests compile successfully
- [ ] Tests can be executed (may fail, but should run)
- [ ] Type safety maintained throughout

Once types are aligned, we can address actual test logic issues and get them passing.
