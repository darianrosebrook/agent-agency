# CAWS Planning & Execution Hub

**Central repository for Agent-Agency P0 sprint planning, tracking, and governance.**

This directory contains all artifacts for the **P0 vertical slice sprint** (2-week mission to ship a working end-to-end task execution demo).

---

## Quick Links (Read In This Order)

### 1. **Leadership Overview** (5 min read)
📄 **[EXEC_SUMMARY.md](EXEC_SUMMARY.md)**

What: Problem statement, strategy, timeline, risks  
Who: Leadership, stakeholders, product team  
When: Read first to understand the big picture

---

### 2. **Detailed Roadmap** (15 min read)
📄 **[FAST_WINS_ROADMAP.md](FAST_WINS_ROADMAP.md)**

What: All 10 P0 items with acceptance criteria, effort estimates, and dependencies  
Who: Engineers (both frontend + backend)  
When: Reference during sprint planning; detailed implementation guide

---

### 3. **Daily Execution Checklist** (5 min read)
📄 **[P0_DAILY_CHECKLIST.md](P0_DAILY_CHECKLIST.md)**

What: Status board, weekly schedule, known blockers, code review template  
Who: Daily standup (whole team)  
When: Print this; pin to your desk; reference every morning

---

### 4. **Code Implementation Map** (20 min read)
📄 **[P0_CODE_MAP.md](P0_CODE_MAP.md)**

What: Exact file locations, line numbers, implementation patterns, code snippets  
Who: Engineers implementing each P0 item  
When: Reference while coding; copy/paste patterns

---

## Artifact Directory

```
.caws/
├── README.md                      ← You are here
├── EXEC_SUMMARY.md               ← Leadership + stakeholders
├── FAST_WINS_ROADMAP.md          ← Detailed P0 breakdown
├── P0_DAILY_CHECKLIST.md         ← Daily standup reference
├── P0_CODE_MAP.md                ← Implementation guide
├── working-spec.yaml             ← CAWS spec (governance)
└── ...
```

---

## The 10 P0 Items (2-Week Sprint)

| # | Item | What | Effort | Status |
|---|------|------|--------|--------|
| 1 | **P0-1** | Wire backend proxy + health check | 30m | Pending |
| 2 | **P0-2** | Real worker HTTP execution + circuit-breaker | 2–3h | Pending |
| 3 | **P0-3** | Database + connection pool (tasks, audit, queries) | 3–4h | Pending |
| 4 | **P0-4** | Task API endpoints (GET /tasks, /tasks/:id) | 2–3h | Pending |
| 5 | **P0-5** | Metrics SSE + real-time KPI updates | 2h | Pending |
| 6 | **P0-6** | Chat WebSocket sessions (create + persist) | 2h | Pending |
| 7 | **P0-7** | Audit trail persistence + provenance | 1–2h | Pending |
| 8 | **P0-8** | Security keystore + sandbox path guards | 1–2h | Pending |
| 9 | **P0-9** | Cancelable work (cancel path + UI) | 1–2h | Pending |
| 10 | **P0-10** | Hard-fail on placeholder TODOs (CI gate) | 1h | Pending |

**Total effort**: ~18–23 hours (achievable in 2 weeks, 1–2 engineers, parallel work)

---

## How to Use These Documents

### For Team Leads / Managers

1. Read **EXEC_SUMMARY.md** (understand strategy + risks)
2. Review **P0_DAILY_CHECKLIST.md** (run daily standups from this)
3. Track blockers + escalate immediately
4. Expected outcome: Demo by EOW2

### For Frontend Engineers (P0-1, P0-4, P0-5, P0-6, P0-10)

1. Skim **EXEC_SUMMARY.md** (context)
2. Read **FAST_WINS_ROADMAP.md** (your P0 items in detail)
3. Open **P0_CODE_MAP.md** (file-by-file implementation guide)
4. For each item:
   - Find file locations in code map
   - Read "already exists" section
   - Copy code snippets and adapt
   - Write tests

### For Backend Engineers (P0-2, P0-3, P0-7, P0-8)

1. Skim **EXEC_SUMMARY.md** (context)
2. Read **FAST_WINS_ROADMAP.md** (your P0 items in detail)
3. Open **P0_CODE_MAP.md** (Rust patterns + SQL migrations)
4. Database first: Create migration 007 + pool
5. Then: Worker HTTP, audit trail, security

### For DevOps / Platform

1. Read **EXEC_SUMMARY.md** (understand dependencies)
2. Focus on **P0_CODE_MAP.md** (CI gate setup in P0-10)
3. Ensure test infrastructure ready (Docker, migrations, mock backend)

---

## Daily Standup Agenda (15 min)

Use **P0_DAILY_CHECKLIST.md** for this:

1. **Status board**: What's in Progress? What's blocked?
2. **Blockers**: Any issues? Need help?
3. **Next 24h**: What's the priority?
4. **Risks**: Anything that could slip?

Example:
```
Person A: "P0-3 database migration done, wiring pool now. On track."
Person B: "P0-1 proxy works. Starting P0-4 task API. Need backend contact for worker endpoint?"
Lead:     "OK, escalate that, I'll get the info. Any other blockers?"
```

---

## Making Code Changes

### Before You Code

