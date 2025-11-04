# Phase 3: Implementation Guide - Bridges & Integration

**Status**: Ready to implement  
**Priority**: PostgreSQL pgvector → Swift bridges → Python bridges → System integration

---

## Week 1: PostgreSQL pgvector Integration

### Step 1.1: Enable pgvector Extension

Before running any migrations, ensure pgvector is installed in your PostgreSQL:

```bash
# On macOS with Homebrew PostgreSQL
brew install postgresql
createdb your_db
psql your_db -c "CREATE EXTENSION IF NOT EXISTS vector"

# Verify installation
psql your_db -c "SELECT extversion FROM pg_extension WHERE extname = 'vector'"
```

### Step 1.2: Create HNSW Indices

Add to `iterations/v3/database/migrations/006_multimodal_rag_schema.sql`:

```sql
-- Create HNSW indices for each active embedding model
-- These must be created AFTER the extension is enabled

-- e5-small-v2 text embeddings (1536-dim, cosine distance)
CREATE INDEX idx_block_vectors_e5_small_v2_hnsw ON block_vectors 
USING hnsw (vec vector_cosine_ops)
WHERE model_id = 'e5-small-v2'
WITH (m = 16, ef_construction = 200);

-- clip-vit-b32 visual embeddings (512-dim, inner product)
CREATE INDEX idx_block_vectors_clip_vit_b32_hnsw ON block_vectors
USING hnsw (vec vector_ip_ops)
WHERE model_id = 'clip-vit-b32'
WITH (m = 16, ef_construction = 200);

-- Create BM25 index for full-text search (when using Tantivy)
-- Note: For now, keeping as placeholder for Tantivy integration
```

### Step 1.3: Implement VectorStore Methods in `indexers/src/database.rs`

Replace the placeholder TODOs with actual pgvector SQL:

```rust
#[async_trait]
impl VectorStore for PostgresVectorStore {
    async fn store_vector(&self, record: BlockVectorRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO block_vectors (block_id, model_id, modality, vec)
            VALUES ($1, $2, $3, $4::vector)
            ON CONFLICT (block_id, model_id) 
            DO UPDATE SET vec = EXCLUDED.vec
            WHERE EXCLUDED.vec IS DISTINCT FROM block_vectors.vec
            "#
        )
        .bind(record.block_id)
        .bind(&record.model_id)
        .bind(&record.modality)
        .bind(&format!("[{}]", record.vector.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",")))
        .execute(&self.pool)
        .await?;

        debug!("Stored vector for block {} with model {}", record.block_id, record.model_id);
        Ok(())
    }

    async fn search_similar(
        &self,
        query_vector: &[f32],
        model_id: &str,
        k: usize,
        project_scope: Option<&str>,
    ) -> Result<Vec<(Uuid, f32)>> {
        // Get the metric for this model to choose the right operator
        let model = sqlx::query_as::<_, (String,)>(
            "SELECT metric FROM embedding_models WHERE id = $1"
        )
        .bind(model_id)
        .fetch_one(&self.pool)
        .await?;

        let operator = match model.0.as_str() {
            "cosine" => "<=>",  // Returns distance (lower is better)
            "ip" => "<#>",      // Inner product
            "l2" => "<->",      // L2 distance
            _ => "<=>"
        };

        let query_str = format!(
            r#"
            SELECT bv.block_id, (bv.vec {} $1::vector) AS similarity
            FROM block_vectors bv
            INNER JOIN blocks b ON bv.block_id = b.id
            INNER JOIN segments s ON b.segment_id = s.id
            WHERE bv.model_id = $2
            AND (s.project_scope IS NULL OR s.project_scope = $3)
            ORDER BY similarity ASC
            LIMIT $4
            "#,
            operator
        );

        let vec_str = format!("[{}]", query_vector.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(","));

        let results: Vec<(Uuid, f32)> = sqlx::query_as(&query_str)
            .bind(&vec_str)
            .bind(model_id)
            .bind(project_scope)
            .bind(k as i64)
            .fetch_all(&self.pool)
            .await?;

        debug!("Found {} similar vectors for model {} (k={})", results.len(), model_id, k);
        Ok(results)
    }

    async fn log_search(&self, entry: SearchAuditEntry) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO search_logs (query, results, features, created_at)
            VALUES ($1, $2::jsonb, $3::jsonb, NOW())
            "#
        )
        .bind(&entry.query)
        .bind(serde_json::to_string(&entry.results)?)
        .bind(serde_json::to_string(&entry.features)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_search_logs(&self, limit: usize) -> Result<Vec<SearchAuditEntry>> {
        let logs = sqlx::query_as::<_, (String, String, String)>(
            r#"
            SELECT query, results::text, features::text
            FROM search_logs
            ORDER BY created_at DESC
            LIMIT $1
            "#
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let entries = logs
            .into_iter()
            .map(|(query, results_str, features_str)| SearchAuditEntry {
                query,
                results: serde_json::from_str(&results_str).unwrap_or_default(),
                features: serde_json::from_str(&features_str).unwrap_or_default(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
            .collect();

        Ok(entries)
    }
}
```

