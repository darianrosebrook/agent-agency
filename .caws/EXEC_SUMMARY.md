# Executive Summary: Agent-Agency P0 Vertical Slice

**Status**: Ready to implement  
**Duration**: 2 weeks (parallel work)  
**Resource needs**: 1–2 engineers  
**Expected outcome**: Working demo with real task execution, persistence, and governance

---

## The Problem We're Solving

**Current state**:  
- Dashboard and backend components exist but **don't talk** (proxy not configured)
- Worker execution is **simulated** (no real task runs)
- Persistence is **in-memory** (data lost on restart)
- Observability is **logged, not surfaced** (no real-time metrics)
- Code **full of TODOs** (no gate preventing incomplete work)

**Impact**: Can't demo to stakeholders; can't get real feedback; technical debt accumulates.

---

## P0 Strategy: Vertical Slice

Instead of building breadth (10% of everything), build **depth on one path** (100% of core loop):

```
User/CLI
  ↓
Dashboard (mock)
  ↓
Backend API (Rust)
  ↓
Worker (real HTTP call)
  ↓
Database (persist task + results)
  ↓
Dashboard (show result + audit)
```

**Why**: Every component is real. Every link is tested. Feedback loops work. Foundation is solid.

---

## The 10 P0 Items (2-Week Sprint)

| Item | What | Effort | Blocker | Impact |
|------|------|--------|---------|--------|
| **P0-1** | Wire proxy + health | 30m | — | Unblocks all APIs |
| **P0-3** | DB + connection pool | 3–4h | — | Persistence foundation |
| **P0-4** | Task API (list/detail) | 2–3h | P0-1 + P0-3 | Dashboard visibility |
| **P0-5** | Metrics SSE + KPIs | 2h | P0-1 | Real-time monitoring |
| **P0-6** | Chat WebSocket | 2h | P0-1 + P0-3 | Demo feature parity |
| **P0-2** | Worker HTTP + CB | 2–3h | P0-3 | Real execution |
| **P0-7** | Audit trail DB | 1–2h | P0-3 | Governance |
| **P0-9** | Cancel path | 1–2h | P0-2 | Task control |
| **P0-8** | Keystore + sandbox | 1–2h | — | Security baseline |
| **P0-10** | TODO hard-fail gate | 1h | — | Code quality |

**Total**: ~18–23 hours of focused work (achievable in 2 weeks with 1–2 engineers working in parallel).

---

## Timeline: Week by Week

### Week 1

**Mon**: P0-1 (proxy) + P0-3 start (DB schema)  
**Tue**: P0-4 (task API) + P0-5 (metrics SSE)  
**Wed**: P0-6 (chat WS) + P0-8 (security)  
**Thu**: P0-2 (worker HTTP) + testing  
**Fri**: P0-7 (audit) + P0-9 (cancel) + P0-10 (CI gate)

### Week 2

**Mon**: Integration testing + edge cases  
**Tue–Wed**: Performance optimization + documentation  
**Thu**: Demo prep + stakeholder walkthrough  
**Fri**: Deployment + rollout monitoring

---

## Success Criteria (At P0 Completion)

- [ ] `curl http://localhost:3000/api/health` returns backend status
- [ ] Dashboard shows real task list from DB
- [ ] Creating a task executes on real worker (not simulated)
- [ ] Worker can be canceled; audit log records it
- [ ] Metrics tiles update in real-time via SSE
- [ ] Chat sessions persist across reconnects
- [ ] Secrets never logged in plaintext
- [ ] File ops blocked outside sandbox
- [ ] PRs with `PLACEHOLDER` tags fail CI
- [ ] **End-to-end task lifecycle works**: create → execute → result → cancel/audit

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Backend not running | P0-1 fails | Use mock backend / scaffold stubs |
| DB connection issues | P0-3 fails | Use SQLite locally; PG in prod |
| Worker endpoint undefined | P0-2 fails | Use mock HTTP server for testing |
| Scope creep (P1/P2 bleeding in) | Timeline slips | Strict scope gate; defer analytics/ML |
| Linter/type errors block progress | Velocity dips | Run lints early; CI gates strict |

---

## Dependency Management

**Parallel tracks** (minimal blocking):

