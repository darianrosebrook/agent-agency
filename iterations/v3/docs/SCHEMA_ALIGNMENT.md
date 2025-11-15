# Frontend-Backend Schema Alignment

**Date**: 2025-11-15  
**Status**: 📋 **DOCUMENTATION IN PROGRESS**  
**Author**: @darianrosebrook

## Summary

This document tracks the alignment between backend Rust data models and frontend TypeScript types, ensuring both sides use the same field names and data structures for API communication.

## Current Status

### ✅ Schemas Defined

**Backend (Rust)**:

- Location: `iterations/v3/data-infrastructure/src/models.rs`
- Uses: `serde`, `schemars::JsonSchema`, `sqlx::FromRow`
- Format: Snake case field names (matching database)

**Frontend (TypeScript)**:

- Location: `apps/agent_management_dashboard/src/lib/schemas/`
- Uses: Zod schemas for validation
- Format: Mixed (API responses use snake_case, internal types use camelCase)

**Contract Schemas**:

- Location: `iterations/v3/docs/contracts/*.schema.json`
- JSON Schema definitions for contracts (WorkingSpec, WorkerOutput, etc.)

### ⚠️ Schema Alignment Issues

There are **field mismatches** and **missing fields** between backend and frontend schemas.

---

## Task Schema Alignment

### Backend Rust Task Model

```rust
// iterations/v3/data-infrastructure/src/models.rs
pub struct Task {
    pub id: Uuid,                        // ✅ Matches
    pub title: String,                   // ✅ Matches
    pub description: String,             // ✅ Matches
    pub risk_tier: String,               // ⚠️ Missing in some frontend types
    pub scope: serde_json::Value,        // ❌ Missing in frontend
    pub acceptance_criteria: serde_json::Value,  // ❌ Missing in frontend
    pub context: serde_json::Value,      // ❌ Missing in frontend
    pub caws_spec: Option<serde_json::Value>,    // ❌ Missing in frontend
    pub status: String,                  // ✅ Matches
    pub assigned_worker_id: Option<Uuid>, // ⚠️ Named differently (worker_id vs assigned_worker_id)
    pub project_id: Option<Uuid>,        // ⚠️ Missing in Task interface (present in ProjectTask)
    pub created_at: DateTime<Utc>,       // ✅ Matches (snake_case)
    pub updated_at: DateTime<Utc>,       // ✅ Matches (snake_case)
    pub completed_at: Option<DateTime<Utc>>,  // ✅ Matches
    pub priority: Option<i32>,           // ⚠️ Type mismatch (i32 vs string)
    pub deadline: Option<DateTime<Utc>>, // ❌ Missing in frontend
    pub metadata: Option<serde_json::Value>,  // ✅ Matches
}
```

### Frontend TypeScript Task Types

**Global Task Interface** (`apps/agent_management_dashboard/src/lib/api/tasks.ts`):

```typescript
export interface Task {
  id: string; // ✅ Matches
  title: string; // ✅ Matches
  description?: string; // ✅ Matches (optional in frontend, required in backend)
  priority?: string; // ⚠️ Type mismatch (string vs i32)
  type?: string; // ❌ Not in backend
  status: string; // ✅ Matches
  created_at: string; // ✅ Matches (snake_case for API)
  updated_at: string; // ✅ Matches (snake_case for API)
  started_at?: string; // ❌ Not in backend
  completed_at?: string | null; // ✅ Matches
  worker_id?: string | null; // ⚠️ Named differently (assigned_worker_id in backend)
  metadata?: Record<string, unknown>; // ✅ Matches
}
```

**Project Task Interface** (`apps/agent_management_dashboard/src/lib/api/projects.ts`):

```typescript
export interface ProjectTask {
  task_id: string; // ⚠️ Named differently (id in backend)
  title: string; // ✅ Matches
  description?: string | null; // ✅ Matches
  status: string; // ✅ Matches
  risk_tier?: string | null; // ✅ Matches (present!)
  priority?: number | null; // ✅ Matches type (number vs i32)
  assigned_worker_id?: string | null; // ✅ Matches name!
  created_at: string; // ✅ Matches
  updated_at: string; // ✅ Matches
  completed_at?: string | null; // ✅ Matches
  // ❌ Missing: project_id (but makes sense - already in URL path)
  // ❌ Missing: scope, acceptance_criteria, context, caws_spec
  // ❌ Missing: deadline
}
```

### Issues Identified

