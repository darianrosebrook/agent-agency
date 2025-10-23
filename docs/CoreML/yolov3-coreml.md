# YOLOv3-CoreML Integration Plan

**Model**: YOLOv3 (CoreML-optimized)  
**Primary Use Case**: Real-time object detection for UI analysis and security monitoring  
**Target Performance**: 2.5-3x ANE speedup vs CPU inference  
**Integration Priority**: MEDIUM (Enhances vision capabilities, enables new features)

## Executive Summary

YOLOv3-CoreML will provide real-time object detection capabilities, enabling our vision enricher to detect UI elements, monitor development environments, and enhance multimodal evidence gathering. This transforms our current OCR-only vision processing into comprehensive scene understanding with bounding box detection.

## Current State Assessment

### Existing Vision Infrastructure
- **Vision Enricher**: `enrichers/src/vision_enricher.rs` with FFI bindings
- **Vision Bridge**: `apple-silicon/src/vision_bridge.rs` (placeholder)
- **Multiple Ingestors**: Diagrams, slides, video frames
- **Circuit Breaker**: Protection against failures
- **Object Detection**: No bounding box or classification capabilities

### Performance Baseline
- **Current**: OCR-only text extraction using Vision Framework
- **Target Detection Speed**: <100ms per image for real-time analysis
- **Accuracy Target**: >85% mAP (mean Average Precision) on COCO dataset
- **Classes Supported**: 80 COCO object classes

## Implementation Details

### Model Specifications
```yaml
Model: YOLOv3-CoreML
Size: ~240MB (FP16 quantized)
Input: 416x416 RGB images
Output: Bounding boxes with class probabilities (80 classes)
ANE Coverage: ~80% (estimated)
Memory Usage: ~600MB peak during inference
Real-time Capable: Yes (30+ FPS on M-series)
```

### CoreML Bridge Integration

#### 1. Model Loading (`apple-silicon/src/ane/`)
```rust
// Extend CoreMLModelLoader for YOLO detection
impl CoreMLModelLoader {
    pub async fn load_yolo_model(&self) -> Result<YOLOModel> {
        let model_path = self.models_dir.join("YOLOv3.mlmodelc");
        let compiled_path = self.compile_if_needed(&model_path, &CompilationOptions {
            precision: Some("fp16".to_string()),
            compute_units: Some("all".to_string()),
            ..Default::default()
        }).await?;

        let handle = coreml_load_model(compiled_path.to_str().unwrap())?;
        let schema = coreml_model_schema(handle)?;

        Ok(YOLOModel {
            handle,
            input_size: (416, 416),
            confidence_threshold: 0.5,
            iou_threshold: 0.45,
            telemetry: self.telemetry.clone(),
            circuit_breaker: self.circuit_breaker.clone(),
        })
    }
}
```

#### 2. Object Detection Pipeline
```rust
pub struct YOLOInference {
    model: YOLOModel,
    preprocessor: ImagePreprocessor,
    postprocessor: DetectionPostprocessor,
}

impl YOLOInference {
    pub async fn detect_objects(&self, image: &DynamicImage) -> Result<Vec<Detection>> {
        // Preprocessing: Resize and normalize image
        let input_tensor = self.preprocessor.process(image, self.model.input_size)?;

        // ANE-accelerated inference
        let start_time = Instant::now();
        let outputs = self.model.predict(input_tensor).await?;
        let inference_time = start_time.elapsed();

        // Postprocessing: Decode detections with NMS
        let detections = self.postprocessor.decode_detections(
            outputs,
            self.model.confidence_threshold,
            self.model.iou_threshold,
        )?;

        // Telemetry recording
        self.model.telemetry.record_inference("yolo", inference_time, 1);

        Ok(detections)
    }
}
```

### Image Preprocessing Bridge

