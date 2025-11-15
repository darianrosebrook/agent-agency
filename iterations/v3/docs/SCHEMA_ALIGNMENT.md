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
  id: string;                    // ✅ Matches
  title: string;                 // ✅ Matches
  description?: string;          // ✅ Matches (optional in frontend, required in backend)
  priority?: string;             // ⚠️ Type mismatch (string vs i32)
  type?: string;                 // ❌ Not in backend
  status: string;                // ✅ Matches
  created_at: string;            // ✅ Matches (snake_case for API)
  updated_at: string;            // ✅ Matches (snake_case for API)
  started_at?: string;           // ❌ Not in backend
  completed_at?: string | null;  // ✅ Matches
  worker_id?: string | null;     // ⚠️ Named differently (assigned_worker_id in backend)
  metadata?: Record<string, unknown>;  // ✅ Matches
}
```

**Project Task Interface** (`apps/agent_management_dashboard/src/lib/api/projects.ts`):
```typescript
export interface ProjectTask {
  task_id: string;               // ⚠️ Named differently (id in backend)
  title: string;                 // ✅ Matches
  description?: string | null;   // ✅ Matches
  status: string;                // ✅ Matches
  risk_tier?: string | null;     // ✅ Matches (present!)
  priority?: number | null;      // ✅ Matches type (number vs i32)
  assigned_worker_id?: string | null;  // ✅ Matches name!
  created_at: string;            // ✅ Matches
  updated_at: string;            // ✅ Matches
  completed_at?: string | null;  // ✅ Matches
  // ❌ Missing: project_id (but makes sense - already in URL path)
  // ❌ Missing: scope, acceptance_criteria, context, caws_spec
  // ❌ Missing: deadline
}
```

### Issues Identified

#### 1. **Field Name Inconsistencies**

| Backend Field | Frontend Global Task | Frontend ProjectTask | Status |
|---------------|---------------------|---------------------|---------|
| `id` | ✅ `id` | ⚠️ `task_id` | **Inconsistent** |
| `assigned_worker_id` | ⚠️ `worker_id` | ✅ `assigned_worker_id` | **Inconsistent** |
| `priority` | ⚠️ `string` | ✅ `number` | **Type mismatch** |

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

| Field | Backend Type | Frontend Type | Issue |
|-------|-------------|---------------|-------|
| `priority` | `Option<i32>` | `string` (Global Task) / `number` (ProjectTask) | **Inconsistent types** |
| `description` | `String` (required) | `string | null` (optional) | **Nullability mismatch** |

#### 4. **🚨 CRITICAL: Status Workflow Mismatch**

**Backend Task Status** (database constraint):
```sql
status VARCHAR(50) CHECK (status IN ('pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed'))
```

**Frontend Task Status** (Zod enum):
```typescript
status: z.enum(['backlog', 'todo', 'in-progress', 'done'])
```

**Impact**: **Status values don't match!** This will break the Jira-like workflow where agents update task status.

**Backend supports**:
- `pending` - Task is waiting to be assigned/started
- `in_progress` - Task is actively being worked on
- `paused` - Task is paused (can be resumed)
- `completed` - Task is finished successfully
- `cancelled` - Task was cancelled
- `failed` - Task failed during execution

**Frontend expects**:
- `backlog` - Task is in backlog (not in backend!)
- `todo` - Task is to do (not in backend!)
- `in-progress` - Matches backend `in_progress` (with hyphen vs underscore)
- `done` - Should map to backend `completed`

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
>
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
  name: string;                     // Maps from backend "title"
  summary?: string | null;          // Maps from backend "overview"
  description?: string | null;      // Also maps from backend "overview"?
  state?: string | null;            // Maps from backend "state"
  created_at: string;
  updated_at?: string | null;
  last_accessed?: string | null;    // ❌ Not in backend
  milestones?: ProjectMilestone[];  // ✅ Extracted from JSON
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
     status: z.enum(['pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed'])
     ```
   - **Option B**: Update backend to support frontend statuses (not recommended - breaks existing data)
   - **Recommended**: Option A - Frontend should match backend database constraints
   - Add status mapping functions if needed for UI display

