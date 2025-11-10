# V3 Theory Alignment Implementation Roadmap

**Author:** @darianrosebrook  
**Date:** 2025-01-28  
**Purpose:** Prioritized action plan to achieve full alignment with theory.md requirements

---

## Executive Summary

**Current Alignment:** 78%  
**Target Alignment:** 95%+  
**Estimated Timeline:** 4-6 weeks

**Critical Path:**
1. Complete CoreML integration (blocker for local high-performance)
2. Connect existing agent/telemetry endpoints to dashboard
3. Implement settings management system
4. Complete verdict history and notifications

---

## Priority 1: CoreML Integration (Blocker)

**Status:** 60% complete - Infrastructure exists, API incomplete  
**Target:** 95% complete  
**Timeline:** 2-3 weeks  
**Blocker Level:** HIGH - Prevents verification of 2.8x ANE speedup

### Tasks

1. **Complete CoreML Swift/C++ Bridge** (1 week)
   - Implement Swift bridge for CoreML API calls
   - Create C++ wrapper if needed
   - Replace placeholder implementations
   - **Files:** `system-acceleration/src/ane/compat/coreml_direct.rs`

2. **Implement Real Model Inference** (1 week)
   - Replace placeholder `prediction_from_features` implementation
   - Implement MLFeatureProvider conversion
   - Add error handling for CoreML API errors
   - **Files:** `system-acceleration/src/ane/compat/coreml_direct.rs:34-50`

3. **Verify ANE Acceleration** (3-5 days)
   - Test with actual Mistral model
   - Measure ANE vs CPU performance
   - Verify 2.8x speedup target
   - Document performance benchmarks

4. **Complete Whisper Integration** (3-5 days)
   - Implement real Whisper decoder
   - Replace placeholder transcriptions
   - **Files:** `system-acceleration/src/ane/infer/whisper.rs:296-416`

### Success Criteria

- ✅ CoreML models load and execute successfully
- ✅ ANE acceleration verified (2.8x speedup measured)
- ✅ No placeholder implementations remain
- ✅ Performance benchmarks documented

---

## Priority 2: Dashboard Integration (High Value)

**Status:** 76% complete - Core features work, advanced features missing UI  
**Target:** 90% complete  
**Timeline:** 1-2 weeks  
**Blocker Level:** MEDIUM - Blocks advanced dashboard features

### Tasks

1. **Create Missing API Clients** (1 day)
   - `src/lib/api/agents.ts` - Agent management endpoints
   - `src/lib/api/telemetry.ts` - Telemetry endpoints
   - `src/lib/api/judges.ts` - Judge management endpoints
   - **Effort:** 4-6 hours

2. **Create Agent Stats Page** (2-3 days)
   - Route: `/agent-stats`
   - Components:
     - Agent list with stats
     - Contribution charts
     - Model usage visualization
   - **Endpoints:** `/api/v1/agents`, `/api/v1/agents/stats`, `/api/v1/telemetry/contributions`

3. **Create Agent Health Page** (2-3 days)
   - Route: `/agent-health`
   - Components:
     - Health dashboard
     - Metrics visualization
     - Alert display
   - **Endpoints:** `/api/v1/agents/:id/health`, `/api/v1/observability/alerts`

4. **Create Telemetry Dashboard** (2-3 days)
   - Route: `/telemetry` or integrate into system page
   - Components:
     - Contribution charts
     - Model contribution stream
     - Agent activity timeline
   - **Endpoints:** All telemetry endpoints

### Success Criteria

- ✅ All existing endpoints have dashboard UI
- ✅ Agent management fully accessible via dashboard
- ✅ Telemetry visualizations functional
- ✅ No "endpoint exists but no UI" gaps

---

## Priority 3: Settings Management System

**Status:** 0% complete - No endpoints, no UI  
**Target:** 100% complete  
**Timeline:** 1-2 weeks  
**Blocker Level:** MEDIUM - Blocks settings pages

### Tasks

1. **Database Schema** (1 day)
   - Create `user_settings` table
   - Create `app_settings` table
   - Create `integrations` table
   - Create `api_keys` table
   - **File:** `data-infrastructure/migrations/016_settings.sql`

2. **Implement Settings Endpoints** (1 week)
   - User settings CRUD (4 endpoints)
   - App settings CRUD (2 endpoints)
   - Integration management (3 endpoints)
   - API key management (3 endpoints)
   - Password change (1 endpoint)
   - 2FA endpoints (2 endpoints)
   - **File:** `data-interfaces-adapters/src/bin/api-server.rs`

3. **Create Settings Dashboard** (3-5 days)
   - Route: `/settings`
   - Components:
     - User profile settings
     - App configuration
     - Integration management
     - API key management
     - Security settings (2FA)
   - **File:** `apps/v3-agent-agency-dashboard/app/(dashboard)/settings/`

### Success Criteria

- ✅ All 12 settings endpoints implemented
- ✅ Settings dashboard fully functional
- ✅ User can manage profile, integrations, API keys
- ✅ 2FA can be enabled/disabled

---

## Priority 4: Complete Council Features

**Status:** 90% complete - Minor gaps  
**Target:** 100% complete  
**Timeline:** 1 week  
**Blocker Level:** LOW - Features work, enhancements needed

### Tasks

1. **Implement Verdict History** (2-3 days)
   - Complete `get_verdict_history` implementation
   - Add query endpoints for historical decisions
   - **File:** `agent-constitutional-council/src/verdict_writer.rs:370`

