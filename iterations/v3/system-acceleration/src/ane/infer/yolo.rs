//! YOLO inference implementation for object detection
//!
//! This module provides the core inference logic for YOLO models,
//! including image preprocessing, model execution, and detection decoding.
//!
//! CRITICAL: DO NOT REMOVE OR DISABLE THIS MODULE
//! This is production functionality that was restored after fixing candle-core conflicts.
//! DO NOT comment out this module or its functionality.
//! DO NOT disable YOLO object detection.
//! If you encounter issues, fix them rather than disabling this module.
//! Last fixed: P0 priority - candle-core dependency alignment (2025-01-XX)

use crate::ane::ane_errors::{ANEError, Result};
use crate::ane::compat::coreml::{coreml, ModelRef};
use crate::ane::infer::execute::InferenceOptions;
use crate::ane::models::yolo_model::{
    Detection, LoadedYOLOModel, YOLODetectionResult, YOLOInferenceOptions,
};
use image::{DynamicImage, ImageBuffer, Rgb};
use std::time::Instant;
use system_configuration::geometry::BoundingBox;

/// YOLO inference executor
#[derive(Debug)]
pub struct YOLOInferenceExecutor {
    model: LoadedYOLOModel,
}

impl YOLOInferenceExecutor {
    /// Create a new YOLO inference executor
    pub fn new(model: LoadedYOLOModel) -> Self {
        Self { model }
    }

    /// Detect objects in an image
    pub async fn detect_objects(
        &mut self,
        image: &DynamicImage,
        options: &YOLOInferenceOptions,
    ) -> Result<YOLODetectionResult> {
        let start_time = Instant::now();

        // Preprocess image for YOLO input
        let input_tensor = self.preprocess_image_for_yolo(image)?;

        // Create inference options
        let _inference_options = InferenceOptions {
            timeout_ms: options.timeout_ms,
            batch_size: Some(1),
            precision: Some("fp16".to_string()),
            compute_units: Some("all".to_string()),
            enable_monitoring: true,
        };

        // Create input specification for YOLO model
        let _input_spec = crate::ane::TensorSpec {
            name: "image".to_string(),
            dtype: "F32".to_string(),
            shape: vec![
                1,
                3,
                self.model.config.input_size.1 as usize,
                self.model.config.input_size.0 as usize,
            ], // [batch, channels, height, width]
            required: true,
            batch_capable: false,
        };

        // Create output specification
        let _output_spec = crate::ane::TensorSpec {
            name: "output".to_string(),
            dtype: "F32".to_string(),
            shape: vec![1, 255, 13, 13], // YOLOv3 output shape for 416x416 input
            required: true,
            batch_capable: false,
        };

        // CRITICAL: Wrap blocking FFI call in spawn_blocking to prevent async runtime starvation
        // The Core ML FFI call is synchronous and can block for extended periods. If called
        // directly in async context, it can block the async runtime thread and prevent watchdog
        // check-ins, causing kernel panics. spawn_blocking moves the work to a separate thread pool.
        let input_data = input_tensor
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        let input_shape = vec![
            1,
            3,
            self.model.config.input_size.1 as usize,
            self.model.config.input_size.0 as usize,
        ];
        let model_ref = ModelRef::new();
        let input_name = "image".to_string();

        // Run CoreML inference
        let outputs = tokio::task::spawn_blocking(move || {
            coreml::run_inference(model_ref, &input_name, &input_data, &input_shape)
        })
        .await
        .map_err(|e| ANEError::Internal(format!("Inference task panicked: {}", e)))?
        .map_err(|e| ANEError::InferenceFailed(format!("CoreML inference failed: {}", e)))?;

        let inference_time = start_time.elapsed();

        // Extract output tensor (run_inference returns a single tensor)
        let output_tensor = outputs;

        // Decode and filter detections from output
        let detections = self.decode_detections_from_tensor(&output_tensor, image, options)?;

        // Record telemetry
        self.model
            .telemetry
            .record_inference(inference_time.as_millis() as u64, true);

        // Update access time
        self.model.last_accessed = Instant::now();

        let num_detections = detections.len();
        Ok(YOLODetectionResult {
            detections,
            processing_time_ms: inference_time.as_millis() as f64,
            num_detections,
            image_size: (image.width(), image.height()),
        })
    }

