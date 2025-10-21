import React from 'react';
import { SatisficingDashboardProps } from '../../types/tasks';

import styles from './SatisficingDashboard.module.scss';

export const SatisficingDashboard: React.FC<SatisficingDashboardProps> = ({
  metrics,
  thresholds,
  recommendations,
}) => {
  const getEfficiencyScore = () => {
    if (metrics.stopped_early && metrics.quality_delta > thresholds.min_improvement) {
      return 'excellent';
    } else if (metrics.ceiling_detected) {
      return 'good';
    } else if (metrics.cost_benefit_ratio < thresholds.cost_benefit_ratio) {
      return 'poor';
    }
    return 'adequate';
  };

  const efficiencyScore = getEfficiencyScore();

  const getThresholdStatus = (value: number, threshold: number, higherIsBetter = true) => {
    if (higherIsBetter) {
      return value >= threshold ? 'good' : value >= threshold * 0.8 ? 'warning' : 'bad';
    } else {
      return value <= threshold ? 'good' : value <= threshold * 1.2 ? 'warning' : 'bad';
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h3>Satisficing Analysis</h3>
        <div className={`${styles.efficiencyBadge} ${styles[efficiencyScore]}`}>
          {efficiencyScore.toUpperCase()} EFFICIENCY
        </div>
      </div>

      <div className={styles.metrics}>
        {/* Key Metrics */}
        <div className={styles.metricGrid}>
          <div className={styles.metric}>
            <h4>Quality Delta</h4>
            <div className={`${styles.value} ${metrics.quality_delta >= 0 ? styles.positive : styles.negative}`}>
              {metrics.quality_delta >= 0 ? '+' : ''}{metrics.quality_delta.toFixed(3)}
            </div>
            <div className={styles.threshold}>
              Threshold: {thresholds.min_improvement}
            </div>
          </div>

          <div className={styles.metric}>
            <h4>Iterations Saved</h4>
            <div className={`${styles.value} ${metrics.iterations_saved > 0 ? styles.positive : styles.neutral}`}>
              {metrics.iterations_saved > 0 ? '+' : ''}{metrics.iterations_saved}
            </div>
            <div className={styles.threshold}>
              Max: {thresholds.quality_ceiling_budget}
            </div>
          </div>

          <div className={styles.metric}>
            <h4>Cost-Benefit Ratio</h4>
            <div className={`${styles.value} ${getThresholdStatus(metrics.cost_benefit_ratio, thresholds.cost_benefit_ratio)}`}>
              {metrics.cost_benefit_ratio.toFixed(3)}
            </div>
            <div className={styles.threshold}>
              Min: {thresholds.cost_benefit_ratio}
            </div>
          </div>

          <div className={styles.metric}>
            <h4>Early Stopping</h4>
            <div className={`${styles.value} ${metrics.stopped_early ? styles.good : styles.neutral}`}>
              {metrics.stopped_early ? 'YES' : 'NO'}
            </div>
            <div className={styles.threshold}>
              Ceiling Detected: {metrics.ceiling_detected ? 'YES' : 'NO'}
            </div>
          </div>
        </div>

        {/* Decision Flow Visualization */}
        <div className={styles.decisionFlow}>
          <h4>Decision Flow</h4>
          <div className={styles.flow}>
            <div className={`${styles.step} ${styles.completed}`}>
              <div className={styles.stepNumber}>1</div>
              <div className={styles.stepLabel}>Start Iteration</div>
            </div>

            <div className={styles.arrow}>→</div>

            <div className={`${styles.step} ${styles.completed}`}>
              <div className={styles.stepNumber}>2</div>
              <div className={styles.stepLabel}>Evaluate Quality</div>
            </div>

            <div className={styles.arrow}>→</div>

            <div className={`${styles.step} ${metrics.stopped_early ? styles.earlyStop : styles.completed}`}>
              <div className={styles.stepNumber}>3</div>
              <div className={styles.stepLabel}>
                Check Satisficing
                {metrics.stopped_early && <span className={styles.stopReason}>(Early Stop)</span>}
              </div>
            </div>

            <div className={styles.arrow}>→</div>

            <div className={`${styles.step} ${metrics.ceiling_detected ? styles.ceiling : metrics.stopped_early ? styles.completed : styles.pending}`}>
              <div className={styles.stepNumber}>4</div>
              <div className={styles.stepLabel}>
                Quality Ceiling Check
                {metrics.ceiling_detected && <span className={styles.stopReason}>(Ceiling Detected)</span>}
              </div>
            </div>

            <div className={styles.arrow}>→</div>

            <div className={`${styles.step} ${styles.completed}`}>
              <div className={styles.stepNumber}>5</div>
              <div className={styles.stepLabel}>Generate Refinement</div>
            </div>
          </div>
        </div>

        {/* Recommendations */}
        {recommendations.length > 0 && (
          <div className={styles.recommendations}>
            <h4>Recommendations</h4>
            <ul>
              {recommendations.map((rec, index) => (
                <li key={index} className={styles.recommendation}>
                  {rec}
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Threshold Analysis */}
        <div className={styles.thresholds}>
          <h4>Threshold Analysis</h4>
          <div className={styles.thresholdGrid}>
            <div className={styles.threshold}>
              <span className={styles.label}>Min Improvement:</span>
              <span className={`${styles.value} ${getThresholdStatus(metrics.quality_delta, thresholds.min_improvement)}`}>
                {metrics.quality_delta.toFixed(3)} / {thresholds.min_improvement}
              </span>
              <span className={styles.status}>
                {metrics.quality_delta >= thresholds.min_improvement ? '✓ Met' : '✗ Below'}
              </span>
            </div>

            <div className={styles.threshold}>
              <span className={styles.label}>Ceiling Budget:</span>
              <span className={`${styles.value} ${metrics.iterations_saved <= thresholds.quality_ceiling_budget ? styles.good : styles.warning}`}>
                {metrics.iterations_saved} / {thresholds.quality_ceiling_budget}
              </span>
              <span className={styles.status}>
                {metrics.iterations_saved <= thresholds.quality_ceiling_budget ? '✓ Within Budget' : '⚠ Over Budget'}
              </span>
            </div>

            <div className={styles.threshold}>
              <span className={styles.label}>Cost-Benefit:</span>
              <span className={`${styles.value} ${getThresholdStatus(metrics.cost_benefit_ratio, thresholds.cost_benefit_ratio)}`}>
                {metrics.cost_benefit_ratio.toFixed(3)} / {thresholds.cost_benefit_ratio}
              </span>
              <span className={styles.status}>
                {metrics.cost_benefit_ratio >= thresholds.cost_benefit_ratio ? '✓ Good Ratio' : '⚠ Poor Ratio'}
              </span>
            </div>
          </div>
        </div>

        {/* Performance Insights */}
        <div className={styles.insights}>
          <h4>Performance Insights</h4>
          <div className={styles.insightGrid}>
            <div className={styles.insight}>
              <h5>Early Stopping Effectiveness</h5>
              <p>
                {metrics.stopped_early
                  ? `Successfully stopped ${metrics.iterations_saved} iterations early, saving computational resources while maintaining quality.`
                  : "Did not trigger early stopping. Consider adjusting satisficing thresholds if quality plateaued."}
              </p>
            </div>

            <div className={styles.insight}>
              <h5>Quality Improvement</h5>
              <p>
                {metrics.quality_delta > 0
                  ? `Achieved ${metrics.quality_delta.toFixed(3)} quality improvement across iterations.`
                  : `Quality ${Math.abs(metrics.quality_delta).toFixed(3)} units worse than initial attempt.`}
              </p>
            </div>

            <div className={styles.insight}>
              <h5>Resource Efficiency</h5>
              <p>
                Cost-benefit ratio of {metrics.cost_benefit_ratio.toFixed(3)} indicates
                {metrics.cost_benefit_ratio >= thresholds.cost_benefit_ratio
                  ? " efficient use of computational resources."
                  : " potential for optimization in iteration strategy."}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
