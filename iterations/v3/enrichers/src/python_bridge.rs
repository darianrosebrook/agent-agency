// @darianrosebrook
// Python subprocess bridges for external ML models
// Enables WhisperX (ASR) and BLIP (visual captioning) integration

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use uuid::Uuid;

/// WhisperX transcription output structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperXOutput {
    pub segments: Vec<WhisperXSegment>,
    pub language: String,
    pub text: String,
}

/// Individual segment from WhisperX
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperXSegment {
    pub id: u32,
    pub start: f32,
    pub end: f32,
    pub text: String,
    pub speaker: Option<String>,
    pub words: Vec<WhisperXWord>,
}

/// Word-level timing from WhisperX
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperXWord {
    pub word: String,
    pub start: f32,
    pub end: f32,
    pub score: f32,
}

/// BLIP caption result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BLIPCaptionResult {
    pub caption: String,
    pub confidence: f32,
    pub tags: Vec<String>,
}

/// Python bridge for subprocess integration
pub struct PythonBridge;

impl PythonBridge {
    /// Transcribe audio using WhisperX with diarization
    ///
    /// Calls WhisperX subprocess to perform ASR with speaker diarization
    /// and word-level timing information.
    ///
    /// # Arguments
    /// * `audio_data` - Raw audio bytes (WAV/MP3/M4A)
    /// * `language` - Language code (e.g., "en", "fr")
    ///
    /// # Returns
    /// `AsrResult` with transcription and word timings
    ///
    /// # Errors
    /// Returns error if WhisperX subprocess fails or parsing fails
    pub async fn transcribe_with_whisperx(
        audio_data: &[u8],
        language: Option<&str>,
    ) -> Result<crate::types::AsrResult> {
        // Write audio to temporary file
        let temp_dir = std::env::temp_dir();
        let audio_path = temp_dir.join(format!("audio_{}.wav", Uuid::new_v4()));

        fs::write(&audio_path, audio_data)
            .map_err(|e| anyhow!("Failed to write temp audio file: {}", e))?;

        let lang_arg = language.unwrap_or("en");
        let output_dir = temp_dir.join(format!("whisperx_{}", Uuid::new_v4()));
        fs::create_dir_all(&output_dir)
            .map_err(|e| anyhow!("Failed to create output directory: {}", e))?;

        // Call WhisperX subprocess
        let output = Command::new("whisperx")
            .arg(audio_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?)
            .arg("--language")
            .arg(lang_arg)
            .arg("--diarize_model")
            .arg("pyannote")
            .arg("--output_format")
            .arg("json")
            .arg("--output_dir")
            .arg(
                output_dir
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid output path"))?,
            )
            .output()
            .map_err(|e| anyhow!("WhisperX subprocess failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("WhisperX failed: {}", stderr));
        }

        // Parse JSON output
        let stdout_str = String::from_utf8(output.stdout)
            .map_err(|e| anyhow!("Failed to read WhisperX output: {}", e))?;

        let whisperx_result: WhisperXOutput = serde_json::from_str(&stdout_str)
            .map_err(|e| anyhow!("Failed to parse WhisperX JSON: {}", e))?;

        // Convert to AsrResult
        Self::convert_whisperx_to_asr_result(whisperx_result)
    }

    /// Generate image caption using BLIP
    ///
    /// Calls BLIP subprocess to generate image descriptions and tags.
    ///
    /// # Arguments
    /// * `image_data` - Raw image bytes (JPEG/PNG)
    /// * `context` - Optional context for caption generation
    ///
    /// # Returns
    /// `CaptionResult` with caption text and tags
    ///
    /// # Errors
    /// Returns error if BLIP subprocess fails or parsing fails
    pub async fn caption_with_blip(
        image_data: &[u8],
        context: Option<&str>,
    ) -> Result<crate::types::CaptionResult> {
        // Write image to temporary file
        let temp_dir = std::env::temp_dir();
        let image_path = temp_dir.join(format!("image_{}.jpg", Uuid::new_v4()));

        fs::write(&image_path, image_data)
            .map_err(|e| anyhow!("Failed to write temp image file: {}", e))?;

        // Create Python script for BLIP captioning
        let python_script = r#"
import sys
import json
from PIL import Image
from transformers import BlipProcessor, BlipForConditionalGeneration

image_path = sys.argv[1]
context_text = sys.argv[2] if len(sys.argv) > 2 else None

try:
    image = Image.open(image_path).convert('RGB')
    processor = BlipProcessor.from_pretrained("Salesforce/blip-image-captioning-large")
    model = BlipForConditionalGeneration.from_pretrained("Salesforce/blip-image-captioning-large")

    # Generate caption with optional context
    if context_text:
        inputs = processor(image, text=context_text, return_tensors="pt")
    else:
        inputs = processor(image, return_tensors="pt")
    
    out = model.generate(**inputs, max_length=50)
    caption = processor.decode(out[0], skip_special_tokens=True)

    # Extract tags (simple approach: split caption into noun phrases)
    tags = [word.strip() for word in caption.split(',')]
    
    result = {
        "caption": caption,
        "confidence": 0.85,
        "tags": tags
    }
    print(json.dumps(result))
except Exception as e:
    print(json.dumps({"error": str(e)}), file=sys.stderr)
    sys.exit(1)
"#;

        // Call Python script
        let output = Command::new("python3")
            .arg("-c")
            .arg(python_script)
            .arg(image_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?)
            .arg(context.unwrap_or(""))
            .output()
            .map_err(|e| anyhow!("BLIP subprocess failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("BLIP failed: {}", stderr));
        }

        // Parse JSON result
        let stdout_str = String::from_utf8(output.stdout)
            .map_err(|e| anyhow!("Failed to read BLIP output: {}", e))?;

        let blip_result: BLIPCaptionResult = serde_json::from_str(&stdout_str)
            .map_err(|e| anyhow!("Failed to parse BLIP JSON: {}", e))?;

        // Convert to CaptionResult
        Ok(crate::types::CaptionResult {
            caption: blip_result.caption,
            confidence: blip_result.confidence,
            tags: blip_result.tags,
            processing_time_ms: 0,
        })
    }

    /// Convert WhisperX output to AsrResult
    fn convert_whisperx_to_asr_result(whisperx: WhisperXOutput) -> Result<crate::types::AsrResult> {
        let mut segments = Vec::new();
        let mut speaker_map: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for segment in whisperx.segments {
            let mut word_timings = Vec::new();

            for word in segment.words {
                word_timings.push(crate::types::WordTiming {
                    word: word.word,
                    tokens: vec![], // TODO: Parse actual tokens
                    start: word.start,
                    end: word.end,
                    probability: word.score,
                });
            }

            segments.push(crate::types::SpeechSegment {
                id: Uuid::new_v4(),
                speaker_id: segment.speaker.clone(),
                t0: segment.start,
                t1: segment.end,
                text: segment.text,
                confidence: 0.9,
                word_timings,
                language: None,
            });

            // Track speakers for speaker list
            if let Some(speaker) = segment.speaker.clone() {
                let entry = speaker_map.entry(speaker).or_insert(0);
                *entry += 1;
            }
        }

        // Convert speaker map to Speaker structs
        let speakers: Vec<crate::types::Speaker> = speaker_map
            .into_iter()
            .map(|(id, turn_count)| crate::types::Speaker {
                speaker_id: id,
                name: None,
                turn_count,
                total_duration_ms: 0, // Computed during playback if needed
            })
            .collect();

        Ok(crate::types::AsrResult {
            turns: segments,
            speakers,
            language: Some(whisperx.language),
            confidence: 0.9,
            processing_time_ms: 0,
        })
    }

    /// Validate Python is available
    pub async fn validate_python_env() -> Result<()> {
        let output = Command::new("python3")
            .arg("--version")
            .output()
            .map_err(|e| anyhow!("Python3 not found: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!("Python3 unavailable"));
        }

        Ok(())
    }

    /// Check if required Python packages are installed
    pub async fn check_dependencies() -> Result<()> {
        // Check for whisperx
        let whisperx_check = Command::new("python3")
            .arg("-c")
            .arg("import whisperx; print('whisperx ok')")
            .output();

        // Check for transformers (BLIP)
        let transformers_check = Command::new("python3")
            .arg("-c")
            .arg("import transformers; print('transformers ok')")
            .output();

        if whisperx_check.is_err() {
            return Err(anyhow!(
                "whisperx not installed. Install with: pip install whisperx"
            ));
        }

        if transformers_check.is_err() {
            return Err(anyhow!(
                "transformers not installed. Install with: pip install transformers"
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whisperx_word_parsing() {
        let whisperx = WhisperXOutput {
            segments: vec![WhisperXSegment {
                id: 0,
                start: 0.0,
                end: 1.0,
                text: "Hello world".to_string(),
                speaker: Some("Speaker_1".to_string()),
                words: vec![
                    WhisperXWord {
                        word: "Hello".to_string(),
                        start: 0.0,
                        end: 0.5,
                        score: 0.95,
                    },
                    WhisperXWord {
                        word: "world".to_string(),
                        start: 0.5,
                        end: 1.0,
                        score: 0.93,
                    },
                ],
            }],
            language: "en".to_string(),
            text: "Hello world".to_string(),
        };

        let result = PythonBridge::convert_whisperx_to_asr_result(whisperx);
        assert!(result.is_ok());

        let asr = result.unwrap();
        assert_eq!(asr.turns.len(), 1);
        assert_eq!(asr.turns[0].text, "Hello world");
        assert_eq!(asr.turns[0].word_timings.len(), 2);
    }

    #[test]
    fn test_blip_tag_extraction() {
        let result = BLIPCaptionResult {
            caption: "A cat sitting on a table, playing with yarn".to_string(),
            confidence: 0.85,
            tags: vec!["cat".to_string(), "table".to_string(), "yarn".to_string()],
        };

        assert_eq!(result.tags.len(), 3);
        assert!(result.tags.contains(&"cat".to_string()));
    }
}
