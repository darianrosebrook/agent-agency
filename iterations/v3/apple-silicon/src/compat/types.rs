//! Type compatibility layer for serialization and external API mismatches

use crate::inference::DType;

/// Serde-friendly mirror of DType for stable serialization
#[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SerdeDType {
    U8,
    I8,
    I32,
    F16,
    F32,
    F64,
    Bool,
}

impl From<DType> for SerdeDType {
    fn from(dtype: DType) -> Self {
        match dtype {
            DType::U8 => SerdeDType::U8,
            DType::F16 => SerdeDType::F16,
            DType::F32 => SerdeDType::F32,
            DType::F64 => SerdeDType::F64,
            // Map unsupported variants to closest equivalents
            _ => SerdeDType::F32,
        }
    }
}

impl TryFrom<SerdeDType> for DType {
    type Error = anyhow::Error;

    fn try_from(dtype: SerdeDType) -> Result<Self, Self::Error> {
        match dtype {
            SerdeDType::U8 => Ok(DType::U8),
            SerdeDType::F16 => Ok(DType::F16),
            SerdeDType::F32 => Ok(DType::F32),
            SerdeDType::F64 => Ok(DType::F64),
            // Error on unsupported variants
            SerdeDType::I8 | SerdeDType::I32 | SerdeDType::Bool => {
                Err(anyhow::anyhow!("DType variant {:?} not supported by current backend", dtype))
            }
        }
    }
}

use crate::inference::ModelFmt;

/// Serde-friendly mirror of ModelFmt
#[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SerdeModelFmt {
    Onnx,
    Safetensors,
    #[serde(other)]
    Unknown,
}

impl From<ModelFmt> for SerdeModelFmt {
    fn from(fmt: ModelFmt) -> Self {
        match fmt {
            ModelFmt::Onnx => SerdeModelFmt::Onnx,
            ModelFmt::SafeTensors => SerdeModelFmt::Safetensors,
            _ => SerdeModelFmt::Unknown,
        }
    }
}

impl TryFrom<SerdeModelFmt> for ModelFmt {
    type Error = anyhow::Error;

    fn try_from(fmt: SerdeModelFmt) -> Result<Self, Self::Error> {
        match fmt {
            SerdeModelFmt::Onnx => Ok(ModelFmt::Onnx),
            SerdeModelFmt::Safetensors => Ok(ModelFmt::SafeTensors),
            SerdeModelFmt::Unknown => Err(anyhow::anyhow!("Unknown model format")),
        }
    }
}

// Display implementations for types that need them
use crate::types::{ComputeUnit, QuantizationMethod};

impl std::fmt::Display for ComputeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputeUnit::CPU => write!(f, "CPU"),
            ComputeUnit::GPU => write!(f, "GPU"),
            ComputeUnit::ANE => write!(f, "ANE"),
            ComputeUnit::All => write!(f, "ALL"),
        }
    }
}

impl std::fmt::Display for QuantizationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantizationMethod::None => write!(f, "none"),
            QuantizationMethod::Int8 => write!(f, "int8"),
            QuantizationMethod::Int4 => write!(f, "int4"),
            QuantizationMethod::Float16 => write!(f, "float16"),
        }
    }
}
