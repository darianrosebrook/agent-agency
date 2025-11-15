# Frontend-Backend Schema Divergence Catalog

**Date**: 2025-01-28  
**Status**: In Progress  
**Author**: @darianrosebrook

## Purpose

This document provides a comprehensive catalog of all divergences between backend Rust data models and frontend TypeScript types, specifically focusing on Task and Project schemas. It identifies:
- Fields that exist in backend but not frontend
- Fields that exist in frontend but not backend
- Field name inconsistencies
- Type mismatches
- Status enum value mismatches
- Update capability gaps

---

## Part 1: Backend Task Schema Catalog

### Complete Backend Task Model

**Location**: `iterations/v3/data-infrastructure/src/models.rs`  
**Rust Struct**: `Task`  
**Database Table**: `tasks`

#### Field Catalog (17 Fields)

| Field Name | Rust Type | Database Type | Nullable | Constraints | Notes |
|------------|-----------|---------------|----------|-------------|-------|
| `id` | `Uuid` | `UUID` | No | PRIMARY KEY | ✅ Matches frontend (as string) |
| `title` | `String` | `VARCHAR(500)` | No | NOT NULL | ✅ Matches frontend |
| `description` | `String` | `TEXT` | No | NOT NULL | ⚠️ Required in backend, optional in frontend |
| `risk_tier` | `String` | `VARCHAR(50)` | No | NOT NULL, DEFAULT '2' | CHECK ('1', '2', '3') |
| `scope` | `serde_json::Value` | `JSONB` | No | NOT NULL, DEFAULT '{}' | ❌ Missing in frontend |
| `acceptance_criteria` | `serde_json::Value` | `JSONB` | No | NOT NULL, DEFAULT '[]' | ❌ Missing in frontend |
| `context` | `serde_json::Value` | `JSONB` | No | NOT NULL, DEFAULT '{}' | ❌ Missing in frontend |
| `caws_spec` | `Option<serde_json::Value>` | `JSONB` | Yes | NULL allowed | ❌ Missing in frontend |
| `status` | `String` | `VARCHAR(50)` | No | NOT NULL, DEFAULT 'pending' | CHECK ('pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed') |
| `assigned_worker_id` | `Option<Uuid>` | `UUID` | Yes | REFERENCES workers(id) | ⚠️ Name inconsistent in frontend |
| `project_id` | `Option<Uuid>` | `UUID` | Yes | REFERENCES execution_plans(id) | ❌ Missing in global Task interface |
| `priority` | `Option<i32>` | `INTEGER` | Yes | CHECK (>= 0 AND <= 10) | ⚠️ Type mismatch (string vs number) |
| `deadline` | `Option<DateTime<Utc>>` | `TIMESTAMP WITH TIME ZONE` | Yes | NULL allowed | ❌ Missing in frontend |
| `created_at` | `DateTime<Utc>` | `TIMESTAMP WITH TIME ZONE` | No | NOT NULL, DEFAULT NOW() | ✅ Matches frontend (as string) |
| `updated_at` | `DateTime<Utc>` | `TIMESTAMP WITH TIME ZONE` | No | NOT NULL, DEFAULT NOW() | ✅ Matches frontend (as string) |
| `completed_at` | `Option<DateTime<Utc>>` | `TIMESTAMP WITH TIME ZONE` | Yes | NULL allowed | ✅ Matches frontend (as string) |
| `metadata` | `Option<serde_json::Value>` | `JSONB` | Yes | NULL allowed | ✅ Matches frontend |

### Backend UpdateTask Capabilities

**Location**: `iterations/v3/data-infrastructure/src/database_operations.rs`  
**Rust Struct**: `UpdateTask`

All fields are `Option<T>`, allowing partial updates:

| Field | Type | Updateable | Notes |
|-------|------|------------|-------|
| `title` | `Option<String>` | ✅ | ✅ Frontend supports |
| `description` | `Option<String>` | ✅ | ✅ Frontend supports |
| `risk_tier` | `Option<String>` | ✅ | ❌ Frontend cannot update |
| `scope` | `Option<serde_json::Value>` | ✅ | ❌ Frontend cannot update |
| `acceptance_criteria` | `Option<serde_json::Value>` | ✅ | ❌ Frontend cannot update |
| `context` | `Option<serde_json::Value>` | ✅ | ❌ Frontend cannot update |
| `caws_spec` | `Option<serde_json::Value>` | ✅ | ❌ Frontend cannot update |
| `status` | `Option<String>` | ✅ | ⚠️ Frontend supports but enum mismatch |
| `assigned_worker_id` | `Option<Uuid>` | ✅ | ✅ Frontend supports |
| `project_id` | `Option<Uuid>` | ✅ | ❌ Frontend cannot update |
| `priority` | `Option<i32>` | ✅ | ✅ Frontend supports (type mismatch) |
| `deadline` | `Option<DateTime<Utc>>` | ✅ | ❌ Frontend cannot update |
| `metadata` | `Option<serde_json::Value>` | ✅ | ❌ Frontend cannot update |
| `completed_at` | `Option<DateTime<Utc>>` | ✅ | ❌ Frontend cannot update |

