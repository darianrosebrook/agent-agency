# Extended Schema Alignment - Worker/Agent and Chat

**Date**: 2025-01-28  
**Status**: Documentation in Progress  
**Author**: @darianrosebrook

## Purpose

This document extends `SCHEMA_ALIGNMENT.md` to cover Worker/Agent and Chat session schemas. It identifies divergences between backend Rust models and frontend TypeScript types for these critical agent interaction entities.

---

## Worker/Agent Schema Alignment

### Backend Worker Model

**Location**: `iterations/v3/data-infrastructure/src/models.rs`  
**Rust Struct**: `Worker`  
**Database Table**: `workers`

#### Field Catalog (11 Fields)

| Field Name            | Rust Type           | Database Type              | Nullable | Notes                   |
| --------------------- | ------------------- | -------------------------- | -------- | ----------------------- |
| `id`                  | `Uuid`              | `UUID`                     | No       | PRIMARY KEY             |
| `name`                | `String`            | `VARCHAR(255)`             | No       | NOT NULL                |
| `worker_type`         | `String`            | `VARCHAR(100)`             | No       | NOT NULL                |
| `specialty`           | `Option<String>`    | `VARCHAR(255)`             | Yes      | NULL allowed            |
| `model_name`          | `String`            | `VARCHAR(255)`             | No       | NOT NULL                |
| `endpoint`            | `String`            | `VARCHAR(500)`             | No       | NOT NULL                |
| `capabilities`        | `serde_json::Value` | `JSONB`                    | No       | NOT NULL, DEFAULT '{}'  |
| `performance_history` | `serde_json::Value` | `JSONB`                    | No       | NOT NULL, DEFAULT '{}'  |
| `is_active`           | `bool`              | `BOOLEAN`                  | No       | NOT NULL, DEFAULT TRUE  |
| `created_at`          | `DateTime<Utc>`     | `TIMESTAMP WITH TIME ZONE` | No       | NOT NULL, DEFAULT NOW() |
| `updated_at`          | `DateTime<Utc>`     | `TIMESTAMP WITH TIME ZONE` | No       | NOT NULL, DEFAULT NOW() |

### Frontend Agent Interface

**Location**: `apps/agent_management_dashboard/src/lib/api/agents.ts`  
**Type**: `Agent`

```typescript
export interface Agent {
  id: string; // ✅ Matches backend
  name: string; // ✅ Matches backend
  worker_type: string; // ✅ Matches backend
  specialty: string | null; // ✅ Matches backend
  model_name: string | null; // ⚠️ Nullable (backend: required)
  endpoint: string | null; // ⚠️ Nullable (backend: required)
  capabilities: string[] | null; // ⚠️ Type mismatch (backend: JSONB)
  performance_history: unknown; // ✅ Matches backend (as unknown)
  is_active: boolean; // ✅ Matches backend
}
```

**Fields**: 9 fields  
**Missing Backend Fields**: `created_at`, `updated_at`  
**Type Mismatches**: `capabilities` (string[] vs JSONB), `model_name` nullability, `endpoint` nullability

### Issues Identified

#### 1. Missing Timestamp Fields

| Backend Field | Frontend Status | Impact                                     |
| ------------- | --------------- | ------------------------------------------ |
| `created_at`  | ❌ Missing      | Cannot display when agent was created      |
| `updated_at`  | ❌ Missing      | Cannot display when agent was last updated |

#### 2. Type Mismatches

| Field          | Backend Type                | Frontend Type | Issue |
| -------------- | --------------------------- | ------------- | ----- | ------------------------------------------------------------------------------------- |
| `capabilities` | `serde_json::Value` (JSONB) | `string[]     | null` | **Type mismatch** - Backend stores structured JSON, frontend expects array of strings |
| `model_name`   | `String` (required)         | `string       | null` | **Nullability mismatch**                                                              |
| `endpoint`     | `String` (required)         | `string       | null` | **Nullability mismatch**                                                              |

#### 3. Capabilities Field Structure

**Backend**: JSONB can store structured data:

```json
{
  "supported_models": ["gpt-4", "claude-3"],
  "max_context_length": 128000,
  "features": ["code_generation", "analysis"],
  "rate_limits": {
    "requests_per_minute": 100
  }
}
```

**Frontend**: Expects simple string array:

```typescript
capabilities: ["code_generation", "analysis"];
```

**Impact**: Frontend cannot access structured capability information (rate limits, context lengths, etc.)

---

## Chat Session Schema Alignment

