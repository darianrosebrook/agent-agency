# Disk Space Cleanup Plan

## Current Disk Usage Analysis

**Total Project Size**: 259GB  
**Available Disk Space**: 191GB free (95% used on 3.6TB drive)

### Top Space Consumers

1. **Root `target/debug/`**: **90GB** ⚠️ (Largest contributor)
2. **`iterations/v3/target/`**: **37GB** ⚠️
3. **`models/coreml/`**: **44GB** (ML models - likely needed)
4. **`models/languages/`**: **2.6GB** (Language models - likely needed)
5. **`iterations/v3/`** (excluding target): **~71GB** (source code + other files)

## Space Requirements for Build

**Estimated space needed for current build**: **5-10GB**
- Debug builds are typically 10-20x larger than release builds
- Current build failure is due to insufficient space, not build size requirements

## Cleanup Strategy

### Phase 1: Safe Cleanup (Immediate - ~127GB freed)

These can be safely deleted and regenerated:

1. **Clean root `target/debug/`** (~90GB)
   ```bash
   cd /Users/darianrosebrook/Desktop/Projects/agent-agency
   cargo clean --target-dir target
   ```
   - **Risk**: Low - Debug builds can be regenerated
   - **Time to regenerate**: 30-60 minutes on rebuild
   - **Space freed**: ~90GB

2. **Clean `iterations/v3/target/`** (~37GB)
   ```bash
   cd /Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3
   cargo clean
   ```
   - **Risk**: Low - Debug builds can be regenerated
   - **Time to regenerate**: 20-40 minutes on rebuild
   - **Space freed**: ~37GB

**Total Phase 1 space freed**: ~127GB

### Phase 2: Selective Cleanup (If needed - ~7.8GB)

3. **Archive or remove old iterations** (~7.8GB)
   - `iterations/v2/`: 7.8GB
   - Consider archiving to external storage if not actively used
   - **Risk**: Medium - May contain useful historical code
   - **Space freed**: ~7.8GB

### Phase 3: Model Optimization (Optional - ~2.6GB)

4. **Review language models** (~2.6GB)
   - `models/languages/`: 2.6GB
   - Check if all language models are actively used
   - Consider removing unused language models
   - **Risk**: Medium - May need to re-download if removed
   - **Space freed**: ~2.6GB (if unused models removed)

### Phase 4: CoreML Model Review (Optional - ~44GB)

5. **Review CoreML models** (~44GB)
   - `models/coreml/`: 44GB
   - These are likely needed for production
   - Only remove if confirmed unused
   - **Risk**: High - May break CoreML features if removed
   - **Space freed**: ~44GB (only if confirmed unused)

## Recommended Immediate Actions

### Quick Win (Execute Now)

```bash
# 1. Clean root target directory (frees ~90GB)
cd /Users/darianrosebrook/Desktop/Projects/agent-agency
cargo clean --target-dir target

# 2. Clean v3 target directory (frees ~37GB)
cd /Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3
cargo clean

# 3. Verify space freed
df -h /Users/darianrosebrook/Desktop/Projects/agent-agency
```

**Expected result**: ~127GB freed, bringing total project size to ~132GB

### After Cleanup

1. **Rebuild only what's needed**:
   ```bash
   cd /Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3/data-interfaces-adapters
   cargo build --bin agent-agency-api-server
   ```

2. **Consider using release builds for production**:
   - Release builds are ~10-20x smaller than debug builds
   - Use `cargo build --release` for production deployments
   - Keep debug builds only during active development

3. **Set up `.gitignore` exclusions** (if not already):
   - Ensure `target/` directories are git-ignored
   - Consider adding `target/` to `.git/info/exclude` if needed

## Long-term Maintenance

### Prevent Future Bloat

1. **Regular cleanup script**:
   ```bash
   # Add to scripts/cleanup.sh
   #!/bin/bash
   find . -type d -name "target" -exec cargo clean --target-dir {} \;
   ```

2. **Use release builds for CI/CD**:
   - CI/CD should use `cargo build --release`
   - Only use debug builds for local development

3. **Archive old iterations**:
   - Move `iterations/v2/` to external storage or archive
   - Keep only active iterations in main project

4. **Monitor disk usage**:
   ```bash
   # Add to scripts/check-disk-usage.sh
   du -sh */target 2>/dev/null | sort -hr
   ```

## Space Requirements by Build Type

| Build Type | Typical Size | Regeneration Time |
|------------|-------------|-------------------|
| Debug (full) | 90-127GB | 30-60 minutes |
| Debug (incremental) | 1-5GB | 2-5 minutes |
| Release (full) | 5-10GB | 20-40 minutes |
| Release (incremental) | 100-500MB | 1-3 minutes |

**Recommendation**: Use incremental builds during development, full builds only when needed.

## Verification

After cleanup, verify:
1. Disk space is sufficient (>10GB free)
2. Build can complete successfully
3. No critical files were removed

```bash
# Check disk space
df -h /Users/darianrosebrook/Desktop/Projects/agent-agency

# Verify build works
cd /Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3/data-interfaces-adapters
cargo build --bin agent-agency-api-server
```

