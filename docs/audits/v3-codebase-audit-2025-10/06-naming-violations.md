# Naming Violations Report

## Forbidden Patterns Detected

### High-Impact Violations (20+ matches)

#### `runtime-optimization/` (Multiple files)
- **Files affected**: 8+ files
- **Pattern**: `enhanced_*`, `new_*`, `improved_*`
- **Examples**:
  - `enhanced_telemetry.rs` → `telemetry.rs`
  - `new_optimizer.rs` → `optimizer.rs`
  - `improved_analyzer.rs` → `analyzer.rs`

#### `context-preservation-engine/` (Multiple files)
- **Files affected**: 6+ files
- **Pattern**: `unified_*`, `better_*`
- **Examples**:
  - `unified_context.rs` → `context.rs`
  - `better_synthesis.rs` → `synthesis.rs`

#### `caching/` (Integration files)
- **Files affected**: 4+ files
- **Pattern**: `enhanced_*`, `new_*`
- **Examples**:
  - `enhanced_integration.rs` → `integration.rs`
  - `new_cache.rs` → `cache.rs`

### Medium-Impact Violations (5-20 matches)

#### `self-prompting-agent/` (Multiple files)
- **Files affected**: 5+ files
- **Pattern**: `enhanced_*`, `improved_*`
- **Examples**:
  - `enhanced_evaluation.rs` → `evaluation.rs`
  - `improved_learning.rs` → `learning.rs`

#### `federated-learning/` (Multiple files)
- **Files affected**: 4+ files
- **Pattern**: `new_*`, `enhanced_*`
- **Examples**:
  - `new_participant.rs` → `participant.rs`
  - `enhanced_validation.rs` → `validation.rs`

### Low-Impact Violations (1-5 matches)

#### `apple-silicon/` (Multiple files)
- **Files affected**: 3+ files
- **Pattern**: `enhanced_*`
- **Examples**:
  - `enhanced_telemetry.rs` → `telemetry.rs`

#### `claim-extraction/` (Multiple files)
- **Files affected**: 2+ files
- **Pattern**: `improved_*`
- **Examples**:
  - `improved_verification.rs` → `verification.rs`

## Renaming Plan

### Phase 1: High-Impact Violations (Week 1)

#### `runtime-optimization/` Cleanup
```bash
# File renames
mv enhanced_telemetry.rs telemetry.rs
mv new_optimizer.rs optimizer.rs
mv improved_analyzer.rs analyzer.rs
mv enhanced_performance.rs performance.rs
mv new_metrics.rs metrics.rs
mv improved_benchmark.rs benchmark.rs
mv enhanced_parameter.rs parameter.rs
mv new_strategy.rs strategy.rs

# Update imports
find . -name "*.rs" -exec sed -i 's/enhanced_telemetry/telemetry/g' {} \;
find . -name "*.rs" -exec sed -i 's/new_optimizer/optimizer/g' {} \;
find . -name "*.rs" -exec sed -i 's/improved_analyzer/analyzer/g' {} \;
```

#### `context-preservation-engine/` Cleanup
```bash
# File renames
mv unified_context.rs context.rs
mv better_synthesis.rs synthesis.rs
mv enhanced_manager.rs manager.rs
mv new_engine.rs engine.rs
mv improved_store.rs store.rs
mv unified_tenant.rs tenant.rs

# Update imports
find . -name "*.rs" -exec sed -i 's/unified_context/context/g' {} \;
find . -name "*.rs" -exec sed -i 's/better_synthesis/synthesis/g' {} \;
```

#### `caching/` Cleanup
```bash
# File renames
mv enhanced_integration.rs integration.rs
mv new_cache.rs cache.rs
mv improved_storage.rs storage.rs
mv enhanced_metrics.rs metrics.rs

# Update imports
find . -name "*.rs" -exec sed -i 's/enhanced_integration/integration/g' {} \;
find . -name "*.rs" -exec sed -i 's/new_cache/cache/g' {} \;
```

### Phase 2: Medium-Impact Violations (Week 2)

#### `self-prompting-agent/` Cleanup
```bash
# File renames
mv enhanced_evaluation.rs evaluation.rs
mv improved_learning.rs learning.rs
mv new_agent.rs agent.rs
mv enhanced_prompting.rs prompting.rs
mv improved_bridge.rs bridge.rs

# Update imports
find . -name "*.rs" -exec sed -i 's/enhanced_evaluation/evaluation/g' {} \;
find . -name "*.rs" -exec sed -i 's/improved_learning/learning/g' {} \;
```