### Backend ChatSession Model

**Location**: `iterations/v3/data-infrastructure/src/chat_service.rs`  
**Rust Struct**: `ChatSession`  
**Database Table**: `chat_sessions`

#### Field Catalog (12 Fields)

| Field Name        | Rust Type               | Database Type              | Nullable | Notes                   |
| ----------------- | ----------------------- | -------------------------- | -------- | ----------------------- |
| `id`              | `Uuid`                  | `UUID`                     | No       | PRIMARY KEY             |
| `workspace_id`    | `Option<Uuid>`          | `UUID`                     | Yes      | NULL allowed            |
| `tenant_id`       | `Option<Uuid>`          | `UUID`                     | Yes      | NULL allowed            |
| `title`           | `Option<String>`        | `VARCHAR(500)`             | Yes      | NULL allowed            |
| `created_at`      | `DateTime<Utc>`         | `TIMESTAMP WITH TIME ZONE` | No       | NOT NULL, DEFAULT NOW() |
| `updated_at`      | `DateTime<Utc>`         | `TIMESTAMP WITH TIME ZONE` | No       | NOT NULL, DEFAULT NOW() |
| `last_message_at` | `Option<DateTime<Utc>>` | `TIMESTAMP WITH TIME ZONE` | Yes      | NULL allowed            |
| `message_count`   | `i32`                   | `INTEGER`                  | No       | DEFAULT 0               |
| `metadata`        | `serde_json::Value`     | `JSONB`                    | No       | DEFAULT '{}'            |
| `archived`        | `bool`                  | `BOOLEAN`                  | No       | DEFAULT FALSE           |
| `pinned`          | `bool`                  | `BOOLEAN`                  | No       | DEFAULT FALSE           |
| `folder_id`       | `Option<Uuid>`          | `UUID`                     | Yes      | NULL allowed            |

**Note**: Database has `archived_at` field (TIMESTAMPTZ) but Rust model doesn't include it!

### Frontend ChatSessionResponse Interface

**Location**: `apps/agent_management_dashboard/src/lib/api/chat.ts`  
**Type**: `ChatSessionResponse`

```typescript
export interface ChatSessionResponse {
  id: string; // ✅ Matches backend
  workspace_id?: string; // ✅ Matches backend
  tenant_id?: string; // ✅ Matches backend
  title?: string; // ✅ Matches backend
  created_at: string; // ✅ Matches backend (as RFC3339)
  updated_at: string; // ✅ Matches backend (as RFC3339)
  last_message_at?: string; // ✅ Matches backend (as RFC3339)
  message_count: number; // ✅ Matches backend (as number)
  metadata: Record<string, unknown>; // ✅ Matches backend (JSONB)
  archived: boolean; // ✅ Matches backend
  pinned: boolean; // ✅ Matches backend
  folder_id?: string; // ✅ Matches backend
}
```

**Fields**: 12 fields  
**Missing Backend Fields**: None (all fields present!)  
**Database Field Not in Rust Model**: `archived_at` (exists in database but not in Rust `ChatSession`)

### Frontend ChatData Interface (Internal)

**Location**: `apps/agent_management_dashboard/src/components/ChatContext.tsx`  
**Type**: `ChatData`

```typescript
interface ChatData {
  id: string;
  title: string; // ⚠️ Required (backend: optional)
  messages: Message[]; // ✅ Internal structure
  createdAt: Date; // ✅ Matches backend created_at
  groupId?: string; // ❌ Not in backend (invented field)
}
```

**Fields**: 4 fields (simplified for UI)  
**Missing Backend Fields**: Many (simplified for UI display)  
**Invented Fields**: `groupId` (not in backend)

### Issues Identified

#### 1. Database vs Rust Model Mismatch

| Database Field | Rust Model | Frontend   | Status                         |
| -------------- | ---------- | ---------- | ------------------------------ |
| `archived_at`  | ❌ Missing | ❌ Missing | **Database field not exposed** |

**Impact**: Cannot track when chat was archived

#### 2. ChatData Simplification

**Frontend `ChatData`** is simplified for UI purposes and doesn't include all backend fields. This is acceptable for internal UI state, but API responses should match backend exactly.

**Mapping**:

- `ChatSessionResponse` (API) → Should match backend exactly ✅
- `ChatData` (UI state) → Simplified, acceptable ⚠️

#### 3. Invented Fields

