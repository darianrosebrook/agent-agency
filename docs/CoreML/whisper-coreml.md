# Whisper-CoreML Integration Plan

**Model**: OpenAI Whisper Large-v3 (CoreML-optimized)  
**Primary Use Case**: Multilingual speech-to-text transcription with timestamps  
**Target Performance**: 2.8-3.5x ANE speedup vs CPU inference  
**Integration Priority**: 🔴 HIGH (Critical for multimodal RAG and video analysis)

## Executive Summary

Whisper-CoreML will replace the placeholder speech processing in our ASR enricher, providing high-accuracy, timestamped transcripts for video content analysis. This enables the Research Agent to process audio/video content for contextual evidence gathering and the multimodal RAG system to index spoken content with precise timing information.

## Current State Assessment

### Existing Infrastructure
- ✅ **ASR Enricher**: `enrichers/src/asr_enricher.rs` with FFI bindings
- ✅ **Speech Bridge**: `apple-silicon/src/speech_bridge.rs` (placeholder)
- ✅ **Video Ingestor**: `ingestors/src/video_ingestor.rs` with AVAssetReader
- ✅ **Circuit Breaker**: Protection against failures
- ❌ **CoreML Integration**: No Whisper model loaded

### Performance Baseline
- **Current**: Placeholder implementation returning "transcribed text"
- **Target ANE Speedup**: 2.8x minimum (measured vs CPU-only Whisper)
- **Latency Target**: <500ms for 30-second audio segments
- **Accuracy Target**: >95% WER (Word Error Rate) on technical content

## Implementation Details

### Model Specifications
```yaml
Model: Whisper-Large-v3-CoreML
Size: ~1.5GB (quantized FP16)
Input: 16kHz mono audio (raw samples or WAV)
Output: Timestamped transcript with confidence scores
Languages: 99+ supported (English-optimized)
ANE Coverage: ~85% (estimated)
Memory Usage: ~2.2GB peak during inference
```

### CoreML Bridge Integration

#### 1. Model Loading (`apple-silicon/src/ane/`)
```rust
// Extend CoreMLModelLoader to support Whisper
impl CoreMLModelLoader {
    pub async fn load_whisper_model(&self) -> Result<WhisperModel> {
        let model_path = self.models_dir.join("Whisper-Large-v3.mlmodelc");
        let compiled_path = self.compile_if_needed(&model_path, &CompilationOptions {
            precision: Some("fp16".to_string()),
            compute_units: Some("all".to_string()),
            ..Default::default()
        }).await?;

        // Load with CoreML bridge
        let handle = coreml_load_model(compiled_path.to_str().unwrap())?;
        let schema = coreml_model_schema(handle)?;

        Ok(WhisperModel {
            handle,
            schema,
            telemetry: self.telemetry.clone(),
            circuit_breaker: self.circuit_breaker.clone(),
        })
    }
}
```

#### 2. Inference Pipeline
```rust
pub struct WhisperInference {
    model: WhisperModel,
    preprocessor: AudioPreprocessor,
    postprocessor: TranscriptPostprocessor,
}

impl WhisperInference {
    pub async fn transcribe(&self, audio: &[f32]) -> Result<WhisperTranscript> {
        // Preprocessing: Convert to model input format
        let input_tensor = self.preprocessor.process(audio)?;

        // ANE-accelerated inference
        let start_time = Instant::now();
        let outputs = self.model.predict(input_tensor).await?;
        let inference_time = start_time.elapsed();

        // Postprocessing: Decode tokens to text with timestamps
        let transcript = self.postprocessor.decode(outputs)?;

        // Telemetry recording
        self.model.telemetry.record_inference("whisper", inference_time, audio.len());

        Ok(transcript)
    }
}
```

### Audio Preprocessing Bridge

