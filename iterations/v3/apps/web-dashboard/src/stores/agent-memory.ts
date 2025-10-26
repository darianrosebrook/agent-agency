/**
 * Agent Memory Store
 * Zustand store for agent memory management, context preservation, and knowledge graph state
 *
 * @author @darianrosebrook
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import {
  AgentMemory,
  MemoryEntry,
  MemoryRelationship,
  KnowledgeGraph,
  ContextSnapshot,
  MemoryHealthMetrics,
  MemoryAlert,
  MemoryOptimization,
  AgentLearningMetrics,
  MemoryQuery,
  MemorySearchResult,
} from '@/lib/agent-memory-api';

interface AgentMemoryState {
  // Core data
  agents: AgentMemory[];
  memoryEntries: Record<string, MemoryEntry>; // keyed by memoryId
  knowledgeGraphs: Record<string, KnowledgeGraph>; // keyed by agentId
  contextSnapshots: ContextSnapshot[];
  memoryHealth: Record<string, MemoryHealthMetrics>; // keyed by agentId
  memoryAlerts: MemoryAlert[];
  memoryOptimizations: MemoryOptimization[];
  learningMetrics: Record<string, AgentLearningMetrics>; // keyed by agentId

  // UI state
  selectedAgent: AgentMemory | null;
  selectedMemory: MemoryEntry | null;
  selectedSnapshot: ContextSnapshot | null;
  currentQuery: MemoryQuery | null;
  searchResults: MemorySearchResult | null;
  graphLayout: 'force' | 'hierarchical' | 'circular';
  activeOptimizations: MemoryOptimization[];

  // Loading states
  loading: {
    agents: boolean;
    memories: boolean;
    knowledgeGraph: boolean;
    context: boolean;
    health: boolean;
    search: boolean;
    optimization: boolean;
    learning: boolean;
  };

  // Error states
  errors: {
    agents: string | null;
    memories: string | null;
    knowledgeGraph: string | null;
    context: string | null;
    health: string | null;
    search: string | null;
    optimization: string | null;
    learning: string | null;
  };

  // Pagination and filtering
  pagination: {
    memoriesPage: number;
    snapshotsPage: number;
    alertsPage: number;
    optimizationsPage: number;
    limit: number;
  };

  filters: {
    agentType: AgentMemory['type'][] | null;
    memoryType: MemoryEntry['type'][] | null;
    memoryImportance: { min: number; max: number } | null;
    memoryConfidence: { min: number; max: number } | null;
    alertSeverity: MemoryAlert['severity'][] | null;
    alertStatus: 'active' | 'acknowledged' | 'resolved' | null;
    optimizationType: MemoryOptimization['type'][] | null;
    optimizationStatus: MemoryOptimization['status'][] | null;
  };

  // Settings
  settings: {
    autoRefresh: boolean;
    refreshInterval: number; // seconds
    graphMaxNodes: number;
    graphMinImportance: number;
    showRelationships: boolean;
    enableAnimations: boolean;
    defaultLayout: 'force' | 'hierarchical' | 'circular';
  };
}

interface AgentMemoryActions {
  // Core data actions
  setAgents: (agents: AgentMemory[]) => void;
  addAgent: (agent: AgentMemory) => void;
  updateAgent: (agentId: string, updates: Partial<AgentMemory>) => void;
  removeAgent: (agentId: string) => void;
  setMemoryEntries: (entries: MemoryEntry[]) => void;
  addMemoryEntry: (entry: MemoryEntry) => void;
  updateMemoryEntry: (memoryId: string, updates: Partial<MemoryEntry>) => void;
  removeMemoryEntry: (memoryId: string) => void;
  setKnowledgeGraphs: (graphs: Record<string, KnowledgeGraph>) => void;
  updateKnowledgeGraph: (agentId: string, graph: KnowledgeGraph) => void;
  setContextSnapshots: (snapshots: ContextSnapshot[]) => void;
  addContextSnapshot: (snapshot: ContextSnapshot) => void;
  updateContextSnapshot: (snapshotId: string, updates: Partial<ContextSnapshot>) => void;
  removeContextSnapshot: (snapshotId: string) => void;
  setMemoryHealth: (agentId: string, health: MemoryHealthMetrics) => void;
  setMemoryAlerts: (alerts: MemoryAlert[]) => void;
  addMemoryAlert: (alert: MemoryAlert) => void;
  updateMemoryAlert: (alertId: string, updates: Partial<MemoryAlert>) => void;
  setMemoryOptimizations: (optimizations: MemoryOptimization[]) => void;
  addMemoryOptimization: (optimization: MemoryOptimization) => void;
  updateMemoryOptimization: (optimizationId: string, updates: Partial<MemoryOptimization>) => void;
  setLearningMetrics: (agentId: string, metrics: AgentLearningMetrics) => void;
  setSearchResults: (results: MemorySearchResult | null) => void;

  // UI state actions
  setSelectedAgent: (agent: AgentMemory | null) => void;
  setSelectedMemory: (memory: MemoryEntry | null) => void;
  setSelectedSnapshot: (snapshot: ContextSnapshot | null) => void;
  setCurrentQuery: (query: MemoryQuery | null) => void;
  setGraphLayout: (layout: 'force' | 'hierarchical' | 'circular') => void;
  addActiveOptimization: (optimization: MemoryOptimization) => void;
  removeActiveOptimization: (optimizationId: string) => void;

  // Loading actions
  setLoading: (key: keyof AgentMemoryState['loading'], loading: boolean) => void;
  setError: (key: keyof AgentMemoryState['errors'], error: string | null) => void;
  clearErrors: () => void;

  // Pagination actions
  setPagination: (pagination: Partial<AgentMemoryState['pagination']>) => void;
  nextMemoriesPage: () => void;
  nextSnapshotsPage: () => void;
  nextAlertsPage: () => void;
  nextOptimizationsPage: () => void;
  resetPagination: () => void;

  // Filter actions
  setFilters: (filters: Partial<AgentMemoryState['filters']>) => void;
  clearFilters: () => void;

  // Settings actions
  updateSettings: (settings: Partial<AgentMemoryState['settings']>) => void;

  // Utility actions
  reset: () => void;
}

const initialState: AgentMemoryState = {
  agents: [],
  memoryEntries: {},
  knowledgeGraphs: {},
  contextSnapshots: [],
  memoryHealth: {},
  memoryAlerts: [],
  memoryOptimizations: [],
  learningMetrics: {},
  selectedAgent: null,
  selectedMemory: null,
  selectedSnapshot: null,
  currentQuery: null,
  searchResults: null,
  graphLayout: 'force',
  activeOptimizations: [],
  loading: {
    agents: false,
    memories: false,
    knowledgeGraph: false,
    context: false,
    health: false,
    search: false,
    optimization: false,
    learning: false,
  },
  errors: {
    agents: null,
    memories: null,
    knowledgeGraph: null,
    context: null,
    health: null,
    search: null,
    optimization: null,
    learning: null,
  },
  pagination: {
    memoriesPage: 1,
    snapshotsPage: 1,
    alertsPage: 1,
    optimizationsPage: 1,
    limit: 50,
  },
  filters: {
    agentType: null,
    memoryType: null,
    memoryImportance: null,
    memoryConfidence: null,
    alertSeverity: null,
    alertStatus: null,
    optimizationType: null,
    optimizationStatus: null,
  },
  settings: {
    autoRefresh: true,
    refreshInterval: 30,
    graphMaxNodes: 1000,
    graphMinImportance: 0.1,
    showRelationships: true,
    enableAnimations: true,
    defaultLayout: 'force',
  },
};

export const useAgentMemoryStore = create<AgentMemoryState & AgentMemoryActions>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // Core data actions
      setAgents: (agents) => set({ agents }),
      addAgent: (agent) => set((state) => ({
        agents: [agent, ...state.agents]
      })),
      updateAgent: (agentId, updates) => set((state) => ({
        agents: state.agents.map(agent =>
          agent.id === agentId ? { ...agent, ...updates } : agent
        ),
        selectedAgent: state.selectedAgent?.id === agentId
          ? { ...state.selectedAgent, ...updates }
          : state.selectedAgent
      })),
      removeAgent: (agentId) => set((state) => ({
        agents: state.agents.filter(agent => agent.id !== agentId),
        selectedAgent: state.selectedAgent?.id === agentId ? null : state.selectedAgent
      })),
      setMemoryEntries: (entries) => set((state) => ({
        memoryEntries: entries.reduce((acc, entry) => {
          acc[entry.id] = entry;
          return acc;
        }, {} as Record<string, MemoryEntry>)
      })),
      addMemoryEntry: (entry) => set((state) => ({
        memoryEntries: { ...state.memoryEntries, [entry.id]: entry }
      })),
      updateMemoryEntry: (memoryId, updates) => set((state) => ({
        memoryEntries: {
          ...state.memoryEntries,
          [memoryId]: { ...state.memoryEntries[memoryId], ...updates }
        },
        selectedMemory: state.selectedMemory?.id === memoryId
          ? { ...state.selectedMemory, ...updates }
          : state.selectedMemory
      })),
      removeMemoryEntry: (memoryId) => set((state) => {
        const newEntries = { ...state.memoryEntries };
        delete newEntries[memoryId];
        return {
          memoryEntries: newEntries,
          selectedMemory: state.selectedMemory?.id === memoryId ? null : state.selectedMemory
        };
      }),
      setKnowledgeGraphs: (graphs) => set({ knowledgeGraphs: graphs }),
      updateKnowledgeGraph: (agentId, graph) => set((state) => ({
        knowledgeGraphs: { ...state.knowledgeGraphs, [agentId]: graph }
      })),
      setContextSnapshots: (snapshots) => set({ contextSnapshots: snapshots }),
      addContextSnapshot: (snapshot) => set((state) => ({
        contextSnapshots: [snapshot, ...state.contextSnapshots]
      })),
      updateContextSnapshot: (snapshotId, updates) => set((state) => ({
        contextSnapshots: state.contextSnapshots.map(snapshot =>
          snapshot.id === snapshotId ? { ...snapshot, ...updates } : snapshot
        ),
        selectedSnapshot: state.selectedSnapshot?.id === snapshotId
          ? { ...state.selectedSnapshot, ...updates }
          : state.selectedSnapshot
      })),
      removeContextSnapshot: (snapshotId) => set((state) => ({
        contextSnapshots: state.contextSnapshots.filter(snapshot => snapshot.id !== snapshotId),
        selectedSnapshot: state.selectedSnapshot?.id === snapshotId ? null : state.selectedSnapshot
      })),
      setMemoryHealth: (agentId, health) => set((state) => ({
        memoryHealth: { ...state.memoryHealth, [agentId]: health }
      })),
      setMemoryAlerts: (alerts) => set({ memoryAlerts: alerts }),
      addMemoryAlert: (alert) => set((state) => ({
        memoryAlerts: [alert, ...state.memoryAlerts]
      })),
      updateMemoryAlert: (alertId, updates) => set((state) => ({
        memoryAlerts: state.memoryAlerts.map(alert =>
          alert.id === alertId ? { ...alert, ...updates } : alert
        )
      })),
      setMemoryOptimizations: (optimizations) => set({ memoryOptimizations: optimizations }),
      addMemoryOptimization: (optimization) => set((state) => ({
        memoryOptimizations: [optimization, ...state.memoryOptimizations],
        activeOptimizations: optimization.status === 'running' || optimization.status === 'pending'
          ? [...state.activeOptimizations, optimization]
          : state.activeOptimizations
      })),
      updateMemoryOptimization: (optimizationId, updates) => set((state) => {
        const updated = state.memoryOptimizations.map(opt =>
          opt.id === optimizationId ? { ...opt, ...updates } : opt
        );
        const updatedActive = state.activeOptimizations.map(opt =>
          opt.id === optimizationId ? { ...opt, ...updates } : opt
        ).filter(opt => opt.status === 'running' || opt.status === 'pending');

        return {
          memoryOptimizations: updated,
          activeOptimizations: updatedActive
        };
      }),
      setLearningMetrics: (agentId, metrics) => set((state) => ({
        learningMetrics: { ...state.learningMetrics, [agentId]: metrics }
      })),
      setSearchResults: (results) => set({ searchResults: results }),

      // UI state actions
      setSelectedAgent: (agent) => set({ selectedAgent: agent }),
      setSelectedMemory: (memory) => set({ selectedMemory: memory }),
      setSelectedSnapshot: (snapshot) => set({ selectedSnapshot: snapshot }),
      setCurrentQuery: (query) => set({ currentQuery: query }),
      setGraphLayout: (layout) => set({ graphLayout: layout }),
      addActiveOptimization: (optimization) => set((state) => ({
        activeOptimizations: [...state.activeOptimizations, optimization]
      })),
      removeActiveOptimization: (optimizationId) => set((state) => ({
        activeOptimizations: state.activeOptimizations.filter(opt => opt.id !== optimizationId)
      })),

      // Loading actions
      setLoading: (key, loading) => set((state) => ({
        loading: { ...state.loading, [key]: loading }
      })),
      setError: (key, error) => set((state) => ({
        errors: { ...state.errors, [key]: error }
      })),
      clearErrors: () => set({ errors: initialState.errors }),

      // Pagination actions
      setPagination: (pagination) => set((state) => ({
        pagination: { ...state.pagination, ...pagination }
      })),
      nextMemoriesPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          memoriesPage: state.pagination.memoriesPage + 1
        }
      })),
      nextSnapshotsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          snapshotsPage: state.pagination.snapshotsPage + 1
        }
      })),
      nextAlertsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          alertsPage: state.pagination.alertsPage + 1
        }
      })),
      nextOptimizationsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          optimizationsPage: state.optimizationsPage + 1
        }
      })),
      resetPagination: () => set({ pagination: initialState.pagination }),

      // Filter actions
      setFilters: (filters) => set((state) => ({
        filters: { ...state.filters, ...filters }
      })),
      clearFilters: () => set({ filters: initialState.filters }),

      // Settings actions
      updateSettings: (settings) => set((state) => ({
        settings: { ...state.settings, ...settings }
      })),

      // Utility actions
      reset: () => set(initialState),
    }),
    {
      name: 'agent-memory-store',
    }
  )
);

// Selector hooks for better performance
export const useAgents = () => useAgentMemoryStore((state) => state.agents);
export const useMemoryEntries = () => useAgentMemoryStore((state) => state.memoryEntries);
export const useKnowledgeGraphs = () => useAgentMemoryStore((state) => state.knowledgeGraphs);
export const useContextSnapshots = () => useAgentMemoryStore((state) => state.contextSnapshots);
export const useMemoryAlerts = () => useAgentMemoryStore((state) => state.memoryAlerts);
export const useMemoryOptimizations = () => useAgentMemoryStore((state) => state.memoryOptimizations);
export const useSelectedAgent = () => useAgentMemoryStore((state) => state.selectedAgent);
export const useSelectedMemory = () => useAgentMemoryStore((state) => state.selectedMemory);
export const useSelectedSnapshot = () => useAgentMemoryStore((state) => state.selectedSnapshot);
export const useSearchResults = () => useAgentMemoryStore((state) => state.searchResults);
export const useGraphLayout = () => useAgentMemoryStore((state) => state.graphLayout);
export const useActiveOptimizations = () => useAgentMemoryStore((state) => state.activeOptimizations);
export const useAgentMemoryLoading = () => useAgentMemoryStore((state) => state.loading);
export const useAgentMemoryErrors = () => useAgentMemoryStore((state) => state.errors);

// Computed selectors
export const useAgentHealthSummary = () => useAgentMemoryStore((state) => {
  return state.agents.map(agent => ({
    agent,
    health: state.memoryHealth[agent.id],
    alertCount: state.memoryAlerts.filter(alert =>
      alert.agentId === agent.id && !alert.acknowledged && !alert.resolved
    ).length,
    optimizationCount: state.activeOptimizations.filter(opt => opt.agentId === agent.id).length,
  }));
});

export const useMemoryStats = () => useAgentMemoryStore((state) => {
  const entries = Object.values(state.memoryEntries);
  return {
    total: entries.length,
    byType: entries.reduce((acc, entry) => {
      acc[entry.type] = (acc[entry.type] || 0) + 1;
      return acc;
    }, {} as Record<string, number>),
    averageImportance: entries.length > 0
      ? entries.reduce((sum, entry) => sum + entry.metadata.importance, 0) / entries.length
      : 0,
    averageConfidence: entries.length > 0
      ? entries.reduce((sum, entry) => sum + entry.metadata.confidence, 0) / entries.length
      : 0,
    totalSize: entries.reduce((sum, entry) => sum + entry.size, 0),
    compressedCount: entries.filter(entry => entry.compressed).length,
  };
});

export const useKnowledgeGraphStats = () => useAgentMemoryStore((state) => {
  return Object.entries(state.knowledgeGraphs).reduce((acc, [agentId, graph]) => {
    acc[agentId] = {
      nodes: graph.nodes.length,
      edges: graph.edges.length,
      averageConnectivity: graph.nodes.length > 0
        ? (graph.edges.length * 2) / graph.nodes.length
        : 0,
      clusteringCoefficient: graph.metadata.clusteringCoefficient,
    };
    return acc;
  }, {} as Record<string, {
    nodes: number;
    edges: number;
    averageConnectivity: number;
    clusteringCoefficient: number;
  }>);
});

export const useMemoryAlertStats = () => useAgentMemoryStore((state) => {
  const alerts = state.memoryAlerts;
  return {
    total: alerts.length,
    active: alerts.filter(a => !a.acknowledged && !a.resolved).length,
    acknowledged: alerts.filter(a => a.acknowledged && !a.resolved).length,
    resolved: alerts.filter(a => a.resolved).length,
    bySeverity: {
      low: alerts.filter(a => a.severity === 'low').length,
      medium: alerts.filter(a => a.severity === 'medium').length,
      high: alerts.filter(a => a.severity === 'high').length,
      critical: alerts.filter(a => a.severity === 'critical').length,
    },
    byType: {
      memory_pressure: alerts.filter(a => a.type === 'memory_pressure').length,
      consistency_violation: alerts.filter(a => a.type === 'consistency_violation').length,
      fragmentation: alerts.filter(a => a.type === 'fragmentation').length,
      access_latency: alerts.filter(a => a.type === 'access_latency').length,
      size_limit: alerts.filter(a => a.type === 'size_limit').length,
    },
  };
});

export const useOptimizationStats = () => useAgentMemoryStore((state) => {
  const optimizations = state.memoryOptimizations;
  return {
    total: optimizations.length,
    running: optimizations.filter(o => o.status === 'running').length,
    completed: optimizations.filter(o => o.status === 'completed').length,
    failed: optimizations.filter(o => o.status === 'failed').length,
    pending: optimizations.filter(o => o.status === 'pending').length,
    byType: {
      compression: optimizations.filter(o => o.type === 'compression').length,
      consolidation: optimizations.filter(o => o.type === 'consolidation').length,
      cleanup: optimizations.filter(o => o.type === 'cleanup').length,
      defragmentation: optimizations.filter(o => o.type === 'defragmentation').length,
      reindexing: optimizations.filter(o => o.type === 'reindexing').length,
    },
    totalSpaceSaved: optimizations
      .filter(o => o.results)
      .reduce((sum, o) => sum + (o.results?.spaceSaved || 0), 0),
  };
});

export const useLearningProgress = () => useAgentMemoryStore((state) => {
  return Object.entries(state.learningMetrics).map(([agentId, metrics]) => ({
    agentId,
    period: metrics.period,
    growth: metrics.metrics.knowledgeGrowth,
    efficiency: metrics.metrics.learningEfficiency,
    retention: metrics.metrics.memoryRetention,
    adaptation: metrics.metrics.adaptationRate,
    insights: metrics.insights,
  }));
});

export const useRecentMemoryActivity = () => useAgentMemoryStore((state) => {
  const entries = Object.values(state.memoryEntries);
  return entries
    .sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime())
    .slice(0, 10)
    .map(entry => ({
      entry,
      agent: state.agents.find(a => a.id === entry.agentId),
    }));
});

export const useMemoryPressureIndicators = () => useAgentMemoryStore((state) => {
  return Object.entries(state.memoryHealth).map(([agentId, health]) => ({
    agentId,
    pressure: health.metrics.memoryPressure,
    fragmentation: health.metrics.fragmentation,
    accessEfficiency: health.metrics.accessEfficiency,
    alerts: state.memoryAlerts.filter(alert =>
      alert.agentId === agentId &&
      !alert.resolved &&
      ['memory_pressure', 'fragmentation'].includes(alert.type)
    ),
  }));
});

export const useAgentMemoryActions = () => useAgentMemoryStore((state) => ({
  setAgents: state.setAgents,
  addAgent: state.addAgent,
  updateAgent: state.updateAgent,
  removeAgent: state.removeAgent,
  setMemoryEntries: state.setMemoryEntries,
  addMemoryEntry: state.addMemoryEntry,
  updateMemoryEntry: state.updateMemoryEntry,
  removeMemoryEntry: state.removeMemoryEntry,
  setKnowledgeGraphs: state.setKnowledgeGraphs,
  updateKnowledgeGraph: state.updateKnowledgeGraph,
  setContextSnapshots: state.setContextSnapshots,
  addContextSnapshot: state.addContextSnapshot,
  updateContextSnapshot: state.updateContextSnapshot,
  removeContextSnapshot: state.removeContextSnapshot,
  setMemoryHealth: state.setMemoryHealth,
  setMemoryAlerts: state.setMemoryAlerts,
  addMemoryAlert: state.addMemoryAlert,
  updateMemoryAlert: state.updateMemoryAlert,
  setMemoryOptimizations: state.setMemoryOptimizations,
  addMemoryOptimization: state.addMemoryOptimization,
  updateMemoryOptimization: state.updateMemoryOptimization,
  setLearningMetrics: state.setLearningMetrics,
  setSearchResults: state.setSearchResults,
  setSelectedAgent: state.setSelectedAgent,
  setSelectedMemory: state.setSelectedMemory,
  setSelectedSnapshot: state.setSelectedSnapshot,
  setCurrentQuery: state.setCurrentQuery,
  setGraphLayout: state.setGraphLayout,
  addActiveOptimization: state.addActiveOptimization,
  removeActiveOptimization: state.removeActiveOptimization,
  setLoading: state.setLoading,
  setError: state.setError,
  clearErrors: state.clearErrors,
  setPagination: state.setPagination,
  nextMemoriesPage: state.nextMemoriesPage,
  nextSnapshotsPage: state.nextSnapshotsPage,
  nextAlertsPage: state.nextAlertsPage,
  nextOptimizationsPage: state.nextOptimizationsPage,
  resetPagination: state.resetPagination,
  setFilters: state.setFilters,
  clearFilters: state.clearFilters,
  updateSettings: state.updateSettings,
  reset: state.reset,
}));
