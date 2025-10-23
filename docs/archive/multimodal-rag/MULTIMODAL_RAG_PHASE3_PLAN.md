# V3 Multimodal RAG System - Phase 3 Integration Plan

**Status**: Planning Phase 3 - Bridges & System Integration  
**Target**: End-to-end multimodal RAG working with Council and Claim Extraction

---

## Phase 3 Overview

Phase 3 focuses on **bridging** to external systems and **integrating** the multimodal RAG into V3's core:

1. **Database Layer** - PostgreSQL pgvector queries
2. **Swift Bridges** - Vision Framework, Apple Speech
3. **Python Bridges** - WhisperX, pyannote, BLIP
4. **System Integration** - Council context, Claim evidence
5. **End-to-End Testing** - Full pipeline validation

---

## Priority 1: PostgreSQL pgvector Integration

**File**: `iterations/v3/indexers/src/database.rs`

### 1.1 Enable pgvector Extension

```sql
-- Run once in target database
CREATE EXTENSION IF NOT EXISTS vector;

-- Create HNSW indices for active models
CREATE INDEX idx_block_vectors_e5_small_v2_hnsw 
  ON block_vectors USING hnsw (vec vector_cosine_ops)
  WHERE model_id = 'e5-small-v2';

CREATE INDEX idx_block_vectors_clip_vit_b32_hnsw
  ON block_vectors USING hnsw (vec vector_ip_ops)
  WHERE model_id = 'clip-vit-b32';
```

### 1.2 Implement VectorStore Methods

```rust
#[async_trait]
impl VectorStore for PostgresVectorStore {
    async fn store_vector(&self, record: BlockVectorRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO block_vectors (block_id, model_id, modality, vec)
            VALUES ($1, $2, $3, $4::vector)
            ON CONFLICT (block_id, model_id) DO UPDATE
            SET vec = EXCLUDED.vec
            "#
        )
        .bind(record.block_id)
        .bind(&record.model_id)
        .bind(&record.modality)
        .bind(&record.vector) // Vec<f32> → pgvector
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn search_similar(
        &self,
        query_vector: &[f32],
        model_id: &str,
        k: usize,
        project_scope: Option<&str>,
    ) -> Result<Vec<(Uuid, f32)>> {
        // Choose operator based on metric
        let operator = match self.get_metric(model_id).await? {
            "cosine" => "<=>",
            "ip" => "<#>",
            "l2" => "<->",
            _ => "<=>"
        };
        
        let query = format!(
            r#"
            SELECT bv.block_id, (bv.vec {} $1::vector) AS distance
            FROM block_vectors bv
            JOIN blocks b ON bv.block_id = b.id
            JOIN segments s ON b.segment_id = s.id
            WHERE bv.model_id = $2
              AND s.project_scope IS NULL OR s.project_scope = $3
            ORDER BY distance
            LIMIT $4
            "#,
            operator
        );

        let results = sqlx::query_as::<_, (Uuid, f32)>(&query)
            .bind(query_vector)
            .bind(model_id)
            .bind(project_scope)
            .bind(k as i64)
            .fetch_all(&self.pool)
            .await?;

        Ok(results)
    }

    async fn log_search(&self, entry: SearchAuditEntry) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO search_logs (query, results, features, created_at)
            VALUES ($1, $2, $3, NOW())
            "#
        )
        .bind(&entry.query)
        .bind(serde_json::to_value(&entry.results)?)
        .bind(serde_json::to_value(&entry.features)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```

---

## Priority 2: Swift Bridges (Vision & Speech)

**File**: `iterations/v3/apple-silicon/src/vision_bridge.rs` (NEW)

### 2.1 Vision Framework Bridge

```rust
// apple-silicon/src/vision_bridge.rs

use anyhow::Result;
use std::ffi::{CStr, CString};

#[link(name = "Foundation", kind = "framework")]
#[link(name = "Vision", kind = "framework")]
extern "C" {
    fn analyze_document_request(
        image_bytes: *const u8,
        image_len: usize,
        timeout_ms: u64,
    ) -> *mut c_char;
}

pub struct VisionBridge;

#[derive(Debug, Serialize, Deserialize)]
pub struct VisionAnalysisResult {
    pub blocks: Vec<VisionBlock>,
    pub tables: Vec<VisionTable>,
    pub confidence: f32,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisionBlock {
    pub text: String,
    pub role: String, // "title", "bullet", "code", "table", "figure"
    pub bbox: BoundingBox,
    pub confidence: f32,
}

impl VisionBridge {
    pub async fn analyze_document(
        image_data: &[u8],
        timeout_ms: u64,
    ) -> Result<VisionAnalysisResult> {
        // Safety: Call Swift bridge with proper memory management
        unsafe {
            let result_ptr = analyze_document_request(
                image_data.as_ptr(),
                image_data.len(),
                timeout_ms,
            );

            if result_ptr.is_null() {
                return Err(anyhow::anyhow!("Vision analysis failed"));
            }

            let result_str = CStr::from_ptr(result_ptr)
                .to_string_lossy()
                .to_string();

            // Free C string
            libc::free(result_ptr as *mut libc::c_void);

            // Parse JSON result
            let analysis: VisionAnalysisResult = serde_json::from_str(&result_str)?;
            Ok(analysis)
        }
    }
}
```