| Frontend Field | Location | Backend Equivalent | Issue                                           |
| -------------- | -------- | ------------------ | ----------------------------------------------- |
| `groupId`      | ChatData | None               | ❌ **Should derive from `folder_id` or remove** |

---

## Chat Message Schema Alignment

### Backend ChatMessage Model

**Location**: `iterations/v3/data-infrastructure/src/chat_service.rs`  
**Rust Struct**: `ChatMessage`  
**Database Table**: `chat_messages`

#### Field Catalog (9 Fields)

| Field Name        | Rust Type               | Database Type              | Nullable | Notes                                           |
| ----------------- | ----------------------- | -------------------------- | -------- | ----------------------------------------------- |
| `id`              | `Uuid`                  | `UUID`                     | No       | PRIMARY KEY                                     |
| `session_id`      | `Uuid`                  | `UUID`                     | No       | NOT NULL, REFERENCES chat_sessions(id)          |
| `role`            | `String`                | `VARCHAR(50)`              | No       | NOT NULL, CHECK ('user', 'assistant', 'system') |
| `content`         | `String`                | `TEXT`                     | No       | NOT NULL                                        |
| `metadata`        | `serde_json::Value`     | `JSONB`                    | No       | DEFAULT '{}'                                    |
| `created_at`      | `DateTime<Utc>`         | `TIMESTAMP WITH TIME ZONE` | No       | NOT NULL, DEFAULT NOW()                         |
| `edited_at`       | `Option<DateTime<Utc>>` | `TIMESTAMP WITH TIME ZONE` | Yes      | NULL allowed                                    |
| `token_count`     | `Option<i32>`           | `INTEGER`                  | Yes      | NULL allowed                                    |
| `model_used`      | `Option<String>`        | `VARCHAR(255)`             | Yes      | NULL allowed                                    |
| `sequence_number` | `i32`                   | `INTEGER`                  | No       | NOT NULL, DEFAULT 0                             |

**Note**: Database has `parent_message_id` field (UUID REFERENCES chat_messages(id)) but Rust model doesn't include it!

### Frontend ChatMessageResponse Interface

**Location**: `apps/agent_management_dashboard/src/lib/api/chat.ts`  
**Type**: `ChatMessageResponse`

```typescript
export interface ChatMessageResponse {
  id: string; // ✅ Matches backend
  session_id: string; // ✅ Matches backend
  role: string; // ✅ Matches backend
  content: string; // ✅ Matches backend
  metadata: Record<string, unknown>; // ✅ Matches backend (JSONB)
  created_at: string; // ✅ Matches backend (as RFC3339)
  edited_at?: string; // ✅ Matches backend (as RFC3339)
  token_count?: number; // ✅ Matches backend
  model_used?: string; // ✅ Matches backend
  sequence_number: number; // ✅ Matches backend
}
```

**Fields**: 9 fields  
**Missing Backend Fields**: None (all fields present!)  
**Database Field Not in Rust Model**: `parent_message_id` (exists in database but not in Rust `ChatMessage`)

### Frontend Message Schema (Internal)

**Location**: `apps/agent_management_dashboard/src/lib/schemas/chat.ts`  
**Type**: `Message`

Need to check this schema for comparison...

---

## Update Capabilities Analysis

### Worker Update Capabilities

**Backend UpdateWorker** (`iterations/v3/data-infrastructure/src/database_operations.rs`):

```rust
pub struct UpdateWorker {
    pub name: Option<String>,
    pub worker_type: Option<String>,
    pub specialty: Option<String>,
    pub model_name: Option<String>,
    pub endpoint: Option<String>,
    pub capabilities: Option<serde_json::Value>,
    pub performance_history: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}
```

**Total Updateable Fields**: 8 (all fields except `id`, `created_at`, `updated_at`)

**Frontend Update Capabilities**: Need to check if frontend has update worker function...

---

## Summary of Divergences

### Worker/Agent Schema

| Issue Type       | Count | Examples                                                                             |
| ---------------- | ----- | ------------------------------------------------------------------------------------ |
| Missing Fields   | 2     | `created_at`, `updated_at`                                                           |
| Type Mismatches  | 3     | `capabilities` (JSONB vs string[]), `model_name` nullability, `endpoint` nullability |
| **Total Issues** | **5** |                                                                                      |

### Chat Session Schema

| Issue Type                | Count | Examples                                        |
| ------------------------- | ----- | ----------------------------------------------- |
| Database vs Rust Mismatch | 1     | `archived_at` in database but not in Rust model |
| Frontend Simplification   | 1     | `ChatData` simplified for UI (acceptable)       |
| Invented Fields           | 1     | `groupId` in ChatData                           |
| **Total Issues**          | **3** | (minor, mostly acceptable)                      |

