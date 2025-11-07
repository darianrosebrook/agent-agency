import React from 'react';
import Link from 'next/link';
import { Card, Button } from '@/components/ui';
import styles from './page.module.scss';

export default function SystemPage() {
  return (
    <div className={styles.system}>
      <h1>System</h1>

      <div className={styles.grid}>
        <Card>
          <div className={styles.section}>
            <h2>Health</h2>
            <p>Monitor system health and component status</p>
            <Link href="/system/health">
              <Button variant="primary">View Health</Button>
            </Link>
          </div>
        </Card>

        <Card>
          <div className={styles.section}>
            <h2>Metrics</h2>
            <p>View real-time system metrics and performance data</p>
            <Link href="/system/metrics">
              <Button variant="primary">View Metrics</Button>
            </Link>
          </div>
        </Card>

        <Card>
          <div className={styles.section}>
            <h2>Analytics</h2>
            <p>Analyze task performance and success rates</p>
            <Link href="/system/analytics">
              <Button variant="primary">View Analytics</Button>
            </Link>
          </div>
        </Card>
      </div>
    </div>
  );
}

