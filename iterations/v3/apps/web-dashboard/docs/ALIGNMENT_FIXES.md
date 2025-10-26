# Targeted Alignment Fixes

## Critical Alignment Issues & Fixes

### 1. **API Endpoint Standardization** ⚠️ HIGH PRIORITY

**Issue**: Planning document specifies exact endpoints, but implementation uses different patterns.

**Planning Document Endpoints:**
```typescript
GET /api/council/verdicts?status=pending&limit=50
GET /api/council/verdicts/{id}
POST /api/council/verdicts/{id}/override
GET /api/council/verdicts/{id}/evidence
```

**Current Implementation:**
- Uses `/api/council/verdicts?${params}` (correct)
- But VerdictList.tsx calls `councilApiClient.getVerdicts()` with different signature

**Fix Required:**
```typescript
// In VerdictList.tsx - Update API call to match planning
const fetchVerdicts = async (currentFilters = filters, page = pagination.currentPage) => {
  try {
    actions.setLoading('verdicts', true);
    actions.setError('verdicts', undefined);

    // Fix: Use correct API method signature
    const response = await councilApiClient.getVerdicts({
      status: currentFilters.status,
      judgeId: currentFilters.judgeId,
      dateRange: currentFilters.dateRange,
      search: currentFilters.search
    }, page, pagination.pageSize);

    actions.setVerdicts(response.verdicts, response.total, response.hasMore);
    // ... rest of implementation
  }
};
```

### 2. **Data Structure Alignment** ⚠️ HIGH PRIORITY

**Issue**: Verdict interface in implementation doesn't match planning document structure.

**Planning Document Verdict Structure:**
```typescript
interface Verdict {
  id: string;
  taskId: string;
  status: 'pending' | 'approved' | 'rejected' | 'intervened';
  judges: JudgeAssignment[];
  consensus: ConsensusResult;
  ethicalAssessment: EthicalAssessment;
  evidence: Evidence[];
  intervention?: { /* ... */ };
  createdAt: Date;
  completedAt?: Date;
  updatedAt: Date;
}
```

**Current Implementation Verdict Structure:**
```typescript
interface Verdict {
  id: string;
  taskId: string;           // ✅ Matches
  status: VerdictStatus;    // ✅ Matches
  title: string;            // ❌ Missing in planning
  summary: string;          // ❌ Missing in planning
  judgeCount: number;       // ❌ Missing in planning
  consensusScore: number;   // ❌ Missing in planning
  ethicalConcerns: number;  // ❌ Missing in planning
  createdAt: Date;          // ✅ Matches
  updatedAt: Date;          // ✅ Matches
  judges: Judge[];          // ❌ Different structure
  evidence: Evidence[];     // ✅ Matches
}
```

**Fix Required:**
```typescript
// Update VerdictList.tsx interface to match planning document
export interface Verdict {
  id: string;
  taskId: string;
  status: 'pending' | 'approved' | 'rejected' | 'intervened';
  judges: JudgeAssignment[];  // Use JudgeAssignment[] instead of Judge[]
  consensus: ConsensusResult;
  ethicalAssessment: EthicalAssessment;
  evidence: Evidence[];
  intervention?: {
    type: 'manual_override' | 'escalation' | 'pause';
    reason: string;
    operator: string;
    timestamp: Date;
  };
  createdAt: Date;
  completedAt?: Date;
  updatedAt: Date;
}

// Computed fields should be derived, not stored
export interface VerdictDisplay extends Verdict {
  // These should be computed from verdict data
  title: string;              // Computed from taskId + status
  summary: string;            // Computed from consensus rationale
  judgeCount: number;         // Computed from judges.length
  consensusScore: number;     // Computed from consensus.confidence
  ethicalConcerns: number;    // Computed from ethicalAssessment.concerns.length
}
```

### 3. **Component Naming Inconsistencies** ⚠️ MEDIUM PRIORITY

**Issue**: Planning document specifies specific component names, but implementation may use different names.

**Planning Document Components:**
- `VerdictCard` ✅ (Exists)
- `VerdictDetailModal` ✅ (Exists)
- `EvidenceViewer` ❌ (Missing - may be part of VerdictDetailModal)
- `InterventionForm` ✅ (Exists)

**Fix Required:**
```typescript
// Check if EvidenceViewer exists or needs to be extracted
// If it's embedded in VerdictDetailModal, extract it as separate component

// EvidenceViewer.tsx - Extract from VerdictDetailModal if needed
export function EvidenceViewer({ evidence }: { evidence: Evidence[] }) {
  // Implementation to display evidence chain
}

// Update VerdictDetailModal to use EvidenceViewer
<EvidenceViewer evidence={verdict.evidence} />
```

