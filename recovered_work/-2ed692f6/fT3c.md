# Agent Agency V3 - Usage Examples

**Real examples of using the Constitutional AI System**

---

## 🚀 Basic Task Execution

**Example**: Implement a simple user greeting feature

### CLI Execution
```bash
# Execute in auto mode with quality gates
cargo run --bin agent-agency-cli execute \
  "Create a simple greeting function that takes a name parameter and returns a personalized greeting message" \
  --mode auto \
  --risk-tier 3 \
  --watch
```

### API Execution
```bash
# Submit via REST API
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Create a simple greeting function that takes a name parameter and returns a personalized greeting message",
    "execution_mode": "auto",
    "max_iterations": 3,
    "risk_tier": 3
  }'
```

**Expected Output**:
```
🚀 Agent Agency V3 - Autonomous Execution
═══════════════════════════════════════════

🎯 Task: Create a simple greeting function...
🔢 Task ID: 550e8400-e29b-41d4-a716-446655440000
⚡ Mode: AUTO (Quality Gates Enabled)
🎚️  Risk Tier: 3

📋 Phase: Planning and validation
   ✅ Council review passed
   ✅ CAWS compliance validated

📋 Phase: Worker execution
   🔧 Executing implementation...
   📊 Progress: 100%
   ✅ Code generation completed
   ✅ Tests written and passing

🎉 Task completed successfully!
⏱️  Total time: 15s
```

---

## 🛡️ Strict Mode with Manual Approval

**Example**: Implement user authentication (high-risk feature)

### CLI Execution
```bash
cargo run --bin agent-agency-cli execute \
  "Implement user authentication with secure password hashing, JWT tokens, and proper error handling" \
  --mode strict \
  --risk-tier 1 \
  --watch
```

**Expected Interaction**:
```
📋 Phase: Planning and validation
   ⏳ Analyzing requirements...
   ✅ Council review passed

🔒 STRICT MODE: Manual approval required
   📋 Phase: Planning complete - awaiting approval
   Apply planning phase? (y/n): y
   ✅ Approved by user

📋 Phase: Worker execution
   🔧 Executing implementation...
   📊 Progress: 75%
   🔒 STRICT MODE: Manual approval required
   📋 Phase: Code generation complete - awaiting approval
   Apply code changes? (y/n): y
   ✅ Approved by user

🎉 Task completed successfully!
```

---

## 👁️ Dry-Run Mode for Safe Testing

**Example**: Test deployment preparation without making changes

### CLI Execution
```bash
cargo run --bin agent-agency-cli execute \
  "Prepare deployment configuration for production environment with proper environment variables and security settings" \
  --mode dry-run
```

**Expected Output**:
```
👁️ DRY-RUN MODE: All artifacts generated, no filesystem changes

📋 Phase: Planning and validation
   ✅ Analysis completed
   ✅ Artifacts generated

📋 Phase: Worker execution (simulated)
   🔧 Simulating implementation...
   📊 Progress: 100%
   ✅ Code generation simulated
   ✅ Tests simulated

💡 No actual filesystem changes were made
🎉 Dry-run completed successfully!
```

---

## 🎛️ Real-time Intervention

**Example**: Monitor and control a running task

### Start a Long-Running Task
```bash
# Submit a complex task
TASK_ID=$(curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Implement comprehensive user management system with roles, permissions, and audit logging",
    "execution_mode": "auto",
    "max_iterations": 10,
    "risk_tier": 2
  }' | jq -r '.task_id')
```

### Monitor Progress
```bash
# Check status
curl http://localhost:8080/api/v1/tasks/$TASK_ID

# Pause execution
curl -X POST http://localhost:8080/api/v1/tasks/$TASK_ID/pause

# Resume execution
curl -X POST http://localhost:8080/api/v1/tasks/$TASK_ID/resume

# Cancel if needed
curl -X POST http://localhost:8080/api/v1/tasks/$TASK_ID/cancel
```

---

## 🛠️ CLI Intervention Commands

**Example**: Override council decisions

```bash
# Override verdict in strict mode
cargo run --bin agent-agency-cli intervene override $TASK_ID \
  --verdict accept \
  --reason "Approved by security review"

# Modify task parameters
cargo run --bin agent-agency-cli intervene parameters $TASK_ID \
  --param "security_level=high" \
  --param "audit_required=true"

# Inject guidance
cargo run --bin agent-agency-cli intervene guidance $TASK_ID \
  --guidance "Use bcrypt for password hashing with minimum cost of 12"
```

