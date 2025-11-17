/**
 * Worker Utilities
 * 
 * Utilities for worker/agent ID lookups and name resolution.
 * 
 * @author @darianrosebrook
 */

import { getAgents, type Agent } from '../api/agents';

/**
 * Worker name cache
 * 
 * Maps worker ID to worker name for fast lookups
 */
const workerNameCache = new Map<string, string>();

/**
 * Worker cache timestamp
 * 
 * Tracks when cache was last updated
 */
let cacheTimestamp: number = 0;

/**
 * Cache TTL: 5 minutes
 */
const CACHE_TTL = 5 * 60 * 1000;

/**
 * Get worker name by ID
 * 
 * Uses cache if available, otherwise fetches from API
 * 
 * @param workerId - Worker UUID
 * @returns Worker name or null if not found
 */
export async function getWorkerName(workerId: string | null | undefined): Promise<string | null> {
  if (!workerId) {
    return null;
  }

  // Check cache first
  if (workerNameCache.has(workerId)) {
    return workerNameCache.get(workerId) || null;
  }

  // Check if cache is stale
  const now = Date.now();
  if (now - cacheTimestamp > CACHE_TTL) {
    // Refresh cache
    await refreshWorkerCache();
  }

  // Check cache again after refresh
  return workerNameCache.get(workerId) || null;
}

/**
 * Refresh worker name cache
 * 
 * Fetches all agents and caches their names
 */
export async function refreshWorkerCache(): Promise<void> {
  try {
    const agents = await getAgents();
    const agentArray: Agent[] = Array.isArray(agents) ? agents : [];

    workerNameCache.clear();

    for (const agent of agentArray) {
      if (agent.id && agent.name) {
        workerNameCache.set(agent.id, agent.name);
      }
    }

    cacheTimestamp = Date.now();
  } catch (error) {
    console.error('Failed to refresh worker cache:', error);
    // Don't throw - cache will be empty, lookups will return null
  }
}

/**
 * Get multiple worker names
 * 
 * Batch lookup for multiple worker IDs
 * 
 * @param workerIds - Array of worker UUIDs
 * @returns Map of worker ID to name
 */
export async function getWorkerNames(
  workerIds: (string | null | undefined)[]
): Promise<Map<string, string>> {
  const result = new Map<string, string>();

  // Filter out null/undefined and duplicates
  const uniqueIds = Array.from(new Set(workerIds.filter((id): id is string => !!id)));

  if (uniqueIds.length === 0) {
    return result;
  }

  // Refresh cache if needed
  const now = Date.now();
  if (now - cacheTimestamp > CACHE_TTL) {
    await refreshWorkerCache();
  }

  // Look up each ID
  for (const id of uniqueIds) {
    const name = workerNameCache.get(id);
    if (name) {
      result.set(id, name);
    }
  }

  return result;
}

/**
 * Clear worker name cache
 * 
 * Forces next lookup to refresh from API
 */
export function clearWorkerCache(): void {
  workerNameCache.clear();
  cacheTimestamp = 0;
}

/**
 * Preload worker names for given IDs
 * 
 * Useful for batch loading worker names before displaying tasks
 * 
 * @param workerIds - Array of worker UUIDs to preload
 */
export async function preloadWorkerNames(workerIds: (string | null | undefined)[]): Promise<void> {
  const uniqueIds = Array.from(new Set(workerIds.filter((id): id is string => !!id)));

  if (uniqueIds.length === 0) {
    return;
  }

  // Check which IDs are missing from cache
  const missingIds = uniqueIds.filter((id) => !workerNameCache.has(id));

  if (missingIds.length === 0) {
    return; // All already cached
  }

  // Refresh cache to get missing names
  await refreshWorkerCache();
}