#### 1. **Field Name Inconsistencies**

| Backend Field        | Frontend Global Task | Frontend ProjectTask    | Status            |
| -------------------- | -------------------- | ----------------------- | ----------------- |
| `id`                 | ✅ `id`              | ⚠️ `task_id`            | **Inconsistent**  |
| `assigned_worker_id` | ⚠️ `worker_id`       | ✅ `assigned_worker_id` | **Inconsistent**  |
| `priority`           | ⚠️ `string`          | ✅ `number`             | **Type mismatch** |

#### 2. **Missing Fields in Frontend**

**Global Task Interface** missing:

- `risk_tier` - Risk tier information
- `project_id` - Project association
- `assigned_worker_id` - Worker assignment (has `worker_id` instead)
- `scope` - Task scope definition
- `acceptance_criteria` - Acceptance criteria
- `context` - Task context
- `caws_spec` - CAWS specification reference
- `deadline` - Task deadline
- `started_at` - Present in frontend but not backend

**Project Task Interface** missing:

- `scope` - Task scope definition
- `acceptance_criteria` - Acceptance criteria
- `context` - Task context
- `caws_spec` - CAWS specification reference
- `deadline` - Task deadline
- `id` - Uses `task_id` instead (should standardize)

#### 3. **Type Mismatches**

| Field | Backend Type | Frontend Types | Issue |
|-------|--------------|----------------|-------|
| `priority` | `Option<i32>` (0-10) | `string` (Global Task) / `number` (ProjectTask) / `"low"\|"medium"\|"high"` (TasksTab) | **Three different types** |
| `description` | `String` (required) | `string?` / `string \| null` (optional) | **Nullability mismatch** |
| `assigned_worker_id` | `Option<Uuid>` | `string \| null` (UUID strings) | ✅ **Type conversion OK** |
| `id` | `Uuid` | `string` (UUID strings) | ✅ **Type conversion OK** |
| `created_at` / `updated_at` | `DateTime<Utc>` | `string` (RFC3339) / `Date` (Zod) | ✅ **Type conversion OK** |
| `completed_at` / `deadline` | `Option<DateTime<Utc>>` | `string \| null` (RFC3339) | ✅ **Type conversion OK** |

### Type Conversion Rules

**UUID Types**:
- Backend: `Uuid` → Frontend: `string` (UUID string representation)
- Conversion: `.to_string()` (Rust) → `string` (TypeScript)
- Validation: Use Zod `.uuid()` or manual UUID validation

**DateTime Types**:
- Backend: `DateTime<Utc>` → Frontend: `string` (RFC3339 format)
- Conversion: `.to_rfc3339()` (Rust) → `new Date(string)` (TypeScript)
- Format: `"2025-01-01T00:00:00Z"`

**Priority Type**:
- Backend: `Option<i32>` with constraint `CHECK (>= 0 AND <= 10)`
- Frontend Current: `string` (Global Task) / `number` (ProjectTask) / `"low"\|"medium"\|"high"` (TasksTab)
- **Recommended**: `number | null` matching backend `i32` (0-10)
- **UI Display**: Map number to string labels for display (0-3: "low", 4-6: "medium", 7-10: "high")

**JSONB Types**:
- Backend: `serde_json::Value` → Frontend: `Record<string, unknown>` or typed interface
- Conversion: Serialize/deserialize JSON
- Validation: Use Zod schemas for runtime validation of JSONB content

#### 4. **🚨 CRITICAL: Status Workflow Mismatch**

**Backend Task Status** (database constraint):

```sql
status VARCHAR(50) CHECK (status IN ('pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed'))
```

**Frontend Task Status** (multiple variations):

1. **Zod Schema**: `z.enum(["backlog", "todo", "in-progress", "done"])`
2. **ProjectContext**: `"backlog" | "todo" | "in-progress" | "done"`
3. **TimelineTab**: `"completed" | "in-progress" | "pending"` (simplified 3-value)
4. **Global Task**: `string` (no enum validation)

**Impact**: **Status values don't match!** This will break the Jira-like workflow where agents update task status.

**Backend Status Values** (6 values):

