# Predictive Learning System Enhancements (Tier 1)

## Scope & Objectives
- Introduce a `predictive` module inside `reflexive-learning` that houses V3's `PredictiveLearningSystem`.
- Implement the three core predictive components called out in the task: `PerformancePredictor`, `StrategyOptimizer`, and `ResourcePredictor`.
- Expose orchestration via `PredictiveLearningSystem::learn_and_predict` so reflexive learning can consume proactive insights (performance, strategy, resource).
- Stay within existing `reflexive-learning` scope; no cross-crate behavioral changes beyond the new public API and logs.

## Architecture Sketch
```
TaskOutcome (+ optional snapshot data)
        │
        ▼
PredictiveLearningSystem
    ├── PerformancePredictor::predict_future
    │       - blends recent progress metrics with outcome signals
    │       - outputs quality/success/time forecasts + risk
    ├── StrategyOptimizer::optimize_strategies
    │       - examines outcome category + trend data
    │       - suggests proactive strategy & weighted adjustments
    └── ResourcePredictor::predict_needs
            - infers CPU/memory/token/time load & pressure level
            - emits rationale for adaptive allocation
```
- Shared helper `TaskLearningSnapshot` encapsulates task outcome, optional progress metrics, and resource statistics.
- Insights aggregated into `PredictiveLearningInsights` struct returned to callers.

## Data Plan
- Primary input: `TaskOutcome` plus optional `ProgressMetrics` and `ResourceUtilization` captured from the reflexive loop.
- Leverage light-weight statistical heuristics (moving averages, categorical weights) computed in-memory; no persistent schema changes required.
- Preserve deterministic calculations by parameterizing smoothing constants (`PredictiveLearningConfig`) and avoiding real-time randomness.

## Observability
- Add structured `tracing` spans around prediction/optimization to surface component-level timing.
- Emit debug-level fields with predicted scores, recommended strategies, and resource pressure for downstream metrics ingestion.
- Surface prediction confidence in returned insights so higher layers can fan-out to metrics (`consensus-time`, `agent-success-rate`) already tracked in spec.

## Test Matrix
| Test Type | Scenario | Assertion |
|-----------|----------|-----------|
| Unit | Success outcome with strong quality indicators | Performance predictor yields high success probability & low risk |
| Unit | Partial success with remediation issues | Strategy optimizer recommends adaptive strategy with targeted adjustments |
| Unit | Failure due to resource exhaustion | Resource predictor flags high pressure and elevated resource needs |
| Unit | Timeout with partial progress metrics | Performance predictor lengthens completion time, resource predictor estimates retry cost |
| Integration | `PredictiveLearningSystem::learn_and_predict` on contrasting outcomes | Aggregated insights combine component outputs coherently and preserve confidence fields |

## Acceptance Alignment
- Supports working spec acceptance A3 (learning adaptation) by enabling proactive adjustments.
- Reinforces invariants around auditable decisions via structured outputs and logging.
- Complements Adaptive Resource Manager work without exceeding current change budget.
