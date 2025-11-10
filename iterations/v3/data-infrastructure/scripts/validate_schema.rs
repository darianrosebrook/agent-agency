//! Schema validation script
//!
//! Validates that database schema matches model definitions in models.rs
//! Checks table existence, field types, constraints, and relationships.
//!
//! Author: @darianrosebrook

use anyhow::{Context, Result};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Expected table schemas based on models.rs
struct TableSchema {
    name: &'static str,
    fields: Vec<FieldSchema>,
    indexes: Vec<&'static str>,
    foreign_keys: Vec<ForeignKeySchema>,
}

struct FieldSchema {
    name: &'static str,
    data_type: &'static str,
    nullable: bool,
}

struct ForeignKeySchema {
    field: &'static str,
    references_table: &'static str,
    references_field: &'static str,
}

/// Validate all tables from migration 014
pub async fn validate_migration_014_tables(pool: &PgPool) -> Result<bool> {
    info!("Validating migration 014 tables (agent management)");

    let mut all_valid = true;

    // Validate workers table
    all_valid &= validate_table(
        pool,
        "workers",
        &[
            FieldSchema {
                name: "id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "name",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "worker_type",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "specialty",
                data_type: "character varying",
                nullable: true,
            },
            FieldSchema {
                name: "model_name",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "endpoint",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "capabilities",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "performance_history",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "is_active",
                data_type: "boolean",
                nullable: false,
            },
            FieldSchema {
                name: "created_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "updated_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
        ],
    )
    .await?;

    // Validate judges table
    all_valid &= validate_table(
        pool,
        "judges",
        &[
            FieldSchema {
                name: "id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "name",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "model_name",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "endpoint",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "weight",
                data_type: "real",
                nullable: false,
            },
            FieldSchema {
                name: "timeout_ms",
                data_type: "integer",
                nullable: false,
            },
            FieldSchema {
                name: "optimization_target",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "is_active",
                data_type: "boolean",
                nullable: false,
            },
            FieldSchema {
                name: "created_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "updated_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
        ],
    )
    .await?;

    // Validate tasks table
    all_valid &= validate_table(
        pool,
        "tasks",
        &[
            FieldSchema {
                name: "id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "title",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "description",
                data_type: "text",
                nullable: false,
            },
            FieldSchema {
                name: "risk_tier",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "scope",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "acceptance_criteria",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "context",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "caws_spec",
                data_type: "jsonb",
                nullable: true,
            },
            FieldSchema {
                name: "status",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "assigned_worker_id",
                data_type: "uuid",
                nullable: true,
            },
            FieldSchema {
                name: "priority",
                data_type: "integer",
                nullable: true,
            },
            FieldSchema {
                name: "deadline",
                data_type: "timestamp with time zone",
                nullable: true,
            },
            FieldSchema {
                name: "metadata",
                data_type: "jsonb",
                nullable: true,
            },
            FieldSchema {
                name: "created_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "updated_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "completed_at",
                data_type: "timestamp with time zone",
                nullable: true,
            },
        ],
    )
    .await?;

    // Validate task_executions table
    all_valid &= validate_table(
        pool,
        "task_executions",
        &[
            FieldSchema {
                name: "id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "task_id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "worker_id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "execution_started_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "execution_completed_at",
                data_type: "timestamp with time zone",
                nullable: true,
            },
            FieldSchema {
                name: "execution_time_ms",
                data_type: "integer",
                nullable: true,
            },
            FieldSchema {
                name: "status",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "worker_output",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "self_assessment",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "metadata",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "error_message",
                data_type: "text",
                nullable: true,
            },
            FieldSchema {
                name: "tokens_used",
                data_type: "integer",
                nullable: true,
            },
            FieldSchema {
                name: "execution_metadata",
                data_type: "jsonb",
                nullable: true,
            },
            FieldSchema {
                name: "result_data",
                data_type: "jsonb",
                nullable: true,
            },
            FieldSchema {
                name: "created_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "updated_at",
                data_type: "timestamp with time zone",
                nullable: true,
            },
        ],
    )
    .await?;

    // Validate council_verdicts table
    all_valid &= validate_table(
        pool,
        "council_verdicts",
        &[
            FieldSchema {
                name: "id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "task_id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "verdict_id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "consensus_score",
                data_type: "real",
                nullable: false,
            },
            FieldSchema {
                name: "final_verdict",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "individual_verdicts",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "debate_rounds",
                data_type: "integer",
                nullable: false,
            },
            FieldSchema {
                name: "evaluation_time_ms",
                data_type: "integer",
                nullable: false,
            },
            FieldSchema {
                name: "contract",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "verdict_details",
                data_type: "jsonb",
                nullable: true,
            },
            FieldSchema {
                name: "created_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "updated_at",
                data_type: "timestamp with time zone",
                nullable: true,
            },
        ],
    )
    .await?;

    // Validate judge_evaluations table
    all_valid &= validate_table(
        pool,
        "judge_evaluations",
        &[
            FieldSchema {
                name: "id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "verdict_id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "judge_id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "judge_verdict",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "evaluation_time_ms",
                data_type: "integer",
                nullable: false,
            },
            FieldSchema {
                name: "tokens_used",
                data_type: "integer",
                nullable: true,
            },
            FieldSchema {
                name: "confidence",
                data_type: "real",
                nullable: true,
            },
            FieldSchema {
                name: "evaluation_score",
                data_type: "real",
                nullable: true,
            },
            FieldSchema {
                name: "confidence_score",
                data_type: "real",
                nullable: true,
            },
            FieldSchema {
                name: "reasoning",
                data_type: "text",
                nullable: true,
            },
            FieldSchema {
                name: "evidence_used",
                data_type: "jsonb",
                nullable: true,
            },
            FieldSchema {
                name: "evaluation_metadata",
                data_type: "jsonb",
                nullable: true,
            },
            FieldSchema {
                name: "verdict_decision",
                data_type: "character varying",
                nullable: true,
            },
            FieldSchema {
                name: "risk_assessment",
                data_type: "jsonb",
                nullable: true,
            },
            FieldSchema {
                name: "created_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "updated_at",
                data_type: "timestamp with time zone",
                nullable: true,
            },
        ],
    )
    .await?;

    // Validate debate_sessions table
    all_valid &= validate_table(
        pool,
        "debate_sessions",
        &[
            FieldSchema {
                name: "id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "session_id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "task_id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "conflicting_judges",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "rounds",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "status",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "final_consensus",
                data_type: "jsonb",
                nullable: true,
            },
            FieldSchema {
                name: "created_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "resolved_at",
                data_type: "timestamp with time zone",
                nullable: true,
            },
        ],
    )
    .await?;

    if all_valid {
        info!("All migration 014 tables validated successfully");
    } else {
        error!("Some migration 014 tables failed validation");
    }

    Ok(all_valid)
}

/// Validate all tables from migration 015
pub async fn validate_migration_015_tables(pool: &PgPool) -> Result<bool> {
    info!("Validating migration 015 tables (observation & API)");

    let mut all_valid = true;

    // Validate saved_queries table
    all_valid &= validate_table(
        pool,
        "saved_queries",
        &[
            FieldSchema {
                name: "id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "name",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "description",
                data_type: "text",
                nullable: true,
            },
            FieldSchema {
                name: "query_sql",
                data_type: "text",
                nullable: false,
            },
            FieldSchema {
                name: "parameters",
                data_type: "text",
                nullable: true,
            },
            FieldSchema {
                name: "created_by",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "is_public",
                data_type: "boolean",
                nullable: false,
            },
            FieldSchema {
                name: "created_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "updated_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
        ],
    )
    .await?;

    // Validate provenance_entries table
    all_valid &= validate_table(
        pool,
        "provenance_entries",
        &[
            FieldSchema {
                name: "id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "task_id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "action",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "actor",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "resource_id",
                data_type: "uuid",
                nullable: true,
            },
            FieldSchema {
                name: "resource_type",
                data_type: "character varying",
                nullable: true,
            },
            FieldSchema {
                name: "change_summary",
                data_type: "text",
                nullable: false,
            },
            FieldSchema {
                name: "timestamp",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "created_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
            FieldSchema {
                name: "metadata",
                data_type: "jsonb",
                nullable: false,
            },
        ],
    )
    .await?;

    // Validate audit_trail_entries table
    all_valid &= validate_table(
        pool,
        "audit_trail_entries",
        &[
            FieldSchema {
                name: "id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "entity_type",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "entity_id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "action",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "details",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "user_id",
                data_type: "character varying",
                nullable: true,
            },
            FieldSchema {
                name: "ip_address",
                data_type: "character varying",
                nullable: true,
            },
            FieldSchema {
                name: "created_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
        ],
    )
    .await?;

    // Validate audit_logs table
    all_valid &= validate_table(
        pool,
        "audit_logs",
        &[
            FieldSchema {
                name: "id",
                data_type: "uuid",
                nullable: false,
            },
            FieldSchema {
                name: "event_type",
                data_type: "character varying",
                nullable: false,
            },
            FieldSchema {
                name: "event_data",
                data_type: "jsonb",
                nullable: false,
            },
            FieldSchema {
                name: "created_at",
                data_type: "timestamp with time zone",
                nullable: false,
            },
        ],
    )
    .await?;

    if all_valid {
        info!("All migration 015 tables validated successfully");
    } else {
        error!("Some migration 015 tables failed validation");
    }

    Ok(all_valid)
}

/// Validate a single table exists and has correct schema
async fn validate_table(
    pool: &PgPool,
    table_name: &str,
    expected_fields: &[FieldSchema],
) -> Result<bool> {
    info!("Validating table: {}", table_name);

    // Check if table exists
    let table_exists = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = $1
        )
        "#,
    )
    .bind(table_name)
    .fetch_one(pool)
    .await?
    .get::<bool, _>(0);

    if !table_exists {
        error!("Table {} does not exist", table_name);
        return Ok(false);
    }

    info!("Table {} exists", table_name);

    // Get actual table schema
    let actual_fields = sqlx::query(
        r#"
        SELECT 
            column_name,
            data_type,
            is_nullable
        FROM information_schema.columns
        WHERE table_schema = 'public'
        AND table_name = $1
        ORDER BY ordinal_position
        "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await?;

    let mut field_map: HashMap<String, (String, bool)> = HashMap::new();
    for row in actual_fields {
        let name: String = row.get(0);
        let data_type: String = row.get(1);
        let is_nullable: String = row.get(2);
        let nullable = is_nullable == "YES";
        field_map.insert(name.clone(), (data_type, nullable));
    }

    // Validate each expected field
    let mut all_valid = true;
    for expected in expected_fields {
        match field_map.get(expected.name) {
            Some((actual_type, actual_nullable)) => {
                // Normalize data type names (PostgreSQL can return different names)
                let normalized_expected = normalize_data_type(expected.data_type);
                let normalized_actual = normalize_data_type(actual_type);

                if normalized_expected != normalized_actual {
                    error!(
                        "Table {} field {}: expected type {}, got {}",
                        table_name, expected.name, normalized_expected, normalized_actual
                    );
                    all_valid = false;
                }

                if expected.nullable != *actual_nullable {
                    error!(
                        "Table {} field {}: expected nullable={}, got nullable={}",
                        table_name, expected.name, expected.nullable, actual_nullable
                    );
                    all_valid = false;
                }
            }
            None => {
                error!(
                    "Table {} missing expected field: {}",
                    table_name, expected.name
                );
                all_valid = false;
            }
        }
    }

    // Check for unexpected fields (warn but don't fail)
    for (field_name, _) in &field_map {
        if !expected_fields.iter().any(|f| f.name == field_name.as_str()) {
            warn!(
                "Table {} has unexpected field: {} (not in model definition)",
                table_name, field_name
            );
        }
    }

    if all_valid {
        info!("Table {} schema validation passed", table_name);
    }

    Ok(all_valid)
}

/// Normalize PostgreSQL data type names for comparison
fn normalize_data_type(data_type: &str) -> String {
    match data_type {
        "character varying" | "varchar" => "character varying".to_string(),
        "timestamp with time zone" | "timestamptz" => "timestamp with time zone".to_string(),
        "double precision" => "real".to_string(),
        dt => dt.to_string(),
    }
}

/// Validate foreign key relationships
pub async fn validate_foreign_keys(pool: &PgPool) -> Result<bool> {
    info!("Validating foreign key relationships");

    let expected_fks = vec![
        ("tasks", "assigned_worker_id", "workers", "id"),
        ("task_executions", "task_id", "tasks", "id"),
        ("task_executions", "worker_id", "workers", "id"),
        ("council_verdicts", "task_id", "tasks", "id"),
        ("judge_evaluations", "judge_id", "judges", "id"),
        ("judge_evaluations", "verdict_id", "council_verdicts", "verdict_id"),
        ("debate_sessions", "task_id", "tasks", "id"),
        ("provenance_entries", "task_id", "tasks", "id"),
    ];

    let mut all_valid = true;

    for (table, column, ref_table, ref_column) in expected_fks {
        let fk_exists = sqlx::query(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                JOIN information_schema.constraint_column_usage ccu
                    ON ccu.constraint_name = tc.constraint_name
                    AND ccu.table_schema = tc.table_schema
                WHERE tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_name = $1
                AND kcu.column_name = $2
                AND ccu.table_name = $3
                AND ccu.column_name = $4
            )
            "#,
        )
        .bind(table)
        .bind(column)
        .bind(ref_table)
        .bind(ref_column)
        .fetch_one(pool)
        .await?
        .get::<bool, _>(0);

        if !fk_exists {
            error!(
                "Missing foreign key: {}.{} -> {}.{}",
                table, column, ref_table, ref_column
            );
            all_valid = false;
        } else {
            info!(
                "Foreign key validated: {}.{} -> {}.{}",
                table, column, ref_table, ref_column
            );
        }
    }

    if all_valid {
        info!("All foreign key relationships validated");
    }

    Ok(all_valid)
}

/// Main validation function
pub async fn validate_all_schemas(pool: &PgPool) -> Result<bool> {
    info!("Starting comprehensive schema validation");

    let mut all_valid = true;

    all_valid &= validate_migration_014_tables(pool).await?;
    all_valid &= validate_migration_015_tables(pool).await?;
    all_valid &= validate_foreign_keys(pool).await?;

    if all_valid {
        info!("All schema validations passed");
    } else {
        error!("Some schema validations failed");
    }

    Ok(all_valid)
}


