//! Source Integrity Verification Service
//!
//! This module provides comprehensive source integrity verification capabilities including:
//! - Content hash calculation and verification
//! - Tampering detection and alerting
//! - Source integrity record management
//! - Performance monitoring and optimization
//!
//! @author @darianrosebrook

pub mod hasher;
pub mod service;
pub mod storage;
pub mod tampering_detector;
pub mod types;

pub use service::SourceIntegrityService;
pub use types::*;