| Backend Value | Description | Frontend Equivalent | Mapping Issue |
|---------------|-------------|---------------------|---------------|
| `pending` | Task is waiting to be assigned/started | `backlog` or `todo` | ❌ No direct mapping |
| `in_progress` | Task is actively being worked on | `in-progress` | ⚠️ Underscore vs hyphen |
| `paused` | Task is paused (can be resumed) | None | ❌ **Not supported in frontend** |
| `completed` | Task is finished successfully | `done` | ❌ Different name |
| `cancelled` | Task was cancelled | None | ❌ **Not supported in frontend** |
| `failed` | Task failed during execution | None | ❌ **Not supported in frontend** |

**Frontend Status Values** (4 values):

| Frontend Value | Intended Meaning | Backend Equivalent | Mapping Issue |
|----------------|------------------|-------------------|---------------|
| `backlog` | Task in backlog | `pending` | ⚠️ Semantic difference |
| `todo` | Task to do | `pending` | ⚠️ Semantic difference |
| `in-progress` | Task in progress | `in_progress` | ⚠️ Hyphen vs underscore |
| `done` | Task completed | `completed` | ❌ Different name |

### Status Enum Mapping Table

| Backend → Frontend | Frontend → Backend | Transition Rules |
|-------------------|-------------------|------------------|
| `pending` → `todo` | `backlog` → `pending` | Both frontend values map to `pending` |
| `pending` → `backlog` | `todo` → `pending` | Use `todo` for newly created tasks |
| `in_progress` → `in-progress` | `in-progress` → `in_progress` | Replace hyphen with underscore |
| `paused` → ❌ No mapping | ❌ None → `paused` | Frontend cannot set paused |
| `completed` → `done` | `done` → `completed` | Map `done` to `completed` |
| `cancelled` → ❌ No mapping | ❌ None → `cancelled` | Frontend cannot set cancelled |
| `failed` → ❌ No mapping | ❌ None → `failed` | Frontend cannot set failed |

**Status Transition Flow** (Jira-like):

1. Create task → `pending` (backend) / `todo` (frontend) ❌ **MISMATCH**
2. Assign to worker → `pending` → `in_progress` (backend) / `todo` → `in-progress` (frontend) ⚠️ **INCONSISTENT**
3. Pause task → `paused` (backend) / ❌ **NOT SUPPORTED** (frontend)
4. Complete task → `completed` (backend) / `done` (frontend) ❌ **MISMATCH**
5. Fail task → `failed` (backend) / ❌ **NOT SUPPORTED** (frontend)
6. Cancel task → `cancelled` (backend) / ❌ **NOT SUPPORTED** (frontend)

#### 5. **🚨 CRITICAL: Update Task API Limitations**

**Backend UpdateTask** supports:

```rust
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub risk_tier: Option<String>,
    pub scope: Option<serde_json::Value>,
    pub acceptance_criteria: Option<serde_json::Value>,
    pub context: Option<serde_json::Value>,
    pub caws_spec: Option<serde_json::Value>,
    pub status: Option<String>,
    pub assigned_worker_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub priority: Option<i32>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

**Frontend updateProjectTask** only allows:

```typescript
updates: Partial<
  Pick<
    ProjectTask,
    "title" | "description" | "status" | "priority" | "assigned_worker_id"
  >
>;
```

**Missing update capabilities**:

- ❌ `risk_tier` - Cannot update risk tier
- ❌ `scope` - Cannot update scope
- ❌ `acceptance_criteria` - Cannot update acceptance criteria
- ❌ `context` - Cannot update context (critical for agent workflow!)
- ❌ `caws_spec` - Cannot update CAWS spec
- ❌ `deadline` - Cannot update deadline
- ❌ `metadata` - Cannot update metadata
- ❌ `completed_at` - Cannot mark as completed with timestamp

---

## Project Schema Alignment

### Backend Rust Project Model

```rust
// From database operations (ExecutionPlan)
pub struct ExecutionPlan {
    pub id: Uuid,
    pub title: String,              // Maps to "name" in frontend
    pub overview: Option<String>,   // Maps to "description" or "summary"
    pub state: Option<String>,      // Maps to "status" or not used
    pub milestones: Option<serde_json::Value>,
    pub dependency_graph: Option<serde_json::Value>,
    pub change_budget: Option<serde_json::Value>,
    pub quality_gates: Option<serde_json::Value>,
    pub evidence_requirements: Option<serde_json::Value>,
    pub active_waivers: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Frontend TypeScript Project Types

```typescript
// apps/agent_management_dashboard/src/lib/api/projects.ts
export interface ProjectApiResponse {
  id: string;
  name: string; // Maps from backend "title"
  summary?: string | null; // Maps from backend "overview"
  description?: string | null; // Also maps from backend "overview"?
  state?: string | null; // Maps from backend "state"
  created_at: string;
  updated_at?: string | null;
  last_accessed?: string | null; // ❌ Not in backend
  milestones?: ProjectMilestone[]; // ✅ Extracted from JSON
  // ❌ Missing: dependency_graph, change_budget, quality_gates, evidence_requirements, active_waivers, metadata
}
```

### Issues Identified

1. **Field Name Mapping**:

   - Backend `title` → Frontend `name` ✅ (handled in API layer)
   - Backend `overview` → Frontend `summary` or `description` ⚠️ (unclear which)
   - Backend `state` → Frontend `state` ✅ (present)

2. **Missing Fields**:

   - `dependency_graph` - Project dependency information
   - `change_budget` - Change budget constraints
   - `quality_gates` - Quality gate definitions
   - `evidence_requirements` - Evidence requirements
   - `active_waivers` - Active waivers
   - `metadata` - Additional metadata

3. **Extra Fields in Frontend**:
   - `last_accessed` - Not in backend (could be calculated or added)

---

## Workflow Capabilities (Jira-like)

### ✅ Supported Workflows

1. **Task Assignment**:

   - ✅ Backend: `assigned_worker_id` field exists
   - ✅ Frontend: `updateProjectTask` supports `assigned_worker_id`
   - ⚠️ Frontend Global Task uses `worker_id` instead (inconsistent)

2. **Task Comments**:

   - ✅ Backend: Task comments API exists (`/api/v1/tasks/:task_id/comments`)
   - ✅ Frontend: Comments API client exists (`apps/agent_management_dashboard/src/lib/api/comments.ts`)
   - ✅ Schema alignment: Frontend `comment_id` matches backend `id` (returned as `comment_id`)
   - ✅ CRUD operations: GET, POST, PATCH, DELETE all supported

3. **Task Status Updates**:

   - ✅ Backend: `PATCH /api/v1/tasks/:task_id` supports `status` field
   - ✅ Frontend: `updateProjectTask` supports `status` field
   - 🚨 **CRITICAL**: Status values don't match (see Status Workflow Mismatch above)

4. **Task Priority Updates**:
   - ✅ Backend: `priority` field exists (`Option<i32>`)
   - ✅ Frontend ProjectTask: `priority` field exists (`number`)
   - ⚠️ Frontend Global Task: `priority` is `string` (type mismatch)

### ❌ Missing Workflow Capabilities

1. **Context Updates**:

   - ❌ Frontend cannot update `context` field (not in `updateProjectTask`)
   - **Impact**: Agents cannot update task context as they work

2. **Acceptance Criteria Updates**:

   - ❌ Frontend cannot update `acceptance_criteria` field
   - **Impact**: Cannot refine acceptance criteria during execution

3. **Scope Updates**:

   - ❌ Frontend cannot update `scope` field
   - **Impact**: Cannot adjust scope as task evolves

4. **Deadline Management**:

   - ❌ Frontend cannot set/update `deadline` field
   - **Impact**: Cannot track deadlines for tasks

5. **Status Workflow**:

   - ❌ Frontend status enum doesn't match backend
   - ❌ Missing: `paused`, `failed`, `cancelled` statuses in frontend
   - ❌ Frontend uses `backlog`, `todo`, `done` which don't exist in backend
   - **Impact**: Agents cannot properly track task status through workflow

6. **Task Completion Tracking**:
   - ❌ Frontend cannot set `completed_at` timestamp
   - **Impact**: Cannot track when tasks were actually completed

## Recommendations

### 🚨 Critical Immediate Actions

1. **Fix Status Workflow Mismatch** (BLOCKING):

   - **Option A**: Update frontend status enum to match backend:
     ```typescript
     status: z.enum([
       "pending",
       "in_progress",
       "paused",
       "completed",
       "cancelled",
       "failed",
     ]);
     ```
   - **Option B**: Update backend to support frontend statuses (not recommended - breaks existing data)
   - **Recommended**: Option A - Frontend should match backend database constraints
   - Add status mapping functions if needed for UI display

2. **Expand updateProjectTask Capabilities** (BLOCKING):

