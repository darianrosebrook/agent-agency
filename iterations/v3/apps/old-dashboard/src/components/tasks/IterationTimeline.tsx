import React from "react";

import styles from "./IterationTimeline.module.scss";

interface IterationTimelineProps {
  iterations: any[];
  currentIteration?: number;
  onIterationSelect?: (iteration: number) => void;
  showDetails?: boolean;
}

export const IterationTimeline: React.FC<IterationTimelineProps> = ({
  iterations,
  currentIteration,
  onIterationSelect: _onIterationSelect,
  showDetails = false,
}) => {
  // const iterations = Array.from(
  //   { length: task.current_iteration },
  //   (_, i) => i + 1
  // );
  const maxIterations = 10; // Default max iterations

  const getIterationStatus = (
    iteration: number
  ): "pending" | "running" | "completed" | "failed" => {
    if (iteration < (currentIteration || 0)) {
      // Check if this iteration completed successfully
      return "completed";
    } else if (
      iteration === (currentIteration || 0)
    ) {
      return "running";
    } else if (iteration > (currentIteration || 0)) {
      return "pending";
    }
    return "completed";
  };

  const getIterationData = (_iteration: number) => {
    // Placeholder data since we don't have task object
    return {
      model: "unknown",
      latency: 0,
      tokens: 0,
      success: true,
    };
  };

  return (
    <div className={styles.container}>
      <div className={styles.timeline}>
        {iterations.map((iteration) => {
          const status = getIterationStatus(iteration);
          const data = getIterationData(iteration);
          const isSelected = currentIteration === iteration;

          return (
            <div
              key={iteration}
              className={`${styles.iteration} ${styles[status]} ${
                isSelected ? styles.selected : ""
              }`}
              onClick={() => _onIterationSelect?.(iteration)}
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
            {currentIteration || 0} / {maxIterations}
          </span>
        </div>
        <div className={styles.metric}>
          <span className={styles.label}>Average Latency:</span>
          <span className={styles.value}>
            {0} ms
          </span>
        </div>
        <div className={styles.metric}>
          <span className={styles.label}>Total Tokens:</span>
          <span className={styles.value}>
            {0}
          </span>
        </div>
        <div className={styles.metric}>
          <span className={styles.label}>Current Model:</span>
          <span className={styles.value}>
            {"unknown"}
          </span>
        </div>
      </div>
    </div>
  );
};