#### Swift Audio Bridge (`coreml-bridge/`)
```swift
// Audio preprocessing in Swift for optimal performance
class AudioPreprocessor {
    static func preprocessAudio(_ samples: [Float], sampleRate: Double) -> MLMultiArray {
        // Convert to 16kHz if needed
        let resampled = resampleTo16kHz(samples, from: sampleRate)

        // Normalize and pad to 30 seconds
        let normalized = normalizeAudio(resampled)
        let padded = padTo30Seconds(normalized)

        // Convert to MLMultiArray
        return createMLMultiArray(from: padded)
    }

    private static func resampleTo16kHz(_ samples: [Float], from inputRate: Double) -> [Float] {
        // High-quality resampling using Accelerate framework
        // ...implementation...
    }
}
```

## Integration Points

### 1. Video Ingestor Enhancement (`ingestors/src/video_ingestor.rs`)

#### Current Implementation Gap
```rust
// Current: No audio extraction
pub async fn ingest(&self, path: &Path, project_scope: Option<&str>) -> Result<IngestResult> {
    // Extract video frames only
    let frames = self.extract_frames(path).await?;
    // MISSING: Audio track extraction and transcription
}
```

#### Enhanced Implementation
```rust
pub async fn ingest(&self, path: &Path, project_scope: Option<&str>) -> Result<IngestResult> {
    // Extract video frames
    let frames = self.extract_frames(path).await?;

    // NEW: Extract and transcribe audio
    let transcript = self.extract_audio_transcript(path).await?;

    // Integrate transcript with frame metadata
    let enriched_frames = self.enrich_frames_with_transcript(frames, &transcript).await?;

    Ok(IngestResult {
        frames: enriched_frames,
        transcript: Some(transcript),
        // ... other metadata
    })
}

async fn extract_audio_transcript(&self, video_path: &Path) -> Result<Transcript> {
    // Extract audio track using AVAssetReader
    let audio_samples = self.av_asset_reader.extract_audio(video_path).await?;

    // Transcribe using Whisper-CoreML
    let whisper = self.apple_silicon.whisper()?;
    let transcript = whisper.transcribe(&audio_samples).await?;

    Ok(transcript)
}
```

### 2. ASR Enricher Integration (`enrichers/src/asr_enricher.rs`)

#### Current FFI Bindings (Replace with CoreML)
```rust
// Current: Apple Speech Framework (limited accuracy)
extern "C" {
    fn speech_recognize_audio(
        audioPath: *const std::ffi::c_char,
        outText: *mut *mut std::ffi::c_char,
        outConfidence: *mut f32,
        outError: *mut *mut std::ffi::c_char,
    ) -> std::ffi::c_int;
}
```

#### Enhanced CoreML Integration
```rust
pub struct AsrEnricher {
    whisper: WhisperInference,
    apple_speech_fallback: AppleSpeechBridge, // Fallback for unsupported languages
    circuit_breaker: CircuitBreaker,
}

impl AsrEnricher {
    pub async fn enrich(&self, audio_path: &Path) -> Result<AsrResult> {
        // Primary: Whisper-CoreML for accuracy
        match self.whisper.transcribe_audio_file(audio_path).await {
            Ok(transcript) => Ok(transcript),
            Err(e) => {
                // Fallback: Apple Speech Framework
                self.apple_speech_fallback.transcribe(audio_path).await
            }
        }
    }
}
```

### 3. Research Agent Integration (`research/src/multimodal_context_provider.rs`)

#### Evidence Enrichment
```rust
impl MultimodalContextProvider {
    pub async fn enrich_with_transcript(
        &self,
        query: &str,
        transcript: &WhisperTranscript,
    ) -> Result<EnrichedEvidence> {
        // Extract relevant segments using timestamps
        let relevant_segments = self.extract_relevant_segments(query, transcript).await?;

        // Generate contextual summaries
        let summaries = self.generate_segment_summaries(relevant_segments).await?;

        Ok(EnrichedEvidence {
            transcript_segments: relevant_segments,
            contextual_summaries: summaries,
            confidence_scores: transcript.confidence_scores.clone(),
        })
    }

    async fn extract_relevant_segments(
        &self,
        query: &str,
        transcript: &WhisperTranscript,
    ) -> Result<Vec<TranscriptSegment>> {
        // Use semantic search to find relevant audio segments
        // Return segments with precise timestamps for video playback
    }
}
```

