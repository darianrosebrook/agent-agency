<!-- 2bc53c18-b916-4db8-90bd-2f3172f486e9 c80c19c1-ec0f-4b16-bcf4-ab5477646c8e -->
# Schema Divergence Catalog and Realignment Plan

## Phase 1: Comprehensive Catalog Creation

### 1.1 Backend Task Schema Catalog

Document complete backend Task model from `iterations/v3/data-infrastructure/src/models.rs`:

- All 17 fields with types, nullability, constraints
- Backend UpdateTask capabilities (13 updateable fields)
- Status enum values from database constraint
- Priority type (i32) and constraints (0-10)
- JSONB fields: scope, acceptance_criteria, context, caws_spec, metadata

### 1.2 Frontend Task Schema Catalog

Document ALL frontend Task interface variations:

- `apps/agent_management_dashboard/src/lib/api/tasks.ts` - Global Task interface
- `apps/agent_management_dashboard/src/lib/api/projects.ts` - ProjectTask interface
- `apps/agent_management_dashboard/src/lib/schemas/project.ts` - ProjectTaskSchema (Zod)
- `apps/agent_management_dashboard/src/components/projects/ProjectContext.tsx` - Task interface
- `apps/agent_management_dashboard/src/components/projects/TasksTab.tsx` - Task interface
- `apps/agent_management_dashboard/src/components/projects/phase-manager/types.ts` - Task interface
- `apps/agent_management_dashboard/src/components/composers/TimelineTab.tsx` - TimelineTask interface
- Document field names, types, status enums for each

### 1.3 Frontend Update Capabilities Catalog

Document what frontend can actually update:

- `updateProjectTask` function allowed fields (currently 5: title, description, status, priority, assigned_worker_id)
- `UpdateTaskRequestSchema` allowed fields (4: title, description, status, priority, assignee)
- Compare against backend UpdateTask (13 fields)

## Phase 2: Divergence Analysis

### 2.1 Field Inventory Comparison

Create side-by-side comparison tables:

- **Fields Backend Has, Frontend Missing**:
- risk_tier (missing in global Task)
- scope (missing in all frontend types)
- acceptance_criteria (missing in all frontend types)
- context (missing in all frontend types)
- caws_spec (missing in all frontend types)
- deadline (missing in all frontend types)
- project_id (missing in global Task)

- **Fields Frontend Has, Backend Doesn't**:
- type (in global Task - not in backend)
- started_at (in global Task - not in backend)
- assignee (name string - backend has assigned_worker_id UUID only)
- commentCount (TasksTab - not in Task model, separate table)

- **Field Name Inconsistencies**:
- `id` vs `task_id` (backend: id, ProjectTask: task_id)
- `assigned_worker_id` vs `worker_id` vs `assignee` (backend: assigned_worker_id)
- `priority` type: string vs number vs i32

### 2.2 Status Enum Divergence

Document status value mismatches:

- **Backend**: 'pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed'
- **Frontend variations**:
- Zod schemas: 'backlog', 'todo', 'in-progress', 'done'
- Components: Various combinations
- **Mapping needed**: backlog→pending, todo→pending, in-progress→in_progress, done→completed

### 2.3 Type Mismatches

Document type inconsistencies:

- `priority`: Backend `Option<i32>`, Frontend has `string | number | undefined`
- `description`: Backend `String` (required), Frontend `string? | string | null` (optional)
- `assigned_worker_id`: Backend `Option<Uuid>`, Frontend has `string | null` (UUID strings) ✅
- Timestamps: Backend `DateTime<Utc>`, Frontend `string` (RFC3339) ✅

### 2.4 Update Capability Gaps

Document what frontend cannot update but backend supports:

- Missing from `updateProjectTask`: risk_tier, scope, acceptance_criteria, context, caws_spec, deadline, metadata, completed_at, project_id
- Missing from `UpdateTaskRequestSchema`: All JSONB fields, deadline, metadata

## Phase 3: Schema Alignment Document Updates