**Total Updateable Fields**: 14 (all fields except `id`, `created_at`, `updated_at`)

### Backend Status Enum Values

**Source**: Database constraint in `014_create_agent_management_tables.sql`

```sql
status VARCHAR(50) CHECK (status IN ('pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed'))
```

**Allowed Values**:
- `pending` - Task is waiting to be assigned/started
- `in_progress` - Task is actively being worked on
- `paused` - Task is paused (can be resumed)
- `completed` - Task is finished successfully
- `cancelled` - Task was cancelled
- `failed` - Task failed during execution

---

## Part 2: Frontend Task Schema Catalog

### Frontend Task Interface Variants

The frontend has **7+ different Task interface definitions** across different files:

#### 2.1 Global Task Interface

**Location**: `apps/agent_management_dashboard/src/lib/api/tasks.ts`  
**Purpose**: Global task listing (all projects)

```typescript
export interface Task {
  id: string;                    // ✅ Matches backend
  title: string;                 // ✅ Matches backend
  description?: string;          // ⚠️ Optional (backend: required)
  priority?: string;             // ⚠️ Type mismatch (backend: i32)
  type?: string;                 // ❌ NOT in backend
  status: string;                // ⚠️ Enum mismatch
  created_at: string;            // ✅ Matches backend
  updated_at: string;            // ✅ Matches backend
  started_at?: string;           // ❌ NOT in backend
  completed_at?: string | null;  // ✅ Matches backend
  worker_id?: string | null;     // ⚠️ Name mismatch (backend: assigned_worker_id)
  metadata?: Record<string, unknown>; // ✅ Matches backend
}
```

**Fields**: 11 fields  
**Missing Backend Fields**: risk_tier, scope, acceptance_criteria, context, caws_spec, project_id, deadline  
**Invented Fields**: type, started_at  
**Name Inconsistencies**: worker_id (should be assigned_worker_id)

#### 2.2 ProjectTask Interface

**Location**: `apps/agent_management_dashboard/src/lib/api/projects.ts`  
**Purpose**: Project-specific tasks

```typescript
export interface ProjectTask {
  task_id: string;               // ⚠️ Name mismatch (backend: id)
  title: string;                 // ✅ Matches backend
  description?: string | null;   // ⚠️ Optional (backend: required)
  status: string;                // ⚠️ Enum mismatch
  risk_tier?: string | null;     // ✅ Present (better than Global Task)
  priority?: number | null;      // ✅ Type matches (backend: i32)
  assigned_worker_id?: string | null; // ✅ Name matches!
  created_at: string;            // ✅ Matches backend
  updated_at: string;            // ✅ Matches backend
  completed_at?: string | null;  // ✅ Matches backend
}
```

**Fields**: 9 fields  
**Missing Backend Fields**: scope, acceptance_criteria, context, caws_spec, project_id, deadline  
**Name Inconsistencies**: task_id (should be id)

#### 2.3 ProjectTaskSchema (Zod)

**Location**: `apps/agent_management_dashboard/src/lib/schemas/project.ts`  
**Purpose**: Runtime validation for project tasks

```typescript
export const ProjectTaskSchema = z.object({
  id: z.string(),
  title: z.string(),
  description: z.string().optional(),
  status: z.enum(['backlog', 'todo', 'in-progress', 'done']), // 🚨 CRITICAL: Enum mismatch
  priority: z.string().optional(), // ⚠️ Type mismatch (backend: number/i32)
  assignee: z.string().optional(), // ⚠️ Name mismatch (backend: assigned_worker_id UUID)
  createdAt: z.date().or(z.string().transform((str) => new Date(str))),
});
```

**Fields**: 6 fields  
**Status Enum**: `['backlog', 'todo', 'in-progress', 'done']` - **Does NOT match backend**  
**Name Inconsistencies**: assignee (should be assigned_worker_id), createdAt (should be created_at)

#### 2.4 ProjectContext Task Interface

**Location**: `apps/agent_management_dashboard/src/components/projects/ProjectContext.tsx`  
**Purpose**: Component-specific task type

```typescript
export interface Task {
  id: string;
  title: string;
  description?: string;
  status: "backlog" | "todo" | "in-progress" | "done"; // 🚨 CRITICAL: Enum mismatch
  priority?: string; // ⚠️ Type mismatch
  assignee?: string; // ⚠️ Name mismatch
  createdAt: Date;
}
```