### 4. **State Management Structure** ⚠️ MEDIUM PRIORITY

**Issue**: Council store structure doesn't fully align with planning document expectations.

**Planning Document Expected State:**
```typescript
interface CouncilState {
  verdicts: Verdict[];
  judges: Judge[];
  metrics: CouncilMetrics;
  alerts: CouncilAlert[];
  ethicalAssessments: EthicalAssessment[];
  // UI state, loading, errors, pagination
}
```

**Current Implementation State:**
```typescript
interface CouncilState {
  verdicts: Verdict[];      // ✅ Matches
  judges: Judge[];          // ✅ Matches
  metrics: CouncilMetrics;  // ✅ Matches (nullable)
  alerts: CouncilAlert[];   // ✅ Matches
  // ❌ Missing: ethicalAssessments
  // UI state, loading, errors, pagination - ✅ Present
}
```

**Fix Required:**
```typescript
// Add missing ethicalAssessments to CouncilState
interface CouncilState {
  // ... existing fields
  ethicalAssessments: EthicalAssessment[];
  selectedEthicalAssessment: EthicalAssessment | null;
  // ... rest of state
}

// Add ethical assessment actions
interface CouncilActions {
  // ... existing actions
  setEthicalAssessments: (assessments: EthicalAssessment[]) => void;
  addEthicalAssessment: (assessment: EthicalAssessment) => void;
  updateEthicalAssessment: (id: string, updates: Partial<EthicalAssessment>) => void;
  setSelectedEthicalAssessment: (assessment: EthicalAssessment | null) => void;
}
```

### 5. **API Response Format Mismatches** ⚠️ MEDIUM PRIORITY

**Issue**: Planning document specifies exact API response formats, but implementation may differ.

**Planning Document Response:**
```typescript
GET /api/council/verdicts?status=pending&limit=50
// Response:
{
  verdicts: Verdict[],
  total: number,
  page: number,
  limit: number
}
```

**Current Implementation Response:**
```typescript
// CouncilApiClient.getVerdicts() returns:
{
  verdicts: Verdict[],
  total: number,
  page: number,
  limit: number  // ✅ This matches
}
```

**Fix Required:**
```typescript
// Ensure API client returns exactly what planning document specifies
async getVerdicts(
  filters?: VerdictFilter,
  page: number = 1,
  limit: number = 20
): Promise<{
  verdicts: Verdict[];  // ✅
  total: number;        // ✅
  page: number;         // ✅
  limit: number;        // ✅
  hasMore?: boolean;    // ❌ Extra field - remove or make optional in planning
}> {
  // Implementation
}
```

### 6. **Mock Data Removal** ⚠️ HIGH PRIORITY

**Issue**: Implementation heavily uses mock data instead of real API integration.

**Examples of Mock Data Usage:**
```typescript
// In Apple Silicon dashboard - using mock data
setMetrics({
  aneUtilization: 78.5,
  gpuUtilization: 45.2,
  // ... hardcoded values
});

// In Council components - TODO comments about mock data
// TODO: Replace with actual API call
```

**Fix Required:**
```typescript
// Remove mock data and implement proper error handling
const fetchMetrics = async () => {
  try {
    // Remove mock data - use real API
    const response = await appleSiliconApiClient.getCurrentMetrics();

    if (response.success) {
      setMetrics(response.data);
    } else {
      // Handle API errors properly
      console.error('Failed to fetch metrics:', response.error);
      setError(response.error?.message || 'Failed to load metrics');
    }
  } catch (err) {
    // Handle network/other errors
    console.error('Metrics fetch error:', err);
    setError(err instanceof Error ? err.message : 'Network error');
  }
};
```

### 7. **Missing Backend API Routes** ⚠️ CRITICAL PRIORITY

**Issue**: No backend API routes exist for Council and Apple Silicon features.

**Missing API Routes:**
```
❌ /api/council/verdicts/*
❌ /api/council/judges/*
❌ /api/council/metrics
❌ /api/council/alerts/*
❌ /api/council/ethical-assessments/*
❌ /api/apple-silicon/metrics/*
❌ /api/apple-silicon/thermal/*
❌ /api/apple-silicon/models/*
❌ /api/apple-silicon/routing/*
```

