# Agent Agency Reconstruction Plan

## Phase 1: Critical Components (Priority 1)

### ORCHESTRATION: Core orchestration system
- **Files to restore**: 36
- **Total entries**: 240
  - `iterations/v3/orchestration/src/api.rs` (ID: OH6n.rs, Recovery: -34e16b94)
  - `iterations/v3/orchestration/src/arbiter.rs` (ID: kSD0.rs, Recovery: -43104043)
  - `iterations/v3/orchestration/src/artifacts/manager.rs` (ID: 9Nde.rs, Recovery: 4cbe23a7)
  - `iterations/v3/orchestration/src/artifacts/mod.rs` (ID: qWQy.rs, Recovery: 59995ab2)
  - `iterations/v3/orchestration/src/artifacts/storage.rs` (ID: Z72U.rs, Recovery: -1770a9e7)
  - `iterations/v3/orchestration/src/artifacts/versioning.rs` (ID: 7Bbj.rs, Recovery: 6d01fd00)
  - `iterations/v3/orchestration/src/audit_trail.rs` (ID: AStO.rs, Recovery: 5c3df9e4)
  - `iterations/v3/orchestration/src/audited_orchestrator.rs` (ID: c6TR.rs, Recovery: -24987357)
  - `iterations/v3/orchestration/src/autonomous_executor.rs` (ID: PDrG.rs, Recovery: 4200b6c4)
  - `iterations/v3/orchestration/src/caws_runtime.rs` (ID: Ma9t.rs, Recovery: 73b3dc5)
  - `iterations/v3/orchestration/src/frontier.rs` (ID: bfyo.rs, Recovery: 5e99544b)
  - `iterations/v3/orchestration/src/main.rs` (ID: Zb9b.rs, Recovery: -5fdd861)
  - `iterations/v3/orchestration/src/multimodal_orchestration.rs` (ID: l0gU.rs, Recovery: -1390bc9e)
  - `iterations/v3/orchestration/src/planning/acceptance_criteria_extractor.rs` (ID: VeU8.rs, Recovery: d89027e)
  - `iterations/v3/orchestration/src/planning/agent.rs` (ID: WINB.rs, Recovery: 51e85205)
  - `iterations/v3/orchestration/src/planning/clarification.rs` (ID: obOY.rs, Recovery: -1d880b9a)
  - `iterations/v3/orchestration/src/planning/context_builder.rs` (ID: 38sb.rs, Recovery: 5908947f)
  - `iterations/v3/orchestration/src/planning/integration_test.rs` (ID: wx5I.rs, Recovery: 1e428697)
  - `iterations/v3/orchestration/src/planning/llm_client.rs` (ID: 6jdV.rs, Recovery: -4bc63ee9)
  - `iterations/v3/orchestration/src/planning/mod.rs` (ID: 8kBU.rs, Recovery: -57281c58)
  - `iterations/v3/orchestration/src/planning/spec_generator.rs` (ID: TXeT.rs, Recovery: -122afc9b)
  - `iterations/v3/orchestration/src/planning/tests.rs` (ID: 5cak.rs, Recovery: -49f5de97)
  - `iterations/v3/orchestration/src/planning/types.rs` (ID: 44Mv.rs, Recovery: -2deb27ef)
  - `iterations/v3/orchestration/src/planning/validation_loop.rs` (ID: kkD0.rs, Recovery: -39523180)
  - `iterations/v3/orchestration/src/quality/gates.rs` (ID: 425a.rs, Recovery: 361de10e)
  - `iterations/v3/orchestration/src/quality/mod.rs` (ID: H1dJ.rs, Recovery: 10186a94)
  - `iterations/v3/orchestration/src/quality/orchestrator.rs` (ID: mxHk.rs, Recovery: 106b6c1e)
  - `iterations/v3/orchestration/src/quality/satisficing.rs` (ID: 7yez.rs, Recovery: -37eae2dc)
  - `iterations/v3/orchestration/src/refinement/coordinator.rs` (ID: 4JeK.rs, Recovery: -6f974c34)
  - `iterations/v3/orchestration/src/refinement/feedback_loop.rs` (ID: IsK0.rs, Recovery: -33620e22)
  - `iterations/v3/orchestration/src/refinement/mod.rs` (ID: tKqw.rs, Recovery: 31d33bba)
  - `iterations/v3/orchestration/src/refinement/strategy.rs` (ID: spaZ.rs, Recovery: 4d903faf)
  - `iterations/v3/orchestration/src/tracking/event_bus.rs` (ID: jPGo.rs, Recovery: -451ba0ab)
  - `iterations/v3/orchestration/src/tracking/mod.rs` (ID: hm45.rs, Recovery: 4828130e)
  - `iterations/v3/orchestration/src/tracking/progress_tracker.rs` (ID: RitF.rs, Recovery: 36480d48)
  - `iterations/v3/orchestration/src/tracking/websocket.rs` (ID: bWQ4.rs, Recovery: 45599f69)

### SELF-PROMPTING-AGENT: Autonomous agent system
- **Files to restore**: 44
- **Total entries**: 373
  - `iterations/v3/self-prompting-agent/Cargo.toml` (ID: QgkS.toml, Recovery: 3b72e248)
  - `iterations/v3/self-prompting-agent/EDGE_CASE_ANALYSIS.md` (ID: RjPQ.md, Recovery: -627965c0)
  - `iterations/v3/self-prompting-agent/examples/playground_test.rs` (ID: GZxW.rs, Recovery: 6092d3c3)
  - `iterations/v3/self-prompting-agent/src/agent.rs` (ID: fN3t.rs, Recovery: -41354f8f)
  - `iterations/v3/self-prompting-agent/src/caws/budget_checker.rs` (ID: PN7F.rs, Recovery: 81d93a6)
  - `iterations/v3/self-prompting-agent/src/caws/council_approval.rs` (ID: doRY.rs, Recovery: -54ad1000)
  - `iterations/v3/self-prompting-agent/src/caws/mod.rs` (ID: LMgg.rs, Recovery: -4d4beb65)
  - `iterations/v3/self-prompting-agent/src/caws/waiver_generator.rs` (ID: 3lna.rs, Recovery: 2b7fb609)
  - `iterations/v3/self-prompting-agent/src/evaluation/caws_evaluator.rs` (ID: vl9z.rs, Recovery: -9b9d627)
  - `iterations/v3/self-prompting-agent/src/evaluation/code_evaluator.rs` (ID: TQnP.rs, Recovery: 46d66286)
  - `iterations/v3/self-prompting-agent/src/evaluation/flakiness.rs` (ID: 3tzt.rs, Recovery: 7f8dea8b)
  - `iterations/v3/self-prompting-agent/src/evaluation/mod.rs` (ID: 80Rj.rs, Recovery: 5e7f5419)
  - `iterations/v3/self-prompting-agent/src/evaluation/satisficing.rs` (ID: Uk3j.rs, Recovery: -5eef4e57)
  - `iterations/v3/self-prompting-agent/src/evaluation/text_evaluator.rs` (ID: tbPS.rs, Recovery: 138e9346)
  - `iterations/v3/self-prompting-agent/src/evaluation/token_evaluator.rs` (ID: 0a7a.rs, Recovery: -570c6dae)
  - `iterations/v3/self-prompting-agent/src/integration.rs` (ID: wGAt.rs, Recovery: 70697e02)
  - `iterations/v3/self-prompting-agent/src/learning_bridge.rs` (ID: YMG3.rs, Recovery: -6d9ff334)
  - `iterations/v3/self-prompting-agent/src/lib.rs` (ID: DI24.rs, Recovery: 374c9eb1)
  - `iterations/v3/self-prompting-agent/src/loop_controller.rs` (ID: q1Dy.rs, Recovery: 6514795f)
  - `iterations/v3/self-prompting-agent/src/minimal_test.rs` (ID: 6XjH.rs, Recovery: 75c08bf8)
  - `iterations/v3/self-prompting-agent/src/models/coreml.rs` (ID: jFQd.rs, Recovery: -132c010d)
  - `iterations/v3/self-prompting-agent/src/models/coreml_provider.rs` (ID: kNpJ.rs, Recovery: 79bb821b)
  - `iterations/v3/self-prompting-agent/src/models/mod.rs` (ID: 9A7g.rs, Recovery: -3937da55)
  - `iterations/v3/self-prompting-agent/src/models/ollama.rs` (ID: CTNG.rs, Recovery: 6427e64b)
  - `iterations/v3/self-prompting-agent/src/models/selection.rs` (ID: obw8.rs, Recovery: -20735f9f)
  - `iterations/v3/self-prompting-agent/src/policy_hooks.rs` (ID: VNBn.rs, Recovery: 2f042525)
  - `iterations/v3/self-prompting-agent/src/profiling.rs` (ID: aF6U.rs, Recovery: bc54830)
  - `iterations/v3/self-prompting-agent/src/prompting/adaptive.rs` (ID: cqD0.rs, Recovery: -608adf1f)
  - `iterations/v3/self-prompting-agent/src/prompting/frame.rs` (ID: 3aRI.rs, Recovery: 5287d73a)
  - `iterations/v3/self-prompting-agent/src/prompting/mod.rs` (ID: QsIu.rs, Recovery: 296fcb85)
  - `iterations/v3/self-prompting-agent/src/prompting/tool_schema.rs` (ID: HPHL.rs, Recovery: 61c7b29f)
  - `iterations/v3/self-prompting-agent/src/rl_signals.rs` (ID: 9OWj.rs, Recovery: 207e9922)
  - `iterations/v3/self-prompting-agent/src/sandbox/diff_applier.rs` (ID: qSvE.rs, Recovery: 5178762f)
  - `iterations/v3/self-prompting-agent/src/sandbox/diff_generator.rs` (ID: OzIX.rs, Recovery: -7e99b7d9)
  - `iterations/v3/self-prompting-agent/src/sandbox/file_guard.rs` (ID: 5pS7.rs, Recovery: b4554fe)
  - `iterations/v3/self-prompting-agent/src/sandbox/git_worktree.rs` (ID: 68F5.rs, Recovery: 5d7908a4)
  - `iterations/v3/self-prompting-agent/src/sandbox/mod.rs` (ID: wpn1.rs, Recovery: -4e325a4)
  - `iterations/v3/self-prompting-agent/src/sandbox/snapshot.rs` (ID: 1xyw.rs, Recovery: -f6129a4)
  - `iterations/v3/self-prompting-agent/src/sandbox/workspace_manager.rs` (ID: A1FI.rs, Recovery: 699397b)
  - `iterations/v3/self-prompting-agent/src/stubs.rs` (ID: vt82.rs, Recovery: -290b2a0f)
  - `iterations/v3/self-prompting-agent/src/types.rs` (ID: 6zRD.rs, Recovery: 3ef7367d)
  - `iterations/v3/self-prompting-agent/tests/autonomous_agent_integration_tests.rs` (ID: C6pF.rs, Recovery: -64db3568)
  - `iterations/v3/self-prompting-agent/tests/brittleness_integration_tests.rs` (ID: NNXP.rs, Recovery: 7822a8d9)
  - `iterations/v3/self-prompting-agent/tests/integration_tests.rs` (ID: I2nj.rs, Recovery: 56f9e843)

### WORKERS: Worker execution system
- **Files to restore**: 2
- **Total entries**: 35
  - `iterations/v3/workers/src/autonomous_executor.rs` (ID: 4flW.rs, Recovery: 79fa2250)
  - `iterations/v3/workers/src/multimodal_scheduler.rs` (ID: 1KTH.rs, Recovery: 14f5929c)

### COUNCIL: Constitutional AI system
- **Files to restore**: 12
- **Total entries**: 115
  - `iterations/v3/council/build.rs` (ID: GPfh.rs, Recovery: -78d39d3)
  - `iterations/v3/council/src/claim_extraction_multimodal.rs` (ID: Ez9f.rs, Recovery: -11b3ab63)
  - `iterations/v3/council/src/council.rs` (ID: 6ZUd.rs, Recovery: -2c141edb)
  - `iterations/v3/council/src/decision_making.rs` (ID: Squb.rs, Recovery: 51caf4b8)
  - `iterations/v3/council/src/error.rs` (ID: drMJ.rs, Recovery: -7cca35e2)
  - `iterations/v3/council/src/error_handling.rs` (ID: 5b1G.rs, Recovery: -54e6d824)
  - `iterations/v3/council/src/judge.rs` (ID: nHXR.rs, Recovery: 10f3deaf)
  - `iterations/v3/council/src/plan_review.rs` (ID: r3ZE.rs, Recovery: -43197c08)
  - `iterations/v3/council/src/plan_review_integration_test.rs` (ID: Flrb.rs, Recovery: 528e350a)
  - `iterations/v3/council/src/risk_scorer.rs` (ID: 4xJG.rs, Recovery: -1d87a5aa)
  - `iterations/v3/council/src/verdict_aggregation.rs` (ID: 6HXI.rs, Recovery: -e740616)
  - `iterations/v3/council/src/workflow.rs` (ID: YBj0.rs, Recovery: -4e6e6c7)

### EMBEDDING-SERVICE: Embedding and tokenization
- **Files to restore**: 2
- **Total entries**: 20
  - `iterations/v3/embedding-service/src/model_loading.rs` (ID: A9SK.rs, Recovery: 1bbf33e2)
  - `iterations/v3/embedding-service/src/tokenization.rs` (ID: R2G9.rs, Recovery: 138206cb)

### APPLE-SILICON: Apple Silicon integration
- **Files to restore**: 22
- **Total entries**: 106
  - `iterations/v3/apple-silicon/.caws/working-spec.yaml` (ID: 7JgW.yaml, Recovery: 12333fc6)
  - `iterations/v3/apple-silicon/BLOCKED_FRAMEWORK_INTEGRATIONS.md` (ID: Z99H.md, Recovery: 5781cc1)
  - `iterations/v3/apple-silicon/build.rs` (ID: x6tq.rs, Recovery: -4777a22e)
  - `iterations/v3/apple-silicon/src/ane/ffi.rs` (ID: ByXP.rs, Recovery: 5af89b19)
  - `iterations/v3/apple-silicon/src/ane/filesystem.rs` (ID: KchQ.rs, Recovery: 3d668c91)
  - `iterations/v3/apple-silicon/src/ane/manager.rs` (ID: YfmO.rs, Recovery: -10d308eb)
  - `iterations/v3/apple-silicon/src/ane/mod.rs` (ID: 8HmH.rs, Recovery: 67671520)
  - `iterations/v3/apple-silicon/src/buffer_pool.rs` (ID: QGOY.rs, Recovery: -5ccf6150)
  - `iterations/v3/apple-silicon/src/enhanced_telemetry.rs` (ID: Kh0B.rs, Recovery: -7254ef1d)
  - `iterations/v3/apple-silicon/src/memory/analysis.rs` (ID: m0FI.rs, Recovery: 46a822e1)
  - `iterations/v3/apple-silicon/src/memory/compression.rs` (ID: HKpw.rs, Recovery: 73e5bcbb)
  - `iterations/v3/apple-silicon/src/memory/manager.rs` (ID: qxeq.rs, Recovery: 7cbbd614)
  - `iterations/v3/apple-silicon/src/memory/metrics.rs` (ID: LJeu.rs, Recovery: -3c537ac2)
  - `iterations/v3/apple-silicon/src/memory/mod.rs` (ID: KclL.rs, Recovery: 23034b9f)
  - `iterations/v3/apple-silicon/src/memory/quantization.rs` (ID: yqoZ.rs, Recovery: -8b88124)
  - `iterations/v3/apple-silicon/src/model_router.rs` (ID: 8d0r.rs, Recovery: 3f0fce94)
  - `iterations/v3/apple-silicon/src/operator_fusion.rs` (ID: gW33.rs, Recovery: 5d3afeec)
  - `iterations/v3/apple-silicon/src/quantization_lab.rs` (ID: gW1M.rs, Recovery: 22dbbda4)
  - `iterations/v3/apple-silicon/src/speech_bridge.rs` (ID: MojQ.rs, Recovery: -9d310fb)
  - `iterations/v3/apple-silicon/src/tokenization.rs` (ID: nWl5.rs, Recovery: -6500b818)
  - `iterations/v3/apple-silicon/src/vision_bridge.rs` (ID: zm47.rs, Recovery: 13bfb28b)
  - `iterations/v3/apple-silicon/tests/phase3b_inference_cycles.rs` (ID: wTR3.rs, Recovery: -5e96d178)

### CAWS-CONFIG: CAWS configuration
- **Files to restore**: 5
- **Total entries**: 5
  - `.caws/EXEC_SUMMARY.md` (ID: O9Rr.md, Recovery: 13c18ee1)
  - `.caws/FAST_WINS_ROADMAP.md` (ID: USYf.md, Recovery: 24b4e8fa)
  - `.caws/P0_CODE_MAP.md` (ID: BvBZ.md, Recovery: 25fe648)
  - `.caws/P0_DAILY_CHECKLIST.md` (ID: e5ZW.md, Recovery: -4b322548)
  - `.caws/README.md` (ID: cjFi.md, Recovery: 19ad3a4b)

### DOCUMENTATION: Documentation updates
- **Files to restore**: 16
- **Total entries**: 36
  - `docs-status/README.md` (ID: FXSR.md, Recovery: -727e7e19)
  - `docs/P0-IMPLEMENTATION-ROADMAP.md` (ID: Opij.md, Recovery: -750b069a)
  - `docs/PLACEHOLDER-DETECTION-GUIDE.md` (ID: xdOf.md, Recovery: -2b27c946)
  - `docs/SYSTEM_HARDENING_ANALYSIS.md` (ID: yHRn.md, Recovery: -51898f09)
  - `docs/agents.md` (ID: yoRB.md, Recovery: 44b8ee68)
  - `docs/agents/examples.md` (ID: xfmw.md, Recovery: -2ed692f6)
  - `docs/agents/full-guide.md` (ID: Wpv9.md, Recovery: -1f8c748b)
  - `docs/agents/tutorial.md` (ID: BDlY.md, Recovery: -1d03762b)
  - `docs/database/README.md` (ID: sERb.md, Recovery: -4963d78e)
  - `docs/monitoring-alerting.md` (ID: kpTc.md, Recovery: 3e28b829)
  - `iterations/v2/docs/3-agent-rl-training/comprehensive-improvement-summary.md` (ID: 4tDQ.md, Recovery: 27ae8948)
  - `iterations/v2/docs/3-agent-rl-training/implementation-roadmap.md` (ID: GLpN.md, Recovery: 77cbb5ab)
  - `iterations/v2/docs/3-agent-rl-training/technical-architecture.md` (ID: Pb2B.md, Recovery: 65cfbbc9)
  - `iterations/v2/docs/3-agent-rl-training/v2-agentic-rl-roadmap.md` (ID: 7RVv.md, Recovery: -58d8e0a)
  - `iterations/v2/docs/feature.plan.md` (ID: TBYh.md, Recovery: -3a5fb931)
  - `iterations/v2/docs/templates/COMPONENT_STATUS_TEMPLATE.md` (ID: AMTT.md, Recovery: -1d687735)