### Step 1.4: Test pgvector Integration

Add integration test in `iterations/v3/integration-tests/tests/pgvector_integration.rs`:

```rust
#[tokio::test]
#[ignore] // Run with `cargo test -- --ignored` when PostgreSQL is available
async fn test_pgvector_similarity_search() -> Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/multimodal_rag_test".to_string());
    
    let db = DatabasePool::new(&db_url, 5).await?;
    let store = PostgresVectorStore::from_pool(&db);

    // Create test vectors
    let v1 = BlockVectorRecord {
        block_id: Uuid::new_v4(),
        model_id: "e5-small-v2".to_string(),
        modality: "text".to_string(),
        vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
    };

    let v2 = BlockVectorRecord {
        block_id: Uuid::new_v4(),
        model_id: "e5-small-v2".to_string(),
        modality: "text".to_string(),
        vector: vec![0.11, 0.21, 0.31, 0.41, 0.51], // Similar to v1
    };

    // Store vectors
    store.store_vector(v1.clone()).await?;
    store.store_vector(v2.clone()).await?;

    // Search
    let query = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let results = store.search_similar(&query, "e5-small-v2", 2, None).await?;

    assert!(!results.is_empty());
    assert_eq!(results[0].0, v1.block_id);
    println!("pgvector similarity search working!");

    Ok(())
}
```

---

## Week 2: Swift Bridges (Vision & Speech)

### Step 2.1: Create Rust-Swift FFI Bridge

Create `iterations/v3/apple-silicon/src/vision_bridge.rs`:

```rust
use anyhow::{anyhow, Result};
use std::ffi::{CStr, CString};
use serde::{Deserialize, Serialize};

#[repr(C)]
pub struct VisionBlockC {
    text: *const libc::c_char,
    role: *const libc::c_char,
    confidence: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[link(name = "Vision", kind = "framework")]
#[link(name = "Foundation", kind = "framework")]
extern "C" {
    fn analyze_document_ocr(
        image_ptr: *const u8,
        image_len: usize,
        timeout_ms: u64,
    ) -> *mut VisionBlockC;

    fn free_vision_blocks(blocks: *mut VisionBlockC, count: usize);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionBlock {
    pub text: String,
    pub role: String,
    pub confidence: f32,
    pub bbox: (f32, f32, f32, f32), // x, y, w, h
}

pub async fn analyze_document(
    image_data: &[u8],
    timeout_ms: u64,
) -> Result<Vec<VisionBlock>> {
    unsafe {
        let mut count: u32 = 0;
        let blocks_ptr = analyze_document_ocr(
            image_data.as_ptr(),
            image_data.len(),
            timeout_ms,
        );

        if blocks_ptr.is_null() {
            return Err(anyhow!("Vision analysis failed"));
        }

        let mut results = Vec::new();
        let mut current = blocks_ptr;

        // Read blocks (assuming null-terminated array)
        while !(*current).text.is_null() {
            let text = CStr::from_ptr((*current).text)
                .to_string_lossy()
                .to_string();
            let role = CStr::from_ptr((*current).role)
                .to_string_lossy()
                .to_string();

            results.push(VisionBlock {
                text,
                role,
                confidence: (*current).confidence,
                bbox: ((*current).x, (*current).y, (*current).w, (*current).h),
            });

            current = current.add(1);
        }

        // Free C memory
        free_vision_blocks(blocks_ptr, results.len());

        Ok(results)
    }
}
```

### Step 2.2: Implement Swift Vision Bridge

Create `iterations/v3/apple-silicon/Sources/VisionBridge.swift`:

```swift
import Foundation
import Vision
import CoreGraphics

@_cdecl("analyze_document_ocr")
func analyzeDocumentOCR(
    imagePtr: UnsafeRawPointer,
    imageLen: Int,
    timeoutMs: UInt64
) -> UnsafeMutablePointer<VisionBlockC>? {
    autoreleasepool {
        let imageData = Data(bytes: imagePtr, count: imageLen)
        guard let cgImage = UIImage(data: imageData)?.cgImage else {
            return nil
        }

        var blocks: [VisionBlockC] = []
        let semaphore = DispatchSemaphore(value: 0)
        
        let request = VNRecognizeTextRequest { request, error in
            if let results = request.results as? [VNRecognizedTextObservation] {
                for obs in results {
                    let text = obs.topCandidates(1).first?.string ?? ""
                    let bbox = obs.boundingBox
                    
                    let block = VisionBlockC(
                        text: strdup(text),
                        role: strdup(self.classifyRole(text)),
                        confidence: Float(obs.confidence),
                        x: Float(bbox.origin.x),
                        y: Float(bbox.origin.y),
                        w: Float(bbox.width),
                        h: Float(bbox.height)
                    )
                    blocks.append(block)
                }
            }
            semaphore.signal()
        }
        
        request.recognitionLevel = .accurate
        request.usesLanguageCorrection = true
        
        let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
        try? handler.perform([request])
        
        // Wait for completion or timeout
        let deadline = DispatchTime.now() + .milliseconds(Int(timeoutMs))
        let _ = semaphore.wait(timeout: deadline)
        
        // Null-terminate the array
        blocks.append(VisionBlockC(
            text: nil,
            role: nil,
            confidence: 0,
            x: 0, y: 0, w: 0, h: 0
        ))
        
        // Allocate C memory for result
        let ptr = UnsafeMutablePointer<VisionBlockC>.allocate(capacity: blocks.count)
        ptr.initialize(from: blocks, count: blocks.count)
        
        return ptr
    }
}

@_cdecl("free_vision_blocks")
func freeVisionBlocks(blocks: UnsafeMutablePointer<VisionBlockC>?, count: Int) {
    guard let blocks = blocks else { return }
    
    for i in 0..<count {
        if blocks[i].text != nil {
            free(blocks[i].text)
        }
        if blocks[i].role != nil {
            free(blocks[i].role)
        }
    }
    
    blocks.deallocate()
}

private func classifyRole(_ text: String) -> String {
    if text.contains("```") || text.contains("import") { return "code" }
    if text.count > 100 { return "paragraph" }
    if text.starts(with: "#") { return "title" }
    return "bullet"
}

