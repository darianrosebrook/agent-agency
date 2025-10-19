//! @darianrosebrook
//! Captions ingestor (SRT/VTT) for speech turn extraction

use crate::types::*;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub struct CaptionsIngestor;

impl CaptionsIngestor {
    pub fn new() -> Self {
        Self
    }

    /// Ingest caption file (SRT/VTT)
    pub async fn ingest(&self, path: &Path, project_scope: Option<&str>) -> Result<IngestResult> {
        tracing::debug!("Ingesting captions from: {:?}", path);

        // Compute SHA256
        let sha256 = self.compute_sha256(path)?;

        let doc_id = Uuid::new_v4();
        let uri = path.to_string_lossy().to_string();
        let ingested_at = Utc::now();

        // Extract file extension
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Parse SRT/VTT segments and extract timings and text

        let speech_turns = match extension.as_str() {
            "srt" => self.parse_srt(path).await?,
            "vtt" => self.parse_vtt(path).await?,
            _ => return Err(anyhow!("Unsupported captions format: {}", extension)),
        };

        // Create a minimal segment for captions
        let segment = Segment {
            id: Uuid::new_v4(),
            segment_type: SegmentType::Speech,
            t0: speech_turns.first().map(|s| s.t0),
            t1: speech_turns.last().map(|s| s.t1),
            bbox: None,
            content_hash: sha256.clone(),
            quality_score: 0.9,
            stability_score: None,
            blocks: vec![Block {
                id: Uuid::new_v4(),
                role: BlockRole::Caption,
                text: format!("{} caption segments", speech_turns.len()),
                bbox: None,
                ocr_confidence: Some(0.95),
                raw_bytes: None,
            }],
        };

        Ok(IngestResult {
            document_id: doc_id,
            uri,
            sha256,
            kind: DocumentKind::Transcript,
            project_scope: project_scope.map(|s| s.to_string()),
            segments: vec![segment],
            speech_turns: Some(speech_turns),
            diagram_data: None,
            ingested_at,
            quality_score: 0.9,
            toolchain: "srt/vtt=native".to_string(),
        })
    }

    async fn parse_srt(&self, path: &Path) -> Result<Vec<SpeechTurn>> {
        // Parse SRT format: sequence number \n timestamp --> timestamp \n text \n
        // Extract timing and text from each segment

        let content = fs::read_to_string(path).context("Failed to read SRT file")?;

        let mut turns = Vec::new();
        let mut lines = content.lines().peekable();

        while lines.peek().is_some() {
            // Skip sequence number
            if let Some(line) = lines.next() {
                if line.trim().is_empty() || !line.chars().all(|c| c.is_ascii_digit()) {
                    continue;
                }
            }

            // Parse timestamp line
            if let Some(time_line) = lines.next() {
                if let Ok((t0, t1)) = self.parse_timestamp_line(time_line) {
                    // Collect text lines until empty line
                    let mut text = String::new();
                    while let Some(text_line) = lines.next() {
                        if text_line.trim().is_empty() {
                            break;
                        }
                        if !text.is_empty() {
                            text.push(' ');
                        }
                        text.push_str(text_line);
                    }

                    if !text.is_empty() {
                        // Extract word timings if available (basic implementation)
                        let word_timings = self.extract_word_timings(&text, t0, t1);
                        
                        turns.push(SpeechTurn {
                            id: Uuid::new_v4(),
                            speaker_id: self.extract_speaker_id(&text),
                            provider: "srt".to_string(),
                            t0,
                            t1,
                            text: self.clean_text(&text),
                            confidence: 0.95,
                            word_timings,
                        });
                    }
                }
            }
        }

        Ok(turns)
    }

    async fn parse_vtt(&self, path: &Path) -> Result<Vec<SpeechTurn>> {
        // Parse WEBVTT format

        let content = fs::read_to_string(path).context("Failed to read VTT file")?;

        // Skip WEBVTT header
        let mut lines = content.lines().peekable();
        if let Some(first) = lines.peek() {
            if first.starts_with("WEBVTT") {
                lines.next();
            }
        }

        let mut turns = Vec::new();

        while lines.peek().is_some() {
            // Parse timestamp line
            if let Some(time_line) = lines.next() {
                if time_line.trim().is_empty() {
                    continue;
                }

                if let Ok((t0, t1)) = self.parse_timestamp_line(time_line) {
                    // Collect text lines until empty line
                    let mut text = String::new();
                    while let Some(text_line) = lines.next() {
                        if text_line.trim().is_empty() {
                            break;
                        }
                        if !text.is_empty() {
                            text.push(' ');
                        }
                        text.push_str(text_line);
                    }

                    if !text.is_empty() {
                        // Extract word timings if available (basic implementation)
                        let word_timings = self.extract_word_timings(&text, t0, t1);
                        
                        turns.push(SpeechTurn {
                            id: Uuid::new_v4(),
                            speaker_id: self.extract_speaker_id(&text),
                            provider: "vtt".to_string(),
                            t0,
                            t1,
                            text: self.clean_text(&text),
                            confidence: 0.95,
                            word_timings,
                        });
                    }
                }
            }
        }

        Ok(turns)
    }

