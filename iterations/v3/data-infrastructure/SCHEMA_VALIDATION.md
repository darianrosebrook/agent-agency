# Database Schema Validation Guide

This document describes the comprehensive schema validation system for ensuring database schemas match model definitions.

## Overview

The schema validation system validates that:
- All tables from migrations 014 and 015 exist
- Field types match model definitions in `models.rs`
- Nullability constraints match
- Foreign key relationships are correct
- No unexpected fields exist (warns but doesn't fail)

## Usage Methods

### 1. Standalone Binary

Run the validation binary directly:

```bash
# Using command line argument
cargo run --bin validate_schema -- --database-url postgresql://user:password@localhost:5432/dbname

# Using environment variable
DATABASE_URL=postgresql://user:password@localhost:5432/dbname cargo run --bin validate_schema

# With feature flag enabled (recommended)
cargo run --features schema-validation --bin validate_schema -- --database-url $DATABASE_URL
```

### 2. Programmatic API

Use the validation functions in your code:

```rust
use data_infrastructure::scripts::validate_schema;
use sqlx::PgPool;

// Validate all schemas
let is_valid = validate_schema::validate_all_schemas(&pool).await?;

// Validate specific migration tables
let migration_014_valid = validate_schema::validate_migration_014_tables(&pool).await?;
let migration_015_valid = validate_schema::validate_migration_015_tables(&pool).await?;

// Validate foreign keys
let fks_valid = validate_schema::validate_foreign_keys(&pool).await?;
```

### 3. Integrated into Database Initialization

The validation is integrated into `database_init.rs`:

```rust
use data_infrastructure::database_init;

// Initialize database with optional schema verification
let db_client = database_init::initialize_database(config).await?;

// Manually verify schema
let is_valid = database_init::verify_schema(pool).await?;

// Detailed validation (always uses full validation)
let is_valid = database_init::verify_schema_detailed(pool).await?;
```

### 4. Automatic Verification After Migrations

Enable automatic schema verification after migrations:

```bash
VERIFY_SCHEMA_AFTER_MIGRATION=true cargo run
```

Or in code:

```rust
std::env::set_var("VERIFY_SCHEMA_AFTER_MIGRATION", "true");
let db_client = database_init::initialize_database(config).await?;
```

## Feature Flag

The comprehensive validation is behind a feature flag to allow optional compilation:

```toml
# In Cargo.toml
[features]
schema-validation = []  # Enable comprehensive schema validation
```

**With feature flag** (recommended):
- Uses full validation script
- Checks all field types, nullability, foreign keys
- Provides detailed error messages

**Without feature flag**:
- Falls back to basic table existence check
- Faster but less comprehensive
- Still checks critical tables

## CI/CD Integration

### GitHub Actions Example

```yaml
- name: Validate Database Schema
  run: |
    cargo run --features schema-validation --bin validate_schema -- \
      --database-url ${{ secrets.DATABASE_URL }}
  env:
    DATABASE_URL: ${{ secrets.DATABASE_URL }}
```

### Pre-commit Hook Example

```bash
#!/bin/bash
# .git/hooks/pre-commit

if [ -n "$DATABASE_URL" ]; then
  cargo run --features schema-validation --bin validate_schema -- \
    --database-url "$DATABASE_URL" || exit 1
fi
```

### Database Setup Script Integration

Add to your database setup scripts:

```javascript
// After running migrations
const { execSync } = require('child_process');

try {
  execSync(
    'cargo run --features schema-validation --bin validate_schema -- ' +
    `--database-url ${process.env.DATABASE_URL}`,
    { stdio: 'inherit' }
  );
  console.log('Schema validation passed');
} catch (error) {
  console.error('Schema validation failed');
  process.exit(1);
}
```

## Exit Codes

- `0`: All validations passed
- `1`: One or more validations failed

## Validation Details

### Migration 014 Tables Validated

- `workers` - Worker agent definitions
- `judges` - Judge agent definitions  
- `tasks` - Task definitions
- `task_executions` - Task execution records
- `council_verdicts` - Council verdict records
- `judge_evaluations` - Individual judge evaluations
- `debate_sessions` - Debate session records

### Migration 015 Tables Validated

- `saved_queries` - Saved database queries
- `provenance_entries` - Provenance tracking entries
- `audit_trail_entries` - Audit trail entries
- `audit_logs` - Audit log entries

### What Gets Checked

For each table:
1. **Existence**: Table exists in database
2. **Fields**: All expected fields exist
3. **Types**: Field types match model definitions
4. **Nullability**: NULL/NOT NULL constraints match
5. **Foreign Keys**: All expected foreign key relationships exist

### Warnings vs Errors

- **Errors**: Missing tables, missing fields, type mismatches, nullability mismatches
- **Warnings**: Unexpected fields (fields in DB not in models)

## Troubleshooting

### Validation Fails After Migration

If validation fails after running migrations:

1. Check migration logs: `SELECT * FROM migration_log ORDER BY applied_at DESC`
2. Verify migration files are correct
3. Check for manual schema changes that weren't migrated
4. Run validation with detailed logging: `RUST_LOG=debug cargo run --bin validate_schema`

### Type Mismatches

If you see type mismatches:

1. Check PostgreSQL type mapping:
   - `character varying` = `VARCHAR`
   - `timestamp with time zone` = `TIMESTAMPTZ`
   - `real` = `FLOAT`
   - `jsonb` = `JSONB`

2. Verify model definitions in `models.rs` match migration SQL

3. Check for type normalization issues (the validator normalizes some types)

### Foreign Key Issues

If foreign keys are missing:

1. Verify migrations ran successfully
2. Check migration SQL for foreign key constraints
3. Verify referenced tables exist
4. Check constraint names match expected patterns

## Best Practices

1. **Always validate after migrations** in development
2. **Enable validation in CI/CD** to catch schema drift
3. **Use feature flag** for production builds (optional but recommended)
4. **Run validation before deployments** to catch issues early
5. **Fix validation failures immediately** - don't ignore schema drift

## Related Documentation

- [Migrations Guide](../migrations/README.md)
- [Model Definitions](../src/models.rs)
- [Database Operations](../src/database_operations.rs)













