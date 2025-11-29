//! Claim extraction functionality
//!
//! Implements the four-stage claim processing pipeline from arbiter theory:
//! 1. Contextual Disambiguation (hard gate)
//! 2. Verifiable Content Qualification (pass/fail gate)
//! 3. Atomic Claim Decomposition
//! 4. CAWS-Compliant Verification

pub mod claim_extractor;
pub mod contextual_disambiguator;
pub mod verifiable_content_qualifier;
pub mod atomic_claim_decomposer;
pub mod caws_verifier;
pub mod council_claim_integration;

pub use claim_extractor::ClaimExtractor;
pub use contextual_disambiguator::{
    AmbiguityInstance, ContextualDisambiguator, ConversationContext, FallbackStrategy,
    ResolutionAttempt, ResolutionFailureReason, ResolutionResult,
};
pub use verifiable_content_qualifier::{
    VerifiableContentQualifier, VerifiableContentResult,
};
pub use atomic_claim_decomposer::{
    AtomicClaimDecomposer, AtomicDecompositionResult, SplitClaimInfo, SplitReason,
};
pub use caws_verifier::{
    CawsCompliantVerifier, CawsVerificationConfig, CawsVerificationResult,
    FailedClaimInfo, QualityGateResults, ScopeViolation, ScopeViolationType,
    VerificationFailureReason, VerifiedClaimInfo,
};
pub use council_claim_integration::{
    ClaimExtractionFailureReason, CouncilClaimIntegrator, CouncilClaimIntegrationResult,
    CouncilDecisionWithClaims, FailedCouncilClaim, VerifiedCouncilClaim,
};