### Chat Message Schema

| Issue Type                | Count | Examples                                              |
| ------------------------- | ----- | ----------------------------------------------------- |
| Database vs Rust Mismatch | 1     | `parent_message_id` in database but not in Rust model |
| **Total Issues**          | **1** |                                                       |

---

## Recommendations

### High Priority

1. **Add Missing Worker Timestamps**:

   - Add `created_at` and `updated_at` to frontend `Agent` interface
   - Update API client to receive these fields
   - Display in UI where appropriate

2. **Fix Worker Capabilities Type**:

   - Change `capabilities` from `string[] | null` to `Record<string, unknown>` to match backend JSONB
   - OR: Create structured `Capabilities` interface matching backend JSON structure
   - Update UI to handle structured capabilities

3. **Fix Worker Nullability**:
   - Change `model_name` and `endpoint` from nullable to required in frontend
   - OR: Update backend to allow null (less breaking)

### Medium Priority

4. **Add `archived_at` to Rust ChatSession Model**:

   - Add `archived_at: Option<DateTime<Utc>>` to Rust `ChatSession` struct
   - Update database queries to include `archived_at`
   - Add to frontend `ChatSessionResponse` interface

5. **Add `parent_message_id` to Rust ChatMessage Model**:

   - Add `parent_message_id: Option<Uuid>` to Rust `ChatMessage` struct
   - Update database queries to include `parent_message_id`
   - Add to frontend `ChatMessageResponse` interface
   - Enable message threading in UI

6. **Remove Invented Fields**:
   - Remove `groupId` from `ChatData` interface
   - Use `folder_id` from backend instead (if needed)

### Low Priority

7. **Worker Update API**:
   - Verify frontend has `updateWorker` function
   - Ensure it supports all 8 backend updateable fields
   - Add validation using Zod schemas

---

## Field Mapping Tables

### Worker/Agent Fields

| Backend (Rust)                           | Frontend (TypeScript)          | Status     |
| ---------------------------------------- | ------------------------------ | ---------- | ----------------------- |
| `id: Uuid`                               | `id: string`                   | ✅ Matches |
| `name: String`                           | `name: string`                 | ✅ Matches |
| `worker_type: String`                    | `worker_type: string`          | ✅ Matches |
| `specialty: Option<String>`              | `specialty: string             | null`      | ✅ Matches              |
| `model_name: String`                     | `model_name: string            | null`      | ⚠️ Nullability mismatch |
| `endpoint: String`                       | `endpoint: string              | null`      | ⚠️ Nullability mismatch |
| `capabilities: serde_json::Value`        | `capabilities: string[]        | null`      | ⚠️ Type mismatch        |
| `performance_history: serde_json::Value` | `performance_history: unknown` | ✅ Matches |
| `is_active: bool`                        | `is_active: boolean`           | ✅ Matches |
| `created_at: DateTime<Utc>`              | ❌ Missing                     | ❌ Missing |
| `updated_at: DateTime<Utc>`              | ❌ Missing                     | ❌ Missing |

### Chat Session Fields

| Backend (Rust)                           | Frontend (TypeScript)               | Database                   | Status                              |
| ---------------------------------------- | ----------------------------------- | -------------------------- | ----------------------------------- |
| `id: Uuid`                               | `id: string`                        | UUID                       | ✅ Matches                          |
| `workspace_id: Option<Uuid>`             | `workspace_id?: string`             | UUID                       | ✅ Matches                          |
| `tenant_id: Option<Uuid>`                | `tenant_id?: string`                | UUID                       | ✅ Matches                          |
| `title: Option<String>`                  | `title?: string`                    | VARCHAR(500)               | ✅ Matches                          |
| `created_at: DateTime<Utc>`              | `created_at: string`                | TIMESTAMPTZ                | ✅ Matches                          |
| `updated_at: DateTime<Utc>`              | `updated_at: string`                | TIMESTAMPTZ                | ✅ Matches                          |
| `last_message_at: Option<DateTime<Utc>>` | `last_message_at?: string`          | TIMESTAMPTZ                | ✅ Matches                          |
| `message_count: i32`                     | `message_count: number`             | INTEGER                    | ✅ Matches                          |
| `metadata: serde_json::Value`            | `metadata: Record<string, unknown>` | JSONB                      | ✅ Matches                          |
| `archived: bool`                         | `archived: boolean`                 | BOOLEAN                    | ✅ Matches                          |
| `pinned: bool`                           | `pinned: boolean`                   | BOOLEAN                    | ✅ Matches                          |
| `folder_id: Option<Uuid>`                | `folder_id?: string`                | UUID                       | ✅ Matches                          |
| ❌ Missing                               | ❌ Missing                          | `archived_at: TIMESTAMPTZ` | ⚠️ Database field not in Rust model |