// C struct definition (matches Rust)
public struct VisionBlockC {
    public var text: UnsafeMutablePointer<CChar>?
    public var role: UnsafeMutablePointer<CChar>?
    public var confidence: Float
    public var x: Float
    public var y: Float
    public var w: Float
    public var h: Float
}
```

---

## Week 3: Python Bridges (ASR & Visual Captioning)

### Step 3.1: WhisperX Integration

Add to `enrichers/src/asr_enricher.rs`:

```rust
pub async fn transcribe_with_whisperx_bridge(
    audio_data: &[u8],
    language: Option<&str>,
) -> Result<AsrResult> {
    // Write audio to temp file
    let temp_dir = std::env::temp_dir();
    let temp_audio = temp_dir.join(format!("audio_{}.wav", uuid::Uuid::new_v4()));
    
    std::fs::write(&temp_audio, audio_data)
        .context("Failed to write temp audio")?;

    // Call WhisperX subprocess
    let lang = language.unwrap_or("en");
    let output = std::process::Command::new("whisperx")
        .arg(temp_audio.to_str().unwrap())
        .arg("--language")
        .arg(lang)
        .arg("--diarize_model")
        .arg("pyannote")
        .arg("--output_format")
        .arg("json")
        .arg("--output_dir")
        .arg(temp_dir.to_str().unwrap())
        .output()
        .context("Failed to run WhisperX")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("WhisperX failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: WhisperXOutput = serde_json::from_str(&stdout)
        .context("Failed to parse WhisperX output")?;

    // Convert to AsrResult
    let mut turns = Vec::new();
    let mut speakers = std::collections::HashMap::new();

    for segment in result.segments {
        let speaker = segment.speaker.clone().unwrap_or_else(|| "SPEAKER_00".to_string());
        
        speakers.entry(speaker.clone())
            .or_insert_with(|| Speaker {
                speaker_id: speaker.clone(),
                name: None,
                turn_count: 0,
                total_duration_ms: 0,
            })
            .turn_count += 1;

        let word_timings = segment.words.into_iter().map(|w| WordTiming {
            t0: w.start,
            t1: w.end,
            token: w.word,
            confidence: w.score,
        }).collect();

        turns.push(SpeechSegment {
            id: Uuid::new_v4(),
            speaker_id: Some(speaker),
            t0: segment.start,
            t1: segment.end,
            text: segment.text,
            confidence: 0.9,
            word_timings,
        });
    }

    // Cleanup temp file
    let _ = std::fs::remove_file(&temp_audio);

    Ok(AsrResult {
        turns,
        speakers: speakers.into_values().collect(),
        language: Some(lang.to_string()),
        confidence: 0.9,
        processing_time_ms: 0,
    })
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

---

## Week 4: System Integration

### Step 4.1: Council Integration

Create `council/src/multimodal_context_provider.rs`:

```rust
use async_trait::async_trait;
use research::MultimodalRetriever;
use indexers::JobScheduler;

pub struct MultimodalContextProvider {
    retriever: Arc<MultimodalRetriever>,
    scheduler: Arc<JobScheduler>,
}

#[async_trait]
pub trait ContextProvider {
    async fn gather_context(
        &self,
        topic: &str,
        budget: ContextBudget,
    ) -> Result<Vec<ContextBlock>>;
}

#[derive(Clone)]
pub struct ContextBudget {
    pub max_tokens: usize,
    pub k: usize,
    pub diversity: bool,
    pub time_window: Option<(f32, f32)>,
}

#[derive(Clone)]
pub struct ContextBlock {
    pub text: String,
    pub confidence: f32,
    pub citation: Option<String>,
    pub modality: String,
}

#[async_trait]
impl ContextProvider for MultimodalContextProvider {
    async fn gather_context(
        &self,
        topic: &str,
        budget: ContextBudget,
    ) -> Result<Vec<ContextBlock>> {
        // Check job scheduler for capacity
        if !self.scheduler.try_acquire(JobType::Embedding)? {
            return Err(anyhow!("Embedding queue full - try again later"));
        }

        // Build multimodal query
        let query = MultimodalQuery {
            text: Some(topic.to_string()),
            query_type: QueryType::Hybrid,
            project_scope: None,
            max_results: budget.k,
        };

        // Execute search
        let mut results = self.retriever.search(&query).await?;

        // Apply token budgeting
        let mut total_tokens = 0;
        let mut context_blocks = Vec::new();

        for result in results {
            let tokens = result.snippet.split_whitespace().count();
            if total_tokens + tokens > budget.max_tokens {
                break;
            }

            total_tokens += tokens;
            context_blocks.push(ContextBlock {
                text: result.snippet,
                confidence: result.feature.fused_score,
                citation: result.citation,
                modality: format!("{:?}", result.kind),
            });
        }

        // Release scheduler slot
        self.scheduler.release(JobType::Embedding, true);

        Ok(context_blocks)
    }
}
```

---

## Week 5: Performance & Polish

### Step 5.1: Add Monitoring

Create `iterations/v3/monitoring/src/rag_metrics.rs`:

```rust
use prometheus::{Counter, Histogram};

pub struct RAGMetrics {
    pub ingest_duration: Histogram,
    pub enrich_duration: Histogram,
    pub index_duration: Histogram,
    pub search_duration: Histogram,
    pub ingest_errors: Counter,
    pub enrich_errors: Counter,
}

impl RAGMetrics {
    pub fn new() -> Self {
        Self {
            ingest_duration: Histogram::new("rag_ingest_seconds", "Time to ingest media").unwrap(),
            enrich_duration: Histogram::new("rag_enrich_seconds", "Time to enrich content").unwrap(),
            index_duration: Histogram::new("rag_index_seconds", "Time to index vectors").unwrap(),
            search_duration: Histogram::new("rag_search_seconds", "Time for multimodal search").unwrap(),
            ingest_errors: Counter::new("rag_ingest_errors_total", "Total ingest errors").unwrap(),
            enrich_errors: Counter::new("rag_enrich_errors_total", "Total enrichment errors").unwrap(),
        }
    }
}
```

---

## Success Criteria for Phase 3

- PostgreSQL pgvector queries returning results < 100ms (warm)
- Vision Framework extracting text blocks from images
- WhisperX producing speaker-aligned transcripts
- Council requesting multimodal context with budgets
- Claim extraction finding cross-modal evidence
- End-to-end: file watch → ingest → enrich → index → retrieve working
- All 23+ tests passing + new integration tests

---

## Timeline Summary

| Week | Focus | Tests | Status |
|------|-------|-------|--------|
| 1 | PostgreSQL pgvector | 3+ | Priority |
| 2 | Swift bridges | 4+ | High |
| 3 | Python bridges | 2+ | High |
| 4 | System integration | 5+ | Medium |
| 5 | Performance & polish | Refine | Final |

---

**Next Action**: Start Week 1 with PostgreSQL pgvector implementation and testing.

