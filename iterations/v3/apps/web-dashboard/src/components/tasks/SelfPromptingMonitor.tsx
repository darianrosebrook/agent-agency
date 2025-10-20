import React, { useState, useEffect } from 'react';
import { SelfPromptingMonitorProps } from '../../types/tasks';
import { IterationTimeline } from './IterationTimeline';
import { ModelPerformanceChart } from './ModelPerformanceChart';
import { SatisficingDashboard } from './SatisficingDashboard';

import styles from './SelfPromptingMonitor.module.scss';

export const SelfPromptingMonitor: React.FC<SelfPromptingMonitorProps> = ({
  task,
  events = [],
  onModelSwitch,
  onIterationSelect,
  onPause,
  onResume,
  onStop,
}) => {
  const [selectedIteration, setSelectedIteration] = useState<number | undefined>();
  const [viewMode, setViewMode] = useState<'timeline' | 'performance' | 'satisficing'>('timeline');

  const handleIterationClick = (iteration: number) => {
    setSelectedIteration(iteration);
    onIterationSelect?.(iteration);
  };

  const currentIteration = events
    .filter(e => e.type === 'iteration_started')
    .length;

  const isRunning = task.status === 'running';
  const isPaused = task.status === 'paused';

  return (
    <div className={styles.container}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.title}>
          <h2>Self-Prompting Agent</h2>
          <span className={styles.taskId}>{task.id}</span>
        </div>

        <div className={styles.controls}>
          <div className={styles.status}>
            <span className={`${styles.statusBadge} ${styles[task.status]}`}>
              {task.status.toUpperCase()}
            </span>
            <span className={styles.iteration}>
              Iteration {currentIteration} / {task.max_iterations}
            </span>
          </div>

          <div className={styles.actions}>
            {isRunning && onPause && (
              <button
                className={`${styles.button} ${styles.secondary}`}
                onClick={onPause}
              >
                Pause
              </button>
            )}
            {isPaused && onResume && (
              <button
                className={`${styles.button} ${styles.primary}`}
                onClick={onResume}
              >
                Resume
              </button>
            )}
            {(isRunning || isPaused) && onStop && (
              <button
                className={`${styles.button} ${styles.danger}`}
                onClick={onStop}
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
          className={`${styles.tab} ${viewMode === 'timeline' ? styles.active : ''}`}
          onClick={() => setViewMode('timeline')}
        >
          Iteration Timeline
        </button>
        <button
          className={`${styles.tab} ${viewMode === 'performance' ? styles.active : ''}`}
          onClick={() => setViewMode('performance')}
        >
          Model Performance
        </button>
        <button
          className={`${styles.tab} ${viewMode === 'satisficing' ? styles.active : ''}`}
          onClick={() => setViewMode('satisficing')}
        >
          Satisficing Metrics
        </button>
      </div>

      {/* Content */}
      <div className={styles.content}>
        {viewMode === 'timeline' && (
          <IterationTimeline
            task={task}
            selectedIteration={selectedIteration}
            onIterationClick={handleIterationClick}
            showDetails={true}
          />
        )}

        {viewMode === 'performance' && (
          <ModelPerformanceChart
            models={task.self_prompting_config.models}
            timeRange="24h"
            onModelSelect={onModelSwitch}
          />
        )}

        {viewMode === 'satisficing' && (
          <SatisficingDashboard
            metrics={task.satisficing_metrics}
            thresholds={{
              min_improvement: task.self_prompting_config.min_improvement_threshold,
              quality_ceiling_budget: task.self_prompting_config.quality_ceiling_budget,
              cost_benefit_ratio: task.self_prompting_config.cost_benefit_ratio_threshold,
            }}
            recommendations={[]} // TODO: Generate recommendations from events
          />
        )}
      </div>

      {/* Real-time Events */}
      {events.length > 0 && (
        <div className={styles.events}>
          <h3>Recent Events</h3>
          <div className={styles.eventList}>
            {events.slice(-5).reverse().map((event) => (
              <div key={event.event_id} className={styles.event}>
                <span className={styles.eventType}>{event.type.replace('_', ' ')}</span>
                <span className={styles.eventTime}>
                  {new Date(event.timestamp).toLocaleTimeString()}
                </span>
                {event.data.score && (
                  <span className={styles.eventScore}>
                    Score: {event.data.score.toFixed(2)}
                  </span>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
