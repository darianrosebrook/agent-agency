# Integration Tests - Agent Agency V3

## Overview

Comprehensive integration testing suite to ensure all modules work together seamlessly. Tests verify:
- Inter-module communication and data flow
- Health monitoring and alerting coordination
- Learning systems accessing tool ecosystem
- Orchestration coordinating across all components
- Failure scenarios and recovery mechanisms
- End-to-end workflow execution

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Orchestration │◄──►│ Health Monitor  │◄──►│    Alerting     │
│                 │    │                 │    │                 │
│ - Task dispatch │    │ - System health │    │ - Notifications │
│ - Worker coord  │    │ - Metrics        │    │ - Escalation    │
│ - Audit logging │    │ - Resource usage │    │ - Resolution    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         ▼                        ▼                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Claim Extract  │◄──►│   Tool Ecosystem│◄──►│   Learning      │
│                 │    │                 │    │   Systems       │
│ - Multi-modal   │    │ - MCP tools     │    │                 │
│ - Verification  │    │ - Evidence coll │    │ - Adaptation    │
│ - Provenance    │    │ - Code analysis │    │ - Optimization  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         ▼                        ▼                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Apple Silicon   │◄──►│   Workers       │◄──►│   Database      │
│                 │    │                 │    │                 │
│ - Core ML       │    │ - Distributed   │    │ - Persistence   │
│ - ANE accel     │    │ - Circuit break │    │ - Audit logs    │
│ - Async infer   │    │ - Service disc  │    │ - Migration     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Test Categories

### 1. Infrastructure Integration Tests
- **Docker Compose orchestration**
- **Database connectivity and migrations**
- **Network communication between services**
- **Service discovery and registration**

### 2. Core Functionality Integration Tests
- **Claim extraction → multimodal verification**
- **Core ML inference → worker execution**
- **Learning systems → tool ecosystem access**

### 3. Monitoring & Observability Integration Tests
- **Health monitoring → alerting → orchestration**
- **Audit trail persistence → query → export**
- **Metrics collection → dashboard → alerting**

### 4. End-to-End Workflow Integration Tests
- **Complete task lifecycle from submission to completion**
- **Failure scenarios and recovery workflows**
- **Performance under load with monitoring**

### 5. Chaos Engineering Integration Tests
- **Service failures and recovery**
- **Network partitions and healing**
- **Resource exhaustion scenarios**

## Running Integration Tests

```bash
# Run all integration tests
cargo test --test integration -- --nocapture

# Run specific test category
cargo test --test integration -- health_monitoring --nocapture

# Run with Docker Compose
docker-compose -f docker-compose.test.yml up -d
cargo test --test integration -- docker --nocapture
docker-compose -f docker-compose.test.yml down

# Run chaos tests
cargo test --test integration -- chaos --nocapture
```

## Test Structure

```
tests/integration/
├── mod.rs                    # Integration test entry point
├── infrastructure/          # Docker, DB, networking tests
│   ├── docker.rs
│   ├── database.rs
│   └── networking.rs
├── core_functionality/      # Core feature integration
│   ├── claim_extraction.rs
│   ├── apple_silicon.rs
│   └── workers.rs
├── monitoring/              # Health, alerts, audit
│   ├── health_monitoring.rs
│   ├── alerting.rs
│   └── audit_trails.rs
├── workflows/               # End-to-end scenarios
│   ├── task_lifecycle.rs
│   ├── failure_recovery.rs
│   └── performance.rs
├── chaos/                   # Failure injection
│   ├── service_failures.rs
│   ├── network_partitions.rs
│   └── resource_exhaustion.rs
└── fixtures/                # Test data and setup
    ├── test_data.rs
    └── setup.rs
```

## Key Integration Points Tested

### Health Monitoring ↔ Alerting ↔ Orchestration
```
Health Monitor detects issue → Alerting system notified → Orchestration responds
     ↓                           ↓                           ↓
Metrics collected         Notifications sent         Tasks dispatched
Resource thresholds       Escalation rules          Recovery actions
```

