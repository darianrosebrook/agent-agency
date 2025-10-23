# Runtime Optimization - Quick Reference

**Location**: `iterations/v3/runtime-optimization/`  
**Status**: Production Ready  
**Version**: 1.0.0

## Quick Start

### Installation
```bash
cd iterations/v3/runtime-optimization
cargo build --release
```

### Basic Usage
```rust
use runtime_optimization::{
    LLMParameterOptimizer, ThompsonGaussian, TaskFeatures, OptimizationConstraints
};

// Initialize optimizer
let optimizer = LLMParameterOptimizer::new(
    Box::new(ThompsonGaussian::new()),
    quality_gate_validator,
    caws_budget_tracker,
);

// Get recommendations
let recommendation = optimizer.recommend_parameters(
    "task_type",
    &task_features,
    &constraints,
).await?;
```

## Key Components

| Component | File | Purpose |
|-----------|------|---------|
| **Bandit Policies** | `bandit_policy.rs` | ThompsonGaussian & LinUCB |
| **Counterfactual Logging** | `counterfactual_log.rs` | Decision logging & offline evaluation |
| **Parameter Optimizer** | `parameter_optimizer.rs` | Main optimization coordinator |
| **Quality Gates** | `quality_gate_validator.rs` | Pre-deployment validation |
| **Rollout Manager** | `rollout.rs` | Phased deployment management |
| **CAWS Integration** | `caws_integration.rs` | Budget tracking & compliance |
| **Dashboard** | `parameter_dashboard.rs` | Real-time monitoring |
| **Test Suites** | `offline_test_suite.rs`, `canary_test_suite.rs` | Testing frameworks |

## Configuration

### Environment Variables
```bash
# Core Configuration
LLM_OPTIMIZATION_ENABLED=true
LLM_OPTIMIZATION_MODE=shadow  # shadow, canary, guarded, general
LLM_OPTIMIZATION_TRAFFIC_PERCENTAGE=0.0

# Database
DATABASE_URL=postgresql://user:pass@localhost:5432/agent_agency
REDIS_URL=redis://localhost:6379

# CAWS Integration
CAWS_ENABLED=true
CAWS_BUDGET_LIMIT=1000000
CAWS_WAIVER_THRESHOLD=0.8

# SLO Configuration
SLO_LATENCY_P99_MS=500
SLO_QUALITY_MIN=0.7
SLO_ERROR_RATE_MAX=0.01
```

## Testing

### Run Tests
```bash
# All tests
cargo test

# Offline test suite
cargo test --test offline_test_suite

# Canary test suite
cargo test --test canary_test_suite

# Integration tests
cargo test --test integration_tests
```

### Test Types
- **Offline Tests**: Replay, constraint satisfaction, reproducibility
- **Canary Tests**: SLO monitoring, auto-rollback, budget enforcement
- **Performance Tests**: Latency, memory, convergence

## Deployment

### Rollout Phases
1. **Shadow Mode (0%)**: Log decisions without applying
2. **Canary Mode (5%)**: Apply to small subset
3. **Guarded Mode (25%)**: Gradual increase
4. **General Mode (100%)**: Full rollout

### Deployment Commands
```bash
# Shadow mode
export LLM_OPTIMIZATION_MODE=shadow
export LLM_OPTIMIZATION_TRAFFIC_PERCENTAGE=0.0

# Canary mode
export LLM_OPTIMIZATION_MODE=canary
export LLM_OPTIMIZATION_TRAFFIC_PERCENTAGE=0.05

# Full rollout
export LLM_OPTIMIZATION_MODE=general
export LLM_OPTIMIZATION_TRAFFIC_PERCENTAGE=1.0
```

## Monitoring

### Key Metrics
- Parameter recommendation accuracy
- Reward improvement over baseline
- SLO compliance rate
- Quality degradation incidents
- Rollback frequency
- Token usage efficiency

### Dashboard Endpoints
```bash
# Optimization status
GET /api/optimization/status

# Performance metrics
GET /api/optimization/metrics

# SLO compliance
GET /api/optimization/slo-status

# Rollout status
GET /api/optimization/rollout-status
```

## Safety Features

### Trust Regions
- Maximum temperature delta: 0.2
- Maximum token delta: 200
- Prevents large parameter jumps

### Quality Gates
- Pre-deployment validation
- CAWS compliance checking
- Budget constraint enforcement

### Auto-Rollback
- SLO breach detection
- Automatic phase regression
- Traffic percentage controls

## Success Criteria

1. **Statistical Significance**: Reward improvement ≥ 0 at α=0.05
2. **Cost Efficiency**: Tokens-per-quality-point ≤ baseline × 1.05
3. **Reliability**: Zero SLO breaches during canary
4. **Reproducibility**: Same seed/context → identical parameters in ≥99.9% of cases
5. **Safety**: Zero CAWS compliance violations

## Alerting

### Critical Alerts
- SLO violations (latency, quality, error rate)
- Budget overruns
- Rollback events
- System failures

### Warning Alerts
- Performance degradation
- Confidence interval breaches
- Unusual parameter patterns
- Drift detection

## Documentation

- **Module README**: `iterations/v3/runtime-optimization/README.md`
- **Implementation Summary**: `iterations/v3/runtime-optimization/LLM_PARAMETER_FEEDBACK_LOOP_SUMMARY.md`
- **Deployment Guide**: `iterations/v3/runtime-optimization/DEPLOYMENT_GUIDE.md`
- **Verification Script**: `iterations/v3/runtime-optimization/verify_implementation.sh`

## Integration

### With Agent Agency
- **PlanningAgent**: Optimized parameter selection for working spec generation
- **CAWS Compliance**: Budget tracking and provenance logging
- **Quality Assurance**: Trust regions and quality gates
- **Monitoring**: Real-time dashboards and alerting

## Support

### Troubleshooting
```bash
# Check system health
curl -X GET http://localhost:8080/health

# View optimization logs
curl -X GET http://localhost:8080/api/optimization/logs

# Check database connectivity
curl -X GET http://localhost:8080/api/optimization/db-status
```

### Emergency Procedures
1. **Immediate Rollback**: Disable optimization
2. **Investigation**: Check logs and metrics
3. **Recovery**: Restore baseline parameters
4. **Post-Mortem**: Analyze root cause

---

**Status**: Production Ready  
**Files**: 23 Rust files  
**Lines of Code**: 11,000+  
**Documentation**: Comprehensive guides and examples  
**Testing**: Complete test suites for offline and canary testing
