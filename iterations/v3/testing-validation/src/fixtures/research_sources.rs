//! Research sources fixture for testing
//!
//! Provides test research sources and helper functions for creating research test data.

use std::path::PathBuf;
use anyhow::Result;

/// Research source structure
#[derive(Debug, Clone)]
pub struct ResearchSource {
    pub filename: String,
    pub content: String,
}

/// Create research source files in the workspace
pub async fn create_source_files(workspace_path: &PathBuf) -> Result<()> {
    use tokio::fs;
    
    let sources = get_research_sources();
    
    for source in sources {
        let file_path = workspace_path.join(&source.filename);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&file_path, source.content).await?;
    }
    
    Ok(())
}

/// Get research sources for testing
pub fn get_research_sources() -> Vec<ResearchSource> {
    vec![
        ResearchSource {
            filename: "homomorphic_encryption.md".to_string(),
            content: r#"# Homomorphic Encryption Applications

## Overview
Homomorphic encryption allows computation on encrypted data without decryption.

## Healthcare Applications
- Patient data privacy in cloud computing
- Secure medical record analysis
- Privacy-preserving genomic research

## Finance Applications
- Secure financial calculations
- Privacy-preserving credit scoring
- Encrypted transaction processing

## Technical Foundations
- Fully homomorphic encryption (FHE) schemes
- Partially homomorphic encryption (PHE)
- Performance challenges and optimizations

## Current Challenges
- Computational overhead
- Key management complexity
- Standardization efforts
"#.to_string(),
        },
        ResearchSource {
            filename: "cloud_computing.md".to_string(),
            content: r#"# Cloud Computing Security

## Encryption in Cloud
Cloud providers use encryption at rest and in transit.

## Homomorphic Encryption Benefits
- Enables computation on encrypted cloud data
- Maintains data privacy
- Supports regulatory compliance

## Implementation Considerations
- Performance impact
- Cost implications
- Integration complexity
"#.to_string(),
        },
        ResearchSource {
            filename: "privacy_technologies.md".to_string(),
            content: r#"# Privacy-Preserving Technologies

## Overview
Various technologies enable privacy-preserving computation.

## Homomorphic Encryption
- Allows computation on encrypted data
- Strong privacy guarantees
- Performance trade-offs

## Other Technologies
- Secure multi-party computation
- Differential privacy
- Zero-knowledge proofs

## Use Cases
- Healthcare data analysis
- Financial services
- Machine learning on sensitive data
"#.to_string(),
        },
    ]
}