2. **Implement Notification System** (2-3 days)
   - Complete verdict change notifications
   - Add webhook/email integration
   - **File:** `agent-constitutional-council/src/verdict_writer.rs:329`

3. **Complete Git Trailer Integration** (1-2 days)
   - Implement CAWS-VERDICT-ID git trailer
   - Add publication step to adjudication cycle
   - **File:** `agent-constitutional-council/src/verdict_writer.rs`

### Success Criteria

- ✅ Verdict history queryable via API
- ✅ Notifications sent on verdict changes
- ✅ Git trailers added to commits

---

## Priority 5: Rules & Governance System

**Status:** 0% complete - No endpoints, no UI  
**Target:** 100% complete  
**Timeline:** 2-3 weeks  
**Blocker Level:** LOW - Advanced feature

### Tasks

1. **Database Schema** (1 day)
   - Create `rules` table
   - Create `rule_violations` table
   - Create `rule_history` table

2. **Implement Rules Endpoints** (1-2 weeks)
   - Rules CRUD (5 endpoints)
   - Compliance checking (2 endpoints)
   - Violation tracking (2 endpoints)

3. **Create Rules Dashboard** (1 week)
   - Route: `/rules-governance`
   - Components:
     - Rules list and management
     - Compliance dashboard
     - Violation tracking

---

## Priority 6: Documentation Updates

**Status:** Needs update  
**Timeline:** 1 day  
**Blocker Level:** LOW - Documentation accuracy

### Tasks

1. **Update API Gap Analysis** (2-3 hours)
   - Audit all implemented endpoints
   - Update coverage percentages
   - Remove incorrect "missing" claims
   - **File:** `iterations/v3/docs/API_GAP_ANALYSIS.md`

2. **Create Endpoint Reference** (2-3 hours)
   - Complete list of all endpoints
   - Request/response schemas
   - Usage examples
   - **File:** `iterations/v3/docs/API_REFERENCE.md`

---

## Timeline Summary

| Priority | Task | Duration | Start Week | End Week |
|----------|------|----------|------------|----------|
| **P1** | CoreML Integration | 2-3 weeks | Week 1 | Week 3 |
| **P2** | Dashboard Integration | 1-2 weeks | Week 1 | Week 2 |
| **P3** | Settings Management | 1-2 weeks | Week 2 | Week 4 |
| **P4** | Council Features | 1 week | Week 3 | Week 4 |
| **P5** | Rules & Governance | 2-3 weeks | Week 4 | Week 7 |
| **P6** | Documentation | 1 day | Week 1 | Week 1 |

**Total Estimated Timeline:** 6-7 weeks for complete alignment

**Critical Path:** CoreML integration (blocks local high-performance verification)

---

## Success Metrics

### Alignment Targets

| Requirement | Current | Target | Gap |
|------------|---------|--------|-----|
| CAWS Constitutional Authority | 90% | 95% | 5% |
| Local High-Performance | 60% | 95% | 35% |
| Intelligent Arbitration | 95% | 95% | 0% |
| Model-Agnostic Design | 85% | 90% | 5% |
| Low-Level Implementation | 95% | 95% | 0% |
| Correctness & Traceability | 90% | 95% | 5% |
| **Overall Alignment** | **78%** | **95%** | **17%** |

### Dashboard Integration Targets

| Category | Current | Target | Gap |
|----------|---------|--------|-----|
| Core Features | 100% | 100% | 0% |
| Advanced Features | 0% | 100% | 100% |
| Missing Features | 0% | 80% | 80% |
| **Overall Integration** | **76%** | **95%** | **19%** |

---

## Risk Assessment

### High Risk Items

1. **CoreML Integration Complexity**
   - **Risk:** Swift/C++ bridge may be more complex than estimated
   - **Mitigation:** Start with Swift-only approach, add C++ if needed
   - **Contingency:** Extend timeline by 1 week if needed

2. **Settings System Scope**
   - **Risk:** Settings requirements may expand during implementation
   - **Mitigation:** Define clear MVP scope, defer advanced features
   - **Contingency:** Phase advanced features to later sprint

### Medium Risk Items

1. **Dashboard Component Complexity**
   - **Risk:** Agent stats visualization may require complex charts
   - **Mitigation:** Use existing chart libraries, start simple
   - **Contingency:** Use basic tables initially, enhance later

---

## Dependencies

### External Dependencies

- **CoreML Framework:** Apple's CoreML framework availability
- **Swift Toolchain:** Swift compiler for bridge compilation
- **Model Files:** Mistral CoreML model files availability

### Internal Dependencies

- **Database Migrations:** Settings tables must be created before endpoints
- **API Endpoints:** Must exist before dashboard integration
- **Type Definitions:** TypeScript types must match API schemas

---

## Acceptance Criteria

### CoreML Integration Complete When:

- [ ] CoreML models load without errors
- [ ] Inference returns real results (not placeholders)
- [ ] ANE acceleration verified (2.8x speedup measured)
- [ ] Performance benchmarks documented
- [ ] All TODO comments resolved

### Dashboard Integration Complete When:

- [ ] All existing endpoints have UI
- [ ] Agent stats page functional
- [ ] Agent health page functional
- [ ] Telemetry visualizations working
- [ ] Settings pages functional (after endpoints implemented)

### Full Alignment Achieved When:

- [ ] All 6 core requirements at 90%+ alignment
- [ ] CoreML integration functional
- [ ] Dashboard integration at 90%+
- [ ] No critical blockers remaining
- [ ] Documentation accurate and complete

---

**Next Review:** After Priority 1 (CoreML) completion