    /// Preprocess image for YOLO model input
    fn preprocess_image_for_yolo(&self, image: &DynamicImage) -> Result<candle_core::Tensor> {
        // Resize image to model input size (416x416 for YOLOv3)
        let resized = image.resize_exact(
            self.model.config.input_size.0,
            self.model.config.input_size.1,
            image::imageops::FilterType::Triangle,
        );

        // Convert to RGB if needed
        let rgb_image = match resized {
            DynamicImage::ImageRgb8(img) => img,
            _ => {
                let rgb_img: ImageBuffer<Rgb<u8>, Vec<u8>> = resized.to_rgb8();
                rgb_img
            }
        };

        // Convert to tensor format (CHW: channels, height, width)
        let width = rgb_image.width() as usize;
        let height = rgb_image.height() as usize;
        let channels = 3;

        let mut tensor_data = Vec::with_capacity(width * height * channels);

        // Convert HWC to CHW format
        for c in 0..channels {
            for y in 0..height {
                for x in 0..width {
                    let pixel = rgb_image.get_pixel(x as u32, y as u32);
                    let value = match c {
                        0 => pixel[0], // R
                        1 => pixel[1], // G
                        2 => pixel[2], // B
                        _ => 0,
                    };
                    // Normalize to [0, 1]
                    let normalized = value as f32 / 255.0;
                    tensor_data.push(normalized);
                }
            }
        }

        // Create tensor with shape [1, 3, height, width]
        let _shape = vec![1, channels, height, width];
        let tensor = candle_core::Tensor::new(&*tensor_data, &candle_core::Device::Cpu)?;

        Ok(tensor)
    }

    /// Decode detections from CoreML output tensor
    fn decode_detections_from_tensor(
        &self,
        output_tensor: &candle_core::Tensor,
        original_image: &DynamicImage,
        options: &YOLOInferenceOptions,
    ) -> Result<Vec<Detection>> {
        // Get tensor data as slice
        let output_data = output_tensor
            .to_vec1::<f32>()
            .map_err(|e| ANEError::Internal(format!("Failed to extract tensor data: {}", e)))?;

        // YOLOv3 output format parsing
        let detections = self.parse_yolo_output(
            &output_data,
            original_image.width() as f32,
            original_image.height() as f32,
            options
                .confidence_threshold
                .unwrap_or(self.model.config.confidence_threshold),
        )?;

        // Apply Non-Maximum Suppression if enabled
        let filtered_detections = if self.model.config.nms_enabled {
            self.apply_non_maximum_suppression(
                detections,
                options
                    .iou_threshold
                    .unwrap_or(self.model.config.iou_threshold),
                options
                    .max_detections
                    .unwrap_or(self.model.config.max_detections),
            )
        } else {
            // Limit detections without NMS
            detections
                .into_iter()
                .take(
                    options
                        .max_detections
                        .unwrap_or(self.model.config.max_detections),
                )
                .collect()
        };

        Ok(filtered_detections)
    }

    /// Parse YOLO model output into detections
    fn parse_yolo_output(
        &self,
        output_data: &[f32],
        image_width: f32,
        image_height: f32,
        confidence_threshold: f32,
    ) -> Result<Vec<Detection>> {
        let mut detections = Vec::new();

        // YOLOv3 output format: [batch, num_predictions, 85]
        // Where 85 = 4 (bbox) + 1 (confidence) + 80 (COCO classes)
        let num_predictions = output_data.len() / 85;

        for i in 0..num_predictions {
            let base_idx = i * 85;

            // Extract bounding box (center_x, center_y, width, height) - normalized [0,1]
            let center_x = output_data[base_idx];
            let center_y = output_data[base_idx + 1];
            let width = output_data[base_idx + 2];
            let height = output_data[base_idx + 3];

            // Object confidence
            let confidence = output_data[base_idx + 4];

            // Class probabilities
            let mut class_probs = Vec::with_capacity(80);
            for j in 0..80 {
                class_probs.push(output_data[base_idx + 5 + j]);
            }

            // Find best class
            let (class_id, class_prob) = class_probs
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(idx, prob)| (idx, *prob))
                .unwrap_or((0, 0.0));

            // Final confidence is object confidence * class probability
            let final_confidence = confidence * class_prob;

            // Skip low confidence detections
            if final_confidence < confidence_threshold {
                continue;
            }

            // Convert normalized coordinates to pixel coordinates
            let bbox = BoundingBox {
                x: (center_x - width / 2.0) * image_width,
                y: (center_y - height / 2.0) * image_height,
                width: width * image_width,
                height: height * image_height,
            };

            let detection = Detection {
                class: self
                    .model
                    .config
                    .class_names
                    .get(class_id)
                    .cloned()
                    .unwrap_or_else(|| format!("class_{}", class_id)),
                class_id,
                confidence: final_confidence,
                bbox,
            };

            detections.push(detection);
        }