**Swift Side** (`apple-silicon/Sources/VisionBridge.swift`):

```swift
import Foundation
import Vision

@_cdecl("analyze_document_request")
func analyzeDocumentRequest(
    imageBytes: UnsafeRawPointer,
    imageLen: Int,
    timeoutMs: UInt64
) -> UnsafeMutablePointer<CChar> {
    autoreleasepool {
        let imageData = Data(bytes: imageBytes, count: imageLen)
        guard let image = UIImage(data: imageData) else {
            return "{}".cString(using: .utf8).map { UnsafeMutablePointer(mutating: $0) } ?? nil
        }
        
        guard let cgImage = image.cgImage else {
            return "{}".cString(using: .utf8).map { UnsafeMutablePointer(mutating: $0) } ?? nil
        }
        
        var blocks: [[String: Any]] = []
        var tables: [[String: Any]] = []
        
        let request = VNRecognizeDocumentsRequest { request, error in
            if let results = request.results as? [VNRecognizedTextObservation] {
                for observation in results {
                    let text = observation.topCandidates(1).first?.string ?? ""
                    let bbox = observation.boundingBox
                    
                    blocks.append([
                        "text": text,
                        "role": self.classifyBlock(text),
                        "bbox": ["x": bbox.origin.x, "y": bbox.origin.y,
                                "width": bbox.width, "height": bbox.height],
                        "confidence": observation.confidence
                    ])
                }
            }
        }
        
        let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
        try? handler.perform([request])
        
        let result: [String: Any] = [
            "blocks": blocks,
            "tables": tables,
            "confidence": 0.85,
            "processing_time_ms": 0
        ]
        
        if let json = try? JSONSerialization.data(withJSONObject: result),
           let jsonString = String(data: json, encoding: .utf8) {
            return strdup(jsonString)
        }
        
        return strdup("{}")
    }
}

private func classifyBlock(_ text: String) -> String {
    if text.contains("```") { return "code" }
    if text.count > 50 { return "paragraph" }
    return "bullet"
}
```

### 2.2 Apple Speech Framework Bridge

```rust
// Similar pattern for Speech Framework
pub struct SpeechBridge;

pub async fn transcribe_audio(
    audio_data: &[u8],
    language: Option<&str>,
) -> Result<AsrResult> {
    // Call to Swift SFSpeechRecognizer
    // Returns AsrResult with speech_turns + speakers
    todo!()
}
```

---

## Priority 3: Python Bridges (ASR, Visual Captioning)

**File**: `iterations/v3/enrichers/src/python_bridge.rs` (NEW)

### 3.1 WhisperX Integration

```rust
use std::process::{Command, Stdio};
use std::io::Write;

pub async fn transcribe_with_whisperx(
    audio_data: &[u8],
    language: Option<&str>,
) -> Result<AsrResult> {
    // Write audio to temp file
    let temp_path = std::env::temp_dir().join(format!("audio_{}.wav", uuid::Uuid::new_v4()));
    std::fs::write(&temp_path, audio_data)?;
    
    // Call WhisperX subprocess
    let lang_arg = language.unwrap_or("en");
    let output = Command::new("whisperx")
        .arg(temp_path.to_str().unwrap())
        .arg("--language")
        .arg(lang_arg)
        .arg("--diarize_model")
        .arg("pyannote")
        .arg("--output_format")
        .arg("json")
        .arg("--output_dir")
        .arg("/tmp/whisperx_output")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("WhisperX failed: {:?}", String::from_utf8(output.stderr)));
    }

    // Parse JSON output
    let json_output = String::from_utf8(output.stdout)?;
    let result: WhisperXOutput = serde_json::from_str(&json_output)?;
    
    // Convert to AsrResult with word timings
    Ok(convert_whisperx_to_asr_result(result))
}

#[derive(Debug, Deserialize)]
pub struct WhisperXOutput {
    pub segments: Vec<WhisperXSegment>,
    pub language: String,
}