2. **Expand updateProjectTask Capabilities** (BLOCKING):
   - Add all missing fields to `updateProjectTask`:
     ```typescript
     updates: Partial<Pick<
       ProjectTask,
       "title" | "description" | "status" | "priority" | "assigned_worker_id" |
       "risk_tier" | "scope" | "acceptance_criteria" | "context" | "caws_spec" |
       "deadline" | "metadata" | "completed_at"
     >>
     ```
   - **Impact**: Enables full Jira-like workflow for agents

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

## Field Mapping Reference

### Task Fields

| Backend (Rust) | JSON API | Frontend (TypeScript) | Notes |
|----------------|----------|----------------------|-------|
| `id: Uuid` | `"id": "uuid-string"` | `id: string` | ✅ Matches |
| `title: String` | `"title": "string"` | `title: string` | ✅ Matches |
| `description: String` | `"description": "string"` | `description?: string \| null` | ⚠️ Nullability mismatch |
| `risk_tier: String` | `"risk_tier": "string"` | `risk_tier?: string` (ProjectTask only) | ⚠️ Missing in Global Task |
| `status: String` | `"status": "string"` | `status: string` | ✅ Matches |
| `assigned_worker_id: Option<Uuid>` | `"assigned_worker_id": "uuid-string" \| null` | `assigned_worker_id?: string` (ProjectTask) / `worker_id?: string` (Global Task) | ⚠️ Name inconsistency |
| `project_id: Option<Uuid>` | `"project_id": "uuid-string" \| null` | Missing in Global Task | ❌ Missing |
| `priority: Option<i32>` | `"priority": 1 \| null` | `priority?: string` (Global Task) / `priority?: number` (ProjectTask) | ⚠️ Type mismatch |
| `scope: serde_json::Value` | `"scope": {...}` | Missing | ❌ Missing |
| `acceptance_criteria: serde_json::Value` | `"acceptance_criteria": {...}` | Missing | ❌ Missing |
| `context: serde_json::Value` | `"context": {...}` | Missing | ❌ Missing |
| `caws_spec: Option<serde_json::Value>` | `"caws_spec": {...} \| null` | Missing | ❌ Missing |
| `deadline: Option<DateTime<Utc>>` | `"deadline": "2025-12-31T00:00:00Z" \| null` | Missing | ❌ Missing |
| `created_at: DateTime<Utc>` | `"created_at": "2025-01-01T00:00:00Z"` | `created_at: string` | ✅ Matches |
| `updated_at: DateTime<Utc>` | `"updated_at": "2025-01-01T00:00:00Z"` | `updated_at: string` | ✅ Matches |
| `completed_at: Option<DateTime<Utc>>` | `"completed_at": "2025-01-01T00:00:00Z" \| null` | `completed_at?: string \| null` | ✅ Matches |
| `metadata: Option<serde_json::Value>` | `"metadata": {...} \| null` | `metadata?: Record<string, unknown>` | ✅ Matches |

### Project Fields

| Backend (Rust) | JSON API | Frontend (TypeScript) | Notes |
|----------------|----------|----------------------|-------|
| `id: Uuid` | `"id": "uuid-string"` | `id: string` | ✅ Matches |
| `title: String` | `"title": "string"` | `name: string` | ⚠️ Name mapped in API |
| `overview: Option<String>` | `"overview": "string" \| null` | `summary?: string \| null` / `description?: string \| null` | ⚠️ Unclear mapping |
| `state: Option<String>` | `"state": "string" \| null` | `state?: string \| null` | ✅ Matches |
| `milestones: Option<serde_json::Value>` | `"milestones": [...]` | `milestones?: ProjectMilestone[]` | ✅ Extracted |
| `created_at: DateTime<Utc>` | `"created_at": "2025-01-01T00:00:00Z"` | `created_at: string` | ✅ Matches |
| `updated_at: DateTime<Utc>` | `"updated_at": "2025-01-01T00:00:00Z"` | `updated_at?: string \| null` | ✅ Matches |

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
import { z } from 'zod';

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

**Last Updated**: 2025-11-15  
**Next Review**: After schema alignment fixes completed

