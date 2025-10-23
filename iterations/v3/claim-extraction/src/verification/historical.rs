//! Historical claims lookup and aggregation
//!
//! This module handles database and simulated historical lookups with fallback.

use agent_agency_database::DatabaseClient;
use uuid::Uuid;
use crate::{HistoricalClaim, VerificationStatus, ValidationOutcome};
use tracing::warn;

/// Historical claims lookup
pub struct HistoricalLookup {
    db_client: Option<std::sync::Arc<DatabaseClient>>,
}

impl HistoricalLookup {
    pub fn new(db_client: Option<std::sync::Arc<DatabaseClient>>) -> Self {
        Self { db_client }
    }

    /// Lookup historical claims
    pub async fn lookup_historical_claims(&self, search_term: &str) -> Vec<HistoricalClaim> {
        // Try database first, fall back to simulation
        if let Some(ref client) = self.db_client {
            self.query_database_for_historical_claims(client, search_term).await
                .unwrap_or_else(|_| vec![])
        } else {
            self.get_cached_historical_claims(search_term).await
                .unwrap_or_else(|_| vec![])
        }
    }

    /// Record claim access for usage tracking
    pub async fn record_claim_access(&self, claim_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref client) = self.db_client {
            self.record_claim_access_db(client, claim_id).await
        } else {
            // No-op for cached/simulated mode
            Ok(())
        }
    }

    /// Query database for historical claims
    async fn query_database_for_historical_claims(&self, db_client: &DatabaseClient, search_term: &str) -> Result<Vec<HistoricalClaim>, Box<dyn std::error::Error + Send + Sync>> {
        // Use the find_similar_claims function we created in the migration
        let result = db_client.query(
            r#"
            SELECT * FROM find_similar_claims($1, 0.6, 5, 0.5)
            "#,
            &[&search_term],
        ).await;

        match result {
            Ok(rows) => {
                let mut claims = Vec::new();
                for row in rows {
                    // Parse database row into HistoricalClaim
                    let claim = HistoricalClaim {
                        id: row.try_get::<&str, _>("id").unwrap_or("").to_string(),
                        claim_text: row.try_get::<&str, _>("claim_text").unwrap_or("Unknown claim").to_string(),
                        verification_status: row.try_get::<&str, _>("verification_status")
                            .map(|s| match s {
                                "Verified" => VerificationStatus::Verified,
                                "Unverified" => VerificationStatus::Unverified,
                                _ => VerificationStatus::Unverified,
                            })
                            .unwrap_or(VerificationStatus::Unverified),
                        evidence: vec![],
                        confidence_score: row.try_get::<_, f64>("confidence_score").unwrap_or(0.5),
                        timestamp: chrono::Utc::now(),
                        source_count: row.get("source_count"),
                        last_verified: row.try_get::<&str, _>("last_verified_at").ok().and_then(|ts| chrono::DateTime::parse_from_rfc3339(ts).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
                        related_entities: row.get("related_entities"),
                        claim_type: row.get("claim_type"),
                        created_at: row.try_get::<&str, _>("created_at").ok().and_then(|ts| chrono::DateTime::parse_from_rfc3339(ts).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
                        updated_at: None, // Not returned by function
                        metadata: None, // Not returned by function
                        source_references: row.get("source_references"),
                        cross_references: row.get("cross_references"),
                        validation_metadata: None,
                        validation_confidence: row.try_get::<_, f64>("confidence_score").unwrap_or(0.5),
                        validation_timestamp: chrono::Utc::now(),
                        validation_outcome: ValidationOutcome::Validated,
                    };
                    claims.push(claim);
                }
                Ok(claims)
            }
            Err(e) => {
                warn!("Database query failed: {}", e);
                Ok(vec![])
            }
        }
    }

    /// Get cached historical claims (fallback implementation)
    async fn get_cached_historical_claims(&self, _search_term: &str) -> Result<Vec<HistoricalClaim>, Box<dyn std::error::Error + Send + Sync>> {
        // Simulate some historical claims for testing
        // In a real implementation, this would query a cache or local store
        Ok(vec![
            HistoricalClaim {
                id: "hist-001".to_string(),
                claim_text: "System provides REST API endpoints".to_string(),
                verification_status: VerificationStatus::Verified,
                evidence: vec![],
                confidence_score: 0.9,
                timestamp: chrono::Utc::now(),
                source_count: Some(3),
                last_verified: Some(chrono::Utc::now()),
                related_entities: None,
                claim_type: None,
                created_at: Some(chrono::Utc::now()),
                updated_at: None,
                metadata: None,
                source_references: None,
                cross_references: None,
                validation_metadata: None,
                validation_confidence: 0.9,
                validation_timestamp: chrono::Utc::now(),
                validation_outcome: ValidationOutcome::Validated,
            }
        ])
    }

    /// Record claim access in database
    async fn record_claim_access_db(&self, db_client: &DatabaseClient, claim_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let result = db_client.execute_parameterized_query(
            "SELECT record_claim_access($1)",
            vec![serde_json::Value::from(claim_id.to_string())],
        ).await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Failed to record claim access for {}: {}", claim_id, e);
                Ok(()) // Don't fail the whole operation for access tracking
            }
        }
    }
}
