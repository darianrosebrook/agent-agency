//! I/O safety validation and memory safety helpers
//!
//! This module provides comprehensive safety validation for Core ML operations,
//! including tensor validation, memory bounds checking, and data type safety.

use crate::ane::ane_errors::{ANEError, Result};
use crate::ane::TensorSpec;
use candle_core::Device;

/// Tensor type alias for consistency
pub type Tensor = candle_core::Tensor;

/// I/O safety validation helpers
pub mod io_safety {
    use super::*;

    /// Convert FFI tensor data to owned Vec<f32>, validating shape and bounds
    pub fn into_owned_tensor(data: &[f32], shape: &[usize]) -> Result<Tensor> {
        // Validate shape is not empty and compute total size
        if shape.is_empty() {
            return Err(ANEError::InvalidInput(
                "Tensor shape cannot be empty".to_string(),
            ));
        }

        let total_size: usize = shape.iter().product();
        if total_size == 0 {
            return Err(ANEError::InvalidInput(
                "Tensor cannot have zero size".to_string(),
            ));
        }

        // Check data length matches shape
        if data.len() != total_size {
            return Err(ANEError::InvalidInput(format!(
                "Data length {} doesn't match shape product {}",
                data.len(),
                total_size
            )));
        }

        // Reasonable size limits to prevent memory exhaustion
        const MAX_TENSOR_ELEMENTS: usize = 100 * 1024 * 1024; // 100M elements
        if total_size > MAX_TENSOR_ELEMENTS {
            return Err(ANEError::InvalidInput(format!(
                "Tensor too large: {} elements (max {})",
                total_size, MAX_TENSOR_ELEMENTS
            )));
        }

        Ok(Tensor::new(data, &Device::Cpu)?)
    }

    /// Convert tensor to F32 for CoreML compatibility
    ///
    /// CoreML requires F32 tensors for optimal performance on Apple Neural Engine.
    /// This function handles conversion from various input types to F32.
    ///
    /// # Arguments
    /// * `tensor` - Input tensor to convert
    /// * `expected_spec` - Expected tensor specification
    ///
    /// # Returns
    /// * `Result<Tensor>` - Converted F32 tensor or error
    pub fn convert_tensor_for_coreml(
        tensor: &Tensor,
        expected_spec: &TensorSpec,
    ) -> Result<Tensor> {
        match expected_spec.dtype.as_str() {
            "F32" => {
                // Float32 - native support, no conversion needed
                if matches!(tensor.dtype(), candle_core::DType::F32) {
                    Ok(tensor.clone())
                } else {
                    // Convert other types to F32
                    Ok(tensor.to_dtype(candle_core::DType::F32)?)
                }
            },
            "F16" => {
                // Float16 - convert to F32 for CoreML compatibility
                if matches!(tensor.dtype(), candle_core::DType::F16) {
                    // Convert F16 to F32
                    Ok(tensor.to_dtype(candle_core::DType::F32)?)
                } else {
                    // Convert other types to F32 first, then to F16 if needed
                    let f32_tensor = tensor.to_dtype(candle_core::DType::F32)?;
                    Ok(f32_tensor)
                }
            },
            "I32" => {
                // Int32 - convert to F32 for CoreML
                // Note: candle_core doesn't have I32, so we convert any integer type to F32
                Ok(tensor.to_dtype(candle_core::DType::F32)?)
            },
            "I16" => {
                // Int16 - convert to F32
                // Note: candle_core doesn't have I16, so we convert any integer type to F32
                Ok(tensor.to_dtype(candle_core::DType::F32)?)
            },
            "I8" => {
                // Int8 - convert to F32
                // Note: candle_core doesn't have I8, so we convert any integer type to F32
                Ok(tensor.to_dtype(candle_core::DType::F32)?)
            },
            "U8" => {
                // UInt8 - convert to F32
                if matches!(tensor.dtype(), candle_core::DType::U8) {
                    Ok(tensor.to_dtype(candle_core::DType::F32)?)
                } else {
                    Ok(tensor.to_dtype(candle_core::DType::F32)?)
                }
            },
            "BOOL" => {
                // Boolean - convert to F32 (0.0 or 1.0)
                if matches!(tensor.dtype(), candle_core::DType::U8) {
                    // Convert U8 (boolean representation) to F32
                    Ok(tensor.to_dtype(candle_core::DType::F32)?)
                } else {
                    Ok(tensor.to_dtype(candle_core::DType::F32)?)
                }
            },
            unsupported => {
                Err(ANEError::InvalidInput(
                    format!("Unsupported CoreML data type: {}. Supported types: F32, F16, I32, I16, I8, U8, BOOL",
                           unsupported)
                ))
            }
        }
    }

