//! Metal / MPS backend for accelerated inference on Apple Silicon GPUs.
//!
//! This module provides a thin, production-capable wrapper around Candle's
//! Metal backend, enabling MPS execution without CoreML. It is intended as
//! the active acceleration path while the CoreML/ANE toolchain is offline.
//! @author @darianrosebrook

use anyhow::{ensure, Context, Result};
use candle_core::{Device, Tensor};

/// Metal executor backed by Candle's Metal device.
pub struct MetalExecutor {
    device: Device,
}

impl MetalExecutor {
    /// Create a new Metal executor targeting the given device index.
    pub fn new(device_index: usize) -> Result<Self> {
        #[cfg(not(target_os = "macos"))]
        {
            anyhow::bail!("Metal backend requires macOS");
        }

        #[cfg(target_os = "macos")]
        {
            let device = Device::new_metal(device_index)
                .context("failed to create Metal device (MPS backend)")?;
            Ok(Self { device })
        }
    }

    /// Lightweight capability probe to determine if Metal/MPS is usable.
    pub fn is_available() -> bool {
        #[cfg(target_os = "macos")]
        {
            Device::new_metal(0).is_ok()
        }

        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// Access the underlying Candle device.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Perform a matrix multiplication on Metal and return results on host.
    pub fn matmul_to_host(
        &self,
        lhs: &[f32],
        lhs_shape: (usize, usize),
        rhs: &[f32],
        rhs_shape: (usize, usize),
    ) -> Result<Vec<Vec<f32>>> {
        ensure!(
            lhs_shape.1 == rhs_shape.0,
            "incompatible shapes for matmul: {:?} x {:?}",
            lhs_shape,
            rhs_shape
        );

        let lhs = Tensor::from_slice(lhs, (lhs_shape.0, lhs_shape.1), &self.device)
            .context("failed to place lhs on Metal")?;
        let rhs = Tensor::from_slice(rhs, (rhs_shape.0, rhs_shape.1), &self.device)
            .context("failed to place rhs on Metal")?;

        let out = lhs.matmul(&rhs).context("Metal matmul failed")?;
        out.to_vec2::<f32>()
            .context("failed to move Metal result to host")
    }

    /// Run a small warmup matmul to prime Metal pipelines.
    pub fn warmup(&self) -> Result<()> {
        // 2x2 identity matmul
        let lhs = [1.0f32, 0.0, 0.0, 1.0];
        let rhs = [1.0f32, 0.0, 0.0, 1.0];
        let _ = self.matmul_to_host(&lhs, (2, 2), &rhs, (2, 2))?;
        Ok(())
    }
}



