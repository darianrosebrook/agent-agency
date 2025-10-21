# Agent Agency V3 Tutorial - Getting Started

**Step-by-step guide to using the Constitutional AI System**

---

## 🎯 Tutorial Overview

This tutorial walks you through using Agent Agency V3 to execute autonomous tasks with constitutional governance. You'll learn how to submit tasks, monitor progress, and intervene when needed.

**Time**: ~30 minutes  
**Level**: Beginner  
**Prerequisites**: Docker, Rust, Node.js

---

## 📋 Tutorial Scenario

**Task**: Implement a user authentication system with JWT tokens

**Requirements**:
- User registration and login endpoints
- JWT token generation and validation
- Password hashing and secure storage
- Proper error handling and validation
- Database persistence for users

**Risk Level**: Tier 2 (authentication system with data persistence)

---

## Step 1: Set Up the System

### Start Required Services

```bash
# 1. Start PostgreSQL database
docker run -d --name agent-agency-db \
  -e POSTGRES_PASSWORD=mysecretpassword \
  -e POSTGRES_DB=agent_agency_v3 \
  -p 5432:5432 \
  postgres:15

# 2. Navigate to V3 directory
cd iterations/v3

# 3. Run database migrations
cargo run --bin migrate

# 4. Install CAWS Git hooks (optional, for provenance tracking)
./scripts/install-git-hooks.sh
```

### Start the Core Services

```bash
# Terminal 1: Start API server
cargo run --bin api-server

# Terminal 2: Start worker service
cargo run --bin agent-agency-worker

# Terminal 3: Start web dashboard (optional)
cd apps/web-dashboard
npm install
npm run dev
```

**Expected Output:**
```
🔧 Starting Agent Agency API Server
📡 Server: 127.0.0.1:8080
✅ API server ready at http://127.0.0.1:8080

🔧 Starting Agent Agency Worker
📡 Server: 127.0.0.1:8081
👷 Worker ID: default-worker
✅ Worker ready at http://127.0.0.1:8081
```

## Step 2: Submit Your First Task

### Using the CLI

```bash
# Submit task in auto mode (recommended for most cases)
cargo run --bin agent-agency-cli execute \
  "Implement user authentication system with JWT tokens, user registration, login, and secure password storage" \
  --mode auto \
  --risk-tier 2 \
  --watch

# Alternative: Dry-run mode for safe testing
cargo run --bin agent-agency-cli execute \
  "Design the user authentication API schema" \
  --mode dry-run
```

### Using the API Directly

```bash
# Submit via REST API
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Implement user authentication system with JWT tokens, user registration, login, and secure password storage",
    "execution_mode": "auto",
    "max_iterations": 10,
    "risk_tier": 2
  }'
```

**Expected Response:**
```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "accepted",
  "message": "Task submitted successfully"
}
```

## Step 3: Monitor Task Progress

### Real-time Monitoring with CLI

When you use `--watch`, the CLI will show live progress:

```
🚀 Agent Agency V3 - Autonomous Execution
═══════════════════════════════════════════

🎯 Task: Implement user authentication system...
🔢 Task ID: 550e8400-e29b-41d4-a716-446655440000
⚡ Mode: AUTO (Quality Gates Enabled)
🎚️  Risk Tier: 2

📋 Phase: Planning and validation
   ⏳ Analyzing requirements...
   ✅ Council review passed
   ✅ CAWS compliance validated

📋 Phase: Worker execution
   🔧 Executing implementation...
   📊 Progress: 65%
   ✅ Code generation completed
   ✅ Tests written and passing

📋 Phase: Quality assurance
   🧪 Running test suite...
   📊 Coverage: 85% (Target: 80%)
   ✅ All quality gates passed

🎉 Task completed successfully!
⏱️  Total time: 2m 34s
```

### API Monitoring

```bash
# Get task status
TASK_ID="550e8400-e29b-41d4-a716-446655440000"
curl http://localhost:8080/api/v1/tasks/$TASK_ID

# Get detailed progress
curl http://localhost:8080/api/v1/tasks/$TASK_ID/result
```

### Web Dashboard Monitoring

Open http://localhost:3000 to see:
- Live task progress visualization
- System metrics and SLO status
- Database exploration tools
- Real-time alerts and notifications

## Step 4: Real-time Intervention

### Pause and Resume Tasks

```bash
# Pause execution
cargo run --bin agent-agency-cli intervene pause $TASK_ID

# Resume execution
cargo run --bin agent-agency-cli intervene resume $TASK_ID
```

### Override Council Decisions