**Fields**: 7 fields  
**Status Enum**: Same as Zod schema - **Does NOT match backend**

#### 2.5 TasksTab Task Interface

**Location**: `apps/agent_management_dashboard/src/components/projects/TasksTab.tsx`  
**Purpose**: Kanban board task display

```typescript
interface Task {
  id: string;
  title: string;
  description?: string;
  status: KanbanStatus; // Type: "backlog" | "todo" | "in-progress" | "done"
  priority?: "low" | "medium" | "high"; // ⚠️ String enum, not number
  assigned_worker_id?: string | null; // ✅ Name matches!
  commentCount?: number; // ✅ Valid (separate table, not Task field)
}
```

**Fields**: 6 fields (+ commentCount from separate query)  
**Priority**: Uses string enum ("low", "medium", "high") instead of number 0-10

#### 2.6 PhaseManager Task Interface

**Location**: `apps/agent_management_dashboard/src/components/projects/phase-manager/types.ts`  
**Purpose**: Phase planning task type

```typescript
export interface Task {
  id: string;
  title: string;
  description: string; // Required here
  subtasks: Subtask[]; // ❌ Extracted from metadata, not backend field
  contextChips: ContextChip[]; // ❌ Extracted from context JSONB, not backend field
}
```

**Fields**: 5 fields  
**Note**: Missing standard Task fields (status, priority, etc.) - focused on planning view

#### 2.7 TimelineTask Interface

**Location**: `apps/agent_management_dashboard/src/components/composers/TimelineTab.tsx`  
**Purpose**: Timeline/Gantt chart task display

```typescript
export interface TimelineTask {
  id: string;
  title: string;
  worker: string; // Worker name (derived from assigned_worker_id)
  workerId: string; // ✅ assigned_worker_id
  startDate: Date;
  endDate: Date;
  status: "completed" | "in-progress" | "pending"; // ⚠️ Simplified enum
  tags: string[]; // ❌ Derived from metadata/type, not backend field
  description?: string;
}
```

**Fields**: 8 fields  
**Status Enum**: Simplified 3-value enum - **Does NOT match backend**

---

## Part 3: Frontend Update Capabilities Catalog

### 3.1 updateProjectTask Function

**Location**: `apps/agent_management_dashboard/src/lib/api/projects.ts`

```typescript
export async function updateProjectTask(
  projectId: string,
  taskId: string,
  updates: Partial<
    Pick<
      ProjectTask,
      "title" | "description" | "status" | "priority" | "assigned_worker_id"
    >
  >
): Promise<ProjectTask>
```

**Updateable Fields**: 5 fields
- ✅ `title`
- ✅ `description`
- ⚠️ `status` (enum mismatch)
- ✅ `priority` (type mismatch: number vs i32)
- ✅ `assigned_worker_id`

**Backend UpdateTask Supports**: 14 fields  
**Missing Update Capabilities**: 9 fields
- ❌ `risk_tier`
- ❌ `scope`
- ❌ `acceptance_criteria`
- ❌ `context` (CRITICAL for agent workflow)
- ❌ `caws_spec`
- ❌ `deadline`
- ❌ `metadata`
- ❌ `completed_at`
- ❌ `project_id`

### 3.2 UpdateTaskRequestSchema (Zod)

**Location**: `apps/agent_management_dashboard/src/lib/schemas/project.ts`

```typescript
export const UpdateTaskRequestSchema = z.object({
  title: z.string().optional(),
  description: z.string().optional(),
  status: z.enum(['backlog', 'todo', 'in-progress', 'done']).optional(), // 🚨 Enum mismatch
  priority: z.string().optional(), // ⚠️ Type mismatch
  assignee: z.string().optional(), // ⚠️ Name mismatch
});
```

**Updateable Fields**: 4 fields  
**Missing**: All JSONB fields, deadline, metadata, completed_at, risk_tier

---

## Part 4: Complete Divergence Analysis

### 4.1 Fields Backend Has, Frontend Missing

| Backend Field | Global Task | ProjectTask | Zod Schema | Status |
|---------------|-------------|-------------|------------|--------|
| `risk_tier` | ❌ | ✅ | ❌ | Partially missing |
| `scope` | ❌ | ❌ | ❌ | **Missing everywhere** |
| `acceptance_criteria` | ❌ | ❌ | ❌ | **Missing everywhere** |
| `context` | ❌ | ❌ | ❌ | **Missing everywhere (CRITICAL)** |
| `caws_spec` | ❌ | ❌ | ❌ | **Missing everywhere** |
| `deadline` | ❌ | ❌ | ❌ | **Missing everywhere** |
| `project_id` | ❌ | ✅ (inferred from URL) | ❌ | Missing in global Task |

### 4.2 Fields Frontend Has, Backend Doesn't