#### Swift Image Processing
```swift
// High-performance image preprocessing in Swift
class ImagePreprocessor {
    static func preprocessImage(_ image: CGImage, targetSize: CGSize) -> MLMultiArray {
        // Resize image to 416x416 maintaining aspect ratio
        let resizedImage = resizeImage(image, to: targetSize)

        // Convert to RGB pixel buffer
        let pixelBuffer = createPixelBuffer(from: resizedImage)

        // Normalize pixel values (0-255 -> 0-1)
        let normalizedBuffer = normalizePixelBuffer(pixelBuffer)

        // Convert to MLMultiArray
        return createMLMultiArray(from: normalizedBuffer)
    }

    private static func resizeImage(_ image: CGImage, to size: CGSize) -> CGImage {
        // High-quality resizing using Core Graphics
        // Maintain aspect ratio with letterboxing
    }
}
```

## Integration Points

### 1. Vision Enricher Enhancement (`enrichers/src/vision_enricher.rs`)

#### Current Implementation Gap
```rust
// Current: OCR-only text extraction
pub async fn enrich(&self, image_path: &Path) -> Result<VisionResult> {
    // Extract text using Vision Framework
    let text_blocks = self.extract_text(image_path).await?;
    Ok(VisionResult {
        text_blocks,
        objects: vec![], // MISSING: Object detection
        faces: vec![],   // MISSING: Face detection
    })
}
```

#### Enhanced Object Detection
```rust
pub struct VisionEnricher {
    yolo_detector: YOLOInference,
    ocr_extractor: OCRExtractor, // Existing Vision Framework OCR
    face_detector: FaceDetector, // Future enhancement
    circuit_breaker: CircuitBreaker,
}

impl VisionEnricher {
    pub async fn enrich(&self, image_path: &Path) -> Result<VisionResult> {
        // Load image once
        let image = image::open(image_path)?;

        // Parallel processing: OCR + Object Detection
        let (text_blocks, detections) = tokio::try_join!(
            self.ocr_extractor.extract_text(image_path),
            self.yolo_detector.detect_objects(&image)
        )?;

        // Integrate detections with text blocks
        let enriched_text = self.enrich_text_with_objects(text_blocks, &detections)?;

        Ok(VisionResult {
            text_blocks: enriched_text,
            objects: detections,
            faces: vec![], // Future: face detection
        })
    }

    fn enrich_text_with_objects(
        &self,
        text_blocks: Vec<TextBlock>,
        detections: &[Detection],
    ) -> Result<Vec<TextBlock>> {
        // Associate text blocks with detected objects
        // E.g., "button" text block near button detection
    }
}
```

### 2. Diagrams Ingestor Integration (`ingestors/src/diagrams_ingestor.rs`)

#### UI/UX Analysis Enhancement
```rust
impl DiagramsIngestor {
    pub async fn ingest_diagram(&self, path: &Path) -> Result<DiagramAnalysis> {
        // Extract visual elements using YOLO
        let image = image::open(path)?;
        let detections = self.yolo.detect_objects(&image).await?;

        // Categorize detected elements
        let ui_elements = self.categorize_ui_elements(&detections)?;
        let diagram_elements = self.extract_diagram_components(&detections)?;

        // Extract text and associate with elements
        let text_blocks = self.vision_enricher.extract_text(path).await?;
        let enriched_elements = self.associate_text_with_elements(
            ui_elements,
            diagram_elements,
            text_blocks,
        )?;

        Ok(DiagramAnalysis {
            ui_elements: enriched_elements.ui_elements,
            diagram_elements: enriched_elements.diagram_elements,
            layout_analysis: self.analyze_layout(&detections)?,
        })
    }

    fn categorize_ui_elements(&self, detections: &[Detection]) -> Result<Vec<UIElement>> {
        detections.iter()
            .filter(|d| self.is_ui_element(&d.class))
            .map(|d| UIElement {
                element_type: self.map_detection_to_ui_type(&d.class),
                bounding_box: d.bbox.clone(),
                confidence: d.confidence,
                associated_text: None, // To be filled by text association
            })
            .collect()
    }
}
```

### 3. Video Ingestor Integration (`ingestors/src/video_ingestor.rs`)

