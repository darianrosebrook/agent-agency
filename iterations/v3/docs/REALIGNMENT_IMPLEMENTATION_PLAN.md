# Schema Realignment Implementation Plan

**Date**: 2025-01-28  
**Status**: Ready for Implementation  
**Author**: @darianrosebrook

## Purpose

This document provides a prioritized, step-by-step implementation plan for realigning frontend TypeScript types with backend Rust models. It addresses all schema divergences identified in `SCHEMA_DIVERGENCE_CATALOG.md` and `SCHEMA_ALIGNMENT.md`.

---

## Executive Summary

**Current State**:
- Frontend has 7+ different Task interface definitions
- Frontend supports updating only 5/14 backend fields (36%)
- Status enum mismatch prevents proper workflow transitions
- Missing critical fields: context, acceptance_criteria, deadline

**Target State**:
- Single canonical Task interface matching backend
- Frontend supports updating all 14 backend fields
- Status enum aligned with backend (6 values)
- All backend fields available in frontend

**Implementation Timeline**: 3 weeks (12 phases)

---

## Phase 1: Critical Fixes (Week 1)

### Priority: CRITICAL - Blocking Jira-like Workflow

These fixes are required before agents can properly track their work through the task lifecycle.

#### 1.1 Fix Status Enum Mismatch

**Impact**: Agents cannot properly transition tasks through workflow states

**Current State**:
- Backend: `['pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed']`
- Frontend: `['backlog', 'todo', 'in-progress', 'done']`

**Changes Required**:

1. **Update Zod Schema** (`apps/agent_management_dashboard/src/lib/schemas/project.ts`):
   ```typescript
   // Before
   status: z.enum(['backlog', 'todo', 'in-progress', 'done'])
   
   // After
   status: z.enum(['pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed'])
   ```

2. **Update Component Type Definitions**:
   - `apps/agent_management_dashboard/src/components/projects/ProjectContext.tsx`
   - `apps/agent_management_dashboard/src/components/projects/TasksTab.tsx`
   - `apps/agent_management_dashboard/src/components/composers/TimelineTab.tsx`

3. **Create Status Mapping Utilities** (`apps/agent_management_dashboard/src/lib/utils/taskStatus.ts`):
   ```typescript
   export type BackendTaskStatus = 
     | 'pending' 
     | 'in_progress' 
     | 'paused' 
     | 'completed' 
     | 'cancelled' 
     | 'failed';
   
   export const statusLabels: Record<BackendTaskStatus, string> = {
     pending: 'To Do',
     in_progress: 'In Progress',
     paused: 'Paused',
     completed: 'Done',
     cancelled: 'Cancelled',
     failed: 'Failed'
   };
   
   export function getStatusLabel(status: BackendTaskStatus): string {
     return statusLabels[status];
   }
   ```

4. **Update All UI Components**:
   - Replace status enum values in all components
   - Use `getStatusLabel()` for display
   - Update Kanban board columns
   - Update status dropdowns/selects

**Testing**:
- [ ] Unit tests for status mapping utilities
- [ ] Integration tests for status transitions
- [ ] E2E test: Create task → Update status → Verify persistence
- [ ] Visual regression: Kanban board displays correct statuses

**Risk**: Medium - Affects all task status displays  
**Rollback**: Feature flag for new status enum, revert to old values if issues

---

#### 1.2 Expand updateProjectTask to Support Context

**Impact**: Agents cannot update task context as they work (CRITICAL for agent workflow)

**Current State**: `context` field exists in backend but cannot be updated via frontend

**Changes Required**:

1. **Update ProjectTask Interface** (`apps/agent_management_dashboard/src/lib/api/projects.ts`):
   ```typescript
   export interface ProjectTask {
     // ... existing fields ...
     context?: Record<string, unknown> | null;
   }
   ```

2. **Expand updateProjectTask Function**:
   ```typescript
   updates: Partial<
     Pick<
       ProjectTask,
       | "title"
       | "description"
       | "status"
       | "priority"
       | "assigned_worker_id"
       | "context" // ADD THIS
     >
   >
   ```

3. **Update Zod Schema** (`apps/agent_management_dashboard/src/lib/schemas/project.ts`):
   ```typescript
   export const UpdateTaskRequestSchema = z.object({
     // ... existing fields ...
     context: z.record(z.string(), z.unknown()).optional(),
   });
   ```

**Testing**:
- [ ] Integration test: Update task context via API
- [ ] E2E test: Agent updates context → Verify persistence
- [ ] Verify context updates appear in PhaseManager context chips

**Risk**: Low - Adding field, no breaking changes  
**Rollback**: Remove `context` from allowed updates

---

#### 1.3 Expand updateProjectTask to Support Acceptance Criteria

**Impact**: Cannot refine acceptance criteria during execution (CRITICAL for agent workflow)

