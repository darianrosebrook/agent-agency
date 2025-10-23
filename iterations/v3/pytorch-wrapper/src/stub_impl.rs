//! Stub implementation when PyTorch is not available

use super::*;

/// Stub PyTorch engine implementation
pub struct StubPyTorchEngine {
    devices: Vec<Device>,
}

#[async_trait]
impl PyTorchEngineTrait for StubPyTorchEngine {
    async fn new() -> Result<Self> {
        tracing::warn!("PyTorch not available, using stub implementation");

        Ok(Self {
            devices: vec![Device::Cpu], // Only CPU available in stub mode
        })
    }

    async fn load_model(&self, _path: &str) -> Result<Box<dyn Model>> {
        Err(crate::PyTorchError::PyTorchUnavailable {
            message: "PyTorch is not available. Enable the 'pytorch' feature to load models.".to_string(),
        }.into())
    }

    fn is_available(&self) -> bool {
        false // PyTorch is not available in stub mode
    }

    fn version(&self) -> String {
        "Stub Implementation (PyTorch not available)".to_string()
    }

    fn available_devices(&self) -> Vec<Device> {
        self.devices.clone()
    }
}

/// Stub model implementation
struct StubModel {
    input_shape: Vec<usize>,
    output_shape: Vec<usize>,
    metadata: ModelMetadata,
}

#[async_trait]
impl Model for StubModel {
    async fn forward(&self, _input: Tensor) -> Result<Tensor, crate::PyTorchError> {
        Err(crate::PyTorchError::PyTorchUnavailable {
            message: "PyTorch is not available. Enable the 'pytorch' feature for inference.".to_string(),
        }.into())
    }

    fn input_shape(&self) -> Vec<usize> {
        self.input_shape.clone()
    }

    fn output_shape(&self) -> Vec<usize> {
        self.output_shape.clone()
    }

    fn metadata(&self) -> ModelMetadata {
        self.metadata.clone()
    }
}

/// Utility functions for ndarray-based operations when PyTorch is not available
pub mod ndarray_ops {
    use ndarray::ArrayD;

    /// Create a tensor from data and shape using ndarray
    pub fn tensor_from_data(data: &[f32], shape: &[usize]) -> Result<ArrayD<f32>, crate::PyTorchError> {
        if data.len() != shape.iter().product::<usize>() {
            return Err(crate::PyTorchError::InvalidShape {
                message: format!(
                    "Data length {} doesn't match shape product {}",
                    data.len(),
                    shape.iter().product::<usize>()
                ),
            });
        }

        ArrayD::from_shape_vec(shape, data.to_vec())
            .map_err(|e| crate::PyTorchError::InvalidShape {
                message: format!("Failed to create ndarray: {}", e),
            })
    }

    /// Basic matrix multiplication using ndarray
    pub fn matmul(a: &ArrayD<f32>, b: &ArrayD<f32>) -> Result<ArrayD<f32>, crate::PyTorchError> {
        if a.ndim() != 2 || b.ndim() != 2 {
            return Err(crate::PyTorchError::InvalidShape {
                message: "Matrix multiplication requires 2D arrays".to_string(),
            });
        }

        if a.shape()[1] != b.shape()[0] {
            return Err(crate::PyTorchError::InvalidShape {
                message: format!(
                    "Incompatible shapes for matrix multiplication: {}x{} and {}x{}",
                    a.shape()[0], a.shape()[1], b.shape()[0], b.shape()[1]
                ),
            });
        }

        // Simple matrix multiplication implementation
        let (m, k) = (a.shape()[0], a.shape()[1]);
        let n = b.shape()[1];
        let mut result = ArrayD::zeros(vec![m, n]);

        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for p in 0..k {
                    sum += a[[i, p]] * b[[p, j]];
                }
                result[[i, j]] = sum;
            }
        }

        Ok(result)
    }

    /// Element-wise operations
    pub fn relu(input: &ArrayD<f32>) -> ArrayD<f32> {
        input.mapv(|x| if x > 0.0 { x } else { 0.0 })
    }

    pub fn sigmoid(input: &ArrayD<f32>) -> ArrayD<f32> {
        input.mapv(|x| 1.0 / (1.0 + (-x).exp()))
    }

    pub fn tanh(input: &ArrayD<f32>) -> ArrayD<f32> {
        input.mapv(|x| x.tanh())
    }
}

impl Tensor {
    /// Create tensor from vector and shape
    pub fn from_vec(data: Vec<f32>, shape: &[usize]) -> Result<Self, crate::PyTorchError> {
        if data.len() != shape.iter().product::<usize>() {
            return Err(crate::PyTorchError::InvalidShape {
                message: format!(
                    "Data length {} doesn't match shape product {}",
                    data.len(),
                    shape.iter().product::<usize>()
                ),
            });
        }

        Ok(Self {
            shape: shape.to_vec(),
            data,
            device: Device::Cpu,
        })
    }

    /// Convert to ndarray for CPU operations
    pub fn to_ndarray(&self) -> Result<ndarray::ArrayD<f32>, crate::PyTorchError> {
        ndarray::ArrayD::from_shape_vec(self.shape.clone(), self.data.clone())
            .map_err(|e| crate::PyTorchError::InvalidShape {
                message: format!("Failed to convert to ndarray: {}", e),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stub_engine_creation() {
        let engine = StubPyTorchEngine::new().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_stub_not_available() {
        let engine = StubPyTorchEngine::new().await.unwrap();
        assert!(!engine.is_available());
    }

    #[tokio::test]
    async fn test_stub_load_model_fails() {
        let engine = StubPyTorchEngine::new().await.unwrap();
        let result = engine.load_model("dummy.pt").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_tensor_from_vec() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];
        let tensor = Tensor::from_vec(data, &shape);
        assert!(tensor.is_ok());

        let tensor = tensor.unwrap();
        assert_eq!(tensor.shape, shape);
        assert_eq!(tensor.data, vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_tensor_from_vec_invalid_shape() {
        let data = vec![1.0, 2.0, 3.0];
        let shape = vec![2, 2]; // 2*2 = 4, but we have 3 elements
        let tensor = Tensor::from_vec(data, &shape);
        assert!(tensor.is_err());
    }
}
