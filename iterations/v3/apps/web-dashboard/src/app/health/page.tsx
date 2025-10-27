/**
 * System Health Dashboard
 * Real-time monitoring of backend system health and metrics
 *
 * @author @darianrosebrook
 */

"use client";

import React, { useState, useEffect } from 'react';
import { useSSEConnection } from '@/hooks/useSSEConnection';
import { useErrorHandler } from '@/lib/error-handling';
import { getApiClient } from '@/lib/api-client';
import DashboardLayout from "@/components/shared/DashboardLayout";
import { Text } from "@/design-system/primitives";
import { RefreshCw, Activity, Server, Database, Zap, Wifi, WifiOff } from "lucide-react";
import styles from "./page.module.scss";

interface SystemHealth {
  cpu_usage: number;
  memory_usage: number;
  disk_usage: number;
  network_io: number;
  active_connections: number;
  response_time: number;
  uptime: number;
  status: 'healthy' | 'warning' | 'critical';
}

interface ServiceStatus {
  name: string;
  status: 'healthy' | 'degraded' | 'down';
  response_time?: number;
  last_check: string;
  uptime_percentage: number;
}

export default function HealthDashboardPage() {
  const [healthData, setHealthData] = useState<SystemHealth | null>(null);
  const [services, setServices] = useState<ServiceStatus[]>([]);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const [refreshing, setRefreshing] = useState(false);

  // SSE connection for real-time health updates
  const healthSSE = useSSEConnection('/api/health/stream');
  const { handleError } = useErrorHandler();
  const apiClient = getApiClient();

  // Handle SSE messages for real-time updates
  useEffect(() => {
    if (healthSSE.lastEvent) {
      const event = healthSSE.lastEvent;
      console.log('Health SSE event:', event);

      if (event.type === 'health_update') {
        setHealthData(event.data);
        setLastUpdate(new Date(event.timestamp));
      } else if (event.type === 'service_update') {
        setServices(prev => prev.map(service =>
          service.name === event.data.name ? event.data : service
        ));
      }
    }
  }, [healthSSE.lastEvent]);

  // Fetch initial health data
  const fetchHealthData = async () => {
    setRefreshing(true);
    try {
      const response = await apiClient.request<SystemHealth>('/health');
      setHealthData(response.data);
      setLastUpdate(new Date());
    } catch (error) {
      handleError(error as any);
    } finally {
      setRefreshing(false);
    }
  };

  // Initialize with mock data for demonstration
  useEffect(() => {
    // Mock initial data
    setHealthData({
      cpu_usage: 45,
      memory_usage: 62,
      disk_usage: 78,
      network_io: 120,
      active_connections: 23,
      response_time: 125,
      uptime: 86400, // 24 hours in seconds
      status: 'healthy'
    });

    setServices([
      { name: 'API Gateway', status: 'healthy', response_time: 45, last_check: new Date().toISOString(), uptime_percentage: 99.9 },
      { name: 'Database', status: 'healthy', response_time: 12, last_check: new Date().toISOString(), uptime_percentage: 99.95 },
      { name: 'Task Processor', status: 'healthy', response_time: 89, last_check: new Date().toISOString(), uptime_percentage: 99.7 },
      { name: 'Cache Layer', status: 'degraded', response_time: 234, last_check: new Date().toISOString(), uptime_percentage: 98.2 },
      { name: 'File Storage', status: 'healthy', response_time: 67, last_check: new Date().toISOString(), uptime_percentage: 99.8 },
    ]);
  }, []);

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);

    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy': return '#22c55e';
      case 'warning': return '#f59e0b';
      case 'critical':
      case 'degraded':
      case 'down': return '#ef4444';
      default: return '#6b7280';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'healthy': return '🟢';
      case 'warning': return '🟡';
      case 'critical':
      case 'degraded':
      case 'down': return '🔴';
      default: return '⚪';
    }
  };

  const MetricCard = ({
    title,
    value,
    unit,
    icon: Icon,
    status = 'normal',
    maxValue = 100
  }: {
    title: string;
    value: number;
    unit: string;
    icon: React.ComponentType<any>;
    status?: 'normal' | 'warning' | 'critical';
    maxValue?: number;
  }) => (
    <div className={`${styles.metricCard} ${styles[status]}`}>
      <div className={styles.metricHeader}>
        <Icon size={20} />
        <span className={styles.metricTitle}>{title}</span>
      </div>

      <div className={styles.metricValue}>
        <span className={styles.value}>{value}</span>
        <span className={styles.unit}>{unit}</span>
      </div>

      <div className={styles.metricBar}>
        <div
          className={styles.barFill}
          style={{ width: `${Math.min((value / maxValue) * 100, 100)}%` }}
        />
      </div>
    </div>
  );

  const ServiceCard = ({ service }: { service: ServiceStatus }) => (
    <div className={`${styles.serviceCard} ${styles[service.status]}`}>
      <div className={styles.serviceHeader}>
        <span className={styles.statusIcon}>{getStatusIcon(service.status)}</span>
        <span className={styles.serviceName}>{service.name}</span>
      </div>

      <div className={styles.serviceMetrics}>
        {service.response_time && (
          <div className={styles.metric}>
            <span className={styles.label}>Response:</span>
            <span className={styles.value}>{service.response_time}ms</span>
          </div>
        )}

        <div className={styles.metric}>
          <span className={styles.label}>Uptime:</span>
          <span className={styles.value}>{service.uptime_percentage}%</span>
        </div>
      </div>

      <div className={styles.serviceLastCheck}>
        <small>Last check: {new Date(service.last_check).toLocaleTimeString()}</small>
      </div>
    </div>
  );

  return (
    <DashboardLayout>
      <main className={styles.container}>
        {/* Header */}
        <header className={styles.header}>
          <div className={styles.headerContent}>
            <div>
              <Text variant="display-3" className={styles.title}>
                System Health
              </Text>
              <Text variant="paragraph-large" color="secondary">
                Real-time monitoring of backend services and performance
              </Text>
            </div>

            <div className={styles.headerActions}>
              {/* Connection Status */}
              <div className={`${styles.connectionStatus} ${healthSSE.isConnected ? styles.connected : styles.disconnected}`}>
                {healthSSE.isConnected ? <Wifi size={16} /> : <WifiOff size={16} />}
                <span className={styles.connectionText}>
                  {healthSSE.isConnected ? 'Live Updates' : 'Offline'}
                </span>
              </div>

              <button
                onClick={fetchHealthData}
                className={styles.refreshButton}
                disabled={refreshing}
              >
                <RefreshCw size={16} className={refreshing ? styles.spinning : ''} />
                Refresh
              </button>
            </div>
          </div>

          <div className={styles.lastUpdate}>
            Last updated: {lastUpdate.toLocaleTimeString()}
            {healthSSE.eventCount > 0 && (
              <span className={styles.eventCount}>
                ({healthSSE.eventCount} real-time updates)
              </span>
            )}
          </div>
        </header>

        {/* System Overview */}
        <section className={styles.overview}>
          <div className={styles.overviewCard}>
            <div className={styles.overviewHeader}>
              <Activity size={24} />
              <h3>System Overview</h3>
            </div>

            {healthData && (
              <div className={styles.overviewGrid}>
                <div className={styles.overviewMetric}>
                  <span className={styles.label}>Status</span>
                  <span className={styles.value} style={{ color: getStatusColor(healthData.status) }}>
                    {getStatusIcon(healthData.status)} {healthData.status.toUpperCase()}
                  </span>
                </div>

                <div className={styles.overviewMetric}>
                  <span className={styles.label}>Uptime</span>
                  <span className={styles.value}>{formatUptime(healthData.uptime)}</span>
                </div>

                <div className={styles.overviewMetric}>
                  <span className={styles.label}>Active Connections</span>
                  <span className={styles.value}>{healthData.active_connections}</span>
                </div>

                <div className={styles.overviewMetric}>
                  <span className={styles.label}>Avg Response Time</span>
                  <span className={styles.value}>{healthData.response_time}ms</span>
                </div>
              </div>
            )}
          </div>
        </section>

        {/* Performance Metrics */}
        <section className={styles.metrics}>
          <h3>Performance Metrics</h3>

          <div className={styles.metricsGrid}>
            {healthData && (
              <>
                <MetricCard
                  title="CPU Usage"
                  value={healthData.cpu_usage}
                  unit="%"
                  icon={Server}
                  status={healthData.cpu_usage > 80 ? 'critical' : healthData.cpu_usage > 70 ? 'warning' : 'normal'}
                />

                <MetricCard
                  title="Memory Usage"
                  value={healthData.memory_usage}
                  unit="%"
                  icon={Database}
                  status={healthData.memory_usage > 80 ? 'critical' : healthData.memory_usage > 70 ? 'warning' : 'normal'}
                />

                <MetricCard
                  title="Disk Usage"
                  value={healthData.disk_usage}
                  unit="%"
                  icon={Database}
                  status={healthData.disk_usage > 90 ? 'critical' : healthData.disk_usage > 80 ? 'warning' : 'normal'}
                />

                <MetricCard
                  title="Network I/O"
                  value={healthData.network_io}
                  unit="MB/s"
                  icon={Zap}
                  maxValue={200}
                  status={healthData.network_io > 150 ? 'warning' : 'normal'}
                />
              </>
            )}
          </div>
        </section>

        {/* Service Status */}
        <section className={styles.services}>
          <h3>Service Status</h3>

          <div className={styles.servicesGrid}>
            {services.map((service, index) => (
              <ServiceCard key={index} service={service} />
            ))}
          </div>
        </section>
      </main>

      <style jsx>{`
        .container {
          max-width: 1400px;
          margin: 0 auto;
          padding: var(--spacing-6);
        }

        .header {
          margin-bottom: var(--spacing-8);
        }

        .headerContent {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: var(--spacing-4);
        }

        .title {
          margin: 0 0 var(--spacing-2) 0;
        }

        .headerActions {
          display: flex;
          gap: var(--spacing-3);
          align-items: center;
        }

        .connectionStatus {
          display: flex;
          align-items: center;
          gap: var(--spacing-2);
          padding: var(--spacing-2) var(--spacing-3);
          border-radius: var(--border-radius-md);
          font-size: 0.875rem;
          font-weight: 500;
          transition: all 0.2s ease;

          &.connected {
            background: rgba(34, 197, 94, 0.1);
            color: #16a34a;
            border: 1px solid rgba(34, 197, 94, 0.2);
          }

          &.disconnected {
            background: rgba(239, 68, 68, 0.1);
            color: #dc2626;
            border: 1px solid rgba(239, 68, 68, 0.2);
          }
        }

        .connectionText {
          @media (max-width: 640px) {
            display: none;
          }
        }

        .refreshButton {
          display: flex;
          align-items: center;
          gap: var(--spacing-2);
          padding: var(--spacing-2) var(--spacing-3);
          background: #3b82f6;
          color: white;
          border: none;
          border-radius: var(--border-radius-md);
          font-size: 0.875rem;
          font-weight: 500;
          cursor: pointer;
          transition: background 0.2s;

          &:hover:not(:disabled) {
            background: #2563eb;
          }

          &:disabled {
            background: #9ca3af;
            cursor: not-allowed;
          }
        }

        .lastUpdate {
          font-size: 0.875rem;
          color: #6b7280;
        }

        .eventCount {
          color: #22c55e;
          font-weight: 500;
        }

        .spinning {
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }

        /* Overview Section */
        .overview {
          margin-bottom: var(--spacing-8);
        }

        .overviewCard {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: var(--border-radius-lg);
          padding: var(--spacing-6);
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }

        .overviewHeader {
          display: flex;
          align-items: center;
          gap: var(--spacing-3);
          margin-bottom: var(--spacing-6);

          h3 {
            margin: 0;
            color: #374151;
            font-size: 1.25rem;
            font-weight: 600;
          }
        }

        .overviewGrid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: var(--spacing-4);
        }

        .overviewMetric {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: var(--spacing-3);
          background: #f9fafb;
          border-radius: var(--border-radius-md);

          .label {
            color: #6b7280;
            font-size: 0.875rem;
          }

          .value {
            font-weight: 600;
            color: #374151;
          }
        }

        /* Metrics Section */
        .metrics {
          margin-bottom: var(--spacing-8);

          h3 {
            margin: 0 0 var(--spacing-6) 0;
            color: #374151;
            font-size: 1.25rem;
            font-weight: 600;
          }
        }

        .metricsGrid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
          gap: var(--spacing-4);
        }

        .metricCard {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: var(--border-radius-lg);
          padding: var(--spacing-5);
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
          transition: border-color 0.2s;

          &.warning {
            border-color: #f59e0b;
          }

          &.critical {
            border-color: #ef4444;
          }
        }

        .metricHeader {
          display: flex;
          align-items: center;
          gap: var(--spacing-3);
          margin-bottom: var(--spacing-4);

          .metricTitle {
            color: #374151;
            font-weight: 500;
          }
        }

        .metricValue {
          display: flex;
          align-items: baseline;
          gap: var(--spacing-1);
          margin-bottom: var(--spacing-3);

          .value {
            font-size: 2rem;
            font-weight: 700;
            color: #374151;
          }

          .unit {
            color: #6b7280;
            font-size: 0.875rem;
          }
        }

        .metricBar {
          height: 4px;
          background: #e5e7eb;
          border-radius: 2px;
          overflow: hidden;

          .barFill {
            height: 100%;
            background: linear-gradient(90deg, #3b82f6, #06b6d4);
            transition: width 0.3s ease;
          }
        }

        /* Services Section */
        .services {
          h3 {
            margin: 0 0 var(--spacing-6) 0;
            color: #374151;
            font-size: 1.25rem;
            font-weight: 600;
          }
        }

        .servicesGrid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
          gap: var(--spacing-4);
        }

        .serviceCard {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: var(--border-radius-lg);
          padding: var(--spacing-5);
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
          transition: border-color 0.2s;

          &.warning {
            border-color: #f59e0b;
          }

          &.down {
            border-color: #ef4444;
          }
        }

        .serviceHeader {
          display: flex;
          align-items: center;
          gap: var(--spacing-3);
          margin-bottom: var(--spacing-4);

          .statusIcon {
            font-size: 1.25rem;
          }

          .serviceName {
            font-weight: 500;
            color: #374151;
          }
        }

        .serviceMetrics {
          display: flex;
          flex-direction: column;
          gap: var(--spacing-2);
          margin-bottom: var(--spacing-4);
        }

        .metric {
          display: flex;
          justify-content: space-between;
          align-items: center;

          .label {
            color: #6b7280;
            font-size: 0.875rem;
          }

          .value {
            font-weight: 500;
            color: #374151;
          }
        }

        .serviceLastCheck {
          border-top: 1px solid #e5e7eb;
          padding-top: var(--spacing-3);

          small {
            color: #9ca3af;
          }
        }

        @media (max-width: 768px) {
          .container {
            padding: var(--spacing-4);
          }

          .headerContent {
            flex-direction: column;
            align-items: flex-start;
            gap: var(--spacing-4);
          }

          .headerActions {
            width: 100%;
            justify-content: space-between;
          }

          .overviewGrid {
            grid-template-columns: 1fr;
          }

          .metricsGrid {
            grid-template-columns: 1fr;
          }

          .servicesGrid {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </DashboardLayout>
  );
}
