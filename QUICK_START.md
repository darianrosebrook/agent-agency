# Agent Agency V3 - Quick Start Guide

## 5-Minute Overview

Agent Agency is a constitutional AI system with thread-safe CoreML integration for Apple Silicon optimization.

**Status**: ✅ Core operational with Send/Sync safety | ⚠️ Advanced features TODO | 🚧 Production hardening needed

### What's Ready
- Constitutional council governance (4-judge framework)
- Thread-safe CoreML integration (Send/Sync violations resolved)
- Task execution pipeline with worker orchestration
- Ollama/Gemma integration with circuit breakers
- CLI and REST API interfaces
- Real-time task monitoring and intervention

### What's Next
1. Advanced monitoring and SLO tracking
2. Multi-tenant memory systems
3. Distributed processing capabilities
4. Production deployment hardening

## Getting Started (5 Minutes)

```bash
# 1. Clone and navigate
cd agent-agency/iterations/v3

# 2. Verify compilation (Send/Sync safety)
cargo check -p agent-agency-council -p agent-agency-apple-silicon
# Should show 0 errors ✅

# 3. Start database (optional)
docker run -d --name postgres-v3 -e POSTGRES_PASSWORD=password -p 5432:5432 postgres:15

# 4. Run database migrations
cargo run --bin migrate

# 5. Start API server
cargo run --bin api-server &

# 6. Test execution pipeline
cargo run --bin agent-agency-cli execute "Hello world" --mode dry-run
```

## Project Structure

```
iterations/v3/
├── council/                   # Constitutional AI governance
│   ├── src/
│   │   ├── judge.rs           # Constitutional judges (4 types)
│   │   ├── model_client.rs    # Thread-safe CoreML client
│   │   └── council.rs         # Council orchestration
├── apple-silicon/             # CoreML/ANE acceleration
│   ├── src/
│   │   ├── ane/               # Apple Neural Engine integration
│   │   └── async_inference.rs # Thread-safe inference
├── orchestrator/              # Task execution pipeline
├── security/                  # Authentication & authorization
├── database/                  # PostgreSQL persistence
└── docs/                      # Architecture documentation
```

## Common Tasks

### Run Tests
```bash
# All tests
cargo test

# Specific crate tests
cargo test -p agent-agency-council

# With coverage (requires cargo-tarpaulin)
cargo tarpaulin --workspace

# Run specific test
cargo test test_name
```

### Database Operations
```bash
# Connect to local database
psql postgresql://postgres:password@localhost:5432/agent_agency_v3

# Run migrations
cargo run --bin migrate

# Check database status
cargo run --bin agent-agency-cli db status
```

### Code Quality
```bash
# Type check and compile
cargo check

# Run Clippy lints
cargo clippy

# Format code
cargo fmt

# All quality checks
cargo fmt --check && cargo clippy && cargo test
```

### Build & Deploy
```bash
# Build for production
cargo build --release

# Build Docker image
docker build -t agent-agency-v3:latest -f deploy/docker/Dockerfile.orchestrator .

# Run with Docker Compose
docker-compose -f deploy/docker-compose/dev.yml up

# Deploy to Kubernetes
kubectl apply -f deploy/kubernetes/base/
```

## Key Files to Know

| File | Purpose |
|------|---------|
| `council/src/judge.rs` | Constitutional judges implementation |
| `council/src/model_client.rs` | Thread-safe CoreML client |
| `apple-silicon/src/ane/` | Apple Neural Engine integration |
| `orchestrator/src/workflow.rs` | Task execution pipeline |
| `database/src/client.rs` | PostgreSQL persistence layer |
| `security/src/auth.rs` | Authentication & authorization |
| `docs/` | Architecture documentation |

## Troubleshooting

### Compilation Errors
```bash
# Check for compilation errors
cargo check -p agent-agency-council -p agent-agency-apple-silicon

# Most common: Send/Sync violations in CoreML integration
# Solution: Ensure ModelRef is used instead of raw pointers
```

### Test Failures
```bash
# Run specific test to debug
cargo test test_name -- --nocapture

# Check test output for details
# Most common: Async timing issues or fixture configuration
```

### Database Connection Issues
```bash
# Check if PostgreSQL is running
psql --version

# Connect manually to test
psql postgresql://postgres:password@localhost:5432/agent_agency_v3

# Check connection status
cargo run --bin agent-agency-cli db status
```

## Next Steps

1. **Verify Core Functionality**
   - Test constitutional council with dry-run tasks
   - Validate CoreML thread safety with real inference
   - Check task execution pipeline end-to-end

2. **Read the Documentation**
   - README.md → Complete system overview
   - docs/README.md → Documentation structure
   - docs/agents/full-guide.md → CAWS framework guide

3. **Advanced Features**
   - Implement comprehensive monitoring and SLOs
   - Add multi-tenant memory systems
   - Enable distributed processing capabilities

4. **Production Readiness**
   - Set up CI/CD pipeline
   - Configure production monitoring
   - Complete security hardening

## Resources

- **Core Docs**: `README.md` and `docs/README.md`
- **Architecture**: `docs/` directory
- **CAWS Framework**: `docs/agents/full-guide.md`
- **API Reference**: REST endpoints and contracts
- **Issues**: GitHub Issues

## Team Contacts

- Architecture: @darianrosebrook
- Development: Core team
- Security: Core team

---

**Questions?** Check the README.md or create a GitHub issue!
