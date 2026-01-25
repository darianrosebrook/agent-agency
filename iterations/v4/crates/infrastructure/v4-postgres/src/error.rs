//! PostgreSQL Error Types

/// PostgreSQL adapter errors
#[derive(Debug, thiserror::Error)]
pub enum PostgresError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    #[error("Pool error: {0}")]
    PoolError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

impl From<sqlx::Error> for PostgresError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => PostgresError::NotFound("Row not found".to_string()),
            sqlx::Error::PoolTimedOut => PostgresError::PoolError("Pool timed out".to_string()),
            sqlx::Error::PoolClosed => PostgresError::PoolError("Pool closed".to_string()),
            e if e.to_string().contains("duplicate key") => {
                PostgresError::ConstraintViolation(e.to_string())
            }
            e => PostgresError::QueryFailed(e.to_string()),
        }
    }
}

impl From<serde_json::Error> for PostgresError {
    fn from(err: serde_json::Error) -> Self {
        PostgresError::Serialization(err.to_string())
    }
}
