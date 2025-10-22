//! Metal Performance Shaders Graph compatibility layer
//!
//! This module provides a stable API for Metal Performance Shaders Graph operations,
//! with proper feature gating to ensure compilation on all platforms.

#[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "mpsgraph"))]
mod apple_silicon_impl {
    use crate::compat::numerics;
    use anyhow::Result;

    // Re-export the real Metal types
    pub use metal::MTLSize;
    pub use objc2_metal_performance_shaders_graph::MPSGraph;

    /// Apple Silicon implementation with real MPSGraph bindings
    pub struct Graph {
        raw: objc2_metal_performance_shaders_graph::MPSGraph,
    }

    impl Graph {
        pub fn new() -> Self {
            // SAFETY: MPSGraph::new() is unsafe due to ObjC allocation rules
            let raw = unsafe { objc2_metal_performance_shaders_graph::MPSGraph::new() };
            Self { raw }
        }

        pub fn softmax(&self, _input: &Tensor, _name: &str) -> Tensor {
            // TODO: Implement real softmax operation
            todo!("Implement real MPSGraph softmax")
        }

        pub fn gelu(&self, _input: &Tensor, _name: &str) -> Tensor {
            // TODO: Implement real GELU operation
            todo!("Implement real MPSGraph GELU")
        }

        pub fn add(&self, _a: &Tensor, _b: &Tensor, _name: &str) -> Tensor {
            // TODO: Implement real addition operation
            todo!("Implement real MPSGraph addition")
        }

        pub fn encode(
            &self,
            _cmd: &metal::CommandBufferRef,
            _feeds: &[(&Tensor, &crate::compat::numerics::NDArray)],
            _fetches: &[&Tensor],
            _label: &str,
        ) -> Result<()> {
            // TODO: Implement real encoding
            todo!("Implement real MPSGraph encoding")
        }
    }

    // Placeholder tensor type - replace with real MPSGraphTensor
    pub struct Tensor;

    impl Tensor {
        pub fn new() -> Self {
            Self
        }
    }
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64", feature = "mpsgraph")))]
mod stub_impl {
    use anyhow::{anyhow, bail, Result};

    /// Stub Metal MTLSize for non-Apple Silicon platforms
    #[derive(Debug, Clone, Copy)]
    pub struct MTLSize {
        pub width: u64,
        pub height: u64,
        pub depth: u64,
    }

    /// Stub MPSGraph for non-Apple Silicon platforms
    pub struct MPSGraph;

    /// Stub Graph implementation that fails gracefully
    pub struct Graph;

    impl Graph {
        pub fn new() -> Self {
            Graph
        }

        pub fn softmax(&self, _input: &Tensor, _name: &str) -> Tensor {
            panic!("MPSGraph softmax unavailable: requires Apple Silicon macOS + `mpsgraph` feature")
        }

        pub fn gelu(&self, _input: &Tensor, _name: &str) -> Tensor {
            panic!("MPSGraph GELU unavailable: requires Apple Silicon macOS + `mpsgraph` feature")
        }

        pub fn add(&self, _a: &Tensor, _b: &Tensor, _name: &str) -> Tensor {
            panic!("MPSGraph addition unavailable: requires Apple Silicon macOS + `mpsgraph` feature")
        }

        pub fn encode(
            &self,
            _cmd: &(),
            _feeds: &[(&Tensor, &())],
            _fetches: &[&Tensor],
            _label: &str,
        ) -> Result<()> {
            bail!("MPSGraph encoding unavailable: requires Apple Silicon macOS + `mpsgraph` feature. Enable with `--features mpsgraph`")
        }
    }

    /// Stub tensor type
    pub struct Tensor;
}

#[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "mpsgraph"))]
pub use apple_silicon_impl::*;

#[cfg(not(all(target_os = "macos", target_arch = "aarch64", feature = "mpsgraph")))]
pub use stub_impl::*;

// Common types that work on all platforms
/// Threadgroup size for Metal compute shaders
#[derive(Debug, Clone, Copy)]
pub struct MTLSize {
    pub width: u64,
    pub height: u64,
    pub depth: u64,
}

impl Default for MTLSize {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            depth: 1,
        }
    }
}

/// Compute encoder dispatch helpers
pub mod dispatch {
    use super::*;

    pub fn dispatch_threadgroups(
        _encoder: &(),
        _tg: MTLSize,
        _tpt: MTLSize,
    ) {
        // Stub implementation
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64", feature = "mpsgraph")))]
        panic!("Metal compute dispatch unavailable: requires Apple Silicon macOS + `mpsgraph` feature");
    }
}

/// Placeholder NDArray type for Metal operations
#[cfg(not(all(target_os = "macos", target_arch = "aarch64", feature = "mpsgraph")))]
pub type NDArray = ();

#[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "mpsgraph"))]
pub type NDArray = ndarray::ArrayD<f32>;