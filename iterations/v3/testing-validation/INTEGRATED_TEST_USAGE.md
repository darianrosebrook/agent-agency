# Integrated Playground + Quality Evaluation Test Usage

**Purpose**: Run playground tests first (functional correctness), then quality evaluation (quality standards) for comprehensive agent evaluation.

## Quick Start

```bash
cd iterations/v3/testing-validation
cargo run --bin integrated_test --features full
```

## What It Does

### Step 1: Playground Test (Functional Correctness)
1. Creates broken code files (Rust, TypeScript, Python)
2. Simulates agent fixing the code
3. Validates:
   - Code compiles/fixes errors
   - Chain-of-thought is complete
   - Errors are detected and fixed

### Step 2: Quality Evaluation (Quality Standards)
1. Analyzes reasoning depth from decision points
2. Evaluates decision quality
3. Assesses code quality
4. Compares scores against mid-level engineer standards (≥0.7)

### Step 3: Combined Report
- Generates `integrated_test_report.md` with:
  - Playground test results (functional correctness)
  - Quality evaluation scores (reasoning, decision, code quality)
  - Overall pass/fail status
  - Success criteria met/failed

## Test Scenarios

The integrated test runs three scenarios:

1. **Rust Code Fix** (`integrated-rust`)
   - Broken Rust code with compilation errors
   - Agent fixes errors
   - Quality evaluation on fix quality

2. **TypeScript Code Fix** (`integrated-typescript`)
   - Broken TypeScript code with type errors
   - Agent fixes errors
   - Quality evaluation on fix quality

3. **Python Code Fix** (`integrated-python`)
   - Broken Python code with syntax/logic errors
   - Agent fixes errors
   - Quality evaluation on fix quality

## Expected Output

```
🚀 Starting Integrated Playground + Quality Evaluation Test
✅ Test environment initialized
✅ Services initialized
Starting integrated test: integrated-rust (rust)
Running playground test for rust
Playground test passed, running quality evaluation...
Running quality evaluation on fixed code
Starting integrated test: integrated-typescript (typescript)
...

🎯 Test Suite Summary:
   Total tests: 3
   Passed: 3
   Failed: 0
   Success rate: 100.0%

📋 integrated-rust
   Playground: ✅ PASSED
   Quality: ✅ PASSED (Score: 0.78)
   Overall: ✅ PASSED
   Duration: 1234ms

📄 Detailed report saved to: integrated_test_report.md
🎉 All integrated tests passed!
```

## Report Format

The generated `integrated_test_report.md` includes:

```markdown
## integrated-rust

### Playground Test (Functional Correctness)
- File: broken-rust.rs
- Fixed: true
- Errors Detected: 5
- Errors Fixed: 5
- Chain-of-Thought Complete: true

### Quality Evaluation
- Overall Score: 0.78
- Reasoning Depth: 0.82 (Good depth - solid analysis)
- Decision Quality: 0.75
- Output Quality: 0.75
- Status: PASSED

#### Success Criteria Met
- Reasoning depth ≥ 0.7
- Decision quality ≥ 0.7
- Code quality ≥ 0.7
```

## Integration with Real Agents

Currently, the test uses simulated decision points. To integrate with real agents:

1. Replace `simulate_agent_fixing_code()` with actual agent execution
2. Use real `DecisionPoint` structures from agent orchestration
3. Use actual fixed code files for quality analysis
4. Connect to real council verdicts for council transparency analysis

## Success Criteria

A test **PASSES** when:
- ✅ Playground test: Code is fixed (functional correctness)
- ✅ Quality evaluation: Overall score ≥ 0.7 (meets mid-level standards)
- ✅ Reasoning depth ≥ 0.7
- ✅ Decision quality ≥ 0.7
- ✅ Code quality ≥ 0.7

## Troubleshooting

### Feature Flag Required
If you see:
```
❌ Integrated test requires 'full' feature
```

Run with:
```bash
cargo run --bin integrated_test --features full
```

### Compilation Errors
If you see compilation errors, ensure:
- `agent-orchestration` is available with `evaluation` feature
- `agent-constitutional-council` is available
- All dependencies are built

### Test Failures
If tests fail:
1. Check `integrated_test_report.md` for detailed results
2. Review playground test results (did code get fixed?)
3. Review quality scores (which criteria failed?)
4. Check logs for detailed error messages

## Next Steps

1. **Real Agent Integration**: Connect to actual agent execution
2. **Enhanced Analysis**: Use real compilation/linting tools
3. **Council Integration**: Connect to real council verdicts
4. **CI/CD Integration**: Add to continuous integration pipeline

