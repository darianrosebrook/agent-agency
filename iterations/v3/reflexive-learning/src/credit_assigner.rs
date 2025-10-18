//! Credit assignment for learning

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditRecord {
    pub participant_id: Uuid,
    pub session_id: Uuid,
    pub action_sequence: Vec<ActionContribution>,
    pub total_credit: f64,
    pub credit_distribution: HashMap<String, f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub validated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContribution {
    pub action_id: String,
    pub contribution_score: f64,
    pub temporal_weight: f64,
    pub quality_multiplier: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditPolicy {
    pub temporal_decay_factor: f64,    // How much credit decays over time
    pub quality_weight: f64,          // Weight given to action quality
    pub recency_bonus: f64,           // Bonus for recent contributions
    pub collaboration_multiplier: f64, // Multiplier for collaborative actions
    pub max_credit_age_days: i64,     // Maximum age of credit records
}

impl Default for CreditPolicy {
    fn default() -> Self {
        Self {
            temporal_decay_factor: 0.95,
            quality_weight: 1.5,
            recency_bonus: 1.2,
            collaboration_multiplier: 1.3,
            max_credit_age_days: 90,
        }
    }
}

#[derive(Debug)]
pub struct CreditAssigner {
    credit_records: Arc<RwLock<HashMap<Uuid, Vec<CreditRecord>>>>,
    participant_balances: Arc<RwLock<HashMap<Uuid, f64>>>,
    policy: CreditPolicy,
    credit_history: Arc<RwLock<Vec<CreditTransaction>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransaction {
    pub transaction_id: Uuid,
    pub from_participant: Option<Uuid>,
    pub to_participant: Uuid,
    pub amount: f64,
    pub transaction_type: TransactionType,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Assignment,
    Transfer,
    Decay,
    Redemption,
    Adjustment,
}

impl CreditAssigner {
    pub fn new() -> Self {
        Self {
            credit_records: Arc::new(RwLock::new(HashMap::new())),
            participant_balances: Arc::new(RwLock::new(HashMap::new())),
            policy: CreditPolicy::default(),
            credit_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_policy(mut self, policy: CreditPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Assign credit for a sequence of actions leading to an outcome
    pub async fn assign_credit(
        &self,
        session_id: Uuid,
        participant_id: Uuid,
        action_sequence: Vec<ActionContribution>,
        outcome_reward: f64,
    ) -> Result<CreditRecord, Box<dyn std::error::Error + Send + Sync>> {
        // Calculate temporal credit assignment using eligibility traces
        let credit_distribution = self.calculate_temporal_credit(&action_sequence, outcome_reward).await;

        let total_credit = credit_distribution.values().sum();

        let credit_record = CreditRecord {
            participant_id,
            session_id,
            action_sequence: action_sequence.clone(),
            total_credit,
            credit_distribution,
            timestamp: chrono::Utc::now(),
            validated: false,
        };

        // Store the credit record
        let mut records = self.credit_records.write().await;
        records.entry(session_id).or_insert_with(Vec::new).push(credit_record.clone());

        // Update participant balance
        let mut balances = self.participant_balances.write().await;
        let current_balance = balances.get(&participant_id).copied().unwrap_or(0.0);
        balances.insert(participant_id, current_balance + total_credit);

        // Record transaction
        let transaction = CreditTransaction {
            transaction_id: Uuid::new_v4(),
            from_participant: None,
            to_participant: participant_id,
            amount: total_credit,
            transaction_type: TransactionType::Assignment,
            description: format!("Credit assigned for session {}", session_id),
            timestamp: chrono::Utc::now(),
        };

        let mut history = self.credit_history.write().await;
        history.push(transaction);

        Ok(credit_record)
    }

    /// Calculate temporal credit distribution using eligibility traces
    async fn calculate_temporal_credit(
        &self,
        action_sequence: &[ActionContribution],
        outcome_reward: f64,
    ) -> HashMap<String, f64> {
        let mut credit_distribution = HashMap::new();
        let mut eligibility_trace = 0.0;

        // Process actions in reverse chronological order for proper credit assignment
        for (i, action) in action_sequence.iter().enumerate().rev() {
            // Calculate recency factor (more recent actions get higher weight)
            let recency_factor = (i as f64 + 1.0) / action_sequence.len() as f64;

            // Calculate temporal decay
            eligibility_trace = self.policy.temporal_decay_factor * eligibility_trace + action.temporal_weight;

            // Calculate final credit for this action
            let base_credit = outcome_reward * eligibility_trace * recency_factor;
            let quality_adjusted_credit = base_credit * (1.0 + self.policy.quality_weight * action.quality_multiplier);

            credit_distribution.insert(action.action_id.clone(), quality_adjusted_credit);
        }

        credit_distribution
    }

    /// Redistribute credit for collaborative actions
    pub async fn redistribute_collaborative_credit(
        &self,
        session_id: Uuid,
        participant_contributions: HashMap<Uuid, f64>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let total_contribution: f64 = participant_contributions.values().sum();
        if total_contribution == 0.0 {
            return Ok(());
        }

        let collaboration_bonus = total_contribution * (self.policy.collaboration_multiplier - 1.0);
        let bonus_per_participant = collaboration_bonus / participant_contributions.len() as f64;

        let mut balances = self.participant_balances.write().await;
        let mut history = self.credit_history.write().await;

        for (participant_id, contribution) in &participant_contributions {
            let contribution_ratio = contribution / total_contribution;
            let participant_bonus = bonus_per_participant * contribution_ratio;

            let current_balance = balances.get(participant_id).copied().unwrap_or(0.0);
            balances.insert(*participant_id, current_balance + participant_bonus);

            let transaction = CreditTransaction {
                transaction_id: Uuid::new_v4(),
                from_participant: None,
                to_participant: *participant_id,
                amount: participant_bonus,
                transaction_type: TransactionType::Assignment,
                description: format!("Collaborative credit bonus for session {}", session_id),
                timestamp: chrono::Utc::now(),
            };

            history.push(transaction);
        }

        Ok(())
    }

    /// Get participant credit balance
    pub async fn get_balance(&self, participant_id: &Uuid) -> f64 {
        let balances = self.participant_balances.read().await;
        balances.get(participant_id).copied().unwrap_or(0.0)
    }

    /// Get credit history for a participant
    pub async fn get_credit_history(&self, participant_id: &Uuid, limit: usize) -> Vec<CreditTransaction> {
        let history = self.credit_history.read().await;
        history
            .iter()
            .filter(|t| t.to_participant == *participant_id || t.from_participant == Some(*participant_id))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Transfer credit between participants
    pub async fn transfer_credit(
        &self,
        from_participant: Uuid,
        to_participant: Uuid,
        amount: f64,
        description: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut balances = self.participant_balances.write().await;

        let from_balance = balances.get(&from_participant).copied().unwrap_or(0.0);
        if from_balance < amount {
            return Err("Insufficient credit balance".into());
        }

        balances.insert(from_participant, from_balance - amount);

        let to_balance = balances.get(&to_participant).copied().unwrap_or(0.0);
        balances.insert(to_participant, to_balance + amount);

        // Record transaction
        let transaction = CreditTransaction {
            transaction_id: Uuid::new_v4(),
            from_participant: Some(from_participant),
            to_participant,
            amount,
            transaction_type: TransactionType::Transfer,
            description,
            timestamp: chrono::Utc::now(),
        };

        let mut history = self.credit_history.write().await;
        history.push(transaction);

        Ok(())
    }

    /// Apply credit decay based on age
    pub async fn apply_credit_decay(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(self.policy.max_credit_age_days);
        let mut balances = self.participant_balances.write().await;
        let mut history = self.credit_history.write().await;

        let mut decayed_count = 0;

        // Find old transactions and apply decay
        for (participant_id, balance) in balances.iter_mut() {
            if let Some(oldest_transaction) = history
                .iter()
                .filter(|t| t.to_participant == *participant_id)
                .min_by_key(|t| t.timestamp)
            {
                let age_days = (chrono::Utc::now() - oldest_transaction.timestamp).num_days();
                if age_days > self.policy.max_credit_age_days {
                    let decay_factor = (self.policy.temporal_decay_factor as f64).powi(age_days as i32);
                    let decayed_balance = *balance * decay_factor;
                    let decay_amount = *balance - decayed_balance;

                    if decay_amount > 0.0 {
                        *balance = decayed_balance;
                        decayed_count += 1;

                        let transaction = CreditTransaction {
                            transaction_id: Uuid::new_v4(),
                            from_participant: Some(*participant_id),
                            to_participant: Uuid::nil(), // System
                            amount: decay_amount,
                            transaction_type: TransactionType::Decay,
                            description: format!("Credit decay after {} days", age_days),
                            timestamp: chrono::Utc::now(),
                        };

                        history.push(transaction);
                    }
                }
            }
        }

        Ok(decayed_count)
    }

    /// Validate credit assignments for consistency
    pub async fn validate_credit_assignments(&self) -> Result<ValidationReport, Box<dyn std::error::Error + Send + Sync>> {
        let records = self.credit_records.read().await;
        let mut validation_issues = Vec::new();
        let mut validated_records = 0;

        for (session_id, session_records) in records.iter() {
            for record in session_records {
                // Validate credit distribution sums
                let distributed_total: f64 = record.credit_distribution.values().sum();
                if (distributed_total - record.total_credit).abs() > 0.01 {
                    validation_issues.push(format!(
                        "Credit distribution mismatch in session {}: expected {:.2}, got {:.2}",
                        session_id, record.total_credit, distributed_total
                    ));
                }

                // Validate action sequence integrity
                if record.action_sequence.is_empty() {
                    validation_issues.push(format!(
                        "Empty action sequence in session {}", session_id
                    ));
                }

                // Mark as validated
                // TODO: Implement record update with the following requirements:
                // 1. Record update implementation: Update the record in real implementation
                //    - Update the record in real implementation for proper data management
                //    - Handle record update implementation optimization and performance
                //    - Implement record update implementation validation and quality assurance
                //    - Support record update implementation customization and configuration
                // 2. Record management: Manage record lifecycle and operations
                //    - Manage record lifecycle and operational management
                //    - Handle record management optimization and performance
                //    - Implement record management validation and quality assurance
                //    - Support record management customization and configuration
                // 3. Record update optimization: Optimize record update performance and reliability
                //    - Optimize record update performance and reliability for efficiency
                //    - Handle record update optimization and performance
                //    - Implement record update optimization validation and quality assurance
                //    - Support record update optimization customization and configuration
                // 4. Record update system optimization: Optimize record update system performance
                //    - Implement record update system optimization strategies
                //    - Handle record update system monitoring and analytics
                //    - Implement record update system validation and quality assurance
                //    - Ensure record update system meets performance and reliability standards
                validated_records += 1;
            }
        }

        Ok(ValidationReport {
            total_records: records.values().map(|v| v.len()).sum(),
            validated_records,
            issues_found: validation_issues.len(),
            validation_issues,
        })
    }

    /// Get credit leaderboard
    pub async fn get_credit_leaderboard(&self, limit: usize) -> Vec<(Uuid, f64)> {
        let balances = self.participant_balances.read().await;
        let mut leaderboard: Vec<(Uuid, f64)> = balances
            .iter()
            .map(|(id, balance)| (*id, *balance))
            .collect();

        leaderboard.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        leaderboard.truncate(limit);
        leaderboard
    }

    /// Export credit data for analysis
    pub async fn export_credit_data(&self) -> Result<CreditExport, Box<dyn std::error::Error + Send + Sync>> {
        let records = self.credit_records.read().await;
        let balances = self.participant_balances.read().await;
        let history = self.credit_history.read().await;

        Ok(CreditExport {
            participant_balances: balances.clone(),
            credit_records: records.clone(),
            transaction_history: history.clone(),
            export_timestamp: chrono::Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub total_records: usize,
    pub validated_records: usize,
    pub issues_found: usize,
    pub validation_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditExport {
    pub participant_balances: HashMap<Uuid, f64>,
    pub credit_records: HashMap<Uuid, Vec<CreditRecord>>,
    pub transaction_history: Vec<CreditTransaction>,
    pub export_timestamp: chrono::DateTime<chrono::Utc>,
}
