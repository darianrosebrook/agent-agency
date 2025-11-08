import React from "react";
import { Card } from "@/components/ui";
import { analyticsApi } from "@/lib/api";
import { formatPercentage, formatDuration } from "@/lib/utils";
import styles from "./page.module.scss";

export default async function AnalyticsPage() {
  let taskAnalytics;
  let successRates;

  try {
    [taskAnalytics, successRates] = await Promise.allSettled([
      analyticsApi.getTaskAnalytics(),
      analyticsApi.getSuccessRates(),
    ]);
  } catch (error) {
    console.error("Failed to fetch analytics:", error);
  }

  const analytics =
    taskAnalytics?.status === "fulfilled" ? taskAnalytics.value : null;
  const rates =
    successRates?.status === "fulfilled" ? successRates.value : null;

  return (
    <div className={styles.analytics}>
      <h1>Analytics</h1>

      <div className={styles.grid}>
        {analytics && (
          <>
            <Card>
              <div className={styles.section}>
                <h2>Task Overview</h2>
                <dl className={styles.stats}>
                  <div>
                    <dt>Total Tasks</dt>
                    <dd>{analytics.total_tasks}</dd>
                  </div>
                  <div>
                    <dt>Completed</dt>
                    <dd>{analytics.completed ?? 0}</dd>
                  </div>
                  <div>
                    <dt>Failed</dt>
                    <dd>{analytics.failed ?? 0}</dd>
                  </div>
                  {analytics.in_progress !== undefined && (
                    <div>
                      <dt>In Progress</dt>
                      <dd>{analytics.in_progress}</dd>
                    </div>
                  )}
                  {analytics.paused !== undefined && (
                    <div>
                      <dt>Paused</dt>
                      <dd>{analytics.paused}</dd>
                    </div>
                  )}
                  <div>
                    <dt>Success Rate</dt>
                    <dd>
                      {typeof analytics.success_rate === "string"
                        ? analytics.success_rate
                        : formatPercentage(analytics.success_rate)}
                    </dd>
                  </div>
                  {analytics.average_execution_time_ms && (
                    <div>
                      <dt>Avg Execution Time</dt>
                      <dd>
                        {formatDuration(analytics.average_execution_time_ms)}
                      </dd>
                    </div>
                  )}
                </dl>
              </div>
            </Card>

            {analytics.tasks_by_status &&
              Object.keys(analytics.tasks_by_status).length > 0 && (
                <Card>
                  <div className={styles.section}>
                    <h2>Tasks by Status</h2>
                    <ul className={styles.list}>
                      {Object.entries(analytics.tasks_by_status).map(
                        ([status, count]) => (
                          <li key={status} className={styles.listItem}>
                            <span className={styles.status}>{status}</span>
                            <span className={styles.count}>
                              {count as number}
                            </span>
                          </li>
                        )
                      )}
                    </ul>
                  </div>
                </Card>
              )}
          </>
        )}

        {rates && (
          <Card>
            <div className={styles.section}>
              <h2>Success Rates</h2>
              <dl className={styles.stats}>
                <div>
                  <dt>Overall Success Rate</dt>
                  <dd>
                    {rates.overall_success_rate !== undefined
                      ? typeof rates.overall_success_rate === "string"
                        ? rates.overall_success_rate
                        : formatPercentage(rates.overall_success_rate)
                      : "N/A"}
                  </dd>
                </div>
              </dl>
              {rates.success_rate_by_worker &&
                Object.keys(rates.success_rate_by_worker).length > 0 && (
                  <div className={styles.workerRates}>
                    <h3>By Worker</h3>
                    <ul className={styles.list}>
                      {Object.entries(rates.success_rate_by_worker).map(
                        ([worker, rate]) => (
                          <li key={worker} className={styles.listItem}>
                            <span className={styles.worker}>{worker}</span>
                            <span className={styles.rate}>
                              {typeof rate === "string"
                                ? rate
                                : formatPercentage(rate)}
                            </span>
                          </li>
                        )
                      )}
                    </ul>
                  </div>
                )}
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}
