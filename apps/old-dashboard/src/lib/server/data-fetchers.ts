/**
 * Server-Side Data Fetchers
 * Optimized data fetching for server components
 */

import { apiClient } from '../api/unified-client';

// Cache configuration
const CACHE_DURATION = 30; // seconds
const cache = new Map<string, { data: any; timestamp: number }>();

function getCachedData<T>(key: string): T | null {
  const cached = cache.get(key);
  if (cached && Date.now() - cached.timestamp < CACHE_DURATION * 1000) {
    return cached.data;
  }
  return null;
}

function setCachedData<T>(key: string, data: T): void {
  cache.set(key, { data, timestamp: Date.now() });
}

/**
 * Fetch system health data
 */
export async function getSystemHealth() {
  const cacheKey = 'system-health';
  const cached = getCachedData(cacheKey);
  if (cached) return cached;

  try {
    const data = await apiClient.getSystemHealth();
    setCachedData(cacheKey, data);
    return data;
  } catch (error) {
    console.error('Failed to fetch system health:', error);
    return null;
  }
}

/**
 * Fetch business metrics
 */
export async function getBusinessMetrics(timeRange?: string) {
  const cacheKey = `business-metrics-${timeRange || 'default'}`;
  const cached = getCachedData(cacheKey);
  if (cached) return cached;

  try {
    const data = await apiClient.getBusinessMetrics(timeRange);
    setCachedData(cacheKey, data);
    return data;
  } catch (error) {
    console.error('Failed to fetch business metrics:', error);
    return null;
  }
}

/**
 * Fetch agent performance data
 */
export async function getAgentPerformance() {
  const cacheKey = 'agent-performance';
  const cached = getCachedData(cacheKey);
  if (cached) return cached;

  try {
    const data = await apiClient.getAgentPerformance();
    setCachedData(cacheKey, data);
    return data;
  } catch (error) {
    console.error('Failed to fetch agent performance:', error);
    return null;
  }
}

/**
 * Fetch tasks data
 */
export async function getTasks(filters?: {
  status?: string;
  priority?: string;
  limit?: number;
  offset?: number;
}) {
  const cacheKey = `tasks-${JSON.stringify(filters || {})}`;
  const cached = getCachedData(cacheKey);
  if (cached) return cached;

  try {
    const data = await apiClient.getTasks(filters);
    setCachedData(cacheKey, data);
    return data;
  } catch (error) {
    console.error('Failed to fetch tasks:', error);
    return null;
  }
}

/**
 * Fetch alerts data
 */
export async function getAlerts(filters?: {
  severity?: string;
  status?: string;
  limit?: number;
}) {
  const cacheKey = `alerts-${JSON.stringify(filters || {})}`;
  const cached = getCachedData(cacheKey);
  if (cached) return cached;

  try {
    const data = await apiClient.getAlerts(filters);
    setCachedData(cacheKey, data);
    return data;
  } catch (error) {
    console.error('Failed to fetch alerts:', error);
    return null;
  }
}

/**
 * Fetch SLOs data
 */
export async function getSLOs() {
  const cacheKey = 'slos';
  const cached = getCachedData(cacheKey);
  if (cached) return cached;

  try {
    const data = await apiClient.getSLOs();
    setCachedData(cacheKey, data);
    return data;
  } catch (error) {
    console.error('Failed to fetch SLOs:', error);
    return null;
  }
}

/**
 * Fetch analytics data
 */
export async function getAnalytics(timeRange?: string) {
  const cacheKey = `analytics-${timeRange || 'default'}`;
  const cached = getCachedData(cacheKey);
  if (cached) return cached;

  try {
    const data = await apiClient.getAnalytics(timeRange);
    setCachedData(cacheKey, data);
    return data;
  } catch (error) {
    console.error('Failed to fetch analytics:', error);
    return null;
  }
}

/**
 * Clear cache (useful for development)
 */
export function clearCache() {
  cache.clear();
}

/**
 * Get cache statistics
 */
export function getCacheStats() {
  const now = Date.now();
  const entries = Array.from(cache.entries());
  
  return {
    totalEntries: entries.length,
    validEntries: entries.filter(([_, value]) => 
      now - value.timestamp < CACHE_DURATION * 1000
    ).length,
    expiredEntries: entries.filter(([_, value]) => 
      now - value.timestamp >= CACHE_DURATION * 1000
    ).length,
  };
}