### Chat Message Fields

| Backend (Rust)                     | Frontend (TypeScript)               | Database                  | Status                              |
| ---------------------------------- | ----------------------------------- | ------------------------- | ----------------------------------- |
| `id: Uuid`                         | `id: string`                        | UUID                      | ✅ Matches                          |
| `session_id: Uuid`                 | `session_id: string`                | UUID                      | ✅ Matches                          |
| `role: String`                     | `role: string`                      | VARCHAR(50)               | ✅ Matches                          |
| `content: String`                  | `content: string`                   | TEXT                      | ✅ Matches                          |
| `metadata: serde_json::Value`      | `metadata: Record<string, unknown>` | JSONB                     | ✅ Matches                          |
| `created_at: DateTime<Utc>`        | `created_at: string`                | TIMESTAMPTZ               | ✅ Matches                          |
| `edited_at: Option<DateTime<Utc>>` | `edited_at?: string`                | TIMESTAMPTZ               | ✅ Matches                          |
| `token_count: Option<i32>`         | `token_count?: number`              | INTEGER                   | ✅ Matches                          |
| `model_used: Option<String>`       | `model_used?: string`               | VARCHAR(255)              | ✅ Matches                          |
| `sequence_number: i32`             | `sequence_number: number`           | INTEGER                   | ✅ Matches                          |
| ❌ Missing                         | ❌ Missing                          | `parent_message_id: UUID` | ⚠️ Database field not in Rust model |

---

## System-Wide Schema Alignment Summary

### Complete Coverage Statistics

| Schema           | Backend Fields | Frontend Coverage | Missing Fields | Type Issues | Total Issues |
| ---------------- | -------------- | ----------------- | -------------- | ----------- | ------------ |
| **Task**         | 17             | 53-65%            | 6-8 fields     | 3           | 17+          |
| **Worker/Agent** | 11             | 82%               | 2 fields       | 3           | 5            |
| **Chat Session** | 12/13          | 100% (Rust)       | 0 (1 DB field) | 0           | 1            |
| **Chat Message** | 9/10           | 100% (Rust)       | 0 (1 DB field) | 0           | 1            |
| **TOTAL**        | **49/51**      | **~75%**          | **8-10**       | **6**       | **24+**      |

### Critical Issues Breakdown

**🚨 CRITICAL (Blocking Agent Workflow)**:

1. Task status enum mismatch (backend 6 values vs frontend 4)
2. Task context updates not supported
3. Task acceptance criteria updates not supported

**⚠️ HIGH (Feature Completeness)**: 4. Worker capabilities type mismatch (JSONB vs string[]) 5. Worker timestamps missing (created_at, updated_at) 6. Task missing 6-8 fields (scope, acceptance_criteria, context, deadline, etc.) 7. Task update capabilities only 36% of backend 8. Chat `archived_at` not exposed 9. Chat `parent_message_id` not exposed (blocks threading)

**📋 MEDIUM (Cleanup)**: 10. Worker nullability mismatches 11. Task field name inconsistencies 12. Task invented fields (type, started_at)

### Alignment Status by Schema

- **Task Schema**: 🚨 **CRITICAL ISSUES** - Status enum mismatch blocks workflow
- **Worker/Agent Schema**: ⚠️ **HIGH PRIORITY** - Missing timestamps, capabilities type mismatch
- **Chat Session Schema**: ✅ **MOSTLY ALIGNED** - Minor database field missing
- **Chat Message Schema**: ✅ **MOSTLY ALIGNED** - Minor database field missing

---

**Last Updated**: 2025-01-28  
**Reference Documents**:

- `SCHEMA_ALIGNMENT.md` - Task and Project schema alignment (detailed)
- `SCHEMA_DIVERGENCE_CATALOG.md` - Complete field-by-field comparison (all schemas)
- `REALIGNMENT_IMPLEMENTATION_PLAN.md` - Prioritized implementation plan (4 weeks, 16 phases)  
  **Next Review**: After Phase 1 critical fixes completed