**Fix Required:**
```typescript
// Create backend API routes in apps/web-dashboard/src/app/api/

// Council routes
export async function GET(request: Request) {
  // Implementation for /api/council/verdicts
}

export async function POST(request: Request) {
  // Implementation for creating verdicts
}

// Apple Silicon routes
export async function GET(request: Request) {
  // Implementation for /api/apple-silicon/metrics/current
}
```

### 8. **WebSocket Integration Gaps** ⚠️ MEDIUM PRIORITY

**Issue**: WebSocket hooks exist but may not fully implement planning document requirements.

**Planning Document Requirements:**
- Real-time verdict updates
- Live hardware metrics streaming
- Judge verdict submissions
- Alert notifications

**Current Implementation:**
- `useCouncilWebSocket` ✅ (Exists)
- `useAppleSiliconWebSocket` ✅ (Exists)
- Real-time verdict updates ❌ (May not be fully implemented)

**Fix Required:**
```typescript
// Ensure WebSocket hooks implement full real-time functionality
function useCouncilWebSocket() {
  // Should handle:
  // - New verdict notifications
  // - Judge verdict updates
  // - Alert notifications
  // - Ethical assessment updates
}

// Integration with store
useEffect(() => {
  const unsubscribe = useCouncilWebSocket();
  return unsubscribe;
}, []);
```

### 9. **Error Handling Standardization** ⚠️ MEDIUM PRIORITY

**Issue**: Error handling patterns inconsistent with planning document.

**Planning Document Error Handling:**
- Standardized error codes
- User-friendly error messages
- Proper error boundaries
- Retry mechanisms

**Current Implementation:**
- Basic error handling exists
- May not follow planning document patterns

**Fix Required:**
```typescript
// Standardize error handling across all components
interface ApiError {
  code: string;        // Standardized error codes
  message: string;     // User-friendly message
  details?: any;       // Additional error details
  retryable: boolean;  // Whether operation can be retried
}

// Error handling hook
function useErrorHandler() {
  const handleError = (error: ApiError) => {
    // Standardized error handling logic
    switch (error.code) {
      case 'NETWORK_ERROR':
        // Show retry option
        break;
      case 'AUTHENTICATION_ERROR':
        // Redirect to login
        break;
      case 'VALIDATION_ERROR':
        // Show field errors
        break;
      default:
        // Show generic error
        break;
    }
  };

  return { handleError };
}
```

### 10. **Performance Optimization Gaps** ⚠️ LOW PRIORITY

**Issue**: Some performance optimizations from planning document not implemented.

**Planning Document Optimizations:**
- Virtual scrolling for large lists
- Memoization of expensive computations
- Debounced search/filter inputs
- Progressive loading

**Current Implementation:**
- Basic performance exists
- May be missing some optimizations

**Fix Required:**
```typescript
// Implement virtual scrolling for large verdict lists
import { FixedSizeList as List } from 'react-window';

// Debounced search
import { useDebounce } from '@/hooks/useDebounce';

const debouncedSearch = useDebounce(searchQuery, 300);
```

## Implementation Priority

### **Phase 1: Critical Fixes (Week 1)**
1. ✅ **API Endpoint Standardization** - Fix VerdictList API calls
2. ✅ **Data Structure Alignment** - Update Verdict interface
3. ✅ **Backend API Routes** - Create missing API endpoints
4. ✅ **Mock Data Removal** - Implement real API integration

### **Phase 2: Structural Fixes (Week 2)**
1. ⚠️ **State Management Structure** - Add missing ethical assessments
2. ⚠️ **Error Handling Standardization** - Implement consistent patterns
3. ⚠️ **WebSocket Integration** - Complete real-time functionality
4. ⚠️ **Component Naming** - Ensure all planned components exist

### **Phase 3: Optimization (Week 3)**
1. 🟡 **Performance Optimization** - Implement virtual scrolling, debouncing
2. 🟡 **Testing Coverage** - Add comprehensive test coverage
3. 🟡 **Documentation Updates** - Update docs to reflect implementations

## Success Criteria

### **Technical Alignment (100%)**
- ✅ All API endpoints match planning document specifications
- ✅ All data structures align with planning document interfaces
- ✅ All UI components match planning document specifications
- ✅ State management follows planning document patterns

### **Functional Completeness (100%)**
- ✅ Real API integration replaces all mock data
- ✅ WebSocket real-time updates fully functional
- ✅ Error handling follows planning document patterns
- ✅ Performance optimizations implemented

### **Code Quality (95%+)**
- ✅ TypeScript strict mode compliance
- ✅ Comprehensive error handling
- ✅ Accessibility (WCAG 2.1 AA) compliance
- ✅ Performance benchmarks met