### V3-OTHER: Other V3 components
- **Files to restore**: 475
- **Total entries**: 2397
  - `iterations/v3/.github/workflows/ci.yml` (ID: JD1l.yml, Recovery: 6a8acfa7)
  - `iterations/v3/.github/workflows/test.yml` (ID: kNqR.yml, Recovery: 7acc1813)
  - `iterations/v3/DEPLOYMENT_GUIDE.md` (ID: FzFW.md, Recovery: 37b790a5)
  - `iterations/v3/FINAL_PROJECT_SUMMARY.md` (ID: bQJr.md, Recovery: -51ec31b4)
  - `iterations/v3/KNOWLEDGE_BASE_IMPLEMENTATION_SUMMARY.md` (ID: kaCn.md, Recovery: 27f7ca7d)
  - `iterations/v3/KNOWLEDGE_BASE_TESTING_SUMMARY.md` (ID: Shu0.md, Recovery: 79e3d4fd)
  - `iterations/v3/MULTIMODAL_RAG_INTEGRATION_SPEC.md` (ID: k3Ln.md, Recovery: 315aa96b)
  - `iterations/v3/README_V3_IMPLEMENTATION.md` (ID: 7PiY.md, Recovery: 41223ea4)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_10_COMPLETE.md` (ID: ozwv.md, Recovery: 56d6a667)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_11_COMPLETE.md` (ID: HCV1.md, Recovery: 4559ace8)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_12_COMPLETE.md` (ID: 39wo.md, Recovery: 33dcb369)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_13_COMPLETE.md` (ID: hO6h.md, Recovery: 225fb9ea)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_14_COMPLETE.md` (ID: Ujsd.md, Recovery: 10e2c06b)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_2_COMPLETE.md` (ID: rUUH.md, Recovery: -528abf9c)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_3_COMPLETE.md` (ID: K9Xw.md, Recovery: -6407b91b)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_4_COMPLETE.md` (ID: aIFd.md, Recovery: -7584b29a)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_5_COMPLETE.md` (ID: wUAJ.md, Recovery: 78fe53e7)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_6_COMPLETE.md` (ID: O4Qz.md, Recovery: 67815a68)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_7_COMPLETE.md` (ID: T6KF.md, Recovery: 560460e9)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_8_COMPLETE.md` (ID: 4SCn.md, Recovery: 4487676a)
  - `iterations/v3/TODO_IMPLEMENTATION_SESSION_9_COMPLETE.md` (ID: QbHm.md, Recovery: 330a6deb)
  - `iterations/v3/agent-agency-contracts/Cargo.toml` (ID: uMaL.toml, Recovery: 3a20569c)
  - `iterations/v3/agent-agency-contracts/src/error.rs` (ID: lv7c.rs, Recovery: -3c6e321e)
  - `iterations/v3/agent-agency-contracts/src/execution_artifacts.rs` (ID: pvko.rs, Recovery: c7de2f0)
  - `iterations/v3/agent-agency-contracts/src/lib.rs` (ID: FZ6X.rs, Recovery: 35fa1305)
  - `iterations/v3/agent-agency-contracts/src/quality_report.rs` (ID: 04hm.rs, Recovery: -6918c5a0)
  - `iterations/v3/agent-agency-contracts/src/refinement_decision.rs` (ID: 3v2C.rs, Recovery: 220cab5a)
  - `iterations/v3/agent-agency-contracts/src/schema.rs` (ID: EiBx.rs, Recovery: -6e0aaced)
  - `iterations/v3/agent-agency-contracts/src/task_request.rs` (ID: gu8P.rs, Recovery: -3fb22d61)
  - `iterations/v3/agent-agency-contracts/src/task_response.rs` (ID: AwrH.rs, Recovery: 730f7e4f)
  - `iterations/v3/agent-agency-contracts/src/working_spec.rs` (ID: ABjx.rs, Recovery: -43d92c15)
  - `iterations/v3/api-server-config.toml` (ID: pBHv.toml, Recovery: 44be1cd)
  - `iterations/v3/api-server/Cargo.toml` (ID: 8SiF.toml, Recovery: -658c5c39)
  - `iterations/v3/api-server/src/alerts.rs` (ID: 4uNG.rs, Recovery: -3e27a6ae)
  - `iterations/v3/api-server/src/circuit_breaker.rs` (ID: sk4q.rs, Recovery: -6feadb81)
  - `iterations/v3/api-server/src/main.rs` (ID: aWbq.rs, Recovery: 5b1cd970)
  - `iterations/v3/api-server/src/rate_limiter.rs` (ID: pgYn.rs, Recovery: -30ca6e20)
  - `iterations/v3/api-server/src/rto_rpo_monitor.rs` (ID: Pxg8.rs, Recovery: 59adc75b)
  - `iterations/v3/api-server/src/service_failover.rs` (ID: nEln.rs, Recovery: 500b5a0d)
  - `iterations/v3/apps/tools/caws/__tests__/security-provenance.test.ts` (ID: bxwq.ts, Recovery: -233a36d5)
  - `iterations/v3/apps/tools/caws/ci-optimizer.js` (ID: juh5.js, Recovery: -5a438b9)
  - `iterations/v3/apps/tools/caws/dashboard.js.backup` (ID: cRty.backup, Recovery: 92180cb)
  - `iterations/v3/apps/tools/caws/legacy-assessment.ts` (ID: QSxf.ts, Recovery: -13ac77b7)
  - `iterations/v3/apps/tools/caws/modules/README.md` (ID: Yo1B.md, Recovery: -3b642993)
  - `iterations/v3/apps/tools/caws/modules/compliance-checker.js` (ID: Ofx5.js, Recovery: 30433eaa)
  - `iterations/v3/apps/tools/caws/modules/coverage-analysis.js` (ID: 2XRF.js, Recovery: -596a5520)
  - `iterations/v3/apps/tools/caws/modules/data-generator.js` (ID: Ig49.js, Recovery: -70e41e3)
  - `iterations/v3/apps/tools/caws/modules/index.js` (ID: gMV8.js, Recovery: -594ad371)
  - `iterations/v3/apps/tools/caws/modules/mutation-analysis.js` (ID: 3eMR.js, Recovery: -435c51df)
  - `iterations/v3/apps/tools/caws/modules/test-analysis.js` (ID: so4y.js, Recovery: -1fddd076)
  - `iterations/v3/apps/tools/caws/workflow-modules/README.md` (ID: IDZN.md, Recovery: -73d2049d)
  - `iterations/v3/apps/tools/caws/workflow-modules/build-jobs.js` (ID: 11rM.js, Recovery: 4b62c10e)
  - `iterations/v3/apps/tools/caws/workflow-modules/index.js` (ID: WqVr.js, Recovery: -29906f27)
  - `iterations/v3/apps/tools/caws/workflow-modules/quality-jobs.js` (ID: wd64.js, Recovery: 182e79f)
  - `iterations/v3/apps/tools/caws/workflow-modules/workflow-base.js` (ID: b1nN.js, Recovery: -70ee2994)
  - `iterations/v3/apps/web-dashboard/.eslintrc.json` (ID: 8ZsE.json, Recovery: -318d7b07)
  - `iterations/v3/apps/web-dashboard/.gitignore` (ID: tm5q, Recovery: -334d2817)
  - `iterations/v3/apps/web-dashboard/.prettierrc` (ID: N2nC, Recovery: -23c40e21)
  - `iterations/v3/apps/web-dashboard/ENVIRONMENT_SETUP.md` (ID: 56Yo.md, Recovery: -5fd7dbf9)
  - `iterations/v3/apps/web-dashboard/README.md` (ID: dHDP.md, Recovery: 5bccf744)
  - `iterations/v3/apps/web-dashboard/cypress.config.ts` (ID: Cays.ts, Recovery: 2235a5f7)
  - `iterations/v3/apps/web-dashboard/cypress/e2e/self-prompting.cy.ts` (ID: 4b3s.ts, Recovery: 40952d2)
  - `iterations/v3/apps/web-dashboard/env.example` (ID: N0fb.example, Recovery: -229aa86a)
  - `iterations/v3/apps/web-dashboard/jest.config.js` (ID: ZaTB.js, Recovery: -18edb46a)
  - `iterations/v3/apps/web-dashboard/jest.setup.js` (ID: 4qgq.js, Recovery: -782269e7)
  - `iterations/v3/apps/web-dashboard/next.config.js` (ID: 0LuB.js, Recovery: -6afe5313)
  - `iterations/v3/apps/web-dashboard/package.json` (ID: wg4H.json, Recovery: -584ff6fd)
  - `iterations/v3/apps/web-dashboard/postcss.config.js` (ID: r0Kj.js, Recovery: 5f6b0517)
  - `iterations/v3/apps/web-dashboard/src/app/api/alerts/%5BalertId%5D/acknowledge/route.ts` (ID: c9Km.ts, Recovery: 5457528e)
  - `iterations/v3/apps/web-dashboard/src/app/api/alerts/%5BalertId%5D/resolve/route.ts` (ID: ZWv4.ts, Recovery: 15c9a0be)
  - `iterations/v3/apps/web-dashboard/src/app/api/alerts/route.ts` (ID: k5ve.ts, Recovery: a2d6adb)
  - `iterations/v3/apps/web-dashboard/src/app/api/alerts/statistics/route.ts` (ID: 89QX.ts, Recovery: -3806e747)
  - `iterations/v3/apps/web-dashboard/src/app/api/analytics/route.ts` (ID: ogRv.ts, Recovery: -47538dd2)
  - `iterations/v3/apps/web-dashboard/src/app/api/chat/ws/%5BsessionId%5D/route.ts` (ID: yk42.ts, Recovery: -6fe7c1c5)
  - `iterations/v3/apps/web-dashboard/src/app/api/database/connections/route.ts` (ID: jWP0.ts, Recovery: 69128fb1)
  - `iterations/v3/apps/web-dashboard/src/app/api/database/query/route.ts` (ID: Cupn.ts, Recovery: -43464822)
  - `iterations/v3/apps/web-dashboard/src/app/api/database/tables/%5BtableName%5D/schema/route.ts` (ID: n6Ui.ts, Recovery: -47899d87)
  - `iterations/v3/apps/web-dashboard/src/app/api/database/tables/route.ts` (ID: 1XJv.ts, Recovery: -465dd325)
  - `iterations/v3/apps/web-dashboard/src/app/api/database/vector-search/route.ts` (ID: 5brh.ts, Recovery: -501c008c)
  - `iterations/v3/apps/web-dashboard/src/app/api/health/route.ts` (ID: nlbP.ts, Recovery: -1541074a)
  - `iterations/v3/apps/web-dashboard/src/app/api/metrics/route.ts` (ID: jczs.ts, Recovery: 4df4dd1)
  - `iterations/v3/apps/web-dashboard/src/app/api/metrics/stream/route.ts` (ID: p4f9.ts, Recovery: -35f2c43a)
  - `iterations/v3/apps/web-dashboard/src/app/api/proxy/%5B...path%5D/route.ts` (ID: KLjx.ts, Recovery: -3c22c85e)
  - `iterations/v3/apps/web-dashboard/src/app/api/slo-alerts/%5BalertId%5D/acknowledge/route.ts` (ID: Kqqb.ts, Recovery: -57d944c9)
  - `iterations/v3/apps/web-dashboard/src/app/api/slo-alerts/route.ts` (ID: bD8v.ts, Recovery: 1555d4c4)
  - `iterations/v3/apps/web-dashboard/src/app/api/slos/%5BsloName%5D/measurements/route.ts` (ID: zQ66.ts, Recovery: -73caf9f)
  - `iterations/v3/apps/web-dashboard/src/app/api/slos/%5BsloName%5D/status/route.ts` (ID: zteR.ts, Recovery: -3118493a)
  - `iterations/v3/apps/web-dashboard/src/app/api/slos/route.ts` (ID: nXIj.ts, Recovery: 65811f5)
  - `iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D/action/route.ts` (ID: EQM4.ts, Recovery: 9433912)
  - `iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D/arbiter-verdict/route.ts` (ID: xU6w.ts, Recovery: 77daaa9)
  - `iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D/claim-verification/route.ts` (ID: LpKU.ts, Recovery: -22efa384)
  - `iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D/route.ts` (ID: 1i0l.ts, Recovery: 411914cf)
  - `iterations/v3/apps/web-dashboard/src/app/api/tasks/events/route.ts` (ID: tdEK.ts, Recovery: 392766d8)
  - `iterations/v3/apps/web-dashboard/src/app/api/tasks/route.ts` (ID: GHQB.ts, Recovery: -f00fc5a)
  - `iterations/v3/apps/web-dashboard/src/app/api/tts/route.ts` (ID: wQp9.ts, Recovery: -1d4e54df)
  - `iterations/v3/apps/web-dashboard/src/app/globals.scss` (ID: 0UoV.scss, Recovery: -4890fae6)
  - `iterations/v3/apps/web-dashboard/src/app/layout.tsx` (ID: 6KYI.tsx, Recovery: 4dbfe281)
  - `iterations/v3/apps/web-dashboard/src/app/page.module.scss` (ID: bcnS.scss, Recovery: -6fb4a581)
  - `iterations/v3/apps/web-dashboard/src/app/page.tsx` (ID: 647Q.tsx, Recovery: 7cbe7a66)
  - `iterations/v3/apps/web-dashboard/src/app/tasks/%5BtaskId%5D/page.module.scss` (ID: mRPW.scss, Recovery: -536e88ed)
  - `iterations/v3/apps/web-dashboard/src/app/tasks/%5BtaskId%5D/page.tsx` (ID: JO1A.tsx, Recovery: 2e646afa)
  - `iterations/v3/apps/web-dashboard/src/app/tasks/page.module.scss` (ID: UtH4.scss, Recovery: 16046de0)
  - `iterations/v3/apps/web-dashboard/src/app/tasks/page.tsx` (ID: sukC.tsx, Recovery: 41cd7ec7)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/AnalyticsDashboard.module.scss` (ID: tJ7j.scss, Recovery: 7d130356)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/AnalyticsDashboard.tsx` (ID: 1D7Z.tsx, Recovery: -4b1415c3)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/AnomalyDetector.module.scss` (ID: CLQn.scss, Recovery: -6cded1b7)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/AnomalyDetector.tsx` (ID: Ysop.tsx, Recovery: 4621b830)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/CorrelationMatrix.module.scss` (ID: aPDF.scss, Recovery: 7823e843)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/CorrelationMatrix.tsx` (ID: zOqZ.tsx, Recovery: -eacf3d6)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/ForecastingChart.module.scss` (ID: nWE5.scss, Recovery: -5948f2c1)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/ForecastingChart.tsx` (ID: PqlQ.tsx, Recovery: 3068ed26)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/PerformancePredictor.module.scss` (ID: Cb7S.scss, Recovery: 7b7acdd4)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/PerformancePredictor.tsx` (ID: 2xOS.tsx, Recovery: -6b2ed45)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/TrendAnalyzer.module.scss` (ID: sMTo.scss, Recovery: -58cc343d)
  - `iterations/v3/apps/web-dashboard/src/components/analytics/TrendAnalyzer.tsx` (ID: UMDv.tsx, Recovery: -5dc19056)
  - `iterations/v3/apps/web-dashboard/src/components/arbiter/ArbiterVerdictPanel.module.scss` (ID: gNKT.scss, Recovery: 1129cdb7)
  - `iterations/v3/apps/web-dashboard/src/components/arbiter/ArbiterVerdictPanel.tsx` (ID: Un55.tsx, Recovery: -1f005a62)
  - `iterations/v3/apps/web-dashboard/src/components/arbiter/ClaimVerificationPanel.module.scss` (ID: MrpO.scss, Recovery: 6997c772)
  - `iterations/v3/apps/web-dashboard/src/components/arbiter/ClaimVerificationPanel.tsx` (ID: QL6z.tsx, Recovery: -336b75a7)
  - `iterations/v3/apps/web-dashboard/src/components/arbiter/CliInterventionPanel.tsx` (ID: 57z8.tsx, Recovery: 224e06f9)
  - `iterations/v3/apps/web-dashboard/src/components/arbiter/DebateVisualization.tsx` (ID: UioT.tsx, Recovery: 4223849d)
  - `iterations/v3/apps/web-dashboard/src/components/arbiter/index.ts` (ID: 6IB3.ts, Recovery: -1e174aa4)
  - `iterations/v3/apps/web-dashboard/src/components/chat/ChatInterface.module.scss` (ID: wyhr.scss, Recovery: 22b6efc9)
  - `iterations/v3/apps/web-dashboard/src/components/chat/ChatInterface.tsx` (ID: 6alP.tsx, Recovery: -daf0650)
  - `iterations/v3/apps/web-dashboard/src/components/chat/ConnectionStatus.module.scss` (ID: 4Und.scss, Recovery: 6e43d570)
  - `iterations/v3/apps/web-dashboard/src/components/chat/ConnectionStatus.tsx` (ID: aB5P.tsx, Recovery: -3c8a89a9)
  - `iterations/v3/apps/web-dashboard/src/components/chat/ContextPanel.module.scss` (ID: cFFN.scss, Recovery: -11cbdeeb)
  - `iterations/v3/apps/web-dashboard/src/components/chat/ContextPanel.tsx` (ID: zDxe.tsx, Recovery: 340a36fc)
  - `iterations/v3/apps/web-dashboard/src/components/chat/MessageBubble.module.scss` (ID: 6azl.scss, Recovery: -54198885)
  - `iterations/v3/apps/web-dashboard/src/components/chat/MessageBubble.tsx` (ID: PWGu.tsx, Recovery: 4c545362)
  - `iterations/v3/apps/web-dashboard/src/components/chat/MessageInput.module.scss` (ID: ixcX.scss, Recovery: 2e622683)
  - `iterations/v3/apps/web-dashboard/src/components/chat/MessageInput.tsx` (ID: qAO2.tsx, Recovery: 67d38a6a)
  - `iterations/v3/apps/web-dashboard/src/components/chat/MessageList.module.scss` (ID: hxAG.scss, Recovery: -6e6b58b3)
  - `iterations/v3/apps/web-dashboard/src/components/chat/MessageList.tsx` (ID: qlMx.tsx, Recovery: -446f8acc)
  - `iterations/v3/apps/web-dashboard/src/components/chat/VoiceChatInterface.module.scss` (ID: QtJF.scss, Recovery: -464befd1)
  - `iterations/v3/apps/web-dashboard/src/components/chat/VoiceChatInterface.tsx` (ID: ovAY.tsx, Recovery: -27b01fea)
  - `iterations/v3/apps/web-dashboard/src/components/database/DataQualityDashboard.module.scss` (ID: 5lsE.scss, Recovery: 74f26c5c)
  - `iterations/v3/apps/web-dashboard/src/components/database/DataQualityDashboard.tsx` (ID: Lv6O.tsx, Recovery: 106fb943)
  - `iterations/v3/apps/web-dashboard/src/components/database/DatabaseExplorer.module.scss` (ID: hCZN.scss, Recovery: 4b56fb17)
  - `iterations/v3/apps/web-dashboard/src/components/database/DatabaseExplorer.tsx` (ID: cbRs.tsx, Recovery: 306032fe)
  - `iterations/v3/apps/web-dashboard/src/components/database/QueryBuilder.module.scss` (ID: Ncou.scss, Recovery: -1765a310)
  - `iterations/v3/apps/web-dashboard/src/components/database/QueryBuilder.tsx` (ID: 5itu.tsx, Recovery: -4d748229)
  - `iterations/v3/apps/web-dashboard/src/components/database/TableViewer.module.scss` (ID: 5j4O.scss, Recovery: 27ad04ab)
  - `iterations/v3/apps/web-dashboard/src/components/database/TableViewer.tsx` (ID: r5Wp.tsx, Recovery: 53531092)
  - `iterations/v3/apps/web-dashboard/src/components/database/VectorSearchPanel.module.scss` (ID: FHy2.scss, Recovery: 7ada8264)
  - `iterations/v3/apps/web-dashboard/src/components/database/VectorSearchPanel.tsx` (ID: CSX3.tsx, Recovery: -664da8b5)
  - `iterations/v3/apps/web-dashboard/src/components/metrics/MetricsDashboard.module.scss` (ID: k5Hw.scss, Recovery: 4b6559dc)
  - `iterations/v3/apps/web-dashboard/src/components/metrics/MetricsDashboard.tsx` (ID: FHnR.tsx, Recovery: 196826c3)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/AgentPerformanceGrid.module.scss` (ID: 0qA2.scss, Recovery: -6d77175f)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/AgentPerformanceGrid.tsx` (ID: 3to2.tsx, Recovery: -764ab578)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/AlertsDashboard.module.scss` (ID: 51dG.scss, Recovery: 2532a275)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/AlertsDashboard.tsx` (ID: ROuT.tsx, Recovery: 40d0185c)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/BusinessIntelligence.module.scss` (ID: uqPK.scss, Recovery: 3967346f)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/BusinessIntelligence.tsx` (ID: 1Hil.tsx, Recovery: -974bbaa)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/CoordinationMetrics.module.scss` (ID: 99s6.scss, Recovery: 447dd620)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/CoordinationMetrics.tsx` (ID: 9HCc.tsx, Recovery: -34acd8f9)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/MetricTile.module.scss` (ID: cWCT.scss, Recovery: 7208fece)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/MetricTile.tsx` (ID: YARH.tsx, Recovery: -78f224b)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/RealTimeMetricsStream.tsx` (ID: ZhHP.tsx, Recovery: -54103f89)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/SLOAlertsDashboard.module.scss` (ID: 6MuP.scss, Recovery: 76ee2677)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/SLOAlertsDashboard.tsx` (ID: vtnn.tsx, Recovery: -3df741a2)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/SLODashboard.module.scss` (ID: TTiR.scss, Recovery: -3ad27ab2)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/SLODashboard.tsx` (ID: XqiW.tsx, Recovery: -6e3c1bcb)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/SystemHealthOverview.module.scss` (ID: IZ0l.scss, Recovery: 4bf4ca54)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/SystemHealthOverview.tsx` (ID: hRHy.tsx, Recovery: -203470c5)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/SystemResourcesMonitor.module.scss` (ID: omqo.scss, Recovery: -7b99e4ec)
  - `iterations/v3/apps/web-dashboard/src/components/monitoring/SystemResourcesMonitor.tsx` (ID: BsZp.tsx, Recovery: -54066005)
  - `iterations/v3/apps/web-dashboard/src/components/shared/AttentionAlerts.module.scss` (ID: vYmU.scss, Recovery: -742a91d6)
  - `iterations/v3/apps/web-dashboard/src/components/shared/AttentionAlerts.tsx` (ID: d96M.tsx, Recovery: 606911)
  - `iterations/v3/apps/web-dashboard/src/components/shared/ConnectionStatus.module.scss` (ID: I8dN.scss, Recovery: -4ca2b9bd)
  - `iterations/v3/apps/web-dashboard/src/components/shared/ConnectionStatus.tsx` (ID: Dasa.tsx, Recovery: 72ca6a2a)
  - `iterations/v3/apps/web-dashboard/src/components/shared/Header.module.scss` (ID: SycE.scss, Recovery: 4bb36fa0)
  - `iterations/v3/apps/web-dashboard/src/components/shared/Header.test.tsx` (ID: TJTw.tsx, Recovery: 59a5d48f)
  - `iterations/v3/apps/web-dashboard/src/components/shared/Header.tsx` (ID: W5Uy.tsx, Recovery: 187a4087)
  - `iterations/v3/apps/web-dashboard/src/components/shared/Navigation.module.scss` (ID: B4Zu.scss, Recovery: -db75ad9)
  - `iterations/v3/apps/web-dashboard/src/components/shared/Navigation.tsx` (ID: 8Bmb.tsx, Recovery: 1e64ed0e)
  - `iterations/v3/apps/web-dashboard/src/components/shared/SimpleTest.test.tsx` (ID: 6f2R.tsx, Recovery: -3d4f98e8)
  - `iterations/v3/apps/web-dashboard/src/components/shared/SystemHealthOverview.module.scss` (ID: Dj4F.scss, Recovery: -5fe19ba9)
  - `iterations/v3/apps/web-dashboard/src/components/shared/SystemHealthOverview.tsx` (ID: znXL.tsx, Recovery: -153b23c2)
  - `iterations/v3/apps/web-dashboard/src/components/shared/TTSSettings.module.scss` (ID: NLL5.scss, Recovery: 67a3cf0b)
  - `iterations/v3/apps/web-dashboard/src/components/shared/TTSSettings.tsx` (ID: fMBq.tsx, Recovery: -3995c50e)
  - `iterations/v3/apps/web-dashboard/src/components/shared/VoicemailHistory.module.scss` (ID: 2n0z.scss, Recovery: 944bfbe)
  - `iterations/v3/apps/web-dashboard/src/components/shared/VoicemailHistory.tsx` (ID: tWaI.tsx, Recovery: -9cb715b)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/AuditTrailViewer.module.scss` (ID: 9Sjw.scss, Recovery: 215aa37d)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/AuditTrailViewer.tsx` (ID: ysMG.tsx, Recovery: -17925e9c)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/IterationTimeline.module.scss` (ID: KPxc.scss, Recovery: 73c78ea6)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/IterationTimeline.tsx` (ID: IJLQ.tsx, Recovery: -18b73a73)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/ModelPerformanceChart.module.scss` (ID: oY1v.scss, Recovery: -33089841)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/ModelPerformanceChart.tsx` (ID: tDi6.tsx, Recovery: -7014385a)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/SatisficingDashboard.module.scss` (ID: eAFQ.scss, Recovery: -68778f9e)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/SatisficingDashboard.tsx` (ID: f3zj.tsx, Recovery: 38752349)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/SelfPromptingMonitor.module.scss` (ID: vduQ.scss, Recovery: -8bab8f8)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/SelfPromptingMonitor.tsx` (ID: JrE2.tsx, Recovery: 1c6dffef)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/TaskCard.module.scss` (ID: eFou.scss, Recovery: 561b7cd5)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/TaskCard.tsx` (ID: fgik.tsx, Recovery: 7fe952bc)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/TaskFilters.module.scss` (ID: UUG5.scss, Recovery: -4464de2)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/TaskFilters.tsx` (ID: kAHV.tsx, Recovery: -f8e1efb)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/TaskFiltersBar.module.scss` (ID: TUJc.scss, Recovery: 27a6bb1d)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/TaskFiltersBar.tsx` (ID: W1lV.tsx, Recovery: -2a64a6fc)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/TaskList.module.scss` (ID: MQah.scss, Recovery: -22da329d)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/TaskList.tsx` (ID: QEfY.tsx, Recovery: -2f63eeb6)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/TaskMetrics.module.scss` (ID: nqso.scss, Recovery: 76b4e2e6)
  - `iterations/v3/apps/web-dashboard/src/components/tasks/TaskMetrics.tsx` (ID: JpyK.tsx, Recovery: -4211a633)
  - `iterations/v3/apps/web-dashboard/src/hooks/useArbiter.ts` (ID: 2nIV.ts, Recovery: 2362346e)
  - `iterations/v3/apps/web-dashboard/src/hooks/useTTS.ts` (ID: Uu2X.ts, Recovery: 3e2bc24)
  - `iterations/v3/apps/web-dashboard/src/hooks/useVoiceChat.ts` (ID: 3LIt.ts, Recovery: -40157473)
  - `iterations/v3/apps/web-dashboard/src/hooks/useVoiceRecording.ts` (ID: lmY1.ts, Recovery: -44dfa41c)
  - `iterations/v3/apps/web-dashboard/src/lib/analytics-api.ts` (ID: caDq.ts, Recovery: -1abaf47e)
  - `iterations/v3/apps/web-dashboard/src/lib/api-client.ts` (ID: cJAj.ts, Recovery: -7af58bf9)
  - `iterations/v3/apps/web-dashboard/src/lib/api/tasks.ts` (ID: qtxa.ts, Recovery: -11329e44)
  - `iterations/v3/apps/web-dashboard/src/lib/chat-api.ts` (ID: fZDJ.ts, Recovery: -5c2e4aa0)
  - `iterations/v3/apps/web-dashboard/src/lib/database-api.ts` (ID: aaIV.ts, Recovery: 38e29ddd)
  - `iterations/v3/apps/web-dashboard/src/lib/metrics-api.ts` (ID: G5KI.ts, Recovery: -2d3f865b)
  - `iterations/v3/apps/web-dashboard/src/lib/sse/SSEClient.ts` (ID: zEuU.ts, Recovery: -6e233091)
  - `iterations/v3/apps/web-dashboard/src/lib/sse/TaskEventsClient.ts` (ID: KFRU.ts, Recovery: 13ef7972)
  - `iterations/v3/apps/web-dashboard/src/lib/task-api.ts` (ID: y9l9.ts, Recovery: -611b496d)
  - `iterations/v3/apps/web-dashboard/src/lib/tts-api.ts` (ID: 7Kei.ts, Recovery: 7f794675)
  - `iterations/v3/apps/web-dashboard/src/lib/websocket/WebSocketClient.ts` (ID: asyB.ts, Recovery: 1ea1534b)
  - `iterations/v3/apps/web-dashboard/src/styles/globals.scss` (ID: JLuV.scss, Recovery: 3ec7b4bd)
  - `iterations/v3/apps/web-dashboard/src/styles/mixins.scss` (ID: svGv.scss, Recovery: -34eaae3)
  - `iterations/v3/apps/web-dashboard/src/styles/variables.scss` (ID: B3IR.scss, Recovery: -75e80aea)
  - `iterations/v3/apps/web-dashboard/src/types/analytics.ts` (ID: sLVX.ts, Recovery: 1e2f397b)
  - `iterations/v3/apps/web-dashboard/src/types/chat.ts` (ID: GMlG.ts, Recovery: 42acccc1)
  - `iterations/v3/apps/web-dashboard/src/types/database.ts` (ID: ETrm.ts, Recovery: 22cda1be)
  - `iterations/v3/apps/web-dashboard/src/types/metrics.ts` (ID: VpJU.ts, Recovery: -775e7be2)
  - `iterations/v3/apps/web-dashboard/src/types/tasks.ts` (ID: JOVz.ts, Recovery: -5b14198d)
  - `iterations/v3/apps/web-dashboard/src/types/tts.ts` (ID: nAAF.ts, Recovery: -74cbe112)
  - `iterations/v3/apps/web-dashboard/tailwind.config.js` (ID: 4DUx.js, Recovery: -4ddfed6e)
  - `iterations/v3/apps/web-dashboard/test-connection.js` (ID: F3P0.js, Recovery: 29fa6d91)
  - `iterations/v3/apps/web-dashboard/tsconfig.json` (ID: CPY4.json, Recovery: -4adb54de)
  - `iterations/v3/asr-bridge/Package.swift` (ID: ugxX.swift, Recovery: -5004f9d0)
  - `iterations/v3/asr-bridge/Sources/ASRBridge/ASRBridge.swift` (ID: pgFa.swift, Recovery: 6851dd96)
  - `iterations/v3/assets/dashboard.html` (ID: oWXw.html, Recovery: ce370d3)
  - `iterations/v3/bin/main.rs` (ID: Oevo.rs, Recovery: 2ef51210)
  - `iterations/v3/brittleness-test/Cargo.toml` (ID: lgWW.toml, Recovery: 744f8ba7)
  - `iterations/v3/brittleness-test/src/lib.rs` (ID: clzH.rs, Recovery: 70294810)
  - `iterations/v3/caching/Cargo.toml` (ID: sltU.toml, Recovery: -65986a06)
  - `iterations/v3/caching/README.md` (ID: gyfc.md, Recovery: -4f968d77)
  - `iterations/v3/caching/examples/basic_usage.rs` (ID: zYYx.rs, Recovery: -1d9269a9)
  - `iterations/v3/caching/src/integration.rs` (ID: YKFL.rs, Recovery: -736fa04c)
  - `iterations/v3/caching/src/lib.rs` (ID: iStT.rs, Recovery: -69bead9d)
  - `iterations/v3/claim-extraction/src/multi_modal_verification.rs.bak` (ID: 3S3A.bak, Recovery: 2caa316)
  - `iterations/v3/claim-extraction/tests/disambiguation_knowledge_test.rs` (ID: Q3Gi.rs, Recovery: -667a176)
  - `iterations/v3/cli/Cargo.toml` (ID: xYQ0.toml, Recovery: -742f1f07)
  - `iterations/v3/cli/src/main.rs` (ID: hadI.rs, Recovery: -6a98bd82)
  - `iterations/v3/config/production.yaml` (ID: 2gww.yaml, Recovery: 7846242d)
  - `iterations/v3/context-preservation-engine/src/encryption_tests.rs` (ID: 0lvn.rs, Recovery: 12f872e7)
  - `iterations/v3/database/examples/performance_optimization.rs` (ID: nlrE.rs, Recovery: 11b05fa3)
  - `iterations/v3/database/migrations/008_add_caws_violations.sql` (ID: 3XDq.sql, Recovery: 43c4cb3a)
  - `iterations/v3/database/migrations/009_external_knowledge_schema.sql` (ID: 6BQd.sql, Recovery: -2ad844)
  - `iterations/v3/database/migrations/010_database_integration_schema.sql` (ID: 9O5f.sql, Recovery: -4a8f5174)
  - `iterations/v3/database/migrations/011_artifacts_storage.sql` (ID: wbJx.sql, Recovery: -1a745a46)
  - `iterations/v3/database/migrations/012_artifact_versioning.sql` (ID: kBoM.sql, Recovery: 509061f5)
  - `iterations/v3/database/migrations/013_historical_claims.sql` (ID: ClAD.sql, Recovery: 119d646b)
  - `iterations/v3/database/migrations/014_core_persistence.sql` (ID: 0kII.sql, Recovery: 1641755f)
  - `iterations/v3/database/migrations/015_slo_tracking.sql` (ID: 2di4.sql, Recovery: -38202b61)
  - `iterations/v3/database/migrations/016_task_audit_logs.sql` (ID: OE51.sql, Recovery: 27d6f5ef)
  - `iterations/v3/database/migrations/20250120_create_performance_indexes.sql` (ID: 1lOB.sql, Recovery: -2f051867)
  - `iterations/v3/database/src/artifact_store.rs` (ID: f5bz.rs, Recovery: 1985257a)
  - `iterations/v3/database/src/backup_recovery.rs` (ID: u4Ah.rs, Recovery: -e724322)
  - `iterations/v3/database/src/backup_validator.rs` (ID: Nlgo.rs, Recovery: 9e4a539)
  - `iterations/v3/database/src/data_consistency.rs` (ID: ckKN.rs, Recovery: 5700aa8b)
  - `iterations/v3/database/src/knowledge_queries.rs` (ID: BqfA.rs, Recovery: -5e2f8b75)
  - `iterations/v3/database/src/optimization.rs` (ID: huIM.rs, Recovery: 50ee5d61)
  - `iterations/v3/database/src/vector_store.rs` (ID: h57y.rs, Recovery: 4b45ce9)
  - `iterations/v3/demo-integration.rs` (ID: Vbjy.rs, Recovery: -7e77afb9)
  - `iterations/v3/demo-self-prompting.rs` (ID: T53n.rs, Recovery: -7f0eac1a)
  - `iterations/v3/demo-simple.rs` (ID: nwi0.rs, Recovery: 4a1e6cd1)
  - `iterations/v3/demo_app/Cargo.toml` (ID: pbb6.toml, Recovery: 5f870178)
  - `iterations/v3/demo_app/config/demo_config.yaml` (ID: JH2s.yaml, Recovery: 73a01318)
  - `iterations/v3/demo_app/src/demo_runner.rs` (ID: 1yCI.rs, Recovery: 6bca6efa)
  - `iterations/v3/demo_app/src/main.rs` (ID: mnw3.rs, Recovery: 387531df)
  - `iterations/v3/demo_app/tests/integration_test.rs` (ID: ZjI3.rs, Recovery: 77e47e38)
  - `iterations/v3/demo_v3_system.rs` (ID: JBvD.rs, Recovery: -6e6ebd04)
  - `iterations/v3/docker-compose.yml` (ID: 0Wfi.yml, Recovery: 599e8e8d)
  - `iterations/v3/docker/Dockerfile.api` (ID: t0eC.api, Recovery: -5e0a3547)
  - `iterations/v3/docker/Dockerfile.dashboard` (ID: RGdL.dashboard, Recovery: -15fdf80d)
  - `iterations/v3/docker/Dockerfile.demo-app` (ID: WEnW.demo-app, Recovery: 3c8d7df8)
  - `iterations/v3/docker/Dockerfile.federated-learning` (ID: AjeF.federated-learning, Recovery: 25d99e48)
  - `iterations/v3/docker/Dockerfile.model-hotswap` (ID: jsbi.model-hotswap, Recovery: 31f9b29b)
  - `iterations/v3/docker/Dockerfile.multimodal-rag` (ID: FLWM.multimodal-rag, Recovery: 3fd569e0)
  - `iterations/v3/docker/Dockerfile.runtime-optimization` (ID: uWtR.runtime-optimization, Recovery: 69aa8683)
  - `iterations/v3/docker/Dockerfile.tool-ecosystem` (ID: A4t4.tool-ecosystem, Recovery: -16b222f4)
  - `iterations/v3/docker/Dockerfile.worker` (ID: ZP1g.worker, Recovery: -606bb581)
  - `iterations/v3/docker/docker-compose.production.yml` (ID: OvnB.yml, Recovery: -c9b9aa5)
  - `iterations/v3/docker/monitoring/alert_rules.yml` (ID: JdGW.yml, Recovery: 183047c8)
  - `iterations/v3/docker/monitoring/fluent-bit.conf` (ID: eAmN.conf, Recovery: -4aa82f80)
  - `iterations/v3/docker/monitoring/grafana/dashboards/agent-agency-overview.json` (ID: ZiCj.json, Recovery: -76f33f54)
  - `iterations/v3/docker/monitoring/grafana/provisioning/datasources/prometheus.yml` (ID: hIGz.yml, Recovery: 18a0e796)
  - `iterations/v3/docker/monitoring/parsers.conf` (ID: MSy2.conf, Recovery: 518e854)
  - `iterations/v3/docker/nginx/nginx.conf` (ID: mM65.conf, Recovery: 797ac41e)
  - `iterations/v3/docs/DATABASE_TROUBLESHOOTING.md` (ID: DEpk.md, Recovery: -25334607)
  - `iterations/v3/docs/PRODUCTION_DEPLOYMENT.md` (ID: KDfq.md, Recovery: -5150fa50)
  - `iterations/v3/docs/PRODUCTION_DEPLOYMENT_GUIDE.md` (ID: 3jO8.md, Recovery: -1680c58d)
  - `iterations/v3/docs/SYSTEM_OVERVIEW.md` (ID: Dn5G.md, Recovery: 758a99f2)
  - `iterations/v3/docs/agents.md` (ID: V3th.md, Recovery: -5c44b51f)
  - `iterations/v3/docs/contracts/execution-artifacts.schema.json` (ID: MDaw.json, Recovery: -1f81a871)
  - `iterations/v3/docs/contracts/quality-report.schema.json` (ID: gu3f.json, Recovery: -1fd47b93)
  - `iterations/v3/docs/contracts/refinement-decision.schema.json` (ID: NdY3.json, Recovery: -7e4819bf)
  - `iterations/v3/docs/contracts/task-request.schema.json` (ID: kGY5.json, Recovery: 28857a72)
  - `iterations/v3/docs/contracts/task-response.schema.json` (ID: L7sh.json, Recovery: aed242c)
  - `iterations/v3/docs/contracts/working-spec.schema.json` (ID: RuBJ.json, Recovery: -43864efe)
  - `iterations/v3/docs/end-to-end-autonomous-flow-architecture.md` (ID: n4a6.md, Recovery: -785b328c)
  - `iterations/v3/e2e-tests/Cargo.toml` (ID: pU5X.toml, Recovery: 272da6d)
  - `iterations/v3/e2e-tests/assertions.rs` (ID: 3PgW.rs, Recovery: -3b05203)
  - `iterations/v3/e2e-tests/autonomous_pipeline_test.rs` (ID: D7Ww.rs, Recovery: 57ebc98e)
  - `iterations/v3/e2e-tests/fixtures.rs` (ID: DgKK.rs, Recovery: -45e13ebc)
  - `iterations/v3/e2e-tests/harness.rs` (ID: Fk3C.rs, Recovery: -49e9b580)
  - `iterations/v3/e2e-tests/mod.rs` (ID: sLxt.rs, Recovery: -11286cd2)
  - `iterations/v3/e2e-tests/runner.rs` (ID: EaYO.rs, Recovery: 22cb3b9e)
  - `iterations/v3/e2e-tests/scenarios.rs` (ID: RqC3.rs, Recovery: -41898113)
  - `iterations/v3/enrichers/build.rs` (ID: xau6.rs, Recovery: bd6030d)
  - `iterations/v3/enrichers/src/python_bridge.rs` (ID: SauN.rs, Recovery: -716974c6)
  - `iterations/v3/env.production.example` (ID: dZGE.example, Recovery: -3e61ddec)
  - `iterations/v3/federated-learning/Cargo.toml` (ID: HsOo.toml, Recovery: -1da7714a)
  - `iterations/v3/federated-learning/src/aggregation.rs` (ID: qvgI.rs, Recovery: 61db68a2)
  - `iterations/v3/federated-learning/src/coordinator.rs` (ID: Fgm5.rs, Recovery: 779bc994)
  - `iterations/v3/federated-learning/src/differential_privacy.rs` (ID: pBg5.rs, Recovery: 76ce6d76)
  - `iterations/v3/federated-learning/src/encryption.rs` (ID: 13cP.rs, Recovery: 534bb217)
  - `iterations/v3/federated-learning/src/lib.rs` (ID: TUci.rs, Recovery: -21cdb4e1)
  - `iterations/v3/federated-learning/src/model_updates.rs` (ID: 9UFc.rs, Recovery: 6cbc5b50)
  - `iterations/v3/federated-learning/src/participant.rs` (ID: KukA.rs, Recovery: 6305f2b1)
  - `iterations/v3/federated-learning/src/protocol.rs` (ID: ticy.rs, Recovery: -7913ba5e)
  - `iterations/v3/federated-learning/src/security.rs` (ID: AYif.rs, Recovery: -4384406)
  - `iterations/v3/federated-learning/src/validation.rs` (ID: Kao9.rs, Recovery: 2ba04d61)
  - `iterations/v3/file_ops/Cargo.toml` (ID: db1d.toml, Recovery: 5dfcbb8e)
  - `iterations/v3/file_ops/src/git_workspace.rs` (ID: oUKK.rs, Recovery: -52d9a10c)
  - `iterations/v3/file_ops/src/lib.rs` (ID: Yq9z.rs, Recovery: 59d677f7)
  - `iterations/v3/file_ops/src/temp_workspace.rs` (ID: z7GY.rs, Recovery: -6f1eff08)
  - `iterations/v3/final_summary.rs` (ID: 1Wvx.rs, Recovery: -58794750)
  - `iterations/v3/frontier_test.rs` (ID: CVvh.rs, Recovery: -70b86457)
  - `iterations/v3/gap-analysis.md` (ID: 98tJ.md, Recovery: -5918170c)
  - `iterations/v3/integration-tests/src/autonomous_pipeline_test.rs` (ID: Ttrw.rs, Recovery: 5648819f)
  - `iterations/v3/integration-tests/src/integration_tests/config.rs` (ID: vXrD.rs, Recovery: -33bda3c)
  - `iterations/v3/integration-tests/src/integration_tests/logging.rs` (ID: fifG.rs, Recovery: -6f70e347)
  - `iterations/v3/integration-tests/src/integration_tests/mod.rs` (ID: gNPK.rs, Recovery: -36dd594a)
  - `iterations/v3/integration-tests/src/integration_tests/runner.rs` (ID: ullh.rs, Recovery: 29811b16)
  - `iterations/v3/integration-tests/src/integration_tests/types.rs` (ID: y3he.rs, Recovery: abb0f9f)
  - `iterations/v3/integration-tests/src/multimodal_rag_e2e_tests.rs` (ID: WvYr.rs, Recovery: -4e4e8749)
  - `iterations/v3/integration-tests/src/multimodal_rag_integration_test.rs` (ID: 6hzy.rs, Recovery: -1a6b7150)
  - `iterations/v3/integration_test.rs` (ID: d9xq.rs, Recovery: -2dc3a3cc)
  - `iterations/v3/integration_test.sh` (ID: Yo9y.sh, Recovery: -2dc3a3b8)
  - `iterations/v3/interfaces/Cargo.toml` (ID: dA8K.toml, Recovery: -1c3b12bd)
  - `iterations/v3/interfaces/api.rs` (ID: VOZJ.rs, Recovery: -37c94914)
  - `iterations/v3/interfaces/cli.rs` (ID: RNrO.rs, Recovery: -3497f5da)
  - `iterations/v3/interfaces/mcp.rs` (ID: u42X.rs, Recovery: -24032494)
  - `iterations/v3/interfaces/mod.rs` (ID: 7mT2.rs, Recovery: -235f7efc)
  - `iterations/v3/interfaces/src/api.rs` (ID: BRoo.rs, Recovery: -32c0d209)
  - `iterations/v3/interfaces/src/cli.rs` (ID: omuJ.rs, Recovery: -2f8f7ecf)
  - `iterations/v3/interfaces/src/lib.rs` (ID: 0vFF.rs, Recovery: -20615654)
  - `iterations/v3/interfaces/src/mcp.rs` (ID: AteI.rs, Recovery: -1efaad89)
  - `iterations/v3/interfaces/src/websocket.rs` (ID: rcVc.rs, Recovery: -193f0056)
  - `iterations/v3/interfaces/websocket.rs` (ID: VQZA.rs, Recovery: 7c493cdf)
  - `iterations/v3/knowledge-ingestor/Cargo.toml` (ID: TrYr.toml, Recovery: -49e14799)
  - `iterations/v3/knowledge-ingestor/README.md` (ID: 6YDD.md, Recovery: 6f3dce7c)
  - `iterations/v3/knowledge-ingestor/src/bin/load_core_vocabulary.rs` (ID: D60P.rs, Recovery: -5aa8102c)
  - `iterations/v3/knowledge-ingestor/src/core_vocabulary.rs` (ID: C0ot.rs, Recovery: -3ae43ad1)
  - `iterations/v3/knowledge-ingestor/src/cross_reference.rs` (ID: PGXF.rs, Recovery: 626ffae9)
  - `iterations/v3/knowledge-ingestor/src/lib.rs` (ID: 1ZF4.rs, Recovery: -4e078b30)
  - `iterations/v3/knowledge-ingestor/src/on_demand.rs` (ID: izdX.rs, Recovery: 36eed06a)
  - `iterations/v3/knowledge-ingestor/src/types.rs` (ID: AmuM.rs, Recovery: -41f9ff24)
  - `iterations/v3/knowledge-ingestor/src/wikidata.rs` (ID: 8TGp.rs, Recovery: -517e0571)
  - `iterations/v3/knowledge-ingestor/src/wordnet.rs` (ID: IhDu.rs, Recovery: -3ebe4be)
  - `iterations/v3/knowledge-ingestor/tests/integration_test.rs` (ID: a2Vs.rs, Recovery: -74a7d497)
  - `iterations/v3/load-testing/k6-multimodal-rag-test.js` (ID: ZzRD.js, Recovery: -6dc22d0f)
  - `iterations/v3/memory/Cargo.toml` (ID: ISZU.toml, Recovery: 5c6f5d3c)
  - `iterations/v3/memory/README.md` (ID: 6k6b.md, Recovery: 328abb07)
  - `iterations/v3/memory/examples/comprehensive_usage.rs` (ID: 8sch.rs, Recovery: -1f5e8311)
  - `iterations/v3/memory/src/integration.rs` (ID: Fzir.rs, Recovery: -3c7a3b0a)
  - `iterations/v3/memory/src/lib.rs` (ID: 3ls1.rs, Recovery: 584919a5)
  - `iterations/v3/methods.txt` (ID: 1qxF.txt, Recovery: -3243914a)
  - `iterations/v3/migrations/001_enable_pgvector.sql` (ID: awrb.sql, Recovery: 2a10199c)
  - `iterations/v3/migrations/002_create_vector_tables.sql` (ID: h5aS.sql, Recovery: -2f4cd2e9)
  - `iterations/v3/model-benchmarking/src/benchmark_runner.rs.bak2` (ID: OOSR.bak2, Recovery: -725b2c56)
  - `iterations/v3/model-hotswap/Cargo.toml` (ID: cxyf.toml, Recovery: 1e1b365d)
  - `iterations/v3/model-hotswap/src/hotswap_manager.rs` (ID: IcT6.rs, Recovery: -21090323)
  - `iterations/v3/model-hotswap/src/lib.rs` (ID: 86hw.rs, Recovery: 19f4f2c6)
  - `iterations/v3/model-hotswap/src/load_balancer.rs` (ID: Obd1.rs, Recovery: -76fe9e44)
  - `iterations/v3/model-hotswap/src/model_registry.rs` (ID: JmEP.rs, Recovery: -13a8e080)
  - `iterations/v3/model-hotswap/src/performance_router.rs` (ID: wfQ7.rs, Recovery: -e0d2a45)
  - `iterations/v3/model-hotswap/src/rollback_manager.rs` (ID: rW4c.rs, Recovery: c415561)
  - `iterations/v3/model-hotswap/src/traffic_splitter.rs` (ID: dyPo.rs, Recovery: 2826cc8a)
  - `iterations/v3/model-hotswap/src/version_manager.rs` (ID: bClj.rs, Recovery: 56d0fc65)
  - `iterations/v3/monitoring/alert_rules.yml` (ID: IPyn.yml, Recovery: -7facf14b)
  - `iterations/v3/monitoring/alertmanager.yml` (ID: bLoG.yml, Recovery: 46a906a4)
  - `iterations/v3/monitoring/grafana/dashboards/business-intelligence.json` (ID: V3zK.json, Recovery: 30ebba86)
  - `iterations/v3/monitoring/grafana/dashboards/multimodal-rag.yml` (ID: SwjV.yml, Recovery: -3995461b)
  - `iterations/v3/monitoring/grafana/dashboards/slo-tracking.json` (ID: KoSq.json, Recovery: 332ff54c)
  - `iterations/v3/monitoring/grafana/dashboards/system-overview.json` (ID: bs8B.json, Recovery: 5f1ceabb)
  - `iterations/v3/monitoring/grafana/datasources/prometheus.yml` (ID: mkMU.yml, Recovery: -35be2d37)
  - `iterations/v3/monitoring/grafana/provisioning/dashboards/agent-agency-dashboard.json` (ID: 4IRX.json, Recovery: 2a1cfda0)
  - `iterations/v3/monitoring/grafana/provisioning/datasources/prometheus.yml` (ID: 0ZK3.yml, Recovery: -2b1b457d)
  - `iterations/v3/monitoring/multimodal_rag_rules.yml` (ID: n7mN.yml, Recovery: 1df42178)
  - `iterations/v3/monitoring/prometheus.yml` (ID: pbHF.yml, Recovery: -50f5e583)
  - `iterations/v3/observability/src/cache/mod.rs` (ID: NqDn.rs, Recovery: -1805cdef)
  - `iterations/v3/observability/src/cache/redis_cache.rs` (ID: IJay.rs, Recovery: 3238fbd5)
  - `iterations/v3/observability/src/diff_observability.rs` (ID: 42vo.rs, Recovery: -2026a0af)
  - `iterations/v3/observability/src/errors.rs` (ID: IOA9.rs, Recovery: 548618cd)
  - `iterations/v3/observability/src/metrics/prometheus.rs` (ID: YZNr.rs, Recovery: -50c5a6de)
  - `iterations/v3/observability/src/metrics/redis.rs` (ID: jNvr.rs, Recovery: -62230de9)
  - `iterations/v3/observability/src/metrics/statsd.rs` (ID: sz69.rs, Recovery: 3c618b07)
  - `iterations/v3/observability/src/multimodal_metrics.rs` (ID: RH8T.rs, Recovery: 1109d300)
  - `iterations/v3/planning-agent/Cargo.toml` (ID: LFyx.toml, Recovery: 2e4d2f28)
  - `iterations/v3/planning-agent/src/caws_integration.rs` (ID: uBrT.rs, Recovery: 2c3bb9d9)
  - `iterations/v3/planning-agent/src/error.rs` (ID: ybx4.rs, Recovery: 5feab36e)
  - `iterations/v3/planning-agent/src/lib.rs` (ID: D7Fe.rs, Recovery: 2a26eb91)
  - `iterations/v3/planning-agent/src/planner.rs` (ID: rzPV.rs, Recovery: -90eae5c)
  - `iterations/v3/planning-agent/src/refinement_engine.rs` (ID: YVMj.rs, Recovery: 5e2b880)
  - `iterations/v3/planning-agent/src/validation_pipeline.rs` (ID: 9NQp.rs, Recovery: 323e5d8e)
  - `iterations/v3/production/Cargo.toml` (ID: iwrw.toml, Recovery: 7ffed584)
  - `iterations/v3/production/documentation.rs` (ID: MxGl.rs, Recovery: 47f6bd0d)
  - `iterations/v3/production/error_handling.rs` (ID: tJYy.rs, Recovery: 1e73e41b)
  - `iterations/v3/production/mod.rs` (ID: edR6.rs, Recovery: 32d2b1c5)
  - `iterations/v3/production/observability.rs` (ID: P2R0.rs, Recovery: 6c03a1c6)
  - `iterations/v3/production/security.rs` (ID: kf01.rs, Recovery: 3d955597)
  - `iterations/v3/research/src/multimodal_context_provider.rs` (ID: wIWU.rs, Recovery: -5ec343dc)
  - `iterations/v3/runtime-optimization/Cargo.toml` (ID: BYy9.toml, Recovery: -2b1ab925)
  - `iterations/v3/runtime-optimization/src/arbiter_pipeline.rs` (ID: O7Sj.rs, Recovery: -15512c03)
  - `iterations/v3/runtime-optimization/src/bayesian_optimizer.rs` (ID: Xhlo.rs, Recovery: 300d4af1)
  - `iterations/v3/runtime-optimization/src/chunked_execution.rs` (ID: JADE.rs, Recovery: -4a51a21c)
  - `iterations/v3/runtime-optimization/src/chunked_executor.rs` (ID: XSPl.rs, Recovery: -4c634d71)
  - `iterations/v3/runtime-optimization/src/kokoro_tuning.rs` (ID: Br8r.rs, Recovery: -6c93f126)
  - `iterations/v3/runtime-optimization/src/lib.rs` (ID: ugNw.rs, Recovery: -2f40fcbc)
  - `iterations/v3/runtime-optimization/src/performance_monitor.rs` (ID: 9QCm.rs, Recovery: -48427042)
  - `iterations/v3/runtime-optimization/src/precision_engineering.rs` (ID: 25nC.rs, Recovery: 51b74757)
  - `iterations/v3/runtime-optimization/src/quality_guardrails.rs` (ID: lwDE.rs, Recovery: 3768be59)
  - `iterations/v3/runtime-optimization/src/streaming_pipeline.rs` (ID: t7tR.rs, Recovery: b95e076)
  - `iterations/v3/runtime-optimization/src/thermal_scheduler.rs` (ID: oxNO.rs, Recovery: 97a9376)
  - `iterations/v3/scripts/deploy-production.sh` (ID: 0BBw.sh, Recovery: 451c17a9)
  - `iterations/v3/scripts/install-git-hooks.sh` (ID: 92DC.sh, Recovery: -1b460dbb)
  - `iterations/v3/scripts/run-e2e-tests.sh` (ID: tfQm.sh, Recovery: -3a327542)
  - `iterations/v3/scripts/run_integration_tests.sh` (ID: faJa.sh, Recovery: -7b669b9a)
  - `iterations/v3/security/Cargo.toml` (ID: xqLl.toml, Recovery: 48cd5d1d)
  - `iterations/v3/security/src/audit.rs` (ID: tISz.rs, Recovery: -3df24690)
  - `iterations/v3/security/src/authentication.rs` (ID: mGTI.rs, Recovery: -1e74c345)
  - `iterations/v3/security/src/circuit_breaker.rs` (ID: TrRH.rs, Recovery: 50a58cd5)
  - `iterations/v3/security/src/input_validation.rs` (ID: x7tZ.rs, Recovery: -f57759b)
  - `iterations/v3/security/src/lib.rs` (ID: tMgH.rs, Recovery: 44a71986)
  - `iterations/v3/security/src/rate_limiting.rs` (ID: yMgt.rs, Recovery: -44b4e15b)
  - `iterations/v3/security/src/sanitization.rs` (ID: EcDU.rs, Recovery: -3e7a3c66)
  - `iterations/v3/security/src/secret_manager.rs` (ID: XLYS.rs, Recovery: 44df0055)
  - `iterations/v3/security/src/secure_config.rs` (ID: AhvU.rs, Recovery: 43a0f5e1)
  - `iterations/v3/source-integrity/src/storage_new.rs` (ID: 50mn.rs, Recovery: 13ae9c14)
  - `iterations/v3/src/bin/api-server.rs` (ID: nNEd.rs, Recovery: -6e053b98)
  - `iterations/v3/src/bin/cli.rs` (ID: 4WW1.rs, Recovery: 6a5d920)
  - `iterations/v3/system-health-monitor/tests/basic_tests.rs` (ID: RmuI.rs, Recovery: -1b8eee2d)
  - `iterations/v3/test-api-server-Cargo.toml` (ID: UOxW.toml, Recovery: -2000a37a)
  - `iterations/v3/test-api-server.rs` (ID: zm4Q.rs, Recovery: -5480e3e4)
  - `iterations/v3/test-core-functionality.rs` (ID: ePUs.rs, Recovery: -298c7c0b)
  - `iterations/v3/test_api_config.rs` (ID: F4ec.rs, Recovery: 5afa4cd9)
  - `iterations/v3/test_orchestrator.rs` (ID: Ivyr.rs, Recovery: -58d571ca)
  - `iterations/v3/tests/README.md` (ID: 0jcx.md, Recovery: -612d6d15)
  - `iterations/v3/tests/benchmarks/Cargo.toml` (ID: Hf4E.toml, Recovery: 1b8cc419)
  - `iterations/v3/tests/benchmarks/benches/model_inference.rs` (ID: 34l7.rs, Recovery: 7febca0a)
  - `iterations/v3/tests/integration.rs` (ID: Bw4G.rs, Recovery: -2cc09bf9)
  - `iterations/v3/tests/run-test-plan.sh` (ID: YoJV.sh, Recovery: -36b0be93)
  - `iterations/v3/tests/test-plan.md` (ID: aQc1.md, Recovery: -303ecd53)
  - `iterations/v3/tests/test_data/complex_function.rs` (ID: pHNR.rs, Recovery: -64b748bc)
  - `iterations/v3/tests/test_data/style_issues.py` (ID: kJF4.py, Recovery: 4e4b66eb)
  - `iterations/v3/tests/test_data/syntax_error.rs` (ID: R7OL.rs, Recovery: -754f9d01)
  - `iterations/v3/tests/test_data/type_errors.ts` (ID: Olou.ts, Recovery: -65cfc09f)
  - `iterations/v3/tool-ecosystem/Cargo.toml` (ID: 5W09.toml, Recovery: -4072490e)
  - `iterations/v3/tool-ecosystem/src/conflict_resolution_tools.rs` (ID: FGqq.rs, Recovery: 6cb9c7cb)
  - `iterations/v3/tool-ecosystem/src/evidence_collection_tools.rs` (ID: jS3q.rs, Recovery: 10af201e)
  - `iterations/v3/tool-ecosystem/src/lib.rs` (ID: adgK.rs, Recovery: -44988ca5)
  - `iterations/v3/tool-ecosystem/src/tool_coordinator.rs` (ID: L9YA.rs, Recovery: -11608e2b)
  - `iterations/v3/tool-ecosystem/src/tool_discovery.rs` (ID: AcRO.rs, Recovery: -5808db2b)
  - `iterations/v3/tool-ecosystem/src/tool_execution.rs` (ID: wAga.rs, Recovery: 1929c66d)
  - `iterations/v3/tool-ecosystem/src/tool_registry.rs` (ID: WyyX.rs, Recovery: 982d3c)
  - `iterations/v3/validate_implementations.rs` (ID: Et2s.rs, Recovery: 24b7da99)
  - `iterations/v3/vision-bridge/Package.swift` (ID: Un3b.swift, Recovery: 5d812f8c)
  - `iterations/v3/vision-bridge/Sources/VisionBridge/VisionBridge.swift` (ID: 2Sqa.swift, Recovery: 15bee83e)
  - `iterations/v3/worker/Cargo.toml` (ID: s1ic.toml, Recovery: 3a13c69f)
  - `iterations/v3/worker/src/main.rs` (ID: qFGV.rs, Recovery: -507eee68)
  - `iterations/v3/worker_integration_test.rs` (ID: 0EBG.rs, Recovery: -25186151)

