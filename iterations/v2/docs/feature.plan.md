# Feature Plan – V2 Mock Replacement Roadmap

## 1. Context & Goals
- **Working Spec**: `PROJ-785` (Tier 2, mode `feature`)
- **Objective**: Replace mocked infrastructure in V2 iteration with production-ready integrations while preserving existing prompting/orchestration guarantees.
- **Success Criteria**:
  - Critical services start and run continuously without manual keep-alive loops.
  - External systems (agent registry, incident tooling, metrics, cache, LLM providers) integrated through testable adapters.
  - Tier 2 quality gates satisfied: ≥80% branch coverage, ≥50% mutation score, contract tests passing.
  - Observability signals expose health, failures, and compliance state.

## 2. Scope & Change Budget
- **In-Scope** (`.caws/working-spec.yaml`):
  - `iterations/v2/src/**` (startup, allocator, failure manager, verification, memory, validation, evaluation, capabilities, monitoring, orchestration).
  - `iterations/v2/tests/**` (unit, integration, contract, e2e updates).
  - `iterations/v2/docs/**` (this plan, runbooks, integration docs).
  - `apps/mcp-arbiter-observer/**` & `docs/api/**` when adapters require API contract updates.
- **Out of Scope**: `iterations/v2/data/**`, build artifacts, dependency bumps except for required clients.
- **Budget**: 75 files / 7000 LOC → track delta per commit, favor incremental PRs by phase.

## 3. Dependencies & Environment
- **Adapters to Implement** (dependency-first design):
  1. `AgentRegistryClient` – REST/gRPC client or database gateway for agent states.
  2. `IncidentNotifier` – PagerDuty/Slack/email bridge with retry/backoff.
  3. `InfrastructureController` – restart/switchover/scale isolation primitives.
  4. `DistributedCacheClient` – Redis (preferred) or pluggable key-value interface.
  5. `MetricsBackend` – Prometheus remote write or OpenTelemetry exporter.
  6. `LLMProviderFactory` – OpenAI/Anthropic connectors with rate limiting.
  7. `RubricEvaluationAdapter` – DSPy rubric service bridge.
  8. `SecureQueueBackend` – Task queue with ACL + audit log persistence.
- **Secrets & Config**: load via existing config service/env with redaction per invariants.
- **Pre-work**: Create waiver `WV-0001` or adjust spec to reflect actual waivers before implementation.

## 4. Architecture Sketch
```
┌──────────┐   enqueue    ┌────────────────┐   health/evidence    ┌────────────────────┐
│ Orchestr │────────────►│ TaskQueue (Sec) │─────────────────────►│ VerificationEngine │
└────┬─────┘              └──────┬─────────┘                     └─────────┬──────────┘
     │                            │ ACL/audit                                 │ cache ops
     │                            ▼                                            ▼
     │                    ┌───────────────┐                      ┌────────────────────────┐
     │                    │ ResourceAlloc │─────registry────────►│ AgentRegistryClient     │
     │                    └──────┬───────┘                      └────────────────────────┘
     │                           │ agent telemetry                            ▲
     │                           ▼                                            │ incidents
     │                    ┌───────────────┐                      ┌────────────────────────┐
     │ verify results ◄──│ FailureManager│◄────alerts───────────│ IncidentNotifier        │
     │                           │ recover                       └────────────────────────┘
     │                           ▼                                            ▲
     │                    ┌───────────────┐                      ┌────────────────────────┐
     │                    │ MetricsCollect│──metrics/traces────►│ MetricsBackend/O11y     │
     │                    └───────────────┘                      └────────────────────────┘
     │                           │ feedback                                   │ RL training
     ▼                           ▼                                            ▼
┌──────────────┐        ┌─────────────────────┐                     ┌────────────────────┐
│ ModelBasedJudge│─────►│ DSPyEvaluationBridge │────rubric─────────►│ Rubric Service     │
└──────────────┘        └─────────────────────┘                     └────────────────────┘
```
- All adapters exposed behind typed interfaces to enable deterministic tests (inject in constructors).
- Observability flow: structured logs → centralized logger; metrics/traces via OpenTelemetry instrumentation.

## 5. Work Phases & Deliverables
### Phase 1 – Runtime Foundations (Critical)
1. Implement `ServiceOrchestrator` bootstrap: start HTTP (health + metrics), MCP server, task loop, performance monitor.
2. Integrate `ResourceAllocator.getAvailableAgents()` with registry client + health gating.
3. Replace `FailureManager` escalation and recovery stubs with pluggable incident & infra controllers.
4. Reinstate `TaskQueue` secure variant with ACL checks and audit logging.
**Artifacts**: adapter interfaces, configuration docs, integration tests for startup + allocation + escalation.