---

## 📊 Monitoring & SLO Tracking

**Example**: Check system health and metrics

### Health Checks
```bash
# API health
curl http://localhost:8080/health

# System metrics
curl http://localhost:8080/metrics
```

### SLO Monitoring
```bash
# List SLOs
curl http://localhost:8080/api/v1/slos

# Check specific SLO status
curl http://localhost:8080/api/v1/slos/task_completion_rate/status

# View SLO measurements
curl http://localhost:8080/api/v1/slos/task_completion_rate/measurements
```

### Alert Management
```bash
# List active alerts
curl http://localhost:8080/api/v1/slo-alerts

# Acknowledge alert
curl -X POST http://localhost:8080/api/v1/slo-alerts/$ALERT_ID/acknowledge \
  -H "Content-Type: application/json" \
  -d '{"acknowledged_by": "admin", "notes": "Investigating root cause"}'
```

---

## 🛡️ Waiver Management

**Example**: Handle quality gate exceptions

### Create Waiver
```bash
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
# Get waiver ID from previous response
WAIVER_ID="your-waiver-id"

curl -X POST http://localhost:8080/api/v1/waivers/$WAIVER_ID/approve
```

---

## 📚 Database Exploration

**Example**: Query saved queries and explore data

### List Saved Queries
```bash
curl http://localhost:8080/api/v1/queries
```

### Execute Saved Query
```bash
# First, save a query
curl -X POST http://localhost:8080/api/v1/queries \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Active Tasks",
    "query_text": "SELECT id, status, created_at FROM tasks WHERE status != '\''completed'\'' ORDER BY created_at DESC"
  }'

# Then execute it via the web dashboard
# Open http://localhost:3000 and use the database explorer
```

---

## 🔍 Provenance Tracking

**Example**: Verify and audit task provenance

### Check Provenance Records
```bash
# List all provenance records
curl http://localhost:8080/api/v1/provenance

# Get provenance for specific commit
curl http://localhost:8080/api/v1/provenance/verify/$(git rev-parse HEAD)

# Link provenance to commit
curl -X POST http://localhost:8080/api/v1/provenance/link \
  -H "Content-Type: application/json" \
  -d '{
    "verdict_id": "task-verdict-id",
    "commit_hash": "'$(git rev-parse HEAD)'"
  }'
```

---

## 🌐 Web Dashboard Usage

**Example**: Use the web interface for monitoring

### Start Dashboard
```bash
cd iterations/v3/apps/web-dashboard
npm install
npm run dev
# Open http://localhost:3000
```

### Dashboard Features
- **Task Monitoring**: Live task progress and status
- **System Metrics**: Real-time performance data
- **Database Explorer**: Query and explore saved data
- **Alert Dashboard**: SLO alerts and acknowledgments
- **Provenance Viewer**: Audit trails and verification

---

## 🔧 Advanced Configuration

**Example**: Configure execution modes and risk tiers

### Environment Variables
```bash
# Set API server configuration
export AGENT_AGENCY_API_URL=http://localhost:8080
export DATABASE_URL=postgresql://localhost/agent_agency_v3

# Worker configuration
export AGENT_AGENCY_WORKER_ENDPOINT=http://localhost:8081

# Security
export API_KEYS="key1,key2,key3"
```

### Risk Tier Guidelines
- **Tier 1**: Critical infrastructure, auth systems, data migration
- **Tier 2**: User-facing features, API changes, data writes
- **Tier 3**: Internal tools, read-only features, documentation

---

## 🚨 Error Handling Examples

**Example**: Handle common errors and recovery

### Worker Unavailable
```bash
# Check worker status
ps aux | grep agent-agency-worker

# Restart worker
cargo run --bin agent-agency-worker &
```

### Database Connection Issues
```bash
# Check database
docker ps | grep postgres

# Reset connection
docker restart agent-agency-db
```

### Task Stuck in Pending
```bash
# Check API server logs
curl http://localhost:8080/health

# Restart API server
cargo run --bin api-server &
```

---

## 📈 Performance Optimization

**Example**: Monitor and optimize system performance

