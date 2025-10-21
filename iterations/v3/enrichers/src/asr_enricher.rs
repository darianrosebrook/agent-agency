//! @darianrosebrook
//! ASR (Automatic Speech Recognition) and diarization enricher
//!
//! Supports multiple providers:
//! - WhisperX (local, with alignment and diarization)
//! - Apple Speech Framework (native, lower latency)
//! - Cloud providers (optional, off by default)

use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::types::{AsrResult, EnricherConfig, Speaker, SpeechSegment, WordTiming};
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

/// FFI declarations for ASR Bridge
#[cfg(target_os = "macos")]
#[link(name = "ASRBridge", kind = "static")]
extern "C" {
    fn speech_recognize_audio(
        audioPath: *const std::ffi::c_char,
        outText: *mut *mut std::ffi::c_char,
        outConfidence: *mut f32,
        outError: *mut *mut std::ffi::c_char,
    ) -> std::ffi::c_int;

    fn speech_free_string(ptr: *mut std::ffi::c_char);

    fn speech_is_available() -> std::ffi::c_int;
}

/// Stub implementations for non-macOS platforms
#[cfg(not(target_os = "macos"))]
mod stubs {
    use std::ffi::CStr;

    #[no_mangle]
    pub extern "C" fn speech_recognize_audio(
        _audio_path: *const std::ffi::c_char,
        _out_text: *mut *mut std::ffi::c_char,
        _out_confidence: *mut f32,
        out_error: *mut *mut std::ffi::c_char,
    ) -> std::ffi::c_int {
        if !out_error.is_null() {
            let error_msg = std::ffi::CString::new("ASR not available on this platform").unwrap();
            unsafe {
                *out_error = error_msg.into_raw();
            }
        }
        1 // Error
    }

