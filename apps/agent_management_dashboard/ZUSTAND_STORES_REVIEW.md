# Zustand Stores Review & Recommendations

**Author:** @darianrosebrook  
**Date:** 2025-01-28

## Executive Summary

The current Zustand store architecture follows the correct pattern (database as source of truth, Zustand as cache + UI state), but cache management is incomplete. This review provides specific recommendations to complete the implementation.

## Current State Analysis

### ✅ What's Working Well

1. **Architecture Pattern**

   - Database is source of truth (stores fetch from API → database)
   - Zustand used for caching and UI state
   - Optimistic updates implemented
   - Zod validation for API responses

2. **Store Structure**

   - `chatStore` - Chat sessions and messages
   - `projectStore` - Projects and tasks
   - `notificationStore` - Client-side notifications (localStorage)
   - `serverNotificationStore` - Server-side notifications (in-memory)

3. **Features Implemented**
   - Optimistic updates with rollback
   - Error handling with toast notifications
   - Loading states
   - Computed getters

### ❌ Issues Identified

#### 1. chatStore.ts - Incomplete Cache Management

**Problems:**

- Cache metadata only initialized in production build (line 439-442), missing in dev build (line 139)
- `isCacheStale()` method declared in interface (line 69) but not implemented
- `refreshChats()` method declared in interface (line 68) but not implemented
- `fetchChatSessions()` doesn't check cache before fetching (line 215, 518)
- Cache metadata not updated when fetching

**Current Code:**

```typescript
// Interface declares these methods (lines 68-69)
refreshChats: () => Promise<void>;
isCacheStale: () => boolean;

// But they're not implemented in either dev or production builds
```

#### 2. projectStore.ts - No Cache Management

**Problems:**

- No cache metadata structure
- No cache invalidation logic
- Always fetches from API (no cache-first pattern)
- No TTL or stale-while-revalidate

**Current Code:**

```typescript
// fetchProjects always fetches (line 257, 755)
fetchProjects: async () => {
  set({ isLoading: true, error: null });
  // Always fetches - no cache check
  const response = await listProjectsApiClient();
  // ...
};
```

#### 3. Both Stores - Not Using ApiProvider

**Problems:**

- Stores use direct `apiGet`, `apiPost`, `apiPatch` calls
- Should use `ApiProvider` for consistency
- Hard-coded API URLs in some places

**Current Code:**

```typescript
// chatStore.ts line 221, 524
const apiUrl = env.API_URL;
const data = await apiGet<unknown>(`${apiUrl}/api/v1/chat/sessions`, {...});

// Should use:
const api = useApi();
const data = await api.chat.listSessions();
```

## Recommendations

### Priority 1: Complete Cache Management in chatStore

#### 1.1 Add Cache Metadata to Dev Build

```typescript
// In devtools section (around line 139)
chats: [],
cacheMetadata: {
  lastFetched: 0,
  ttl: 30000, // 30 seconds for chat sessions
},
```

#### 1.2 Implement Missing Methods

```typescript
// Add to both dev and production builds
isCacheStale: () => {
  const { cacheMetadata } = get();
  return Date.now() - cacheMetadata.lastFetched > cacheMetadata.ttl;
},

refreshChats: async () => {
  // Force refresh from database
  await get().fetchChatSessions(true); // Pass forceRefresh flag
},
```

#### 1.3 Update fetchChatSessions to Check Cache

```typescript
fetchChatSessions: async (forceRefresh = false) => {
  const { chats, isCacheStale } = get();

  // Use cache if fresh and not forcing refresh
  if (!forceRefresh && chats.length > 0 && !isCacheStale()) {
    return; // Use cached data
  }

  set({ isLoading: true, error: null });
  const loadingToast = toastLoading("Loading chat sessions...");

  try {
    const apiUrl = env.API_URL;
    const data = await apiGet<unknown>(`${apiUrl}/api/v1/chat/sessions`, {
      retry: { maxAttempts: 3, initialDelay: 1000 },
    });

    const validatedSessions = validateApiResponse(
      ChatSessionsResponseSchema,
      data,
      "fetchChatSessions"
    );

    const chats: ChatData[] = validatedSessions.map((session) => ({
      id: session.id,
      title: session.title,
      messages: [],
      createdAt: session.created_at,
      groupId: undefined,
    }));

    // Update cache metadata
    set({
      chats,
      isLoading: false,
      cacheMetadata: {
        lastFetched: Date.now(),
        ttl: 30000, // 30 seconds
      },
    });
    loadingToast();
  } catch (error) {
    const appError = parseApiError(error);
    set({ error: appError, isLoading: false });
    loadingToast();
    toastError(error);
    throw appError;
  }
},
```

#### 1.4 Invalidate Cache on Mutations

```typescript
createChatSession: async (request) => {
  // ... existing code ...

  set((state) => ({
    chats: [newChat, ...state.chats],
    currentChatId: validatedSession.id,
    isLoading: false,
    // Update cache metadata
    cacheMetadata: {
      lastFetched: Date.now(),
      ttl: 30000,
    },
  }));

  // ... rest of code ...
},
```

### Priority 2: Add Cache Management to projectStore

#### 2.1 Add Cache Metadata to Interface

```typescript
interface ProjectState {
  // State
  projects: Project[];
  cacheMetadata: {
    lastFetched: number;
    ttl: number; // 60 seconds for projects
  };
  currentProjectId: string | null;
  isLoading: boolean;
  error: Error | null;

  // ... existing methods ...

  // Cache management
  refreshProjects: () => Promise<void>;
  isCacheStale: () => boolean;
}
```

