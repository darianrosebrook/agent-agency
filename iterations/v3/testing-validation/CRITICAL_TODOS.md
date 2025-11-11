# Critical TODOs: Quality Evaluation Framework

**Created**: 2025-11-10  
**Purpose**: Track critical missing functionality in quality evaluation framework flows

---

## 🔴 CRITICAL: Compilation Check Failures

### Issue
Rust and Python compilation checks are failing even when code compiles correctly.

### Root Causes
1. **Path Resolution**: File path may not match expected structure
   - Rust expects `src/lib.rs` with `Cargo.toml` in parent directory
   - Python expects file in workspace root
   - Current logic may not handle all path variations

2. **Agent Artifact Writing**: Agent may not be writing files correctly
   - Artifact content may have markdown fences that aren't fully stripped
   - File may be written to wrong location
   - File may not be persisted before compilation check

3. **Compilation Check Logic**: May have bugs in `check_code_compiles()`
   - Error detection may be too strict
   - Workspace structure may not match expectations

### Files Affected
- `src/scenarios/integrated_playground_quality.rs`:
  - `check_code_compiles()` (lines 591-698)
  - `run_playground_test()` artifact writing (lines 367-428)
  - File path resolution (lines 228-283)

### Action Items
- [ ] Add detailed logging to `check_code_compiles()`:
  - Log file path before/after agent execution
  - Log file content after agent execution
  - Log compilation command and full output
- [ ] Verify agent file modifications:
  - Check if file exists after agent execution
  - Verify file content is correct
  - Ensure markdown fences are fully stripped
- [ ] Debug compilation check logic:
  - Test manually with `cargo check` from workspace root
  - Test manually with `python3 -m py_compile` on fixed file
  - Verify path resolution logic handles all cases
- [ ] Fix path resolution if needed:
  - Ensure Rust files are in `src/` directory
  - Ensure `Cargo.toml` is in workspace root
  - Ensure Python files are accessible from workspace root

### Priority
**CRITICAL** - Blocks all Rust/Python tests from passing

---

## 🟡 HIGH: Council Transparency Placeholder

### Issue
Quality evaluation uses hardcoded placeholder (0.7) instead of real council verdict.

### Location
`src/scenarios/integrated_playground_quality.rs:547`

```rust
0.7, // Placeholder for council transparency (would come from real council verdict)
```

### Impact
- Overall quality score calculation is inaccurate
- Missing real council evaluation integration
- Cannot properly assess council decision-making quality

### Options
1. **Integrate Constitutional Council**: Create council verdict for agent output
2. **Document Limitation**: Document why council evaluation isn't applicable to playground tests

### Action Items
- [ ] Determine if council evaluation is applicable to playground tests
- [ ] If yes: Integrate council verdict creation into flow
- [ ] If no: Document why and use appropriate placeholder value
- [ ] Use real `CouncilTransparencyScore` instead of placeholder

### Priority
**HIGH** - Affects overall quality score accuracy

---

## 🟡 HIGH: Decision Quality Score Below Threshold

### Issue
Current score: 0.65, needs 0.7 (gap: -0.05)

### Root Causes
1. **Decision Point Extraction**: May not capture all relevant information
   - `extract_decision_points_from_events()` creates synthetic decision points
   - May not reflect actual agent reasoning quality
   - Evidence-based reasoning may not be detected

2. **Decision Quality Analyzer**: Detection patterns may be too strict
   - Evidence keywords: "because", "due to", "based on", "evidence", "data"
   - Logic keywords: "therefore", "thus", "consequently", "however", "alternatively"
   - May miss other evidence patterns

### Files Affected
- `src/scenarios/integrated_playground_quality.rs`:
  - `extract_decision_points_from_events()` (lines 700-844)
- `src/quality_analyzers.rs`:
  - `DecisionQualityScore::analyze()` (lines 128-214)

### Action Items
- [ ] Enhance decision point extraction:
  - Capture more evidence from agent events
  - Extract actual reasoning from agent output
  - Include more alternatives in decision points
- [ ] Improve evidence detection patterns:
  - Add more evidence keywords (e.g., "shows", "indicates", "suggests")
  - Add more logic keywords (e.g., "since", "as", "given that")
  - Detect evidence patterns in reasoning text
- [ ] Ensure risk assessments include mitigation strategies:
  - Check that all risk assessments have mitigation_strategies
  - Add fallback options if missing

### Priority
**HIGH** - Close to threshold, needs +0.05 improvement

---

## 🟡 HIGH: Code Quality Score Below Threshold

### Issue
Current score: 0.58, needs 0.7 (gap: -0.12)

### Breakdown
- Compilation: 0.3 ✅
- Structure: 0.18 ✅
- Error Handling: 0.1 ✅
- Test Coverage: 0.0 ❌ (missing 0.2 points)
- Documentation: 0.0 ❌ (missing 0.2 points)

### Root Causes
1. **Test Coverage Detection**: Too strict
   - Requires tests in same file
   - Agent may create tests in separate files
   - Should check for test files in same directory

2. **Documentation Detection**: May miss patterns
   - Requires doc comments (`///`, `/**`, `"""`)
   - Agent may use regular comments (`//`, `#`)
   - Should accept regular comments as basic documentation

### Files Affected
- `src/quality_analyzers.rs`:
  - `CodeQualityScore::analyze()` (lines 389-648)
  - Test coverage detection (lines 574-598)
  - Documentation detection (lines 600-636)

### Action Items
- [ ] Improve test detection:
  - Check for test files in same directory (e.g., `*_test.rs`, `*.test.ts`, `test_*.py`)
  - Check for test directories (e.g., `tests/`, `__tests__/`)
  - Give partial credit for test file existence
- [ ] Improve documentation detection:
  - Accept regular comments (`//`, `#`) as basic documentation
  - Give partial credit for inline comments
  - Detect documentation patterns in code structure
- [ ] Add partial credit system:
  - Basic documentation: 0.1 points
  - Good documentation: 0.15 points
  - Excellent documentation: 0.2 points

### Priority
**HIGH** - Needs +0.12 improvement, missing test/doc detection

---

## Summary

### Critical Issues (Blocking)
1. ✅ Rust compilation check failure
2. ✅ Python compilation check failure

### High Priority Issues (Affecting Scores)
3. ✅ Council transparency placeholder
4. ✅ Decision quality below threshold (0.65 → 0.7)
5. ✅ Code quality below threshold (0.58 → 0.7)

### Success Criteria Progress
- Reasoning Depth: ✅ 0.85 (exceeds threshold)
- Decision Quality: ⚠️ 0.65 (needs +0.05)
- Code Quality: ⚠️ 0.58 (needs +0.12)
- Rust Compilation: ❌ FAILING
- Python Compilation: ❌ FAILING
- Council Transparency: ⚠️ PLACEHOLDER

**Overall**: 1/6 criteria fully met (17%)

### Next Session Priorities
1. Fix Rust/Python compilation checks (CRITICAL)
2. Improve Decision Quality score (+0.05)
3. Improve Code Quality score (+0.12)
4. Integrate or document council transparency