   - Add all missing fields to `updateProjectTask`:
     ```typescript
     // Current: 5 fields
     updates: Partial<Pick<ProjectTask, "title" | "description" | "status" | "priority" | "assigned_worker_id">>
     
     // Recommended: All 14 backend updateable fields
     updates: Partial<{
       title?: string;
       description?: string;
       risk_tier?: string;
       scope?: Record<string, unknown>;
       acceptance_criteria?: unknown[];
       context?: Record<string, unknown>;
       caws_spec?: Record<string, unknown> | null;
       status?: "pending" | "in_progress" | "paused" | "completed" | "cancelled" | "failed";
       assigned_worker_id?: string | null;
       project_id?: string | null;
       priority?: number | null; // 0-10
       deadline?: string | null; // RFC3339
       metadata?: Record<string, unknown> | null;
       completed_at?: string | null; // RFC3339
     }>;
     ```
   - **Impact**: Enables full Jira-like workflow for agents
   - **Update Zod Schema**: Expand `UpdateTaskRequestSchema` to validate all fields

3. **Standardize Task Field Names**:

   - Use `assigned_worker_id` consistently (not `worker_id`)
   - Use `id` consistently (not `task_id`)
   - Align `priority` type (use `number` everywhere, match backend `i32`)

4. **Add Missing Fields to Frontend**:

   - Add `risk_tier` to global Task interface
   - Add `project_id` to global Task interface
   - Add `scope`, `acceptance_criteria`, `context`, `caws_spec` to ProjectTask interface
   - Add `deadline` to both Task interfaces
   - Remove `started_at` from frontend (not in backend)
   - Remove `type` from frontend (not in backend)

5. **Fix Type Mismatches**:
   - Change `priority` from `string` to `number` in global Task interface
   - Make `description` required in backend or optional in both (currently mismatched)

### Documentation Actions

4. **Create Field Mapping Document**:

   - Document all field name conversions (snake_case ↔ camelCase)
   - Document API response transformations
   - Document missing fields and why

5. **Generate Schema Validation**:
   - Use backend `JsonSchema` to generate TypeScript types
   - Add runtime validation in frontend using generated types
   - Add CI/CD checks for schema alignment

---

## Complete Field Mapping Reference

### Task Fields - Complete Backend to Frontend Mapping

| Backend (Rust)                           | JSON API                                         | Global Task Interface | ProjectTask Interface | Zod Schema | Status |
| ---------------------------------------- | ------------------------------------------------ | -------------------- | -------------------- | ---------- | ------ |
| `id: Uuid`                               | `"id": "uuid-string"`                            | ✅ `id: string`      | ⚠️ `task_id: string` | ✅ `id: string` | ⚠️ Inconsistent |
| `title: String`                          | `"title": "string"`                              | ✅ `title: string`   | ✅ `title: string`   | ✅ `title: string` | ✅ Matches |
| `description: String`                    | `"description": "string"`                        | ⚠️ `description?: string` | ⚠️ `description?: string \| null` | ⚠️ `description?: string` | ⚠️ Nullability mismatch |
| `risk_tier: String`                      | `"risk_tier": "string"`                          | ❌ Missing           | ✅ `risk_tier?: string \| null` | ❌ Missing | ⚠️ Partially missing |
| `scope: serde_json::Value`               | `"scope": {...}`                                 | ❌ Missing           | ❌ Missing           | ❌ Missing | ❌ Missing everywhere |
| `acceptance_criteria: serde_json::Value` | `"acceptance_criteria": {...}`                   | ❌ Missing           | ❌ Missing           | ❌ Missing | ❌ Missing everywhere |
| `context: serde_json::Value`             | `"context": {...}`                               | ❌ Missing           | ❌ Missing           | ❌ Missing | ❌ Missing everywhere (CRITICAL) |
| `caws_spec: Option<serde_json::Value>`   | `"caws_spec": {...} \| null`                     | ❌ Missing           | ❌ Missing           | ❌ Missing | ❌ Missing everywhere |
| `status: String`                         | `"status": "string"`                             | ⚠️ `status: string` (enum mismatch) | ⚠️ `status: string` (enum mismatch) | 🚨 `status: enum(['backlog'...])` | 🚨 CRITICAL: Enum mismatch |
| `assigned_worker_id: Option<Uuid>`       | `"assigned_worker_id": "uuid-string" \| null`    | ⚠️ `worker_id?: string` | ✅ `assigned_worker_id?: string` | ⚠️ `assignee?: string` | ⚠️ Three different names |
| `project_id: Option<Uuid>`               | `"project_id": "uuid-string" \| null`            | ❌ Missing           | ✅ (inferred from URL) | ❌ Missing | ⚠️ Missing in global |
| `priority: Option<i32>`                  | `"priority": 1 \| null`                          | ⚠️ `priority?: string` | ✅ `priority?: number` | ⚠️ `priority?: string` | ⚠️ Type mismatch |
| `deadline: Option<DateTime<Utc>>`        | `"deadline": "2025-12-31T00:00:00Z" \| null`     | ❌ Missing           | ❌ Missing           | ❌ Missing | ❌ Missing everywhere |
| `created_at: DateTime<Utc>`              | `"created_at": "2025-01-01T00:00:00Z"`           | ✅ `created_at: string` | ✅ `created_at: string` | ⚠️ `createdAt: Date` | ⚠️ Case inconsistency |
| `updated_at: DateTime<Utc>`              | `"updated_at": "2025-01-01T00:00:00Z"`           | ✅ `updated_at: string` | ✅ `updated_at: string` | ❌ Missing | ⚠️ Missing in Zod |
| `completed_at: Option<DateTime<Utc>>`    | `"completed_at": "2025-01-01T00:00:00Z" \| null` | ✅ `completed_at?: string \| null` | ✅ `completed_at?: string \| null` | ❌ Missing | ✅ Matches (missing in Zod) |
| `metadata: Option<serde_json::Value>`    | `"metadata": {...} \| null`                      | ✅ `metadata?: Record<string, unknown>` | ❌ Missing | ❌ Missing | ⚠️ Missing in ProjectTask |

**Field Coverage**:
- Backend Total: **17 fields**
- Global Task: **11/17** (65%) - Missing 6 fields
- ProjectTask: **9/17** (53%) - Missing 8 fields
- Zod Schema: **6/17** (35%) - Missing 11 fields

### Frontend Invented Fields (Not in Backend)

| Frontend Field | Location | Notes |
|----------------|----------|-------|
| `type?: string` | Global Task interface | ❌ **Should be removed** |
| `started_at?: string` | Global Task interface | ❌ **Should be removed** |
| `assignee?: string` | Multiple (name string) | ⚠️ **Should derive from `assigned_worker_id` via worker lookup** |
| `commentCount?: number` | TasksTab interface | ✅ Valid (separate table, not Task field) |
| `subtasks: Subtask[]` | PhaseManager Task | ⚠️ Derived from `metadata`, should extract from metadata |
| `contextChips: ContextChip[]` | PhaseManager Task | ⚠️ Derived from `context` JSONB, should extract from context |

### Project Fields

| Backend (Rust)                          | JSON API                               | Frontend (TypeScript)                                       | Notes                 |
| --------------------------------------- | -------------------------------------- | ----------------------------------------------------------- | --------------------- |
| `id: Uuid`                              | `"id": "uuid-string"`                  | `id: string`                                                | ✅ Matches            |
| `title: String`                         | `"title": "string"`                    | `name: string`                                              | ⚠️ Name mapped in API |
| `overview: Option<String>`              | `"overview": "string" \| null`         | `summary?: string \| null` / `description?: string \| null` | ⚠️ Unclear mapping    |
| `state: Option<String>`                 | `"state": "string" \| null`            | `state?: string \| null`                                    | ✅ Matches            |
| `milestones: Option<serde_json::Value>` | `"milestones": [...]`                  | `milestones?: ProjectMilestone[]`                           | ✅ Extracted          |
| `created_at: DateTime<Utc>`             | `"created_at": "2025-01-01T00:00:00Z"` | `created_at: string`                                        | ✅ Matches            |
| `updated_at: DateTime<Utc>`             | `"updated_at": "2025-01-01T00:00:00Z"` | `updated_at?: string \| null`                               | ✅ Matches            |

---

## Schema Generation Recommendations

### Option 1: Generate TypeScript from Rust (Recommended)

Use `schemars` to generate JSON Schema from Rust, then generate TypeScript types:

```bash
# Generate JSON Schema from Rust
cargo run --bin generate_schema > schemas/backend-api.schema.json

# Generate TypeScript types from JSON Schema
npm run generate-types -- schemas/backend-api.schema.json --output src/lib/schemas/generated/
```

### Option 2: Shared Schema Definition

Define schemas in a language-agnostic format (JSON Schema), then generate both Rust and TypeScript:

```yaml
# schemas/task.schema.yaml
Task:
  properties:
    id:
      type: string
      format: uuid
    title:
      type: string
    description:
      type: string
    # ... etc
```

Generate from YAML:

- Rust: Use `schemars` or `openapi-generator`
- TypeScript: Use `json-schema-to-typescript` or `quicktype`

### Option 3: Manual Alignment Documentation

Maintain explicit field mapping tables (current approach, but needs maintenance).

---

## Testing Schema Alignment

### Backend Schema Validation

```rust
// Use JsonSchema to validate API responses
use schemars::JsonSchema;

#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskResponse {
    // ... fields
}

// Validate response matches schema
let schema = TaskResponse::json_schema(&gen);
```

### Frontend Schema Validation

```typescript
// Use Zod schemas to validate API responses
import { z } from "zod";

export const TaskResponseSchema = z.object({
  id: z.string().uuid(),
  title: z.string(),
  // ... etc
});

// Validate API response
const task = TaskResponseSchema.parse(apiResponse);
```

### Integration Testing

1. **Test API Response Format**:

   - Send request from frontend
   - Verify response matches backend schema
   - Verify response matches frontend schema

2. **Test Field Mapping**:
   - Verify snake_case ↔ camelCase conversions
   - Verify UUID ↔ string conversions
   - Verify DateTime ↔ string conversions

---

## Next Steps

1. **Create unified Task schema** that includes all fields from backend
2. **Fix field name inconsistencies** (`worker_id` vs `assigned_worker_id`, `task_id` vs `id`)
3. **Fix type mismatches** (`priority` type, `description` nullability)
4. **Add missing fields** to frontend types
5. **Generate TypeScript types from Rust schemas** (automated alignment)
6. **Add runtime validation** using Zod schemas in frontend
7. **Add CI/CD checks** to detect schema drift

---

---

## Assignment Tracking Analysis

### How Assignees Are Tracked

The backend has **two levels of assignment tracking** with different granularity:

#### 1. **Task-Level Assignment** (Simple)

**Backend** (`tasks` table):

```sql
assigned_worker_id UUID REFERENCES workers(id) ON DELETE SET NULL
```

- **Simple field** on tasks table
- Just tracks **current assigned worker** (UUID reference)
- **No history tracking** at task level
- Updated via `UpdateTask.assigned_worker_id`

**Frontend**:

- Uses `assigned_worker_id?: string` (ProjectTask) ✅
- Uses `worker_id?: string` (Global Task) ⚠️ **Inconsistent**
- Also uses `assignee?: string` (ProjectTaskSchema) ⚠️ **Inconsistent** - stores worker name, not ID
- **No assignment history** in frontend

**Limitations**:

- No timestamp of when task was assigned
- No tracking of who assigned the task
- No history of reassignments
- No assignment status (just assigned or not)

#### 2. **Milestone-Level Assignment** (Rich)

**Backend** (`worker_assignments` table):

```sql
CREATE TABLE worker_assignments (
    id UUID PRIMARY KEY,
    worker_id UUID NOT NULL,
    milestone_id VARCHAR(255) NOT NULL,
    plan_id UUID,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(50) CHECK (status IN ('Assigned', 'Active', 'Completed', 'Failed', 'Cancelled', 'Reassigned')),
    priority VARCHAR(50) CHECK (priority IN ('Low', 'Normal', 'High', 'Critical')),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    failure_reason TEXT,
    cpu_cores INTEGER,
    memory_mb INTEGER,
    disk_mb INTEGER,
    network_mbps FLOAT,
    time_limit_ms BIGINT,
    metadata JSONB,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);
```

**Assignment History** (`assignment_history` table):

```sql
CREATE TABLE assignment_history (
    id UUID PRIMARY KEY,
    assignment_id UUID REFERENCES worker_assignments(id),
    worker_id UUID NOT NULL,
    milestone_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(50) CHECK (event_type IN ('assigned', 'started', 'completed', 'failed', 'cancelled', 'reassigned', 'status_changed')),
    old_status VARCHAR(50),
    new_status VARCHAR(50),
    event_description TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ
);
```

**Features**:

- ✅ **Full history tracking** via `assignment_history` table
- ✅ **Timestamp tracking**: `assigned_at`, `started_at`, `completed_at`, `failed_at`
- ✅ **Status tracking**: Assignment has its own status separate from task status
- ✅ **Resource requirements**: CPU, memory, disk, network allocation
- ✅ **Event tracking**: Every status change logged with old/new status
- ✅ **Helper functions**: `update_assignment_status()` automatically creates history entries
- ✅ **Statistics view**: `assignment_statistics` view for worker performance

**Frontend**:

- ❌ **No API integration** for `worker_assignments` table
- ❌ **No milestone assignment tracking** in frontend
- ❌ **No assignment history** in frontend

**Note**: `worker_assignments` is for **milestone** assignments (plan-level), not **task** assignments!

### Assignment Tracking Mismatch

| Aspect               | Task Assignment                         | Milestone Assignment                                        |
| -------------------- | --------------------------------------- | ----------------------------------------------------------- |
| **Backend Table**    | `tasks.assigned_worker_id`              | `worker_assignments`                                        |
| **History Tracking** | ❌ None                                 | ✅ Full history (`assignment_history`)                      |
| **Timestamps**       | ❌ None (only `updated_at`)             | ✅ `assigned_at`, `started_at`, `completed_at`, `failed_at` |
| **Status Tracking**  | Uses task status                        | Has own assignment status                                   |
| **Frontend Support** | ⚠️ Partial (inconsistent field names)   | ❌ None                                                     |
| **API Endpoints**    | ✅ Update via `PATCH /api/v1/tasks/:id` | ❌ Not exposed                                              |
| **Use Case**         | Simple task-to-worker assignment        | Complex milestone resource allocation                       |

### Issues Identified

1. **Task Assignment Limitations**:

   - No history of who assigned the task
   - No timestamp of when task was assigned
   - No reassignment tracking
   - Cannot see assignment timeline

2. **Milestone Assignment Not Exposed**:

   - Rich assignment tracking exists but no API endpoints
   - Frontend cannot access assignment history
   - Frontend cannot track milestone-level assignments

3. **Frontend Inconsistencies**:

   - `assigned_worker_id` (UUID) vs `worker_id` (UUID) vs `assignee` (name string)
   - No clear pattern for which to use

### Recommendations

#### Option A: Enhance Task Assignment (Recommended)

Add assignment history to task level (similar to milestone level):

1. **Add `task_assignment_history` table**:

   ```sql
   CREATE TABLE task_assignment_history (
       id UUID PRIMARY KEY,
       task_id UUID REFERENCES tasks(id),
       worker_id UUID,
       assigned_by UUID,  -- Who made the assignment
       assigned_at TIMESTAMPTZ,
       unassigned_at TIMESTAMPTZ,
       event_type VARCHAR(50),
       metadata JSONB
   );
   ```

2. **Add assignment timestamps to tasks**:

   ```sql
   ALTER TABLE tasks ADD COLUMN assigned_at TIMESTAMPTZ;
   ALTER TABLE tasks ADD COLUMN assigned_by UUID;
   ```

3. **Create API endpoints**:

   - `GET /api/v1/tasks/:task_id/assignment-history` - Get assignment history
   - `POST /api/v1/tasks/:task_id/assign` - Assign with history tracking

#### Option B: Expose Milestone Assignments

Add API endpoints for milestone-level assignments:

1. **Create API endpoints**:

   - `GET /api/v1/projects/:project_id/milestones/:milestone_id/assignments`
   - `GET /api/v1/projects/:project_id/milestones/:milestone_id/assignment-history`
   - `POST /api/v1/projects/:project_id/milestones/:milestone_id/assign`

#### Option C: Link Tasks to Milestone Assignments

If tasks belong to milestones, link task assignments to milestone assignments:

1. **Add `milestone_id` to tasks** (if not already present via project hierarchy)
2. **Query milestone assignment** when fetching task
3. **Inherit assignment context** from milestone

### Current Assignment Tracking Summary

**For Tasks** (Simple):

- Backend: `tasks.assigned_worker_id` (current assignment only)
- Frontend: `assigned_worker_id` or `worker_id` (inconsistent)
- **Missing**: History, timestamps, who assigned

**For Milestones** (Rich):

- Backend: `worker_assignments` table with full history
- Frontend: ❌ Not exposed
- **Missing**: API endpoints, frontend integration

**Impact on Jira-like Workflow**:

- Agents can see **current assignment** ✅
- Agents **cannot see** assignment history ❌
- Agents **cannot see** when task was assigned ❌
- Agents **cannot see** who assigned the task ❌
- Agents **cannot see** reassignment timeline ❌

---

**Last Updated**: 2025-11-15  
**Status**: 🚨 **CRITICAL ISSUES IDENTIFIED** - Status workflow mismatch blocks agent workflow  
**Next Review**: After Phase 1 critical fixes completed