#### 2.2 Initialize Cache Metadata

```typescript
// In both dev and production builds
projects: [],
cacheMetadata: {
  lastFetched: 0,
  ttl: 60000, // 60 seconds for projects
},
```

#### 2.3 Implement Cache Methods

```typescript
isCacheStale: () => {
  const { cacheMetadata } = get();
  return Date.now() - cacheMetadata.lastFetched > cacheMetadata.ttl;
},

refreshProjects: async () => {
  await get().fetchProjects(true); // Force refresh
},
```

#### 2.4 Update fetchProjects to Check Cache

```typescript
fetchProjects: async (forceRefresh = false) => {
  const { projects, isCacheStale } = get();

  // Use cache if fresh and not forcing refresh
  if (!forceRefresh && projects.length > 0 && !isCacheStale()) {
    return; // Use cached data
  }

  set({ isLoading: true, error: null });
  const loadingToast = toastLoading("Loading projects...");

  try {
    // ... existing fetch logic ...

    set({
      projects: projectsData,
      isLoading: false,
      cacheMetadata: {
        lastFetched: Date.now(),
        ttl: 60000, // 60 seconds
      },
    });
    loadingToast();
  } catch (error) {
    // ... existing error handling ...
  }
},
```

### Priority 3: Migrate to ApiProvider (Optional but Recommended)

#### 3.1 Update Stores to Use ApiProvider

**Challenge:** Zustand stores can't use React hooks directly.

**Solution Options:**

**Option A: Pass API as Parameter**

```typescript
// Store receives API instance
interface ChatState {
  // ... existing state ...
  fetchChatSessions: (
    api: ApiContextValue,
    forceRefresh?: boolean
  ) => Promise<void>;
}

// Usage in components
const api = useApi();
await store.fetchChatSessions(api);
```

**Option B: Store API in Zustand**

```typescript
// Store API instance in Zustand
interface ChatState {
  api: ApiContextValue | null;
  setApi: (api: ApiContextValue) => void;
  fetchChatSessions: (forceRefresh?: boolean) => Promise<void>;
}

// Initialize in component
useEffect(() => {
  const api = useApi();
  store.setApi(api);
}, []);

// Use in store
fetchChatSessions: async (forceRefresh = false) => {
  const { api } = get();
  if (!api) throw new Error("API not initialized");

  const data = await api.chat.listSessions();
  // ...
};
```

**Option C: Keep Current Pattern (Recommended for Now)**

- Current pattern works fine
- Migration can be done later
- Focus on completing cache management first

## Implementation Plan

### Phase 1: Complete chatStore Cache Management (High Priority)

1. ✅ Add cache metadata to dev build
2. ✅ Implement `isCacheStale()` method
3. ✅ Implement `refreshChats()` method
4. ✅ Update `fetchChatSessions()` to check cache
5. ✅ Update cache metadata on fetch
6. ✅ Invalidate cache on mutations

**Estimated Time:** 30 minutes

### Phase 2: Add Cache Management to projectStore (High Priority)

1. ✅ Add cache metadata to interface
2. ✅ Initialize cache metadata
3. ✅ Implement `isCacheStale()` method
4. ✅ Implement `refreshProjects()` method
5. ✅ Update `fetchProjects()` to check cache
6. ✅ Update cache metadata on fetch
7. ✅ Invalidate cache on mutations

**Estimated Time:** 30 minutes

### Phase 3: Update Components to Use Cache (Medium Priority)

1. Update components to check cache before fetching
2. Add manual refresh buttons where appropriate
3. Show cache status in dev mode

**Estimated Time:** 20 minutes

### Phase 4: Migrate to ApiProvider (Low Priority - Future)

1. Decide on approach (Option A, B, or C)
2. Update stores to use ApiProvider
3. Remove hard-coded API URLs
4. Update all components

**Estimated Time:** 2-3 hours

## Recommended TTL Values

Based on data update frequency:

- **Chat sessions**: 30 seconds (frequently updated)
- **Chat messages**: 15 seconds (very frequently updated)
- **Projects**: 60 seconds (moderately updated)
- **Tasks**: 15 seconds (very frequently updated)
- **Agents**: 60 seconds (rarely updated)
- **System metrics**: 5 seconds (real-time data)

## Testing Checklist

After implementing cache management:

- [ ] Cache is checked before fetching
- [ ] Cache is used when fresh
- [ ] Cache is refreshed when stale
- [ ] Cache is invalidated on mutations
- [ ] Manual refresh works
- [ ] Cache metadata is updated correctly
- [ ] No infinite fetch loops
- [ ] Error handling works with cache

## Code Quality Notes

1. **DRY Principle**: Both stores have duplicate code - consider extracting cache utilities
2. **Type Safety**: Cache metadata should be typed consistently
3. **Error Handling**: Cache errors shouldn't break the app
4. **Performance**: Cache checks should be fast (no API calls)

## Conclusion

The current architecture is sound, but cache management needs to be completed. Priority 1 and 2 should be implemented immediately to improve performance and data freshness. Priority 3 can be done incrementally, and Priority 4 is a future enhancement.

**Next Steps:**

1. Implement Phase 1 (chatStore cache management)
2. Implement Phase 2 (projectStore cache management)
3. Test thoroughly
4. Update components to leverage cache







