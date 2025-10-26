/**
 * Demo Page
 * Showcase of enhanced analytics and vector database capabilities
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
  D3Visualization, 
  Vector3DVisualization 
} from '@/design-system/analytics';
import { mockDataService } from '@/lib/mock-data-service';
import { 
  BarChart3, 
  Database, 
  Zap, 
  Activity,
  TrendingUp,
  RefreshCw,
  Play,
  Pause,
  RotateCcw
} from 'lucide-react';
import styles from './page.module.scss';

export default function DemoPage() {
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentDataIndex, setCurrentDataIndex] = useState(0);
  const [refreshing, setRefreshing] = useState(false);

  // Generate multiple datasets for animation
  const datasets = Array.from({ length: 5 }, (_, i) => ({
    timeSeries: mockDataService.generateTimeSeriesData(20, 'latency'),
    scatter: mockDataService.generateScatterData(30 + i * 10),
    vectors: mockDataService.getEmbeddingsWithProjections().slice(0, 5 + i * 2),
    clusters: mockDataService.getClustersWith3DCenters().slice(0, 2 + i),
    metrics: [
      {
        title: 'Total Vectors',
        value: `${(1247 + i * 100).toLocaleString()}`,
        subtitle: 'Embeddings in database',
        change: { value: 12.5 + i * 2, type: 'increase' as const, period: 'vs last month' },
        status: 'good' as const,
        trend: 'up' as const,
        icon: <Database size={20} />
      },
      {
        title: 'Search Latency',
        value: `${(23.5 - i * 2).toFixed(1)}ms`,
        subtitle: 'Average response time',
        change: { value: -8.3 - i, type: 'decrease' as const, period: 'vs last week' },
        status: 'good' as const,
        trend: 'down' as const,
        icon: <Zap size={20} />
      },
      {
        title: 'Query Throughput',
        value: `${(1.2 + i * 0.1).toFixed(1)}k/s`,
        subtitle: 'Queries per second',
        change: { value: 15.2 + i * 3, type: 'increase' as const, period: 'vs last week' },
        status: 'good' as const,
        trend: 'up' as const,
        icon: <Activity size={20} />
      },
      {
        title: 'Accuracy',
        value: `${(94.2 + i * 0.5).toFixed(1)}%`,
        subtitle: 'Search precision',
        change: { value: 2.1 + i * 0.3, type: 'increase' as const, period: 'vs last month' },
        status: 'good' as const,
        trend: 'up' as const,
        icon: <TrendingUp size={20} />
      }
    ]
  }));

  const currentDataset = datasets[currentDataIndex] || datasets[0];

  // Auto-play animation
  useEffect(() => {
    if (isPlaying) {
      const interval = setInterval(() => {
        setCurrentDataIndex(prev => (prev + 1) % datasets.length);
      }, 2000);
      return () => clearInterval(interval);
    }
    return undefined;
  }, [isPlaying, datasets.length]);

  const handlePlayPause = () => {
    setIsPlaying(!isPlaying);
  };

  const handleReset = () => {
    setIsPlaying(false);
    setCurrentDataIndex(0);
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    // Simulate refresh
    await new Promise(resolve => setTimeout(resolve, 1000));
    setRefreshing(false);
  };

  return (
    <div className={styles.demoPage}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Analytics Demo</Text>
          <Text variant="paragraph-large" color="secondary">
            Interactive showcase of enhanced analytics and vector database capabilities
          </Text>
        </div>

        <div className={styles.headerRight}>
          <div className={styles.controls}>
            <Button
              variant={isPlaying ? 'primary' : 'secondary'}
              size="sm"
              onClick={handlePlayPause}
            >
              {isPlaying ? <Pause size={16} /> : <Play size={16} />}
              {isPlaying ? 'Pause' : 'Play'} Animation
            </Button>
            
            <Button
              variant="secondary"
              size="sm"
              onClick={handleReset}
            >
              <RotateCcw size={16} />
              Reset
            </Button>

            <Button
              variant="secondary"
              size="sm"
              onClick={handleRefresh}
              disabled={refreshing}
            >
              <RefreshCw size={16} className={refreshing ? styles.spinning : ''} />
              Refresh
            </Button>
          </div>

          <div className={styles.progress}>
            <Text variant="paragraph-small" color="secondary">
              Dataset {currentDataIndex + 1} of {datasets.length}
            </Text>
            <div className={styles.progressBar}>
              <div 
                className={styles.progressFill}
                style={{ width: `${((currentDataIndex + 1) / datasets.length) * 100}%` }}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Metrics Grid */}
      <AnalyticsGrid
        title="Real-time Metrics"
        subtitle="Live performance indicators with animated data"
        columns={4}
        gap="md"
      >
        {currentDataset?.metrics.map((metric, index) => (
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

      {/* Visualizations Grid */}
      <div className={styles.visualizationsGrid}>
        {/* 3D Vector Space */}
        <div className={styles.visualizationCard}>
          <Vector3DVisualization
            title="3D Vector Space"
            subtitle="High-dimensional embedding visualization"
            vectors={currentDataset?.vectors || []}
            clusters={currentDataset?.clusters || []}
            projection="pca"
            onVectorClick={(vector) => console.log('Vector clicked:', vector)}
            onVectorHover={(vector) => console.log('Vector hovered:', vector)}
          />
        </div>

        {/* Time Series Chart */}
        <div className={styles.visualizationCard}>
          <D3Visualization
            title="Performance Trends"
            subtitle="Latency over time with animated data"
            data={currentDataset?.timeSeries || []}
            type="line"
            config={{ xAxis: 'x', yAxis: 'y', color: 'category' }}
            onDataPointClick={(data) => console.log('Data point clicked:', data)}
            width={600}
            height={300}
          />
        </div>

        {/* Scatter Plot */}
        <div className={styles.visualizationCard}>
          <D3Visualization
            title="Model Distribution"
            subtitle="Scatter plot of model performance"
            data={currentDataset?.scatter || []}
            type="scatter"
            config={{ xAxis: 'x', yAxis: 'y', color: 'category', size: 'size' }}
            onDataPointClick={(data) => console.log('Scatter point clicked:', data)}
            width={600}
            height={300}
          />
        </div>

        {/* Bar Chart */}
        <div className={styles.visualizationCard}>
          <D3Visualization
            title="Category Distribution"
            subtitle="Vector categories breakdown"
            data={(currentDataset?.scatter || []).slice(0, 10).map((item, i) => ({
              x: i,
              y: Math.random() * 100,
              category: item.category
            }))}
            type="bar"
            config={{ xAxis: 'x', yAxis: 'y', color: 'category' }}
            onDataPointClick={(data) => console.log('Bar clicked:', data)}
            width={600}
            height={300}
          />
        </div>
      </div>

      {/* Features Showcase */}
      <div className={styles.featuresShowcase}>
        <Text variant="h3">Key Features</Text>
        <div className={styles.featuresGrid}>
          <div className={styles.featureCard}>
            <Database className={styles.featureIcon} />
            <Text variant="h4">Vector Database</Text>
            <Text variant="paragraph-medium" color="secondary">
              High-dimensional vector storage and similarity search with real-time analytics
            </Text>
          </div>

          <div className={styles.featureCard}>
            <BarChart3 className={styles.featureIcon} />
            <Text variant="h4">D3.js Visualizations</Text>
            <Text variant="paragraph-medium" color="secondary">
              Interactive charts and graphs with professional styling and animations
            </Text>
          </div>

          <div className={styles.featureCard}>
            <Zap className={styles.featureIcon} />
            <Text variant="h4">Real-time Updates</Text>
            <Text variant="paragraph-medium" color="secondary">
              Live data streaming with WebSocket connections and automatic refresh
            </Text>
          </div>

          <div className={styles.featureCard}>
            <Activity className={styles.featureIcon} />
            <Text variant="h4">3D Exploration</Text>
            <Text variant="paragraph-medium" color="secondary">
              Interactive 3D vector space visualization with Three.js and WebGL
            </Text>
          </div>
        </div>
      </div>
    </div>
  );
}