```bash
# Override verdict in strict mode
cargo run --bin agent-agency-cli intervene override $TASK_ID --verdict accept --reason "Approved by human review"
```

### Cancel Running Tasks

```bash
# Cancel task execution
cargo run --bin agent-agency-cli intervene cancel $TASK_ID
```

## Step 5: Review Results and Provenance

### Check Task Artifacts

```bash
# Get complete task results
curl http://localhost:8080/api/v1/tasks/$TASK_ID/result

# View generated code and tests
# (Results will show file paths and changes made)
```

### Verify Provenance

```bash
# Check provenance records
curl http://localhost:8080/api/v1/provenance

# Verify specific commit
curl http://localhost:8080/api/v1/provenance/verify/$(git rev-parse HEAD)
```

## Step 6: Handle Quality Gate Exceptions

### Create Waiver for Special Cases

```bash
# Create waiver for emergency deployment
curl -X POST http://localhost:8080/api/v1/waivers \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Emergency security patch",
    "reason": "security_patch",
    "description": "Deploying critical security fix without full test coverage",
    "gates": ["test-coverage"],
    "approved_by": "security-team",
    "impact_level": "high",
    "mitigation_plan": "Security review completed, monitoring in place"
  }'
```

### Approve Waiver

```bash
# Approve the waiver
WAIVER_ID="your-waiver-id"
curl -X POST http://localhost:8080/api/v1/waivers/$WAIVER_ID/approve
```

## Troubleshooting

### Common Issues

**Worker not responding:**
```bash
# Check worker logs
ps aux | grep agent-agency-worker
# Restart worker if needed
cargo run --bin agent-agency-worker
```

**Database connection failed:**
```bash
# Check database status
docker ps | grep postgres
# Reset database if needed
docker restart agent-agency-db
```

**Task stuck in pending:**
```bash
# Check API server logs
curl http://localhost:8080/health
# Restart API server if needed
cargo run --bin api-server
```

### Getting Help

```bash
# CLI help
cargo run --bin agent-agency-cli --help

# API health check
curl http://localhost:8080/health

# System metrics
curl http://localhost:8080/metrics
```

## Next Steps

- Try different execution modes (strict, auto, dry-run)
- Experiment with task intervention capabilities
- Explore the web dashboard features
- Learn about CAWS compliance and quality gates
- Set up SLO monitoring and alerting

---

**🎉 Congratulations!** You've successfully used Agent Agency V3 to execute an autonomous task with constitutional governance. The system provides complete oversight, real-time control, and comprehensive audit trails for all AI operations.

```bash
caws validate --suggestions
```

**Expected**: ✅ Valid spec

---

## Step 2: Plan the Implementation

### Copy Feature Template

```bash
cp .caws/templates/feature.plan.md docs/plans/PREF-001.md
```

### Fill in Feature Plan

Edit `docs/plans/PREF-001.md`:

```markdown
# PREF-001: Add User Preferences Storage

## Problem Statement
Users need to persist their preferences (theme, language, etc.) across browser sessions.

## Proposed Solution
Implement a preferences system using localStorage with TypeScript types and validation.

## Technical Approach
- Create TypeScript interfaces for preferences
- Implement storage layer with error handling
- Add validation and defaults
- Integrate with existing UI components

## Files to Create/Modify
1. `src/types/preferences.ts` - TypeScript interfaces
2. `src/preferences/storage.ts` - Storage implementation
3. `src/preferences/validation.ts` - Input validation
4. `src/preferences/hooks.ts` - React hooks (if applicable)
5. `tests/preferences/storage.test.ts` - Unit tests
6. `tests/preferences/validation.test.ts` - Validation tests

## Testing Strategy
- Unit tests for storage operations
- Validation tests for edge cases
- Integration tests for error handling
- Manual testing for UI integration

## Risk Assessment
- **Data Loss**: localStorage can be cleared → Mitigation: Graceful degradation
- **Type Safety**: Runtime validation needed → Mitigation: TypeScript + runtime checks
- **Browser Support**: localStorage availability → Mitigation: Feature detection

## Rollback Plan
1. Remove preference-related imports
2. Delete preference files
3. Revert any UI changes
4. Clear localStorage keys (optional)
```

---

## Step 3: Write Tests First (TDD)

### Create Test Files

