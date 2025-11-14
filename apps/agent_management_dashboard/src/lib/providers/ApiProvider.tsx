/**
 * API Provider
 *
 * Centralized React Context provider for API state management and endpoint access.
 * All API calls should go through this provider to ensure consistency, error handling,
 * and avoid hard-coded endpoints.
 *
 * @author @darianrosebrook
 */

import React, { createContext, useContext, useMemo, ReactNode } from 'react';
import { apiFetch } from '../utils/api';
import * as healthApi from '../api/health';
import * as tasksApi from '../api/tasks';
import * as chatApi from '../api/chat';
import * as agentsApi from '../api/agents';
import * as observabilityApi from '../api/observability';
import * as projectsApi from '../api/projects';
import * as analyticsApi from '../api/analytics';
import * as searchApi from '../api/search';

// New API modules
import * as authApi from '../api/auth';
import * as judgesApi from '../api/judges';
import * as systemApi from '../api/system';
import * as sessionsApi from '../api/sessions';
import * as queriesApi from '../api/queries';
import * as queryPerformanceApi from '../api/queryPerformance';
import * as provenanceApi from '../api/provenance';
import * as waiversApi from '../api/waivers';
import * as slosApi from '../api/slos';
import * as databaseApi from '../api/database';

/**
 * API Provider Context Value
 */
export interface ApiContextValue {
  // Health endpoints
  health: typeof healthApi;
  
  // Task management
  tasks: typeof tasksApi;
  
  // Chat endpoints
  chat: typeof chatApi;
  
  // Authentication
  auth: typeof authApi;
  
  // Agent management
  agents: typeof agentsApi;
  
  // Judge management
  judges: typeof judgesApi;
  
  // Observability & Telemetry
  observability: typeof observabilityApi;
  
  // System monitoring
  system: typeof systemApi;
  
  // Session control
  sessions: typeof sessionsApi;
  
  // Search & Queries
  search: typeof searchApi;
  queries: typeof queriesApi;
  
  // Query performance
  queryPerformance: typeof queryPerformanceApi;
  
  // Provenance
  provenance: typeof provenanceApi;
  
  // Waivers
  waivers: typeof waiversApi;
  
  // SLOs
  slos: typeof slosApi;
  
  // Projects
  projects: typeof projectsApi;
  
  // Database
  database: typeof databaseApi;
  
  // Analytics
  analytics: typeof analyticsApi;
  
  // Direct API access (for advanced use cases)
  fetch: typeof apiFetch;
}

const ApiContext = createContext<ApiContextValue | null>(null);

/**
 * API Provider Props
 */
export interface ApiProviderProps {
  children: ReactNode;
  /**
   * Custom API base URL (defaults to /api/proxy/api/v1)
   */
  baseUrl?: string;
}

/**
 * API Provider Component
 *
 * Provides centralized API access through React Context.
 * All API modules are exposed through the context value.
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const api = useApi();
 *
 *   const handleSubmit = async () => {
 *     const result = await api.tasks.submitTask({
 *       description: 'Fix bug',
 *       priority: 'high'
 *     });
 *   };
 * }
 * ```
 */
export function ApiProvider({ children, baseUrl }: ApiProviderProps) {
  const contextValue = useMemo<ApiContextValue>(() => ({
    health: healthApi,
    tasks: tasksApi,
    chat: chatApi,
    auth: authApi,
    agents: agentsApi,
    judges: judgesApi,
    observability: observabilityApi,
    system: systemApi,
    sessions: sessionsApi,
    search: searchApi,
    queries: queriesApi,
    queryPerformance: queryPerformanceApi,
    provenance: provenanceApi,
    waivers: waiversApi,
    slos: slosApi,
    projects: projectsApi,
    database: databaseApi,
    analytics: analyticsApi,
    fetch: apiFetch,
  }), []);

  return (
    <ApiContext.Provider value={contextValue}>
      {children}
    </ApiContext.Provider>
  );
}

/**
 * Hook to access the API context
 *
 * @throws Error if used outside of ApiProvider
 * @returns API context value with all API modules
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const api = useApi();
 *   const { tasks, agents } = api;
 *
 *   // Use API methods
 *   const allTasks = await tasks.listTasks();
 *   const allAgents = await agents.getAgents();
 * }
 * ```
 */
export function useApi(): ApiContextValue {
  const context = useContext(ApiContext);
  
  if (!context) {
    throw new Error('useApi must be used within an ApiProvider');
  }
  
  return context;
}

/**
 * Hook to access specific API module
 *
 * @param module - API module name
 * @returns API module
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const tasks = useApiModule('tasks');
 *   const agents = useApiModule('agents');
 * }
 * ```
 */
export function useApiModule<K extends keyof ApiContextValue>(
  module: K
): ApiContextValue[K] {
  const api = useApi();
  return api[module];
}