**Changes Required**:

1. **Update ProjectTask Interface**:
   ```typescript
   acceptance_criteria?: unknown[] | null;
   ```

2. **Expand updateProjectTask Function**:
   ```typescript
   updates: Partial<
     Pick<
       ProjectTask,
       | "title"
       | "description"
       | "status"
       | "priority"
       | "assigned_worker_id"
       | "context"
       | "acceptance_criteria" // ADD THIS
     >
   >
   ```

3. **Update Zod Schema**:
   ```typescript
   acceptance_criteria: z.array(z.unknown()).optional(),
   ```

**Testing**:
- [ ] Integration test: Update acceptance criteria via API
- [ ] E2E test: Refine acceptance criteria → Verify persistence
- [ ] Verify acceptance criteria display in UI

**Risk**: Low - Adding field  
**Rollback**: Remove from allowed updates

---

#### 1.4 Add Missing Critical Fields to Frontend Interfaces

**Impact**: Frontend cannot receive or display backend fields that exist in API responses

**Fields to Add**:
- `context` (already added in 1.2)
- `acceptance_criteria` (already added in 1.3)
- `scope`
- `caws_spec`
- `deadline`

**Changes Required**:

1. **Update Global Task Interface** (`apps/agent_management_dashboard/src/lib/api/tasks.ts`):
   ```typescript
   export interface Task {
     // ... existing fields ...
     risk_tier?: string; // ADD
     scope?: Record<string, unknown>; // ADD
     acceptance_criteria?: unknown[]; // ADD
     context?: Record<string, unknown>; // ADD
     caws_spec?: Record<string, unknown> | null; // ADD
     deadline?: string | null; // ADD (RFC3339)
     project_id?: string | null; // ADD
   }
   ```

2. **Update ProjectTask Interface**:
   ```typescript
   export interface ProjectTask {
     // ... existing fields ...
     scope?: Record<string, unknown> | null; // ADD
     acceptance_criteria?: unknown[] | null; // ADD
     context?: Record<string, unknown> | null; // ADD
     caws_spec?: Record<string, unknown> | null; // ADD
     deadline?: string | null; // ADD
   }
   ```

**Testing**:
- [ ] Verify API responses include these fields
- [ ] Verify fields are accessible in components
- [ ] Verify fields can be displayed in UI (even if read-only initially)

**Risk**: Low - Adding optional fields  
**Rollback**: Remove fields from interfaces

---

## Phase 2: High Priority (Week 2)

### Priority: HIGH - Feature Completeness

These fixes complete the feature set and standardize types.

#### 2.5 Expand updateProjectTask to All Backend Fields

**Impact**: Frontend can now update all fields backend supports

**Changes Required**:

1. **Expand updateProjectTask Function**:
   ```typescript
   updates: Partial<{
     title?: string;
     description?: string;
     risk_tier?: string;
     scope?: Record<string, unknown>;
     acceptance_criteria?: unknown[];
     context?: Record<string, unknown>;
     caws_spec?: Record<string, unknown> | null;
     status?: BackendTaskStatus;
     assigned_worker_id?: string | null;
     project_id?: string | null;
     priority?: number | null;
     deadline?: string | null;
     metadata?: Record<string, unknown> | null;
     completed_at?: string | null;
   }>;
   ```

2. **Update Zod Schema**:
   ```typescript
   export const UpdateTaskRequestSchema = z.object({
     title: z.string().optional(),
     description: z.string().optional(),
     risk_tier: z.enum(['1', '2', '3']).optional(),
     scope: z.record(z.string(), z.unknown()).optional(),
     acceptance_criteria: z.array(z.unknown()).optional(),
     context: z.record(z.string(), z.unknown()).optional(),
     caws_spec: z.record(z.string(), z.unknown()).nullable().optional(),
     status: z.enum(['pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed']).optional(),
     assigned_worker_id: z.string().uuid().nullable().optional(),
     project_id: z.string().uuid().nullable().optional(),
     priority: z.number().int().min(0).max(10).nullable().optional(),
     deadline: z.string().datetime().nullable().optional(),
     metadata: z.record(z.string(), z.unknown()).nullable().optional(),
     completed_at: z.string().datetime().nullable().optional(),
   });
   ```

**Testing**:
- [ ] Integration tests for each new field
- [ ] E2E test: Update all fields → Verify persistence
- [ ] Verify validation errors for invalid values

**Risk**: Medium - Expands API surface significantly  
**Rollback**: Feature flag to limit to original 5 fields

---

#### 2.6 Standardize Field Names

**Impact**: Consistent field names across all frontend code

**Changes Required**:

1. **Standardize `id` vs `task_id`**:
   - Update `ProjectTask` interface: `task_id` → `id`
   - Update API response mapping in `getProjectTasks`
   - Update all components using `task_id`

