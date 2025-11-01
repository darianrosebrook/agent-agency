/**
 * Database Connections Component
 * Displays active database connections and their status
 * 
 * @author @darianrosebrook
 */

"use client";

import { useState, useEffect } from "react";
import { Database, Activity, AlertTriangle, CheckCircle } from "lucide-react";
import { Text } from "@/design-system/primitives";
import styles from "./DatabaseConnections.module.scss";

interface DatabaseConnection {
  id: string;
  name: string;
  type: 'postgresql' | 'mysql' | 'mongodb' | 'redis';
  status: 'connected' | 'disconnected' | 'warning';
  host: string;
  port: number;
  database: string;
  lastActivity: Date;
  queryCount: number;
  avgResponseTime: number;
}

export default function DatabaseConnections() {
  const [connections, setConnections] = useState<DatabaseConnection[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadConnections = async () => {
      setLoading(true);
      try {
        // Mock data for demonstration
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        setConnections([
          {
            id: '1',
            name: 'Primary PostgreSQL',
            type: 'postgresql',
            status: 'connected',
            host: 'db-primary.example.com',
            port: 5432,
            database: 'agent_agency',
            lastActivity: new Date(Date.now() - 30000),
            queryCount: 1247,
            avgResponseTime: 12
          },
          {
            id: '2',
            name: 'Redis Cache',
            type: 'redis',
            status: 'connected',
            host: 'redis.example.com',
            port: 6379,
            database: 'cache',
            lastActivity: new Date(Date.now() - 5000),
            queryCount: 8923,
            avgResponseTime: 2
          },
          {
            id: '3',
            name: 'Analytics MongoDB',
            type: 'mongodb',
            status: 'warning',
            host: 'mongo-analytics.example.com',
            port: 27017,
            database: 'analytics',
            lastActivity: new Date(Date.now() - 300000),
            queryCount: 456,
            avgResponseTime: 45
          }
        ]);
      } catch (error) {
        console.error('Failed to load database connections:', error);
      } finally {
        setLoading(false);
      }
    };

    loadConnections();
  }, []);

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'connected':
        return <CheckCircle className={styles.statusIcon} />;
      case 'warning':
        return <AlertTriangle className={styles.statusIcon} />;
      default:
        return <Database className={styles.statusIcon} />;
    }
  };

  const getStatusClass = (status: string) => {
    switch (status) {
      case 'connected':
        return styles.connected;
      case 'warning':
        return styles.warning;
      default:
        return styles.disconnected;
    }
  };

  if (loading) {
    return (
      <div className={styles.loading}>
        <div className={styles.spinner}></div>
        <Text variant="paragraph-medium" color="secondary">
          Loading database connections...
        </Text>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <Database className={styles.headerIcon} />
        <Text variant="h3" className={styles.title}>
          Database Connections
        </Text>
        <Text variant="paragraph-medium" color="secondary">
          {connections.length} active connections
        </Text>
      </div>

      <div className={styles.connectionsList}>
        {connections.map((connection) => (
          <div key={connection.id} className={styles.connectionCard}>
            <div className={styles.connectionHeader}>
              <div className={styles.connectionInfo}>
                <div className={styles.connectionName}>
                  {getStatusIcon(connection.status)}
                  <Text variant="paragraph-large" weight="medium">
                    {connection.name}
                  </Text>
                </div>
                <div className={`${styles.statusIndicator} ${getStatusClass(connection.status)}`}>
                  <Text variant="paragraph-small" weight="medium">
                    {connection.status.toUpperCase()}
                  </Text>
                </div>
              </div>
              <div className={styles.connectionDetails}>
                <Text variant="paragraph-small" color="secondary">
                  {connection.host}:{connection.port}
                </Text>
                <Text variant="paragraph-small" color="secondary">
                  {connection.database}
                </Text>
              </div>
            </div>

            <div className={styles.connectionMetrics}>
              <div className={styles.metric}>
                <Activity className={styles.metricIcon} />
                <div className={styles.metricContent}>
                  <Text variant="paragraph-small" color="secondary">
                    Queries
                  </Text>
                  <Text variant="paragraph-medium" weight="medium">
                    {connection.queryCount.toLocaleString()}
                  </Text>
                </div>
              </div>
              <div className={styles.metric}>
                <div className={styles.metricContent}>
                  <Text variant="paragraph-small" color="secondary">
                    Avg Response
                  </Text>
                  <Text variant="paragraph-medium" weight="medium">
                    {connection.avgResponseTime}ms
                  </Text>
                </div>
              </div>
              <div className={styles.metric}>
                <div className={styles.metricContent}>
                  <Text variant="paragraph-small" color="secondary">
                    Last Activity
                  </Text>
                  <Text variant="paragraph-medium" weight="medium">
                    {Math.floor((Date.now() - connection.lastActivity.getTime()) / 1000)}s ago
                  </Text>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
