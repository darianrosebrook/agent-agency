import React from "react";
import { IterationTimelineProps } from "../../types/tasks";

import styles from "./IterationTimeline.module.scss";

export const IterationTimeline: React.FC<IterationTimelineProps> = ({
  task,
  selectedIteration,
  onIterationClick,
  showDetails = false,
}) => {
  const iterations = Array.from(
    { length: task.current_iteration },
    (_, i) => i + 1
  );
  const maxIterations = task.max_iterations;

  const getIterationStatus = (
    iteration: number
  ): "pending" | "running" | "completed" | "failed" => {
    if (iteration < task.current_iteration) {
      // Check if this iteration completed successfully
      return "completed";
    } else if (
      iteration === task.current_iteration &&
      task.status === "running"
    ) {
      return "running";
    } else if (iteration > task.current_iteration) {
      return "pending";
    }
    return "completed";
  };

  const getIterationData = (iteration: number) => {
    const modelUsage = task.model_history.find(
      (m) => m.iteration === iteration
    );
    return {
      model: modelUsage?.model_id ?? "unknown",
      latency: modelUsage?.latency_ms ?? 0,
      tokens:
        (modelUsage?.prompt_tokens ?? 0) + (modelUsage?.completion_tokens ?? 0),
      success: modelUsage?.success ?? true,
    };
  };

  return (
    <div className={styles.container}>
      <div className={styles.timeline}>
        {iterations.map((iteration) => {
          const status = getIterationStatus(iteration);
          const data = getIterationData(iteration);
          const isSelected = selectedIteration === iteration;

          return (
            <div
              key={iteration}
              className={`${styles.iteration} ${styles[status]} ${
                isSelected ? styles.selected : ""
              }`}
              onClick={() => onIterationClick?.(iteration)}
            >
              <div className={styles.iterationHeader}>
                <span className={styles.iterationNumber}>{iteration}</span>
                <span className={styles.model}>{data.model}</span>
              </div>

              <div className={styles.iterationMetrics}>
                <span className={styles.latency}>{data.latency}ms</span>
                <span className={styles.tokens}>{data.tokens}t</span>
              </div>

              {showDetails && (
                <div className={styles.iterationDetails}>
                  <div className={styles.detail}>
                    <span className={styles.label}>Model:</span>
                    <span className={styles.value}>{data.model}</span>
                  </div>
                  <div className={styles.detail}>
                    <span className={styles.label}>Latency:</span>
                    <span className={styles.value}>{data.latency}ms</span>
                  </div>
                  <div className={styles.detail}>
                    <span className={styles.label}>Tokens:</span>
                    <span className={styles.value}>{data.tokens}</span>
                  </div>
                  <div className={styles.detail}>
                    <span className={styles.label}>Status:</span>
                    <span className={styles.value}>{status}</span>
                  </div>
                </div>
              )}

              <div className={styles.connector} />
            </div>
          );
        })}

        {/* Future iterations */}
        {Array.from(
          { length: Math.max(0, maxIterations - iterations.length) },
          (_, i) => (
            <div
              key={`future-${i}`}
              className={`${styles.iteration} ${styles.pending} ${styles.future}`}
            >
              <div className={styles.iterationHeader}>
                <span className={styles.iterationNumber}>
                  {iterations.length + i + 1}
                </span>
                <span className={styles.model}>pending</span>
              </div>
              <div className={styles.connector} />
            </div>
          )
        )}
      </div>

      {/* Progress Summary */}
      <div className={styles.summary}>
        <div className={styles.metric}>
          <span className={styles.label}>Iterations Completed:</span>
          <span className={styles.value}>
            {task.current_iteration} / {maxIterations}
          </span>
        </div>
        <div className={styles.metric}>
          <span className={styles.label}>Average Latency:</span>
          <span className={styles.value}>
            {task.model_history.length > 0
              ? Math.round(
                  task.model_history.reduce((sum, m) => sum + m.latency_ms, 0) /
                    task.model_history.length
                )
              : 0}
            ms
          </span>
        </div>
        <div className={styles.metric}>
          <span className={styles.label}>Total Tokens:</span>
          <span className={styles.value}>
            {task.model_history.reduce(
              (sum, m) => sum + m.prompt_tokens + m.completion_tokens,
              0
            )}
          </span>
        </div>
        <div className={styles.metric}>
          <span className={styles.label}>Current Model:</span>
          <span className={styles.value}>
            {task.self_prompting_config.current_model}
          </span>
        </div>
      </div>
    </div>
  );
};