#[derive(Debug, Deserialize)]
pub struct WhisperXSegment {
    pub id: u32,
    pub start: f32,
    pub end: f32,
    pub text: String,
    pub speaker: Option<String>,
    pub words: Vec<WhisperXWord>,
}

#[derive(Debug, Deserialize)]
pub struct WhisperXWord {
    pub word: String,
    pub start: f32,
    pub end: f32,
    pub score: f32,
}
```

### 3.2 BLIP Visual Captioning

```rust
pub async fn caption_with_blip(
    image_data: &[u8],
    context: Option<&str>,
) -> Result<CaptionResult> {
    // Write image to temp file
    let temp_image = std::env::temp_dir().join(format!("image_{}.jpg", uuid::Uuid::new_v4()));
    std::fs::write(&temp_image, image_data)?;
    
    // Call Python script
    let python_script = r#"
import sys
from PIL import Image
from transformers import BlipProcessor, BlipForConditionalGeneration
import json

image_path = sys.argv[1]
image = Image.open(image_path).convert('RGB')
processor = BlipProcessor.from_pretrained("Salesforce/blip-image-captioning-large")
model = BlipForConditionalGeneration.from_pretrained("Salesforce/blip-image-captioning-large")

inputs = processor(image, return_tensors="pt")
out = model.generate(**inputs, max_length=50)
caption = processor.decode(out[0], skip_special_tokens=True)

result = {
    "caption": caption,
    "confidence": 0.85,
    "tags": extract_tags(caption)
}
print(json.dumps(result))
"#;
    
    let output = Command::new("python3")
        .arg("-c")
        .arg(python_script)
        .arg(temp_image.to_str().unwrap())
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("BLIP captioning failed"));
    }

    let result: CaptionResult = serde_json::from_slice(&output.stdout)?;
    Ok(result)
}
```

---

## Priority 4: Council Integration

**File**: `iterations/v3/council/src/multimodal_provider.rs` (NEW)

```rust
use indexers::{JobScheduler, JobType};
use research::MultimodalRetriever;

pub struct MultimodalContextProvider {
    retriever: Arc<MultimodalRetriever>,
    scheduler: Arc<JobScheduler>,
}

#[async_trait]
impl ContextProvider for MultimodalContextProvider {
    async fn gather_context(
        &self,
        topic: &str,
        budget: ContextBudget,  // {max_tokens, k, diversity, time_window}
    ) -> Result<Vec<ContextBlock>> {
        // Check scheduler
        if !self.scheduler.try_acquire(JobType::Embedding)? {
            return Err(anyhow::anyhow!("Embedding queue full"));
        }

        // Search across modalities
        let query = MultimodalQuery {
            text: Some(topic.to_string()),
            query_type: QueryType::Hybrid,
            project_scope: None,
            max_results: budget.k,
        };

        let results = self.retriever.search(&query).await?;

        // Deduplicate and respect token budget
        let blocks = results
            .into_iter()
            .take_while(|r| {
                // Token budgeting logic
                true
            })
            .map(|r| ContextBlock {
                text: r.snippet,
                confidence: r.feature.fused_score,
                citation: r.citation,
                modality: r.kind,
            })
            .collect();

        self.scheduler.release(JobType::Embedding, true);
        Ok(blocks)
    }
}
```

---

## Priority 5: Claim Extraction Integration

**File**: `iterations/v3/claim-extraction/src/multimodal_evidence.rs` (NEW)

```rust
pub struct MultimodalEvidenceCollector {
    retriever: Arc<MultimodalRetriever>,
}

impl MultimodalEvidenceCollector {
    pub async fn collect_evidence_for_claim(
        &self,
        claim: &Claim,
        project_scope: Option<&str>,
    ) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();

        // 1. Visual evidence (diagrams, figures)
        let visual_query = MultimodalQuery {
            text: Some(format!("Diagram or figure related to: {}", claim.text)),
            query_type: QueryType::Visual,
            project_scope: project_scope.map(|s| s.to_string()),
            max_results: 5,
        };

        let visual_results = self.retriever.search(&visual_query).await?;
        for result in visual_results {
            if let Some(citation) = result.citation {
                evidence.push(Evidence {
                    evidence_type: "visual".to_string(),
                    content: result.snippet,
                    confidence: result.feature.fused_score,
                    citation,
                    timestamp: self.extract_timestamp(&result.citation),
                });
            }
        }

        // 2. Speech evidence with timestamps
        let speech_query = MultimodalQuery {
            text: Some(claim.text.clone()),
            query_type: QueryType::Text,
            project_scope: project_scope.map(|s| s.to_string()),
            max_results: 10,
        };

