# LLM Parameter Feedback Loop - Deployment Guide

**Version**: 1.0.0  
**Status**: Production Ready  
**Last Updated**: January 2025

## 🚀 **Quick Start**

### Prerequisites
- Rust 1.70+ (for async/await and modern features)
- PostgreSQL 13+ (for counterfactual logging)
- Redis 6+ (for caching and session management)
- CAWS integration configured

### Installation
```bash
cd iterations/v3/runtime-optimization
cargo build --release
```

## 📋 **Deployment Checklist**

### ✅ **Pre-Deployment Verification**

1. **Code Quality**
   - [ ] All tests pass: `cargo test`
   - [ ] No linting errors: `cargo clippy`
   - [ ] Documentation complete: `cargo doc`
   - [ ] Security scan clean: `cargo audit`

2. **Configuration**
   - [ ] Environment variables set
   - [ ] Database connections configured
   - [ ] Redis connections configured
   - [ ] CAWS integration enabled

3. **Infrastructure**
   - [ ] Database migrations applied
   - [ ] Monitoring systems configured
   - [ ] Alerting thresholds set
   - [ ] Backup systems enabled

### ✅ **Deployment Steps**

#### Phase 1: Shadow Mode (0% Traffic)
```bash
# 1. Deploy with shadow mode enabled
export LLM_OPTIMIZATION_MODE=shadow
export LLM_OPTIMIZATION_TRAFFIC_PERCENTAGE=0.0

# 2. Start the service
cargo run --release --bin runtime-optimization

# 3. Verify shadow logging
curl -X GET http://localhost:8080/api/optimization/status
```

#### Phase 2: Canary Mode (5% Traffic)
```bash
# 1. Update configuration
export LLM_OPTIMIZATION_MODE=canary
export LLM_OPTIMIZATION_TRAFFIC_PERCENTAGE=0.05

# 2. Monitor canary metrics
curl -X GET http://localhost:8080/api/optimization/metrics
```

#### Phase 3: Guarded Mode (25% Traffic)
```bash
# 1. Increase traffic gradually
export LLM_OPTIMIZATION_TRAFFIC_PERCENTAGE=0.25

# 2. Monitor SLO compliance
curl -X GET http://localhost:8080/api/optimization/slo-status
```

#### Phase 4: General Mode (100% Traffic)
```bash
# 1. Full rollout
export LLM_OPTIMIZATION_MODE=general
export LLM_OPTIMIZATION_TRAFFIC_PERCENTAGE=1.0

# 2. Monitor full system
curl -X GET http://localhost:8080/api/optimization/dashboard
```

## 🔧 **Configuration**

### Environment Variables

```bash
# Core Configuration
LLM_OPTIMIZATION_ENABLED=true
LLM_OPTIMIZATION_MODE=shadow  # shadow, canary, guarded, general
LLM_OPTIMIZATION_TRAFFIC_PERCENTAGE=0.0

# Database Configuration
DATABASE_URL=postgresql://user:pass@localhost:5432/agent_agency
REDIS_URL=redis://localhost:6379

# CAWS Integration
CAWS_ENABLED=true
CAWS_BUDGET_LIMIT=1000000
CAWS_WAIVER_THRESHOLD=0.8

# Bandit Policy Configuration
BANDIT_POLICY=thompson_gaussian  # thompson_gaussian, linucb
EXPLORATION_FACTOR=0.1
CONFIDENCE_THRESHOLD=0.7

# SLO Configuration
SLO_LATENCY_P99_MS=500
SLO_QUALITY_MIN=0.7
SLO_ERROR_RATE_MAX=0.01

# Rollout Configuration
ROLLOUT_SHADOW_DURATION_HOURS=24
ROLLOUT_CANARY_DURATION_HOURS=48
ROLLOUT_GUARDED_DURATION_HOURS=72
```

### Database Schema

```sql
-- Counterfactual logging table
CREATE TABLE logged_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL,
    task_type VARCHAR(255) NOT NULL,
    model_name VARCHAR(255) NOT NULL,
    context_fingerprint BIGINT NOT NULL,
    context_features JSONB NOT NULL,
    chosen_params JSONB NOT NULL,
    log_propensity DOUBLE PRECISION NOT NULL,
    outcome JSONB NOT NULL,
    policy_version VARCHAR(50) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Parameter optimization results
CREATE TABLE optimization_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type VARCHAR(255) NOT NULL,
    parameter_set JSONB NOT NULL,
    performance_metrics JSONB NOT NULL,
    confidence_interval JSONB NOT NULL,
    rollout_phase VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- SLO monitoring
CREATE TABLE slo_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_name VARCHAR(255) NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    compliant BOOLEAN NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

## 📊 **Monitoring & Observability**

### Key Metrics to Monitor

1. **Optimization Metrics**
   - Parameter recommendation accuracy
   - Reward improvement over baseline
   - Confidence interval coverage
   - Policy convergence rate

2. **Safety Metrics**
   - SLO compliance rate
   - Quality degradation incidents
   - Rollback frequency
   - Constraint violations

3. **Performance Metrics**
   - Latency percentiles (P50, P95, P99)
   - Token usage efficiency
   - Throughput rates
   - Error rates

4. **Business Metrics**
   - Cost per request
   - Quality score trends
   - User satisfaction
   - System reliability

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

# Dashboard data
GET /api/optimization/dashboard
```

## 🚨 **Alerting Configuration**

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