### Claim Extraction ↔ Tool Ecosystem ↔ Learning
```
Claims extracted → Tools analyze code → Learning adapts behavior
     ↓                ↓                    ↓
Multi-modal data   Evidence collected    Optimization applied
Verification runs  Analysis performed    Performance improved
```

### Core ML ↔ Workers ↔ Orchestration
```
Inference requested → Worker executes → Orchestration coordinates
     ↓                   ↓                    ↓
ANE acceleration     Circuit breaker        Task scheduling
Async processing    Retry logic            Audit logging
```

### Database ↔ Audit Trails ↔ Monitoring
```
Events logged → Database persisted → Monitoring queries
     ↓             ↓                      ↓
Transactions        Indexes/optimization   Dashboards/alerts
Migration safety    Query performance      Real-time metrics
```

## Test Data Management

### Shared Test Fixtures
- **Sample claims** for extraction testing
- **Model files** for Core ML/Candle testing
- **Worker configurations** for distributed testing
- **Audit events** for persistence testing

### Test Database
- **Isolated schema** per test run
- **Migration verification** before tests
- **Cleanup procedures** after tests
- **Performance baselines** for queries

### Mock Services
- **External APIs** for controlled testing
- **Slow/failing services** for chaos testing
- **Network proxies** for partition testing

## Performance Benchmarks

### Response Time SLAs
- **API endpoints**: P95 < 250ms
- **Inference tasks**: P95 < 500ms
- **Database queries**: P95 < 100ms
- **End-to-end workflows**: P95 < 2s

### Throughput Targets
- **Concurrent tasks**: 100+ simultaneous
- **Database operations**: 1000+ ops/sec
- **Network requests**: 500+ req/sec

### Resource Limits
- **Memory usage**: < 512MB per service
- **CPU usage**: < 80% sustained
- **Disk I/O**: < 100MB/sec
- **Network bandwidth**: < 50MB/sec

## Failure Scenarios Tested

### Service Failures
- **Worker crashes**: Recovery and redistribution
- **Database outage**: Circuit breaker activation
- **Network issues**: Retry and fallback logic

### Resource Exhaustion
- **Memory pressure**: Garbage collection and cleanup
- **CPU saturation**: Load shedding and queuing
- **Disk full**: Rotation and cleanup

### Data Corruption
- **Invalid inputs**: Validation and rejection
- **Partial writes**: Transaction rollback
- **Stale data**: Cache invalidation

## Monitoring Integration

### Metrics Collected
- **Service health**: Uptime, response times, error rates
- **Resource usage**: CPU, memory, disk, network
- **Business metrics**: Tasks completed, claims verified, models served
- **Performance metrics**: Latency percentiles, throughput rates

### Alert Conditions
- **Critical**: Service down, data loss, security breach
- **Warning**: High latency, resource exhaustion, error spikes
- **Info**: Configuration changes, deployment events

### Dashboard Verification
- **Real-time metrics**: Current system state
- **Historical trends**: Performance over time
- **Alert status**: Active incidents and resolutions
- **Capacity planning**: Resource utilization forecasts

## Continuous Integration

### Test Execution
- **Pre-commit**: Unit tests only (fast feedback)
- **Merge**: Integration tests (comprehensive validation)
- **Nightly**: Chaos and performance tests (regression detection)

### Environment Management
- **Test isolation**: Separate databases and networks
- **Resource cleanup**: Automatic teardown after tests
- **Parallel execution**: Tests run concurrently for speed
- **Flaky test detection**: Retry logic and reporting

## Troubleshooting Integration Issues

### Common Problems
- **Service startup timing**: Race conditions between services
- **Network connectivity**: DNS resolution and port conflicts
- **Database state**: Migration issues and data consistency
- **Resource contention**: Memory/CPU limits in test environment

### Debugging Tools
- **Service logs**: Centralized logging with correlation IDs
- **Network monitoring**: Traffic analysis and latency measurement
- **Database queries**: Slow query detection and optimization
- **Performance profiling**: CPU/memory flame graphs

### Test Reliability
- **Deterministic setup**: Consistent test data and environments
- **Retry logic**: Transient failure handling
- **Timeout management**: Prevent hanging tests
- **Cleanup procedures**: Resource cleanup on failure