## Phase 2: Reconstruction Commands

### Step 1: Create Directory Structure
```bash
mkdir -p '.caws'
mkdir -p '.cursor/rules'
mkdir -p '.github/workflows'
mkdir -p 'apps/tools/caws'
mkdir -p 'apps/tools/caws/shared'
mkdir -p 'demo'
mkdir -p 'demo/src'
mkdir -p 'deploy'
mkdir -p 'deploy/docker'
mkdir -p 'deploy/docker-compose'
mkdir -p 'deploy/kubernetes/aws'
mkdir -p 'deploy/kubernetes/base'
mkdir -p 'deploy/runbooks'
mkdir -p 'deploy/terraform/aws'
mkdir -p 'docs'
mkdir -p 'docs-status'
mkdir -p 'docs/agents'
mkdir -p 'docs/database'
mkdir -p 'iterations/poc'
mkdir -p 'iterations/poc/apps/tools/caws'
mkdir -p 'iterations/poc/apps/tools/caws/shared'
mkdir -p 'iterations/poc/src/mcp/evaluation/evaluators'
mkdir -p 'iterations/poc/src/mcp/resources'
mkdir -p 'iterations/poc/src/mcp/tools/categories'
mkdir -p 'iterations/v2'
mkdir -p 'iterations/v2/apps/web-observer'
mkdir -p 'iterations/v2/apps/web-observer/src/app'
mkdir -p 'iterations/v2/apps/web-observer/src/components'
mkdir -p 'iterations/v2/docs'
mkdir -p 'iterations/v2/docs/3-agent-rl-training'
mkdir -p 'iterations/v2/docs/templates'
mkdir -p 'iterations/v2/python-services/dspy-integration'
mkdir -p 'iterations/v2/src/config/validation'
mkdir -p 'iterations/v2/tests/fixtures/caws-integration'
mkdir -p 'iterations/v3'
mkdir -p 'iterations/v3/.github/workflows'
mkdir -p 'iterations/v3/agent-agency-contracts'
mkdir -p 'iterations/v3/agent-agency-contracts/src'
mkdir -p 'iterations/v3/api-server'
mkdir -p 'iterations/v3/api-server/src'
mkdir -p 'iterations/v3/apple-silicon'
mkdir -p 'iterations/v3/apple-silicon/.caws'
mkdir -p 'iterations/v3/apple-silicon/src'
mkdir -p 'iterations/v3/apple-silicon/src/ane'
mkdir -p 'iterations/v3/apple-silicon/src/memory'
mkdir -p 'iterations/v3/apple-silicon/tests'
mkdir -p 'iterations/v3/apps/tools/caws'
mkdir -p 'iterations/v3/apps/tools/caws/__tests__'
mkdir -p 'iterations/v3/apps/tools/caws/modules'
mkdir -p 'iterations/v3/apps/tools/caws/workflow-modules'
mkdir -p 'iterations/v3/apps/web-dashboard'
mkdir -p 'iterations/v3/apps/web-dashboard/cypress/e2e'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/alerts'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/alerts/%5BalertId%5D/acknowledge'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/alerts/%5BalertId%5D/resolve'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/alerts/statistics'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/analytics'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/chat/ws/%5BsessionId%5D'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/database/connections'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/database/query'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/database/tables'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/database/tables/%5BtableName%5D/schema'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/database/vector-search'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/health'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/metrics'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/metrics/stream'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/proxy/%5B...path%5D'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/slo-alerts'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/slo-alerts/%5BalertId%5D/acknowledge'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/slos'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/slos/%5BsloName%5D/measurements'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/slos/%5BsloName%5D/status'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/tasks'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D/action'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D/arbiter-verdict'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D/claim-verification'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/tasks/events'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/api/tts'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/tasks'
mkdir -p 'iterations/v3/apps/web-dashboard/src/app/tasks/%5BtaskId%5D'
mkdir -p 'iterations/v3/apps/web-dashboard/src/components/analytics'
mkdir -p 'iterations/v3/apps/web-dashboard/src/components/arbiter'
mkdir -p 'iterations/v3/apps/web-dashboard/src/components/chat'
mkdir -p 'iterations/v3/apps/web-dashboard/src/components/database'
mkdir -p 'iterations/v3/apps/web-dashboard/src/components/metrics'
mkdir -p 'iterations/v3/apps/web-dashboard/src/components/monitoring'
mkdir -p 'iterations/v3/apps/web-dashboard/src/components/shared'
mkdir -p 'iterations/v3/apps/web-dashboard/src/components/tasks'
mkdir -p 'iterations/v3/apps/web-dashboard/src/hooks'
mkdir -p 'iterations/v3/apps/web-dashboard/src/lib'
mkdir -p 'iterations/v3/apps/web-dashboard/src/lib/api'
mkdir -p 'iterations/v3/apps/web-dashboard/src/lib/sse'
mkdir -p 'iterations/v3/apps/web-dashboard/src/lib/websocket'
mkdir -p 'iterations/v3/apps/web-dashboard/src/styles'
mkdir -p 'iterations/v3/apps/web-dashboard/src/types'
mkdir -p 'iterations/v3/asr-bridge'
mkdir -p 'iterations/v3/asr-bridge/Sources/ASRBridge'
mkdir -p 'iterations/v3/assets'
mkdir -p 'iterations/v3/bin'
mkdir -p 'iterations/v3/brittleness-test'
mkdir -p 'iterations/v3/brittleness-test/src'
mkdir -p 'iterations/v3/caching'
mkdir -p 'iterations/v3/caching/examples'
mkdir -p 'iterations/v3/caching/src'
mkdir -p 'iterations/v3/claim-extraction/src'
mkdir -p 'iterations/v3/claim-extraction/tests'
mkdir -p 'iterations/v3/cli'
mkdir -p 'iterations/v3/cli/src'
mkdir -p 'iterations/v3/config'
mkdir -p 'iterations/v3/context-preservation-engine/src'
mkdir -p 'iterations/v3/council'
mkdir -p 'iterations/v3/council/src'
mkdir -p 'iterations/v3/database/examples'
mkdir -p 'iterations/v3/database/migrations'
mkdir -p 'iterations/v3/database/src'
mkdir -p 'iterations/v3/demo_app'
mkdir -p 'iterations/v3/demo_app/config'
mkdir -p 'iterations/v3/demo_app/src'
mkdir -p 'iterations/v3/demo_app/tests'
mkdir -p 'iterations/v3/docker'
mkdir -p 'iterations/v3/docker/monitoring'
mkdir -p 'iterations/v3/docker/monitoring/grafana/dashboards'
mkdir -p 'iterations/v3/docker/monitoring/grafana/provisioning/datasources'
mkdir -p 'iterations/v3/docker/nginx'
mkdir -p 'iterations/v3/docs'
mkdir -p 'iterations/v3/docs/contracts'
mkdir -p 'iterations/v3/e2e-tests'
mkdir -p 'iterations/v3/embedding-service/src'
mkdir -p 'iterations/v3/enrichers'
mkdir -p 'iterations/v3/enrichers/src'
mkdir -p 'iterations/v3/federated-learning'
mkdir -p 'iterations/v3/federated-learning/src'
mkdir -p 'iterations/v3/file_ops'
mkdir -p 'iterations/v3/file_ops/src'
mkdir -p 'iterations/v3/integration-tests/src'
mkdir -p 'iterations/v3/integration-tests/src/integration_tests'
mkdir -p 'iterations/v3/interfaces'
mkdir -p 'iterations/v3/interfaces/src'
mkdir -p 'iterations/v3/knowledge-ingestor'
mkdir -p 'iterations/v3/knowledge-ingestor/src'
mkdir -p 'iterations/v3/knowledge-ingestor/src/bin'
mkdir -p 'iterations/v3/knowledge-ingestor/tests'
mkdir -p 'iterations/v3/load-testing'
mkdir -p 'iterations/v3/memory'
mkdir -p 'iterations/v3/memory/examples'
mkdir -p 'iterations/v3/memory/src'
mkdir -p 'iterations/v3/migrations'
mkdir -p 'iterations/v3/model-benchmarking/src'
mkdir -p 'iterations/v3/model-hotswap'
mkdir -p 'iterations/v3/model-hotswap/src'
mkdir -p 'iterations/v3/monitoring'
mkdir -p 'iterations/v3/monitoring/grafana/dashboards'
mkdir -p 'iterations/v3/monitoring/grafana/datasources'
mkdir -p 'iterations/v3/monitoring/grafana/provisioning/dashboards'
mkdir -p 'iterations/v3/monitoring/grafana/provisioning/datasources'
mkdir -p 'iterations/v3/observability/src'
mkdir -p 'iterations/v3/observability/src/cache'
mkdir -p 'iterations/v3/observability/src/metrics'
mkdir -p 'iterations/v3/orchestration/src'
mkdir -p 'iterations/v3/orchestration/src/artifacts'
mkdir -p 'iterations/v3/orchestration/src/planning'
mkdir -p 'iterations/v3/orchestration/src/quality'
mkdir -p 'iterations/v3/orchestration/src/refinement'
mkdir -p 'iterations/v3/orchestration/src/tracking'
mkdir -p 'iterations/v3/planning-agent'
mkdir -p 'iterations/v3/planning-agent/src'
mkdir -p 'iterations/v3/production'
mkdir -p 'iterations/v3/research/src'
mkdir -p 'iterations/v3/runtime-optimization'
mkdir -p 'iterations/v3/runtime-optimization/src'
mkdir -p 'iterations/v3/scripts'
mkdir -p 'iterations/v3/security'
mkdir -p 'iterations/v3/security/src'
mkdir -p 'iterations/v3/self-prompting-agent'
mkdir -p 'iterations/v3/self-prompting-agent/examples'
mkdir -p 'iterations/v3/self-prompting-agent/src'
mkdir -p 'iterations/v3/self-prompting-agent/src/caws'
mkdir -p 'iterations/v3/self-prompting-agent/src/evaluation'
mkdir -p 'iterations/v3/self-prompting-agent/src/models'
mkdir -p 'iterations/v3/self-prompting-agent/src/prompting'
mkdir -p 'iterations/v3/self-prompting-agent/src/sandbox'
mkdir -p 'iterations/v3/self-prompting-agent/tests'
mkdir -p 'iterations/v3/source-integrity/src'
mkdir -p 'iterations/v3/src/bin'
mkdir -p 'iterations/v3/system-health-monitor/tests'
mkdir -p 'iterations/v3/tests'
mkdir -p 'iterations/v3/tests/benchmarks'
mkdir -p 'iterations/v3/tests/benchmarks/benches'
mkdir -p 'iterations/v3/tests/test_data'
mkdir -p 'iterations/v3/tool-ecosystem'
mkdir -p 'iterations/v3/tool-ecosystem/src'
mkdir -p 'iterations/v3/vision-bridge'
mkdir -p 'iterations/v3/vision-bridge/Sources/VisionBridge'
mkdir -p 'iterations/v3/worker'
mkdir -p 'iterations/v3/worker/src'
mkdir -p 'iterations/v3/workers/src'
mkdir -p 'scripts'
mkdir -p 'scripts/disaster-recovery'
mkdir -p 'test-project'
```