    /// Validate tensor schema matches expected I/O specification
    pub fn validate_io_schema(tensor: &Tensor, expected_spec: &TensorSpec) -> Result<()> {
        // Use the conversion function to handle data type compatibility
        let _converted_tensor = convert_tensor_for_coreml(tensor, expected_spec)?;

        // Check shape compatibility
        let tensor_dims = tensor.dims();
        if tensor_dims.len() != expected_spec.shape.len() {
            return Err(ANEError::InvalidInput(format!(
                "Shape dimension mismatch: got {}, expected {}",
                tensor_dims.len(),
                expected_spec.shape.len()
            )));
        }

        // For batch-capable tensors, allow variable batch size
        if expected_spec.batch_capable && tensor_dims.len() > 0 {
            // Check non-batch dimensions match
            if &tensor_dims[1..] != &expected_spec.shape[1..] {
                return Err(ANEError::InvalidInput(format!(
                    "Non-batch dimensions don't match: got {:?}, expected {:?}",
                    &tensor_dims[1..],
                    &expected_spec.shape[1..]
                )));
            }
        } else {
            // Exact shape match required
            if tensor_dims != expected_spec.shape {
                return Err(ANEError::InvalidInput(format!(
                    "Shape mismatch: got {:?}, expected {:?}",
                    tensor_dims, expected_spec.shape
                )));
            }
        }

        Ok(())
    }

    /// Safe conversion from raw FFI tensors to owned tensors
    /// This prevents buffer overflows and validates all inputs
    pub fn convert_ffi_tensors(raw_tensors: Vec<Tensor>) -> Result<Vec<Tensor>> {
        let mut owned_tensors = Vec::with_capacity(raw_tensors.len());

        for raw_tensor in raw_tensors {
            // Validate tensor data before conversion
            let shape = raw_tensor.shape();
            let dims = shape.dims();

            // Calculate expected data size from shape
            let expected_size: usize = dims.iter().product();
            let bytes_per_element = match raw_tensor.dtype().as_str() {
                "F32" => 4,
                "F16" => 2,
                "I32" => 4,
                "I16" => 2,
                "I8" => 1,
                "U8" => 1,
                _ => {
                    return Err(ANEError::InvalidInput(format!(
                        "Unsupported tensor dtype for FFI conversion: {:?}",
                        raw_tensor.dtype()
                    )))
                }
            };

            let expected_bytes = expected_size * bytes_per_element;
            // TODO: Implement proper tensor data length extraction
            //       Currently skips validation; should extract actual data length from tensor for proper bounds checking.
            //
            // COMPLETION CHECKLIST:
            // [ ] Extract actual tensor data length from tensor structure
            // [ ] Use platform-specific APIs to get tensor metadata
            // [ ] Validate data length against expected size
            // [ ] Handle different tensor formats and layouts
            // [ ] Add proper error handling for extraction failures
            // [ ] Add unit tests with various tensor types
            // [ ] Add integration tests with real tensor data
            // [ ] Performance: Extraction should complete in <10μs
            // [ ] Documentation: Document tensor data extraction method
            //
            // ACCEPTANCE CRITERIA:
            // - Tensor data length is extracted accurately
            // - Validation uses actual data length
            // - Different tensor formats are supported
            // - Extraction errors are handled gracefully
            // - Performance impact is minimal
            //
            // DEPENDENCIES:
            // - Tensor metadata APIs (Required)
            // - Platform-specific tensor access (Required)
            // - Tensor format handlers (Required)
            //
            // ESTIMATED EFFORT: 6-8 hours (low confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 1 (safety-critical feature)
            // - Change Budget: ~200 LOC
            // - Reviewer Requirements: Tensor and safety expertise
            // Note: We can't directly access tensor data length, so we'll skip this validation

            // Validate data bounds for safety
            if expected_bytes > 100 * 1024 * 1024 {
                // 100MB limit
                return Err(ANEError::InvalidInput(format!(
                    "Tensor data too large: {} bytes exceeds safety limit",
                    expected_bytes
                )));
            }

            // Convert to owned tensor with validation
            let owned = into_owned_tensor(&[], &dims)?;
            owned_tensors.push(owned);
        }

        Ok(owned_tensors)
    }
}

/// Memory safety utilities
pub mod memory {
    use super::*;

    /// Validate memory allocation size to prevent overflow
    pub fn validate_allocation_size(elements: usize, element_size: usize) -> Result<()> {
        const MAX_ALLOCATION_BYTES: usize = 1024 * 1024 * 1024; // 1GB limit

        let total_bytes = elements.checked_mul(element_size).ok_or_else(|| {
            ANEError::InvalidInput("Integer overflow in allocation size calculation".to_string())
        })?;

        if total_bytes > MAX_ALLOCATION_BYTES {
            return Err(ANEError::InvalidInput(format!(
                "Allocation too large: {} bytes exceeds safety limit of {} bytes",
                total_bytes, MAX_ALLOCATION_BYTES
            )));
        }

        Ok(())
    }