### Alert Examples
```yaml
# SLO Violation Alert
- name: "SLO Latency Violation"
  condition: "latency_p99 > 500ms"
  severity: "critical"
  action: "auto_rollback"

# Budget Alert
- name: "Token Budget Exceeded"
  condition: "token_usage > budget_limit * 0.9"
  severity: "warning"
  action: "notify_team"

# Quality Alert
- name: "Quality Degradation"
  condition: "quality_score < 0.7"
  severity: "critical"
  action: "investigate_and_rollback"
```

## 🔄 **Rollback Procedures**

### Automatic Rollback Triggers
1. SLO violations (latency > threshold)
2. Quality degradation (score < minimum)
3. Error rate spikes (> threshold)
4. Budget overruns (with waiver approval)

### Manual Rollback
```bash
# 1. Stop optimization
export LLM_OPTIMIZATION_ENABLED=false

# 2. Revert to baseline parameters
curl -X POST http://localhost:8080/api/optimization/rollback

# 3. Verify rollback
curl -X GET http://localhost:8080/api/optimization/status
```

### Rollback Verification
```bash
# Check rollback status
curl -X GET http://localhost:8080/api/optimization/rollback-status

# Verify baseline parameters
curl -X GET http://localhost:8080/api/optimization/baseline-parameters

# Monitor recovery
curl -X GET http://localhost:8080/api/optimization/health
```

## 🧪 **Testing Procedures**

### Pre-Deployment Testing
```bash
# 1. Run offline test suite
cargo test --test offline_test_suite

# 2. Run canary test suite
cargo test --test canary_test_suite

# 3. Run integration tests
cargo test --test integration_tests

# 4. Run performance tests
cargo test --test performance_tests
```

### Post-Deployment Testing
```bash
# 1. Verify shadow mode
curl -X POST http://localhost:8080/api/optimization/test-shadow

# 2. Test parameter recommendations
curl -X POST http://localhost:8080/api/optimization/recommend \
  -H "Content-Type: application/json" \
  -d '{"task_type": "test", "context": {...}}'

# 3. Test outcome recording
curl -X POST http://localhost:8080/api/optimization/record-outcome \
  -H "Content-Type: application/json" \
  -d '{"request_id": "...", "outcome": {...}}'
```

## 📈 **Performance Optimization**

### Tuning Parameters
```rust
// Bandit policy configuration
let thompson_config = ThompsonGaussianConfig {
    prior_mean: 0.0,
    prior_precision: 1.0,
    exploration_factor: 0.1,
};

// LinUCB configuration
let linucb_config = LinUCBConfig {
    alpha: 0.1,
    feature_dimension: 10,
    regularization: 0.01,
};

// Optimization constraints
let constraints = OptimizationConstraints {
    max_latency_ms: 500,
    max_tokens: 2000,
    require_caws: true,
    max_delta_temperature: 0.2,
    max_delta_max_tokens: 500,
};
```

### Performance Monitoring
```bash
# Monitor optimization performance
curl -X GET http://localhost:8080/api/optimization/performance

# Check convergence metrics
curl -X GET http://localhost:8080/api/optimization/convergence

# View parameter evolution
curl -X GET http://localhost:8080/api/optimization/parameter-evolution
```

## 🔒 **Security Considerations**

### Data Protection
- All parameter changes logged with provenance
- Sensitive data encrypted at rest
- Audit trails for compliance
- Access controls for optimization endpoints

### CAWS Compliance
- Token budget enforcement
- Quality gate validation
- Trust region constraints
- Rollback mechanisms

### Monitoring Security
```bash
# Check security status
curl -X GET http://localhost:8080/api/optimization/security-status

# View audit logs
curl -X GET http://localhost:8080/api/optimization/audit-logs

# Check compliance
curl -X GET http://localhost:8080/api/optimization/compliance-status
```

## 📚 **Troubleshooting**

### Common Issues

1. **High Latency**
   - Check SLO thresholds
   - Verify parameter constraints
   - Monitor resource usage

2. **Quality Degradation**
   - Review quality gates
   - Check trust region settings
   - Analyze parameter changes

3. **Budget Overruns**
   - Verify CAWS configuration
   - Check waiver thresholds
   - Monitor token usage

4. **Rollback Issues**
   - Check rollback triggers
   - Verify baseline parameters
   - Monitor recovery time

### Debug Commands
```bash
# Check system health
curl -X GET http://localhost:8080/health

# View optimization logs
curl -X GET http://localhost:8080/api/optimization/logs

# Check database connectivity
curl -X GET http://localhost:8080/api/optimization/db-status

# Verify CAWS integration
curl -X GET http://localhost:8080/api/optimization/caws-status
```

## 📞 **Support & Maintenance**

### Regular Maintenance Tasks
- [ ] Review optimization performance weekly
- [ ] Update baseline parameters monthly
- [ ] Clean up old counterfactual logs quarterly
- [ ] Review and update SLO thresholds annually

### Emergency Procedures
1. **Immediate Rollback**: Disable optimization
2. **Investigation**: Check logs and metrics
3. **Recovery**: Restore baseline parameters
4. **Post-Mortem**: Analyze root cause

### Contact Information
- **On-Call Engineer**: [Contact Info]
- **Escalation Path**: [Management Chain]
- **Documentation**: [Internal Wiki]

---

**Deployment Guide Version**: 1.0.0  
**Last Updated**: January 2025  
**Next Review**: February 2025