1. **API Infrastructure** (P0-1, P0-3, P0-4, P0-5, P0-6)
   - Only dependency: proxy must work first
   - ~1 week effort
   - Can be done in parallel with track 2

2. **Worker Execution** (P0-2, P0-9)
   - Depends on P0-3 (DB) for task state
   - ~1 week effort
   - Can start after P0-3

3. **Governance** (P0-7, P0-8, P0-10)
   - Independent; can run concurrently
   - ~3–5 days effort

---

## What We're NOT Doing (P1/P2)

**Deliberately deferred**:

- ❌ Core ML / Candle / ONNX model inference (too many unknowns)
- ❌ Advanced analytics (forecasting, correlation, ML anomaly detection)
- ❌ Vision/ASR bridges (macOS-specific; complex)
- ❌ ONNX quantization
- ❌ Workload profiling + optimization
- ❌ Distributed tracing (Jaeger/Zipkin)

**Why**: These are P2 (after MVP ships). Focus on core loop first.

---

## Resource Allocation

### Recommended Team

- **Engineer A** (2 weeks, 40h): Backend (P0-2, P0-3, P0-7, P0-8)
  - Database setup + migrations
  - Worker HTTP integration
  - Security minimums
  - Audit trail

- **Engineer B** (2 weeks, 40h): Frontend (P0-1, P0-4, P0-5, P0-6) + DevOps (P0-10)
  - Proxy configuration
  - API client implementation
  - Metrics SSE + KPI updates
  - Chat WebSocket
  - CI gate for TODOs

**Daily sync**: 15 min standup (blockers + next actions)  
**PR review**: Same-day turnaround (blocking → escalate)

---

## Expected Outcomes

### By EOW1

- Dashboard and backend talk (health + proxy working)
- Task API populated with real data from DB
- Real worker HTTP calls (with retry)
- Metrics tiles update live

### By EOW2

- Full task lifecycle works: create → execute → audit → cancel
- Chat sessions persist + reconnect works
- PRs with critical TODOs blocked by CI
- Demo ready for stakeholders

### Business Impact

- **Feedback loop**: Real data shows what works/breaks → iterate fast
- **Governance**: Audit trail + security baseline → leadership confidence
- **Foundation**: Persistence + real execution → can build P1 features on solid ground
- **Morale**: Working demo → team momentum

---

## Success Metrics

Track these to measure P0 impact:

| Metric | Target | How |
|--------|--------|-----|
| Demo-able | 1 end-to-end flow | Task creation → result in dashboard |
| Code quality | Zero critical TODOs in prod | CI gate enforcement |
| Uptime | 99%+ during demo | No mid-demo failures |
| Latency | <500ms API response | P95 measured + monitored |
| Data persistence | 100% | Restart service; verify tasks still there |
| Coverage | 80%+ unit tests | Linter gate + CI report |

---

## Communication Plan

**Internal**:
- Daily standup (15 min, blocking items)
- Weekly sync with product (metrics + feedback)
- PR reviews (same-day if possible)

**External** (stakeholders):
- EOW1 demo (dashboard + proxy working)
- EOW2 demo (full task lifecycle)
- Post-launch retrospective (what worked, what didn't)

---

## Next Steps (Start Monday)

1. **Read the roadmap**: `.caws/FAST_WINS_ROADMAP.md` (10 min)
2. **Review code map**: `.caws/P0_CODE_MAP.md` (15 min per file)
3. **Assign items**: Team splits P0-1..P0-10 into parallel tracks
4. **First commit**: P0-1 (set `V3_BACKEND_HOST` + verify proxy)
5. **Daily standups**: 15 min each morning

---

## Questions?

**"How long before we can show this to customers?"**  
EOW2 (2 weeks). Working demo with real data + audit trail.

**"What if we find blockers?"**  
Escalate immediately in standup. Have workarounds ready (mock backend, SQLite, etc.).

**"Why not just ship what exists?"**  
Current state is 40% simulation + in-memory. Can't trust it for real feedback.

**"What about P1 and P2?"**  
After P0 ships and stabilizes. These are iterative improvements, not blockers.

---

**Prepared by**: Agent-Agency Team  
**Date**: October 2025  
**Target**: EOW2 (2-week sprint)
