# Cache Strategy for Zustand Stores

**Author:** @darianrosebrook  
**Date:** 2025-01-28

## Cache Invalidation Patterns

### Pattern 1: Time-Based Invalidation

```typescript
interface CacheMetadata {
  lastFetched: number;
  ttl: number; // milliseconds
}

function isCacheStale(metadata: CacheMetadata): boolean {
  return Date.now() - metadata.lastFetched > metadata.ttl;
}

// Usage in store
fetchChatSessions: async () => {
  const { chats, cacheMetadata } = get();
  
  // Use cache if fresh
  if (chats.length > 0 && !isCacheStale(cacheMetadata)) {
    return;
  }
  
  // Fetch from database
  const data = await api.chat.listSessions();
  set({ 
    chats: data,
    cacheMetadata: { lastFetched: Date.now(), ttl: 60000 } // 1 minute
  });
}
```

### Pattern 2: Event-Based Invalidation

```typescript
// Invalidate cache when data changes
createChatSession: async (request) => {
  const session = await api.chat.createSession(request);
  
  // Add to cache optimistically
  set(state => ({ chats: [session, ...state.chats] }));
  
  // Refresh from database to ensure consistency
  await get().fetchChatSessions();
}
```

### Pattern 3: Manual Refresh

```typescript
refreshChats: async () => {
  // Force refresh from database
  const data = await api.chat.listSessions();
  set({ 
    chats: data,
    cacheMetadata: { lastFetched: Date.now(), ttl: 60000 }
  });
}
```

## Recommended TTL Values

- **Chat sessions**: 30 seconds (frequently updated)
- **Projects**: 60 seconds (moderately updated)
- **Tasks**: 15 seconds (very frequently updated)
- **Agents**: 60 seconds (rarely updated)
- **System metrics**: 5 seconds (real-time data)

## Implementation

Add to each store:

```typescript
interface StoreState {
  // Data
  items: Item[];
  
  // Cache metadata
  cacheMetadata: {
    lastFetched: number;
    ttl: number;
  };
  
  // Actions
  fetchItems: () => Promise<void>;
  refreshItems: () => Promise<void>; // Force refresh
  isCacheStale: () => boolean;
}
```