### Performance Monitoring
```bash
# Check response times
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8080/health

# Monitor system metrics
curl http://localhost:8080/metrics | grep task_execution_time
```

### Scaling Considerations
- **Concurrent Tasks**: 50+ simultaneous executions supported
- **API Throughput**: 1000+ requests/minute sustained
- **Database Performance**: <10ms average query time
- **Memory Usage**: Monitor with system health checks

---

**These examples demonstrate the full capabilities of Agent Agency V3 for autonomous task execution with constitutional governance, real-time control, and comprehensive monitoring.**

**Project**: Add accessible button component to design system  
**Risk Tier**: 2 (API stability required)  
**Files Changed**: 8  
**Lines Changed**: 320

```yaml
id: LIB-003
title: "Add Accessible Button Component"
risk_tier: 2
mode: feature
change_budget:
  max_files: 10
  max_loc: 400
blast_radius:
  modules: ["components", "types", "stories"]
  data_migration: false
operational_rollback_slo: "5m"
threats:
  - "Breaking API changes"
  - "Accessibility regressions"
  - "Bundle size increase"
scope:
  in: ["src/components/Button/", "src/types/", "stories/"]
  out: ["src/components/other/", "node_modules/"]
invariants:
  - "Component API remains backward compatible"
  - "All variants pass accessibility audits"
  - "Bundle size impact < 2KB"
  - "TypeScript types exported correctly"
acceptance:
  - id: "A1"
    given: "Developer imports Button component"
    when: "Component is rendered"
    then: "Button displays with correct styling"
  - id: "A2"
    given: "Screen reader user navigates to button"
    when: "Button receives focus"
    then: "Accessible name is announced"
  - id: "A3"
    given: "Button has loading state"
    when: "Loading prop is true"
    then: "Button shows loading indicator and is disabled"
non_functional:
  a11y:
    - "WCAG 2.1 AA compliance"
    - "Keyboard navigation support"
    - "Screen reader compatibility"
    - "Focus management"
  perf:
    bundle_size_kb: 5
  security:
    - "XSS prevention in button content"
    - "Safe event handler binding"
contracts:
  - type: "typescript"
    path: "src/types/button.ts"
observability:
  logs: []
  metrics:
    - "button_click_count"
    - "button_render_count"
  traces: []
migrations: []
rollback:
  - "Remove Button component files"
  - "Update component exports"
  - "Remove button stories"
human_override:
  enabled: false
experimental_mode:
  enabled: false
ai_assessment:
  confidence_level: 8
  uncertainty_areas:
    - "Cross-browser accessibility support"
  complexity_factors:
    - "Multiple button variants (primary, secondary, ghost)"
    - "Loading and disabled states"
  risk_factors: []
```

---

## 🌐 REST API - User Authentication

**Project**: Add JWT-based authentication to user service  
**Risk Tier**: 1 (security critical)  
**Files Changed**: 18  
**Lines Changed**: 850

