/**
 * HardwareUtilizationDashboard Component
 * Real-time monitoring of ANE, GPU, CPU, and memory utilization
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect, useMemo } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { Progress } from '@/design-system/primitives';
import { Badge } from '@/design-system/primitives';
import {
  Cpu,
  Zap,
  HardDrive,
  Thermometer,
  Activity,
  TrendingUp,
  TrendingDown,
  Minus,
  RefreshCw,
  AlertTriangle,
  CheckCircle,
  Clock
} from 'lucide-react';
import { appleSiliconApiClient } from '@/lib/apple-silicon-api';
import {
  useAppleSiliconStore,
  useAppleSiliconActions,
  useHardwareUtilization
} from '@/stores/apple-silicon';
import styles from './HardwareUtilizationDashboard.module.scss';


// Local interfaces
interface UtilizationTrend {
  current: number;
  previous: number;
  change: number;
  direction: 'up' | 'down' | 'stable';
}

interface HardwareStatus {
  aneStatus: 'optimal' | 'high' | 'critical';
  gpuStatus: 'optimal' | 'high' | 'critical';
  cpuStatus: 'optimal' | 'high' | 'critical';
  memoryStatus: 'optimal' | 'high' | 'critical';
  thermalStatus: 'normal' | 'elevated' | 'critical';
  overallHealth: 'healthy' | 'warning' | 'critical';
}

export function HardwareUtilizationDashboard() {
  // Use Zustand store and hooks
  const utilization = useHardwareUtilization();
  const { loading } = useAppleSiliconStore();
  const actions = useAppleSiliconActions();

  // Local state for UI
  const [refreshing, setRefreshing] = useState(false);

  // Fetch hardware metrics
  const fetchMetrics = async () => {
    try {
      actions.setLoading('metrics', true);
      actions.setError('metrics', null);

      // Fetch current metrics
      const metrics = await appleSiliconApiClient.getCurrentMetrics();
      actions.setCurrentMetrics(metrics);


      // Fetch historical data for trending
      const historical = await appleSiliconApiClient.getHistoricalMetrics('1h');
      actions.setHistoricalMetrics(historical);

    } catch (err) {
      console.error('Failed to fetch hardware metrics:', err);
      actions.setError('metrics', err instanceof Error ? err.message : 'Failed to fetch metrics');
    } finally {
      actions.setLoading('metrics', false);
    }
  };

  // Handle refresh
  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await fetchMetrics();
    } finally {
      setRefreshing(false);
    }
  };


  // Calculate utilization trends
  const utilizationTrends = useMemo(() => {
    if (!utilization || ([]).length < 2) {
      return {
        ane: { current: 0, previous: 0, change: 0, direction: 'stable' as const },
        gpu: { current: 0, previous: 0, change: 0, direction: 'stable' as const },
        cpu: { current: 0, previous: 0, change: 0, direction: 'stable' as const },
        memory: { current: 0, previous: 0, change: 0, direction: 'stable' as const },
        0: { current: 0, previous: 0, change: 0, direction: 'stable' as const },
      };
    }


    const calculateTrend = (current: number, previous: number): UtilizationTrend => {
      const change = current - previous;
      const direction = Math.abs(change) < 1 ? 'stable' : change > 0 ? 'up' : 'down';

      return {
        current,
        previous,
        change: Math.abs(change),
        direction,
      };
    };

    return {
      ane: calculateTrend(utilization.ane, 0),
      gpu: calculateTrend(utilization.gpu, 0),
      cpu: calculateTrend(utilization.cpu, 0),
      memory: calculateTrend(utilization.memory, 0),
      0: calculateTrend(0, 0),
    };
  }, [utilization]);

  // Determine hardware status
  const hardwareStatus = useMemo((): HardwareStatus => {
    if (!utilization) {
      return {
        aneStatus: 'optimal',
        gpuStatus: 'optimal',
        cpuStatus: 'optimal',
        memoryStatus: 'optimal',
        thermalStatus: 'normal',
        overallHealth: 'healthy',
      };
    }

    const getComponentStatus = (utilization: number, thresholdHigh = 80, thresholdCritical = 95) => {
      if (utilization >= thresholdCritical) return 'critical';
      if (utilization >= thresholdHigh) return 'high';
      return 'optimal';
    };

    const getThermalStatus = (temp: number) => {
      if (temp >= 85) return 'critical';
      if (temp >= 75) return 'elevated';
      return 'normal';
    };

    const aneStatus = getComponentStatus(utilization.ane);
    const gpuStatus = getComponentStatus(utilization.gpu);
    const cpuStatus = getComponentStatus(utilization.cpu);
    const memoryStatus = getComponentStatus(utilization.memory);
    const thermalStatus = getThermalStatus(0);

    // Determine overall health
    const statuses = [aneStatus, gpuStatus, cpuStatus, memoryStatus];
    const thermalCritical = thermalStatus === 'critical';
    const hasCritical = statuses.includes('critical') || thermalCritical;
    const hasHigh = statuses.includes('high') || thermalStatus === 'elevated';

    let overallHealth: 'healthy' | 'warning' | 'critical' = 'healthy';
    if (hasCritical) overallHealth = 'critical';
    else if (hasHigh) overallHealth = 'warning';

    return {
      aneStatus,
      gpuStatus,
      cpuStatus,
      memoryStatus,
      thermalStatus,
      overallHealth,
    };
  }, [utilization]);

  // Initial data load
  useEffect(() => {
    fetchMetrics();
  }, []);

  // Set up polling
  useEffect(() => {
    const interval = setInterval(() => {
      fetchMetrics();
    }, 10000); // 10 seconds

    return () => clearInterval(interval);
  }, []);

  // Get status icon and color
  const getStatusDisplay = (status: string) => {
    switch (status) {
      case 'optimal':
      case 'normal':
      case 'healthy':
        return { icon: CheckCircle, color: 'success', label: status };
      case 'high':
      case 'elevated':
      case 'warning':
        return { icon: AlertTriangle, color: 'warning', label: status };
      case 'critical':
        return { icon: AlertTriangle, color: 'error', label: status };
      default:
        return { icon: Clock, color: 'secondary', label: status };
    }
  };

  // Get trend icon
  const getTrendIcon = (direction: string) => {
    switch (direction) {
      case 'up':
        return <TrendingUp size={14} />;
      case 'down':
        return <TrendingDown size={14} />;
      default:
        return <Minus size={14} />;
    }
  };


  const isLoading = loading.metrics;

  if (isLoading) {
    return (
      <div className={styles.loading}>
        <div className={styles.spinner}></div>
        <Text variant="paragraph-medium" color="secondary">
          Loading hardware metrics...
        </Text>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      {/* Header with controls */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h3">Hardware Utilization</Text>
          <Text variant="paragraph-small" color="secondary">
            Real-time ANE, GPU, CPU, and memory monitoring
          </Text>
        </div>

        <div className={styles.headerRight}>

          <Button
            variant="secondary"
            size="sm"
            onClick={handleRefresh}
            disabled={refreshing}
            aria-label="Refresh hardware data"
          >
            <RefreshCw
              size={16}
              className={refreshing ? styles.spinning : ''}
            />
          </Button>
        </div>
      </div>

      {/* System Health Status */}
      <div className={styles.healthStatus}>
        <div className={styles.healthCard}>
          <div className={styles.healthHeader}>
            <Activity className={styles.healthIcon} />
            <Text variant="h4">System Health</Text>
          </div>

          <div className={styles.healthGrid}>
            <div className={styles.healthItem}>
              <Text variant="paragraph-small" color="secondary">Overall</Text>
              <Badge variant={getStatusDisplay(hardwareStatus.overallHealth).color as any}>
                {getStatusDisplay(hardwareStatus.overallHealth).label}
              </Badge>
            </div>

            <div className={styles.healthItem}>
              <Text variant="paragraph-small" color="secondary">Thermal</Text>
              <Badge variant={getStatusDisplay(hardwareStatus.thermalStatus).color as any}>
                {getStatusDisplay(hardwareStatus.thermalStatus).label}
              </Badge>
            </div>

          </div>
        </div>
      </div>

      {/* Hardware Metrics Grid */}
      <div className={styles.metricsGrid}>
        {/* ANE Utilization */}
        <div className={styles.metricCard}>
          <div className={styles.metricHeader}>
            <Zap className={`${styles.metricIcon} ${styles.aneIcon}`} />
            <div className={styles.metricInfo}>
              <Text variant="h4" className={styles.metricTitle}>Apple Neural Engine</Text>
              <div className={styles.metricStatus}>
                <Badge variant={getStatusDisplay(hardwareStatus.aneStatus).color as any} size="sm">
                  {getStatusDisplay(hardwareStatus.aneStatus).label}
                </Badge>
              </div>
            </div>
          </div>

          <div className={styles.metricValue}>
            <Text variant="display-2">{utilization?.ane.toFixed(1)}%</Text>
            <div className={styles.metricTrend}>
              {getTrendIcon(utilizationTrends.ane.direction)}
              <Text variant="paragraph-small" color="secondary">
                {utilizationTrends.ane.change.toFixed(1)}%
              </Text>
            </div>
          </div>

          <Progress
            value={utilization?.ane || 0}
            className={styles.metricProgress}
            variant={utilization && utilization.ane > 80 ? 'warning' : 'default'}
          />
        </div>

        {/* GPU Utilization */}
        <div className={styles.metricCard}>
          <div className={styles.metricHeader}>
            <Cpu className={`${styles.metricIcon} ${styles.gpuIcon}`} />
            <div className={styles.metricInfo}>
              <Text variant="h4" className={styles.metricTitle}>Metal GPU</Text>
              <div className={styles.metricStatus}>
                <Badge variant={getStatusDisplay(hardwareStatus.gpuStatus).color as any} size="sm">
                  {getStatusDisplay(hardwareStatus.gpuStatus).label}
                </Badge>
              </div>
            </div>
          </div>

          <div className={styles.metricValue}>
            <Text variant="display-2">{utilization?.gpu.toFixed(1)}%</Text>
            <div className={styles.metricTrend}>
              {getTrendIcon(utilizationTrends.gpu.direction)}
              <Text variant="paragraph-small" color="secondary">
                {utilizationTrends.gpu.change.toFixed(1)}%
              </Text>
            </div>
          </div>

          <Progress
            value={utilization?.gpu || 0}
            className={styles.metricProgress}
            variant={utilization && utilization.gpu > 80 ? 'warning' : 'default'}
          />
        </div>

        {/* CPU Utilization */}
        <div className={styles.metricCard}>
          <div className={styles.metricHeader}>
            <Activity className={`${styles.metricIcon} ${styles.cpuIcon}`} />
            <div className={styles.metricInfo}>
              <Text variant="h4" className={styles.metricTitle}>CPU Cores</Text>
              <div className={styles.metricStatus}>
                <Badge variant={getStatusDisplay(hardwareStatus.cpuStatus).color as any} size="sm">
                  {getStatusDisplay(hardwareStatus.cpuStatus).label}
                </Badge>
              </div>
            </div>
          </div>

          <div className={styles.metricValue}>
            <Text variant="display-2">{utilization?.cpu.toFixed(1)}%</Text>
            <div className={styles.metricTrend}>
              {getTrendIcon(utilizationTrends.cpu.direction)}
              <Text variant="paragraph-small" color="secondary">
                {utilizationTrends.cpu.change.toFixed(1)}%
              </Text>
            </div>
          </div>

          <Progress
            value={utilization?.cpu || 0}
            className={styles.metricProgress}
            variant={utilization && utilization.cpu > 80 ? 'warning' : 'default'}
          />
        </div>

        {/* Memory Usage */}
        <div className={styles.metricCard}>
          <div className={styles.metricHeader}>
            <HardDrive className={`${styles.metricIcon} ${styles.memoryIcon}`} />
            <div className={styles.metricInfo}>
              <Text variant="h4" className={styles.metricTitle}>Unified Memory</Text>
              <div className={styles.metricStatus}>
                <Badge variant={getStatusDisplay(hardwareStatus.memoryStatus).color as any} size="sm">
                  {getStatusDisplay(hardwareStatus.memoryStatus).label}
                </Badge>
              </div>
            </div>
          </div>

          <div className={styles.metricValue}>
            <Text variant="display-2">
              {utilization?.memory.toFixed(1)}GB
            </Text>
            <Text variant="paragraph-small" color="secondary">
              of {utilization?.memory}GB
            </Text>
          </div>

          <Progress
            value={utilization ? (utilization.memory / utilization.memory) * 100 : 0}
            className={styles.metricProgress}
            variant={utilization && (utilization.memory / utilization.memory) > 0.8 ? 'warning' : 'default'}
          />
        </div>

        {/* Temperature */}
        <div className={styles.metricCard}>
          <div className={styles.metricHeader}>
            <Thermometer className={`${styles.metricIcon} ${styles.tempIcon}`} />
            <div className={styles.metricInfo}>
              <Text variant="h4" className={styles.metricTitle}>Temperature</Text>
              <div className={styles.metricStatus}>
                <Badge variant={getStatusDisplay(hardwareStatus.thermalStatus).color as any} size="sm">
                  {getStatusDisplay(hardwareStatus.thermalStatus).label}
                </Badge>
              </div>
            </div>
          </div>

          <div className={styles.metricValue}>
            <Text variant="display-2">{utilization?.[0].toFixed(1)}°C</Text>
            <div className={styles.metricTrend}>
              {getTrendIcon(utilizationTrends[0].direction)}
              <Text variant="paragraph-small" color="secondary">
                {utilizationTrends[0].change.toFixed(1)}°C
              </Text>
            </div>
          </div>

          <div className={styles.zeroScale}>
            <div className={styles.zeroBar}>
              <div
                className={`${styles.zeroProgress} ${
                  utilization && utilization[0] > 80 ? styles.critical :
                  utilization && utilization[0] > 70 ? styles.warning : styles.normal
                }`}
                style={{ width: `${utilization ? Math.min((utilization[0] / 100) * 100, 100) : 0}%` }}
              />
            </div>
            <div className={styles.zeroLabels}>
              <span>0°C</span>
              <span>50°C</span>
              <span>100°C</span>
            </div>
          </div>
        </div>

        {/* Performance Metrics */}
        <div className={styles.metricCard}>
          <div className={styles.metricHeader}>
            <TrendingUp className={`${styles.metricIcon} ${styles.perfIcon}`} />
            <div className={styles.metricInfo}>
              <Text variant="h4" className={styles.metricTitle}>Performance</Text>
            </div>
          </div>

          <div className={styles.performanceMetrics}>
            <div className={styles.perfItem}>
              <Text variant="paragraph-small" color="secondary">Throughput</Text>
              <Text variant="h4">{performanceMetrics?.inferenceThroughput.toLocaleString()}/s</Text>
            </div>

            <div className={styles.perfItem}>
              <Text variant="paragraph-small" color="secondary">Load Time</Text>
              <Text variant="h4">{performanceMetrics?.modelLoadTime.toFixed(2)}s</Text>
            </div>

            <div className={styles.perfItem}>
              <Text variant="paragraph-small" color="secondary">Efficiency</Text>
              <Text variant="h4">{performanceMetrics?.computeEfficiency.toFixed(1)}%</Text>
            </div>
          </div>
        </div>
      </div>

      {/* Additional Info */}
      <div className={styles.additionalInfo}>
        <div className={styles.infoCard}>
          <Text variant="h5" className={styles.infoTitle}>System Information</Text>
          <div className={styles.infoGrid}>
            <div className={styles.infoItem}>
              <Text variant="paragraph-small" color="secondary">Power Consumption</Text>
              <Text variant="paragraph-medium">{utilization?.[0].toFixed(1)}W</Text>
            </div>

            {utilization?.[0] && (
              <div className={styles.infoItem}>
                <Text variant="paragraph-small" color="secondary">Fan Speed</Text>
                <Text variant="paragraph-medium">{utilization[0]} RPM</Text>
              </div>
            )}

            <div className={styles.infoItem}>
              <Text variant="paragraph-small" color="secondary">Last Updated</Text>
              <Text variant="paragraph-medium">
                {new Date().toLocaleTimeString()}
              </Text>
            </div>

            <div className={styles.infoItem}>
              <Text variant="paragraph-small" color="secondary">Data Points</Text>
              <Text variant="paragraph-medium">{historicalMetrics.length}</Text>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
