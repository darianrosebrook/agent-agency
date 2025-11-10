import React from 'react';
import { Card } from '@/components/ui';
import { systemApi } from '@/lib/api';
import { formatPercentage } from '@/lib/utils';
import styles from './page.module.scss';

export default async function MetricsPage() {
  let metrics;

  try {
    metrics = await systemApi.getSystemMetrics();
  } catch (error) {
    console.error('Failed to fetch metrics:', error);
  }

  return (
    <div className={styles.metrics}>
      <h1>System Metrics</h1>

      {metrics ? (
        <div className={styles.grid}>
          {metrics.cpu_usage !== undefined && (
            <Card>
              <div className={styles.metric}>
                <h2>CPU Usage</h2>
                <div className={styles.value}>{formatPercentage(metrics.cpu_usage)}</div>
              </div>
            </Card>
          )}

          {metrics.memory_usage !== undefined && (
            <Card>
              <div className={styles.metric}>
                <h2>Memory Usage</h2>
                <div className={styles.value}>{formatPercentage(metrics.memory_usage)}</div>
              </div>
            </Card>
          )}

          {metrics.disk_usage !== undefined && (
            <Card>
              <div className={styles.metric}>
                <h2>Disk Usage</h2>
                <div className={styles.value}>{formatPercentage(metrics.disk_usage)}</div>
              </div>
            </Card>
          )}

          {metrics.network_io && (
            <Card>
              <div className={styles.metric}>
                <h2>Network I/O</h2>
                <div className={styles.network}>
                  <div>
                    <span>Sent: </span>
                    <strong>{metrics.network_io.bytes_sent.toLocaleString()} bytes</strong>
                  </div>
                  <div>
                    <span>Received: </span>
                    <strong>{metrics.network_io.bytes_received.toLocaleString()} bytes</strong>
                  </div>
                </div>
              </div>
            </Card>
          )}
        </div>
      ) : (
        <Card>
          <p>Unable to fetch system metrics</p>
        </Card>
      )}
    </div>
  );
}

