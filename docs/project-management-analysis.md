# Project Management Analysis

**Date:** 2025-01-28  
**Author:** @darianrosebrook  
**Purpose:** Comprehensive analysis of project creation, persistence, and tab management

---

## Executive Summary

The project management system has **dual implementations** - one using local state (`ProjectContext`) and one using Zustand store with API integration (`projectStore`). Projects are not consistently persisted, and tab management uses Next.js routing but could be improved.

---

## Current Architecture

### 1. Project Creation Flow

#### **Two Implementation Paths:**

**Path A: ProjectContext (Local State Only)**
- **Location:** `components/projects/ProjectContext.tsx`
- **Status:** ❌ **Not Persisted** - Uses local React state
- **Flow:**
  1. User fills form in `NewProjectModal`
  2. Calls `createProject()` from context
  3. Generates client-side ID: `project-${Date.now()}`
  4. Updates local state only
  5. **Data is lost on page refresh**

**Path B: projectStore (API Integration)**
- **Location:** `lib/stores/projectStore.ts`
- **Status:** ✅ **Partially Persisted** - Has API methods but not always used
- **Flow:**
  1. User fills form in `NewProjectModal`
  2. Calls `createProject()` (local) OR `createProjectApi()` (API)
  3. `createProjectApi()` calls `POST /api/v1/projects`
  4. Updates Zustand store with server response
  5. **Data persists in database**

#### **Current Usage:**
- `Projects.tsx` uses `useProjectStore()` → calls `createProject()` (local)
- `ProjectContext.tsx` provides local state version
- **Issue:** Components using `ProjectContext` don't persist to database

---

### 2. Project Persistence

#### **API Endpoints Available:**
```typescript
// ✅ Available in API client
GET    /api/v1/projects              // List projects
GET    /api/v1/projects/:id          // Get project details
PATCH  /api/v1/projects/:id          // Update project
POST   /api/v1/projects              // Create project (used in store, not in API client)

// ❌ Missing from API client
POST   /api/v1/projects              // createProject() function missing
```

#### **Store Implementation:**
```typescript
// projectStore.ts has:
- fetchProjects()      // ✅ Fetches from API
- createProjectApi()   // ✅ Creates via API
- createProject()      // ❌ Local only (dev mode)
- updateProject()     // ✅ Updates via API
```

#### **Issues:**
1. **No `createProject()` in API client** - Store calls API directly
2. **Projects not fetched on mount** - No `useEffect` in `Projects.tsx` to load initial data
3. **ProjectContext doesn't use API** - Still uses local state
4. **Inconsistent state management** - Two different systems

---

### 3. Tab Management

#### **Current Implementation:**
- **Location:** `app/projects/[projectId]/layout.tsx`
- **Routing:** Next.js App Router with dynamic routes
- **Tabs:**
  - `overview` → `/projects/[projectId]`
  - `workspace` → `/projects/[projectId]/workspace`
  - `tasks` → `/projects/[projectId]/tasks`
  - `timeline` → `/projects/[projectId]/timeline`
  - `manage` → `/projects/[projectId]/manage`

#### **Tab State Management:**
```typescript
// Tab state derived from pathname
const getActiveTab = (): TabType => {
  const pathParts = pathname.split("/").filter(Boolean);
  const projectIndex = pathParts.indexOf(projectId);
  const tab = pathParts[projectIndex + 1];
  return tabMap[tab] ?? "overview";
};
```

#### **Strengths:**
- ✅ URL reflects current tab (shareable/bookmarkable)
- ✅ Browser back/forward works correctly
- ✅ No client-side state needed for tab

#### **Issues:**
1. **Project not loaded from API** - Relies on `ProjectContext` which may not have data
2. **No loading state** - Shows "Loading project..." but doesn't fetch
3. **Tab persistence** - Tab state not saved per project (could be feature)

---

## Detailed Findings

### Project Creation

#### **NewProjectModal Component:**
- **Location:** `components/projects/ProjectModal.tsx`
- **Fields:** name, description, status, priority, assignees, dueDate, tags
- **Current Behavior:** Only sends `name` and `description` to `onCreateProject`
- **Missing:** Status, priority, assignees, dueDate, tags are collected but not used

#### **API Request Schema:**
```typescript
// From projectStore.ts
CreateProjectRequestSchema = z.object({
  name: z.string().min(1),
  summary: z.string().optional(),
  description: z.string().optional(),
});
```

#### **Gap:** Modal collects more data than API accepts

---

### Project Loading

#### **Initial Load:**
- **Projects Page:** No `useEffect` to call `fetchProjects()`
- **Project Detail Page:** Relies on `ProjectContext.getProjectById()` which may be empty
- **Result:** Projects may not be loaded from database on page load

#### **Store Methods:**
```typescript
// Available but not called automatically
fetchProjects()        // Loads all projects from API
getProjectById()      // Gets from local store (may be empty)
```

---

### Tab Management Details

