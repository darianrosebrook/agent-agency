/**
 * Vector Database Store
 * Zustand store for vector database state management
 *
 * @author @darianrosebrook
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { VectorEmbedding, VectorSearchResult, VectorCluster, VectorAnalytics } from '@/lib/vector-database-api';

interface VectorDatabaseState {
  // Vector data
  vectors: VectorEmbedding[];
  searchResults: VectorSearchResult[];
  selectedVector: VectorEmbedding | null;
  
  // Clusters
  clusters: VectorCluster[];
  selectedCluster: VectorCluster | null;
  
  // Analytics
  analytics: VectorAnalytics | null;
  performanceMetrics: {
    searchLatency: number[];
    indexingTime: number[];
    memoryUsage: number[];
    queryThroughput: number[];
    timestamps: Date[];
  } | null;
  
  // UI state
  loading: {
    vectors: boolean;
    search: boolean;
    clusters: boolean;
    analytics: boolean;
    performance: boolean;
  };
  
  errors: {
    vectors: string | null;
    search: string | null;
    clusters: string | null;
    analytics: string | null;
    performance: string | null;
  };
  
  // Filters and search
  searchQuery: string;
  searchFilters: {
    category?: string;
    model?: string;
    dateRange?: {
      start: Date;
      end: Date;
    };
  };
  
  // Pagination
  pagination: {
    page: number;
    limit: number;
    total: number;
  };
  
  // View settings
  viewSettings: {
    projection: 'pca' | 'tsne' | 'umap' | 'custom';
    dimensions: 2 | 3;
    showClusters: boolean;
    showConnections: boolean;
    clusterOpacity: number;
  };
}

interface VectorDatabaseActions {
  // Vector actions
  setVectors: (vectors: VectorEmbedding[]) => void;
  addVector: (vector: VectorEmbedding) => void;
  updateVector: (id: string, updates: Partial<VectorEmbedding>) => void;
  deleteVector: (id: string) => void;
  setSelectedVector: (vector: VectorEmbedding | null) => void;
  
  // Search actions
  setSearchResults: (results: VectorSearchResult[]) => void;
  setSearchQuery: (query: string) => void;
  setSearchFilters: (filters: VectorDatabaseState['searchFilters']) => void;
  clearSearch: () => void;
  
  // Cluster actions
  setClusters: (clusters: VectorCluster[]) => void;
  addCluster: (cluster: VectorCluster) => void;
  updateCluster: (id: string, updates: Partial<VectorCluster>) => void;
  deleteCluster: (id: string) => void;
  setSelectedCluster: (cluster: VectorCluster | null) => void;
  
  // Analytics actions
  setAnalytics: (analytics: VectorAnalytics) => void;
  setPerformanceMetrics: (metrics: VectorDatabaseState['performanceMetrics']) => void;
  
  // Loading actions
  setLoading: (key: keyof VectorDatabaseState['loading'], loading: boolean) => void;
  setError: (key: keyof VectorDatabaseState['errors'], error: string | null) => void;
  clearErrors: () => void;
  
  // Pagination actions
  setPagination: (pagination: Partial<VectorDatabaseState['pagination']>) => void;
  nextPage: () => void;
  prevPage: () => void;
  
  // View settings actions
  setViewSettings: (settings: Partial<VectorDatabaseState['viewSettings']>) => void;
  
  // Utility actions
  reset: () => void;
}

const initialState: VectorDatabaseState = {
  vectors: [],
  searchResults: [],
  selectedVector: null,
  clusters: [],
  selectedCluster: null,
  analytics: null,
  performanceMetrics: null,
  loading: {
    vectors: false,
    search: false,
    clusters: false,
    analytics: false,
    performance: false,
  },
  errors: {
    vectors: null,
    search: null,
    clusters: null,
    analytics: null,
    performance: null,
  },
  searchQuery: '',
  searchFilters: {},
  pagination: {
    page: 1,
    limit: 20,
    total: 0,
  },
  viewSettings: {
    projection: 'pca',
    dimensions: 3,
    showClusters: true,
    showConnections: true,
    clusterOpacity: 0.3,
  },
};

export const useVectorDatabaseStore = create<VectorDatabaseState & VectorDatabaseActions>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // Vector actions
      setVectors: (vectors) => set({ vectors }),
      addVector: (vector) => set((state) => ({ vectors: [...state.vectors, vector] })),
      updateVector: (id, updates) => set((state) => ({
        vectors: state.vectors.map(v => v.id === id ? { ...v, ...updates } : v)
      })),
      deleteVector: (id) => set((state) => ({
        vectors: state.vectors.filter(v => v.id !== id),
        selectedVector: state.selectedVector?.id === id ? null : state.selectedVector
      })),
      setSelectedVector: (vector) => set({ selectedVector: vector }),

      // Search actions
      setSearchResults: (results) => set({ searchResults: results }),
      setSearchQuery: (query) => set({ searchQuery: query }),
      setSearchFilters: (filters) => set({ searchFilters: filters }),
      clearSearch: () => set({ searchResults: [], searchQuery: '', searchFilters: {} }),

      // Cluster actions
      setClusters: (clusters) => set({ clusters }),
      addCluster: (cluster) => set((state) => ({ clusters: [...state.clusters, cluster] })),
      updateCluster: (id, updates) => set((state) => ({
        clusters: state.clusters.map(c => c.id === id ? { ...c, ...updates } : c)
      })),
      deleteCluster: (id) => set((state) => ({
        clusters: state.clusters.filter(c => c.id !== id),
        selectedCluster: state.selectedCluster?.id === id ? null : state.selectedCluster
      })),
      setSelectedCluster: (cluster) => set({ selectedCluster: cluster }),

      // Analytics actions
      setAnalytics: (analytics) => set({ analytics }),
      setPerformanceMetrics: (metrics) => set({ performanceMetrics: metrics }),

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
      nextPage: () => set((state) => ({
        pagination: { ...state.pagination, page: state.pagination.page + 1 }
      })),
      prevPage: () => set((state) => ({
        pagination: { ...state.pagination, page: Math.max(1, state.pagination.page - 1) }
      })),

      // View settings actions
      setViewSettings: (settings) => set((state) => ({
        viewSettings: { ...state.viewSettings, ...settings }
      })),

      // Utility actions
      reset: () => set(initialState),
    }),
    {
      name: 'vector-database-store',
    }
  )
);

// Selector hooks for better performance
export const useVectors = () => useVectorDatabaseStore((state) => state.vectors);
export const useSearchResults = () => useVectorDatabaseStore((state) => state.searchResults);
export const useSelectedVector = () => useVectorDatabaseStore((state) => state.selectedVector);
export const useClusters = () => useVectorDatabaseStore((state) => state.clusters);
export const useSelectedCluster = () => useVectorDatabaseStore((state) => state.selectedCluster);
export const useAnalytics = () => useVectorDatabaseStore((state) => state.analytics);
export const usePerformanceMetrics = () => useVectorDatabaseStore((state) => state.performanceMetrics);
export const useVectorDatabaseLoading = () => useVectorDatabaseStore((state) => state.loading);
export const useVectorDatabaseErrors = () => useVectorDatabaseStore((state) => state.errors);
export const useVectorDatabaseActions = () => useVectorDatabaseStore((state) => ({
  setVectors: state.setVectors,
  addVector: state.addVector,
  updateVector: state.updateVector,
  deleteVector: state.deleteVector,
  setSelectedVector: state.setSelectedVector,
  setSearchResults: state.setSearchResults,
  setSearchQuery: state.setSearchQuery,
  setSearchFilters: state.setSearchFilters,
  clearSearch: state.clearSearch,
  setClusters: state.setClusters,
  addCluster: state.addCluster,
  updateCluster: state.updateCluster,
  deleteCluster: state.deleteCluster,
  setSelectedCluster: state.setSelectedCluster,
  setAnalytics: state.setAnalytics,
  setPerformanceMetrics: state.setPerformanceMetrics,
  setLoading: state.setLoading,
  setError: state.setError,
  clearErrors: state.clearErrors,
  setPagination: state.setPagination,
  nextPage: state.nextPage,
  prevPage: state.prevPage,
  setViewSettings: state.setViewSettings,
  reset: state.reset,
}));
