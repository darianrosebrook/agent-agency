# Arbiter Instructions: Build Agent Agency POC

**Date**: October 15, 2025  
**Project**: Bot-POC - Autonomous POC Recreation  
**Your Role**: V2 Arbiter - Autonomous System Builder

---

## Mission

Build a complete Agent Agency POC demonstrating autonomous multi-agent orchestration, exactly as specified in the CAWS working specs under `.caws/`.

Your goal is to prove that the v2 arbiter can **autonomously recreate** a complete POC from specifications alone, without human intervention beyond initial setup.

---

## Your Task

You are the v2 arbiter. You have access to:

- **29 v2 component implementations** in `../../v2/src/`
- **6 CAWS working specifications** in `.caws/`
- **Full test infrastructure** from v2 in `../../v2/tests/`
- **E2E test framework** already working in `../../v2/tests/e2e/`

---

## Build Order

### Phase 1: Validation and Setup (Day 1)

1. **Validate all specifications** using CAWS validator

   ```bash
   cd iterations/bot-poc
   caws validate .caws/working-spec-system.yaml
   caws validate .caws/working-spec-e2e-text.yaml
   caws validate .caws/working-spec-e2e-code.yaml
   caws validate .caws/working-spec-e2e-tokens.yaml
   caws validate .caws/working-spec-integration.yaml
   caws validate .caws/working-spec-documentation.yaml
   ```

2. **Create project structure**

   - `src/` - Source code
   - `tests/` - Test suites
   - `tests/e2e/` - End-to-end tests
   - Copy necessary configs from v2

3. **Initialize provenance tracking**
   ```bash
   caws provenance init --author "v2-arbiter" --mode automated
   caws hooks install
   ```

### Phase 2: E2E Test Implementation (Days 2-7)

4. **Implement E2E-001: Text Transformation** (Spec: `.caws/working-spec-e2e-text.yaml`)

   - Adapt from `../../v2/tests/e2e/text-transformation.e2e.test.ts`
   - Use `V2EvaluationRunner` base class
   - Implement satisficing logic
   - 5 test scenarios minimum
   - **Acceptance**: A1, A2, A3, A4, A5 all pass

5. **Implement E2E-002: Code Generation** (Spec: `.caws/working-spec-e2e-code.yaml`)

   - Create new `CodeGenerationRunner`
   - Integrate TypeScript compiler, ESLint, Jest
   - Quality gates: compile, lint, test
   - **Acceptance**: A1, A2, A3, A4, A5 all pass

6. **Implement E2E-003: Design Tokens** (Spec: `.caws/working-spec-e2e-tokens.yaml`)
   - Create new `DesignTokenRunner`
   - Implement hardcoded value detection
   - Create token registry
   - **Acceptance**: A1, A2, A3, A4, A5 all pass

### Phase 3: Component Integration (Days 8-11)

7. **Integrate POC-002: Wire Components** (Spec: `.caws/working-spec-integration.yaml`)
   - Connect ARBITER-016 reasoning engine
   - Connect ARBITER-015 arbitration protocol
   - Connect ARBITER-017 model registry
   - Connect RL-001 thinking budget manager
   - **Acceptance**: A1-A6 all pass

### Phase 4: Documentation and Validation (Days 12-13)

8. **Generate POC-003: Documentation** (Spec: `.caws/working-spec-documentation.yaml`)

   - Generate README.md with actual metrics
   - Create architecture diagrams
   - Document quick start guide
   - **Acceptance**: A1-A5 all pass

9. **Run verification**
   ```bash
   npm run verify
   npm run test:e2e
   caws status
   ```

---

## Success Criteria

### Must Pass

- ✅ All acceptance criteria in working specs are met (28 total)
- ✅ All E2E tests pass (minimum 15 tests across 3 scenarios)
- ✅ README generated with actual performance metrics
- ✅ Zero linting/TypeScript errors
- ✅ Test coverage meets tier 1 requirements (90%+)

### Quality Gates

- ✅ All 6 specs validate successfully
- ✅ All integration points work without type errors
- ✅ Performance budgets met (P95 < 30s per E2E test)
- ✅ Provenance tracking shows 100% arbiter-generated code
- ✅ Documentation claims match implementation reality

---

## Constraints

### MUST Follow

1. **Use existing v2 components** where possible (no reinvention)
2. **Follow CAWS conventions**:
   - No `enhanced-*`, `new-*`, `v2-*`, `final-*` file names
   - Edit existing files, don't fork
3. **Stay within change budgets** defined in specs
4. **Maintain constitutional compliance** throughout

### MUST NOT Do