### Phase 2 – Verification & Memory Core (High)
1. Real `VerificationEngine` health checks, evidence aggregation, conflict resolution; wire `ArbiterOrchestrator.verifyEvidence`.
2. Implement `FederatedLearningEngine` distributed cache persistence, retrieval, tenant attribution.
3. Add version-filtered metrics in `ModelDeploymentManager`.
**Artifacts**: verification method registry spec, cache schema doc, conflict resolution tests.

### Phase 3 – Compliance & Observability (High)
1. `WaiverManager` notifications + durable audit logging (append-only store).
2. `CAWSValidator.executeQualityGates` orchestrating lint, coverage, mutation, contract tasks with waiver awareness.
3. `MetricsCollector.getHistoricalMetrics` + `ImprovementEngine` metrics querying backed by telemetry store.
4. Restore CoT log retrieval in MCP server.
**Artifacts**: runbook for quality gate CLI, observability dashboard specs, audit storage schema.

### Phase 4 – Advanced Capabilities (Medium)
1. `FeedbackPipeline.sendToTraining` integration + RL capability fixes (dynamic agent IDs, routing telemetry).
2. `ConstitutionalRuleEngine` precedent matching via ML/NLP adapter (service stub + contract tests).
3. `ModelBasedJudge` real LLM provider routing with retries, tracking.
4. `DSPyEvaluationBridge` rubric integration, `ViolationHandler` modify actions with sanitizer hooks.
**Artifacts**: evaluation contracts, RL telemetry dashboards, security review checklist.

## 6. Test Strategy
| Layer        | Key Scenarios                                                           | Tooling                                  |
| ------------ | ----------------------------------------------------------------------- | ---------------------------------------- |
| Unit         | Adapter failure modes, waiver transitions, cache serializers            | Vitest/Jest with dependency injection    |
| Contract     | Agent registry API schema, incident notifier payloads, LLM provider I/O | Pact / Nock fixtures, OpenAPI validation |
| Integration  | Service startup (happy/fail), secure queue auth, verification pipeline  | Testcontainers (Redis/Postgres), worker fakes |
| E2E Smoke    | Task submission → verification → feedback loop → metrics emission       | Existing orchestrator e2e harness        |
| Mutation     | Focus on failure recovery paths, waiver decisions, validation gates     | `npm run test:mutation` targeted modules |

- **Regression Controls**: ensure new tests fail before implementation; maintain coverage >80% via coverage thresholds.
- **CI Hooks**: extend `npm run verify` to include new adapters (e.g., linting generated clients).

## 7. Data & Fixture Plan
- **Synthetic Fixtures**: JSON payloads for agent registry responses, incident tickets, metrics snapshots.
- **Seeded Stores**: Local Redis/Postgres docker-compose for cache/audit tests; seeded tenants/topics for federated learning.
- **Time Control**: Use fake timers/interfaces for expiry, cooldown, retry logic.
- **PII/Sensitive Data**: sanitize fixtures to comply with invariant (“Sensitive data is redacted…”).

## 8. Observability Plan
- **Logs**: Structured logs (`logger.info/warn/error`) with event ids (`startup_completed`, `incident_escalated`).
- **Metrics**:
  - `agent_registry.available_agents`
  - `failure_manager.escalations_total`
  - `verification.method_health_ratio`
  - `federated_cache.hit_ratio`
  - `quality_gates.failures_total`
- **Traces**: Span around `TaskQueue.enqueue`, `VerificationEngine.verify`, `FeedbackPipeline.sendToTraining`.
- **Dashboards**: Configure Grafana panels for escalation latency, verification SLA, RL performance delta.

## 9. Risks & Mitigations
- **External Dependency Unavailability** → Provide fallback mocks with clear feature flags; document degraded behavior.
- **Performance Degradation** → Benchmark adapters (Phase 1) and add circuit breakers, connection pools.
- **Security Regression** → Security review for task queue ACL, audit log retention policy, incident data redaction.
- **Change Budget Overrun** → Phase-based PRs, reuse shared utilities, auto-gen clients where possible.

## 10. Open Questions
1. Which incident management platform (PagerDuty, OpsGenie, custom)? Confirm API credentials.
2. Preferred distributed cache (Redis, DynamoDB) and existing infrastructure endpoints?
3. Are LLM provider credentials and usage policies already handled by config service?
4. Does quality gate execution run locally only or needs remote runner integration?
5. Any backward-compat requirements with V1 components still in production?

## 11. Next Immediate Actions
1. Resolve waiver validation (`caws waiver create WV-0001`) or adjust spec.
2. Confirm adapter targets (registry URL, cache host, metrics backend, incident tooling).
3. Draft interface definitions + config wiring for Phase 1 adapters.
4. Prepare test harness updates (Redis container, incident notifier fake).