```yaml
id: API-004
title: "Implement JWT Authentication"
risk_tier: 1
mode: feature
change_budget:
  max_files: 20
  max_loc: 1000
blast_radius:
  modules: ["auth", "users", "middleware", "database"]
  data_migration: true
operational_rollback_slo: "15m"
threats:
  - "Authentication bypass vulnerabilities"
  - "Token exposure in logs"
  - "Database migration failures"
  - "Service downtime during deployment"
scope:
  in: ["src/auth/", "src/users/", "src/middleware/", "migrations/"]
  out: ["src/other-services/", "node_modules/"]
invariants:
  - "All endpoints require valid authentication"
  - "JWT tokens expire within 24 hours"
  - "Failed auth attempts are rate limited"
  - "User passwords are properly hashed"
  - "No sensitive data in application logs"
acceptance:
  - id: "A1"
    given: "User provides valid credentials"
    when: "Login endpoint is called"
    then: "JWT token is returned"
  - id: "A2"
    given: "User provides invalid credentials"
    when: "Login endpoint is called"
    then: "401 Unauthorized is returned"
  - id: "A3"
    given: "Request includes valid JWT"
    when: "Protected endpoint is called"
    then: "Request succeeds with user context"
  - id: "A4"
    given: "Request includes expired JWT"
    when: "Protected endpoint is called"
    then: "401 Unauthorized is returned"
  - id: "A5"
    given: "User logs out"
    when: "Token is used afterward"
    then: "401 Unauthorized is returned"
non_functional:
  a11y:
    - "API documentation accessible"
  perf:
    api_p95_ms: 200
  security:
    - "JWT tokens properly signed"
    - "Password hashing with bcrypt"
    - "Rate limiting on auth endpoints"
    - "CORS properly configured"
    - "Helmet security headers"
    - "Input validation and sanitization"
contracts:
  - type: "openapi"
    path: "docs/api/auth.yaml"
observability:
  logs:
    - "auth.login.success"
    - "auth.login.failure"
    - "auth.token.expired"
    - "auth.logout"
  metrics:
    - "auth_requests_total"
    - "auth_failures_total"
    - "active_sessions"
  traces:
    - "auth_flow"
    - "token_validation"
migrations:
  - "Add users table with password_hash column"
  - "Add user_sessions table for token blacklisting"
  - "Update existing users with hashed passwords"
rollback:
  - "Revert database migration"
  - "Remove authentication middleware"
  - "Restore original endpoint access"
  - "Clear any cached tokens"
human_override:
  enabled: false
experimental_mode:
  enabled: false
ai_assessment:
  confidence_level: 7
  uncertainty_areas:
    - "JWT key rotation strategy"
    - "Session management edge cases"
  complexity_factors:
    - "Database migration with existing users"
    - "Token refresh implementation"
    - "Rate limiting implementation"
  risk_factors:
    - "Security-critical functionality"
    - "Database migration complexity"
```

---

## 🧹 Code Refactor - Extract Service Layer

**Project**: Extract business logic into service layer  
**Risk Tier**: 2 (behavior preservation required)  
**Files Changed**: 12  
**Lines Changed**: 380

```yaml
id: REFACTOR-005
title: "Extract User Service Layer"
risk_tier: 2
mode: refactor
change_budget:
  max_files: 15
  max_loc: 500
blast_radius:
  modules: ["controllers", "services", "models"]
  data_migration: false
operational_rollback_slo: "5m"
threats:
  - "Behavioral changes during extraction"
  - "Import/reference update failures"
  - "Test coverage gaps"
scope:
  in: ["src/controllers/", "src/services/", "src/models/", "tests/"]
  out: ["src/views/", "src/public/", "node_modules/"]
invariants:
  - "All existing APIs return identical responses"
  - "All existing tests continue to pass"
  - "No performance regressions"
  - "Type safety maintained throughout"
acceptance:
  - id: "A1"
    given: "Existing API endpoints"
    when: "Called with same parameters"
    then: "Return identical responses"
  - id: "A2"
    given: "Existing controller tests"
    when: "Executed after refactor"
    then: "All tests pass without modification"
  - id: "A3"
    given: "Service layer methods"
    when: "Called directly"
    then: "Behave identically to controller logic"
non_functional:
  a11y: []
  perf:
    api_p95_ms: 250
  security:
    - "Input validation preserved"
    - "Authorization checks maintained"
contracts:
  - type: "typescript"
    path: "src/types/services.ts"
observability:
  logs: []
  metrics:
    - "service_method_calls"
  traces:
    - "service_operation_flow"
migrations: []
rollback:
  - "Revert controller files to original state"
  - "Remove service layer files"
  - "Update imports back to original"
human_override:
  enabled: false
experimental_mode:
  enabled: false
ai_assessment:
  confidence_level: 9
  uncertainty_areas:
    - "Edge case behavior preservation"
  complexity_factors:
    - "Maintaining exact API compatibility"
    - "Updating all imports and references"
  risk_factors: []
```

---

## 🐛 Bug Fix - Memory Leak in Data Processing

**Project**: Fix memory leak in CSV processing pipeline  
**Risk Tier**: 1 (data integrity + performance)  
**Files Changed**: 3  
**Lines Changed**: 45