| Frontend Field | Location | Backend Equivalent | Issue |
|----------------|----------|-------------------|-------|
| `type` | Global Task | None | ❌ **Invented field** |
| `started_at` | Global Task | None | ❌ **Invented field** |
| `assignee` (name string) | Multiple | `assigned_worker_id` (UUID) | ⚠️ **Should derive from worker lookup** |
| `commentCount` | TasksTab | Separate `task_comments` table | ✅ Valid (not Task field) |
| `subtasks` | PhaseManager | Extracted from `metadata` | ⚠️ **Derived field, not direct** |
| `contextChips` | PhaseManager | Extracted from `context` JSONB | ⚠️ **Derived field, not direct** |

### 4.3 Field Name Inconsistencies

| Backend Field | Frontend Variants | Issue |
|---------------|-------------------|-------|
| `id` | `id` (Global Task) / `task_id` (ProjectTask) | ⚠️ **Inconsistent** |
| `assigned_worker_id` | `assigned_worker_id` (ProjectTask) / `worker_id` (Global Task) / `assignee` (Zod) | ⚠️ **Three different names** |
| `created_at` | `created_at` / `createdAt` (camelCase) | ⚠️ **Case inconsistency** |
| `updated_at` | `updated_at` / `updatedAt` (camelCase) | ⚠️ **Case inconsistency** |

### 4.4 Type Mismatches

| Field | Backend Type | Frontend Types | Issue |
|-------|--------------|----------------|-------|
| `priority` | `Option<i32>` (0-10) | `string` (Global Task) / `number` (ProjectTask) / `"low"\|"medium"\|"high"` (TasksTab) | ⚠️ **Three different types** |
| `description` | `String` (required) | `string?` (optional) | ⚠️ **Nullability mismatch** |
| `status` | `String` (enum with 6 values) | Multiple enums with 4 values | 🚨 **CRITICAL: Enum mismatch** |

### 4.5 Status Enum Divergence Details

| Backend Value | Frontend Values | Mapping |
|---------------|-----------------|---------|
| `pending` | `backlog`, `todo` | ❌ **No direct mapping** |
| `in_progress` | `in-progress` | ⚠️ **Underscore vs hyphen** |
| `paused` | None | ❌ **Not supported in frontend** |
| `completed` | `done` | ❌ **Different name** |
| `cancelled` | None | ❌ **Not supported in frontend** |
| `failed` | None | ❌ **Not supported in frontend** |

**Impact**: Frontend cannot properly display or transition through backend status values.

---

## Part 5: Update Capability Matrix

| Backend UpdateTask Field | updateProjectTask | UpdateTaskRequestSchema | Backend Support |
|--------------------------|-------------------|------------------------|-----------------|
| `title` | ✅ | ✅ | ✅ |
| `description` | ✅ | ✅ | ✅ |
| `risk_tier` | ❌ | ❌ | ✅ |
| `scope` | ❌ | ❌ | ✅ |
| `acceptance_criteria` | ❌ | ❌ | ✅ |
| `context` | ❌ | ❌ | ✅ |
| `caws_spec` | ❌ | ❌ | ✅ |
| `status` | ⚠️ (enum mismatch) | ⚠️ (enum mismatch) | ✅ |
| `assigned_worker_id` | ✅ | ⚠️ (as `assignee`) | ✅ |
| `project_id` | ❌ | ❌ | ✅ |
| `priority` | ⚠️ (type mismatch) | ⚠️ (type mismatch) | ✅ |
| `deadline` | ❌ | ❌ | ✅ |
| `metadata` | ❌ | ❌ | ✅ |
| `completed_at` | ❌ | ❌ | ✅ |

**Summary**:
- Backend supports: **14 fields**
- Frontend `updateProjectTask` supports: **5 fields** (36%)
- Frontend `UpdateTaskRequestSchema` supports: **4 fields** (29%)
- **Missing critical fields**: context, acceptance_criteria, deadline, metadata

---

## Part 6: Summary Statistics

### Coverage Metrics

- **Backend Fields**: 17 total
- **Frontend Coverage (Global Task)**: 11/17 (65%)
- **Frontend Coverage (ProjectTask)**: 9/17 (53%)
- **Frontend Coverage (Zod Schema)**: 6/17 (35%)

### Update Capability Metrics

- **Backend Updateable Fields**: 14
- **Frontend Updateable Fields**: 5 (36%)
- **Missing Critical Updates**: 9 fields including context, deadline, metadata

### Status Enum Coverage

- **Backend Status Values**: 6
- **Frontend Status Values**: 4 (different set)
- **Matching Values**: 1 (`in_progress` / `in-progress`)
- **Missing in Frontend**: `paused`, `cancelled`, `failed`

---

**Last Updated**: 2025-01-28  
**Next Steps**: See SCHEMA_ALIGNMENT.md for prioritized realignment plan