**`tests/preferences/storage.test.ts`**:
```typescript
import { describe, it, expect, beforeEach } from '@jest/globals';
import { PreferencesStorage } from '../../src/preferences/storage';

describe('PreferencesStorage', () => {
  let storage: PreferencesStorage;

  beforeEach(() => {
    storage = new PreferencesStorage();
    localStorage.clear();
  });

  it('should store and retrieve preferences', () => {
    const prefs = { theme: 'dark', language: 'en' };
    storage.save(prefs);

    const retrieved = storage.load();
    expect(retrieved).toEqual(prefs);
  });

  it('should handle localStorage unavailable', () => {
    // Mock localStorage unavailable
    const originalGetItem = Storage.prototype.getItem;
    Storage.prototype.getItem = () => { throw new Error('Storage unavailable'); };

    expect(() => storage.load()).not.toThrow();
    expect(storage.load()).toEqual({});

    Storage.prototype.getItem = originalGetItem;
  });

  it('should validate preferences before saving', () => {
    const invalidPrefs = { theme: 'invalid' };
    expect(() => storage.save(invalidPrefs)).toThrow('Invalid theme');
  });
});
```

**`tests/preferences/validation.test.ts`**:
```typescript
import { describe, it, expect } from '@jest/globals';
import { validatePreferences, DEFAULT_PREFERENCES } from '../../src/preferences/validation';

describe('validatePreferences', () => {
  it('should accept valid preferences', () => {
    const validPrefs = { theme: 'dark', language: 'en' };
    expect(validatePreferences(validPrefs)).toBe(true);
  });

  it('should reject invalid theme', () => {
    const invalidPrefs = { theme: 'invalid' };
    expect(() => validatePreferences(invalidPrefs)).toThrow();
  });

  it('should provide defaults for missing values', () => {
    const partialPrefs = { theme: 'light' };
    const result = validatePreferences(partialPrefs);
    expect(result).toEqual({
      theme: 'light',
      language: DEFAULT_PREFERENCES.language
    });
  });
});
```

### Run Tests (Should Fail)

```bash
npm test
```

**Expected**: Tests fail because implementation doesn't exist yet

---

## Step 4: Implement the Feature

### Create TypeScript Interfaces

**`src/types/preferences.ts`**:
```typescript
export interface UserPreferences {
  theme: 'light' | 'dark' | 'auto';
  language: string;
  notifications: boolean;
  autoSave: boolean;
}

export const DEFAULT_PREFERENCES: UserPreferences = {
  theme: 'light',
  language: 'en',
  notifications: true,
  autoSave: true,
};
```

### Implement Validation

**`src/preferences/validation.ts`**:
```typescript
import { UserPreferences, DEFAULT_PREFERENCES } from '../types/preferences';

export function validatePreferences(prefs: Partial<UserPreferences>): UserPreferences {
  const validated: UserPreferences = { ...DEFAULT_PREFERENCES };

  if (prefs.theme) {
    if (!['light', 'dark', 'auto'].includes(prefs.theme)) {
      throw new Error(`Invalid theme: ${prefs.theme}`);
    }
    validated.theme = prefs.theme;
  }

  if (prefs.language) {
    if (typeof prefs.language !== 'string' || prefs.language.length < 2) {
      throw new Error('Invalid language code');
    }
    validated.language = prefs.language;
  }

  if (typeof prefs.notifications === 'boolean') {
    validated.notifications = prefs.notifications;
  }

  if (typeof prefs.autoSave === 'boolean') {
    validated.autoSave = prefs.autoSave;
  }

  return validated;
}
```

### Implement Storage Layer

**`src/preferences/storage.ts`**:
```typescript
import { UserPreferences, DEFAULT_PREFERENCES } from '../types/preferences';
import { validatePreferences } from './validation';

const STORAGE_KEY = 'user-preferences';

export class PreferencesStorage {
  save(preferences: Partial<UserPreferences>): void {
    try {
      const validated = validatePreferences(preferences);
      localStorage.setItem(STORAGE_KEY, JSON.stringify(validated));
    } catch (error) {
      // Validation error or storage error
      throw new Error(`Failed to save preferences: ${error.message}`);
    }
  }

  load(): UserPreferences {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (!stored) return DEFAULT_PREFERENCES;

      const parsed = JSON.parse(stored);
      return validatePreferences(parsed);
    } catch (error) {
      // Storage unavailable or corrupted data
      console.warn('Failed to load preferences, using defaults:', error.message);
      return DEFAULT_PREFERENCES;
    }
  }

  clear(): void {
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch (error) {
      // Storage unavailable, ignore
    }
  }
}
```

### Run Tests (Should Pass)

```bash
npm test
```

**Expected**: All tests pass ✅

---

## Step 5: Verify Quality Gates

### Check Change Budget

```bash
# Count files changed
find src/ tests/ -name "*.ts" -newer .caws/working-spec.yaml | wc -l

# Count lines changed
find src/ tests/ -name "*.ts" -newer .caws/working-spec.yaml -exec wc -l {} + | tail -1
```