#### Scene Analysis Enhancement
```rust
impl VideoIngestor {
    pub async fn analyze_video_scene(&self, frame: &VideoFrame) -> Result<SceneAnalysis> {
        // Detect objects in video frame
        let image = DynamicImage::ImageRgb8(frame.data.clone().try_into()?);
        let detections = self.yolo.detect_objects(&image).await?;

        // Analyze scene composition
        let scene_type = self.classify_scene_type(&detections)?;
        let key_objects = self.extract_key_objects(&detections)?;
        let activity_indicators = self.detect_activity(&detections)?;

        Ok(SceneAnalysis {
            scene_type,
            key_objects,
            activity_indicators,
            timestamp: frame.timestamp,
        })
    }

    fn classify_scene_type(&self, detections: &[Detection]) -> Result<SceneType> {
        // Classify video scene based on detected objects
        // E.g., "presentation", "code_demo", "ui_walkthrough"
    }

    fn detect_activity(&self, detections: &[Detection]) -> Result<Vec<ActivityIndicator>> {
        // Detect user activity from object changes
        // E.g., cursor movement, window switching, typing indicators
    }
}
```

### 4. Research Agent Integration (`research/src/multimodal_context_provider.rs`)

#### Visual Evidence Enhancement
```rust
impl MultimodalContextProvider {
    pub async fn enrich_with_visual_evidence(
        &self,
        query: &str,
        detections: &[Detection],
        image_context: &ImageContext,
    ) -> Result<VisualEvidence> {
        // Find relevant objects for the query
        let relevant_objects = self.find_relevant_objects(query, detections).await?;

        // Generate visual context summaries
        let visual_summaries = self.generate_visual_summaries(relevant_objects).await?;

        // Extract spatial relationships
        let spatial_layout = self.analyze_spatial_relationships(detections)?;

        Ok(VisualEvidence {
            relevant_objects,
            visual_summaries,
            spatial_layout,
            confidence_scores: detections.iter().map(|d| d.confidence).collect(),
        })
    }

    async fn find_relevant_objects(&self, query: &str, detections: &[Detection]) -> Result<Vec<Detection>> {
        // Use semantic matching to find objects relevant to query
        // E.g., query "button" should match "button", "switch", "control"
    }
}
```

## Performance Improvements

### Quantitative Targets

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| **Detection Speed** | N/A (OCR only) | <100ms per image | Real-time capable |
| **Accuracy (mAP)** | N/A | >85% on COCO | Industry standard |
| **Memory Usage** | 200MB (OCR) | 600MB | Required for model |
| **ANE Utilization** | 0% | 80% | Full acceleration |
| **Concurrent Processing** | 1 image | 4-8 images | Parallel detection |

### Qualitative Benefits

1. **Enhanced UI Analysis**: Detect buttons, inputs, navigation elements in screenshots
2. **Security Monitoring**: Identify unauthorized windows, suspicious activity
3. **Diagram Understanding**: Recognize flowcharts, architecture diagrams, wireframes
4. **Video Scene Analysis**: Understand presentations, demos, tutorials
5. **Evidence Enrichment**: Provide visual context for judge deliberations

## Requirements Checklist

### Critical Requirements (Must Complete)
- [ ] **Model Acquisition**: Download YOLOv3-CoreML model (~240MB)
- [ ] **Image Preprocessing**: Swift bridge for 416x416 resizing and normalization
- [ ] **Detection Decoding**: Non-maximum suppression and confidence thresholding
- [ ] **Bounding Box Processing**: Coordinate transformation and scaling
- [ ] **ANE Integration**: Load with existing telemetry and circuit breaker
- [ ] **FFI Bindings**: Rust FFI to Swift CoreML wrapper

### High Priority Requirements
- [ ] **Vision Enricher Integration**: Add object detection alongside OCR
- [ ] **Diagrams Ingestor Enhancement**: UI element and diagram component detection
- [ ] **Video Scene Analysis**: Object detection in video frames
- [ ] **Research Agent Integration**: Visual evidence enrichment
- [ ] **COCO Classes Mapping**: Map 80 classes to relevant use cases
- [ ] **Confidence Filtering**: Configurable detection thresholds