### 4. Multimodal RAG Integration (`research/src/multimodal_retriever.rs`)

#### Vector Indexing with Timestamps
```rust
impl MultimodalRetriever {
    pub async fn index_transcript(&self, transcript: &WhisperTranscript, video_id: &str) -> Result<()> {
        for segment in &transcript.segments {
            // Create searchable chunks with temporal metadata
            let chunk = SearchChunk {
                content: segment.text.clone(),
                metadata: ChunkMetadata {
                    video_id: video_id.to_string(),
                    start_time: segment.start_time,
                    end_time: segment.end_time,
                    confidence: segment.confidence,
                },
            };

            // Generate embeddings and index
            let embedding = self.embedding_service.embed_text(&chunk.content).await?;
            self.vector_store.insert(chunk, embedding).await?;
        }

        Ok(())
    }

    pub async fn search_with_temporal_context(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<TemporalSearchResult>> {
        // Semantic search with temporal ranking
        let results = self.vector_store.search(query, max_results).await?;

        // Enrich with video playback URLs and timestamps
        let enriched_results = self.enrich_with_video_context(results).await?;

        Ok(enriched_results)
    }
}
```

## Performance Improvements

### Quantitative Targets

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| **Transcription Accuracy** | 85% WER | 95% WER | +12% accuracy |
| **Processing Speed** | N/A (placeholder) | 2.8x CPU baseline | 2.8x faster |
| **Memory Usage** | 500MB | 2.2GB | Required for large model |
| **ANE Utilization** | 0% | 85% | Full ANE acceleration |
| **Latency (30s audio)** | N/A | <500ms | Real-time capable |

### Qualitative Benefits

1. **Enhanced Video Analysis**: Precise timestamped transcripts enable frame-accurate video evidence
2. **Improved Research Context**: Spoken content becomes searchable and retrievable
3. **Multilingual Support**: 99+ languages vs current English-only Apple Speech
4. **Better Evidence Quality**: Higher accuracy transcripts improve judge deliberations
5. **Offline Processing**: No external API dependencies for audio transcription

## Requirements Checklist

### 🔴 Critical Requirements (Must Complete)
- [ ] **Model Acquisition**: Download and validate Whisper-Large-v3-CoreML model (~1.5GB)
- [ ] **CoreML Bridge**: Implement Swift bridge for audio preprocessing
- [ ] **ANE Integration**: Load model with existing telemetry and circuit breaker
- [ ] **FFI Bindings**: Rust FFI to Swift CoreML wrapper
- [ ] **Audio Pipeline**: 16kHz resampling and 30-second chunking
- [ ] **Timestamp Decoding**: Extract precise start/end times from model output
- [ ] **Error Handling**: Circuit breaker integration and CPU fallback

### 🟡 High Priority Requirements
- [ ] **Video Ingestor Integration**: Extract audio from video files using AVAssetReader
- [ ] **ASR Enricher Migration**: Replace Apple Speech with Whisper as primary
- [ ] **Fallback Strategy**: Apple Speech as backup for unsupported languages
- [ ] **Transcript Segmentation**: Split long audio into optimal chunks
- [ ] **Confidence Scoring**: Surface model confidence in results
- [ ] **Memory Management**: Efficient memory usage for large model

### 🟢 Enhancement Requirements
- [ ] **Multilingual Optimization**: Language detection and model selection
- [ ] **Speaker Diarization**: Identify different speakers in audio
- [ ] **Keyword Spotting**: Fast search for specific terms in transcripts
- [ ] **Compression**: Reduce model size while maintaining accuracy
- [ ] **Batch Processing**: Process multiple audio files concurrently
- [ ] **Real-time Streaming**: Support for live audio transcription

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_whisper_model_loading() {
    // Verify model loads successfully
    // Check schema matches expected input/output
    // Validate telemetry integration
}