**Expected**: Within budget (8 files, 200 lines)

### Run Validation

```bash
caws validate --quiet
```

**Expected**: ✅ No validation errors

### Check Coverage

```bash
npm run test:coverage
```

**Expected**: ≥80% branch coverage for Tier 2

### Manual Verification

1. **Acceptance Criteria**:
   - ✅ Theme preference persists across sessions
   - ✅ Invalid values are rejected with defaults
   - ✅ Storage errors handled gracefully

2. **Invariants**:
   - ✅ Preferences are validated before storage
   - ✅ Invalid preferences fall back to defaults
   - ✅ Storage errors don't crash the application
   - ✅ TypeScript types prevent runtime errors

---

## Step 6: Update Documentation

### Update Working Spec

If scope changed during implementation, update `.caws/working-spec.yaml`:

```yaml
# Add any new files to scope.in
scope:
  in: ["src/preferences/", "src/types/", "tests/", "src/hooks/"]
```

### Generate Final Report

```bash
# Generate completion report
node apps/tools/caws/completion-report.js PREF-001
```

---

## Step 7: Create PR

### Use PR Template

```bash
cp .caws/templates/pr.md docs/prs/PREF-001.md
```

### Fill in PR Template

```markdown
## Title: feat: Add user preferences storage

## Description
Implements persistent user preferences with localStorage, including theme and language settings.

## Working Spec
- **ID**: PREF-001
- **Risk Tier**: 2
- **Change Budget**: 8 files, 200 lines
- **Actual Changes**: 6 files, 145 lines

## Files Changed
- `src/types/preferences.ts` (25 lines)
- `src/preferences/storage.ts` (45 lines)
- `src/preferences/validation.ts` (35 lines)
- `tests/preferences/storage.test.ts` (60 lines)
- `tests/preferences/validation.test.ts` (40 lines)
- `.caws/working-spec.yaml` (minor updates)

## Testing
- Unit tests: 100% coverage
- Validation tests: All edge cases covered
- Manual testing: All acceptance criteria verified

## Quality Gates
- ✅ Validation: Working spec valid
- ✅ Coverage: 85% branch coverage
- ✅ Tests: All passing
- ✅ Manual Review: Ready for review

## Rollback Plan
1. Remove preference imports from components
2. Delete preference files
3. Clear localStorage keys
4. Revert UI integration changes

## Acceptance Criteria
- [x] Theme preference persists across sessions
- [x] Invalid preferences fall back to defaults
- [x] Storage errors handled gracefully
- [x] TypeScript types prevent runtime errors
```

---

## 🎉 Tutorial Complete!

You've successfully implemented a CAWS-managed feature with:

- ✅ Proper planning with working spec
- ✅ Test-driven development
- ✅ Quality gates and validation
- ✅ Comprehensive documentation
- ✅ Rollback planning

### What You Learned

1. **Planning First**: Working spec prevents scope creep
2. **Tests as Safety**: TDD catches issues early
3. **Validation Matters**: Automated checks maintain quality
4. **Documentation Pays**: Clear PRs speed up reviews
5. **Rollback Ready**: Always plan for failures

### Next Steps

1. **Apply to Real Work**: Use this pattern for your next feature
2. **Customize Templates**: Adapt templates to your team's needs
3. **Set Up CI**: Configure quality gates in your pipeline
4. **Team Training**: Share this approach with your team

### Resources

- **Working Spec**: `.caws/working-spec.yaml`
- **Tests**: `tests/preferences/`
- **Documentation**: `docs/plans/PREF-001.md`
- **Templates**: `.caws/templates/`

---

## 🧪 Bonus: Advanced Testing

For more robust testing, add property-based tests:

```typescript
import { fc } from 'fast-check';

describe('PreferencesStorage - Property Tests', () => {
  it('should round-trip any valid preferences', () => {
    fc.assert(
      fc.property(
        fc.record({
          theme: fc.constantFrom('light', 'dark', 'auto'),
          language: fc.string({ minLength: 2, maxLength: 5 }),
          notifications: fc.boolean(),
          autoSave: fc.boolean(),
        }),
        (prefs) => {
          const storage = new PreferencesStorage();
          storage.save(prefs);
          const loaded = storage.load();
          expect(loaded).toEqual({ ...DEFAULT_PREFERENCES, ...prefs });
        }
      )
    );
  });
});
```

This ensures your storage works correctly with any valid input combination.

---

**Happy coding with CAWS! 🎯**

**Tutorial Version**: 1.0  
**CAWS Version**: 3.1.0  
**Last Updated**: October 2, 2025