```yaml
id: FIX-006
title: "Fix Memory Leak in CSV Processor"
risk_tier: 1
mode: fix
change_budget:
  max_files: 5
  max_loc: 100
blast_radius:
  modules: ["csv-processor", "file-upload"]
  data_migration: false
operational_rollback_slo: "1m"
threats:
  - "Service crashes under load"
  - "Data corruption during processing"
  - "Incomplete processing of large files"
scope:
  in: ["src/csv-processor.ts", "tests/csv-processor.test.ts"]
  out: ["src/other-modules/", "node_modules/"]
invariants:
  - "All CSV files process completely"
  - "Memory usage remains bounded"
  - "Processing performance maintained"
  - "Data integrity preserved"
acceptance:
  - id: "A1"
    given: "Large CSV file (10MB+)"
    when: "Processed through pipeline"
    then: "Memory usage stays under 100MB"
  - id: "A2"
    given: "CSV with malformed data"
    when: "Processed"
    then: "Invalid rows are skipped, valid rows processed"
  - id: "A3"
    given: "Processing interrupted"
    when: "Restarted"
    then: "Can resume from interruption point"
non_functional:
  a11y: []
  perf:
    api_p95_ms: 5000
  security:
    - "File upload size limits enforced"
    - "Path traversal prevented"
contracts: []
observability:
  logs:
    - "csv.processing.started"
    - "csv.processing.completed"
    - "csv.processing.error"
  metrics:
    - "csv_files_processed"
    - "csv_processing_duration"
    - "memory_usage_peak"
  traces:
    - "csv_processing_pipeline"
migrations: []
rollback:
  - "Revert csv-processor.ts to previous version"
  - "Remove any new test files"
human_override:
  enabled: false
experimental_mode:
  enabled: false
ai_assessment:
  confidence_level: 8
  uncertainty_areas:
    - "Memory usage patterns in production"
  complexity_factors:
    - "Streaming processing implementation"
  risk_factors:
    - "Memory leak could cause service outages"
```

---

## 📖 Documentation - API Reference

**Project**: Add comprehensive API documentation  
**Risk Tier**: 3 (no functional changes)  
**Files Changed**: 8  
**Lines Changed**: 1200

```yaml
id: DOC-007
title: "Add API Documentation"
risk_tier: 3
mode: doc
change_budget:
  max_files: 10
  max_loc: 1500
blast_radius:
  modules: ["docs"]
  data_migration: false
operational_rollback_slo: "1m"
threats:
  - "Documentation becomes outdated"
  - "Incomplete coverage"
scope:
  in: ["docs/", "src/"]
  out: ["src/tests/", "node_modules/"]
invariants:
  - "All public APIs are documented"
  - "Examples are runnable"
  - "Documentation builds successfully"
acceptance:
  - id: "A1"
    given: "Developer visits docs site"
    when: "Looks for API reference"
    then: "Finds complete method signatures and descriptions"
  - id: "A2"
    given: "Developer copies example code"
    when: "Runs it"
    then: "Code executes successfully"
non_functional:
  a11y:
    - "Documentation accessible without JavaScript"
  perf: {}
  security: []
contracts: []
observability:
  logs: []
  metrics:
    - "docs_page_views"
  traces: []
migrations: []
rollback:
  - "Remove documentation files"
  - "Revert any API changes made for documentation"
human_override:
  enabled: false
experimental_mode:
  enabled: false
ai_assessment:
  confidence_level: 9
  uncertainty_areas: []
  complexity_factors:
    - "Comprehensive API surface"
  risk_factors: []
```

---

## 🔧 CLI Tool - Add Interactive Mode

**Project**: Add interactive mode to CLI tool  
**Risk Tier**: 3 (low risk feature addition)  
**Files Changed**: 4  
**Lines Changed**: 120

```yaml
id: CLI-008
title: "Add Interactive Mode to CLI"
risk_tier: 3
mode: feature
change_budget:
  max_files: 5
  max_loc: 150
blast_radius:
  modules: ["cli", "commands"]
  data_migration: false
operational_rollback_slo: "1m"
threats:
  - "Interactive mode disrupts existing usage"
  - "New dependencies increase bundle size"
scope:
  in: ["src/cli/", "src/commands/"]
  out: ["src/other/", "node_modules/"]
invariants:
  - "Existing CLI usage unchanged"
  - "Help text remains informative"
  - "Exit codes remain standard"
acceptance:
  - id: "A1"
    given: "User runs command with --interactive"
    when: "Provides inputs"
    then: "Command executes with provided parameters"
  - id: "A2"
    given: "User runs command normally"
    when: "No --interactive flag"
    then: "Behavior identical to before"
non_functional:
  a11y: []
  perf:
    api_p95_ms: 50
  security: []
contracts: []
observability:
  logs:
    - "cli.interactive.started"
    - "cli.interactive.completed"
  metrics:
    - "cli_interactive_usage"
  traces: []
migrations: []
rollback:
  - "Remove interactive mode code"
  - "Remove inquirer dependency"
human_override:
  enabled: false
experimental_mode:
  enabled: false
ai_assessment:
  confidence_level: 9
  uncertainty_areas: []
  complexity_factors:
    - "CLI UX design"
  risk_factors: []
```