1. Read your P0 item in **FAST_WINS_ROADMAP.md**
2. Check **P0_CODE_MAP.md** for your files
3. Note what "already exists"
4. Identify what you need to do

### While You Code

1. Follow patterns from **P0_CODE_MAP.md**
2. Add tests (unit + integration)
3. Run lints + typechecks
4. Run full test suite before PR

### Before You Submit PR

Use the **PR template** in **P0_DAILY_CHECKLIST.md**:
- What changed?
- Acceptance criteria met?
- Tests pass?
- Lints pass?
- Blockers for next item?

---

## Tracking Progress

Update **P0_DAILY_CHECKLIST.md** status board daily:

```
| P0-1 | DONE   | Alice | —  | ✅ |
| P0-2 | IN PROGRESS | Bob | — | ⏳ |
| P0-3 | DONE   | Alice | —  | ✅ |
| ...
```

Commit changes to reflect current state.

---

## Risking Hitting a Blocker?

**Immediate steps**:

1. Note it in **P0_DAILY_CHECKLIST.md** (blockers section)
2. Escalate in standup
3. Reference the **mitigation** (in EXEC_SUMMARY.md)
4. **Pivot**: Is there a parallel P0 item you can work on instead?

**Example mitigations**:
- Backend not running? Use mock backend in Node (scaffold it quick)
- DB connection fails? Use SQLite locally; switch to PG later
- Worker endpoint undefined? Use curl to a mock endpoint first

---

## What's P0? What's NOT?

### In P0 (This Sprint)

✅ Backend proxy  
✅ Real task execution (worker HTTP)  
✅ Database persistence (tasks + audit)  
✅ Task API endpoints  
✅ Real-time metrics (SSE)  
✅ Chat WebSocket  
✅ Governance (audit + provenance)  
✅ Security basics (keystore + sandbox)  
✅ Code quality gate (CI block for TODOs)

### NOT in P0 (P1/P2 Later)

❌ Core ML / Candle / ONNX  
❌ Advanced analytics (forecast, correlation, ML)  
❌ Vision/ASR (macOS bridges)  
❌ ONNX quantization  
❌ Workload profiling  
❌ Distributed tracing

**Reason**: Too many unknowns; would block the core loop. Defer after MVP.

---

## Success Criteria (At End of P0)

- [ ] Proxy + health check working
- [ ] Dashboard shows real task list from DB
- [ ] Real worker execution (not simulated)
- [ ] Cancel works + audit logs it
- [ ] Metrics update live
- [ ] Chat sessions persist
- [ ] No secrets in plaintext
- [ ] File ops sandboxed
- [ ] CI blocks TODOs
- [ ] **Full loop**: create task → execute → show result → cancel/audit

---

## Troubleshooting

### "I don't know where to start"

1. Find your P0 item in the status board
2. Read it in **FAST_WINS_ROADMAP.md**
3. Go to **P0_CODE_MAP.md** and find the section
4. Follow the "What You Need to Do" steps
5. Copy patterns from code snippets

### "I'm blocked on something"

1. Note it in **P0_DAILY_CHECKLIST.md** (blockers section)
2. Escalate in standup (don't wait)
3. Check if there's a parallel P0 item you can work on
4. Reference mitigations in **EXEC_SUMMARY.md**

### "I don't understand the acceptance criteria"

1. Re-read it in **FAST_WINS_ROADMAP.md**
2. Ask in standup (no dumb questions)
3. Check **P0_CODE_MAP.md** for test patterns

### "Code changes don't align with the roadmap"

1. Pause; read your P0 item again
2. Check if it's scope creep (P1/P2 sneaking in)
3. If so: defer to later sprint
4. Keep P0 lean

---

## Key Files in This Directory

| File | Purpose | Audience |
|------|---------|----------|
| **EXEC_SUMMARY.md** | Leadership brief (why + when) | Leadership, stakeholders |
| **FAST_WINS_ROADMAP.md** | Detailed P0 breakdown (what + how) | All engineers |
| **P0_DAILY_CHECKLIST.md** | Daily standup reference | Whole team |
| **P0_CODE_MAP.md** | Implementation patterns (where + code) | Implementing engineers |
| **working-spec.yaml** | CAWS governance spec | Compliance, reviews |

---

## Questions? FAQs

**Q: How long is this sprint?**  
A: 2 weeks (EOW1: demo with proxy + real API data; EOW2: full cycle + stakeholder ready).

**Q: What if we slip?**  
A: Identify blockers early. Escalate. Pivot to parallel items. Extend to week 3 if critical.

**Q: Can I do P1/P2 stuff while waiting?**  
A: **No**. P0 is tight. Stay focused. P1 starts after P0 ships.

**Q: How do I know if I'm done?**  
A: Read acceptance criteria in **FAST_WINS_ROADMAP.md**. Check off boxes. PR review with lead.

**Q: What about testing?**  
A: Unit + integration tests for every P0 item. No exceptions. Linter + typecheck before PR.

---

## Version

**Version**: 1.0  
**Date**: October 2025  
**Sprint Target**: EOW2 (2 weeks)  
**Status**: Ready to implement

---

**Next Step**: Start with **EXEC_SUMMARY.md** if you're new. Then grab your P0 item from **P0_DAILY_CHECKLIST.md** and reference **P0_CODE_MAP.md** for implementation.

Let's ship it.
