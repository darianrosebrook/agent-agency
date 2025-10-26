/**
 * Vector Database Dashboard
 * Main dashboard for vector database analytics and management
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { 
  MetricCard, 
  AnalyticsGrid, 
  Vector3DVisualization 
} from '@/design-system/analytics';
import { 
  Database, 
  Search, 
  BarChart3, 
  TrendingUp, 
  Zap, 
  Activity,
  RefreshCw,
  Settings,
  Download,
  Upload,
  Filter,
  Grid3X3
} from 'lucide-react';
import { vectorDatabaseApiClient } from '@/lib/vector-database-api';
import { mockDataService } from '@/lib/mock-data-service';
import { useVectorDatabaseStore, useVectorDatabaseActions } from '@/stores/vector-database';
import { useVectorDatabaseWebSocket, useRealTimeVectorMonitoring } from '@/hooks/useVectorDatabaseWebSocket';
import styles from './VectorDatabaseDashboard.module.scss';

export function VectorDatabaseDashboard() {
  const [viewMode, setViewMode] = useState<'overview' | 'vectors' | 'search' | 'analytics'>('overview');
  const [refreshing, setRefreshing] = useState(false);

  // Store state
  const { analytics, performanceMetrics, loading, errors } = useVectorDatabaseStore();
  const actions = useVectorDatabaseActions();
  const { isConnected } = useVectorDatabaseWebSocket();
  const { vectors, clusters, totalVectors, clusterCount, averageDimensions } = useRealTimeVectorMonitoring();

  // Fetch initial data
  useEffect(() => {
    fetchDashboardData();
  }, []);

  const fetchDashboardData = async () => {
    try {
      setRefreshing(true);
      actions.clearErrors();

      // Use mock data service for development
      const analyticsData = mockDataService.getAnalytics();
      actions.setAnalytics(analyticsData);

      const performanceData = mockDataService.getPerformanceMetrics('24h');
      actions.setPerformanceMetrics(performanceData);

      const vectorsData = mockDataService.getEmbeddingsWithProjections();
      actions.setVectors(vectorsData);

      const clustersData = mockDataService.getClustersWith3DCenters();
      actions.setClusters(clustersData);

    } catch (error) {
      console.error('Failed to fetch vector database dashboard data:', error);
      actions.setError('analytics', error instanceof Error ? error.message : 'Failed to fetch data');
    } finally {
      actions.setLoading('analytics', false);
      actions.setLoading('performance', false);
      actions.setLoading('vectors', false);
      actions.setLoading('clusters', false);
      setRefreshing(false);
    }
  };

  const handleRefresh = async () => {
    await fetchDashboardData();
  };

  const handleVectorClick = (vector: any) => {
    actions.setSelectedVector(vector);
    console.log('Vector clicked:', vector);
  };

  const handleVectorHover = (vector: any) => {
    console.log('Vector hovered:', vector);
  };

  const handleSearch = async (query: string) => {
    try {
      actions.setLoading('search', true);
      actions.setError('search', null);

      // Use mock data service for search
      const results = mockDataService.searchVectors(query, 20, 0.7);
      actions.setSearchResults(results);
    } catch (error) {
      console.error('Search failed:', error);
      actions.setError('search', error instanceof Error ? error.message : 'Search failed');
    } finally {
      actions.setLoading('search', false);
    }
  };

  const handleExport = async () => {
    try {
      const blob = await vectorDatabaseApiClient.exportVectors('json');
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `vector-database-export-${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Export failed:', error);
    }
  };

  const handleImport = async (file: File) => {
    try {
      const result = await vectorDatabaseApiClient.importVectors(file);
      console.log('Import result:', result);
      await fetchDashboardData(); // Refresh data
    } catch (error) {
      console.error('Import failed:', error);
    }
  };

  // Use real data from store or fallback to mock
  const displayVectors = vectors.length > 0 ? vectors : mockDataService.getEmbeddingsWithProjections();
  const displayClusters = clusters.length > 0 ? clusters : mockDataService.getClustersWith3DCenters();

  const mockMetrics = [
    {
      title: 'Total Vectors',
      value: analytics?.totalVectors.toLocaleString() || '2.4M',
      subtitle: 'Embeddings in database',
      change: { value: 12.5, type: 'increase' as const, period: 'vs last month' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <Database size={20} />
    },
    {
      title: 'Search Latency',
      value: `${analytics?.performanceMetrics.searchLatency.toFixed(1) || '23.5'}ms`,
      subtitle: 'Average response time',
      change: { value: -8.3, type: 'decrease' as const, period: 'vs last week' },
      status: 'good' as const,
      trend: 'down' as const,
      icon: <Zap size={20} />
    },
    {
      title: 'Query Throughput',
      value: `${analytics?.performanceMetrics.queryThroughput.toFixed(1) || '1.2k'}/s`,
      subtitle: 'Queries per second',
      change: { value: 15.2, type: 'increase' as const, period: 'vs last week' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <Activity size={20} />
    },
    {
      title: 'Cluster Count',
      value: clusterCount.toString(),
      subtitle: 'Active clusters',
      change: { value: 2, type: 'increase' as const, period: 'vs last month' },
      status: 'neutral' as const,
      trend: 'up' as const,
      icon: <Grid3X3 size={20} />
    }
  ];

  return (
    <div className={styles.vectorDatabaseDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Vector Database</Text>
          <Text variant="paragraph-large" color="secondary">
            High-dimensional vector analytics and neural search
          </Text>
        </div>

        <div className={styles.headerRight}>
          {/* Connection Status */}
          <div className={styles.connectionStatus}>
            {isConnected ? (
              <div className={styles.connected}>
                <Activity size={12} />
                <span>Live</span>
              </div>
            ) : (
              <div className={styles.disconnected}>
                <Database size={12} />
                <span>Offline</span>
              </div>
            )}
          </div>

          {/* View Mode Tabs */}
          <div className={styles.viewModeTabs}>
            <Button
              variant={viewMode === 'overview' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setViewMode('overview')}
            >
              <BarChart3 size={16} />
              Overview
            </Button>
            <Button
              variant={viewMode === 'vectors' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setViewMode('vectors')}
            >
              <Database size={16} />
              Vectors
            </Button>
            <Button
              variant={viewMode === 'search' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setViewMode('search')}
            >
              <Search size={16} />
              Search
            </Button>
            <Button
              variant={viewMode === 'analytics' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setViewMode('analytics')}
            >
              <TrendingUp size={16} />
              Analytics
            </Button>
          </div>

          {/* Actions */}
          <div className={styles.actions}>
            <Button variant="secondary" size="sm">
              <Settings size={16} />
            </Button>
            <Button variant="secondary" size="sm" onClick={handleExport}>
              <Download size={16} />
            </Button>
            <Button variant="secondary" size="sm">
              <Upload size={16} />
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={handleRefresh}
              disabled={refreshing}
            >
              <RefreshCw size={16} className={refreshing ? styles.spinning : ''} />
            </Button>
          </div>
        </div>
      </div>

      {/* Overview Mode */}
      {viewMode === 'overview' && (
        <div className={styles.overview}>
          <AnalyticsGrid
            title="Vector Database Metrics"
            subtitle="Real-time performance and capacity indicators"
            columns={4}
            gap="md"
          >
            {mockMetrics.map((metric, index) => (
              <MetricCard
                key={index}
                title={metric.title}
                value={metric.value}
                subtitle={metric.subtitle}
                change={metric.change}
                status={metric.status}
                trend={metric.trend}
                icon={metric.icon}
                size="medium"
              />
            ))}
          </AnalyticsGrid>

          <div className={styles.visualizationSection}>
            <Vector3DVisualization
              title="3D Vector Space"
              subtitle="High-dimensional embedding visualization"
              vectors={displayVectors}
              clusters={displayClusters}
              projection="pca"
              onVectorClick={handleVectorClick}
              onVectorHover={handleVectorHover}
            />
          </div>
        </div>
      )}

      {/* Vectors Mode */}
      {viewMode === 'vectors' && (
        <div className={styles.vectors}>
          <Vector3DVisualization
            title="Vector Space Explorer"
            subtitle="Interactive 3D visualization of high-dimensional embeddings"
            vectors={displayVectors}
            clusters={displayClusters}
            projection="pca"
            onVectorClick={handleVectorClick}
            onVectorHover={handleVectorHover}
          />
        </div>
      )}

      {/* Search Mode */}
      {viewMode === 'search' && (
        <div className={styles.search}>
          <div className={styles.searchInterface}>
            <Text variant="h3">Vector Search</Text>
            <Text variant="paragraph-medium" color="secondary">
              Search for similar vectors using semantic similarity
            </Text>
            
            <div className={styles.searchInput}>
              <input
                type="text"
                placeholder="Enter search query..."
                className={styles.input}
                onChange={(e) => actions.setSearchQuery(e.target.value)}
              />
              <Button onClick={() => handleSearch(actions.searchQuery)}>
                <Search size={16} />
                Search
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Analytics Mode */}
      {viewMode === 'analytics' && (
        <div className={styles.analytics}>
          <Text variant="h3">Advanced Analytics</Text>
          <Text variant="paragraph-medium" color="secondary">
            Detailed performance metrics and insights
          </Text>
        </div>
      )}
    </div>
  );
}
