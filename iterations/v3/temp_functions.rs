    async fn verify_with_council(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!("Submitting claim to council for verification: {}", claim.id);

        // 1. Claim preparation: Format council submission payloads using TaskSpec-compatible schemas
        let task_spec = self.prepare_council_submission(claim, context)?;

        // 2. Submission + retry strategy: Stream requests through the council async client
        let submission_result = self.submit_to_council_with_retry(&task_spec).await?;

        // 3. Verdict ingestion: Parse debate transcripts and consensus metrics from council
        let evidence = self.process_council_verdict(&submission_result, claim)?;

        debug!("Council verification completed for claim: {}", claim.id);
        Ok(evidence)
    }

    /// Prepare council submission payload using TaskSpec-compatible schemas
    fn prepare_council_submission(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<CouncilTaskSpec> {
        let task_id = Uuid::new_v4();
        let timestamp = Utc::now();

        // Determine risk tier based on claim type and scope
        let risk_tier = self.determine_risk_tier(claim);

        // Create acceptance criteria from claim
        let acceptance_criteria = vec![CouncilAcceptanceCriterion {
            id: format!("claim_{}", claim.id),
            description: format!("Verify claim: {}", claim.claim_text),
        }];

        // Build task context
        let task_context = CouncilTaskContext {
            workspace_root: context.source_file.clone().unwrap_or_default(),
            git_branch: "main".to_string(), // TODO: Extract from context
            recent_changes: vec![claim.claim_text.clone()],
            dependencies: std::collections::HashMap::new(),
            environment: CouncilEnvironment::Development,
        };

        // Create worker output from claim
        let worker_output = CouncilWorkerOutput {
            content: claim.claim_text.clone(),
            files_modified: vec![],
            rationale: format!("Claim verification for: {}", claim.claim_text),
            self_assessment: CouncilSelfAssessment {
                caws_compliance: 0.8,
                quality_score: claim.confidence as f32,
                confidence: claim.confidence as f32,
                concerns: vec![],
            },
            metadata: std::collections::HashMap::new(),
        };