### Step 2: Restore Files
```bash
cp 'recovered_work/13c18ee1/O9Rr.md' '.caws/EXEC_SUMMARY.md'
cp 'recovered_work/24b4e8fa/USYf.md' '.caws/FAST_WINS_ROADMAP.md'
cp 'recovered_work/25fe648/BvBZ.md' '.caws/P0_CODE_MAP.md'
cp 'recovered_work/-4b322548/e5ZW.md' '.caws/P0_DAILY_CHECKLIST.md'
cp 'recovered_work/19ad3a4b/cjFi.md' '.caws/README.md'
cp 'recovered_work/4173dda5/uMot.mdc' '.cursor/rules/15-documentation-organization.mdc'
cp 'recovered_work/-2de0c91/8xHK.yml' '.github/workflows/deploy.yml'
cp 'recovered_work/1c1d7d75/sCKi' '.pnpmrc'
cp 'recovered_work/-5e0e1d2b/i0PK.md' 'AUDIT_TRAIL_README.md'
cp 'recovered_work/1c8ec1bb/UuFc.md' 'BUILD_OPTIMIZATION_SUMMARY.md'
cp 'recovered_work/-47e6c78c/N0tC.md' 'CODE_QUALITY_IMPROVEMENTS_SUMMARY.md'
cp 'recovered_work/3e361f59/Vi0E.md' 'DEVELOPMENT_LEARNINGS.md'
cp 'recovered_work/-323271b4/qvFa.md' 'EDGE_CASE_TESTING_DOCUMENTATION.md'
cp 'recovered_work/-64e5aee4/q094.md' 'END_TO_END_ARCHITECTURE_ANALYSIS.md'
cp 'recovered_work/-129afbdf/VElx.md' 'FINAL_PRODUCTION_ROADMAP.md'
cp 'recovered_work/-7b1880d6/97I6.md' 'FINAL_SYSTEM_OVERVIEW.md'
cp 'recovered_work/1bc12ce9/uqCB.md' 'RELEASE_NOTES_V3.0.0.md'
cp 'recovered_work/280f748d/J4L4.md' 'SYSTEM_HARDENING_ANALYSIS.md'
cp 'recovered_work/-16363520/tKzJ.sh' 'analyze-edge-case-results.sh'
cp 'recovered_work/-2f2779b4/j3R9.js' 'apps/tools/caws/dashboard.js'
cp 'recovered_work/-66ae0e38/SJTe.ts' 'apps/tools/caws/security-provenance.ts'
cp 'recovered_work/7bed419/9Vro.ts' 'apps/tools/caws/shared/gate-checker.ts'
cp 'recovered_work/-103b7756/Zbo6.js' 'apps/tools/caws/test-quality.js'
cp 'recovered_work/-c6a99b2/0zzt.rs' 'audit-trail-demo.rs'
cp 'recovered_work/-6896d8bc/laTy.rs' 'audit-trail-showcase.rs'
cp 'recovered_work/5fcbc0f3/J0y2.toml' 'demo/Cargo.toml'
cp 'recovered_work/40c861c4/Sr4I.rs' 'demo/src/main.rs'
cp 'recovered_work/5f882534/c5vO.md' 'deploy/README.md'
cp 'recovered_work/-4298c6e8/3qm7.yml' 'deploy/docker-compose/dev.yml'
cp 'recovered_work/-32fefef2/5mx9.yml' 'deploy/docker-compose/kong.yml'
cp 'recovered_work/-63cb2fe5/aufB.council' 'deploy/docker/Dockerfile.council'
cp 'recovered_work/6b791350/Wgsi.orchestrator' 'deploy/docker/Dockerfile.orchestrator'
cp 'recovered_work/-206dff42/j1zO.txt' 'deploy/docker/requirements.txt'
cp 'recovered_work/ad54aad/0YD0.yml' 'deploy/kubernetes/aws/kustomization.yml'
cp 'recovered_work/-7c65f6ad/Bi4h.yml' 'deploy/kubernetes/base/council-deployment.yml'
cp 'recovered_work/-11fc3333/NYHy.yml' 'deploy/kubernetes/base/kustomization.yml'
cp 'recovered_work/-434675b3/DXjv.yml' 'deploy/kubernetes/base/namespace.yml'
cp 'recovered_work/8df345a/fzMl.yml' 'deploy/kubernetes/base/orchestrator-deployment.yml'
cp 'recovered_work/-38fce95c/yXPj.yml' 'deploy/kubernetes/base/postgres-statefulset.yml'
cp 'recovered_work/-2bc316c7/Lu5b.yml' 'deploy/kubernetes/base/redis-deployment.yml'
cp 'recovered_work/-4eac7f6/CYvN.md' 'deploy/runbooks/README.md'
cp 'recovered_work/-6a09db4d/B4ac.md' 'deploy/runbooks/incident-api-outage.md'
cp 'recovered_work/453342d7/Logr.tf' 'deploy/terraform/aws/main.tf'
cp 'recovered_work/1192ac39/XrZn.tf' 'deploy/terraform/aws/variables.tf'
cp 'recovered_work/-727e7e19/FXSR.md' 'docs-status/README.md'
cp 'recovered_work/-750b069a/Opij.md' 'docs/P0-IMPLEMENTATION-ROADMAP.md'
cp 'recovered_work/-2b27c946/xdOf.md' 'docs/PLACEHOLDER-DETECTION-GUIDE.md'
cp 'recovered_work/-51898f09/yHRn.md' 'docs/SYSTEM_HARDENING_ANALYSIS.md'
cp 'recovered_work/44b8ee68/yoRB.md' 'docs/agents.md'
cp 'recovered_work/-2ed692f6/xfmw.md' 'docs/agents/examples.md'
cp 'recovered_work/-1f8c748b/Wpv9.md' 'docs/agents/full-guide.md'
cp 'recovered_work/-1d03762b/BDlY.md' 'docs/agents/tutorial.md'
cp 'recovered_work/-4963d78e/sERb.md' 'docs/database/README.md'
cp 'recovered_work/3e28b829/kpTc.md' 'docs/monitoring-alerting.md'
cp 'recovered_work/-2b577aa8/v4iN.rs' 'error-handling-validation.rs'
cp 'recovered_work/-36a01474/SF3L.rs' 'integration-test-full-pipeline.rs'
cp 'recovered_work/1e65b546/DlgE.rs' 'integration-test-optimizations.rs'
cp 'recovered_work/-51b94ba/gGRJ.js' 'iterations/poc/apps/tools/caws/dashboard.js'
cp 'recovered_work/8276642/gRLS.ts' 'iterations/poc/apps/tools/caws/security-provenance.ts'
cp 'recovered_work/76944893/lsU7.ts' 'iterations/poc/apps/tools/caws/shared/gate-checker.ts'
cp 'recovered_work/-12103690/rPi1.js' 'iterations/poc/apps/tools/caws/test-quality.js'
cp 'recovered_work/-3c01e5c6/fv6N.ts' 'iterations/poc/run-test.ts'
cp 'recovered_work/5396573/D4XX.ts' 'iterations/poc/src/mcp/evaluation/evaluators/DesignEvaluator.ts'
cp 'recovered_work/60ea5777/0UJX.ts' 'iterations/poc/src/mcp/resources/ResourceManager.ts'
cp 'recovered_work/-7782a806/DuTY.ts' 'iterations/poc/src/mcp/tools/categories/AgentManagementTools.ts'
cp 'recovered_work/-d85bb3f/T28J.ts' 'iterations/poc/test-runner.ts'
cp 'recovered_work/-5fbe811f/IWuT' 'iterations/v2/.swcrc'
cp 'recovered_work/4b8f3d9c/ctn7.js' 'iterations/v2/apps/web-observer/next.config.js'
cp 'recovered_work/2cdd4997/NG5K.mjs' 'iterations/v2/apps/web-observer/postcss.config.mjs'
cp 'recovered_work/1e6950a/DJAw.css' 'iterations/v2/apps/web-observer/src/app/globals.css'
cp 'recovered_work/40378a48/jCVN.tsx' 'iterations/v2/apps/web-observer/src/components/AlertManager.tsx'
cp 'recovered_work/-14e62825/KUsh.tsx' 'iterations/v2/apps/web-observer/src/components/DatabaseAuditPanel.tsx'
cp 'recovered_work/-61b419f8/SOig.tsx' 'iterations/v2/apps/web-observer/src/components/DebugPanel.tsx'
cp 'recovered_work/-7974ea65/RpVE.tsx' 'iterations/v2/apps/web-observer/src/components/HealthCheckRunner.tsx'
cp 'recovered_work/-1878bd76/39oa.tsx' 'iterations/v2/apps/web-observer/src/components/ObservabilityDashboard.tsx'
cp 'recovered_work/1c87b521/hcz2.tsx' 'iterations/v2/apps/web-observer/src/components/PerformanceMonitor.tsx'
cp 'recovered_work/45264631/FCU7.tsx' 'iterations/v2/apps/web-observer/src/components/TaskTraceViewer.tsx'
cp 'recovered_work/5107b3c1/pK9y.js' 'iterations/v2/apps/web-observer/tailwind.config.js'
cp 'recovered_work/27ae8948/4tDQ.md' 'iterations/v2/docs/3-agent-rl-training/comprehensive-improvement-summary.md'
cp 'recovered_work/77cbb5ab/GLpN.md' 'iterations/v2/docs/3-agent-rl-training/implementation-roadmap.md'
cp 'recovered_work/65cfbbc9/Pb2B.md' 'iterations/v2/docs/3-agent-rl-training/technical-architecture.md'
cp 'recovered_work/-58d8e0a/7RVv.md' 'iterations/v2/docs/3-agent-rl-training/v2-agentic-rl-roadmap.md'
cp 'recovered_work/-3a5fb931/TBYh.md' 'iterations/v2/docs/feature.plan.md'
cp 'recovered_work/-1d687735/AMTT.md' 'iterations/v2/docs/templates/COMPONENT_STATUS_TEMPLATE.md'
cp 'recovered_work/5c8aa203/PROj.py' 'iterations/v2/python-services/dspy-integration/parallel_optimization.py'
cp 'recovered_work/796cee9/O6ZU.toml' 'iterations/v2/python-services/dspy-integration/pyproject.toml'
cp 'recovered_work/335afeb3/OGbC.ts' 'iterations/v2/src/config/validation/ConfigValidationError.ts'
cp 'recovered_work/-4a9615a/3PbF.json' 'iterations/v2/tests/fixtures/caws-integration/package.json'
cp 'recovered_work/6a8acfa7/JD1l.yml' 'iterations/v3/.github/workflows/ci.yml'
cp 'recovered_work/7acc1813/kNqR.yml' 'iterations/v3/.github/workflows/test.yml'
cp 'recovered_work/37b790a5/FzFW.md' 'iterations/v3/DEPLOYMENT_GUIDE.md'
cp 'recovered_work/-51ec31b4/bQJr.md' 'iterations/v3/FINAL_PROJECT_SUMMARY.md'
cp 'recovered_work/27f7ca7d/kaCn.md' 'iterations/v3/KNOWLEDGE_BASE_IMPLEMENTATION_SUMMARY.md'
cp 'recovered_work/79e3d4fd/Shu0.md' 'iterations/v3/KNOWLEDGE_BASE_TESTING_SUMMARY.md'
cp 'recovered_work/315aa96b/k3Ln.md' 'iterations/v3/MULTIMODAL_RAG_INTEGRATION_SPEC.md'
cp 'recovered_work/41223ea4/7PiY.md' 'iterations/v3/README_V3_IMPLEMENTATION.md'
cp 'recovered_work/56d6a667/ozwv.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_10_COMPLETE.md'
cp 'recovered_work/4559ace8/HCV1.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_11_COMPLETE.md'
cp 'recovered_work/33dcb369/39wo.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_12_COMPLETE.md'
cp 'recovered_work/225fb9ea/hO6h.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_13_COMPLETE.md'
cp 'recovered_work/10e2c06b/Ujsd.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_14_COMPLETE.md'
cp 'recovered_work/-528abf9c/rUUH.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_2_COMPLETE.md'
cp 'recovered_work/-6407b91b/K9Xw.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_3_COMPLETE.md'
cp 'recovered_work/-7584b29a/aIFd.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_4_COMPLETE.md'
cp 'recovered_work/78fe53e7/wUAJ.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_5_COMPLETE.md'
cp 'recovered_work/67815a68/O4Qz.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_6_COMPLETE.md'
cp 'recovered_work/560460e9/T6KF.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_7_COMPLETE.md'
cp 'recovered_work/4487676a/4SCn.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_8_COMPLETE.md'
cp 'recovered_work/330a6deb/QbHm.md' 'iterations/v3/TODO_IMPLEMENTATION_SESSION_9_COMPLETE.md'
cp 'recovered_work/3a20569c/uMaL.toml' 'iterations/v3/agent-agency-contracts/Cargo.toml'
cp 'recovered_work/-3c6e321e/lv7c.rs' 'iterations/v3/agent-agency-contracts/src/error.rs'
cp 'recovered_work/c7de2f0/pvko.rs' 'iterations/v3/agent-agency-contracts/src/execution_artifacts.rs'
cp 'recovered_work/35fa1305/FZ6X.rs' 'iterations/v3/agent-agency-contracts/src/lib.rs'
cp 'recovered_work/-6918c5a0/04hm.rs' 'iterations/v3/agent-agency-contracts/src/quality_report.rs'
cp 'recovered_work/220cab5a/3v2C.rs' 'iterations/v3/agent-agency-contracts/src/refinement_decision.rs'
cp 'recovered_work/-6e0aaced/EiBx.rs' 'iterations/v3/agent-agency-contracts/src/schema.rs'
cp 'recovered_work/-3fb22d61/gu8P.rs' 'iterations/v3/agent-agency-contracts/src/task_request.rs'
cp 'recovered_work/730f7e4f/AwrH.rs' 'iterations/v3/agent-agency-contracts/src/task_response.rs'
cp 'recovered_work/-43d92c15/ABjx.rs' 'iterations/v3/agent-agency-contracts/src/working_spec.rs'
cp 'recovered_work/44be1cd/pBHv.toml' 'iterations/v3/api-server-config.toml'
cp 'recovered_work/-658c5c39/8SiF.toml' 'iterations/v3/api-server/Cargo.toml'
cp 'recovered_work/-3e27a6ae/4uNG.rs' 'iterations/v3/api-server/src/alerts.rs'
cp 'recovered_work/-6feadb81/sk4q.rs' 'iterations/v3/api-server/src/circuit_breaker.rs'
cp 'recovered_work/5b1cd970/aWbq.rs' 'iterations/v3/api-server/src/main.rs'
cp 'recovered_work/-30ca6e20/pgYn.rs' 'iterations/v3/api-server/src/rate_limiter.rs'
cp 'recovered_work/59adc75b/Pxg8.rs' 'iterations/v3/api-server/src/rto_rpo_monitor.rs'
cp 'recovered_work/500b5a0d/nEln.rs' 'iterations/v3/api-server/src/service_failover.rs'
cp 'recovered_work/12333fc6/7JgW.yaml' 'iterations/v3/apple-silicon/.caws/working-spec.yaml'
cp 'recovered_work/5781cc1/Z99H.md' 'iterations/v3/apple-silicon/BLOCKED_FRAMEWORK_INTEGRATIONS.md'
cp 'recovered_work/-4777a22e/x6tq.rs' 'iterations/v3/apple-silicon/build.rs'
cp 'recovered_work/5af89b19/ByXP.rs' 'iterations/v3/apple-silicon/src/ane/ffi.rs'
cp 'recovered_work/3d668c91/KchQ.rs' 'iterations/v3/apple-silicon/src/ane/filesystem.rs'
cp 'recovered_work/-10d308eb/YfmO.rs' 'iterations/v3/apple-silicon/src/ane/manager.rs'
cp 'recovered_work/67671520/8HmH.rs' 'iterations/v3/apple-silicon/src/ane/mod.rs'
cp 'recovered_work/-5ccf6150/QGOY.rs' 'iterations/v3/apple-silicon/src/buffer_pool.rs'
cp 'recovered_work/-7254ef1d/Kh0B.rs' 'iterations/v3/apple-silicon/src/enhanced_telemetry.rs'
cp 'recovered_work/46a822e1/m0FI.rs' 'iterations/v3/apple-silicon/src/memory/analysis.rs'
cp 'recovered_work/73e5bcbb/HKpw.rs' 'iterations/v3/apple-silicon/src/memory/compression.rs'
cp 'recovered_work/7cbbd614/qxeq.rs' 'iterations/v3/apple-silicon/src/memory/manager.rs'
cp 'recovered_work/-3c537ac2/LJeu.rs' 'iterations/v3/apple-silicon/src/memory/metrics.rs'
cp 'recovered_work/23034b9f/KclL.rs' 'iterations/v3/apple-silicon/src/memory/mod.rs'
cp 'recovered_work/-8b88124/yqoZ.rs' 'iterations/v3/apple-silicon/src/memory/quantization.rs'
cp 'recovered_work/3f0fce94/8d0r.rs' 'iterations/v3/apple-silicon/src/model_router.rs'
cp 'recovered_work/5d3afeec/gW33.rs' 'iterations/v3/apple-silicon/src/operator_fusion.rs'
cp 'recovered_work/22dbbda4/gW1M.rs' 'iterations/v3/apple-silicon/src/quantization_lab.rs'
cp 'recovered_work/-9d310fb/MojQ.rs' 'iterations/v3/apple-silicon/src/speech_bridge.rs'
cp 'recovered_work/-6500b818/nWl5.rs' 'iterations/v3/apple-silicon/src/tokenization.rs'
cp 'recovered_work/13bfb28b/zm47.rs' 'iterations/v3/apple-silicon/src/vision_bridge.rs'
cp 'recovered_work/-5e96d178/wTR3.rs' 'iterations/v3/apple-silicon/tests/phase3b_inference_cycles.rs'
cp 'recovered_work/-233a36d5/bxwq.ts' 'iterations/v3/apps/tools/caws/__tests__/security-provenance.test.ts'
cp 'recovered_work/-5a438b9/juh5.js' 'iterations/v3/apps/tools/caws/ci-optimizer.js'
cp 'recovered_work/92180cb/cRty.backup' 'iterations/v3/apps/tools/caws/dashboard.js.backup'
cp 'recovered_work/-13ac77b7/QSxf.ts' 'iterations/v3/apps/tools/caws/legacy-assessment.ts'
cp 'recovered_work/-3b642993/Yo1B.md' 'iterations/v3/apps/tools/caws/modules/README.md'
cp 'recovered_work/30433eaa/Ofx5.js' 'iterations/v3/apps/tools/caws/modules/compliance-checker.js'
cp 'recovered_work/-596a5520/2XRF.js' 'iterations/v3/apps/tools/caws/modules/coverage-analysis.js'
cp 'recovered_work/-70e41e3/Ig49.js' 'iterations/v3/apps/tools/caws/modules/data-generator.js'
cp 'recovered_work/-594ad371/gMV8.js' 'iterations/v3/apps/tools/caws/modules/index.js'
cp 'recovered_work/-435c51df/3eMR.js' 'iterations/v3/apps/tools/caws/modules/mutation-analysis.js'
cp 'recovered_work/-1fddd076/so4y.js' 'iterations/v3/apps/tools/caws/modules/test-analysis.js'
cp 'recovered_work/-73d2049d/IDZN.md' 'iterations/v3/apps/tools/caws/workflow-modules/README.md'
cp 'recovered_work/4b62c10e/11rM.js' 'iterations/v3/apps/tools/caws/workflow-modules/build-jobs.js'
cp 'recovered_work/-29906f27/WqVr.js' 'iterations/v3/apps/tools/caws/workflow-modules/index.js'
cp 'recovered_work/182e79f/wd64.js' 'iterations/v3/apps/tools/caws/workflow-modules/quality-jobs.js'
cp 'recovered_work/-70ee2994/b1nN.js' 'iterations/v3/apps/tools/caws/workflow-modules/workflow-base.js'
cp 'recovered_work/-318d7b07/8ZsE.json' 'iterations/v3/apps/web-dashboard/.eslintrc.json'
cp 'recovered_work/-334d2817/tm5q' 'iterations/v3/apps/web-dashboard/.gitignore'
cp 'recovered_work/-23c40e21/N2nC' 'iterations/v3/apps/web-dashboard/.prettierrc'
cp 'recovered_work/-5fd7dbf9/56Yo.md' 'iterations/v3/apps/web-dashboard/ENVIRONMENT_SETUP.md'
cp 'recovered_work/5bccf744/dHDP.md' 'iterations/v3/apps/web-dashboard/README.md'
cp 'recovered_work/2235a5f7/Cays.ts' 'iterations/v3/apps/web-dashboard/cypress.config.ts'
cp 'recovered_work/40952d2/4b3s.ts' 'iterations/v3/apps/web-dashboard/cypress/e2e/self-prompting.cy.ts'
cp 'recovered_work/-229aa86a/N0fb.example' 'iterations/v3/apps/web-dashboard/env.example'
cp 'recovered_work/-18edb46a/ZaTB.js' 'iterations/v3/apps/web-dashboard/jest.config.js'
cp 'recovered_work/-782269e7/4qgq.js' 'iterations/v3/apps/web-dashboard/jest.setup.js'
cp 'recovered_work/-6afe5313/0LuB.js' 'iterations/v3/apps/web-dashboard/next.config.js'
cp 'recovered_work/-584ff6fd/wg4H.json' 'iterations/v3/apps/web-dashboard/package.json'
cp 'recovered_work/5f6b0517/r0Kj.js' 'iterations/v3/apps/web-dashboard/postcss.config.js'
cp 'recovered_work/5457528e/c9Km.ts' 'iterations/v3/apps/web-dashboard/src/app/api/alerts/%5BalertId%5D/acknowledge/route.ts'
cp 'recovered_work/15c9a0be/ZWv4.ts' 'iterations/v3/apps/web-dashboard/src/app/api/alerts/%5BalertId%5D/resolve/route.ts'
cp 'recovered_work/a2d6adb/k5ve.ts' 'iterations/v3/apps/web-dashboard/src/app/api/alerts/route.ts'
cp 'recovered_work/-3806e747/89QX.ts' 'iterations/v3/apps/web-dashboard/src/app/api/alerts/statistics/route.ts'
cp 'recovered_work/-47538dd2/ogRv.ts' 'iterations/v3/apps/web-dashboard/src/app/api/analytics/route.ts'
cp 'recovered_work/-6fe7c1c5/yk42.ts' 'iterations/v3/apps/web-dashboard/src/app/api/chat/ws/%5BsessionId%5D/route.ts'
cp 'recovered_work/69128fb1/jWP0.ts' 'iterations/v3/apps/web-dashboard/src/app/api/database/connections/route.ts'
cp 'recovered_work/-43464822/Cupn.ts' 'iterations/v3/apps/web-dashboard/src/app/api/database/query/route.ts'
cp 'recovered_work/-47899d87/n6Ui.ts' 'iterations/v3/apps/web-dashboard/src/app/api/database/tables/%5BtableName%5D/schema/route.ts'
cp 'recovered_work/-465dd325/1XJv.ts' 'iterations/v3/apps/web-dashboard/src/app/api/database/tables/route.ts'
cp 'recovered_work/-501c008c/5brh.ts' 'iterations/v3/apps/web-dashboard/src/app/api/database/vector-search/route.ts'
cp 'recovered_work/-1541074a/nlbP.ts' 'iterations/v3/apps/web-dashboard/src/app/api/health/route.ts'
cp 'recovered_work/4df4dd1/jczs.ts' 'iterations/v3/apps/web-dashboard/src/app/api/metrics/route.ts'
cp 'recovered_work/-35f2c43a/p4f9.ts' 'iterations/v3/apps/web-dashboard/src/app/api/metrics/stream/route.ts'
cp 'recovered_work/-3c22c85e/KLjx.ts' 'iterations/v3/apps/web-dashboard/src/app/api/proxy/%5B...path%5D/route.ts'
cp 'recovered_work/-57d944c9/Kqqb.ts' 'iterations/v3/apps/web-dashboard/src/app/api/slo-alerts/%5BalertId%5D/acknowledge/route.ts'
cp 'recovered_work/1555d4c4/bD8v.ts' 'iterations/v3/apps/web-dashboard/src/app/api/slo-alerts/route.ts'
cp 'recovered_work/-73caf9f/zQ66.ts' 'iterations/v3/apps/web-dashboard/src/app/api/slos/%5BsloName%5D/measurements/route.ts'
cp 'recovered_work/-3118493a/zteR.ts' 'iterations/v3/apps/web-dashboard/src/app/api/slos/%5BsloName%5D/status/route.ts'
cp 'recovered_work/65811f5/nXIj.ts' 'iterations/v3/apps/web-dashboard/src/app/api/slos/route.ts'
cp 'recovered_work/9433912/EQM4.ts' 'iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D/action/route.ts'
cp 'recovered_work/77daaa9/xU6w.ts' 'iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D/arbiter-verdict/route.ts'
cp 'recovered_work/-22efa384/LpKU.ts' 'iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D/claim-verification/route.ts'
cp 'recovered_work/411914cf/1i0l.ts' 'iterations/v3/apps/web-dashboard/src/app/api/tasks/%5BtaskId%5D/route.ts'
cp 'recovered_work/392766d8/tdEK.ts' 'iterations/v3/apps/web-dashboard/src/app/api/tasks/events/route.ts'
cp 'recovered_work/-f00fc5a/GHQB.ts' 'iterations/v3/apps/web-dashboard/src/app/api/tasks/route.ts'
cp 'recovered_work/-1d4e54df/wQp9.ts' 'iterations/v3/apps/web-dashboard/src/app/api/tts/route.ts'
cp 'recovered_work/-4890fae6/0UoV.scss' 'iterations/v3/apps/web-dashboard/src/app/globals.scss'
cp 'recovered_work/4dbfe281/6KYI.tsx' 'iterations/v3/apps/web-dashboard/src/app/layout.tsx'
cp 'recovered_work/-6fb4a581/bcnS.scss' 'iterations/v3/apps/web-dashboard/src/app/page.module.scss'
cp 'recovered_work/7cbe7a66/647Q.tsx' 'iterations/v3/apps/web-dashboard/src/app/page.tsx'
cp 'recovered_work/-536e88ed/mRPW.scss' 'iterations/v3/apps/web-dashboard/src/app/tasks/%5BtaskId%5D/page.module.scss'
cp 'recovered_work/2e646afa/JO1A.tsx' 'iterations/v3/apps/web-dashboard/src/app/tasks/%5BtaskId%5D/page.tsx'
cp 'recovered_work/16046de0/UtH4.scss' 'iterations/v3/apps/web-dashboard/src/app/tasks/page.module.scss'
cp 'recovered_work/41cd7ec7/sukC.tsx' 'iterations/v3/apps/web-dashboard/src/app/tasks/page.tsx'
cp 'recovered_work/7d130356/tJ7j.scss' 'iterations/v3/apps/web-dashboard/src/components/analytics/AnalyticsDashboard.module.scss'
cp 'recovered_work/-4b1415c3/1D7Z.tsx' 'iterations/v3/apps/web-dashboard/src/components/analytics/AnalyticsDashboard.tsx'
cp 'recovered_work/-6cded1b7/CLQn.scss' 'iterations/v3/apps/web-dashboard/src/components/analytics/AnomalyDetector.module.scss'
cp 'recovered_work/4621b830/Ysop.tsx' 'iterations/v3/apps/web-dashboard/src/components/analytics/AnomalyDetector.tsx'
cp 'recovered_work/7823e843/aPDF.scss' 'iterations/v3/apps/web-dashboard/src/components/analytics/CorrelationMatrix.module.scss'
cp 'recovered_work/-eacf3d6/zOqZ.tsx' 'iterations/v3/apps/web-dashboard/src/components/analytics/CorrelationMatrix.tsx'
cp 'recovered_work/-5948f2c1/nWE5.scss' 'iterations/v3/apps/web-dashboard/src/components/analytics/ForecastingChart.module.scss'
cp 'recovered_work/3068ed26/PqlQ.tsx' 'iterations/v3/apps/web-dashboard/src/components/analytics/ForecastingChart.tsx'
cp 'recovered_work/7b7acdd4/Cb7S.scss' 'iterations/v3/apps/web-dashboard/src/components/analytics/PerformancePredictor.module.scss'
cp 'recovered_work/-6b2ed45/2xOS.tsx' 'iterations/v3/apps/web-dashboard/src/components/analytics/PerformancePredictor.tsx'
cp 'recovered_work/-58cc343d/sMTo.scss' 'iterations/v3/apps/web-dashboard/src/components/analytics/TrendAnalyzer.module.scss'
cp 'recovered_work/-5dc19056/UMDv.tsx' 'iterations/v3/apps/web-dashboard/src/components/analytics/TrendAnalyzer.tsx'
cp 'recovered_work/1129cdb7/gNKT.scss' 'iterations/v3/apps/web-dashboard/src/components/arbiter/ArbiterVerdictPanel.module.scss'
cp 'recovered_work/-1f005a62/Un55.tsx' 'iterations/v3/apps/web-dashboard/src/components/arbiter/ArbiterVerdictPanel.tsx'
cp 'recovered_work/6997c772/MrpO.scss' 'iterations/v3/apps/web-dashboard/src/components/arbiter/ClaimVerificationPanel.module.scss'
cp 'recovered_work/-336b75a7/QL6z.tsx' 'iterations/v3/apps/web-dashboard/src/components/arbiter/ClaimVerificationPanel.tsx'
cp 'recovered_work/224e06f9/57z8.tsx' 'iterations/v3/apps/web-dashboard/src/components/arbiter/CliInterventionPanel.tsx'
cp 'recovered_work/4223849d/UioT.tsx' 'iterations/v3/apps/web-dashboard/src/components/arbiter/DebateVisualization.tsx'
cp 'recovered_work/-1e174aa4/6IB3.ts' 'iterations/v3/apps/web-dashboard/src/components/arbiter/index.ts'
cp 'recovered_work/22b6efc9/wyhr.scss' 'iterations/v3/apps/web-dashboard/src/components/chat/ChatInterface.module.scss'
cp 'recovered_work/-daf0650/6alP.tsx' 'iterations/v3/apps/web-dashboard/src/components/chat/ChatInterface.tsx'
cp 'recovered_work/6e43d570/4Und.scss' 'iterations/v3/apps/web-dashboard/src/components/chat/ConnectionStatus.module.scss'
cp 'recovered_work/-3c8a89a9/aB5P.tsx' 'iterations/v3/apps/web-dashboard/src/components/chat/ConnectionStatus.tsx'
cp 'recovered_work/-11cbdeeb/cFFN.scss' 'iterations/v3/apps/web-dashboard/src/components/chat/ContextPanel.module.scss'
cp 'recovered_work/340a36fc/zDxe.tsx' 'iterations/v3/apps/web-dashboard/src/components/chat/ContextPanel.tsx'
cp 'recovered_work/-54198885/6azl.scss' 'iterations/v3/apps/web-dashboard/src/components/chat/MessageBubble.module.scss'
cp 'recovered_work/4c545362/PWGu.tsx' 'iterations/v3/apps/web-dashboard/src/components/chat/MessageBubble.tsx'
cp 'recovered_work/2e622683/ixcX.scss' 'iterations/v3/apps/web-dashboard/src/components/chat/MessageInput.module.scss'
cp 'recovered_work/67d38a6a/qAO2.tsx' 'iterations/v3/apps/web-dashboard/src/components/chat/MessageInput.tsx'
cp 'recovered_work/-6e6b58b3/hxAG.scss' 'iterations/v3/apps/web-dashboard/src/components/chat/MessageList.module.scss'
cp 'recovered_work/-446f8acc/qlMx.tsx' 'iterations/v3/apps/web-dashboard/src/components/chat/MessageList.tsx'
cp 'recovered_work/-464befd1/QtJF.scss' 'iterations/v3/apps/web-dashboard/src/components/chat/VoiceChatInterface.module.scss'
cp 'recovered_work/-27b01fea/ovAY.tsx' 'iterations/v3/apps/web-dashboard/src/components/chat/VoiceChatInterface.tsx'
cp 'recovered_work/74f26c5c/5lsE.scss' 'iterations/v3/apps/web-dashboard/src/components/database/DataQualityDashboard.module.scss'
cp 'recovered_work/106fb943/Lv6O.tsx' 'iterations/v3/apps/web-dashboard/src/components/database/DataQualityDashboard.tsx'
cp 'recovered_work/4b56fb17/hCZN.scss' 'iterations/v3/apps/web-dashboard/src/components/database/DatabaseExplorer.module.scss'
cp 'recovered_work/306032fe/cbRs.tsx' 'iterations/v3/apps/web-dashboard/src/components/database/DatabaseExplorer.tsx'
cp 'recovered_work/-1765a310/Ncou.scss' 'iterations/v3/apps/web-dashboard/src/components/database/QueryBuilder.module.scss'
cp 'recovered_work/-4d748229/5itu.tsx' 'iterations/v3/apps/web-dashboard/src/components/database/QueryBuilder.tsx'
cp 'recovered_work/27ad04ab/5j4O.scss' 'iterations/v3/apps/web-dashboard/src/components/database/TableViewer.module.scss'
cp 'recovered_work/53531092/r5Wp.tsx' 'iterations/v3/apps/web-dashboard/src/components/database/TableViewer.tsx'
cp 'recovered_work/7ada8264/FHy2.scss' 'iterations/v3/apps/web-dashboard/src/components/database/VectorSearchPanel.module.scss'
cp 'recovered_work/-664da8b5/CSX3.tsx' 'iterations/v3/apps/web-dashboard/src/components/database/VectorSearchPanel.tsx'
cp 'recovered_work/4b6559dc/k5Hw.scss' 'iterations/v3/apps/web-dashboard/src/components/metrics/MetricsDashboard.module.scss'
cp 'recovered_work/196826c3/FHnR.tsx' 'iterations/v3/apps/web-dashboard/src/components/metrics/MetricsDashboard.tsx'
cp 'recovered_work/-6d77175f/0qA2.scss' 'iterations/v3/apps/web-dashboard/src/components/monitoring/AgentPerformanceGrid.module.scss'
cp 'recovered_work/-764ab578/3to2.tsx' 'iterations/v3/apps/web-dashboard/src/components/monitoring/AgentPerformanceGrid.tsx'
cp 'recovered_work/2532a275/51dG.scss' 'iterations/v3/apps/web-dashboard/src/components/monitoring/AlertsDashboard.module.scss'
cp 'recovered_work/40d0185c/ROuT.tsx' 'iterations/v3/apps/web-dashboard/src/components/monitoring/AlertsDashboard.tsx'
cp 'recovered_work/3967346f/uqPK.scss' 'iterations/v3/apps/web-dashboard/src/components/monitoring/BusinessIntelligence.module.scss'
cp 'recovered_work/-974bbaa/1Hil.tsx' 'iterations/v3/apps/web-dashboard/src/components/monitoring/BusinessIntelligence.tsx'
cp 'recovered_work/447dd620/99s6.scss' 'iterations/v3/apps/web-dashboard/src/components/monitoring/CoordinationMetrics.module.scss'
cp 'recovered_work/-34acd8f9/9HCc.tsx' 'iterations/v3/apps/web-dashboard/src/components/monitoring/CoordinationMetrics.tsx'
cp 'recovered_work/7208fece/cWCT.scss' 'iterations/v3/apps/web-dashboard/src/components/monitoring/MetricTile.module.scss'
cp 'recovered_work/-78f224b/YARH.tsx' 'iterations/v3/apps/web-dashboard/src/components/monitoring/MetricTile.tsx'
cp 'recovered_work/-54103f89/ZhHP.tsx' 'iterations/v3/apps/web-dashboard/src/components/monitoring/RealTimeMetricsStream.tsx'
cp 'recovered_work/76ee2677/6MuP.scss' 'iterations/v3/apps/web-dashboard/src/components/monitoring/SLOAlertsDashboard.module.scss'
cp 'recovered_work/-3df741a2/vtnn.tsx' 'iterations/v3/apps/web-dashboard/src/components/monitoring/SLOAlertsDashboard.tsx'
cp 'recovered_work/-3ad27ab2/TTiR.scss' 'iterations/v3/apps/web-dashboard/src/components/monitoring/SLODashboard.module.scss'
cp 'recovered_work/-6e3c1bcb/XqiW.tsx' 'iterations/v3/apps/web-dashboard/src/components/monitoring/SLODashboard.tsx'
cp 'recovered_work/4bf4ca54/IZ0l.scss' 'iterations/v3/apps/web-dashboard/src/components/monitoring/SystemHealthOverview.module.scss'
cp 'recovered_work/-203470c5/hRHy.tsx' 'iterations/v3/apps/web-dashboard/src/components/monitoring/SystemHealthOverview.tsx'
cp 'recovered_work/-7b99e4ec/omqo.scss' 'iterations/v3/apps/web-dashboard/src/components/monitoring/SystemResourcesMonitor.module.scss'
cp 'recovered_work/-54066005/BsZp.tsx' 'iterations/v3/apps/web-dashboard/src/components/monitoring/SystemResourcesMonitor.tsx'
cp 'recovered_work/-742a91d6/vYmU.scss' 'iterations/v3/apps/web-dashboard/src/components/shared/AttentionAlerts.module.scss'
cp 'recovered_work/606911/d96M.tsx' 'iterations/v3/apps/web-dashboard/src/components/shared/AttentionAlerts.tsx'
cp 'recovered_work/-4ca2b9bd/I8dN.scss' 'iterations/v3/apps/web-dashboard/src/components/shared/ConnectionStatus.module.scss'
cp 'recovered_work/72ca6a2a/Dasa.tsx' 'iterations/v3/apps/web-dashboard/src/components/shared/ConnectionStatus.tsx'
cp 'recovered_work/4bb36fa0/SycE.scss' 'iterations/v3/apps/web-dashboard/src/components/shared/Header.module.scss'
cp 'recovered_work/59a5d48f/TJTw.tsx' 'iterations/v3/apps/web-dashboard/src/components/shared/Header.test.tsx'
cp 'recovered_work/187a4087/W5Uy.tsx' 'iterations/v3/apps/web-dashboard/src/components/shared/Header.tsx'
cp 'recovered_work/-db75ad9/B4Zu.scss' 'iterations/v3/apps/web-dashboard/src/components/shared/Navigation.module.scss'
cp 'recovered_work/1e64ed0e/8Bmb.tsx' 'iterations/v3/apps/web-dashboard/src/components/shared/Navigation.tsx'
cp 'recovered_work/-3d4f98e8/6f2R.tsx' 'iterations/v3/apps/web-dashboard/src/components/shared/SimpleTest.test.tsx'
cp 'recovered_work/-5fe19ba9/Dj4F.scss' 'iterations/v3/apps/web-dashboard/src/components/shared/SystemHealthOverview.module.scss'
cp 'recovered_work/-153b23c2/znXL.tsx' 'iterations/v3/apps/web-dashboard/src/components/shared/SystemHealthOverview.tsx'
cp 'recovered_work/67a3cf0b/NLL5.scss' 'iterations/v3/apps/web-dashboard/src/components/shared/TTSSettings.module.scss'
cp 'recovered_work/-3995c50e/fMBq.tsx' 'iterations/v3/apps/web-dashboard/src/components/shared/TTSSettings.tsx'
cp 'recovered_work/944bfbe/2n0z.scss' 'iterations/v3/apps/web-dashboard/src/components/shared/VoicemailHistory.module.scss'
cp 'recovered_work/-9cb715b/tWaI.tsx' 'iterations/v3/apps/web-dashboard/src/components/shared/VoicemailHistory.tsx'
cp 'recovered_work/215aa37d/9Sjw.scss' 'iterations/v3/apps/web-dashboard/src/components/tasks/AuditTrailViewer.module.scss'
cp 'recovered_work/-17925e9c/ysMG.tsx' 'iterations/v3/apps/web-dashboard/src/components/tasks/AuditTrailViewer.tsx'
cp 'recovered_work/73c78ea6/KPxc.scss' 'iterations/v3/apps/web-dashboard/src/components/tasks/IterationTimeline.module.scss'
cp 'recovered_work/-18b73a73/IJLQ.tsx' 'iterations/v3/apps/web-dashboard/src/components/tasks/IterationTimeline.tsx'
cp 'recovered_work/-33089841/oY1v.scss' 'iterations/v3/apps/web-dashboard/src/components/tasks/ModelPerformanceChart.module.scss'
cp 'recovered_work/-7014385a/tDi6.tsx' 'iterations/v3/apps/web-dashboard/src/components/tasks/ModelPerformanceChart.tsx'
cp 'recovered_work/-68778f9e/eAFQ.scss' 'iterations/v3/apps/web-dashboard/src/components/tasks/SatisficingDashboard.module.scss'
cp 'recovered_work/38752349/f3zj.tsx' 'iterations/v3/apps/web-dashboard/src/components/tasks/SatisficingDashboard.tsx'
cp 'recovered_work/-8bab8f8/vduQ.scss' 'iterations/v3/apps/web-dashboard/src/components/tasks/SelfPromptingMonitor.module.scss'
cp 'recovered_work/1c6dffef/JrE2.tsx' 'iterations/v3/apps/web-dashboard/src/components/tasks/SelfPromptingMonitor.tsx'
cp 'recovered_work/561b7cd5/eFou.scss' 'iterations/v3/apps/web-dashboard/src/components/tasks/TaskCard.module.scss'
cp 'recovered_work/7fe952bc/fgik.tsx' 'iterations/v3/apps/web-dashboard/src/components/tasks/TaskCard.tsx'
cp 'recovered_work/-4464de2/UUG5.scss' 'iterations/v3/apps/web-dashboard/src/components/tasks/TaskFilters.module.scss'
cp 'recovered_work/-f8e1efb/kAHV.tsx' 'iterations/v3/apps/web-dashboard/src/components/tasks/TaskFilters.tsx'
cp 'recovered_work/27a6bb1d/TUJc.scss' 'iterations/v3/apps/web-dashboard/src/components/tasks/TaskFiltersBar.module.scss'
cp 'recovered_work/-2a64a6fc/W1lV.tsx' 'iterations/v3/apps/web-dashboard/src/components/tasks/TaskFiltersBar.tsx'
cp 'recovered_work/-22da329d/MQah.scss' 'iterations/v3/apps/web-dashboard/src/components/tasks/TaskList.module.scss'
cp 'recovered_work/-2f63eeb6/QEfY.tsx' 'iterations/v3/apps/web-dashboard/src/components/tasks/TaskList.tsx'
cp 'recovered_work/76b4e2e6/nqso.scss' 'iterations/v3/apps/web-dashboard/src/components/tasks/TaskMetrics.module.scss'
cp 'recovered_work/-4211a633/JpyK.tsx' 'iterations/v3/apps/web-dashboard/src/components/tasks/TaskMetrics.tsx'
cp 'recovered_work/2362346e/2nIV.ts' 'iterations/v3/apps/web-dashboard/src/hooks/useArbiter.ts'
cp 'recovered_work/3e2bc24/Uu2X.ts' 'iterations/v3/apps/web-dashboard/src/hooks/useTTS.ts'
cp 'recovered_work/-40157473/3LIt.ts' 'iterations/v3/apps/web-dashboard/src/hooks/useVoiceChat.ts'
cp 'recovered_work/-44dfa41c/lmY1.ts' 'iterations/v3/apps/web-dashboard/src/hooks/useVoiceRecording.ts'
cp 'recovered_work/-1abaf47e/caDq.ts' 'iterations/v3/apps/web-dashboard/src/lib/analytics-api.ts'
cp 'recovered_work/-7af58bf9/cJAj.ts' 'iterations/v3/apps/web-dashboard/src/lib/api-client.ts'
cp 'recovered_work/-11329e44/qtxa.ts' 'iterations/v3/apps/web-dashboard/src/lib/api/tasks.ts'
cp 'recovered_work/-5c2e4aa0/fZDJ.ts' 'iterations/v3/apps/web-dashboard/src/lib/chat-api.ts'
cp 'recovered_work/38e29ddd/aaIV.ts' 'iterations/v3/apps/web-dashboard/src/lib/database-api.ts'
cp 'recovered_work/-2d3f865b/G5KI.ts' 'iterations/v3/apps/web-dashboard/src/lib/metrics-api.ts'
cp 'recovered_work/-6e233091/zEuU.ts' 'iterations/v3/apps/web-dashboard/src/lib/sse/SSEClient.ts'
cp 'recovered_work/13ef7972/KFRU.ts' 'iterations/v3/apps/web-dashboard/src/lib/sse/TaskEventsClient.ts'
cp 'recovered_work/-611b496d/y9l9.ts' 'iterations/v3/apps/web-dashboard/src/lib/task-api.ts'
cp 'recovered_work/7f794675/7Kei.ts' 'iterations/v3/apps/web-dashboard/src/lib/tts-api.ts'
cp 'recovered_work/1ea1534b/asyB.ts' 'iterations/v3/apps/web-dashboard/src/lib/websocket/WebSocketClient.ts'
cp 'recovered_work/3ec7b4bd/JLuV.scss' 'iterations/v3/apps/web-dashboard/src/styles/globals.scss'
cp 'recovered_work/-34eaae3/svGv.scss' 'iterations/v3/apps/web-dashboard/src/styles/mixins.scss'
cp 'recovered_work/-75e80aea/B3IR.scss' 'iterations/v3/apps/web-dashboard/src/styles/variables.scss'
cp 'recovered_work/1e2f397b/sLVX.ts' 'iterations/v3/apps/web-dashboard/src/types/analytics.ts'
cp 'recovered_work/42acccc1/GMlG.ts' 'iterations/v3/apps/web-dashboard/src/types/chat.ts'
cp 'recovered_work/22cda1be/ETrm.ts' 'iterations/v3/apps/web-dashboard/src/types/database.ts'
cp 'recovered_work/-775e7be2/VpJU.ts' 'iterations/v3/apps/web-dashboard/src/types/metrics.ts'
cp 'recovered_work/-5b14198d/JOVz.ts' 'iterations/v3/apps/web-dashboard/src/types/tasks.ts'
cp 'recovered_work/-74cbe112/nAAF.ts' 'iterations/v3/apps/web-dashboard/src/types/tts.ts'
cp 'recovered_work/-4ddfed6e/4DUx.js' 'iterations/v3/apps/web-dashboard/tailwind.config.js'
cp 'recovered_work/29fa6d91/F3P0.js' 'iterations/v3/apps/web-dashboard/test-connection.js'
cp 'recovered_work/-4adb54de/CPY4.json' 'iterations/v3/apps/web-dashboard/tsconfig.json'
cp 'recovered_work/-5004f9d0/ugxX.swift' 'iterations/v3/asr-bridge/Package.swift'
cp 'recovered_work/6851dd96/pgFa.swift' 'iterations/v3/asr-bridge/Sources/ASRBridge/ASRBridge.swift'
cp 'recovered_work/ce370d3/oWXw.html' 'iterations/v3/assets/dashboard.html'
cp 'recovered_work/2ef51210/Oevo.rs' 'iterations/v3/bin/main.rs'
cp 'recovered_work/744f8ba7/lgWW.toml' 'iterations/v3/brittleness-test/Cargo.toml'
cp 'recovered_work/70294810/clzH.rs' 'iterations/v3/brittleness-test/src/lib.rs'
cp 'recovered_work/-65986a06/sltU.toml' 'iterations/v3/caching/Cargo.toml'
cp 'recovered_work/-4f968d77/gyfc.md' 'iterations/v3/caching/README.md'
cp 'recovered_work/-1d9269a9/zYYx.rs' 'iterations/v3/caching/examples/basic_usage.rs'
cp 'recovered_work/-736fa04c/YKFL.rs' 'iterations/v3/caching/src/integration.rs'
cp 'recovered_work/-69bead9d/iStT.rs' 'iterations/v3/caching/src/lib.rs'
cp 'recovered_work/2caa316/3S3A.bak' 'iterations/v3/claim-extraction/src/multi_modal_verification.rs.bak'
cp 'recovered_work/-667a176/Q3Gi.rs' 'iterations/v3/claim-extraction/tests/disambiguation_knowledge_test.rs'
cp 'recovered_work/-742f1f07/xYQ0.toml' 'iterations/v3/cli/Cargo.toml'
cp 'recovered_work/-6a98bd82/hadI.rs' 'iterations/v3/cli/src/main.rs'
cp 'recovered_work/7846242d/2gww.yaml' 'iterations/v3/config/production.yaml'
cp 'recovered_work/12f872e7/0lvn.rs' 'iterations/v3/context-preservation-engine/src/encryption_tests.rs'
cp 'recovered_work/-78d39d3/GPfh.rs' 'iterations/v3/council/build.rs'
cp 'recovered_work/-11b3ab63/Ez9f.rs' 'iterations/v3/council/src/claim_extraction_multimodal.rs'
cp 'recovered_work/-2c141edb/6ZUd.rs' 'iterations/v3/council/src/council.rs'
cp 'recovered_work/51caf4b8/Squb.rs' 'iterations/v3/council/src/decision_making.rs'
cp 'recovered_work/-7cca35e2/drMJ.rs' 'iterations/v3/council/src/error.rs'
cp 'recovered_work/-54e6d824/5b1G.rs' 'iterations/v3/council/src/error_handling.rs'
cp 'recovered_work/10f3deaf/nHXR.rs' 'iterations/v3/council/src/judge.rs'
cp 'recovered_work/-43197c08/r3ZE.rs' 'iterations/v3/council/src/plan_review.rs'
cp 'recovered_work/528e350a/Flrb.rs' 'iterations/v3/council/src/plan_review_integration_test.rs'
cp 'recovered_work/-1d87a5aa/4xJG.rs' 'iterations/v3/council/src/risk_scorer.rs'
cp 'recovered_work/-e740616/6HXI.rs' 'iterations/v3/council/src/verdict_aggregation.rs'
cp 'recovered_work/-4e6e6c7/YBj0.rs' 'iterations/v3/council/src/workflow.rs'
cp 'recovered_work/11b05fa3/nlrE.rs' 'iterations/v3/database/examples/performance_optimization.rs'
cp 'recovered_work/43c4cb3a/3XDq.sql' 'iterations/v3/database/migrations/008_add_caws_violations.sql'
cp 'recovered_work/-2ad844/6BQd.sql' 'iterations/v3/database/migrations/009_external_knowledge_schema.sql'
cp 'recovered_work/-4a8f5174/9O5f.sql' 'iterations/v3/database/migrations/010_database_integration_schema.sql'
cp 'recovered_work/-1a745a46/wbJx.sql' 'iterations/v3/database/migrations/011_artifacts_storage.sql'
cp 'recovered_work/509061f5/kBoM.sql' 'iterations/v3/database/migrations/012_artifact_versioning.sql'
cp 'recovered_work/119d646b/ClAD.sql' 'iterations/v3/database/migrations/013_historical_claims.sql'
cp 'recovered_work/1641755f/0kII.sql' 'iterations/v3/database/migrations/014_core_persistence.sql'
cp 'recovered_work/-38202b61/2di4.sql' 'iterations/v3/database/migrations/015_slo_tracking.sql'
cp 'recovered_work/27d6f5ef/OE51.sql' 'iterations/v3/database/migrations/016_task_audit_logs.sql'
cp 'recovered_work/-2f051867/1lOB.sql' 'iterations/v3/database/migrations/20250120_create_performance_indexes.sql'
cp 'recovered_work/1985257a/f5bz.rs' 'iterations/v3/database/src/artifact_store.rs'
cp 'recovered_work/-e724322/u4Ah.rs' 'iterations/v3/database/src/backup_recovery.rs'
cp 'recovered_work/9e4a539/Nlgo.rs' 'iterations/v3/database/src/backup_validator.rs'
cp 'recovered_work/5700aa8b/ckKN.rs' 'iterations/v3/database/src/data_consistency.rs'
cp 'recovered_work/-5e2f8b75/BqfA.rs' 'iterations/v3/database/src/knowledge_queries.rs'
cp 'recovered_work/50ee5d61/huIM.rs' 'iterations/v3/database/src/optimization.rs'
cp 'recovered_work/4b45ce9/h57y.rs' 'iterations/v3/database/src/vector_store.rs'
cp 'recovered_work/-7e77afb9/Vbjy.rs' 'iterations/v3/demo-integration.rs'
cp 'recovered_work/-7f0eac1a/T53n.rs' 'iterations/v3/demo-self-prompting.rs'
cp 'recovered_work/4a1e6cd1/nwi0.rs' 'iterations/v3/demo-simple.rs'
cp 'recovered_work/5f870178/pbb6.toml' 'iterations/v3/demo_app/Cargo.toml'
cp 'recovered_work/73a01318/JH2s.yaml' 'iterations/v3/demo_app/config/demo_config.yaml'
cp 'recovered_work/6bca6efa/1yCI.rs' 'iterations/v3/demo_app/src/demo_runner.rs'
cp 'recovered_work/387531df/mnw3.rs' 'iterations/v3/demo_app/src/main.rs'
cp 'recovered_work/77e47e38/ZjI3.rs' 'iterations/v3/demo_app/tests/integration_test.rs'
cp 'recovered_work/-6e6ebd04/JBvD.rs' 'iterations/v3/demo_v3_system.rs'
cp 'recovered_work/599e8e8d/0Wfi.yml' 'iterations/v3/docker-compose.yml'
cp 'recovered_work/-5e0a3547/t0eC.api' 'iterations/v3/docker/Dockerfile.api'
cp 'recovered_work/-15fdf80d/RGdL.dashboard' 'iterations/v3/docker/Dockerfile.dashboard'
cp 'recovered_work/3c8d7df8/WEnW.demo-app' 'iterations/v3/docker/Dockerfile.demo-app'
cp 'recovered_work/25d99e48/AjeF.federated-learning' 'iterations/v3/docker/Dockerfile.federated-learning'
cp 'recovered_work/31f9b29b/jsbi.model-hotswap' 'iterations/v3/docker/Dockerfile.model-hotswap'
cp 'recovered_work/3fd569e0/FLWM.multimodal-rag' 'iterations/v3/docker/Dockerfile.multimodal-rag'
cp 'recovered_work/69aa8683/uWtR.runtime-optimization' 'iterations/v3/docker/Dockerfile.runtime-optimization'
cp 'recovered_work/-16b222f4/A4t4.tool-ecosystem' 'iterations/v3/docker/Dockerfile.tool-ecosystem'
cp 'recovered_work/-606bb581/ZP1g.worker' 'iterations/v3/docker/Dockerfile.worker'
cp 'recovered_work/-c9b9aa5/OvnB.yml' 'iterations/v3/docker/docker-compose.production.yml'
cp 'recovered_work/183047c8/JdGW.yml' 'iterations/v3/docker/monitoring/alert_rules.yml'
cp 'recovered_work/-4aa82f80/eAmN.conf' 'iterations/v3/docker/monitoring/fluent-bit.conf'
cp 'recovered_work/-76f33f54/ZiCj.json' 'iterations/v3/docker/monitoring/grafana/dashboards/agent-agency-overview.json'
cp 'recovered_work/18a0e796/hIGz.yml' 'iterations/v3/docker/monitoring/grafana/provisioning/datasources/prometheus.yml'
cp 'recovered_work/518e854/MSy2.conf' 'iterations/v3/docker/monitoring/parsers.conf'
cp 'recovered_work/797ac41e/mM65.conf' 'iterations/v3/docker/nginx/nginx.conf'
cp 'recovered_work/-25334607/DEpk.md' 'iterations/v3/docs/DATABASE_TROUBLESHOOTING.md'
cp 'recovered_work/-5150fa50/KDfq.md' 'iterations/v3/docs/PRODUCTION_DEPLOYMENT.md'
cp 'recovered_work/-1680c58d/3jO8.md' 'iterations/v3/docs/PRODUCTION_DEPLOYMENT_GUIDE.md'
cp 'recovered_work/758a99f2/Dn5G.md' 'iterations/v3/docs/SYSTEM_OVERVIEW.md'
cp 'recovered_work/-5c44b51f/V3th.md' 'iterations/v3/docs/agents.md'
cp 'recovered_work/-1f81a871/MDaw.json' 'iterations/v3/docs/contracts/execution-artifacts.schema.json'
cp 'recovered_work/-1fd47b93/gu3f.json' 'iterations/v3/docs/contracts/quality-report.schema.json'
cp 'recovered_work/-7e4819bf/NdY3.json' 'iterations/v3/docs/contracts/refinement-decision.schema.json'
cp 'recovered_work/28857a72/kGY5.json' 'iterations/v3/docs/contracts/task-request.schema.json'
cp 'recovered_work/aed242c/L7sh.json' 'iterations/v3/docs/contracts/task-response.schema.json'
cp 'recovered_work/-43864efe/RuBJ.json' 'iterations/v3/docs/contracts/working-spec.schema.json'
cp 'recovered_work/-785b328c/n4a6.md' 'iterations/v3/docs/end-to-end-autonomous-flow-architecture.md'
cp 'recovered_work/272da6d/pU5X.toml' 'iterations/v3/e2e-tests/Cargo.toml'
cp 'recovered_work/-3b05203/3PgW.rs' 'iterations/v3/e2e-tests/assertions.rs'
cp 'recovered_work/57ebc98e/D7Ww.rs' 'iterations/v3/e2e-tests/autonomous_pipeline_test.rs'
cp 'recovered_work/-45e13ebc/DgKK.rs' 'iterations/v3/e2e-tests/fixtures.rs'
cp 'recovered_work/-49e9b580/Fk3C.rs' 'iterations/v3/e2e-tests/harness.rs'
cp 'recovered_work/-11286cd2/sLxt.rs' 'iterations/v3/e2e-tests/mod.rs'
cp 'recovered_work/22cb3b9e/EaYO.rs' 'iterations/v3/e2e-tests/runner.rs'
cp 'recovered_work/-41898113/RqC3.rs' 'iterations/v3/e2e-tests/scenarios.rs'
cp 'recovered_work/1bbf33e2/A9SK.rs' 'iterations/v3/embedding-service/src/model_loading.rs'
cp 'recovered_work/138206cb/R2G9.rs' 'iterations/v3/embedding-service/src/tokenization.rs'
cp 'recovered_work/bd6030d/xau6.rs' 'iterations/v3/enrichers/build.rs'
cp 'recovered_work/-716974c6/SauN.rs' 'iterations/v3/enrichers/src/python_bridge.rs'
cp 'recovered_work/-3e61ddec/dZGE.example' 'iterations/v3/env.production.example'
cp 'recovered_work/-1da7714a/HsOo.toml' 'iterations/v3/federated-learning/Cargo.toml'
cp 'recovered_work/61db68a2/qvgI.rs' 'iterations/v3/federated-learning/src/aggregation.rs'
cp 'recovered_work/779bc994/Fgm5.rs' 'iterations/v3/federated-learning/src/coordinator.rs'
cp 'recovered_work/76ce6d76/pBg5.rs' 'iterations/v3/federated-learning/src/differential_privacy.rs'
cp 'recovered_work/534bb217/13cP.rs' 'iterations/v3/federated-learning/src/encryption.rs'
cp 'recovered_work/-21cdb4e1/TUci.rs' 'iterations/v3/federated-learning/src/lib.rs'
cp 'recovered_work/6cbc5b50/9UFc.rs' 'iterations/v3/federated-learning/src/model_updates.rs'
cp 'recovered_work/6305f2b1/KukA.rs' 'iterations/v3/federated-learning/src/participant.rs'
cp 'recovered_work/-7913ba5e/ticy.rs' 'iterations/v3/federated-learning/src/protocol.rs'
cp 'recovered_work/-4384406/AYif.rs' 'iterations/v3/federated-learning/src/security.rs'
cp 'recovered_work/2ba04d61/Kao9.rs' 'iterations/v3/federated-learning/src/validation.rs'
cp 'recovered_work/5dfcbb8e/db1d.toml' 'iterations/v3/file_ops/Cargo.toml'
cp 'recovered_work/-52d9a10c/oUKK.rs' 'iterations/v3/file_ops/src/git_workspace.rs'
cp 'recovered_work/59d677f7/Yq9z.rs' 'iterations/v3/file_ops/src/lib.rs'
cp 'recovered_work/-6f1eff08/z7GY.rs' 'iterations/v3/file_ops/src/temp_workspace.rs'
cp 'recovered_work/-58794750/1Wvx.rs' 'iterations/v3/final_summary.rs'
cp 'recovered_work/-70b86457/CVvh.rs' 'iterations/v3/frontier_test.rs'
cp 'recovered_work/-5918170c/98tJ.md' 'iterations/v3/gap-analysis.md'
cp 'recovered_work/5648819f/Ttrw.rs' 'iterations/v3/integration-tests/src/autonomous_pipeline_test.rs'
cp 'recovered_work/-33bda3c/vXrD.rs' 'iterations/v3/integration-tests/src/integration_tests/config.rs'
cp 'recovered_work/-6f70e347/fifG.rs' 'iterations/v3/integration-tests/src/integration_tests/logging.rs'
cp 'recovered_work/-36dd594a/gNPK.rs' 'iterations/v3/integration-tests/src/integration_tests/mod.rs'
cp 'recovered_work/29811b16/ullh.rs' 'iterations/v3/integration-tests/src/integration_tests/runner.rs'
cp 'recovered_work/abb0f9f/y3he.rs' 'iterations/v3/integration-tests/src/integration_tests/types.rs'
cp 'recovered_work/-4e4e8749/WvYr.rs' 'iterations/v3/integration-tests/src/multimodal_rag_e2e_tests.rs'
cp 'recovered_work/-1a6b7150/6hzy.rs' 'iterations/v3/integration-tests/src/multimodal_rag_integration_test.rs'
cp 'recovered_work/-2dc3a3cc/d9xq.rs' 'iterations/v3/integration_test.rs'
cp 'recovered_work/-2dc3a3b8/Yo9y.sh' 'iterations/v3/integration_test.sh'
cp 'recovered_work/-1c3b12bd/dA8K.toml' 'iterations/v3/interfaces/Cargo.toml'
cp 'recovered_work/-37c94914/VOZJ.rs' 'iterations/v3/interfaces/api.rs'
cp 'recovered_work/-3497f5da/RNrO.rs' 'iterations/v3/interfaces/cli.rs'
cp 'recovered_work/-24032494/u42X.rs' 'iterations/v3/interfaces/mcp.rs'
cp 'recovered_work/-235f7efc/7mT2.rs' 'iterations/v3/interfaces/mod.rs'
cp 'recovered_work/-32c0d209/BRoo.rs' 'iterations/v3/interfaces/src/api.rs'
cp 'recovered_work/-2f8f7ecf/omuJ.rs' 'iterations/v3/interfaces/src/cli.rs'
cp 'recovered_work/-20615654/0vFF.rs' 'iterations/v3/interfaces/src/lib.rs'
cp 'recovered_work/-1efaad89/AteI.rs' 'iterations/v3/interfaces/src/mcp.rs'
cp 'recovered_work/-193f0056/rcVc.rs' 'iterations/v3/interfaces/src/websocket.rs'
cp 'recovered_work/7c493cdf/VQZA.rs' 'iterations/v3/interfaces/websocket.rs'
cp 'recovered_work/-49e14799/TrYr.toml' 'iterations/v3/knowledge-ingestor/Cargo.toml'
cp 'recovered_work/6f3dce7c/6YDD.md' 'iterations/v3/knowledge-ingestor/README.md'
cp 'recovered_work/-5aa8102c/D60P.rs' 'iterations/v3/knowledge-ingestor/src/bin/load_core_vocabulary.rs'
cp 'recovered_work/-3ae43ad1/C0ot.rs' 'iterations/v3/knowledge-ingestor/src/core_vocabulary.rs'
cp 'recovered_work/626ffae9/PGXF.rs' 'iterations/v3/knowledge-ingestor/src/cross_reference.rs'
cp 'recovered_work/-4e078b30/1ZF4.rs' 'iterations/v3/knowledge-ingestor/src/lib.rs'
cp 'recovered_work/36eed06a/izdX.rs' 'iterations/v3/knowledge-ingestor/src/on_demand.rs'
cp 'recovered_work/-41f9ff24/AmuM.rs' 'iterations/v3/knowledge-ingestor/src/types.rs'
cp 'recovered_work/-517e0571/8TGp.rs' 'iterations/v3/knowledge-ingestor/src/wikidata.rs'
cp 'recovered_work/-3ebe4be/IhDu.rs' 'iterations/v3/knowledge-ingestor/src/wordnet.rs'
cp 'recovered_work/-74a7d497/a2Vs.rs' 'iterations/v3/knowledge-ingestor/tests/integration_test.rs'
cp 'recovered_work/-6dc22d0f/ZzRD.js' 'iterations/v3/load-testing/k6-multimodal-rag-test.js'
cp 'recovered_work/5c6f5d3c/ISZU.toml' 'iterations/v3/memory/Cargo.toml'
cp 'recovered_work/328abb07/6k6b.md' 'iterations/v3/memory/README.md'
cp 'recovered_work/-1f5e8311/8sch.rs' 'iterations/v3/memory/examples/comprehensive_usage.rs'
cp 'recovered_work/-3c7a3b0a/Fzir.rs' 'iterations/v3/memory/src/integration.rs'
cp 'recovered_work/584919a5/3ls1.rs' 'iterations/v3/memory/src/lib.rs'
cp 'recovered_work/-3243914a/1qxF.txt' 'iterations/v3/methods.txt'
cp 'recovered_work/2a10199c/awrb.sql' 'iterations/v3/migrations/001_enable_pgvector.sql'
cp 'recovered_work/-2f4cd2e9/h5aS.sql' 'iterations/v3/migrations/002_create_vector_tables.sql'
cp 'recovered_work/-725b2c56/OOSR.bak2' 'iterations/v3/model-benchmarking/src/benchmark_runner.rs.bak2'
cp 'recovered_work/1e1b365d/cxyf.toml' 'iterations/v3/model-hotswap/Cargo.toml'
cp 'recovered_work/-21090323/IcT6.rs' 'iterations/v3/model-hotswap/src/hotswap_manager.rs'
cp 'recovered_work/19f4f2c6/86hw.rs' 'iterations/v3/model-hotswap/src/lib.rs'
cp 'recovered_work/-76fe9e44/Obd1.rs' 'iterations/v3/model-hotswap/src/load_balancer.rs'
cp 'recovered_work/-13a8e080/JmEP.rs' 'iterations/v3/model-hotswap/src/model_registry.rs'
cp 'recovered_work/-e0d2a45/wfQ7.rs' 'iterations/v3/model-hotswap/src/performance_router.rs'
cp 'recovered_work/c415561/rW4c.rs' 'iterations/v3/model-hotswap/src/rollback_manager.rs'
cp 'recovered_work/2826cc8a/dyPo.rs' 'iterations/v3/model-hotswap/src/traffic_splitter.rs'
cp 'recovered_work/56d0fc65/bClj.rs' 'iterations/v3/model-hotswap/src/version_manager.rs'
cp 'recovered_work/-7facf14b/IPyn.yml' 'iterations/v3/monitoring/alert_rules.yml'
cp 'recovered_work/46a906a4/bLoG.yml' 'iterations/v3/monitoring/alertmanager.yml'
cp 'recovered_work/30ebba86/V3zK.json' 'iterations/v3/monitoring/grafana/dashboards/business-intelligence.json'
cp 'recovered_work/-3995461b/SwjV.yml' 'iterations/v3/monitoring/grafana/dashboards/multimodal-rag.yml'
cp 'recovered_work/332ff54c/KoSq.json' 'iterations/v3/monitoring/grafana/dashboards/slo-tracking.json'
cp 'recovered_work/5f1ceabb/bs8B.json' 'iterations/v3/monitoring/grafana/dashboards/system-overview.json'
cp 'recovered_work/-35be2d37/mkMU.yml' 'iterations/v3/monitoring/grafana/datasources/prometheus.yml'
cp 'recovered_work/2a1cfda0/4IRX.json' 'iterations/v3/monitoring/grafana/provisioning/dashboards/agent-agency-dashboard.json'
cp 'recovered_work/-2b1b457d/0ZK3.yml' 'iterations/v3/monitoring/grafana/provisioning/datasources/prometheus.yml'
cp 'recovered_work/1df42178/n7mN.yml' 'iterations/v3/monitoring/multimodal_rag_rules.yml'
cp 'recovered_work/-50f5e583/pbHF.yml' 'iterations/v3/monitoring/prometheus.yml'
cp 'recovered_work/-1805cdef/NqDn.rs' 'iterations/v3/observability/src/cache/mod.rs'
cp 'recovered_work/3238fbd5/IJay.rs' 'iterations/v3/observability/src/cache/redis_cache.rs'
cp 'recovered_work/-2026a0af/42vo.rs' 'iterations/v3/observability/src/diff_observability.rs'
cp 'recovered_work/548618cd/IOA9.rs' 'iterations/v3/observability/src/errors.rs'
cp 'recovered_work/-50c5a6de/YZNr.rs' 'iterations/v3/observability/src/metrics/prometheus.rs'
cp 'recovered_work/-62230de9/jNvr.rs' 'iterations/v3/observability/src/metrics/redis.rs'
cp 'recovered_work/3c618b07/sz69.rs' 'iterations/v3/observability/src/metrics/statsd.rs'
cp 'recovered_work/1109d300/RH8T.rs' 'iterations/v3/observability/src/multimodal_metrics.rs'
cp 'recovered_work/-34e16b94/OH6n.rs' 'iterations/v3/orchestration/src/api.rs'
cp 'recovered_work/-43104043/kSD0.rs' 'iterations/v3/orchestration/src/arbiter.rs'
cp 'recovered_work/4cbe23a7/9Nde.rs' 'iterations/v3/orchestration/src/artifacts/manager.rs'
cp 'recovered_work/59995ab2/qWQy.rs' 'iterations/v3/orchestration/src/artifacts/mod.rs'
cp 'recovered_work/-1770a9e7/Z72U.rs' 'iterations/v3/orchestration/src/artifacts/storage.rs'
cp 'recovered_work/6d01fd00/7Bbj.rs' 'iterations/v3/orchestration/src/artifacts/versioning.rs'
cp 'recovered_work/5c3df9e4/AStO.rs' 'iterations/v3/orchestration/src/audit_trail.rs'
cp 'recovered_work/-24987357/c6TR.rs' 'iterations/v3/orchestration/src/audited_orchestrator.rs'
cp 'recovered_work/4200b6c4/PDrG.rs' 'iterations/v3/orchestration/src/autonomous_executor.rs'
cp 'recovered_work/73b3dc5/Ma9t.rs' 'iterations/v3/orchestration/src/caws_runtime.rs'
cp 'recovered_work/5e99544b/bfyo.rs' 'iterations/v3/orchestration/src/frontier.rs'
cp 'recovered_work/-5fdd861/Zb9b.rs' 'iterations/v3/orchestration/src/main.rs'
cp 'recovered_work/-1390bc9e/l0gU.rs' 'iterations/v3/orchestration/src/multimodal_orchestration.rs'
cp 'recovered_work/d89027e/VeU8.rs' 'iterations/v3/orchestration/src/planning/acceptance_criteria_extractor.rs'
cp 'recovered_work/51e85205/WINB.rs' 'iterations/v3/orchestration/src/planning/agent.rs'
cp 'recovered_work/-1d880b9a/obOY.rs' 'iterations/v3/orchestration/src/planning/clarification.rs'
cp 'recovered_work/5908947f/38sb.rs' 'iterations/v3/orchestration/src/planning/context_builder.rs'
cp 'recovered_work/1e428697/wx5I.rs' 'iterations/v3/orchestration/src/planning/integration_test.rs'
cp 'recovered_work/-4bc63ee9/6jdV.rs' 'iterations/v3/orchestration/src/planning/llm_client.rs'
cp 'recovered_work/-57281c58/8kBU.rs' 'iterations/v3/orchestration/src/planning/mod.rs'
cp 'recovered_work/-122afc9b/TXeT.rs' 'iterations/v3/orchestration/src/planning/spec_generator.rs'
cp 'recovered_work/-49f5de97/5cak.rs' 'iterations/v3/orchestration/src/planning/tests.rs'
cp 'recovered_work/-2deb27ef/44Mv.rs' 'iterations/v3/orchestration/src/planning/types.rs'
cp 'recovered_work/-39523180/kkD0.rs' 'iterations/v3/orchestration/src/planning/validation_loop.rs'
cp 'recovered_work/361de10e/425a.rs' 'iterations/v3/orchestration/src/quality/gates.rs'
cp 'recovered_work/10186a94/H1dJ.rs' 'iterations/v3/orchestration/src/quality/mod.rs'
cp 'recovered_work/106b6c1e/mxHk.rs' 'iterations/v3/orchestration/src/quality/orchestrator.rs'
cp 'recovered_work/-37eae2dc/7yez.rs' 'iterations/v3/orchestration/src/quality/satisficing.rs'
cp 'recovered_work/-6f974c34/4JeK.rs' 'iterations/v3/orchestration/src/refinement/coordinator.rs'
cp 'recovered_work/-33620e22/IsK0.rs' 'iterations/v3/orchestration/src/refinement/feedback_loop.rs'
cp 'recovered_work/31d33bba/tKqw.rs' 'iterations/v3/orchestration/src/refinement/mod.rs'
cp 'recovered_work/4d903faf/spaZ.rs' 'iterations/v3/orchestration/src/refinement/strategy.rs'
cp 'recovered_work/-451ba0ab/jPGo.rs' 'iterations/v3/orchestration/src/tracking/event_bus.rs'
cp 'recovered_work/4828130e/hm45.rs' 'iterations/v3/orchestration/src/tracking/mod.rs'
cp 'recovered_work/36480d48/RitF.rs' 'iterations/v3/orchestration/src/tracking/progress_tracker.rs'
cp 'recovered_work/45599f69/bWQ4.rs' 'iterations/v3/orchestration/src/tracking/websocket.rs'
cp 'recovered_work/2e4d2f28/LFyx.toml' 'iterations/v3/planning-agent/Cargo.toml'
cp 'recovered_work/2c3bb9d9/uBrT.rs' 'iterations/v3/planning-agent/src/caws_integration.rs'
cp 'recovered_work/5feab36e/ybx4.rs' 'iterations/v3/planning-agent/src/error.rs'
cp 'recovered_work/2a26eb91/D7Fe.rs' 'iterations/v3/planning-agent/src/lib.rs'
cp 'recovered_work/-90eae5c/rzPV.rs' 'iterations/v3/planning-agent/src/planner.rs'
cp 'recovered_work/5e2b880/YVMj.rs' 'iterations/v3/planning-agent/src/refinement_engine.rs'
cp 'recovered_work/323e5d8e/9NQp.rs' 'iterations/v3/planning-agent/src/validation_pipeline.rs'
cp 'recovered_work/7ffed584/iwrw.toml' 'iterations/v3/production/Cargo.toml'
cp 'recovered_work/47f6bd0d/MxGl.rs' 'iterations/v3/production/documentation.rs'
cp 'recovered_work/1e73e41b/tJYy.rs' 'iterations/v3/production/error_handling.rs'
cp 'recovered_work/32d2b1c5/edR6.rs' 'iterations/v3/production/mod.rs'
cp 'recovered_work/6c03a1c6/P2R0.rs' 'iterations/v3/production/observability.rs'
cp 'recovered_work/3d955597/kf01.rs' 'iterations/v3/production/security.rs'
cp 'recovered_work/-5ec343dc/wIWU.rs' 'iterations/v3/research/src/multimodal_context_provider.rs'
cp 'recovered_work/-2b1ab925/BYy9.toml' 'iterations/v3/runtime-optimization/Cargo.toml'
cp 'recovered_work/-15512c03/O7Sj.rs' 'iterations/v3/runtime-optimization/src/arbiter_pipeline.rs'
cp 'recovered_work/300d4af1/Xhlo.rs' 'iterations/v3/runtime-optimization/src/bayesian_optimizer.rs'
cp 'recovered_work/-4a51a21c/JADE.rs' 'iterations/v3/runtime-optimization/src/chunked_execution.rs'
cp 'recovered_work/-4c634d71/XSPl.rs' 'iterations/v3/runtime-optimization/src/chunked_executor.rs'
cp 'recovered_work/-6c93f126/Br8r.rs' 'iterations/v3/runtime-optimization/src/kokoro_tuning.rs'
cp 'recovered_work/-2f40fcbc/ugNw.rs' 'iterations/v3/runtime-optimization/src/lib.rs'
cp 'recovered_work/-48427042/9QCm.rs' 'iterations/v3/runtime-optimization/src/performance_monitor.rs'
cp 'recovered_work/51b74757/25nC.rs' 'iterations/v3/runtime-optimization/src/precision_engineering.rs'
cp 'recovered_work/3768be59/lwDE.rs' 'iterations/v3/runtime-optimization/src/quality_guardrails.rs'
cp 'recovered_work/b95e076/t7tR.rs' 'iterations/v3/runtime-optimization/src/streaming_pipeline.rs'
cp 'recovered_work/97a9376/oxNO.rs' 'iterations/v3/runtime-optimization/src/thermal_scheduler.rs'
cp 'recovered_work/451c17a9/0BBw.sh' 'iterations/v3/scripts/deploy-production.sh'
cp 'recovered_work/-1b460dbb/92DC.sh' 'iterations/v3/scripts/install-git-hooks.sh'
cp 'recovered_work/-3a327542/tfQm.sh' 'iterations/v3/scripts/run-e2e-tests.sh'
cp 'recovered_work/-7b669b9a/faJa.sh' 'iterations/v3/scripts/run_integration_tests.sh'
cp 'recovered_work/48cd5d1d/xqLl.toml' 'iterations/v3/security/Cargo.toml'
cp 'recovered_work/-3df24690/tISz.rs' 'iterations/v3/security/src/audit.rs'
cp 'recovered_work/-1e74c345/mGTI.rs' 'iterations/v3/security/src/authentication.rs'
cp 'recovered_work/50a58cd5/TrRH.rs' 'iterations/v3/security/src/circuit_breaker.rs'
cp 'recovered_work/-f57759b/x7tZ.rs' 'iterations/v3/security/src/input_validation.rs'
cp 'recovered_work/44a71986/tMgH.rs' 'iterations/v3/security/src/lib.rs'
cp 'recovered_work/-44b4e15b/yMgt.rs' 'iterations/v3/security/src/rate_limiting.rs'
cp 'recovered_work/-3e7a3c66/EcDU.rs' 'iterations/v3/security/src/sanitization.rs'
cp 'recovered_work/44df0055/XLYS.rs' 'iterations/v3/security/src/secret_manager.rs'
cp 'recovered_work/43a0f5e1/AhvU.rs' 'iterations/v3/security/src/secure_config.rs'
cp 'recovered_work/3b72e248/QgkS.toml' 'iterations/v3/self-prompting-agent/Cargo.toml'
cp 'recovered_work/-627965c0/RjPQ.md' 'iterations/v3/self-prompting-agent/EDGE_CASE_ANALYSIS.md'
cp 'recovered_work/6092d3c3/GZxW.rs' 'iterations/v3/self-prompting-agent/examples/playground_test.rs'
cp 'recovered_work/-41354f8f/fN3t.rs' 'iterations/v3/self-prompting-agent/src/agent.rs'
cp 'recovered_work/81d93a6/PN7F.rs' 'iterations/v3/self-prompting-agent/src/caws/budget_checker.rs'
cp 'recovered_work/-54ad1000/doRY.rs' 'iterations/v3/self-prompting-agent/src/caws/council_approval.rs'
cp 'recovered_work/-4d4beb65/LMgg.rs' 'iterations/v3/self-prompting-agent/src/caws/mod.rs'
cp 'recovered_work/2b7fb609/3lna.rs' 'iterations/v3/self-prompting-agent/src/caws/waiver_generator.rs'
cp 'recovered_work/-9b9d627/vl9z.rs' 'iterations/v3/self-prompting-agent/src/evaluation/caws_evaluator.rs'
cp 'recovered_work/46d66286/TQnP.rs' 'iterations/v3/self-prompting-agent/src/evaluation/code_evaluator.rs'
cp 'recovered_work/7f8dea8b/3tzt.rs' 'iterations/v3/self-prompting-agent/src/evaluation/flakiness.rs'
cp 'recovered_work/5e7f5419/80Rj.rs' 'iterations/v3/self-prompting-agent/src/evaluation/mod.rs'
cp 'recovered_work/-5eef4e57/Uk3j.rs' 'iterations/v3/self-prompting-agent/src/evaluation/satisficing.rs'
cp 'recovered_work/138e9346/tbPS.rs' 'iterations/v3/self-prompting-agent/src/evaluation/text_evaluator.rs'
cp 'recovered_work/-570c6dae/0a7a.rs' 'iterations/v3/self-prompting-agent/src/evaluation/token_evaluator.rs'
cp 'recovered_work/70697e02/wGAt.rs' 'iterations/v3/self-prompting-agent/src/integration.rs'
cp 'recovered_work/-6d9ff334/YMG3.rs' 'iterations/v3/self-prompting-agent/src/learning_bridge.rs'
cp 'recovered_work/374c9eb1/DI24.rs' 'iterations/v3/self-prompting-agent/src/lib.rs'
cp 'recovered_work/6514795f/q1Dy.rs' 'iterations/v3/self-prompting-agent/src/loop_controller.rs'
cp 'recovered_work/75c08bf8/6XjH.rs' 'iterations/v3/self-prompting-agent/src/minimal_test.rs'
cp 'recovered_work/-132c010d/jFQd.rs' 'iterations/v3/self-prompting-agent/src/models/coreml.rs'
cp 'recovered_work/79bb821b/kNpJ.rs' 'iterations/v3/self-prompting-agent/src/models/coreml_provider.rs'
cp 'recovered_work/-3937da55/9A7g.rs' 'iterations/v3/self-prompting-agent/src/models/mod.rs'
cp 'recovered_work/6427e64b/CTNG.rs' 'iterations/v3/self-prompting-agent/src/models/ollama.rs'
cp 'recovered_work/-20735f9f/obw8.rs' 'iterations/v3/self-prompting-agent/src/models/selection.rs'
cp 'recovered_work/2f042525/VNBn.rs' 'iterations/v3/self-prompting-agent/src/policy_hooks.rs'
cp 'recovered_work/bc54830/aF6U.rs' 'iterations/v3/self-prompting-agent/src/profiling.rs'
cp 'recovered_work/-608adf1f/cqD0.rs' 'iterations/v3/self-prompting-agent/src/prompting/adaptive.rs'
cp 'recovered_work/5287d73a/3aRI.rs' 'iterations/v3/self-prompting-agent/src/prompting/frame.rs'
cp 'recovered_work/296fcb85/QsIu.rs' 'iterations/v3/self-prompting-agent/src/prompting/mod.rs'
cp 'recovered_work/61c7b29f/HPHL.rs' 'iterations/v3/self-prompting-agent/src/prompting/tool_schema.rs'
cp 'recovered_work/207e9922/9OWj.rs' 'iterations/v3/self-prompting-agent/src/rl_signals.rs'
cp 'recovered_work/5178762f/qSvE.rs' 'iterations/v3/self-prompting-agent/src/sandbox/diff_applier.rs'
cp 'recovered_work/-7e99b7d9/OzIX.rs' 'iterations/v3/self-prompting-agent/src/sandbox/diff_generator.rs'
cp 'recovered_work/b4554fe/5pS7.rs' 'iterations/v3/self-prompting-agent/src/sandbox/file_guard.rs'
cp 'recovered_work/5d7908a4/68F5.rs' 'iterations/v3/self-prompting-agent/src/sandbox/git_worktree.rs'
cp 'recovered_work/-4e325a4/wpn1.rs' 'iterations/v3/self-prompting-agent/src/sandbox/mod.rs'
cp 'recovered_work/-f6129a4/1xyw.rs' 'iterations/v3/self-prompting-agent/src/sandbox/snapshot.rs'
cp 'recovered_work/699397b/A1FI.rs' 'iterations/v3/self-prompting-agent/src/sandbox/workspace_manager.rs'
cp 'recovered_work/-290b2a0f/vt82.rs' 'iterations/v3/self-prompting-agent/src/stubs.rs'
cp 'recovered_work/3ef7367d/6zRD.rs' 'iterations/v3/self-prompting-agent/src/types.rs'
cp 'recovered_work/-64db3568/C6pF.rs' 'iterations/v3/self-prompting-agent/tests/autonomous_agent_integration_tests.rs'
cp 'recovered_work/7822a8d9/NNXP.rs' 'iterations/v3/self-prompting-agent/tests/brittleness_integration_tests.rs'
cp 'recovered_work/56f9e843/I2nj.rs' 'iterations/v3/self-prompting-agent/tests/integration_tests.rs'
cp 'recovered_work/13ae9c14/50mn.rs' 'iterations/v3/source-integrity/src/storage_new.rs'
cp 'recovered_work/-6e053b98/nNEd.rs' 'iterations/v3/src/bin/api-server.rs'
cp 'recovered_work/6a5d920/4WW1.rs' 'iterations/v3/src/bin/cli.rs'
cp 'recovered_work/-1b8eee2d/RmuI.rs' 'iterations/v3/system-health-monitor/tests/basic_tests.rs'
cp 'recovered_work/-2000a37a/UOxW.toml' 'iterations/v3/test-api-server-Cargo.toml'
cp 'recovered_work/-5480e3e4/zm4Q.rs' 'iterations/v3/test-api-server.rs'
cp 'recovered_work/-298c7c0b/ePUs.rs' 'iterations/v3/test-core-functionality.rs'
cp 'recovered_work/5afa4cd9/F4ec.rs' 'iterations/v3/test_api_config.rs'
cp 'recovered_work/-58d571ca/Ivyr.rs' 'iterations/v3/test_orchestrator.rs'
cp 'recovered_work/-612d6d15/0jcx.md' 'iterations/v3/tests/README.md'
cp 'recovered_work/1b8cc419/Hf4E.toml' 'iterations/v3/tests/benchmarks/Cargo.toml'
cp 'recovered_work/7febca0a/34l7.rs' 'iterations/v3/tests/benchmarks/benches/model_inference.rs'
cp 'recovered_work/-2cc09bf9/Bw4G.rs' 'iterations/v3/tests/integration.rs'
cp 'recovered_work/-36b0be93/YoJV.sh' 'iterations/v3/tests/run-test-plan.sh'
cp 'recovered_work/-303ecd53/aQc1.md' 'iterations/v3/tests/test-plan.md'
cp 'recovered_work/-64b748bc/pHNR.rs' 'iterations/v3/tests/test_data/complex_function.rs'
cp 'recovered_work/4e4b66eb/kJF4.py' 'iterations/v3/tests/test_data/style_issues.py'
cp 'recovered_work/-754f9d01/R7OL.rs' 'iterations/v3/tests/test_data/syntax_error.rs'
cp 'recovered_work/-65cfc09f/Olou.ts' 'iterations/v3/tests/test_data/type_errors.ts'
cp 'recovered_work/-4072490e/5W09.toml' 'iterations/v3/tool-ecosystem/Cargo.toml'
cp 'recovered_work/6cb9c7cb/FGqq.rs' 'iterations/v3/tool-ecosystem/src/conflict_resolution_tools.rs'
cp 'recovered_work/10af201e/jS3q.rs' 'iterations/v3/tool-ecosystem/src/evidence_collection_tools.rs'
cp 'recovered_work/-44988ca5/adgK.rs' 'iterations/v3/tool-ecosystem/src/lib.rs'
cp 'recovered_work/-11608e2b/L9YA.rs' 'iterations/v3/tool-ecosystem/src/tool_coordinator.rs'
cp 'recovered_work/-5808db2b/AcRO.rs' 'iterations/v3/tool-ecosystem/src/tool_discovery.rs'
cp 'recovered_work/1929c66d/wAga.rs' 'iterations/v3/tool-ecosystem/src/tool_execution.rs'
cp 'recovered_work/982d3c/WyyX.rs' 'iterations/v3/tool-ecosystem/src/tool_registry.rs'
cp 'recovered_work/24b7da99/Et2s.rs' 'iterations/v3/validate_implementations.rs'
cp 'recovered_work/5d812f8c/Un3b.swift' 'iterations/v3/vision-bridge/Package.swift'
cp 'recovered_work/15bee83e/2Sqa.swift' 'iterations/v3/vision-bridge/Sources/VisionBridge/VisionBridge.swift'
cp 'recovered_work/3a13c69f/s1ic.toml' 'iterations/v3/worker/Cargo.toml'
cp 'recovered_work/-507eee68/qFGV.rs' 'iterations/v3/worker/src/main.rs'
cp 'recovered_work/-25186151/0EBG.rs' 'iterations/v3/worker_integration_test.rs'
cp 'recovered_work/79fa2250/4flW.rs' 'iterations/v3/workers/src/autonomous_executor.rs'
cp 'recovered_work/14f5929c/1KTH.rs' 'iterations/v3/workers/src/multimodal_scheduler.rs'
cp 'recovered_work/7b0dbd64/s9jV.md' 'missing.md'
cp 'recovered_work/32c8074d/noof.yaml' 'pnpm-workspace.yaml'
cp 'recovered_work/-3630b97e/bDFj.js' 'scripts/benchmark-build-performance.js'
cp 'recovered_work/4dc5d732/eqSF.,toml' 'scripts/cargo.%2Ctoml'
cp 'recovered_work/16779fcc/y9CP.sh' 'scripts/disaster-recovery/failover-api-server.sh'
cp 'recovered_work/18421a47/rMPt.sh' 'scripts/disaster-recovery/failover-database.sh'
cp 'recovered_work/-98189fd/5dhN.sh' 'scripts/disaster-recovery/test-disaster-recovery.sh'
cp 'recovered_work/20e0b227/Ux1O.sh' 'scripts/setup-distributed-cache.sh'
cp 'recovered_work/625a085b/vKG9.sh' 'scripts/setup-tmpfs.sh'
cp 'recovered_work/-1b432b2d/zf9F.rs' 'scripts/test-optimization.rs'
cp 'recovered_work/238324c9/aneS.rs' 'test-autonomous.rs'
cp 'recovered_work/-2e22e3ef/LhQ2.rs' 'test-clarification-workflow.rs'
cp 'recovered_work/10e8b12f/12J9.rs' 'test-edge-case-improvements.rs'
cp 'recovered_work/47809e26/Qhpu.rs' 'test-edge-cases.rs'
cp 'recovered_work/501c4c6f/ohDE.rs' 'test-enhanced-feasibility.rs'
cp 'recovered_work/-4f606a11/mNCv.rs' 'test-error-handling.rs'
cp 'recovered_work/407c7616/VJLL.rs' 'test-ethical-reasoning.rs'
cp 'recovered_work/-70e04d45/Oqvx.rs' 'test-full-autonomous-workflow.rs'
cp 'recovered_work/-7ee0ce46/MaPM.rs' 'test-multi-dimensional-risk.rs'
cp 'recovered_work/-ea56394/2fCJ.rs' 'test-optimization-validation.rs'
cp 'recovered_work/-325527e8/HNqS.toml' 'test-project/Cargo.toml'
cp 'recovered_work/-4eada1a3/KHWG.jsonl' 'test-results-history.jsonl'
cp 'recovered_work/37d380fd/ON7c.json' 'turbo.json'
```

## Phase 3: Validation Checklist

- [ ] All directories created
- [ ] All files restored
- [ ] Compilation errors fixed
- [ ] Dependencies updated
- [ ] Tests passing
- [ ] Integration validated