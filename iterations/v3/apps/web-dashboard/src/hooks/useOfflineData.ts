"use client";

import { useState, useEffect, useCallback } from "react";

export interface CachedData<T> {
  data: T;
  timestamp: number;
  expiresAt?: number;
  version: string;
}

export interface OfflineDataOptions {
  key: string;
  ttl?: number; // Time to live in milliseconds
  version?: string; // Data version for invalidation
}

export interface UseOfflineDataReturn<T> {
  data: T | null;
  isLoading: boolean;
  error: string | null;
  isStale: boolean;
  lastUpdated: Date | null;
  refresh: () => Promise<void>;
  updateCache: (data: T) => void;
  clearCache: () => void;
}

/**
 * Hook for managing offline data with caching and synchronization
 * Provides fallback data when API is unavailable
 */
export function useOfflineData<T>(
  fetcher: () => Promise<T>,
  options: OfflineDataOptions,
  fallbackData?: T
): UseOfflineDataReturn<T> {
  const { key, ttl = 5 * 60 * 1000, version = "1.0" } = options; // 5 minutes default TTL

  const [data, setData] = useState<T | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isStale, setIsStale] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  // Cache key with version
  const cacheKey = `offline_${key}_v${version}`;

  // Load data from cache on mount
  useEffect(() => {
    const loadFromCache = () => {
      try {
        const cached = localStorage.getItem(cacheKey);
        if (cached) {
          const parsed: CachedData<T> = JSON.parse(cached);
          const now = Date.now();

          // Check if cache is still valid
          if (!parsed.expiresAt || now < parsed.expiresAt) {
            setData(parsed.data);
            setLastUpdated(new Date(parsed.timestamp));
            setIsStale(now - parsed.timestamp > ttl / 2); // Consider stale after half TTL
          } else {
            // Cache expired, remove it
            localStorage.removeItem(cacheKey);
          }
        }
      } catch (err) {
        console.warn("Failed to load cached data:", err);
        localStorage.removeItem(cacheKey);
      }
    };

    loadFromCache();
  }, [cacheKey, ttl]);

  const updateCache = useCallback((newData: T) => {
    try {
      const cacheEntry: CachedData<T> = {
        data: newData,
        timestamp: Date.now(),
        expiresAt: Date.now() + ttl,
        version,
      };

      localStorage.setItem(cacheKey, JSON.stringify(cacheEntry));
      setData(newData);
      setLastUpdated(new Date());
      setIsStale(false);
      setError(null);
    } catch (err) {
      console.warn("Failed to update cache:", err);
    }
  }, [cacheKey, ttl, version]);

  const clearCache = useCallback(() => {
    localStorage.removeItem(cacheKey);
    setData(null);
    setLastUpdated(null);
    setIsStale(false);
  }, [cacheKey]);

  const refresh = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const freshData = await fetcher();
      updateCache(freshData);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to fetch data";

      // If we have cached data, keep using it but mark as error
      if (data) {
        setError(`Failed to refresh: ${errorMessage}`);
        setIsStale(true);
      } else {
        // No cached data, use fallback if available
        setError(errorMessage);
        if (fallbackData) {
          setData(fallbackData);
        }
      }
    } finally {
      setIsLoading(false);
    }
  }, [fetcher, updateCache, data, fallbackData]);

  // Auto-refresh when component mounts and data is stale or missing
  useEffect(() => {
    if (!data || isStale) {
      refresh();
    }
  }, []); // Only run on mount

  return {
    data: data || fallbackData || null,
    isLoading,
    error,
    isStale,
    lastUpdated,
    refresh,
    updateCache,
    clearCache,
  };
}

// Predefined hooks for common data types
export function useOfflineTasks() {
  const fallbackTasks = [
    {
      id: "sample-task-1",
      title: "Sample Task (Offline Mode)",
      status: "pending",
      description: "This is a sample task shown when the API is unavailable.",
      createdAt: new Date().toISOString(),
    },
  ];

  return useOfflineData(
    async () => {
      const response = await fetch("/api/tasks");
      if (!response.ok) throw new Error("Failed to fetch tasks");
      return response.json();
    },
    { key: "tasks", ttl: 10 * 60 * 1000 }, // 10 minutes TTL
    fallbackTasks
  );
}

export function useOfflineMetrics() {
  const fallbackMetrics = {
    totalTasks: 0,
    completedTasks: 0,
    activeTasks: 0,
    failedTasks: 0,
    averageCompletionTime: 0,
    systemHealth: "unknown",
  };

  return useOfflineData(
    async () => {
      const response = await fetch("/api/metrics");
      if (!response.ok) throw new Error("Failed to fetch metrics");
      return response.json();
    },
    { key: "metrics", ttl: 2 * 60 * 1000 }, // 2 minutes TTL
    fallbackMetrics
  );
}