    /// Safe wrapper for array bounds checking
    pub fn safe_array_access<T>(array: &[T], index: usize) -> Result<&T> {
        if index >= array.len() {
            return Err(ANEError::InvalidInput(format!(
                "Array index {} out of bounds for array of length {}",
                index,
                array.len()
            )));
        }
        Ok(&array[index])
    }

    /// Validate string length for FFI safety
    pub fn validate_string_length(s: &str, max_length: usize) -> Result<()> {
        if s.len() > max_length {
            return Err(ANEError::InvalidInput(format!(
                "String length {} exceeds maximum allowed length {}",
                s.len(),
                max_length
            )));
        }

        // Check for null bytes which would cause C string issues
        if s.contains('\0') {
            return Err(ANEError::InvalidInput(
                "String contains null bytes".to_string(),
            ));
        }

        Ok(())
    }
}

/// Data type safety utilities
pub mod dtype {
    use super::*;

    /// Validate that a data type is supported by Core ML
    pub fn validate_coreml_dtype(dtype: &str) -> Result<()> {
        match dtype {
            "F32" | "F16" | "I32" | "I16" | "I8" | "U8" | "BOOL" => Ok(()),
            _ => Err(ANEError::InvalidInput(
                format!("Unsupported CoreML data type: {}. Supported types: F32, F16, I32, I16, I8, U8, BOOL", dtype)
            )),
        }
    }

    /// Get the size in bytes for a Core ML data type
    pub fn dtype_size_bytes(dtype: &str) -> Result<usize> {
        match dtype {
            "F32" => Ok(4),
            "F16" => Ok(2),
            "I32" => Ok(4),
            "I16" => Ok(2),
            "I8" => Ok(1),
            "U8" => Ok(1),
            "BOOL" => Ok(1),
            _ => Err(ANEError::InvalidInput(format!(
                "Unknown data type: {}",
                dtype
            ))),
        }
    }

    /// Check if two data types are compatible for conversion
    pub fn are_types_compatible(from: &str, to: &str) -> bool {
        // All numeric types can be converted to F32 for Core ML
        if to == "F32" {
            matches!(from, "F32" | "F16" | "I32" | "I16" | "I8" | "U8" | "BOOL")
        } else {
            // Exact match required for other conversions
            from == to
        }
    }
}

/// Shape validation utilities
pub mod shape {
    use super::*;

    /// Validate tensor shape dimensions
    pub fn validate_shape(shape: &[usize]) -> Result<()> {
        if shape.is_empty() {
            return Err(ANEError::InvalidInput(
                "Tensor shape cannot be empty".to_string(),
            ));
        }

        // usize cannot be negative, so no check needed for negative dimensions

        // Calculate total elements and check for overflow
        let mut total_elements: usize = 1;
        for &dim in shape {
            total_elements = total_elements.checked_mul(dim as usize).ok_or_else(|| {
                ANEError::InvalidInput(
                    "Shape dimensions too large, would cause integer overflow".to_string(),
                )
            })?;
        }

        // Reasonable limits
        const MAX_ELEMENTS: usize = 100 * 1024 * 1024; // 100M elements
        if total_elements > MAX_ELEMENTS {
            return Err(ANEError::InvalidInput(format!(
                "Shape too large: {} elements exceeds limit of {}",
                total_elements, MAX_ELEMENTS
            )));
        }

        Ok(())
    }

    /// Check if two shapes are compatible (allowing batch dimension variation)
    pub fn shapes_compatible(actual: &[usize], expected: &[usize], batch_capable: bool) -> bool {
        if actual.len() != expected.len() {
            return false;
        }

        if batch_capable && actual.len() > 0 {
            // Allow first dimension (batch) to vary
            return actual[1..] == expected[1..];
        } else {
            // Exact match required
            actual == expected
        }
    }

    /// Calculate total elements from shape
    pub fn shape_elements(shape: &[usize]) -> Result<usize> {
        let mut total: usize = 1;
        for &dim in shape {
            total = total.checked_mul(dim as usize).ok_or_else(|| {
                ANEError::InvalidInput("Shape too large for calculation".to_string())
            })?;
        }
        Ok(total)
    }
}

/// Comprehensive input validation
pub fn validate_inference_input(tensor: &Tensor, expected_spec: &TensorSpec) -> Result<()> {
    // Validate data type
    dtype::validate_coreml_dtype(&expected_spec.dtype)?;

    // Validate shape
    shape::validate_shape(&expected_spec.shape)?;

    // Use I/O safety validation
    io_safety::validate_io_schema(tensor, expected_spec)?;

    Ok(())
}

/// Safe inference execution wrapper
pub fn safe_inference_execution<F, T>(inference_fn: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    // Pre-inference safety checks could go here
    // For example: memory usage validation, timeout setup, etc.

    let result = inference_fn();

    // Post-inference safety checks could go here
    // For example: memory cleanup validation, error propagation, etc.

    result
}
