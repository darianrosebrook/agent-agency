/**
 * Thermal Management Panel
 * Real-time temperature monitoring and cooling system controls
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { Progress } from '@/design-system/primitives';
import {
  Thermometer,
  Fan,
  AlertTriangle,
  Settings,
  RefreshCw,
  Activity,
  Zap,
  Shield
} from 'lucide-react';
import { appleSiliconApiClient } from '@/lib/apple-silicon-api';
import { useAppleSiliconWebSocket } from '@/hooks/useAppleSiliconWebSocket';
import { useAppleSiliconStore } from '@/stores/apple-silicon';
import styles from './ThermalManagementPanel.module.scss';

export function ThermalManagementPanel() {
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h'>('1h');
  const [refreshing, setRefreshing] = useState(false);
  const [thermalData, setThermalData] = useState<any>(null);
  const [thermalHistory, setThermalHistory] = useState<any[]>([]);
  const [thermalPolicies, setThermalPolicies] = useState<any[]>([]);
  const [thermalEvents, setThermalEvents] = useState<any[]>([]);

  const { isConnected, lastUpdate } = useAppleSiliconWebSocket();
  const { thermalMetrics, thermalStatus } = useAppleSiliconStore();

  useEffect(() => {
    loadThermalData();
  }, [timeRange]);

  const loadThermalData = async () => {
    try {
      setRefreshing(true);
      const [thermal, history, policies, events] = await Promise.all([
        appleSiliconApiClient.getThermalStatus(),
        appleSiliconApiClient.getThermalHistory(timeRange),
        appleSiliconApiClient.getThermalPolicies(),
        appleSiliconApiClient.getThermalEvents(timeRange)
      ]);

      setThermalData(thermal);
      setThermalHistory(history);
      setThermalPolicies(policies);
      setThermalEvents(events);
    } catch (error) {
      console.error('Failed to load thermal data:', error);
    } finally {
      setRefreshing(false);
    }
  };

  const getThermalStatusColor = (status: string) => {
    switch (status) {
      case 'normal':
        return 'green';
      case 'warning':
        return 'orange';
      case 'critical':
        return 'red';
      default:
        return 'gray';
    }
  };

  const getThermalStatusIcon = (status: string) => {
    switch (status) {
      case 'normal':
        return <Shield size={16} />;
      case 'warning':
        return <AlertTriangle size={16} />;
      case 'critical':
        return <Zap size={16} />;
      default:
        return <Thermometer size={16} />;
    }
  };

  const formatTemperature = (temp: number) => {
    return `${temp.toFixed(1)}°C`;
  };

  const formatFanSpeed = (speed: number) => {
    return `${speed.toFixed(0)} RPM`;
  };

  const formatTimestamp = (timestamp: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(new Date(timestamp));
  };

  return (
    <div className={styles.thermalManagementPanel}>
      {/* Header */}
      <div className={styles.panelHeader}>
        <div className={styles.headerLeft}>
          <Text variant="h3">Thermal Management</Text>
          <Text variant="paragraph-medium" color="secondary">
            Real-time temperature monitoring and cooling system controls
          </Text>
        </div>

        <div className={styles.headerRight}>
          <div className={styles.connectionStatus}>
            <Activity size={16} />
            <Text variant="paragraph-small">
              {isConnected ? 'Connected' : 'Disconnected'}
            </Text>
          </div>

          <div className={styles.timeRangeSelector}>
            <select
              value={timeRange}
              onChange={(e) => setTimeRange(e.target.value as '1h' | '6h' | '24h')}
              className={styles.timeRangeSelect}
            >
              <option value="1h">Last Hour</option>
              <option value="6h">Last 6 Hours</option>
              <option value="24h">Last 24 Hours</option>
            </select>
          </div>

          <div className={styles.actions}>
            <Button
              variant="secondary"
              size="sm"
              onClick={loadThermalData}
              disabled={refreshing}
            >
              <RefreshCw size={14} />
              Refresh
            </Button>
            <Button variant="secondary" size="sm">
              <Settings size={14} />
              Settings
            </Button>
          </div>
        </div>
      </div>

      {/* Thermal Status Overview */}
      <div className={styles.thermalStatusSection}>
        <div className={styles.statusHeader}>
          <Text variant="h4">Current Thermal Status</Text>
          <div className={styles.statusIndicator}>
            {getThermalStatusIcon(thermalData?.status || 'normal')}
            <Text variant="paragraph-medium" className={styles[getThermalStatusColor(thermalData?.status || 'normal')]}>
              {thermalData?.status || 'Normal'}
            </Text>
          </div>
        </div>

        <div className={styles.thermalMetrics}>
          <div className={styles.metricCard}>
            <div className={styles.metricHeader}>
              <Thermometer size={20} />
              <Text variant="label">CPU Temperature</Text>
            </div>
            <div className={styles.metricValue}>
              <Text variant="display-small">{formatTemperature(thermalData?.cpuTemp || 0)}</Text>
              <div className={styles.metricProgress}>
                <Progress
                  value={thermalData?.cpuTemp || 0}
                  max={100}
                  className={styles.progressBar}
                />
              </div>
            </div>
          </div>

          <div className={styles.metricCard}>
            <div className={styles.metricHeader}>
              <Fan size={20} />
              <Text variant="label">Fan Speed</Text>
            </div>
            <div className={styles.metricValue}>
              <Text variant="display-small">{formatFanSpeed(thermalData?.fanSpeed || 0)}</Text>
              <div className={styles.metricProgress}>
                <Progress
                  value={thermalData?.fanSpeed || 0}
                  max={3000}
                  className={styles.progressBar}
                />
              </div>
            </div>
          </div>

          <div className={styles.metricCard}>
            <div className={styles.metricHeader}>
              <Zap size={20} />
              <Text variant="label">Power Consumption</Text>
            </div>
            <div className={styles.metricValue}>
              <Text variant="display-small">{thermalData?.powerConsumption || 0}W</Text>
              <div className={styles.metricProgress}>
                <Progress
                  value={thermalData?.powerConsumption || 0}
                  max={100}
                  className={styles.progressBar}
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Thermal History Chart */}
      <div className={styles.thermalHistorySection}>
        <div className={styles.historyHeader}>
          <Text variant="h4">Thermal History</Text>
          <div className={styles.historyControls}>
            <Button variant="secondary" size="sm">
              <Activity size={14} />
              Export Data
            </Button>
          </div>
        </div>

        <div className={styles.historyChart}>
          <div className={styles.chartContainer}>
            <div className={styles.chartPlaceholder}>
              <Activity size={48} />
              <Text variant="h5">Thermal History Chart</Text>
              <Text variant="paragraph-medium" color="secondary">
                Temperature and fan speed over time
              </Text>
            </div>
          </div>
        </div>
      </div>

      {/* Thermal Policies */}
      <div className={styles.thermalPoliciesSection}>
        <div className={styles.policiesHeader}>
          <Text variant="h4">Thermal Policies</Text>
          <Button variant="secondary" size="sm">
            <Settings size={14} />
            Manage Policies
          </Button>
        </div>

        <div className={styles.policiesGrid}>
          {thermalPolicies.map((policy, index) => (
            <div key={index} className={styles.policyCard}>
              <div className={styles.policyHeader}>
                <Text variant="paragraph-medium" className={styles.policyName}>
                  {policy.name}
                </Text>
                <div className={styles.policyStatus}>
                  <span className={`${styles.statusBadge} ${styles[policy.status]}`}>
                    {policy.status}
                  </span>
                </div>
              </div>
              <Text variant="paragraph-small" color="secondary" className={styles.policyDescription}>
                {policy.description}
              </Text>
              <div className={styles.policyDetails}>
                <div className={styles.policyDetail}>
                  <Text variant="label">Threshold:</Text>
                  <Text variant="paragraph-small">{policy.threshold}°C</Text>
                </div>
                <div className={styles.policyDetail}>
                  <Text variant="label">Action:</Text>
                  <Text variant="paragraph-small">{policy.action}</Text>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Thermal Events */}
      <div className={styles.thermalEventsSection}>
        <div className={styles.eventsHeader}>
          <Text variant="h4">Recent Thermal Events</Text>
          <Button variant="secondary" size="sm">
            <Activity size={14} />
            View All Events
          </Button>
        </div>

        <div className={styles.eventsList}>
          {thermalEvents.map((event, index) => (
            <div key={index} className={styles.eventCard}>
              <div className={styles.eventHeader}>
                <div className={styles.eventIcon}>
                  {getThermalStatusIcon(event.severity)}
                </div>
                <div className={styles.eventInfo}>
                  <Text variant="paragraph-medium" className={styles.eventTitle}>
                    {event.title}
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    {formatTimestamp(event.timestamp)}
                  </Text>
                </div>
                <div className={styles.eventSeverity}>
                  <span className={`${styles.severityBadge} ${styles[event.severity]}`}>
                    {event.severity}
                  </span>
                </div>
              </div>
              <Text variant="paragraph-small" color="secondary" className={styles.eventDescription}>
                {event.description}
              </Text>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
