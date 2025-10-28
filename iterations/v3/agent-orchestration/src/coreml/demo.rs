//! Demonstration of Core ML integration

use std::path::PathBuf;
use agent_orchestration::coreml::{CoreMLManager, CoreMLModelType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🧠 Agent Agency - Core ML Integration Demo");
    println!("==========================================");

    // Initialize Core ML manager with model path
    let model_path = std::env::var("COREML_MODELS_PATH")
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|_| PathBuf::from("/Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml"));

    println!("📁 Model path: {:?}", model_path);

    let manager = CoreMLManager::new(model_path);
    println!("🚀 ANE Available: {}", manager.is_ane_available());

    // Load available models
    println!("⏳ Loading Core ML models...");
    match manager.load_available_models().await {
        Ok(_) => println!("✅ Models loaded successfully"),
        Err(e) => {
            println!("❌ Failed to load models: {}", e);
            return Ok(());
        }
    }

    let model_count = manager.model_count().await;
    println!("📊 Loaded {} models", model_count);

    // Enumerate available models
    println!("\n🤖 Available Models:");
    for model_type in [
        CoreMLModelType::Vision,
        CoreMLModelType::Language,
        CoreMLModelType::SpeechToText,
        CoreMLModelType::ObjectDetection,
    ].iter() {
        let models = manager.get_models_by_type(*model_type).await;
        if !models.is_empty() {
            println!("  {:?} ({})", model_type, models.len());
            for model in models {
                println!("    📦 {} - Performance: {:.1}, ANE: {}",
                    model.metadata.name,
                    model.metadata.performance_score.unwrap_or(0.0),
                    model.metadata.supports_ane
                );
            }
        }
    }

    // Test inference on vision model if available
    if let Some(vision_model) = manager.get_model(CoreMLModelType::Vision, "FastViT-T8-F16").await {
        println!("\n🔍 Testing Vision Model Inference...");

        // Create mock input (simulated image data)
        let mut inputs = std::collections::HashMap::new();
        inputs.insert("input".to_string(), vec![0.5f32; 3 * 256 * 256]); // RGB image

        match manager.run_inference(&vision_model, inputs).await {
            Ok(outputs) => {
                println!("✅ Inference successful!");
                for (output_name, data) in outputs {
                    println!("  📤 {}: {} elements", output_name, data.len());
                }
            }
            Err(e) => println!("❌ Inference failed: {}", e),
        }
    } else {
        println!("\n⚠️  FastViT vision model not available for testing");
    }

    // Test inference on language model if available
    if let Some(language_model) = manager.get_model(CoreMLModelType::Language, "Mistral-7B-Instruct-FP16").await {
        println!("\n💬 Testing Language Model Inference...");

        // Create mock input (simulated token sequence)
        let mut inputs = std::collections::HashMap::new();
        inputs.insert("input_ids".to_string(), vec![1.0f32; 512]); // Token sequence

        match manager.run_inference(&language_model, inputs).await {
            Ok(outputs) => {
                println!("✅ Language inference successful!");
                for (output_name, data) in outputs {
                    println!("  📤 {}: {} elements", output_name, data.len());
                }
            }
            Err(e) => println!("❌ Language inference failed: {}", e),
        }
    } else {
        println!("\n⚠️  Mistral language model not available for testing");
    }

    println!("\n🎉 Core ML integration demo complete!");
    println!("💡 Ready for Phase 3B: Actual inference testing and ANE speedup measurement");

    Ok(())
}
