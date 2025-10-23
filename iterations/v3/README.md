# Agent Agency V3: Advanced Agent Infrastructure

## Overview

**Agent Agency V3** provides functional infrastructure for autonomous agent systems with a working constitutional AI framework. Built in Rust for memory safety and performance, it delivers core orchestration and governance capabilities, with many advanced features remaining as TODO implementations.

This system has an operational core execution loop with constitutional oversight, featuring modular architecture and basic monitoring, though comprehensive enterprise features are still under development.

## Core Operational Components

V3 provides functional infrastructure with operational core components:

### Task Execution Pipeline Operational
- **Orchestrator**: HTTP-based task routing with circuit breaker patterns
- **Worker Services**: Task execution with configurable timeouts and retries
- **Progress Tracking**: Real-time task status and intervention capabilities
- **Execution Modes**: Strict, Auto, and Dry-Run modes for different risk levels

### Governance Framework Core Implemented
- **Constitutional Council**: Four-judge oversight framework (logic partially implemented)
- **CAWS Compliance**: Runtime validation with waiver system for exceptions
- **Provenance Tracking**: Basic Git integration with cryptographic signing
- **Quality Gates**: Automated testing and validation pipelines

### Monitoring & Control Partially Implemented
- **API Server**: RESTful endpoints with authentication and basic monitoring
- **CLI Tools**: Command-line interface with intervention commands
- **Web Dashboard**: Basic metrics display and database exploration
- **Real-time Metrics**: Task progress and system health monitoring

### Advanced Components Planned/Incomplete

#### Runtime Optimization (Framework exists, many TODOs)
- **Bayesian Optimization**: Framework for ML-based hyper-parameter tuning
- **Thermal Management**: Apple Silicon temperature monitoring (basic implementation)
- **Performance Monitoring**: Real-time metrics collection (basic)
- **Resource Management**: Workload distribution (framework only)

#### Tool Ecosystem (Basic framework, many TODOs)
- **MCP Integration**: Model Context Protocol framework (basic)
- **Conflict Resolution**: AI-mediated dispute resolution (framework)
- **Security Validation**: Tool execution audit trails (basic)
- **Dynamic Discovery**: Capability detection (framework)

#### Federated Learning (Infrastructure scaffolding, many TODOs)
- **Privacy Preservation**: Differential privacy framework (basic)
- **Secure Aggregation**: Encryption framework (basic)
- **Cross-Tenant Learning**: Communication protocols (framework)
- **Participant Management**: Agent lifecycle handling (basic)
- **Compliance Enforcement**: Constraint verification (framework)

#### Model Hot-Swapping (Framework exists, many TODOs)
- **Version Management**: Model lifecycle tracking (basic)
- **Traffic Control**: Load balancing framework (basic)
- **Performance Routing**: Metrics-based selection (framework)
- **Zero-Downtime Updates**: Deployment framework (basic)

## System Architecture

### Core Modules

#### Task Execution Pipeline Operational
- **Orchestrator (`orchestration`)**: HTTP-based task routing with circuit breaker patterns
- **Worker Services (`workers`)**: Task execution with configurable timeouts and retries
- **Progress Tracking (`orchestration/tracking`)**: Real-time status updates and intervention
- **CLI Interface (`cli`)**: Command-line task submission and monitoring

#### Governance Framework Core Implemented
- **Council System (`council`)**: Four-judge constitutional oversight framework
- **CAWS Integration (`caws`)**: Runtime compliance validation with waivers
- **Provenance System (`provenance`)**: Git-backed audit trails with JWS signing
- **Planning Agent (`planning-agent`)**: Working specification generation

#### Monitoring & Infrastructure Partially Implemented
- **API Server (`interfaces/api`)**: RESTful endpoints with authentication
- **Database Layer (`database`)**: PostgreSQL persistence with core task storage
- **Web Dashboard (`apps/web-dashboard`)**: Basic metrics and database exploration
- **Observability (`observability`)**: Metrics collection and SLO framework

#### Advanced Components Framework/Incomplete

##### Runtime Optimization (`runtime-optimization`) - Framework exists, many TODOs
- **Kokoro Tuner**: Bayesian optimization framework (basic)
- **Thermal Scheduler**: Apple Silicon temperature monitoring (basic implementation)
- **Performance Monitor**: Metrics collection (basic)
- **Resource Manager**: Workload distribution (framework only)

##### Tool Ecosystem (`tool-ecosystem`) - Basic framework, many TODOs
- **Tool Coordinator**: Dynamic discovery framework (basic)
- **Conflict Resolution**: AI-mediated resolution (framework)
- **Security Validator**: Audit trails (basic)
- **MCP Integration**: Model Context Protocol (basic framework)

##### Federated Learning (`federated-learning`) - Infrastructure scaffolding, many TODOs
- **Federation Coordinator**: Cross-tenant orchestration (framework)
- **Secure Aggregator**: Encryption framework (basic)
- **Differential Privacy**: Privacy guarantees (framework)
- **Participant Manager**: Agent lifecycle (basic)