#### **Tab Navigation:**
- Uses Next.js `<Link>` components
- Tab state derived from URL pathname
- No client-side state management needed

#### **Tab Content Loading:**
- Each tab has its own page component:
  - `app/projects/[projectId]/overview/page.tsx`
  - `app/projects/[projectId]/workspace/page.tsx`
  - `app/projects/[projectId]/tasks/page.tsx`
  - `app/projects/[projectId]/timeline/page.tsx`
  - `app/projects/[projectId]/manage/page.tsx`

#### **Potential Issues:**
1. Each tab page may need to fetch project data independently
2. No shared loading state across tabs
3. Project data may be fetched multiple times

---

## Recommendations

### Priority 1: Fix Project Persistence

#### **1.1 Add `createProject()` to API Client**
```typescript
// Add to lib/api/projects.ts
export async function createProject(request: {
  name: string;
  summary?: string;
  description?: string;
}): Promise<ProjectApiResponse> {
  return apiPost<ProjectApiResponse>(`${API_BASE}/projects`, request);
}
```

#### **1.2 Update ProjectContext to Use API**
- Replace local state with API calls
- Use `createProject()` from API client
- Fetch projects on mount
- Update `last_accessed` on selection

#### **1.3 Ensure Projects Load on Page Mount**
```typescript
// Add to Projects.tsx
useEffect(() => {
  const { fetchProjects } = useProjectStore.getState();
  fetchProjects();
}, []);
```

### Priority 2: Unify State Management

#### **2.1 Choose Single Source of Truth**
- **Recommendation:** Use `projectStore` (Zustand) exclusively
- Remove `ProjectContext` or make it a wrapper around store
- Update all components to use store

#### **2.2 Standardize Project Creation**
- All components should call `createProjectApi()` from store
- Remove local-only `createProject()` method
- Ensure all created projects persist to database

### Priority 3: Enhance Tab Management

#### **3.1 Add Project Loading to Layout**
```typescript
// In layout.tsx
useEffect(() => {
  if (projectId) {
    const { fetchProjects, getProjectById } = useProjectStore.getState();
    const project = getProjectById(projectId);
    if (!project) {
      fetchProjects(); // Load if not in store
    }
  }
}, [projectId]);
```

#### **3.2 Add Tab State Persistence (Optional)**
- Save last visited tab per project
- Restore tab on project selection
- Store in project metadata or localStorage

#### **3.3 Optimize Data Fetching**
- Fetch project data once in layout
- Share data across tab pages via context/store
- Avoid duplicate API calls

### Priority 4: Complete Modal Implementation

#### **4.1 Use All Modal Fields**
- Update API schema to accept: status, priority, assignees, dueDate, tags
- Map modal fields to API request
- Store additional metadata in database

#### **4.2 Add Validation**
- Validate required fields before submission
- Show error messages for invalid data
- Handle API validation errors

---

## Implementation Plan

### Phase 1: API Client Enhancement
1. Add `createProject()` function to `lib/api/projects.ts`
2. Add proper TypeScript types
3. Add error handling

### Phase 2: Store Integration
1. Update `ProjectContext` to use store methods
2. Add `useEffect` to fetch projects on mount
3. Update project creation to use API

### Phase 3: Tab Management
1. Add project loading to layout
2. Share project data across tabs
3. Add loading states

### Phase 4: Modal Enhancement
1. Update API schema to accept all fields
2. Map modal fields to API request
3. Add validation

---

## Files to Modify

### High Priority
- `lib/api/projects.ts` - Add `createProject()` function
- `components/projects/ProjectContext.tsx` - Replace local state with API calls
- `components/projects/Projects.tsx` - Add `fetchProjects()` on mount
- `app/projects/[projectId]/layout.tsx` - Add project loading

### Medium Priority
- `components/projects/ProjectModal.tsx` - Use all form fields
- `lib/stores/projectStore.ts` - Ensure API methods are used consistently

### Low Priority
- Add tab persistence feature
- Optimize data fetching across tabs

---

## Testing Checklist

- [ ] Projects persist after page refresh
- [ ] Projects load on page mount
- [ ] Project creation calls API
- [ ] Tab navigation works correctly
- [ ] Project data shared across tabs
- [ ] Loading states display correctly
- [ ] Error handling works
- [ ] Last accessed timestamp updates

---

## Questions to Resolve

1. **Should we keep `ProjectContext` or migrate everything to `projectStore`?**
   - Recommendation: Migrate to store for consistency

2. **Should tabs persist per project?**
   - Could store last visited tab in project metadata

3. **Should we fetch project data in layout or each tab page?**
   - Recommendation: Fetch once in layout, share via context/store

4. **What fields should the create project API accept?**
   - Current: name, summary, description
   - Modal collects: status, priority, assignees, dueDate, tags
   - Need to decide which to support

---

## Next Steps

1. Review this analysis with team
2. Decide on state management approach
3. Prioritize implementation phases
4. Begin Phase 1 implementation


