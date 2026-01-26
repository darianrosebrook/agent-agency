# Agent Agency V3 Database Initialization Guide

> **Note:** For a streamlined setup experience, see the **[Getting Started Guide](./docs/GETTING_STARTED.md)** which covers both Docker and local PostgreSQL options with troubleshooting.
>
> This document provides detailed Docker-specific configuration and migration reference.

## Overview

This document describes how to initialize the Agent Agency V3 database from a fresh start.

## Prerequisites

- Docker with PostgreSQL container running
- Database: `agent_agency_test` on port `5433`
- User: `test_user` with password `test_password`

## Quick Start

### 1. Start the Database Container

```bash
cd iterations/v3
docker-compose up -d
```

### 2. Initialize the Database

Run the initialization script:

```bash
./scripts/init_fresh_database.sh
```

Or manually apply migrations:

```bash
export DATABASE_URL="postgresql://test_user:test_password@localhost:5433/agent_agency_test"
export PGPASSWORD=test_password

# Apply all migrations
for f in data-infrastructure/migrations/*.sql; do
    psql -h localhost -p 5433 -U test_user -d agent_agency_test -f "$f"
done
```

### 3. Start the API Server

```bash
export DATABASE_URL="postgresql://test_user:test_password@localhost:5433/agent_agency_test"
export RUST_LOG=info
cargo run --bin agent-agency-api-server --features orchestration,testing -- --port 8889
```

## Migration Files

The migrations are located in `data-infrastructure/migrations/` and are applied in numerical order:

| Migration | Description |
|-----------|-------------|
| 000 | Workspace registry table |
| 001 | Enable pgvector extension |
| 002 | Vector storage tables |
| 003 | Agent experiences |
| 004 | Memory system |
| 005 | Planning system |
| 006 | Telemetry storage |
| 007 | Worker assignment tracking |
| 008 | Agent context management |
| 009 | WAL storage |
| 010 | Chat persistence |
| 011 | Multi-tenant isolation |
| 012 | Chat query optimization |
| 013 | Chat search and organization |
| 014 | Agent management tables |
| 015 | Observation tables |
| 016 | Authentication tables |
| 017 | Composite indexes |
| 018 | Settings tables |
| 018b | Waivers table |
| 019 | Rules governance tables |
| 020 | Task state persistence |
| 021 | Rate limit tables |
| 022 | Task plan index |
| 023 | Execution results table |
| 024 | Task project_id foreign key |
| 025 | Task comments |
| 026 | Council sessions |
| 027 | Project overview versions |
| 028 | Planning audit events description |
| 029 | Task progress table |
| 030 | Telemetry tracking tables |
| 031 | Milestone completions |

## Critical Tables

The following tables are required for the system to function:

| Table | Purpose |
|-------|---------|
| `tasks` | Task management |
| `workers` | Worker/agent registry |
| `waivers` | Quality gate waivers |
| `workspace_registry` | Workspace management |
| `execution_plans` | Project/plan storage |
| `planning_audit_events` | Planning audit trail |
| `council_sessions` | Council decision sessions |
| `users` | User authentication |
| `sessions` | User sessions |
| `chat_sessions` | Chat session storage |
| `chat_messages` | Chat message storage |
| `curriculum_profiles` | Agent skill profiles |
| `learning_history` | Learning records |
| `milestone_completions` | Milestone tracking |

## Verification

After initialization, verify the database:

```bash
# Check table count (should be ~72)
psql -h localhost -p 5433 -U test_user -d agent_agency_test -c \
    "SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'public';"

# Check migration count (should be ~31)
psql -h localhost -p 5433 -U test_user -d agent_agency_test -c \
    "SELECT COUNT(*) FROM migration_log;"

# Check critical tables
psql -h localhost -p 5433 -U test_user -d agent_agency_test -c \
    "SELECT tablename FROM pg_tables WHERE schemaname = 'public' 
     AND tablename IN ('tasks', 'workers', 'waivers', 'workspace_registry');"
```

## API Endpoints

Once the server is running, test with:

```bash
# Health check
curl http://localhost:8889/api/v1/system/health

# List tasks
curl http://localhost:8889/api/v1/tasks

# List projects
curl http://localhost:8889/api/v1/projects

# List agents
curl http://localhost:8889/api/v1/agents

# Submit a task
curl -X POST http://localhost:8889/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{"description": "Test task", "execution_mode": "dry-run", "risk_tier": "3"}'
```

## Troubleshooting

### UnifiedOrchestrator not available

If you see this error, check that the `waivers` and `workspace_registry` tables exist:

```bash
psql -h localhost -p 5433 -U test_user -d agent_agency_test -c "\dt waivers"
psql -h localhost -p 5433 -U test_user -d agent_agency_test -c "\dt workspace_registry"
```

If missing, apply migrations 018b and 000:

```bash
psql -h localhost -p 5433 -U test_user -d agent_agency_test \
    -f data-infrastructure/migrations/000_create_workspace_registry.sql
psql -h localhost -p 5433 -U test_user -d agent_agency_test \
    -f data-infrastructure/migrations/018b_create_waivers_table.sql
```

### Migration transaction aborted

If migrations fail with "current transaction is aborted", the migration runner continues with remaining migrations. This is usually due to objects already existing. Check the server logs for specific errors.

### Task status constraint violation

Valid task statuses are: `pending`, `in_progress`, `paused`, `completed`, `cancelled`, `failed`.

If you see constraint violations, ensure the code uses `in_progress` instead of `running`.







