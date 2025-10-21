# P0 Daily Checklist & Status

**Last Updated**: [Date of session]  
**Target**: Ship P0 by EOW2 (2 weeks from start)

---

## Quick Status Board

| Item   | Status | Owner | Blocker | ETA |
|--------|--------|-------|---------|-----|
| P0-1   | TODO   | —     | —       | 30m |
| P0-2   | TODO   | —     | P0-3    | 2–3h |
| P0-3   | TODO   | —     | —       | 3–4h |
| P0-4   | TODO   | —     | P0-1 + P0-3 | 2–3h |
| P0-5   | TODO   | —     | P0-1    | 2h |
| P0-6   | TODO   | —     | P0-1 + P0-3 | 2h |
| P0-7   | TODO   | —     | P0-3    | 1–2h |
| P0-8   | TODO   | —     | —       | 1–2h |
| P0-9   | TODO   | —     | P0-2    | 1–2h |
| P0-10  | TODO   | —     | —       | 1h |

---

## This Week's Targets

### Monday (Today)

- [ ] P0-1: Set `V3_BACKEND_HOST`; verify `/api/health` and `/api/proxy/api/tasks` proxies correctly
- [ ] P0-3: Create migration 007 (tasks + audit_logs + saved_queries tables)
- [ ] P0-3: Initialize `PgPool` in `database/src/lib.rs`

**Acceptance**: Health and proxy routes return 200 with real backend data.

**Effort**: 2–3 hours

---

### Tuesday

- [ ] P0-4: Implement `GET /api/v1/tasks` endpoint (list from DB)
- [ ] P0-4: Implement `GET /api/v1/tasks/:id` endpoint
- [ ] P0-4: Wire frontend `api-client.ts` to call real backend

**Acceptance**: Dashboard task table populates with real data; drill-down works.

**Effort**: 2–3 hours

---

### Wednesday

- [ ] P0-5: Implement `GET /api/v1/metrics/stream` (SSE)
- [ ] P0-5: Connect MetricsDashboard to SSE; mutate tiles on new events

**Acceptance**: KPI tiles update in real-time without page refresh.

**Effort**: 2 hours

---

### Thursday

- [ ] P0-6: Create `chat_sessions` + `chat_messages` tables
- [ ] P0-6: Implement `POST /api/v1/chat/session` + WS handler
- [ ] P0-6: Wire frontend chat component

**Acceptance**: Create session, send message via WS, verify persistence on reconnect.

**Effort**: 2 hours

---

### Friday

- [ ] P0-2: Replace simulated worker with real HTTP calls
- [ ] P0-2: Add circuit-breaker + retry logic (already imported)
- [ ] P0-2: Test failure injection + verify retry behavior

**Acceptance**: Task executes on real worker; retries on transient failures.

**Effort**: 2–3 hours

---

## Next Week

- P0-7: Audit trail wiring (Monday)
- P0-9: Cancel path (Tuesday)
- P0-8: Security minimums (Wednesday)
- P0-10: TODO hard-fail gate (Thursday)
- Integration + demo prep (Friday + Monday)

---

## Known Blockers & Mitigations

| Blocker | Impact | Mitigation |
|---------|--------|-----------|
| Backend not running | P0-1 fails | Use mock backend / scaffold fake endpoints |
| Database not accessible | P0-3 fails | Use SQLite for local dev; migrate to PG later |
| Worker endpoint not defined | P0-2 fails | Use mock HTTP endpoint; tunnel through proxy |
| —  | — | — |

---

## Code Review Checklist (Before PR)

- [ ] All TODOs converted to tasks (not in code)
- [ ] Tests written + pass (`npm test` + `cargo test`)
- [ ] Lints pass (`npm run lint` + `cargo check`)
- [ ] Migrations are idempotent
- [ ] No new external dependencies without approval
- [ ] Docstrings added for public APIs
- [ ] Commit message includes P0-X reference

---

## PR Template (Copy/Paste)

```markdown
# P0-X: [Item Name]

## What
[1-line summary of change]

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

## Files Changed
- `path/to/file1.rs`
- `path/to/file2.ts`

## Testing
- Unit: [yes/no]
- Integration: [yes/no]
- Manual: [description]

## Risks
[Any known issues or rollback needed]

## Blockers
[Dependencies on other P0 items]
```

---

## Fast Commands

```bash
# Run health check
curl http://localhost:3000/api/health

# Run proxy test
curl -X GET http://localhost:3000/api/proxy/tasks

# Run migrations
cargo sqlx migrate run --database-url $DATABASE_URL

# Run tests
npm run test:v3
cargo test --workspace

# Lint
npm run lint:v3
cargo check --workspace
```

---

## Parking Lot (Defer to P1/P2)

- Core ML integration (P2)
- Advanced analytics (forecast, correlation) (P2)
- Vision/ASR Swift bridges (P2)
- ONNX quantization (P2)
- Workload analyzer profiling (P2)
- Historical trend analysis (P1)
- Advanced waiver governance (P1)

---

## Notes for Next Session

- If backend not ready: scaffold fake task API with static data in `mock-backend.rs`
- Feature-flag simulated worker to keep old paths until real path lands
- Consider breaking P0-3 into smaller migration PRs if DB schema gets large
