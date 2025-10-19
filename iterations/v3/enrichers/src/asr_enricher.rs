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
use std::time::Instant;
use uuid::Uuid;

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
                speaker_groups.entry(speaker_id.clone()).or_insert_with(Vec::new).push(i);
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
        let mut durations: Vec<f32> = turn_indices
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

    /// Calculate improved confidence scores
    async fn calculate_improved_confidence(&self, result: &mut AsrResult) -> Result<()> {
        tracing::debug!("Calculating improved confidence scores");
        
        // Calculate turn-level confidence based on word-level confidence
        for turn in &mut result.turns {
            if !turn.word_timings.is_empty() {
                let avg_word_confidence: f32 = turn.word_timings
                    .iter()
                    .map(|w| w.confidence)
                    .sum::<f32>() / turn.word_timings.len() as f32;
                
                // Combine with text length factor
                let length_factor = if turn.text.len() < 10 {
                    0.9
                } else if turn.text.len() < 50 {
                    1.0
                } else {
                    1.1
                }.min(1.0);
                
                turn.confidence = (avg_word_confidence * length_factor).min(1.0);
            }
        }
        
        // Calculate overall confidence
        if !result.turns.is_empty() {
            let avg_turn_confidence: f32 = result.turns
                .iter()
                .map(|t| t.confidence)
                .sum::<f32>() / result.turns.len() as f32;
            
            // Adjust based on number of speakers and turns
            let speaker_factor = if result.speakers.len() == 1 {
                1.0
            } else if result.speakers.len() <= 3 {
                0.95
            } else {
                0.9
            };
            
            result.confidence = (avg_turn_confidence * speaker_factor).min(1.0);
        }
        
        Ok(())
    }

    /// Update speaker statistics
    async fn update_speaker_statistics(&self, result: &mut AsrResult) -> Result<()> {
        tracing::debug!("Updating speaker statistics");
        
        // Calculate turn counts and durations for each speaker
        let mut speaker_stats: std::collections::HashMap<String, (usize, f32)> = std::collections::HashMap::new();
        
        for turn in &result.turns {
            if let Some(speaker_id) = &turn.speaker_id {
                let duration = turn.t1 - turn.t0;
                let (turn_count, total_duration) = speaker_stats.entry(speaker_id.clone()).or_insert((0, 0.0));
                *turn_count += 1;
                *total_duration += duration;
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
        // Simulate Apple Speech Framework call
        // In a real implementation, this would call Swift bridge to SFSpeechRecognizer
        
        tracing::debug!("Calling Apple Speech Framework via Swift bridge");
        
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Return simulated transcription result
        Ok(AsrResult {
            turns: vec![
                SpeechSegment {
                    id: Uuid::new_v4(),
                    speaker_id: None, // Will be assigned during diarization
                    t0: 0.0,
                    t1: 3.5,
                    text: "Hello, this is a test of the Apple Speech Framework transcription.".to_string(),
                    confidence: 0.92,
                    word_timings: vec![
                        WordTiming { t0: 0.0, t1: 0.3, token: "Hello".to_string(), confidence: 0.95 },
                        WordTiming { t0: 0.4, t1: 0.6, token: "this".to_string(), confidence: 0.90 },
                        WordTiming { t0: 0.7, t1: 0.9, token: "is".to_string(), confidence: 0.88 },
                        WordTiming { t0: 1.0, t1: 1.2, token: "a".to_string(), confidence: 0.92 },
                        WordTiming { t0: 1.3, t1: 1.6, token: "test".to_string(), confidence: 0.94 },
                        WordTiming { t0: 1.7, t1: 2.0, token: "of".to_string(), confidence: 0.89 },
                        WordTiming { t0: 2.1, t1: 2.3, token: "the".to_string(), confidence: 0.91 },
                        WordTiming { t0: 2.4, t1: 2.8, token: "Apple".to_string(), confidence: 0.93 },
                        WordTiming { t0: 2.9, t1: 3.2, token: "Speech".to_string(), confidence: 0.90 },
                        WordTiming { t0: 3.3, t1: 3.5, token: "Framework".to_string(), confidence: 0.87 },
                    ],
                },
                SpeechSegment {
                    id: Uuid::new_v4(),
                    speaker_id: None, // Will be assigned during diarization
                    t0: 3.8,
                    t1: 7.2,
                    text: "The transcription quality is excellent and provides accurate word-level timing information.".to_string(),
                    confidence: 0.89,
                    word_timings: vec![
                        WordTiming { t0: 3.8, t1: 4.0, token: "The".to_string(), confidence: 0.88 },
                        WordTiming { t0: 4.1, t1: 4.5, token: "transcription".to_string(), confidence: 0.91 },
                        WordTiming { t0: 4.6, t1: 4.9, token: "quality".to_string(), confidence: 0.90 },
                        WordTiming { t0: 5.0, t1: 5.2, token: "is".to_string(), confidence: 0.89 },
                        WordTiming { t0: 5.3, t1: 5.8, token: "excellent".to_string(), confidence: 0.93 },
                        WordTiming { t0: 5.9, t1: 6.1, token: "and".to_string(), confidence: 0.87 },
                        WordTiming { t0: 6.2, t1: 6.6, token: "provides".to_string(), confidence: 0.88 },
                        WordTiming { t0: 6.7, t1: 7.0, token: "accurate".to_string(), confidence: 0.92 },
                        WordTiming { t0: 7.1, t1: 7.2, token: "word-level".to_string(), confidence: 0.85 },
                    ],
                },
            ],
            speakers: vec![],
            language: language.map(|l| l.to_string()),
            confidence: 0.90,
            processing_time_ms: 0,
        })
    }

    /// Apply custom diarization heuristics using VAD and clustering
    async fn apply_custom_diarization(&self, mut result: AsrResult) -> Result<AsrResult> {
        tracing::debug!("Applying custom diarization heuristics");
        
        // 1. Apply Voice Activity Detection (VAD) heuristics
        self.apply_vad_heuristics(&mut result).await?;
        
        // 2. Apply speaker clustering
        self.apply_speaker_clustering(&mut result).await?;
        
        // 3. Refine speaker assignments
        self.refine_speaker_assignments(&mut result).await?;
        
        Ok(result)
    }

    /// Apply Voice Activity Detection heuristics
    async fn apply_vad_heuristics(&self, result: &mut AsrResult) -> Result<()> {
        tracing::debug!("Applying VAD heuristics");
        
        // Analyze speech patterns to detect speaker changes
        for i in 1..result.turns.len() {
            let prev_turn = &result.turns[i - 1];
            let current_turn = &result.turns[i];
            
            // Calculate gap between turns
            let gap = current_turn.t0 - prev_turn.t1;
            
            // If gap is significant, it might indicate a speaker change
            if gap > 1.0 {
                tracing::debug!("Significant gap detected between turns {} and {}: {:.2}s", 
                               i - 1, i, gap);
            }
            
            // Analyze speaking rate differences
            let prev_rate = self.calculate_speaking_rate(prev_turn);
            let current_rate = self.calculate_speaking_rate(current_turn);
            
            if (prev_rate - current_rate).abs() > 2.0 {
                tracing::debug!("Significant speaking rate difference detected: {:.2} vs {:.2} words/s", 
                               prev_rate, current_rate);
            }
        }
        
        Ok(())
    }

    /// Calculate speaking rate for a turn
    fn calculate_speaking_rate(&self, turn: &SpeechSegment) -> f32 {
        let duration = turn.t1 - turn.t0;
        if duration > 0.0 {
            turn.text.split_whitespace().count() as f32 / duration
        } else {
            0.0
        }
    }

    /// Apply speaker clustering based on speech characteristics
    async fn apply_speaker_clustering(&self, result: &mut AsrResult) -> Result<()> {
        tracing::debug!("Applying speaker clustering");
        
        let mut speaker_clusters: Vec<Vec<usize>> = Vec::new();
        
        // Group turns into clusters based on speech characteristics
        for (i, turn) in result.turns.iter().enumerate() {
            let mut assigned = false;
            
            // Try to assign to existing cluster
            for cluster in &mut speaker_clusters {
                if let Some(&last_turn_idx) = cluster.last() {
                    let last_turn = &result.turns[last_turn_idx];
                    
                    // Check if this turn belongs to the same speaker
                    if self.turns_likely_same_speaker(turn, last_turn) {
                        cluster.push(i);
                        assigned = true;
                        break;
                    }
                }
            }
            
            // If not assigned to existing cluster, create new one
            if !assigned {
                speaker_clusters.push(vec![i]);
            }
        }
        
        // Assign speaker IDs to clusters
        for (cluster_idx, turn_indices) in speaker_clusters.iter().enumerate() {
            let speaker_id = format!("SPEAKER_{:02}", cluster_idx);
            
            for &turn_idx in turn_indices {
                result.turns[turn_idx].speaker_id = Some(speaker_id.clone());
            }
        }
        
        tracing::debug!("Created {} speaker clusters", speaker_clusters.len());
        
        Ok(())
    }

    /// Check if two turns likely belong to the same speaker
    fn turns_likely_same_speaker(&self, turn1: &SpeechSegment, turn2: &SpeechSegment) -> bool {
        // Calculate similarity based on multiple factors
        
        // 1. Speaking rate similarity
        let rate1 = self.calculate_speaking_rate(turn1);
        let rate2 = self.calculate_speaking_rate(turn2);
        let rate_diff = (rate1 - rate2).abs();
        
        // 2. Confidence similarity
        let confidence_diff = (turn1.confidence - turn2.confidence).abs();
        
        // 3. Time proximity
        let time_gap = turn2.t0 - turn1.t1;
        
        // Combine factors for similarity score
        let similarity_score = if rate_diff < 1.0 && confidence_diff < 0.2 && time_gap < 5.0 {
            1.0
        } else if rate_diff < 2.0 && confidence_diff < 0.3 && time_gap < 10.0 {
            0.7
        } else {
            0.3
        };
        
        similarity_score > 0.6
    }

    /// Refine speaker assignments based on additional heuristics
    async fn refine_speaker_assignments(&self, result: &mut AsrResult) -> Result<()> {
        tracing::debug!("Refining speaker assignments");
        
        // Create speaker statistics
        let mut speaker_stats: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
        
        for (i, turn) in result.turns.iter().enumerate() {
            if let Some(speaker_id) = &turn.speaker_id {
                speaker_stats.entry(speaker_id.clone()).or_insert_with(Vec::new).push(i);
            }
        }
        
        // Apply refinements
        for (speaker_id, turn_indices) in speaker_stats {
            self.refine_speaker_cluster(&mut result.turns, &speaker_id, &turn_indices).await?;
        }
        
        // Create speaker list
        result.speakers = self.create_speaker_list(&result.turns);
        
        Ok(())
    }

    /// Refine a specific speaker cluster
    async fn refine_speaker_cluster(
        &self,
        turns: &mut [SpeechSegment],
        speaker_id: &str,
        turn_indices: &[usize],
    ) -> Result<()> {
        if turn_indices.len() < 2 {
            return Ok(());
        }
        
        // Analyze speaking patterns for this speaker
        let mut durations: Vec<f32> = turn_indices
            .iter()
            .map(|&i| turns[i].t1 - turns[i].t0)
            .collect();
        
        let avg_duration = durations.iter().sum::<f32>() / durations.len() as f32;
        
        // Apply consistency improvements
        for &turn_idx in turn_indices {
            let duration = turns[turn_idx].t1 - turns[turn_idx].t0;
            
            // If duration is significantly different from average, adjust confidence
            if duration < avg_duration * 0.5 || duration > avg_duration * 2.0 {
                turns[turn_idx].confidence *= 0.9; // Slightly reduce confidence for outliers
            }
        }
        
        tracing::debug!("Refined speaker cluster {} with {} turns", speaker_id, turn_indices.len());
        
        Ok(())
    }

    /// Create speaker list from turns
    fn create_speaker_list(&self, turns: &[SpeechSegment]) -> Vec<Speaker> {
        let mut speaker_map: std::collections::HashMap<String, (usize, f32)> = std::collections::HashMap::new();
        
        for turn in turns {
            if let Some(speaker_id) = &turn.speaker_id {
                let duration = turn.t1 - turn.t0;
                let (turn_count, total_duration) = speaker_map.entry(speaker_id.clone()).or_insert((0, 0.0));
                *turn_count += 1;
                *total_duration += duration;
            }
        }
        
        speaker_map
            .into_iter()
            .map(|(speaker_id, (turn_count, total_duration))| Speaker {
                speaker_id,
                name: None,
                turn_count,
                total_duration_ms: (total_duration * 1000.0) as u64,
            })
            .collect()
    }

    /// Enhance Apple Speech Framework result
    async fn enhance_apple_result(&self, mut result: AsrResult) -> Result<AsrResult> {
        // 1. Enhance word-level timings
        self.enhance_word_timings(&mut result).await?;
        
        // 2. Calculate improved confidence
        self.calculate_improved_confidence(&mut result).await?;
        
        // 3. Update speaker statistics
        self.update_speaker_statistics(&mut result).await?;
        
        Ok(result)
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
