# ARBITER v2 - Quick Start Guide

## 5-Minute Overview

ARBITER is a multi-agent orchestration system for AI-powered workflow automation.

**Status**: ✅ Functionally complete | ⚠️ MVP in progress | ❌ Production not ready

### What's Ready
- Core orchestration engine
- Agent registry and task routing
- Security framework with audit logging
- PostgreSQL persistence layer
- Infrastructure management (Docker, K8s, etc.)

### What's Next
1. Fix test fixtures (2-4 hours) → Get tests to 95% pass rate
2. Validate database under load (4-8 hours)
3. Set up CI/CD pipeline (8-16 hours)
4. Configure production monitoring (4-8 hours)

## Getting Started (5 Minutes)

```bash
# 1. Clone and navigate
cd agent-agency/iterations/v2

# 2. Install dependencies
npm install

# 3. Verify code quality
npm run typecheck  # Should show 0 errors
npm run lint       # Should show 0 violations

# 4. Run tests
npm test -- --maxWorkers=1

# 5. Start development server
npm run dev
```

## Project Structure

```
iterations/v2/
├── src/
│   ├── orchestrator/          # Core orchestration engine
│   ├── adapters/              # Infrastructure adapters
│   ├── security/              # Security framework
│   ├── database/              # Database layer
│   ├── observability/         # Logging & monitoring
│   └── ...
├── tests/                     # Test files (228 files)
├── migrations/                # Database migrations (17 files)
├── docs/                      # Documentation
├── docker-compose.yml         # Local dev environment
└── package.json               # Dependencies
```

## Common Tasks

### Run Tests
```bash
# All tests
npm test

# Specific test file
npm test -- --testPathPattern="security"

# With coverage
npm test -- --coverage

# Watch mode
npm test -- --watch
```

### Database Operations
```bash
# Connect to local database
psql postgresql://postgres:test123@localhost:5432/agent_agency_v2

# Run migrations
npm run migrate

# Seed test data
npm run seed:dev
```

### Code Quality
```bash
# Type check
npm run typecheck

# Lint check
npm run lint

# Format code
npm run format

# Both
npm run verify
```

### Build & Deploy
```bash
# Build for production
npm run build

# Build Docker image
docker build -t arbiter-v2:latest .

# Run Docker locally
docker-compose up
```

## Key Files to Know

| File | Purpose |
|------|---------|
| `src/orchestrator/ArbiterOrchestrator.ts` | Main orchestration logic |
| `src/orchestrator/AgentRegistryManager.ts` | Agent registry and management |
| `src/adapters/InfrastructureController.ts` | Infrastructure operations |
| `src/security/AgentRegistrySecurity.ts` | Security controls |
| `src/database/ConnectionPoolManager.ts` | Database connection management |
| `migrations/` | Database schema versions |
| `docs/1-core-orchestration/` | Architecture documentation |

## Troubleshooting

### TypeScript Errors
```bash
# Check for errors
npm run typecheck

# Most common: Missing type definitions
# Solution: Check node_modules/@types or add @types/package
```

### Test Failures
```bash
# Run single test to debug
npm test -- tests/unit/security/CommandValidator.test.ts

# Check test output for details
# Most common: Fixture configuration issues
```

### Database Connection Issues
```bash
# Check if PostgreSQL is running
psql --version

# Connect manually to test
psql postgresql://postgres:test123@localhost:5432/agent_agency_v2

# Check connection pool status
curl http://localhost:3000/health/db
```

## Next Steps

1. **Read the Full Documentation**
   - PRODUCTION_READINESS.md → Comprehensive status
   - DEPLOYMENT_READINESS.md → Deployment guide
   - SESSION_SUMMARY.txt → Detailed accomplishments

2. **Fix Immediate Blockers**
   - Fix test fixtures (2-4 hours)
   - Achieve 95%+ test pass rate
   - Run database validation

3. **Prepare for MVP**
   - Set up Docker/Kubernetes deployment
   - Configure basic monitoring
   - Document operational procedures

4. **Plan for Production**
   - Set up CI/CD pipeline
   - Configure production monitoring
   - Prepare security audit

## Resources

- **Docs**: `docs/`
- **API Docs**: `docs/api/`
- **Deployment**: `docs/deployment/`
- **Architecture**: `docs/1-core-orchestration/`
- **Issues**: GitHub Issues (or Jira)

## Team Contacts

- Architecture: [See CONTRIBUTORS.md]
- DevOps: [See CONTRIBUTORS.md]
- Security: [See CONTRIBUTORS.md]

---

**Questions?** Check the full documentation or create an issue!