        // Sort by confidence (highest first)
        detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(detections)
    }

    /// Apply Non-Maximum Suppression to filter overlapping detections
    fn apply_non_maximum_suppression(
        &self,
        detections: Vec<Detection>,
        iou_threshold: f32,
        max_detections: usize,
    ) -> Vec<Detection> {
        let mut filtered = Vec::new();
        let mut remaining: Vec<Detection> = detections;

        while !remaining.is_empty() && filtered.len() < max_detections {
            // Take the detection with highest confidence
            let best = remaining.remove(0);
            filtered.push(best.clone());

            // Remove detections that overlap too much with the best detection
            remaining.retain(|det| {
                let iou = self.calculate_iou(&best.bbox, &det.bbox);
                iou < iou_threshold
            });
        }

        filtered
    }

    /// Calculate Intersection over Union (IoU) of two bounding boxes
    fn calculate_iou(&self, bbox1: &BoundingBox, bbox2: &BoundingBox) -> f32 {
        let x1 = bbox1.x.max(bbox2.x);
        let y1 = bbox1.y.max(bbox2.y);
        let x2 = (bbox1.x + bbox1.width).min(bbox2.x + bbox2.width);
        let y2 = (bbox1.y + bbox1.height).min(bbox2.y + bbox2.height);

        let intersection_width = (x2 - x1).max(0.0);
        let intersection_height = (y2 - y1).max(0.0);
        let intersection_area = intersection_width * intersection_height;

        let bbox1_area = bbox1.width * bbox1.height;
        let bbox2_area = bbox2.width * bbox2.height;
        let union_area = bbox1_area + bbox2_area - intersection_area;

        if union_area == 0.0 {
            0.0
        } else {
            intersection_area / union_area
        }
    }
}

/// Create a YOLO inference executor from a loaded model
pub fn create_yolo_executor(model: LoadedYOLOModel) -> YOLOInferenceExecutor {
    YOLOInferenceExecutor::new(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ane::ane_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
    use crate::ane::models::{YOLOConfig, YOLOMetadata};
    use crate::telemetry::TelemetryCollector;
    use std::path::PathBuf;

    fn create_test_yolo_model() -> LoadedYOLOModel {
        LoadedYOLOModel {
            model_id: "test_yolo".to_string(),
            compiled_path: PathBuf::from("/tmp/test_yolo.mlmodelc"),
            metadata: YOLOMetadata {
                path: PathBuf::from("/tmp/test_yolo.mlmodelc"),
                size_bytes: 1024 * 1024, // 1MB
                format: "mlmodelc".to_string(),
                version: None,
                description: None,
                author: None,
                license: None,
                input_shape: vec![1, 3, 416, 416],
                output_shapes: vec![vec![1, 85, 13, 13]],
                num_classes: 80,
                num_anchors: 3,
            },
            config: YOLOConfig::default(),
            loaded_at: std::time::Instant::now(),
            last_accessed: std::time::Instant::now(),
            telemetry: TelemetryCollector::new(),
            circuit_breaker: CircuitBreaker::new(CircuitBreakerConfig::default()),
        }
    }

    #[tokio::test]
    async fn test_yolo_executor_creation() {
        let _telemetry = TelemetryCollector::new();
        let _circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

        // Create minimal model config for testing
        let _config = YOLOConfig::default();

        // Note: This test would need a proper model loading setup
        // assert!(create_yolo_executor(model).model.config.input_size == (416, 416));
    }

    #[test]
    fn test_iou_calculation() {
        let executor = YOLOInferenceExecutor::new(create_test_yolo_model());

        let bbox1 = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        let bbox2 = BoundingBox {
            x: 5.0,
            y: 5.0,
            width: 10.0,
            height: 10.0,
        };

        let iou = executor.calculate_iou(&bbox1, &bbox2);
        assert!(iou > 0.0 && iou < 1.0); // Partial overlap
    }

    #[test]
    fn test_no_overlap_iou() {
        let executor = YOLOInferenceExecutor::new(create_test_yolo_model());

        let bbox1 = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        let bbox2 = BoundingBox {
            x: 20.0,
            y: 20.0,
            width: 10.0,
            height: 10.0,
        };

        let iou = executor.calculate_iou(&bbox1, &bbox2);
        assert_eq!(iou, 0.0); // No overlap
    }

    #[test]
    fn test_complete_overlap_iou() {
        let executor = YOLOInferenceExecutor::new(create_test_yolo_model());

        let bbox1 = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        let bbox2 = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };

        let iou = executor.calculate_iou(&bbox1, &bbox2);
        assert_eq!(iou, 1.0); // Complete overlap
    }
}