##### Model Hot-Swapping (`model-hotswap`) - Framework exists, many TODOs
- **Load Balancer**: Traffic distribution (basic)
- **Model Registry**: Version management (basic)
- **Canary Deployer**: Gradual deployment (framework)
- **Performance Router**: Metrics-based routing (framework)

#### Observability (`agent-agency-observability`)
- **Metrics Collection**: Comprehensive system and business metrics
- **Multimodal Metrics**: Specialized metrics for multimodal processing
- **Alerting**: Real-time alerting with configurable thresholds
- **Performance Tracking**: SLA compliance and performance optimization

#### Self-Prompting Agent (`self-prompting-agent`)
- **Loop Controller**: Orchestrates generate-evaluate-refine cycles with safety guards
- **Prompting Strategies**: Adaptive prompting with tool-call envelope validation
- **Model Health Monitoring**: Automatic fallback and reliability tracking
- **Evaluation Orchestrator**: Comprehensive testing and quality assessment

#### File Operations (`file_ops`)
- **Workspace Management**: Isolated editing with Git worktrees or temp directories
- **ChangeSet Processing**: Atomic file modifications with rollback capabilities
- **AllowList Enforcement**: Path and content restrictions for safe editing
- **Budget Controls**: Resource limits and waiver system for constraint management

## Getting Started

### Prerequisites

- **Rust 1.75+** (for development)
- **Docker 20.10+ and Docker Compose 2.0+** (for production deployment)
- **PostgreSQL 14+ with pgvector extension** (for vector storage)
- **Apple Silicon** (recommended for hardware acceleration)
- **k6** (for load testing)

### Database Setup

Agent Agency V3 requires PostgreSQL with several extensions and custom schemas. The system includes comprehensive database integration with 5 core components.

#### Quick Database Setup

```bash
# Using Docker (recommended for development)
docker run -d \
  --name agent-agency-db \
  -e POSTGRES_DB=agent_agency \
  -e POSTGRES_USER=agent_agency \
  -e POSTGRES_PASSWORD=secure_password_123 \
  -p 5432:5432 \
  -v agent_agency_data:/var/lib/postgresql/data \
  pgvector/pgvector:pg15

# Enable required extensions
docker exec -it agent-agency-db psql -U agent_agency -d agent_agency -c "CREATE EXTENSION IF NOT EXISTS pgvector;"
docker exec -it agent-agency-db psql -U agent_agency -d agent_agency -c "CREATE EXTENSION IF NOT EXISTS uuid_ossp;"
```

#### Database Components

The system integrates 5 components with PostgreSQL persistence:

1. **CAWS Checker**: Stores validation results in `caws_validations` table
2. **Source Integrity**: Manages integrity records in `source_integrity_records` table
3. **Council Learning**: Queries historical data from `task_resource_history` table
4. **Claim Extraction**: Accesses knowledge bases via `external_knowledge_entities` table
5. **Analytics Dashboard**: Caches insights in `analytics_cache` table with LRU eviction

#### Schema Migration

```bash
# Run migrations (from iterations/v3 directory)
cd iterations/v3
cargo run --bin migration_runner

# Or use the database package directly
cargo run --package agent-agency-database --bin migrate
```

#### Database Configuration

Set these environment variables in your `.env` file:

```bash
DATABASE_URL=postgresql://agent_agency:secure_password_123@localhost:5432/agent_agency
DATABASE_MAX_CONNECTIONS=20
DATABASE_CONNECTION_TIMEOUT_SECS=30
DATABASE_HEALTH_CHECK_INTERVAL_SECS=60
```

#### Database Troubleshooting

**Common Issues:**

- **pgvector extension not found**: Ensure you're using `pgvector/pgvector:pg15` image
- **Migration failures**: Check database permissions and connection string
- **Performance issues**: Monitor with `pg_stat_activity` and optimize queries
- **Connection pooling**: Adjust `DATABASE_MAX_CONNECTIONS` based on load

**Performance Monitoring:**

```sql
-- Monitor query performance
SELECT query, calls, total_time, mean_time, rows
FROM pg_stat_statements
ORDER BY total_time DESC
LIMIT 10;

-- Check connection usage
SELECT count(*) as active_connections
FROM pg_stat_activity
WHERE state = 'active';
```

**Backup & Recovery:**

```bash
# Backup
pg_dump -U agent_agency -h localhost agent_agency > backup.sql

# Restore
psql -U agent_agency -h localhost agent_agency < backup.sql
```

### Quick Start (Production Deployment)

```bash
# Clone and navigate to V3
cd iterations/v3

# Set up environment
cp env.production.example .env.production
# Edit .env.production with your configuration

# Deploy the complete system
chmod +x scripts/deploy-production.sh
./scripts/deploy-production.sh deploy

# Access services
# - API: http://localhost:8080
# - Metrics: http://localhost:8081
# - Grafana: http://localhost:3000
# - Prometheus: http://localhost:9090
# - Kibana: http://localhost:5601
```

### Task Execution System

Agent Agency V3 includes a functional task execution system with constitutional oversight. The system provides multiple execution modes for different safety levels:

#### Execution Modes
- **Dry-Run Mode**: Safe testing without filesystem changes (implemented)
- **Auto Mode**: Automatic execution with basic quality validation (implemented)
- **Strict Mode**: Manual approval required for execution phases (implemented)

#### Usage Examples

```bash
# Test the execution pipeline safely (recommended for exploration)
cargo run --bin agent-agency-cli execute "Test execution pipeline" --mode dry-run

# Execute with basic quality gates
cargo run --bin agent-agency-cli execute "Implement basic feature" --mode auto

# Require manual approval for changes
cargo run --bin agent-agency-cli execute "Make system changes" --mode strict --watch

# Monitor and intervene in running tasks
cargo run --bin agent-agency-cli intervene pause <task-id>
cargo run --bin agent-agency-cli intervene resume <task-id>
cargo run --bin agent-agency-cli intervene cancel <task-id>
```

#### Current Capabilities
- **Task Submission**: CLI and REST API interfaces (implemented)
- **Worker Orchestration**: HTTP-based task execution (implemented)
- **Progress Monitoring**: Real-time task status updates (implemented)
- **Intervention API**: Pause, resume, cancel operations (implemented)
- **Basic Governance**: Constitutional council framework (partial)
- **Provenance Tracking**: Git integration for audit trails (basic)

### Development Setup

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run integration tests
cargo test --package integration-tests

# Run the system locally
cargo run
```

### Load Testing

```bash
# Install k6 (if not already installed)
# macOS: brew install k6
# Ubuntu: sudo apt-get install k6

# Run comprehensive load tests
cd load-testing
k6 run k6-multimodal-rag-test.js
```

## Production Features

### Deployment Infrastructure
- **Docker Containerization**: Complete containerized deployment with health checks
- **Database Migrations**: Automated schema management with pgvector setup
- **Environment Configuration**: Comprehensive production configuration management
- **Backup & Recovery**: Automated backup procedures with restore capabilities

### Monitoring & Observability
- **Prometheus Metrics**: Comprehensive system and business metrics collection
- **Grafana Dashboards**: Real-time monitoring and visualization
- **Alerting**: Configurable alerts for SLA breaches and system issues
- **Log Aggregation**: Centralized logging with Elasticsearch and Kibana

### Security & Performance
- **JWT Authentication**: Secure API access with configurable expiry
- **Rate Limiting**: Per-IP rate limiting with burst handling
- **Input Validation**: Comprehensive input sanitization and validation
- **Performance Optimization**: Connection pooling, caching, and query optimization

### Scalability & Reliability
- **Horizontal Scaling**: Stateless design for easy horizontal scaling
- **Circuit Breakers**: Fault tolerance for external service dependencies
- **Retry Logic**: Exponential backoff with jitter for resilient operations
- **Health Checks**: Comprehensive health monitoring and automatic recovery

## Documentation

### System Documentation
- **[Production Deployment Guide](./docs/PRODUCTION_DEPLOYMENT.md)**: Complete production deployment instructions
- **[Multimodal RAG Integration Spec](./docs/MULTIMODAL_RAG_INTEGRATION_SPEC.md)**: Technical integration specification
- **[Multimodal RAG README](./docs/MULTIMODAL_RAG_README.md)**: Detailed system architecture and components

### Research Documentation
- **[Arbiter Theory](../../docs/arbiter/theory.md)**: LLM orchestration requirements
- **[CAWS Framework](https://github.com/paths-design/caws)**: Constitutional workflow standards

## Integration Status

### Core System Operational
- **Task Execution Pipeline**: End-to-end task submission, orchestration, and worker execution
- **Constitutional Governance**: Four-judge council framework with basic oversight logic
- **Execution Modes**: Dry-run, auto, and strict modes with appropriate safety controls
- **Intervention API**: Pause, resume, cancel operations for running tasks
- **Progress Tracking**: Real-time task status updates and monitoring
- **CLI Interface**: Command-line task submission and intervention commands
- **API Server**: RESTful endpoints with basic authentication and task management
- **Database Layer**: Core task persistence and basic provenance storage
- **Web Dashboard**: Basic metrics display and database exploration interface
- **Provenance Tracking**: Git integration for basic audit trails

### Partially Implemented (Framework exists, many TODOs)
- **CAWS Compliance**: Runtime validation with waiver system for exceptions
- **Multimodal Processing**: Framework exists, enrichers are mostly TODO placeholders
- **Observability**: Basic metrics collection, comprehensive SLO monitoring TODO
- **Apple Silicon Features**: Some thermal monitoring, most Core ML features TODO
- **Advanced Analytics**: Basic metrics, comprehensive analytics framework TODO

### Planned/Incomplete (Framework scaffolding, major TODOs)
- **Federated Learning**: Infrastructure exists, implementation largely TODO
- **Model Hot-Swapping**: Framework exists, advanced features TODO
- **Tool Ecosystem**: Basic MCP integration, advanced features TODO
- **Runtime Optimization**: Some thermal monitoring, most optimization TODO
- **Distributed Processing**: Single-node only, distributed features TODO

## Author

@darianrosebrook
