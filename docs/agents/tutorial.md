# Agent Agency V3 Tutorial - Getting Started

**Step-by-step guide to using the Constitutional AI System**

---

## Tutorial Overview

This tutorial walks you through using Agent Agency V3 to execute autonomous tasks with constitutional governance. You'll learn how to submit tasks, monitor progress, and intervene when needed.

**Time**: ~30 minutes  
**Level**: Beginner  
**Prerequisites**: Docker, Rust, Node.js

---

## Tutorial Scenario

**Task**: Implement a user authentication system with JWT tokens

**Requirements**:
- User registration and login endpoints
- JWT token generation and validation
- Password hashing and secure storage
- Proper error handling and validation
- Database persistence for users

**Risk Level**: Tier 2 (authentication system with data persistence)

---

## Step 1: Set Up the System

### Start Required Services

```bash
# 1. Start PostgreSQL database
docker run -d --name agent-agency-db \
  -e POSTGRES_PASSWORD=mysecretpassword \
  -e POSTGRES_DB=agent_agency_v3 \
  -p 5432:5432 \
  postgres:15

# 2. Navigate to V3 directory
cd iterations/v3

# 3. Run database migrations
cargo run --bin migrate

# 4. Install CAWS Git hooks (optional, for provenance tracking)
./scripts/install-git-hooks.sh
```

### Start the Core Services

```bash
# Terminal 1: Start API server
cargo run --bin api-server

# Terminal 2: Start worker service
cargo run --bin agent-agency-worker

# Terminal 3: Start web dashboard (optional)
cd apps/web-dashboard
npm install
npm run dev
```

**Expected Output:**
```
Starting Agent Agency API Server
Server: 127.0.0.1:8080
API server ready at http://127.0.0.1:8080

Starting Agent Agency Worker
Server: 127.0.0.1:8081
Worker ID: default-worker
Worker ready at http://127.0.0.1:8081
```

## Step 2: Submit Your First Task

### Using the CLI

```bash
# Submit task in auto mode (recommended for most cases)
cargo run --bin agent-agency-cli execute \
  "Implement user authentication system with JWT tokens, user registration, login, and secure password storage" \
  --mode auto \
  --risk-tier 2 \
  --watch

# Alternative: Dry-run mode for safe testing
cargo run --bin agent-agency-cli execute \
  "Design the user authentication API schema" \
  --mode dry-run
```

### Using the API Directly

```bash
# Submit via REST API
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Implement user authentication system with JWT tokens, user registration, login, and secure password storage",
    "execution_mode": "auto",
    "max_iterations": 10,
    "risk_tier": 2
  }'
```

**Expected Response:**
```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "accepted",
  "message": "Task submitted successfully"
}
```

## Step 3: Monitor Task Progress

### Real-time Monitoring with CLI

When you use `--watch`, the CLI will show live progress:

```
Agent Agency V3 - Autonomous Execution
═══════════════════════════════════════════

Task: Implement user authentication system...
Task ID: 550e8400-e29b-41d4-a716-446655440000
Mode: AUTO (Quality Gates Enabled)
🎚️  Risk Tier: 2

Phase: Planning and validation
   Analyzing requirements...
   Council review passed
   CAWS compliance validated

Phase: Worker execution
   Executing implementation...
   Progress: 65%
   Code generation completed
   Tests written and passing

Phase: Quality assurance
   Running test suite...
   Coverage: 85% (Target: 80%)
   All quality gates passed

Task completed successfully!
⏱️  Total time: 2m 34s
```

### API Monitoring

```bash
# Get task status
TASK_ID="550e8400-e29b-41d4-a716-446655440000"
curl http://localhost:8080/api/v1/tasks/$TASK_ID

# Get detailed progress
curl http://localhost:8080/api/v1/tasks/$TASK_ID/result
```

### Web Dashboard Monitoring

Open http://localhost:3000 to see:
- Live task progress visualization
- System metrics and SLO status
- Database exploration tools
- Real-time alerts and notifications

## Step 4: Real-time Intervention

### Pause and Resume Tasks

```bash
# Pause execution
cargo run --bin agent-agency-cli intervene pause $TASK_ID

# Resume execution
cargo run --bin agent-agency-cli intervene resume $TASK_ID
```

### Override Council Decisions

```bash
# Override verdict in strict mode
cargo run --bin agent-agency-cli intervene override $TASK_ID --verdict accept --reason "Approved by human review"
```

### Cancel Running Tasks

```bash
# Cancel task execution
cargo run --bin agent-agency-cli intervene cancel $TASK_ID
```

## Step 5: Review Results and Provenance

### Check Task Artifacts

```bash
# Get complete task results
curl http://localhost:8080/api/v1/tasks/$TASK_ID/result

# View generated code and tests
# (Results will show file paths and changes made)
```

### Verify Provenance

```bash
# Check provenance records
curl http://localhost:8080/api/v1/provenance

# Verify specific commit
curl http://localhost:8080/api/v1/provenance/verify/$(git rev-parse HEAD)
```

## Step 6: Handle Quality Gate Exceptions

### Create Waiver for Special Cases

```bash
# Create waiver for emergency deployment
curl -X POST http://localhost:8080/api/v1/waivers \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Emergency security patch",
    "reason": "security_patch",
    "description": "Deploying critical security fix without full test coverage",
    "gates": ["test-coverage"],
    "approved_by": "security-team",
    "impact_level": "high",
    "mitigation_plan": "Security review completed, monitoring in place"
  }'
```

### Approve Waiver

```bash
# Approve the waiver
WAIVER_ID="your-waiver-id"
curl -X POST http://localhost:8080/api/v1/waivers/$WAIVER_ID/approve
```

## Troubleshooting

### Common Issues

**Worker not responding:**
```bash
# Check worker logs
ps aux | grep agent-agency-worker
# Restart worker if needed
cargo run --bin agent-agency-worker
```

**Database connection failed:**
```bash
# Check database status
docker ps | grep postgres
# Reset database if needed
docker restart agent-agency-db
```

**Task stuck in pending:**
```bash
# Check API server logs
curl http://localhost:8080/health
# Restart API server if needed
cargo run --bin api-server
```

### Getting Help

```bash
# CLI help
cargo run --bin agent-agency-cli --help

# API health check
curl http://localhost:8080/health

# System metrics
curl http://localhost:8080/metrics
```

## Next Steps

- Try different execution modes (strict, auto, dry-run)
- Experiment with task intervention capabilities
- Explore the web dashboard features
- Learn about CAWS compliance and quality gates
- Set up SLO monitoring and alerting

---

**Congratulations!** You've successfully used Agent Agency V3 to execute an autonomous task with constitutional governance. The system provides complete oversight, real-time control, and comprehensive audit trails for all AI operations.

