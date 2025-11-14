# State Management Architecture

**Author:** @darianrosebrook  
**Date:** 2025-01-28

## Recommendation: Hybrid Approach

**Use both Zustand and Database, each for their strengths:**

### Database = Source of Truth

- **All persistent data** lives in the database
- **All API calls** fetch from database
- **Single source of truth** for all business data
- **Survives page refreshes** and browser sessions

### Zustand = UI State + Performance Cache

- **UI state** (selected items, filters, view preferences)
- **Performance caching** (reduces API calls)
- **Optimistic updates** (better UX)
- **Loading/error states** (component state management)
- **Temporary data** (draft messages, unsaved changes)

## Current Architecture Analysis

### ✅ What's Working Well

1. **Database as Source of Truth**

   - Stores fetch from API → Database
   - All mutations go through API → Database
   - No data stored only in Zustand

2. **Zustand as Cache Layer**

   - `chatStore` - Caches chat sessions/messages from API
   - `projectStore` - Caches projects/tasks from API
   - Reduces unnecessary API calls
   - Provides optimistic updates

3. **UI State Management**
   - `currentChatId` - UI state (which chat is selected)
   - `currentProjectId` - UI state (which project is selected)
   - `isLoading` - UI state (loading indicators)
   - `error` - UI state (error messages)

### 🔄 What Could Be Improved

1. **Cache Invalidation**

   - No automatic cache invalidation
   - Cache can become stale
   - Need manual refresh

2. **Cache TTL**

   - No time-to-live for cached data
   - Cache persists until manual refresh

3. **Direct API Calls**
   - Some components call API directly
   - Should go through stores for consistency

## Recommended Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Components                           │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│              Zustand Stores (Cache + UI)                │
│  • chatStore - Chat sessions/messages cache            │
│  • projectStore - Projects/tasks cache                  │
│  • UI state (selected items, filters, loading)          │
│  • Optimistic updates                                   │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│              API Provider (ApiProvider)                 │
│  • Centralized API access                               │
│  • Type safety with TypeScript                          │
│  • Zod validation                                       │
│  • Error handling                                       │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│              Database (PostgreSQL)                      │
│  • Source of truth                                      │
│  • All persistent data                                  │
│  • Survives refreshes                                   │
└─────────────────────────────────────────────────────────┘
```

## Implementation Guidelines

### When to Use Zustand

✅ **Use Zustand for:**

- UI state (selected items, filters, view preferences)
- Caching API responses (performance)
- Optimistic updates (better UX)
- Loading/error states (component state)
- Temporary/draft data (unsaved changes)

❌ **Don't use Zustand for:**

- Source of truth (use database)
- Data that must persist (use database)
- Data shared across users (use database)
- Critical business data (use database)

### When to Use Database

✅ **Use Database for:**

- All persistent data
- Source of truth
- Data that survives refreshes
- Data shared across sessions/users
- Audit trails and history

### Data Flow Pattern

```tsx
// 1. Component needs data
function MyComponent() {
  const api = useApi();
  const store = useChatStore();

  // 2. Check cache first
  const cachedChats = store.chats;

  // 3. If cache is stale or empty, fetch from database
  useEffect(() => {
    if (!cachedChats.length || isCacheStale()) {
      store.fetchChatSessions(); // Fetches from API → Database
    }
  }, []);

  // 4. Use cached data for rendering
  return <div>{cachedChats.map(...)}</div>;
}
```

## Cache Invalidation Strategy

### Option 1: Time-Based Invalidation (Recommended)

```typescript
interface CacheState {
  data: T[];
  lastFetched: number;
  ttl: number; // Time-to-live in milliseconds
}

function isCacheStale(state: CacheState): boolean {
  return Date.now() - state.lastFetched > state.ttl;
}
```

### Option 2: Event-Based Invalidation

```typescript
// Invalidate cache when data changes
store.createChatSession().then(() => {
  store.fetchChatSessions(); // Refresh cache
});
```

### Option 3: Manual Invalidation

```typescript
// Provide refresh method
store.refreshChats(); // Force refetch from database
```

## Migration Plan

### Phase 1: Keep Current Pattern (✅ Current State)

- Zustand stores cache API responses
- Database is source of truth
- API calls go through ApiProvider

### Phase 2: Add Cache Invalidation

- Add TTL to cached data
- Add refresh methods
- Add stale-while-revalidate pattern

### Phase 3: Consolidate API Calls

- Move all API calls through stores
- Remove direct API calls from components
- Use stores as single access point

### Phase 4: Optimize Performance

- Add request deduplication
- Add request batching
- Add background refresh

## Best Practices

### ✅ DO

1. **Always fetch from database via API**

   ```tsx
   // Good - Fetches from database
   await store.fetchChatSessions();
   ```

2. **Use Zustand for UI state**

   ```tsx
   // Good - UI state
   store.setCurrentChatId(chatId);
   ```

3. **Cache API responses**

   ```tsx
   // Good - Cache for performance
   const chats = store.chats; // From cache
   ```

4. **Invalidate cache on mutations**
   ```tsx
   // Good - Refresh after mutation
   await store.createChatSession();
   await store.fetchChatSessions(); // Refresh cache
   ```

### ❌ DON'T

1. **Don't use Zustand as source of truth**

   ```tsx
   // Bad - Data only in Zustand
   store.chats = [...]; // Without API call
   ```

2. **Don't skip database for mutations**

   ```tsx
   // Bad - Mutation without API
   store.addMessage(message); // Should call API first
   ```

3. **Don't cache forever**
   ```tsx
   // Bad - Never invalidate cache
   const chats = store.chats; // Might be stale
   ```

## Example: Chat Store Pattern

```typescript
interface ChatState {
  // Cache (from database)
  chats: ChatData[];
  lastFetched: number;

  // UI State
  currentChatId: string | null;
  isLoading: boolean;

  // Actions
  fetchChatSessions: () => Promise<void>; // Fetches from DB
  createChatSession: () => Promise<string>; // Creates in DB
  refreshChats: () => Promise<void>; // Force refresh from DB
}

// Usage
const store = useChatStore();

// Fetch from database (caches in Zustand)
await store.fetchChatSessions();

// Use cached data
const chats = store.chats;

// UI state
store.setCurrentChatId(chatId);
```

## Conclusion

**Keep using Zustand** for:

- UI state management
- Performance caching
- Optimistic updates
- Loading/error states

**Always use Database** for:

- Source of truth
- Persistent data
- Data that survives refreshes

**The current pattern is good** - just add cache invalidation and ensure all API calls go through the stores.
