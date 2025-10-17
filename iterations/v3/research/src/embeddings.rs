use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub dimension: usize,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct EmbeddingVector {
    pub values: Vec<f32>,
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn dimension(&self) -> usize;
    async fn embed(&self, inputs: &[String]) -> Result<Vec<EmbeddingVector>>;
}

/// Adapter to integrate embedding-service with research agent
pub struct EmbeddingServiceAdapter {
    service: Box<dyn embedding_service::EmbeddingService>,
}

impl EmbeddingServiceAdapter {
    pub fn new(service: Box<dyn embedding_service::EmbeddingService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl EmbeddingProvider for EmbeddingServiceAdapter {
    fn dimension(&self) -> usize {
        768 // embeddinggemma dimension
    }

    async fn embed(&self, inputs: &[String]) -> Result<Vec<EmbeddingVector>> {
        let mut vectors = Vec::new();

        for input in inputs {
            let embedding = self
                .service
                .generate_embedding(
                    input,
                    embedding_service::ContentType::Knowledge,
                    "research_agent",
                )
                .await?;

            vectors.push(EmbeddingVector {
                values: embedding.vector,
            });
        }

        Ok(vectors)
    }
}

/// Deterministic dummy provider for testing and plumbing.
pub struct DummyEmbeddingProvider {
    dim: usize,
    seed: u64,
}

impl DummyEmbeddingProvider {
    pub fn new(cfg: EmbeddingConfig) -> Self {
        Self {
            dim: cfg.dimension,
            seed: cfg.seed.unwrap_or(42),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for DummyEmbeddingProvider {
    fn dimension(&self) -> usize {
        self.dim
    }
    async fn embed(&self, inputs: &[String]) -> Result<Vec<EmbeddingVector>> {
        // Simple seeded hash → pseudo-random but deterministic floats in [-1,1]
        let mut out = Vec::with_capacity(inputs.len());
        for (i, text) in inputs.iter().enumerate() {
            let mut vals = Vec::with_capacity(self.dim);
            for d in 0..self.dim {
                let h = fxhash::hash64(
                    &(self.seed ^ (i as u64) ^ (d as u64) ^ fxhash::hash64(text.as_bytes())),
                );
                // map to [-1,1]
                let v = ((h as f64 % 10000.0) / 5000.0) - 1.0;
                vals.push(v as f32);
            }
            out.push(EmbeddingVector { values: vals });
        }
        Ok(out)
    }
}
