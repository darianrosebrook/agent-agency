//! Metal/MPS-backed judge engine implementation.
//!
//! Uses Candle on Metal to run a small deterministic neural pipeline for
//! generating structured `JudgeVerdict` responses. This provides real tensor
//! compute on Apple Silicon while the CoreML/ANE path remains disabled.
//! @author @darianrosebrook

use agent_agency_contracts::engine::{
    EngineCaps, EngineError, EngineRequest, EngineResponse, JudgeEngine, TokenUsage,
};
use agent_agency_contracts::judge_io::{JudgeVerdict, VerdictLabel, Violation};
use async_trait::async_trait;
use candle_core::{Device, Tensor};
use system_acceleration::MetalExecutor;

const EMBED_DIM: usize = 128;
const HIDDEN_DIM: usize = 64;

#[derive(Debug)]
pub struct EngineMps {
    caps: EngineCaps,
    device: Device,
    w1: Tensor,
    b1: Tensor,
    w2: Tensor,
    b2: Tensor,
}

impl EngineMps {
    pub fn new(model_id: impl Into<String>) -> Result<Self, EngineError> {
        if !MetalExecutor::is_available() {
            return Err(EngineError::ModelNotAvailable {
                model_id: model_id.into(),
            });
        }

        // Build a deterministic tiny network on Metal.
        let device = MetalExecutor::new(0)
            .map_err(|e| EngineError::Internal {
                message: format!("Metal init failed: {e}"),
            })?
            .device()
            .clone();

        let seed = 42u64;
        let mut rng = deterministic_rng(seed);

        let w1 = random_tensor(&device, &[EMBED_DIM, HIDDEN_DIM], &mut rng).map_err(to_core("w1 init"))?;
        let b1 = random_tensor(&device, &[HIDDEN_DIM], &mut rng).map_err(to_core("b1 init"))?;
        let w2 = random_tensor(&device, &[HIDDEN_DIM, 1], &mut rng).map_err(to_core("w2 init"))?;
        let b2 = random_tensor(&device, &[1], &mut rng).map_err(to_core("b2 init"))?;

        let caps = EngineCaps {
            model_id: model_id.into(),
            family: "mps-heuristic".to_string(),
            max_ctx: 4096,
            max_tokens_out: 256,
            quant: "fp16".to_string(),
            acceleration: vec!["Metal".to_string(), "GPU".to_string()],
        };

        Ok(Self {
            caps,
            device,
            w1,
            b1,
            w2,
            b2,
        })
    }

    fn embed_prompt(&self, req: &EngineRequest) -> Result<Tensor, EngineError> {
        let mut buf = vec![0f32; EMBED_DIM];
        let mut i = 0;
        for byte in req.prompt.objective.as_bytes() {
            buf[i % EMBED_DIM] += (*byte as f32) / 255.0;
            i += 1;
        }
        for item in &req.prompt.rubric {
            for b in item.id.as_bytes() {
                buf[i % EMBED_DIM] += (*b as f32) / 255.0;
                i += 1;
            }
        }
        Tensor::from_slice(&buf, (1, EMBED_DIM), &self.device).map_err(to_core("embed"))
    }

    fn forward(&self, req: &EngineRequest) -> Result<f32, EngineError> {
        let x = self.embed_prompt(req)?;
        let h = x
            .matmul(&self.w1)
            .and_then(|t| t.broadcast_add(&self.b1))
            .and_then(|t| t.tanh())
            .map_err(to_core("layer1"))?;
        let out = h
            .matmul(&self.w2)
            .and_then(|t| t.broadcast_add(&self.b2))
            .map_err(to_core("layer2"))?;
        let v: Vec<f32> = out
            .flatten_all()
            .map_err(to_core("flatten"))?
            .to_vec1()
            .map_err(to_core("to_host"))?;
        let raw = v.get(0).cloned().unwrap_or(0.0);
        Ok(sigmoid(raw))
    }

    fn build_verdict(&self, req: &EngineRequest, score: f32) -> EngineResponse {
        let label = if score >= 0.6 {
            VerdictLabel::Pass
        } else {
            VerdictLabel::Fail
        };

        let violations = if label == VerdictLabel::Fail {
            vec![Violation {
                rule_id: "MPS-HEURISTIC-001".to_string(),
                severity: agent_agency_contracts::judge_io::Severity::Medium,
                waivable: true,
                description: "Heuristic score below threshold".to_string(),
            }]
        } else {
            Vec::new()
        };

        let rationale = format!(
            "MPS heuristic scored {:.3} for objective '{}'",
            score, req.prompt.objective
        );

        let verdict = JudgeVerdict {
            score,
            label,
            rationale: rationale.clone(),
            violations,
            evidence_refs: vec!["auto:mps-heuristic".to_string()],
        };

        let raw_text = serde_json::to_string(&verdict).unwrap_or_else(|_| rationale.clone());

        EngineResponse {
            raw_text,
            parsed: verdict,
            usage: TokenUsage::from_text(&req.prompt.objective),
        }
    }
}

#[async_trait]
impl JudgeEngine for EngineMps {
    async fn complete(&self, req: EngineRequest) -> Result<EngineResponse, EngineError> {
        let score = self.forward(&req)?;
        Ok(self.build_verdict(&req, score))
    }

    fn capabilities(&self) -> EngineCaps {
        self.caps.clone()
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

fn deterministic_rng(seed: u64) -> impl rand::RngCore {
    use rand::{rngs::StdRng, SeedableRng};
    StdRng::seed_from_u64(seed)
}

fn random_tensor(
    device: &Device,
    shape: &[usize],
    rng: &mut impl rand::RngCore,
) -> candle_core::Result<Tensor> {
    use rand::Rng;
    let numel: usize = shape.iter().product();
    let vals: Vec<f32> = (0..numel).map(|_| rng.gen_range(-0.25..0.25)).collect();
    Tensor::from_vec(vals, shape, device)
}

fn to_core(ctx: &'static str) -> impl Fn(candle_core::Error) -> EngineError {
    move |e| EngineError::Internal {
        message: format!("{ctx}: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_contracts::JudgeType;

    #[tokio::test]
    async fn completes_and_returns_score() {
        if !MetalExecutor::is_available() {
            // Skip gracefully on non-macOS or missing Metal.
            return;
        }
        let engine = EngineMps::new("mps-heuristic").expect("engine init");
        let req = EngineRequest {
            prompt: agent_agency_contracts::judge_io::JudgePrompt {
                role: JudgeType::Constitutional,
                objective: "Evaluate privacy posture".to_string(),
                rubric: vec![],
                evidence: agent_agency_contracts::judge_io::WorkingSpecEvidence {
                    spec_text: "Spec text".to_string(),
                    acceptance_criteria: vec![],
                    risk_tier: "medium".to_string(),
                    context: serde_json::Value::Object(serde_json::Map::new()),
                },
                output_schema: "{}".to_string(),
            },
            max_tokens: 64,
            temperature: 0.1,
            seed: Some(7),
        };

        let resp = engine.complete(req).await.expect("inference");
        assert!(resp.parsed.score >= 0.0 && resp.parsed.score <= 1.0);
    }

    #[test]
    fn capabilities_announce_metal() {
        if !MetalExecutor::is_available() {
            return;
        }
        let engine = EngineMps::new("mps-heuristic").expect("engine init");
        let caps = engine.capabilities();
        assert!(caps.acceleration.contains(&"Metal".to_string()));
    }
}

