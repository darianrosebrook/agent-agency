# ARBITER-008 Integration - Immediate Next Steps

**Date**: October 12, 2025  
**Status**: Implementation Complete ✅ | Integration Pending ⏳  
**Priority**: Medium (After ARBITER-006 and ARBITER-007)

---

## Current Situation

### ✅ What's Complete

1. **ARBITER-008 (Web Navigator) Implementation**

   - ✅ All source code (1,910 LOC)
   - ✅ All tests (48/48 passing)
   - ✅ Database migration script ready
   - ✅ Comprehensive documentation
   - ✅ Committed to main (commit: `88048b6`)

2. **Knowledge Seeker Integration**

   - ✅ SearchEngine fully delegates to KnowledgeSeeker
   - ✅ No code changes needed

3. **Type System**
   - ✅ All types defined and compatible

### ⏳ What's Pending

1. **Database Setup**

   - ⏳ Migration 004 not yet applied
   - ⏳ Web Navigator tables don't exist yet
   - ⏳ Test database not configured

2. **Orchestrator Integration**

   - ⏳ WebNavigator not added to ArbiterOrchestrator
   - ⏳ Web task types not registered
   - ⏳ Configuration not added

3. **Integration Testing**
   - ⏳ Cannot run integration tests without database
   - ⏳ End-to-end flows not tested

---

## Database Environment Analysis

### Current Local Setup

```
PostgreSQL: 14.15 (Homebrew) - RUNNING on port 5432
User: darianrosebrook (system user)
Existing Databases:
  - postgres (empty)
  - obsidian_rag
  - obsidian_rag_test
  - template0
  - template1

Missing:
  - agent_agency_test (required for tests)
  - No ARBITER tables exist yet
```

### Required Database

The tests expect:

```
Database: agent_agency_test
Tables needed from migrations:
  - 001: agent_registry tables (3 tables) ← ARBITER-001
  - 002: knowledge_seeker tables (5 tables) ← ARBITER-006
  - 003: verification_engine tables (3 tables) ← ARBITER-007
  - 004: web_navigator tables (6 tables) ← ARBITER-008
```

---

## Option 1: Docker Setup (Recommended)

**Best for**: Clean, isolated environment with all migrations

### Steps

```bash
# 1. Start Docker Desktop

# 2. Start PostgreSQL container
cd iterations/v2
docker-compose up -d postgres

# 3. Wait for PostgreSQL to initialize
sleep 5

# 4. Verify container is running
docker ps | grep postgres

# 5. Check if migrations auto-ran (docker-compose mounts migrations/)
docker exec arbiter-postgres-test psql -U postgres -d arbiter_test -c "\dt"

# 6. If tables exist, you're ready!
# If not, run migrations manually:
docker exec -i arbiter-postgres-test psql -U postgres -d arbiter_test < migrations/001_create_agent_registry_tables.sql
docker exec -i arbiter-postgres-test psql -U postgres -d arbiter_test < migrations/002_create_knowledge_tables.sql
docker exec -i arbiter-postgres-test psql -U postgres -d arbiter_test < migrations/003_create_verification_tables.sql
docker exec -i arbiter-postgres-test psql -U postgres -d arbiter_test < migrations/004_create_web_tables.sql

# 7. Verify all tables created
docker exec arbiter-postgres-test psql -U postgres -d arbiter_test -c "\dt" | grep -E "(agent|knowledge|verification|web)_"

# Should show 17 tables total
```

### Configuration for Docker

Tests will work with these environment variables (docker-compose defaults):

```bash
export DB_HOST=localhost
export DB_PORT=5432
export DB_NAME=arbiter_test
export DB_USER=postgres
export DB_PASSWORD=password
```

---

## Option 2: Local PostgreSQL Setup

**Best for**: Using existing local PostgreSQL without Docker

### Steps

```bash
# 1. Create test database
createdb agent_agency_test

# 2. Run migrations in order
cd iterations/v2
psql -d agent_agency_test -f migrations/001_create_agent_registry_tables.sql
psql -d agent_agency_test -f migrations/002_create_knowledge_tables.sql
psql -d agent_agency_test -f migrations/003_create_verification_tables.sql
psql -d agent_agency_test -f migrations/004_create_web_tables.sql

# 3. Verify tables created
psql -d agent_agency_test -c "\dt" | /usr/bin/grep -E "(agent|knowledge|verification|web)_"

# Should show 17 tables total
```

### Configuration for Local PostgreSQL

Set these environment variables before running tests:

```bash
export DB_HOST=localhost
export DB_PORT=5432
export DB_NAME=agent_agency_test
export DB_USER=darianrosebrook  # Your system user
export DB_PASSWORD=""  # Empty for local trust auth
```

---

## Option 3: Skip Database for Now

**Best for**: Focusing on orchestrator integration first

The Web Navigator tests use the database client, but the Web Navigator itself has graceful degradation and can work without a database. You can:

1. Integrate with ArbiterOrchestrator first (code changes only)
2. Run unit tests (they don't need database)
3. Set up database later for integration tests

---

## Immediate Action Plan

### Phase 1A: Database Setup (30 minutes)

**Choose your approach:**

- [ ] **Docker** (recommended): Follow "Option 1" above
- [ ] **Local PostgreSQL**: Follow "Option 2" above
- [ ] **Skip for now**: Proceed to Phase 1B

### Phase 1B: Orchestrator Integration (1 hour)

**File**: `src/orchestrator/ArbiterOrchestrator.ts`

```typescript
// 1. Add imports (top of file)
import { WebNavigator } from "../web/WebNavigator";
import { WebNavigatorConfig } from "../types/web";
import { WebNavigatorDatabaseClient } from "../database/WebNavigatorDatabaseClient";

// 2. Add to ArbiterOrchestratorConfig interface
export interface ArbiterOrchestratorConfig {
  // ... existing config ...

  /** Web Navigator configuration (ARBITER-008) */
  webNavigator?: {
    enabled: boolean;
    http: {
      timeoutMs: number;
      maxConcurrentRequests: number;
      retryAttempts: number;
      retryDelayMs: number;
      userAgent: string;
      followRedirects: boolean;
      maxRedirects: number;
    };
    cache: {
      enabled: boolean;
      ttlHours: number;
      maxSizeMb: number;
    };
    rateLimit: {
      enabled: boolean;
      requestsPerMinute: number;
      backoffMultiplier: number;
      maxBackoffMs: number;
    };
    security: {
      verifySsl: boolean;
      sanitizeContent: boolean;
      detectMalicious: boolean;
      respectRobotsTxt: boolean;
    };
    limits: {
      maxContentSizeMb: number;
      maxExtractionTimeMs: number;
      maxTraversalDepth: number;
      maxPagesPerTraversal: number;
    };
    observability: {
      enableMetrics: boolean;
      enableTracing: boolean;
      logLevel: "debug" | "info" | "warn" | "error";
    };
  };
}

// 3. Add class properties
export class ArbiterOrchestrator {
  // ... existing properties ...
  private webNavigator?: WebNavigator;
  private webNavigatorDbClient?: WebNavigatorDatabaseClient;

  constructor(config: ArbiterOrchestratorConfig) {
    // ... existing initialization ...

    // Initialize Web Navigator if enabled (after database pool is created)
    if (config.webNavigator?.enabled && this.pool) {
      this.webNavigatorDbClient = new WebNavigatorDatabaseClient(this.pool);
      this.webNavigator = new WebNavigator(
        config.webNavigator,
        this.webNavigatorDbClient,
        this.knowledgeSeeker
      );
    }
  }

  // 4. Add getter method
  public getWebNavigator(): WebNavigator | undefined {
    return this.webNavigator;
  }

  // 5. Update getStatus() method to include Web Navigator
  async getStatus(): Promise<ArbiterOrchestratorStatus> {
    const baseStatus = await this.getBaseStatus(); // existing status logic

    // Add Web Navigator status
    if (this.webNavigator) {
      const webNavStatus = await this.webNavigator.getStatus();
      return {
        ...baseStatus,
        components: {
          ...baseStatus.components,
          webNavigator: webNavStatus.enabled,
        },
        // Include webNavigator health and metrics
      };
    }

    return baseStatus;
  }
}
```

**File**: `src/types/arbiter-orchestration.ts`

```typescript
// Add new task types
export enum TaskType {
  // ... existing types ...
  WEB_SEARCH = "web-search",
  WEB_EXTRACT_CONTENT = "web-extract-content",
  WEB_TRAVERSE_LINKS = "web-traverse-links",
}

// Add web extraction metadata
export interface TaskMetadata {
  // ... existing metadata ...

  webExtraction?: {
    url: string;
    extractionType: "main_content" | "links" | "images" | "full_page";
    enableTraversal: boolean;
    traversalConfig?: {
      maxDepth: number;
      maxPages: number;
      sameDomainOnly: boolean;
    };
  };
}
```

### Phase 1C: Configuration (15 minutes)

Create `config/web-navigator.yaml`:

```yaml
web_navigator:
  enabled: true

  http:
    timeout_ms: 10000
    max_concurrent_requests: 20
    retry_attempts: 3
    retry_delay_ms: 1000
    user_agent: "Agent-Agency-WebNavigator/2.0"
    follow_redirects: true
    max_redirects: 5

  cache:
    enabled: true
    ttl_hours: 24
    max_size_mb: 500

  rate_limit:
    enabled: true
    requests_per_minute: 60
    backoff_multiplier: 2
    max_backoff_ms: 60000

  security:
    verify_ssl: true
    sanitize_content: true
    detect_malicious: true
    respect_robots_txt: true

  limits:
    max_content_size_mb: 10
    max_extraction_time_ms: 10000
    max_traversal_depth: 3
    max_pages_per_traversal: 50

  observability:
    enable_metrics: true
    enable_tracing: true
    log_level: info
```

### Phase 1D: Testing (30 minutes)

```bash
# 1. Run unit tests (don't need database)
npm test -- tests/unit/web/

# 2. Run integration tests (need database)
npm test -- tests/integration/web/

# 3. Test orchestrator initialization
npm test -- tests/integration/orchestrator/

# 4. Manual verification
npm run build
node -e "
const { ArbiterOrchestrator } = require('./dist/orchestrator/ArbiterOrchestrator');
const config = require('./config/test-config');
const orchestrator = new ArbiterOrchestrator(config);
orchestrator.getStatus().then(status => {
  console.log('Web Navigator enabled:', status.components.webNavigator);
  process.exit(0);
});
"
```

---

## Success Criteria

### Phase 1 Complete When:

- ✅ Database has all 17 tables (migrations 001-004)
- ✅ ArbiterOrchestrator has WebNavigator integrated
- ✅ Web task types registered
- ✅ Configuration file created
- ✅ Integration tests passing
- ✅ `orchestrator.getWebNavigator()` returns WebNavigator instance

---

## Quick Start Commands

### For Docker Setup:

```bash
# Complete Phase 1A + 1B + 1C + 1D
cd iterations/v2

# Start database
docker-compose up -d postgres

# Run migrations
for migration in migrations/00{1,2,3,4}*.sql; do
  docker exec -i arbiter-postgres-test psql -U postgres -d arbiter_test < "$migration"
done

# Verify
docker exec arbiter-postgres-test psql -U postgres -d arbiter_test -c "\dt" | /usr/bin/grep -c "_"
# Should output: 17

# Run tests
npm test
```

### For Local PostgreSQL Setup:

```bash
# Complete Phase 1A + 1B + 1C + 1D
cd iterations/v2

# Create database and run migrations
createdb agent_agency_test
for migration in migrations/00{1,2,3,4}*.sql; do
  psql -d agent_agency_test -f "$migration"
done

# Verify
psql -d agent_agency_test -c "\dt" | /usr/bin/grep -c "_"
# Should output: 17

# Run tests
DB_NAME=agent_agency_test DB_USER=$(whoami) npm test
```

---

## Dependencies and Blockers

### No Blockers

✅ All dependencies complete:

- ✅ ARBITER-001 (Agent Registry) - Complete
- ✅ ARBITER-006 (Knowledge Seeker) - Complete
- ✅ ARBITER-007 (Verification Engine) - Complete

### Ready to Integrate

All code is ready, committed, and tested. Only integration work remains!

---

## Timeline

| Phase     | Task                     | Effort         | Status     |
| --------- | ------------------------ | -------------- | ---------- |
| 1A        | Database setup           | 30 min         | ⏳ Pending |
| 1B        | Orchestrator integration | 1 hour         | ⏳ Pending |
| 1C        | Configuration            | 15 min         | ⏳ Pending |
| 1D        | Testing                  | 30 min         | ⏳ Pending |
| **Total** | **Phase 1 Complete**     | **~2.5 hours** | **⏳**     |

---

## Questions?

- **Implementation**: See `docs/implementation/ARBITER-008-IMPLEMENTATION-COMPLETE.md`
- **Integration Plan**: See `docs/status/ARBITER-008-INTEGRATION-PLAN.md`
- **System Status**: See `docs/status/ARBITER-SYSTEM-STATUS.md`
- **Working Spec**: See `components/web-navigator/.caws/working-spec.yaml`

---

**Status**: ✅ Ready to integrate - choose your database setup approach and proceed!  
**Recommendation**: Use Docker for cleanest setup (Option 1)  
**Author**: @darianrosebrook