### 3.1 Update SCHEMA_ALIGNMENT.md

Add comprehensive sections:

- **Complete Field Mapping Table**: All 17 backend fields mapped to all frontend variants
- **Status Enum Mapping Table**: Backend values ↔ Frontend values with transition rules
- **Type Conversion Table**: Backend types ↔ Frontend types with validation rules
- **Update Capability Matrix**: What each frontend function can update vs backend capabilities

### 3.2 Document All Frontend Task Variants

Create section listing all 7+ frontend Task interface definitions:

- Purpose of each (global, project-specific, component-specific)
- Fields each has
- Where each is used
- Which should be canonical

### 3.3 Document Invented Capabilities

List all frontend-invented fields:

- `type` field (not in backend)
- `started_at` field (not in backend)
- `assignee` as name string (backend has UUID only)
- Status values 'backlog', 'todo' (not in backend)

## Phase 4: Realignment Recommendations

### 4.1 Canonical Frontend Task Interface

Recommend single source of truth:

- Base interface matching backend exactly
- Transformations for UI display (e.g., assignee name lookup)
- Status mapping functions (backend→UI, UI→backend)

### 4.2 Status Enum Standardization

Recommend approach:

- Option A: Update all frontend to use backend enum values
- Option B: Create mapping layer (UI uses friendly names, API uses backend values)
- Recommend Option A with UI-friendly labels

### 4.3 Field Name Standardization

Recommend canonical names:

- Use `id` consistently (not `task_id`)
- Use `assigned_worker_id` consistently (not `worker_id` or `assignee`)
- Use `priority` as `number` everywhere (match backend `i32`)

### 4.4 Update API Expansion

Recommend expanding frontend update functions:

- Add all 13 backend UpdateTask fields to `updateProjectTask`
- Update Zod schemas to validate all fields
- Add transformation for JSONB fields

### 4.5 Remove Invented Fields

Recommend removal:

- Remove `type` field (not in backend)
- Remove `started_at` field (not in backend)
- Replace `assignee` (name) with `assigned_worker_id` (UUID) + lookup function

## Phase 5: Implementation Priority

### 5.1 Critical (Blocking Jira-like Workflow)

1. Fix status enum mismatch
2. Expand updateProjectTask to support context, acceptance_criteria, deadline
3. Add missing fields to frontend Task interfaces

### 5.2 High Priority (Feature Completeness)

4. Standardize field names (id, assigned_worker_id, priority type)
5. Add JSONB field support (scope, acceptance_criteria, context, caws_spec)
6. Remove invented fields (type, started_at)

### 5.3 Medium Priority (Cleanup)

7. Consolidate multiple Task interface definitions
8. Create canonical Task type
9. Add type validation/transformation utilities

## Deliverables

1. **SCHEMA_DIVERGENCE_CATALOG.md** - Comprehensive catalog document

- Backend schema reference
- Frontend schema reference (all variants)
- Divergence tables
- Update capability comparison

2. **Updated SCHEMA_ALIGNMENT.md** - Enhanced with catalog data

- Field mapping tables
- Status enum mapping
- Type conversion rules
- Realignment recommendations

3. **Realignment Implementation Plan** - Prioritized action items

- Critical fixes first
- Step-by-step migration path
- Testing requirements
- Rollback considerations

### To-dos

- [ ] Catalog complete backend Task schema from models.rs (17 fields, types, constraints, UpdateTask capabilities)
- [ ] Catalog all frontend Task interface variants (7+ interfaces across tasks.ts, projects.ts, schemas, components)
- [ ] Create divergence analysis: backend-only fields, frontend-only fields, name mismatches, type mismatches, status enum differences
- [ ] Update SCHEMA_ALIGNMENT.md with comprehensive field mapping tables, status enum mapping, type conversion rules
- [ ] Create SCHEMA_DIVERGENCE_CATALOG.md with complete backend/frontend comparison and update capability matrix
- [ ] Create prioritized realignment plan with critical (blocking), high priority, and medium priority fixes