---

## 🏗️ Monorepo - Add Shared Component

**Project**: Add shared Button component to monorepo  
**Risk Tier**: 1 (cross-package compatibility)  
**Files Changed**: 12  
**Lines Changed**: 280

```yaml
id: MONO-009
title: "Add Shared Button Component"
risk_tier: 1
mode: feature
change_budget:
  max_files: 15
  max_loc: 400
blast_radius:
  modules: ["shared/ui", "packages/*"]
  data_migration: false
operational_rollback_slo: "10m"
threats:
  - "Breaking changes in consuming packages"
  - "Version mismatch issues"
  - "Build system incompatibilities"
scope:
  in: ["packages/shared/src/ui/", "packages/app1/", "packages/app2/"]
  out: ["packages/other/", "node_modules/"]
invariants:
  - "All packages continue to build"
  - "Component API remains stable"
  - "TypeScript types work across packages"
  - "Bundle sizes remain acceptable"
acceptance:
  - id: "A1"
    given: "Package imports shared Button"
    when: "Component is used"
    then: "Renders correctly with consistent styling"
  - id: "A2"
    given: "Shared component is updated"
    when: "All packages build"
    then: "No breaking changes detected"
non_functional:
  a11y:
    - "WCAG 2.1 AA compliance"
  perf:
    bundle_size_kb: 15
  security:
    - "XSS prevention"
contracts:
  - type: "typescript"
    path: "packages/shared/src/types/ui.ts"
observability:
  logs: []
  metrics:
    - "shared_component_usage"
  traces: []
migrations: []
rollback:
  - "Remove shared component"
  - "Update package imports"
  - "Revert consuming package changes"
human_override:
  enabled: false
experimental_mode:
  enabled: false
ai_assessment:
  confidence_level: 7
  uncertainty_areas:
    - "Cross-package type compatibility"
  complexity_factors:
    - "Monorepo build coordination"
    - "Package versioning strategy"
  risk_factors:
    - "Breaking changes across multiple packages"
```

---

## 📋 Key Patterns Observed

### Risk Tier Patterns

**Tier 1 Projects** (Critical):
- Authentication, billing, data migrations
- API contracts always required
- Manual review mandatory
- Higher change budgets for complexity

**Tier 2 Projects** (Standard):
- Features, UI components, refactorings
- Contracts required for external APIs
- E2E testing recommended
- Balanced change budgets

**Tier 3 Projects** (Low Risk):
- Internal tools, docs, simple fixes
- Minimal testing requirements
- Lower change budgets
- Fast rollback times

### Project Type Patterns

**Extensions**: Focus on webview security, activation performance, VS Code API compliance

**Libraries**: Bundle size, TypeScript exports, backward compatibility, tree-shaking

**APIs**: Authentication, data validation, performance, API contracts, migration planning

**CLIs**: Exit codes, help text, error messages, ergonomics

**Monorepos**: Cross-package compatibility, build coordination, shared component stability

### Common Invariants

1. **Security**: Input validation, XSS prevention, secure defaults
2. **Performance**: Response times, bundle sizes, memory bounds
3. **Compatibility**: Backward compatibility, API stability
4. **Reliability**: Error handling, graceful degradation
5. **Observability**: Logging, metrics, tracing coverage

---

## 🎯 Using These Examples

1. **Find Similar Project**: Look for examples matching your project type and risk level
2. **Copy Structure**: Use the YAML structure as a starting point
3. **Customize Values**: Update IDs, titles, scopes, and budgets for your specific case
4. **Validate Early**: Run `caws validate --suggestions` to catch issues
5. **Iterate**: Refine based on your project's specific requirements

These examples show how CAWS scales from simple fixes to complex monorepo changes while maintaining consistent quality and safety standards.

---

**Examples Version**: 1.0  
**CAWS Version**: 3.1.0  
**Last Updated**: October 2, 2025
