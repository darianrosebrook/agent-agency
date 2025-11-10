# CoreML Models Location Decision

**Date:** 2025-01-28  
**Status:** ✅ Models found correctly at workspace root

---

## Current Situation

### Models Location
- **Current:** `agent-agency/models/coreml/` (workspace root)
- **Structure:**
  - `models/coreml/fastvit/FastViTT8F16.mlpackage.mlmodelc/`
  - `models/coreml/mistral/StatefulMistral7BInstructFP16.mlpackage.mlmodelc/`

### Service Manager Status
✅ **Working** - Service manager now finds models at workspace root automatically

The service manager checks multiple locations:
1. Workspace root: `models/coreml` (from `iterations/v3/testing-validation`)
2. V3 directory: `iterations/v3/models/coreml` (if moved)
3. Configured path: `COREML_MODELS_PATH` environment variable
4. Relative paths from current working directory

---

## Recommendation: Keep Models at Workspace Root

### Why Keep at Root?

1. **Shared Across Iterations**
   - Models can be used by v2, v3, and future iterations
   - Avoids duplication
   - Single source of truth

2. **Already Working**
   - Service manager finds them correctly
   - Tests can access them
   - No breaking changes needed

3. **Standard Project Structure**
   - Common pattern: shared resources at root
   - Clear separation: `models/` for models, `iterations/` for code
   - Easier to find and manage

### If You Want to Move to V3

**Option:** Move to `iterations/v3/models/coreml/`

**Pros:**
- More self-contained for v3
- Clearer that these are v3-specific
- Simpler path resolution from v3 code

**Cons:**
- Duplication if other iterations need models
- Breaking change for existing code
- Less flexible for future iterations

**Implementation if Moving:**
1. Move models: `mv models/coreml iterations/v3/models/coreml`
2. Update service manager to prioritize v3 location
3. Update test paths if needed
4. Update documentation

---

## Current Path Resolution

The service manager now automatically finds models by checking:

1. `../../models/coreml` (from `iterations/v3/testing-validation`)
2. `../../../models/coreml` (alternative)
3. `models/coreml` (from workspace root)
4. `COREML_MODELS_PATH` environment variable
5. Current directory relative paths

**Result:** ✅ Finds models at workspace root automatically

---

## Verification

```bash
# Check service status
cd iterations/v3/testing-validation
cargo run --bin ensure_services

# Output:
# ✅ CoreML Models: Running
#    Endpoint: /Users/.../agent-agency/models/coreml
```

---

## Recommendation

**Keep models at workspace root** (`agent-agency/models/coreml/`)

**Reasons:**
1. ✅ Already working correctly
2. ✅ Can be shared across iterations
3. ✅ Standard project structure
4. ✅ No breaking changes needed
5. ✅ Service manager handles path resolution automatically

**If you still want to move:**
- Models can be moved to `iterations/v3/models/coreml/`
- Service manager will find them there too
- But consider duplication if other iterations need them

---

**Status:** Models found correctly, no move needed  
**Action:** Keep current location, service manager handles path resolution