### Enhancement Requirements
- [ ] **Custom Training**: Fine-tune for UI elements and diagrams
- [ ] **Multi-scale Detection**: Handle various object sizes
- [ ] **Tracking Integration**: Object tracking across video frames
- [ ] **Real-time Streaming**: Live video object detection
- [ ] **Batch Processing**: Process multiple images efficiently
- [ ] **Model Optimization**: Further quantization for speed

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_yolo_model_loading() {
    // Verify model loads successfully
    // Check input/output schema
    // Validate preprocessing pipeline
}

#[test]
fn test_object_detection_accuracy() {
    // Test against known images
    // Verify bounding box accuracy
    // Check class classification
}

#[test]
fn test_preprocessing_pipeline() {
    // Test image resizing
    // Verify normalization
    // Check MLMultiArray creation
}
```

### Integration Tests
```rust
#[test]
fn test_vision_enricher_with_detection() {
    // End-to-end image processing
    // Verify OCR + object detection integration
    // Test text-object association
}

#[test]
fn test_diagrams_ingestor_ui_analysis() {
    // UI screenshot analysis
    // Button/input detection accuracy
    // Layout understanding
}
```

### Performance Tests
```rust
#[test]
fn test_detection_speed() {
    // Measure <100ms target
    // Profile ANE utilization
    // Test concurrent processing
}

#[test]
fn test_memory_efficiency() {
    // Monitor 600MB peak usage
    // Test memory cleanup
    // Profile memory growth
}
```

## Migration Strategy

### Phase 1: Infrastructure (Week 1)
1. Acquire YOLOv3-CoreML model
2. Implement Swift image preprocessing
3. Create detection decoding logic
4. Add telemetry integration

### Phase 2: Core Integration (Week 2-3)
1. Integrate with vision enricher
2. Add object detection to diagrams ingestor
3. Implement basic video frame analysis
4. Create COCO class mappings

### Phase 3: Enhanced Features (Week 4)
1. Add research agent visual evidence
2. Implement scene analysis for videos
3. Add spatial relationship analysis
4. Performance optimization

### Phase 4: Production (Week 5)
1. Comprehensive testing and validation
2. Performance benchmarking
3. Documentation updates
4. Production deployment

## Risk Mitigation

### Technical Risks
- **Detection Accuracy**: YOLO may not detect UI elements well
  - *Mitigation*: Fine-tuning on UI datasets, confidence thresholds
- **Memory Usage**: 600MB model may limit concurrent processing
  - *Mitigation*: Model unloading, request queuing
- **Preprocessing Overhead**: Image resizing may add latency
  - *Mitigation*: GPU-accelerated preprocessing where possible

### Integration Risks
- **OCR Integration**: Coordinating detection with text extraction
  - *Mitigation*: Parallel processing, result merging
- **Performance Impact**: Additional processing time per image
  - *Mitigation*: Async processing, caching, selective application
- **False Positives**: Incorrect object detections
  - *Mitigation*: Confidence thresholds, human validation for critical cases

## Success Metrics

### Technical Metrics
- **Detection Speed**: <100ms per 416x416 image
- **Accuracy**: >85% mAP on relevant classes
- **Memory Usage**: <600MB peak usage
- **ANE Utilization**: 80%+ acceleration
- **Reliability**: 99.5% successful detection rate

### Business Impact Metrics
- **UI Analysis**: 3x faster interface understanding
- **Security Monitoring**: Real-time workspace monitoring
- **Diagram Processing**: 2x faster architectural analysis
- **Evidence Quality**: 25% more comprehensive visual evidence
- **Research Efficiency**: 30% better context gathering

---

## Implementation Status: Planned
**Next Action**: Begin Phase 1 infrastructure implementation
**Estimated Completion**: 5 weeks
**Dependencies**: YOLOv3-CoreML model availability
**Risk Level**: LOW (Established vision patterns, proven model)




