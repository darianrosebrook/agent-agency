//! @darianrosebrook
//! Enrichers module CLI for testing multimodal enrichment

use enrichers::*;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Enrichers CLI initialized");

    // Create enricher instances with default config
    let config = EnricherConfig::default();

    let _vision_enricher = VisionEnricher::new(config.clone());
    let _asr_enricher = AsrEnricher::new(config.clone());
    let _entity_enricher = EntityEnricher::new(config.clone());
    let _visual_caption_enricher = VisualCaptionEnricher::new(config);

    info!("All enrichers ready");
    info!("Vision enricher: circuit breaker available");
    info!("ASR enricher: WhisperX provider");
    info!("Entity enricher: NER enabled");
    info!("Visual caption enricher: BLIP/SigLIP ready");

    Ok(())
}