#[test]
fn test_audio_preprocessing() {
    // Test 16kHz resampling accuracy
    // Verify 30-second padding
    // Check MLMultiArray creation
}

#[test]
fn test_transcript_accuracy() {
    // Compare against known ground truth
    // Measure WER improvement over baseline
    // Validate timestamp precision
}
```

### Integration Tests
```rust
#[test]
fn test_video_ingestor_with_transcript() {
    // End-to-end video processing
    // Verify transcript-frame synchronization
    // Test multimodal RAG indexing
}

#[test]
fn test_research_agent_enrichment() {
    // Evidence extraction from transcripts
    // Contextual summary generation
    // Confidence score propagation
}
```

### Performance Tests
```rust
#[test]
fn test_ane_speedup_measurement() {
    // Measure inference time vs CPU-only
    // Validate 2.8x speedup target
    // Profile memory usage
}

#[test]
fn test_throughput_under_load() {
    // Concurrent transcription requests
    // Circuit breaker activation
    // Memory pressure handling
}
```

## Migration Strategy

### Phase 1: Infrastructure (Week 1-2)
1. Acquire Whisper-CoreML model
2. Implement Swift preprocessing bridge
3. Create Rust FFI bindings
4. Add telemetry and circuit breaker integration

### Phase 2: Core Integration (Week 3-4)
1. Replace ASR enricher placeholder
2. Integrate with video ingestor
3. Add fallback to Apple Speech
4. Implement basic transcript segmentation

### Phase 3: Enhanced Features (Week 5-6)
1. Add Research Agent integration
2. Implement multimodal RAG indexing
3. Add multilingual support
4. Performance optimization

### Phase 4: Production (Week 7-8)
1. Comprehensive testing
2. Performance benchmarking
3. Documentation updates
4. Production deployment

## Risk Mitigation

### Technical Risks
- **Model Size**: 1.5GB model may strain memory limits
  - *Mitigation*: Implement model unloading and LRU caching
- **ANE Compatibility**: Not all M-series chips support full acceleration
  - *Mitigation*: Automatic CPU fallback with performance warnings
- **Audio Quality**: Poor preprocessing affects accuracy
  - *Mitigation*: Multiple resampling algorithms with quality validation

### Integration Risks
- **Video Pipeline Complexity**: Coordinating audio extraction and transcription
  - *Mitigation*: Incremental integration starting with audio-only files
- **Performance Impact**: ANE usage may affect concurrent model inference
  - *Mitigation*: Thermal monitoring and adaptive batching
- **Accuracy Regression**: Whisper may be less accurate than cloud services
  - *Mitigation*: Accuracy benchmarking against ground truth datasets

## Success Metrics

### Technical Metrics
- ✅ **ANE Speedup**: ≥2.8x vs CPU-only Whisper
- ✅ **Accuracy**: ≥95% WER on technical content
- ✅ **Latency**: <500ms for 30-second segments
- ✅ **Memory**: <2.5GB peak usage
- ✅ **Reliability**: 99.9% transcription success rate

### Business Impact Metrics
- ✅ **Video Processing**: 3x faster evidence extraction from videos
- ✅ **Research Quality**: 40% more comprehensive context gathering
- ✅ **Judge Efficiency**: 25% faster deliberation with better transcripts
- ✅ **User Experience**: Seamless multimodal content analysis

---

## Implementation Status: 📋 Planned
**Next Action**: Begin Phase 1 infrastructure implementation
**Estimated Completion**: 8 weeks
**Dependencies**: CoreML model availability, ANE testing hardware
**Risk Level**: 🟡 Medium (Established patterns, new model integration)