2. **Standardize `assigned_worker_id` vs `worker_id` vs `assignee`**:
   - Update Global Task interface: `worker_id` → `assigned_worker_id`
   - Update Zod schemas: `assignee` → `assigned_worker_id`
   - Update all components

3. **Standardize timestamp field names**:
   - Use `created_at` / `updated_at` consistently (not `createdAt` / `updatedAt`)
   - Update Zod schemas

**Testing**:
- [ ] Verify no broken references after renaming
- [ ] Integration tests verify field names in API calls
- [ ] E2E tests verify UI still works

**Risk**: Medium - Requires updating multiple files  
**Rollback**: Revert field name changes

---

#### 2.7 Fix Type Mismatches

**Impact**: Type safety and consistency

**Changes Required**:

1. **Fix `priority` Type**:
   - Update Global Task interface: `priority?: string` → `priority?: number`
   - Update TasksTab priority mapping to convert number to display label
   - Update all components using priority

2. **Fix `description` Nullability**:
   - Decision: Make backend optional OR make frontend required
   - Recommended: Make backend optional (less breaking)
   - Update backend CreateTask to allow null description
   - OR: Update frontend to require description

**Testing**:
- [ ] TypeScript compilation with strict types
- [ ] Integration tests verify priority as number
- [ ] Verify UI displays priority correctly

**Risk**: Medium - Type changes may break existing code  
**Rollback**: Revert type changes

---

#### 2.8 Remove Invented Fields

**Impact**: Frontend no longer uses fields that don't exist in backend

**Changes Required**:

1. **Remove `type` Field**:
   - Remove from Global Task interface
   - Remove from API response mapping
   - Update components that use `task.type` (extract from metadata instead)

2. **Remove `started_at` Field**:
   - Remove from Global Task interface
   - Update components that use `started_at` (may not be needed, or use `created_at`)

3. **Replace `assignee` (name string) with `assigned_worker_id` (UUID)**:
   - Update all schemas and interfaces
   - Create worker lookup utility: `getWorkerName(workerId: string): Promise<string>`
   - Update UI components to fetch worker name when displaying assignee

**Testing**:
- [ ] Verify no references to removed fields
- [ ] E2E test: Verify UI still works without these fields
- [ ] Verify worker name lookup works

**Risk**: High - Removing fields may break UI  
**Rollback**: Add fields back, keep deprecated

---

## Phase 3: Medium Priority (Week 3)

### Priority: MEDIUM - Code Quality Improvements

These are cleanup tasks to improve maintainability.

#### 2.9 Create Canonical Task Interface

**Impact**: Single source of truth for Task type

**Changes Required**:

1. **Create New File** (`apps/agent_management_dashboard/src/lib/types/task.ts`):
   ```typescript
   export type BackendTaskStatus = 
     | 'pending' 
     | 'in_progress' 
     | 'paused' 
     | 'completed' 
     | 'cancelled' 
     | 'failed';
   
   export interface Task {
     id: string;
     title: string;
     description: string; // Required
     risk_tier: string;
     scope: Record<string, unknown>;
     acceptance_criteria: unknown[];
     context: Record<string, unknown>;
     caws_spec?: Record<string, unknown> | null;
     status: BackendTaskStatus;
     assigned_worker_id?: string | null;
     project_id?: string | null;
     priority?: number | null; // 0-10
     deadline?: string | null; // RFC3339
     created_at: string; // RFC3339
     updated_at: string; // RFC3339
     completed_at?: string | null; // RFC3339
     metadata?: Record<string, unknown> | null;
   }
   ```

2. **Update API Interfaces** to extend canonical Task:
   ```typescript
   // Global Task - extends canonical
   export interface GlobalTask extends Task {
     // No additional fields needed
   }
   
   // ProjectTask - extends canonical
   export interface ProjectTask extends Task {
     // No additional fields needed
   }
   ```

**Testing**:
- [ ] Verify canonical interface matches backend exactly
- [ ] Verify TypeScript compilation

**Risk**: Low - New file, doesn't break existing code  
**Rollback**: Remove file, no impact

---

#### 2.10 Consolidate Multiple Task Interface Definitions

**Impact**: Reduced maintenance burden, single source of truth

**Changes Required**:

1. **Migrate Components to Canonical Task**:
   - Update ProjectContext Task → use canonical Task
   - Update TasksTab Task → use canonical Task
   - Update PhaseManager Task → use canonical Task with derived fields
   - Update TimelineTask → use canonical Task with display transforms

2. **Create UI-Specific Transformation Types** (only where needed):
   ```typescript
   // For Kanban board
   export interface KanbanTask extends Task {
     commentCount?: number; // Derived from separate query
   }
   
   // For Timeline
   export interface TimelineTaskDisplay {
     task: Task;
     worker: string; // Worker name (derived)
     startDate: Date;
     endDate: Date;
     tags: string[]; // Derived from metadata
   }
   ```

