/**
 * Hardware Metrics Grid
 * Real-time display of Apple Silicon hardware utilization
 *
 * @author @darianrosebrook
 */

'use client';

import { useState } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import {
  Cpu,
  Monitor,
  Zap,
  HardDrive,
  Thermometer,
  Activity,
} from 'lucide-react';
import { useAppleSiliconStore, useHardwareUtilization, useThermalStatus, usePowerEfficiency } from '@/stores/apple-silicon';
import { HardwareMetricCard } from './HardwareMetricCard';
import { UtilizationChart } from './UtilizationChart';
import styles from './HardwareMetricsGrid.module.scss';

export function HardwareMetricsGrid() {
  const [selectedTimeRange, setSelectedTimeRange] = useState<'1h' | '6h' | '24h'>('1h');
  const [chartType, setChartType] = useState<'line' | 'area' | 'bar'>('line');

  const currentMetrics = useAppleSiliconStore((state) => state.currentMetrics);
  const historicalMetrics = useAppleSiliconStore((state) => state.historicalMetrics);
  const utilization = useHardwareUtilization();
  const thermal = useThermalStatus();
  const power = usePowerEfficiency();

  if (!currentMetrics) {
    return (
      <div className={styles.loadingState}>
        <div className={styles.spinner}></div>
        <Text variant="paragraph-large">Loading hardware metrics...</Text>
      </div>
    );
  }

  const hardwareMetrics = [
    {
      title: 'Neural Engine (ANE)',
      value: `${currentMetrics.ane.utilization}%`,
      subtitle: `${currentMetrics.ane.activeModels} active models`,
      icon: <Cpu size={20} />,
      status: currentMetrics.ane.utilization > 80 ? 'warning' : 'good',
      details: {
        temperature: `${currentMetrics.ane.temperature}°C`,
        power: `${currentMetrics.ane.powerConsumption.toFixed(1)}W`,
        efficiency: `${currentMetrics.ane.efficiency.toFixed(1)} inf/s/W`,
        queue: currentMetrics.ane.inferenceQueue,
        throttling: currentMetrics.ane.throttling,
      }
    },
    {
      title: 'GPU (Metal)',
      value: `${currentMetrics.gpu.utilization}%`,
      subtitle: `${currentMetrics.gpu.activeComputeTasks} active tasks`,
      icon: <Monitor size={20} />,
      status: currentMetrics.gpu.utilization > 90 ? 'warning' : 'good',
      details: {
        temperature: `${currentMetrics.gpu.temperature}°C`,
        memory: `${(currentMetrics.gpu.memoryUsage / 1024).toFixed(1)}GB`,
        bandwidth: `${(currentMetrics.gpu.memoryBandwidth / 1024).toFixed(1)}GB/s`,
        frequency: `${currentMetrics.gpu.frequency}MHz`,
        power: `${currentMetrics.gpu.powerConsumption.toFixed(1)}W`,
      }
    },
    {
      title: 'CPU Cores',
      value: `${currentMetrics.cpu.utilization}%`,
      subtitle: `${currentMetrics.cpu.activeCores}/${currentMetrics.cpu.coreCount} cores active`,
      icon: <Activity size={20} />,
      status: currentMetrics.cpu.utilization > 85 ? 'warning' : 'good',
      details: {
        temperature: `${currentMetrics.cpu.temperature}°C`,
        frequency: `${currentMetrics.cpu.frequency}MHz`,
        power: `${currentMetrics.cpu.powerConsumption.toFixed(1)}W`,
        throttling: currentMetrics.cpu.thermalThrottling,
      }
    },
    {
      title: 'Unified Memory',
      value: `${((currentMetrics.memory.usedMemory / currentMetrics.memory.totalMemory) * 100).toFixed(1)}%`,
      subtitle: `${(currentMetrics.memory.usedMemory / 1024).toFixed(1)}GB / ${(currentMetrics.memory.totalMemory / 1024).toFixed(1)}GB`,
      icon: <HardDrive size={20} />,
      status: (currentMetrics.memory.usedMemory / currentMetrics.memory.totalMemory) > 0.85 ? 'warning' : 'good',
      details: {
        bandwidth: `${(currentMetrics.memory.bandwidth / 1024).toFixed(1)}GB/s`,
        efficiency: `${currentMetrics.memory.efficiency}%`,
        fragmentation: `${currentMetrics.memory.fragmentation}%`,
        available: `${(currentMetrics.memory.availableMemory / 1024).toFixed(1)}GB`,
      }
    },
    {
      title: 'Power Consumption',
      value: `${currentMetrics.power.totalConsumption.toFixed(1)}W`,
      subtitle: `${power?.efficiency ? power.efficiency.toFixed(1) : 'N/A'}% efficient`,
      icon: <Zap size={20} />,
      status: currentMetrics.power.totalConsumption > currentMetrics.power.thermalDesignPower * 0.9 ? 'warning' : 'good',
      details: {
        cpu: `${currentMetrics.power.cpuConsumption.toFixed(1)}W`,
        gpu: `${currentMetrics.power.gpuConsumption.toFixed(1)}W`,
        ane: `${currentMetrics.power.aneConsumption.toFixed(1)}W`,
        battery: currentMetrics.power.batteryLevel ? `${currentMetrics.power.batteryLevel}%` : 'N/A',
        charging: currentMetrics.power.charging,
        tdp: `${currentMetrics.power.thermalDesignPower}W`,
      }
    },
    {
      title: 'Thermal Status',
      value: thermal?.status === 'optimal' ? 'Optimal' : thermal?.status === 'warning' ? 'Warning' : 'Critical',
      subtitle: `${currentMetrics.thermal.cpuTemperature}°C CPU peak`,
      icon: <Thermometer size={20} />,
      status: thermal?.status === 'optimal' ? 'good' : thermal?.status === 'warning' ? 'warning' : 'error',
      details: {
        cpu: `${currentMetrics.thermal.cpuTemperature}°C`,
        gpu: `${currentMetrics.thermal.gpuTemperature}°C`,
        ane: `${currentMetrics.thermal.aneTemperature}°C`,
        ambient: `${currentMetrics.thermal.ambientTemperature}°C`,
        fanSpeed: currentMetrics.thermal.fanSpeed ? `${currentMetrics.thermal.fanSpeed}RPM` : 'Auto',
        coolingEfficiency: `${currentMetrics.thermal.coolingEfficiency}%`,
        margin: `${currentMetrics.thermal.thermalMargin}°C`,
        throttling: currentMetrics.thermal.thermalThrottling,
      }
    }
  ];

  // Prepare chart data from historical metrics
  const chartData = historicalMetrics.slice(-20).map((metric) => ({
    timestamp: new Date(metric.timestamp),
    ane: metric.ane.utilization,
    gpu: metric.gpu.utilization,
    cpu: metric.cpu.utilization,
    memory: ((metric.memory.totalMemory - metric.memory.availableMemory) / metric.memory.totalMemory) * 100,
    power: metric.power.totalConsumption,
    temperature: Math.max(metric.thermal.cpuTemperature, metric.thermal.gpuTemperature),
  }));

  return (
    <div className={styles.hardwareGrid}>
      {/* Header Controls */}
      <div className={styles.gridHeader}>
        <div className={styles.headerInfo}>
          <Text variant="h3">Hardware Utilization</Text>
          <Text variant="paragraph-medium" color="secondary">
            Real-time Apple Silicon component monitoring
          </Text>
        </div>

        <div className={styles.headerControls}>
          <div className={styles.timeRangeSelector}>
            <select
              value={selectedTimeRange}
              onChange={(e) => setSelectedTimeRange(e.target.value as typeof selectedTimeRange)}
              className={styles.timeSelect}
            >
              <option value="1h">Last Hour</option>
              <option value="6h">Last 6 Hours</option>
              <option value="24h">Last 24 Hours</option>
            </select>
          </div>

          <div className={styles.chartTypeSelector}>
            <Button
              variant={chartType === 'line' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setChartType('line')}
            >
              Line
            </Button>
            <Button
              variant={chartType === 'area' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setChartType('area')}
            >
              Area
            </Button>
            <Button
              variant={chartType === 'bar' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setChartType('bar')}
            >
              Bar
            </Button>
          </div>
        </div>
      </div>

      {/* Metrics Cards */}
      <div className={styles.metricsGrid}>
        {hardwareMetrics.map((metric, index) => (
          <HardwareMetricCard
            key={index}
            title={metric.title}
            value={metric.value}
            subtitle={metric.subtitle}
            icon={metric.icon}
            status={metric.status as 'good' | 'warning' | 'error'}
            details={metric.details}
          />
        ))}
      </div>

      {/* Utilization Chart */}
      <div className={styles.chartSection}>
        <div className={styles.chartHeader}>
          <Text variant="h4">Utilization Trends</Text>
          <Text variant="paragraph-small" color="secondary">
            {selectedTimeRange} historical data
          </Text>
        </div>

        <div className={styles.chartContainer}>
          <UtilizationChart
            data={chartData}
            type={chartType}
            timeRange={selectedTimeRange}
          />
        </div>
      </div>

      {/* System Overview */}
      <div className={styles.overviewSection}>
        <div className={styles.overviewCard}>
          <Text variant="h4">System Overview</Text>
          <div className={styles.overviewGrid}>
            <div className={styles.overviewItem}>
              <Text variant="paragraph-small">Overall Utilization</Text>
              <Text variant="h2">
                {utilization ? `${Math.round(utilization.average)}%` : 'N/A'}
              </Text>
              <div className={styles.utilizationBreakdown}>
                <div className={styles.breakdownItem}>
                  <span className={styles.breakdownLabel}>ANE:</span>
                  <span className={styles.breakdownValue}>{utilization?.ane.toFixed(0)}%</span>
                </div>
                <div className={styles.breakdownItem}>
                  <span className={styles.breakdownLabel}>GPU:</span>
                  <span className={styles.breakdownValue}>{utilization?.gpu.toFixed(0)}%</span>
                </div>
                <div className={styles.breakdownItem}>
                  <span className={styles.breakdownLabel}>CPU:</span>
                  <span className={styles.breakdownValue}>{utilization?.cpu.toFixed(0)}%</span>
                </div>
                <div className={styles.breakdownItem}>
                  <span className={styles.breakdownLabel}>Memory:</span>
                  <span className={styles.breakdownValue}>{utilization?.memory.toFixed(0)}%</span>
                </div>
              </div>
            </div>

            <div className={styles.overviewItem}>
              <Text variant="paragraph-small">Thermal Health</Text>
              <Text variant="h2" className={thermal?.status === 'optimal' ? styles.healthy : thermal?.status === 'warning' ? styles.warning : styles.critical}>
                {thermal?.status.toUpperCase() || 'UNKNOWN'}
              </Text>
              <div className={styles.temperatureReadings}>
                <div className={styles.tempReading}>
                  <span className={styles.tempLabel}>CPU:</span>
                  <span className={`${styles.tempValue} ${currentMetrics.thermal.cpuTemperature > 85 ? styles.hot : ''}`}>
                    {currentMetrics.thermal.cpuTemperature}°C
                  </span>
                </div>
                <div className={styles.tempReading}>
                  <span className={styles.tempLabel}>GPU:</span>
                  <span className={`${styles.tempValue} ${currentMetrics.thermal.gpuTemperature > 85 ? styles.hot : ''}`}>
                    {currentMetrics.thermal.gpuTemperature}°C
                  </span>
                </div>
                <div className={styles.tempReading}>
                  <span className={styles.tempLabel}>ANE:</span>
                  <span className={`${styles.tempValue} ${currentMetrics.thermal.aneTemperature > 85 ? styles.hot : ''}`}>
                    {currentMetrics.thermal.aneTemperature}°C
                  </span>
                </div>
              </div>
            </div>

            <div className={styles.overviewItem}>
              <Text variant="paragraph-small">Power Status</Text>
              <Text variant="h2">
                {currentMetrics.power.totalConsumption.toFixed(1)}W
              </Text>
              <div className={styles.powerBreakdown}>
                <div className={styles.powerItem}>
                  <span className={styles.powerLabel}>Efficiency:</span>
                  <span className={styles.powerValue}>
                    {power?.efficiency ? `${power.efficiency.toFixed(1)}%` : 'N/A'}
                  </span>
                </div>
                <div className={styles.powerItem}>
                  <span className={styles.powerLabel}>Battery:</span>
                  <span className={styles.powerValue}>
                    {currentMetrics.power.batteryLevel ? `${currentMetrics.power.batteryLevel}%` : 'N/A'}
                  </span>
                </div>
                <div className={styles.powerItem}>
                  <span className={styles.powerLabel}>Charging:</span>
                  <span className={styles.powerValue}>
                    {currentMetrics.power.charging ? 'Yes' : 'No'}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
