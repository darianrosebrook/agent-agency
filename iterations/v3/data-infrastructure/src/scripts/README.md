# Database Schema Validation

This module contains scripts for validating that the database schema matches the model definitions in `models.rs`.

## Usage

### Command Line Tool

Run the validation binary:

```bash
# Using command line argument
cargo run --bin validate_schema -- --database-url postgresql://user:password@localhost:5432/dbname

# Using environment variable
DATABASE_URL=postgresql://user:password@localhost:5432/dbname cargo run --bin validate_schema
```

### Programmatic Usage

```rust
use data_infrastructure::scripts::validate_schema;
use sqlx::PgPool;

let pool = PgPool::connect(&database_url).await?;
let is_valid = validate_schema::validate_all_schemas(&pool).await?;
```

## What It Validates

The validation script checks:

1. **Table Existence**: All tables from migrations 014 and 015 exist
2. **Field Types**: All fields match expected types from `models.rs`
3. **Nullability**: Field nullability matches model definitions
4. **Foreign Keys**: All expected foreign key relationships exist
5. **Unexpected Fields**: Warns about fields in database not in models (doesn't fail)

## Tables Validated

### Migration 014 (Agent Management)
- `workers`
- `judges`
- `tasks`
- `task_executions`
- `council_verdicts`
- `judge_evaluations`
- `debate_sessions`

### Migration 015 (Observation & API)
- `saved_queries`
- `provenance_entries`
- `audit_trail_entries`
- `audit_logs`

## Exit Codes

- `0`: All validations passed
- `1`: One or more validations failed

## Integration

This validation can be integrated into:

- CI/CD pipelines (pre-deployment checks)
- Database setup scripts (post-migration verification)
- Development workflows (local schema verification)

Example CI integration:

```yaml
- name: Validate Database Schema
  run: |
    cargo run --bin validate_schema -- --database-url $DATABASE_URL
```

