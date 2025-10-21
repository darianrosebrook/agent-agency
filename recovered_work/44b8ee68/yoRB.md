# Agent Agency - Constitutional AI System

## Overview

Agent Agency V3 implements a **constitutional AI system** where autonomous agents operate under real-time governance oversight. The system provides three execution modes (Strict, Auto, Dry-Run) for different risk levels, with a four-judge council ensuring ethical compliance, technical quality, and system coherence.

## Core Components

### Constitutional Council
Four specialized AI judges provide governance oversight:
- **Constitutional Judge**: Ethical compliance and CAWS validation
- **Technical Auditor**: Code quality and security standards
- **Quality Evaluator**: Requirements satisfaction verification
- **Integration Validator**: System coherence and compatibility

### Execution Modes
- **Strict Mode**: Manual approval required for each execution phase
- **Auto Mode**: Automatic execution with quality gate validation
- **Dry-Run Mode**: Safe testing without filesystem modifications

### Real-time Control
- **Intervention API**: Pause, resume, cancel, and override running tasks
- **Progress Tracking**: Real-time task status and metrics
- **CLI Tools**: Command-line interface for task management
- **Web Dashboard**: Live monitoring and database exploration

## Key Features

### Safety & Governance
- Constitutional oversight for all agent operations
- Risk-appropriate execution modes
- Real-time intervention capabilities
- Comprehensive provenance tracking

### Quality Assurance
- CAWS compliance validation
- Automated testing and quality gates
- Waiver system for exceptional cases
- Git-backed audit trails

### Monitoring & Control
- SLO monitoring and alerting
- Real-time metrics and dashboards
- Database exploration tools
- Task lifecycle management

## Getting Started

### Prerequisites
- Rust 1.75+
- PostgreSQL with pgvector
- Node.js 18+ (for CAWS tools)

### Quick Start
```bash
# 1. Start database
docker run -d --name postgres-v3 -e POSTGRES_PASSWORD=password -p 5432:5432 postgres:15

# 2. Run migrations
cd iterations/v3
cargo run --bin migrate

# 3. Start services
cargo run --bin api-server &
cargo run --bin agent-agency-worker &
cd apps/web-dashboard && npm run dev &

# 4. Execute a task
cargo run --bin agent-agency-cli execute "Implement user authentication" --mode auto
```

## Documentation

### Implementation Guides
- **[CAWS Full Guide](./agents/full-guide.md)** - Complete CAWS workflow implementation
- **[CLI Tutorial](./agents/tutorial.md)** - Getting started with command-line tools
- **[Examples](./agents/examples.md)** - Working examples and use cases

### System Documentation
- **[System Overview](../iterations/v3/docs/SYSTEM_OVERVIEW.md)** - Complete system capabilities
- **[Architecture Guide](../iterations/v3/docs/architecture.md)** - Technical implementation details
- **[API Contracts](../iterations/v3/docs/interaction-contracts.md)** - REST API specifications

### Development Resources
- **[Quality Assurance](./quality-assurance/README.md)** - Testing and CAWS compliance
- **[Database Schema](./database/README.md)** - Data persistence and queries
- **[Deployment Guide](../deploy/README.md)** - Production deployment procedures

## CLI Commands

### Task Execution
```bash
# Execute with different modes
cargo run --bin agent-agency-cli execute "task description" --mode strict
cargo run --bin agent-agency-cli execute "task description" --mode auto
cargo run --bin agent-agency-cli execute "task description" --mode dry-run

# Monitor and intervene
cargo run --bin agent-agency-cli intervene pause task-id
cargo run --bin agent-agency-cli intervene resume task-id
cargo run --bin agent-agency-cli intervene cancel task-id
```

### System Management
```bash
# Waiver management
cargo run --bin agent-agency-cli waiver create --title "Emergency fix"
cargo run --bin agent-agency-cli waiver approve waiver-id

# Provenance tracking
cargo run --bin agent-agency-cli provenance install-hooks
cargo run --bin agent-agency-cli provenance generate
```

## API Endpoints

### Task Management
- `POST /api/v1/tasks` - Submit task for execution
- `GET /api/v1/tasks/:id` - Get task status
- `POST /api/v1/tasks/:id/pause` - Pause task execution
- `POST /api/v1/tasks/:id/resume` - Resume paused task

### Governance
- `GET /api/v1/waivers` - List active waivers
- `POST /api/v1/waivers` - Create waiver
- `GET /api/v1/provenance` - List provenance records

### Monitoring
- `GET /api/v1/slos` - Service level objectives
- `GET /api/v1/slo-alerts` - Active alerts
- `GET /metrics` - System metrics

## Architecture

### Data Flow
1. **Task Submission** → API/CLI receives task with execution mode
2. **Council Validation** → Constitutional judges evaluate compliance
3. **Worker Execution** → HTTP-based task distribution with circuit breakers
4. **Progress Tracking** → Real-time status updates and metrics
5. **Intervention** → Human operators can control execution
6. **Provenance** → Complete audit trail with Git integration

### Safety Model
- **Dry-Run**: Complete simulation without filesystem changes
- **Auto**: Quality gate validation before execution
- **Strict**: Manual approval required for each phase

## Development

### Building
```bash
# Build all components
cd iterations/v3
cargo build --workspace

# Run tests
cargo test --workspace

# CAWS validation
cd ../../apps/tools/caws
npm run validate
```

### Key Technologies
- **Rust**: High-performance system implementation
- **PostgreSQL**: ACID-compliant data persistence
- **Axum**: Async web framework for APIs
- **CAWS Tools**: Quality assurance and compliance
- **React**: Web dashboard interface

## Contributing

### Development Workflow
1. Create working specification (`.caws/working-spec.yaml`)
2. Implement with appropriate risk tier
3. Test against CAWS quality gates
4. Submit with provenance tracking

### Quality Gates
- **Tier 1**: 90%+ coverage, full manual review
- **Tier 2**: 80%+ coverage, automated validation
- **Tier 3**: 70%+ coverage, basic checks

## Support

- **Documentation**: See links above for detailed guides
- **Examples**: Check `docs/agents/examples.md` for working code
- **Issues**: File issues with complete reproduction steps
- **Discussions**: Use discussions for architecture questions

---

*Agent Agency provides constitutional governance for autonomous AI operations, ensuring safety, quality, and accountability in AI agent execution.*
