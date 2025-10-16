# Test Process Management

## Problem Summary

On January 8, 2025, the system accumulated **200+ Jest test processes** and **200+ npm test processes**, causing significant resource consumption and system slowdown.

## Root Causes

1. **Unlimited Parallel Workers**: Jest was configured to use all available CPU cores (`maxWorkers` not set)
2. **Large Test Suite**: 195 test files running in parallel
3. **Hanging Processes**: Tests not properly terminating due to:
   - Database connection pool conflicts
   - `detectOpenHandles: true` causing Jest to wait for cleanup
   - Missing timeout configurations

## Solutions Implemented

### 1. Jest Configuration Updates

**File**: `package.json`

```json
{
  "jest": {
    "maxWorkers": 4, // Limit parallel workers
    "testTimeout": 30000, // 30-second timeout per test
    "detectOpenHandles": false, // Disable open handle detection
    "forceExit": true // Force exit when tests complete
  }
}
```

### 2. New npm Scripts

- `npm run test:coverage` - Limited to 2 workers for development
- `npm run test:coverage:full` - Full parallel execution for CI
- `npm run cleanup:test-processes` - Cleanup script

### 3. Process Cleanup Script

**File**: `scripts/cleanup-test-processes.sh`

Automatically detects and kills hanging test processes:

- Jest processes with `--coverage` or `--detectOpenHandles`
- npm test processes
- Stryker mutation testing processes

## Prevention Guidelines

### For Development

```bash
# Use limited workers for development
npm run test:unit
npm run test:coverage  # Limited to 2 workers

# Check process counts regularly
npm run cleanup:test-processes
```

### For CI/Production

```bash
# Use full parallel execution
npm run test:coverage:full
```

### Manual Cleanup

```bash
# If processes accumulate
pkill -f "jest --coverage"
pkill -f "npm run test"
pkill -f "stryker"

# Or use the cleanup script
./scripts/cleanup-test-processes.sh
```

## Monitoring

### Check Process Counts

```bash
# Count Jest processes
ps aux | grep jest | grep -v grep | wc -l

# Count npm test processes
ps aux | grep "npm.*test" | grep -v grep | wc -l

# Count all Node processes
ps aux | grep node | grep -v grep | wc -l
```

### Warning Signs

- Jest processes > 10
- npm test processes > 10
- Total Node processes > 50 (excluding Cursor IDE)

## Database Connection Management

The test setup uses a centralized connection pool manager to prevent database connection conflicts:

- **Test Environment**: Max 10 connections, 10-second idle timeout
- **Global Setup**: Single pool shared across all tests
- **Global Teardown**: Proper cleanup in `afterAll` hooks

## Best Practices

1. **Use Limited Workers**: Always use `--maxWorkers=2` for development
2. **Regular Cleanup**: Run cleanup script after test sessions
3. **Monitor Resources**: Check process counts before starting new test runs
4. **Proper Timeouts**: Set reasonable timeouts for all async operations
5. **Database Cleanup**: Ensure all database connections are properly closed

## Emergency Procedures

If system becomes unresponsive due to test processes:

1. **Kill All Test Processes**:

   ```bash
   pkill -f "jest"
   pkill -f "npm.*test"
   pkill -f "stryker"
   ```

2. **Check System Resources**:

   ```bash
   top
   htop  # If available
   ```

3. **Restart if Necessary**:
   ```bash
   # Kill all Node processes (WARNING: Will kill Cursor IDE)
   pkill -f "node"
   ```

## Configuration Files Modified

- `package.json` - Jest configuration and npm scripts
- `scripts/cleanup-test-processes.sh` - New cleanup script
- `tests/setup.ts` - Already had proper database cleanup

## Author

@darianrosebrook - January 8, 2025