        let speech_results = self.retriever.search(&speech_query).await?;
        for result in speech_results {
            if let Some(citation) = result.citation {
                evidence.push(Evidence {
                    evidence_type: "speech".to_string(),
                    content: result.snippet,
                    confidence: result.feature.fused_score,
                    citation: citation.clone(),
                    timestamp: self.extract_timestamp(&citation),
                });
            }
        }

        Ok(evidence)
    }

    fn extract_timestamp(&self, citation: &str) -> Option<(f32, f32)> {
        // Parse "uri#t0-t1" format
        if let Some(hash_idx) = citation.find('#') {
            let time_part = &citation[hash_idx + 1..];
            let parts: Vec<&str> = time_part.split('-').collect();
            if parts.len() == 2 {
                let t0 = parts[0].parse().ok()?;
                let t1 = parts[1].parse().ok()?;
                return Some((t0, t1));
            }
        }
        None
    }
}
```

---

## Priority 6: End-to-End Testing

**File**: `iterations/v3/integration-tests/tests/multimodal_rag_e2e.rs` (NEW)

```rust
#[tokio::test]
async fn test_multimodal_rag_full_pipeline() -> Result<()> {
    // 1. Setup
    let db = DatabasePool::new("postgresql://...", 5).await?;
    let scheduler = JobScheduler::new(50);
    let retriever = MultimodalRetriever::new(None);

    // 2. Ingest test media
    let test_video = std::fs::read("tests/fixtures/sample_talk.mp4")?;
    let ingest_result = VideoIngestor::new(None, None)
        .ingest(Path::new("tests/fixtures/sample_talk.mp4"), Some("test-project"))
        .await?;

    // 3. Enrich
    let vision_enricher = VisionEnricher::new(EnricherConfig::default());
    let ocr_result = vision_enricher
        .analyze_document(&ingest_result.segments[0].blocks[0].raw_bytes.unwrap(), None)
        .await?;

    // 4. Index
    let mut indexer = MultimodalIndexer::new();
    for segment in &ingest_result.segments {
        for block in &segment.blocks {
            let embeddings = generate_embeddings(&block.text).await?;
            indexer.index_block(block.id, &block.text, "text", embeddings).await?;
        }
    }

    // 5. Retrieve
    let query = MultimodalQuery {
        text: Some("What was discussed about machine learning?".to_string()),
        query_type: QueryType::Hybrid,
        project_scope: Some("test-project".to_string()),
        max_results: 5,
    };

    let results = retriever.search(&query).await?;
    assert!(!results.is_empty());
    assert!(results[0].feature.fused_score > 0.5);

    Ok(())
}
```

---

## Implementation Timeline

**Week 1: Database Layer**
- Implement pgvector queries (1 day)
- Test similarity search (1 day)
- Create audit logging (0.5 day)

**Week 2: Swift Bridges**
- Vision Framework bridge (2 days)
- Apple Speech bridge (2 days)
- Integration tests (1 day)

**Week 3: Python Bridges**
- WhisperX integration (2 days)
- BLIP captioning (2 days)
- Error handling & fallbacks (1 day)

**Week 4: System Integration**
- Council context provider (2 days)
- Claim extraction evidence (2 days)
- End-to-end testing (1 day)

**Week 5: Performance & Polish**
- Latency optimization (2 days)
- Memory profiling (1 day)
- Documentation (1 day)

---

## Success Criteria (Phase 3)

- PostgreSQL pgvector queries working with HNSW indices
- Vision Framework bridge extracting OCR blocks
- Apple Speech Framework transcribing audio
- WhisperX + pyannote producing speaker-aligned transcripts
- BLIP generating captions for figures
- End-to-end pipeline: file → ingest → enrich → index → retrieve
- Council receiving multimodal context with proper budgeting
- Claim extraction finding cross-modal evidence with citations
- Retrieval P99 latency < 500ms (warm)
- All 23+ tests passing

---

## Risk Mitigation

**Risk**: Vision Framework timeout blocking retriever
**Mitigation**: Circuit breaker (already implemented) + 5s timeout + fallback to vector text

**Risk**: WhisperX subprocess memory pressure
**Mitigation**: Job scheduler cap (ASR=1 concurrent) + queue backpressure

**Risk**: pgvector similarity search too slow
**Mitigation**: HNSW indices + projection → PostgreSQL planner optimizations

**Risk**: Citation extraction brittle
**Mitigation**: Store (uri, t0, t1, bbox) as structured fields, not string parsing

---

**Next**: Begin Week 1 with PostgreSQL pgvector implementation.

