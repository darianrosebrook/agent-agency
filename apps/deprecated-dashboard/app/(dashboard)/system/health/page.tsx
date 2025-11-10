import React from "react";
import { Card, Badge } from "@/components/ui";
import { systemApi } from "@/lib/api";
import { formatRelativeTime } from "@/lib/utils";
import styles from "./page.module.scss";

export default async function SystemHealthPage() {
  let health;

  try {
    health = await systemApi.getSystemHealth();
  } catch (error) {
    console.error("Failed to fetch system health:", error);
  }

  const getStatusVariant = (status: string) => {
    switch (status) {
      case "healthy":
        return "success";
      case "degraded":
        return "warning";
      case "unhealthy":
        return "error";
      default:
        return "default";
    }
  };

  return (
    <div className={styles.health}>
      <h1>System Health</h1>

      {health ? (
        <div className={styles.grid}>
          <Card>
            <div className={styles.overview}>
              <h2>Overall Status</h2>
              <Badge
                variant={getStatusVariant(health.status)}
                className={styles.status}
              >
                {health.status}
              </Badge>
              <p className={styles.timestamp}>
                Last checked: {formatRelativeTime(health.timestamp)}
              </p>
            </div>
          </Card>

          {health.components && Object.keys(health.components).length > 0 && (
            <Card>
              <div className={styles.components}>
                <h2>Component Health</h2>
                <ul className={styles.componentList}>
                  {Object.entries(health.components).map(
                    ([name, component]) => (
                      <li key={name} className={styles.component}>
                        <div className={styles.componentHeader}>
                          <span className={styles.componentName}>{name}</span>
                          <Badge variant={getStatusVariant(component.status)}>
                            {component.status}
                          </Badge>
                        </div>
                        {component.message && (
                          <p className={styles.componentMessage}>
                            {component.message}
                          </p>
                        )}
                        {component.last_check && (
                          <p className={styles.componentTime}>
                            Last check:{" "}
                            {formatRelativeTime(component.last_check)}
                          </p>
                        )}
                      </li>
                    )
                  )}
                </ul>
              </div>
            </Card>
          )}
          {health.database && (
            <Card>
              <div className={styles.components}>
                <h2>Database</h2>
                <div className={styles.component}>
                  <div className={styles.componentHeader}>
                    <span className={styles.componentName}>Database</span>
                    <Badge variant={getStatusVariant(health.database.status)}>
                      {health.database.status}
                    </Badge>
                  </div>
                </div>
              </div>
            </Card>
          )}
        </div>
      ) : (
        <Card>
          <p>Unable to fetch system health information</p>
        </Card>
      )}
    </div>
  );
}