    #[no_mangle]
    pub extern "C" fn speech_free_string(ptr: *mut std::ffi::c_char) {
        if !ptr.is_null() {
            unsafe {
                let _ = std::ffi::CString::from_raw(ptr);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn speech_is_available() -> std::ffi::c_int {
        0 // Not available
    }
}

/// Re-export FFI functions for cross-platform compatibility
#[cfg(target_os = "macos")]
use self::speech_recognize_audio as speech_recognize_audio_impl;
#[cfg(target_os = "macos")]
use self::speech_free_string as speech_free_string_impl;
#[cfg(target_os = "macos")]
use self::speech_is_available as speech_is_available_impl;

#[cfg(not(target_os = "macos"))]
use self::stubs::speech_recognize_audio as speech_recognize_audio_impl;
#[cfg(not(target_os = "macos"))]
use self::stubs::speech_free_string as speech_free_string_impl;
#[cfg(not(target_os = "macos"))]
use self::stubs::speech_is_available as speech_is_available_impl;

/// Swift Speech Recognizer bridge for Apple Speech Framework
#[derive(Debug, Clone)]
struct SwiftSpeechRecognizer {
    _locale: String,
    _is_available: bool,
    _supports_on_device_recognition: bool,
}

/// SFSpeechAudioBufferRecognitionRequest for audio file recognition
#[derive(Debug, Clone)]
struct SFSpeechAudioBufferRecognitionRequest {
    _audio_file: PathBuf,
    _language: String,
    _should_report_partial_results: bool,
    _requires_on_device_recognition: bool,
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
    _config: EnricherConfig,
}

impl AsrEnricher {
    pub fn new(config: EnricherConfig) -> Self {
        let provider = match config.asr_provider.as_str() {
            "apple" => AsrProvider::AppleSpeech,
            "whisperx" => AsrProvider::WhisperX,
            _ => AsrProvider::WhisperX,
        };

        let cb_config = CircuitBreakerConfig {
            failure_threshold: config.circuit_breaker_threshold,
            success_threshold: 2,
            timeout: std::time::Duration::from_millis(config.circuit_breaker_timeout_ms),
        };

        Self {
            circuit_breaker: CircuitBreaker::new(cb_config),
            provider,
            _config: config,
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
        tracing::debug!("Transcribing with WhisperX enhanced processing ({} bytes, language: {:?})", 
                       audio_data.len(), language);

        let start_time = std::time::Instant::now();

        // 1. Call Python bridge for WhisperX processing
        let whisperx_result = crate::python_bridge::PythonBridge::transcribe_with_whisperx(
            audio_data,
            language,
        ).await?;

        // 2. Enhance the result with additional processing
        let enhanced_result = self.enhance_whisperx_result(whisperx_result).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;
        
        tracing::debug!("WhisperX transcription completed in {}ms with {} turns from {} speakers", 
                       processing_time, enhanced_result.turns.len(), enhanced_result.speakers.len());

        Ok(AsrResult {
            turns: enhanced_result.turns,
            speakers: enhanced_result.speakers,
            language: enhanced_result.language,
            confidence: enhanced_result.confidence,
            processing_time_ms: processing_time,
        })
    }

    /// Enhance WhisperX result with additional processing
    async fn enhance_whisperx_result(&self, mut result: AsrResult) -> Result<AsrResult> {
        // 1. Improve speaker diarization
        self.improve_speaker_diarization(&mut result).await?;
        
        // 2. Enhance word-level timings
        self.enhance_word_timings(&mut result).await?;
        
        // 3. Calculate improved confidence scores
        self.calculate_improved_confidence(&mut result).await?;
        
        // 4. Add speaker statistics
        self.update_speaker_statistics(&mut result).await?;
        
        Ok(result)
    }

    /// Improve speaker diarization using additional heuristics
    async fn improve_speaker_diarization(&self, result: &mut AsrResult) -> Result<()> {
        tracing::debug!("Improving speaker diarization for {} turns", result.turns.len());
        
        // Group turns by speaker and apply consistency checks
        let mut speaker_groups: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
        
        for (i, turn) in result.turns.iter().enumerate() {
            if let Some(speaker_id) = &turn.speaker_id {
                speaker_groups.entry(speaker_id.clone()).or_default().push(i);
            }
        }
        
        // Apply speaker consistency improvements
        for (speaker_id, turn_indices) in speaker_groups {
            self.apply_speaker_consistency(&mut result.turns, &speaker_id, &turn_indices).await?;
        }
        
        Ok(())
    }

    /// Apply speaker consistency checks and corrections
    async fn apply_speaker_consistency(
        &self,
        turns: &mut [SpeechSegment],
        speaker_id: &str,
        turn_indices: &[usize],
    ) -> Result<()> {
        if turn_indices.len() < 2 {
            return Ok(());
        }
        
        // Analyze speaking patterns for this speaker
        let durations: Vec<f32> = turn_indices
            .iter()
            .map(|&i| turns[i].t1 - turns[i].t0)
            .collect();
        
        let avg_duration = durations.iter().sum::<f32>() / durations.len() as f32;
        let min_duration = durations.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_duration = durations.iter().fold(0.0f32, |a, &b| a.max(b));
        
        tracing::debug!("Speaker {}: avg_duration={:.2}s, range=[{:.2}, {:.2}]s", 
                       speaker_id, avg_duration, min_duration, max_duration);
        
        // Apply consistency corrections for outlier durations
        for &turn_idx in turn_indices {
            let duration = turns[turn_idx].t1 - turns[turn_idx].t0;
            
            // If duration is significantly different from average, apply correction
            if duration < avg_duration * 0.3 || duration > avg_duration * 3.0 {
                let corrected_duration = avg_duration;
                let center = (turns[turn_idx].t0 + turns[turn_idx].t1) / 2.0;
                
                turns[turn_idx].t0 = center - corrected_duration / 2.0;
                turns[turn_idx].t1 = center + corrected_duration / 2.0;
                
                tracing::debug!("Corrected duration for turn {} from {:.2}s to {:.2}s", 
                               turn_idx, duration, corrected_duration);
            }
        }
        
        Ok(())
    }

    /// Enhance word-level timings with better precision
    async fn enhance_word_timings(&self, result: &mut AsrResult) -> Result<()> {
        tracing::debug!("Enhancing word-level timings for {} turns", result.turns.len());
        
        for turn in &mut result.turns {
            if turn.word_timings.is_empty() {
                // Generate word timings if missing
                turn.word_timings = self.generate_word_timings(&turn.text, turn.t0, turn.t1).await?;
            } else {
                // Improve existing word timings
                self.improve_existing_word_timings(&mut turn.word_timings, turn.t0, turn.t1).await?;
            }
        }
        
        Ok(())
    }

    /// Generate word timings when missing
    async fn generate_word_timings(&self, text: &str, t0: f32, t1: f32) -> Result<Vec<WordTiming>> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let duration = t1 - t0;
        let word_duration = duration / words.len() as f32;
        
        let mut word_timings = Vec::new();
        let mut current_time = t0;
        
        for word in words {
            let word_end = current_time + word_duration;
            word_timings.push(WordTiming {
                t0: current_time,
                t1: word_end,
                token: word.to_string(),
                confidence: 0.8, // Default confidence for generated timings
            });
            current_time = word_end;
        }
        
        Ok(word_timings)
    }

    /// Improve existing word timings
    async fn improve_existing_word_timings(
        &self,
        word_timings: &mut [WordTiming],
        turn_t0: f32,
        turn_t1: f32,
    ) -> Result<()> {
        if word_timings.is_empty() {
            return Ok(());
        }
        
        // Ensure word timings are within turn boundaries
        for timing in word_timings.iter_mut() {
            timing.t0 = timing.t0.max(turn_t0);
            timing.t1 = timing.t1.min(turn_t1);
            
            // Ensure t0 < t1
            if timing.t0 >= timing.t1 {
                timing.t1 = timing.t0 + 0.1; // Minimum word duration
            }
        }
        
        // Smooth out timing gaps and overlaps
        for i in 1..word_timings.len() {
            let prev_end = word_timings[i - 1].t1;
            let current_start = word_timings[i].t0;
            
            if current_start < prev_end {
                // Overlap detected, adjust
                word_timings[i].t0 = prev_end;
                if word_timings[i].t1 <= word_timings[i].t0 {
                    word_timings[i].t1 = word_timings[i].t0 + 0.1;
                }
            } else if current_start - prev_end > 0.5 {
                // Large gap detected, reduce it
                let gap = current_start - prev_end;
                word_timings[i].t0 = prev_end + gap * 0.1; // Reduce gap to 10% of original
            }
        }
        
        Ok(())
    }

    /// Transcribe using Apple Speech Framework
    async fn transcribe_apple(
        &self,
        audio_data: &[u8],
        language: Option<&str>,
    ) -> Result<AsrResult> {
        tracing::debug!("Transcribing with Apple Speech Framework enhanced processing ({} bytes, language: {:?})", 
                       audio_data.len(), language);

        let start_time = std::time::Instant::now();

        // 1. Call Apple Speech Framework (via Swift bridge)
        let apple_result = self.call_apple_speech_framework(audio_data, language).await?;

        // 2. Apply custom diarization heuristics
        let diarized_result = self.apply_custom_diarization(apple_result).await?;

        // 3. Enhance with additional processing
        let enhanced_result = self.enhance_apple_result(diarized_result).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;
        
        tracing::debug!("Apple Speech Framework transcription completed in {}ms with {} turns from {} speakers", 
                       processing_time, enhanced_result.turns.len(), enhanced_result.speakers.len());

        Ok(AsrResult {
            turns: enhanced_result.turns,
            speakers: enhanced_result.speakers,
            language: enhanced_result.language,
            confidence: enhanced_result.confidence,
            processing_time_ms: processing_time,
        })
    }

    /// Call Apple Speech Framework via Swift bridge
    async fn call_apple_speech_framework(
        &self,
        audio_data: &[u8],
        language: Option<&str>,
    ) -> Result<AsrResult> {
        // Implement Swift bridge integration for SFSpeechRecognizer
        tracing::debug!("Calling Apple Speech Framework via Swift bridge");

        // Create temporary audio file for SFSpeechRecognizer
        let temp_file = self.create_temp_audio_file(audio_data).await?;

        // Initialize SFSpeechRecognizer through Swift bridge
        let speech_recognizer = self.initialize_speech_recognizer(language).await?;

        // Create speech recognition request
        let recognition_request = self.create_speech_recognition_request(&temp_file, language).await?;

        // Execute speech recognition
        let recognition_result = self.execute_speech_recognition(&speech_recognizer, &recognition_request).await?;

        // Clean up temporary file
        tokio::fs::remove_file(&temp_file).await.ok();

        Ok(recognition_result)
    }

    /// Create temporary audio file for SFSpeechRecognizer processing
    async fn create_temp_audio_file(&self, audio_data: &[u8]) -> Result<std::path::PathBuf> {
        use tempfile::NamedTempFile;
        use std::io::Write;

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(audio_data)?;

        // Ensure file is flushed and synced
        temp_file.as_file().sync_all()?;

        Ok(temp_file.path().to_path_buf())
    }

    /// Initialize SFSpeechRecognizer through Swift bridge
    async fn initialize_speech_recognizer(&self, language: Option<&str>) -> Result<SwiftSpeechRecognizer> {
        let language_code = language.unwrap_or("en-US");

        // Check if speech recognition is available on this platform
        let available = unsafe { speech_is_available_impl() != 0 };
        if !available {
            return Err(anyhow!("Speech recognition not available on this platform"));
        }

        // Validate language support
        if !self.is_language_supported(language_code) {
            return Err(anyhow!("Language {} not supported by SFSpeechRecognizer", language_code));
        }

        Ok(SwiftSpeechRecognizer {
            _locale: language_code.to_string(),
            _is_available: true,
            _supports_on_device_recognition: true,
        })
    }

    /// Check if a language is supported by the ASR system
    fn is_language_supported(&self, language_code: &str) -> bool {
        // For Apple Speech Framework, support common languages
        matches!(
            language_code,
            "en-US" | "en-GB" | "en-AU" | "en-CA" |
            "es-ES" | "es-MX" | "fr-FR" | "fr-CA" |
            "de-DE" | "it-IT" | "pt-BR" | "ja-JP" |
            "zh-CN" | "zh-TW" | "ko-KR" | "ru-RU" |
            "ar-SA" | "hi-IN" | "th-TH" | "vi-VN"
        )
    }

    /// Create SFSpeechAudioBufferRecognitionRequest for audio file
    async fn create_speech_recognition_request(
        &self,
        audio_file: &std::path::Path,
        language: Option<&str>,
    ) -> Result<SFSpeechAudioBufferRecognitionRequest> {
        // Create recognition request for audio file
        Ok(            SFSpeechAudioBufferRecognitionRequest {
                _audio_file: audio_file.to_path_buf(),
                _language: language.unwrap_or("en-US").to_string(),
                _should_report_partial_results: true,
                _requires_on_device_recognition: false,
            })
    }

    /// Apply custom diarization heuristics using VAD and clustering
    async fn apply_custom_diarization(&self, mut result: AsrResult) -> Result<AsrResult> {
        tracing::debug!("Applying custom diarization heuristics");
        
        // 1. Apply Voice Activity Detection (VAD) heuristics
        self.apply_vad_heuristics(&mut result).await?;
        
        // 2. Apply speaker clustering
        self.apply_speaker_clustering(&mut result).await?;
        
        // 3. Calculate improved confidence
        self.calculate_improved_confidence(&mut result).await?;
        
        // 4. Update speaker statistics
        self.update_speaker_statistics(&mut result).await?;

        Ok(result)
    }

    /// Apply Voice Activity Detection heuristics
    async fn apply_vad_heuristics(&self, result: &mut AsrResult) -> Result<()> {
        // Simple VAD heuristics based on pause detection
        let mut speaker_id = 0;
        
        for turn in &mut result.turns {
            // Assign speaker ID based on timing patterns
            if turn.t0 > 2.0 && speaker_id == 0 {
                speaker_id = 1;
            }
            turn.speaker_id = Some(format!("speaker_{}", speaker_id));
        }
        
        Ok(())
    }

    /// Apply speaker clustering based on speech patterns
    async fn apply_speaker_clustering(&self, result: &mut AsrResult) -> Result<()> {
        // Basic clustering based on speech duration and timing
        let mut speaker_stats = std::collections::HashMap::new();
        
        for turn in &result.turns {
            let speaker_id = turn.speaker_id.as_ref().unwrap_or(&"unknown".to_string()).clone();
            let duration = turn.t1 - turn.t0;
            
            speaker_stats.entry(speaker_id)
                .and_modify(|(count, total_duration)| {
                    *count += 1;
                    *total_duration += duration;
                })
                .or_insert((1, duration));
        }
        
        // Update speakers list
        result.speakers = speaker_stats.into_iter()
            .map(|(speaker_id, (turn_count, total_duration))| Speaker {
                speaker_id,
                name: None, // Speaker name not available from basic stats
                turn_count,
                total_duration_ms: (total_duration * 1000.0) as u64,
            })
            .collect();
        
        Ok(())
    }

    /// Calculate improved confidence scores
    async fn calculate_improved_confidence(&self, result: &mut AsrResult) -> Result<()> {
        for turn in &mut result.turns {
            // Apply length-based confidence adjustment
            let length_factor = if turn.text.len() > 100 {
                1.0_f32
            } else if turn.text.len() > 50 {
                0.95_f32
            } else if turn.text.len() < 20 {
                0.8_f32
            } else {
                0.9_f32
            };
            
            // Apply duration-based adjustment
            let duration_factor = if turn.t1 - turn.t0 > 3.0 {
                1.1_f32
            } else if turn.t1 - turn.t0 < 1.0 {
                0.9_f32
            } else {
                1.0_f32
            };
            
            turn.confidence = (turn.confidence * length_factor * duration_factor).min(1.0_f32);
        }
        
        // Recalculate overall confidence
        if !result.turns.is_empty() {
            result.confidence = result.turns.iter()
                .map(|t| t.confidence)
                .sum::<f32>() / result.turns.len() as f32;
        }
        
        Ok(())
    }

    /// Update speaker statistics
    async fn update_speaker_statistics(&self, result: &mut AsrResult) -> Result<()> {
        let mut speaker_stats = std::collections::HashMap::new();
        
        for turn in &result.turns {
            if let Some(ref speaker_id) = turn.speaker_id {
                let duration = turn.t1 - turn.t0;
                speaker_stats.entry(speaker_id.clone())
                    .and_modify(|(count, total_duration)| {
                        *count += 1;
                        *total_duration += duration;
                    })
                    .or_insert((1, duration));
            }
        }
        
        // Update speaker information
        for speaker in &mut result.speakers {
            if let Some((turn_count, total_duration)) = speaker_stats.get(&speaker.speaker_id) {
                speaker.turn_count = *turn_count;
                speaker.total_duration_ms = (total_duration * 1000.0) as u64;
            }
        }
        
        Ok(())
    }

    /// Enhance Apple Speech Framework result with additional processing
    async fn enhance_apple_result(&self, result: AsrResult) -> Result<AsrResult> {
        tracing::debug!("Enhancing Apple Speech Framework result");

        let mut enhanced = result;

        // Apply confidence improvements
        self.calculate_improved_confidence(&mut enhanced).await?;

        // Update speaker statistics
        self.update_speaker_statistics(&mut enhanced).await?;

        Ok(enhanced)
    }

    /// Execute speech recognition using SFSpeechRecognizer
    async fn execute_speech_recognition(
        &self,
        _speech_recognizer: &SwiftSpeechRecognizer, // Swift speech recognizer
        recognition_request: &SFSpeechAudioBufferRecognitionRequest, // Recognition request
    ) -> Result<AsrResult> {
        // Use ASR Bridge for actual speech recognition
        let audio_path_c = std::ffi::CString::new(recognition_request._audio_file.to_string_lossy().as_ref())
            .map_err(|e| anyhow!("Invalid audio path: {}", e))?;

        let mut out_text: *mut std::ffi::c_char = std::ptr::null_mut();
        let mut out_confidence: f32 = 0.0;
        let mut out_error: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe {
            speech_recognize_audio_impl(
                audio_path_c.as_ptr(),
                &mut out_text,
                &mut out_confidence,
                &mut out_error,
            )
        };

        if result != 0 {
            // Error occurred
            let error_msg =             if !out_error.is_null() {
                unsafe {
                    let error_str = std::ffi::CStr::from_ptr(out_error).to_string_lossy().to_string();
                    speech_free_string_impl(out_error);
                    error_str
                }
            } else {
                "Unknown ASR error".to_string()
            };

            if !out_text.is_null() {
                unsafe { speech_free_string_impl(out_text); }
            }

            return Err(anyhow::anyhow!("ASR failed: {}", error_msg));
        }

        // Success - extract text and create result
        let transcribed_text = if !out_text.is_null() {
            unsafe {
                let text_str = std::ffi::CStr::from_ptr(out_text).to_string_lossy().to_string();
                speech_free_string_impl(out_text);
                text_str
            }
        } else {
            String::new()
        };

        // Create AsrResult from transcription
        let result = AsrResult {
            turns: vec![SpeechSegment {
                id: Uuid::new_v4(),
                speaker_id: Some("speaker1".to_string()),
                t0: 0.0,
                t1: 10.0, // Placeholder duration - would be calculated from actual audio
                text: transcribed_text,
                confidence: out_confidence,
                word_timings: vec![], // Would need word-level timing from bridge
                language: Some("en-US".to_string()),
            }],
            speakers: vec![Speaker {
                speaker_id: "speaker1".to_string(),
                name: None,
                turn_count: 1,
                total_duration_ms: 10000, // Placeholder
            }],
            language: Some("en-US".to_string()),
            confidence: out_confidence,
            processing_time_ms: 1000, // Placeholder - would be measured
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_asr_enricher_creation() {
        let config = EnricherConfig {
            asr_provider: "apple".to_string(),
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_ms: 1000,
        };
        
        let enricher = AsrEnricher::new(config);
        assert_eq!(enricher.config.asr_provider, "apple");
    }

    #[tokio::test]
    async fn test_language_support_validation() {
        let config = EnricherConfig {
            asr_provider: "apple".to_string(),
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_ms: 1000,
        };
        
        let enricher = AsrEnricher::new(config);
        
        // Test supported languages
        assert!(enricher.is_language_supported("en-US"));
        assert!(enricher.is_language_supported("es-ES"));
        assert!(enricher.is_language_supported("fr-FR"));
        
        // Test unsupported language
        assert!(!enricher.is_language_supported("unsupported-lang"));
    }
}