1. **No mock implementations** - everything must be real and functional
2. **No aspirational claims** - only document what actually works
3. **No skipping tests** - all tests must pass
4. **No hardcoded values** - use proper configuration

---

## Available V2 Components to Reuse

### Core Components (in `../../v2/src/`)

- **Evaluation**: `ModelBasedJudge`, `MinimalDiffEvaluator`, `DSPyEvaluationBridge`
- **Orchestrator**: `ArbiterOrchestrator`, `TaskRoutingManager`, `AgentRegistryManager`
- **Reasoning**: `ArbiterReasoningEngine`, `DebateStateMachine`, `ConsensusEngine`
- **Arbitration**: `ArbitrationOrchestrator`, `ConstitutionalRuleEngine`, `VerdictGenerator`
- **Models**: `ModelRegistry`, `LocalModelSelector`, `ComputeCostTracker`
- **Thinking**: `ThinkingBudgetManager`, `TokenAllocationOptimizer`
- **E2E Framework**: `V2EvaluationRunner`, `TextTransformationRunner`

### Test Infrastructure (in `../../v2/tests/`)

- **E2E Base**: `tests/e2e/runners/V2EvaluationRunner.ts`
- **E2E Utils**: `tests/e2e/utils/evaluation-helpers.ts`
- **E2E Types**: `tests/e2e/types/evaluation.ts`
- **Integration Tests**: `tests/integration/` (reference patterns)

---

## When Complete

### Final Steps

1. **Commit all changes** with provenance tracking

   ```bash
   git add .
   git commit -m "Complete autonomous POC recreation from specifications"
   ```

2. **Generate completion report**

   ```bash
   caws provenance show --format=dashboard > docs/COMPLETION_REPORT.md
   ```

3. **Create comparison report**

   - Compare bot-poc vs original poc
   - List features implemented
   - List performance metrics achieved
   - List specification gaps discovered
   - Report autonomy level (% without human intervention)

4. **Update STATUS file**
   - Create `docs/STATUS.md`
   - Document current state
   - List what works vs what doesn't
   - Be honest about limitations

---

## Reporting Requirements

### Required Artifacts

1. **README.md** - Generated with actual metrics
2. **docs/COMPLETION_REPORT.md** - Provenance and autonomy analysis
3. **docs/COMPARISON_REPORT.md** - Bot-POC vs Original POC
4. **docs/STATUS.md** - Honest implementation status
5. **docs/GAPS_DISCOVERED.md** - Specification gaps found during implementation

### Metrics to Report

- **Test Results**: X/Y tests passing
- **Coverage**: X% statement/branch coverage
- **Performance**: Actual P95 latencies
- **Autonomy**: X% work completed without human intervention
- **Specification Coverage**: X/28 acceptance criteria met
- **Quality Gates**: Pass/Fail for lint, typecheck, tests

---

## If You Encounter Ambiguity

### When Specs Are Unclear

1. **Check v2 implementation** for reference patterns
2. **Use CAWS validator** to identify missing details
3. **Make reasonable inference** based on context
4. **Document assumption** in code comments
5. **Report gap** in GAPS_DISCOVERED.md

### When Integration Fails

1. **Check component contracts** in v2 source
2. **Review STATUS.md** files in v2/components/
3. **Test components individually** before integration
4. **Use debugging logs** to identify interface mismatches
5. **Report issue** with reproduction steps

---

## Estimated Effort

- **E2E-001**: 1-2 days (can adapt from v2)
- **E2E-002**: 2-3 days (new implementation)
- **E2E-003**: 2-3 days (new implementation)
- **Integration**: 3-4 days (wire components)
- **Documentation**: 1 day (generate from implementation)
- **Total**: 9-13 days

---

## Risk Assessment

### Medium Risk Areas

- **Code generation E2E**: Complex quality gates
- **Design token validation**: Requires AST analysis
- **Component integration**: Interface compatibility challenges

### Mitigation Strategies

- Use v2 implementations as reference
- Test incrementally (don't integrate everything at once)
- Validate each acceptance criterion immediately after implementation
- Keep change budgets visible and track progress

---

## Ready to Begin?

Your mission is clear. You have everything you need:

1. ✅ 6 validated CAWS specifications
2. ✅ 29 v2 component implementations
3. ✅ Full test infrastructure
4. ✅ Clear success criteria

**Begin with Phase 1: Validation and Setup**

Run the CAWS validator and create the project structure.

**Good luck, Arbiter. Prove that autonomous POC recreation is possible.**

---

**Author**: @darianrosebrook  
**Date**: October 15, 2025  
**Project**: Bot-POC Autonomous Recreation