    fn parse_timestamp_line(&self, line: &str) -> Result<(f32, f32)> {
        // Format: HH:MM:SS.mmm --> HH:MM:SS.mmm
        let parts: Vec<&str> = line.split("-->").collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid timestamp format"));
        }

        let t0 = self.parse_timestamp(parts[0].trim())?;
        let t1 = self.parse_timestamp(parts[1].trim())?;

        Ok((t0, t1))
    }

    fn parse_timestamp(&self, ts: &str) -> Result<f32> {
        // Format: HH:MM:SS.mmm or MM:SS.mmm
        let parts: Vec<&str> = ts.split(':').collect();

        if parts.len() == 2 {
            // MM:SS.mmm
            let mm: f32 = parts[0].parse().context("Invalid minutes")?;
            let ss: f32 = parts[1].parse().context("Invalid seconds")?;
            Ok(mm * 60.0 + ss)
        } else if parts.len() == 3 {
            // HH:MM:SS.mmm
            let hh: f32 = parts[0].parse().context("Invalid hours")?;
            let mm: f32 = parts[1].parse().context("Invalid minutes")?;
            let ss: f32 = parts[2].parse().context("Invalid seconds")?;
            Ok(hh * 3600.0 + mm * 60.0 + ss)
        } else {
            Err(anyhow!("Invalid timestamp format"))
        }
    }

    fn compute_sha256(&self, path: &Path) -> Result<String> {
        let data = fs::read(path).context("Failed to read captions file")?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Extract speaker ID from text if present (e.g., "Speaker 1: Hello world")
    fn extract_speaker_id(&self, text: &str) -> Option<String> {
        if let Some(colon_pos) = text.find(':') {
            let potential_speaker = &text[..colon_pos].trim();
            if potential_speaker.len() < 50 && potential_speaker.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) {
                return Some(potential_speaker.to_string());
            }
        }
        None
    }

    /// Clean text by removing speaker prefixes and normalizing whitespace
    fn clean_text(&self, text: &str) -> String {
        let mut cleaned = text.to_string();
        
        // Remove speaker prefix if present
        if let Some(colon_pos) = cleaned.find(':') {
            let potential_speaker = &cleaned[..colon_pos].trim();
            if potential_speaker.len() < 50 && potential_speaker.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) {
                cleaned = cleaned[colon_pos + 1..].trim().to_string();
            }
        }
        
        // Normalize whitespace
        cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
        
        // Remove HTML tags if present
        cleaned = self.remove_html_tags(&cleaned);
        
        cleaned
    }

    /// Remove HTML tags from text
    fn remove_html_tags(&self, text: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        
        for ch in text.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ => if !in_tag {
                    result.push(ch);
                }
            }
        }
        
        result
    }

    /// Extract basic word timings by dividing the time range evenly
    fn extract_word_timings(&self, text: &str, t0: f32, t1: f32) -> Vec<WordTiming> {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return vec![];
        }
        
        let duration = t1 - t0;
        let word_duration = duration / words.len() as f32;
        
        words.into_iter().enumerate().map(|(i, word)| {
            let start_time = t0 + (i as f32 * word_duration);
            let end_time = start_time + word_duration;
            
            WordTiming {
                t0: start_time,
                t1: end_time,
                token: word.to_string(),
            }
        }).collect()
    }
}

impl Default for CaptionsIngestor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_captions_ingestor_init() {
        let _ingestor = CaptionsIngestor::new();
    }

    #[test]
    fn test_parse_timestamp() {
        let ingestor = CaptionsIngestor::new();

        let t1 = ingestor.parse_timestamp("00:05:23.500").unwrap();
        assert!((t1 - 323.5).abs() < 0.01);

        let t2 = ingestor.parse_timestamp("05:23.500").unwrap();
        assert!((t2 - 323.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_unsupported_format() {
        let ingestor = CaptionsIngestor::new();
        let path = Path::new("/tmp/test.txt");
        let result = ingestor.ingest(path, None).await;
        assert!(result.is_err());
    }
}