**Testing**:
- [ ] Verify all components compile
- [ ] E2E tests verify UI still works
- [ ] Visual regression: No UI changes

**Risk**: Medium - Touches many components  
**Rollback**: Revert to component-specific interfaces

---

#### 2.11 Add Type Validation/Transformation Utilities

**Impact**: Centralized utilities for type conversions

**Changes Required**:

1. **Create Status Mapping Utilities** (`apps/agent_management_dashboard/src/lib/utils/taskStatus.ts`):
   - Status enum definitions
   - Status label mapping
   - Status transition validation

2. **Create Worker Utilities** (`apps/agent_management_dashboard/src/lib/utils/worker.ts`):
   - Worker ID → name lookup
   - Worker name caching

3. **Create Priority Utilities** (`apps/agent_management_dashboard/src/lib/utils/priority.ts`):
   - Priority number → label conversion (0-3: "low", 4-6: "medium", 7-10: "high")
   - Priority label → number conversion

**Testing**:
- [ ] Unit tests for all utilities
- [ ] Edge case testing
- [ ] Performance testing for worker lookup

**Risk**: Low - New utilities, doesn't break existing code  
**Rollback**: Remove utilities, no impact

---

#### 2.12 Add Runtime Schema Validation

**Impact**: Catch API response mismatches early

**Changes Required**:

1. **Create Runtime Validation** (`apps/agent_management_dashboard/src/lib/api/validation.ts`):
   ```typescript
   import { z } from 'zod';
   import { TaskSchema } from '../types/task'; // Zod schema matching Task interface
   
   export function validateTaskResponse(data: unknown): Task {
     return TaskSchema.parse(data);
   }
   ```

2. **Add Validation to API Client Functions**:
   ```typescript
   export async function listTasks(): Promise<TasksListResponse> {
     const response = await apiGet<TasksListResponse>(`${API_BASE}/tasks`);
     // Validate each task
     response.tasks = response.tasks.map(validateTaskResponse);
     return response;
   }
   ```

3. **Add Error Handling**:
   - Log validation errors
   - Return partial data with warnings
   - Report to error tracking service

**Testing**:
- [ ] Unit tests for validation
- [ ] Integration tests with invalid API responses
- [ ] Verify error handling works correctly

**Risk**: Low - Adds validation, doesn't break existing code  
**Rollback**: Remove validation, return data as-is

---

## Testing Strategy

### Unit Tests

- Type conversion utilities
- Status mapping functions
- Priority conversion functions
- Schema validation functions

### Integration Tests

- API client functions with all field updates
- Status transition workflows
- Field mapping between backend and frontend
- Error handling for invalid data

### E2E Tests

**Jira-like Workflow Test**:
1. Create task → Verify `pending` status
2. Assign to worker → Verify assignment persists
3. Update status to `in_progress` → Verify transition
4. Update context → Verify context persists
5. Update acceptance criteria → Verify criteria persists
6. Update status to `completed` → Verify completion
7. Verify `completed_at` timestamp is set

### Regression Tests

- Existing task creation flows
- Existing task update flows
- Existing status transitions (with new enum values)
- UI displays (Kanban, Timeline, etc.)

---

## Rollback Plan

Each phase should be implemented in separate PRs with feature flags:

```typescript
const ENABLE_EXPANDED_TASK_UPDATES = process.env.ENABLE_EXPANDED_TASK_UPDATES === 'true';

if (ENABLE_EXPANDED_TASK_UPDATES) {
  // New code with all 14 fields
} else {
  // Old code with 5 fields
}
```

**Rollback Triggers**:
- Integration test failures
- E2E test failures
- Production errors
- Performance degradation

**Rollback Steps**:
1. Disable feature flag
2. Revert PR
3. Deploy hotfix
4. Investigate issue
5. Fix and re-enable

---

## Success Metrics

**Coverage Metrics**:
- [ ] Frontend Task interface: 17/17 fields (100%)
- [ ] Frontend update capabilities: 14/14 fields (100%)
- [ ] Status enum alignment: 6/6 values (100%)

**Functional Metrics**:
- [ ] All E2E tests passing
- [ ] No type errors in TypeScript compilation
- [ ] Zero validation errors in production logs
- [ ] Agent workflow fully functional

**Code Quality Metrics**:
- [ ] Single canonical Task interface
- [ ] No invented fields
- [ ] Consistent field naming
- [ ] All components using canonical types

---

**Last Updated**: 2025-01-28  
**Reference Documents**:
- `SCHEMA_DIVERGENCE_CATALOG.md` - Complete field comparison
- `SCHEMA_ALIGNMENT.md` - Detailed alignment analysis
- `REALIGNMENT_IMPLEMENTATION_PLAN.md` - This document