import { useState } from "react";
import { IterationTimeline } from "./IterationTimeline";
import { ModelPerformanceChart } from "./ModelPerformanceChart";
import { SatisficingDashboard } from "./SatisficingDashboard";

interface SelfPromptingMonitorProps {
  task: any;
  onConfigChange?: (config: any) => void;
}

import styles from "./SelfPromptingMonitor.module.scss";

export const SelfPromptingMonitor: React.FC<SelfPromptingMonitorProps> = ({
  task: _task,
  onConfigChange: _onConfigChange,
}) => {
  const [selectedIteration, setSelectedIteration] = useState<
    number | undefined
  >();
  const [viewMode, setViewMode] = useState<
    "timeline" | "performance" | "satisficing"
  >("timeline");

  const handleIterationClick = (iteration: number) => {
    setSelectedIteration(iteration);
    // onIterationSelect?.(iteration);
  };

  const currentIteration = 0; // Placeholder since we don't have events

  const isRunning = false; // Placeholder
  const isPaused = false; // Placeholder

  return (
    <div className={styles.container}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.title}>
          <h2>Self-Prompting Agent</h2>
          <span className={styles.taskId}>task-123</span>
        </div>

        <div className={styles.controls}>
          <div className={styles.status}>
            <span className={`${styles.statusBadge} ${styles.running}`}>
              RUNNING
            </span>
            <span className={styles.iteration}>
              Iteration {currentIteration} / 10
            </span>
          </div>

          <div className={styles.actions}>
            {isRunning && (
              <button
                className={`${styles.button} ${styles.secondary}`}
                onClick={() => {}}
              >
                Pause
              </button>
            )}
            {isPaused && (
              <button
                className={`${styles.button} ${styles.primary}`}
                onClick={() => {}}
              >
                Resume
              </button>
            )}
            {(isRunning || isPaused) && (
              <button
                className={`${styles.button} ${styles.danger}`}
                onClick={() => {}}
              >
                Stop
              </button>
            )}
          </div>
        </div>
      </div>

      {/* View Mode Tabs */}
      <div className={styles.tabs}>
        <button
          className={`${styles.tab} ${
            viewMode === "timeline" ? styles.active : ""
          }`}
          onClick={() => setViewMode("timeline")}
        >
          Iteration Timeline
        </button>
        <button
          className={`${styles.tab} ${
            viewMode === "performance" ? styles.active : ""
          }`}
          onClick={() => setViewMode("performance")}
        >
          Model Performance
        </button>
        <button
          className={`${styles.tab} ${
            viewMode === "satisficing" ? styles.active : ""
          }`}
          onClick={() => setViewMode("satisficing")}
        >
          Satisficing Metrics
        </button>
      </div>

      {/* Content */}
      <div className={styles.content}>
        {viewMode === "timeline" && (
          <IterationTimeline
            iterations={[1, 2, 3, 4, 5]}
            currentIteration={selectedIteration || 0}
            onIterationSelect={handleIterationClick}
            showDetails={true}
          />
        )}

        {viewMode === "performance" && (
          <ModelPerformanceChart
            data={[]}
            selectedModel="gpt-4"
            onModelSelect={() => {}}
          />
        )}

        {viewMode === "satisficing" && (
          <SatisficingDashboard
            task={{}}
            onThresholdChange={() => {}}
          />
        )}
      </div>

      {/* Real-time Events */}
      {false && (
        <div className={styles.events}>
          <h3>Recent Events</h3>
          <div className={styles.eventList}>
            {[].slice(-5)
              .reverse()
              .map((_event) => (
                <div key="event-1" className={styles.event}>
                  <span className={styles.eventType}>
                    iteration started
                  </span>
                  <span className={styles.eventTime}>
                    {new Date().toLocaleTimeString()}
                  </span>
                  <span className={styles.eventScore}>
                    Score: 0.85
                  </span>
                </div>
              ))}
          </div>
        </div>
      )}
    </div>
  );
};
