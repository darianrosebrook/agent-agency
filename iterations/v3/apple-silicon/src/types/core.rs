//! Core types and identifiers for Apple Silicon functionality

use serde::{Deserialize, Serialize};

/// Device identifier for Apple Silicon devices
pub type DeviceId = String;

/// Compute units available on Apple Silicon hardware
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComputeUnit {
    /// Apple Neural Engine - specialized for ML inference
    ANE,
    /// Metal GPU - general purpose GPU compute
    GPU,
    /// CPU cores - general purpose CPU compute
    CPU,
    /// All available compute units
    All,
}

impl std::fmt::Display for ComputeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputeUnit::ANE => write!(f, "ANE"),
            ComputeUnit::GPU => write!(f, "GPU"),
            ComputeUnit::CPU => write!(f, "CPU"),
            ComputeUnit::All => write!(f, "All"),
        }
    }
}

/// Data layout for tensor storage in memory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataLayout {
    /// NCHW format (batch, channels, height, width)
    NCHW,
    /// NHWC format (batch, height, width, channels)
    NHWC,
    /// CHW format (channels, height, width) - single image
    CHW,
    /// HWC format (height, width, channels) - single image
    HWC,
}

impl std::fmt::Display for DataLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataLayout::NCHW => write!(f, "NCHW"),
            DataLayout::NHWC => write!(f, "NHWC"),
            DataLayout::CHW => write!(f, "CHW"),
            DataLayout::HWC => write!(f, "HWC"),
        }
    }
}

/// Color spaces for image processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSpace {
    /// RGB color space
    RGB,
    /// BGR color space
    BGR,
    /// Grayscale
    Grayscale,
    /// RGBA color space with alpha
    RGBA,
    /// BGRA color space with alpha
    BGRA,
}

impl std::fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorSpace::RGB => write!(f, "RGB"),
            ColorSpace::BGR => write!(f, "BGR"),
            ColorSpace::Grayscale => write!(f, "Grayscale"),
            ColorSpace::RGBA => write!(f, "RGBA"),
            ColorSpace::BGRA => write!(f, "BGRA"),
        }
    }
}