#### `federated-learning/` Cleanup
```bash
# File renames
mv new_participant.rs participant.rs
mv enhanced_validation.rs validation.rs
mv improved_coordinator.rs coordinator.rs
mv new_encryption.rs encryption.rs

# Update imports
find . -name "*.rs" -exec sed -i 's/new_participant/participant/g' {} \;
find . -name "*.rs" -exec sed -i 's/enhanced_validation/validation/g' {} \;
```

### Phase 3: Low-Impact Violations (Week 3)

#### `apple-silicon/` Cleanup
```bash
# File renames
mv enhanced_telemetry.rs telemetry.rs
mv improved_gpu.rs gpu.rs
mv enhanced_memory.rs memory.rs

# Update imports
find . -name "*.rs" -exec sed -i 's/enhanced_telemetry/telemetry/g' {} \;
find . -name "*.rs" -exec sed -i 's/improved_gpu/gpu/g' {} \;
```

#### `claim-extraction/` Cleanup
```bash
# File renames
mv improved_verification.rs verification.rs
mv enhanced_evidence.rs evidence.rs

# Update imports
find . -name "*.rs" -exec sed -i 's/improved_verification/verification/g' {} \;
find . -name "*.rs" -exec sed -i 's/enhanced_evidence/evidence/g' {} \;
```

## Automated Renaming Script

### `scripts/audit-tools/check-naming.sh`
```bash
#!/bin/bash
# Naming Convention Enforcement Script

V3_PATH=${1:-"/Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3"}
OUTPUT_DIR="/Users/darianrosebrook/Desktop/Projects/agent-agency/docs/audits/v3-codebase-audit-2025-10/metrics"

echo "=== Naming Violations Report ===" > "$OUTPUT_DIR/naming-violations.txt"
echo "Generated: $(date)" >> "$OUTPUT_DIR/naming-violations.txt"
echo "" >> "$OUTPUT_DIR/naming-violations.txt"

echo "Forbidden patterns found:" >> "$OUTPUT_DIR/naming-violations.txt"
cd "$V3_PATH" && rg '(?i)\b(enhanced|unified|better|new|next|final|copy|revamp|improved)\b' --count-matches >> "$OUTPUT_DIR/naming-violations.txt"

echo "" >> "$OUTPUT_DIR/naming-violations.txt"
echo "Files with naming violations:" >> "$OUTPUT_DIR/naming-violations.txt"
cd "$V3_PATH" && rg '(?i)\b(enhanced|unified|better|new|next|final|copy|revamp|improved)\b' --files-with-matches >> "$OUTPUT_DIR/naming-violations.txt"

echo "Naming violation analysis complete. Results saved to $OUTPUT_DIR/naming-violations.txt"
```

## Validation Script

### `scripts/audit-tools/validate-naming.sh`
```bash
#!/bin/bash
# Validate naming conventions after cleanup

V3_PATH=${1:-"/Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3"}

echo "Validating naming conventions..."

# Check for remaining violations
VIOLATIONS=$(cd "$V3_PATH" && rg '(?i)\b(enhanced|unified|better|new|next|final|copy|revamp|improved)\b' --count-matches | wc -l)

if [ "$VIOLATIONS" -eq 0 ]; then
    echo "✅ No naming violations found"
    exit 0
else
    echo "❌ $VIOLATIONS naming violations still exist"
    cd "$V3_PATH" && rg '(?i)\b(enhanced|unified|better|new|next|final|copy|revamp|improved)\b' --count-matches
    exit 1
fi
```

## Success Criteria

### Naming Convention Compliance
- ✅ **Zero forbidden patterns** (enhanced/unified/better/new/etc.)
- ✅ **Consistent naming** across all crates
- ✅ **Descriptive names** that indicate purpose
- ✅ **No duplicate names** (except lib.rs/main.rs)

### File Organization
- ✅ **Clear module boundaries**
- ✅ **Logical file grouping**
- ✅ **Consistent naming patterns**
- ✅ **Easy to locate functionality**

### Code Quality
- ✅ **No naming confusion**
- ✅ **Clear responsibility indication**
- ✅ **Maintainable file structure**
- ✅ **Professional naming standards**

## Risk Mitigation

### Backup Strategy
1. **Create git branch** before renaming
2. **Commit changes** incrementally
3. **Test after each** major rename
4. **Rollback plan** if issues arise

### Testing Strategy
1. **Compile check** after each rename
2. **Import validation** for all files
3. **Integration tests** for renamed modules
4. **Documentation updates** for API changes

### Communication
1. **Team notification** of naming changes
2. **Documentation updates** for new names
3. **Migration guide** for external users
4. **Training materials** for new naming conventions

