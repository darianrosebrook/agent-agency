// @darianrosebrook
// Apple Speech Framework bridge for macOS ASR
// Provides speech recognition with speaker detection and word-level timing

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::ffi::CStr;
use std::os::raw::c_char;

/// Result from Speech Framework transcription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechTranscriptionResult {
    pub text: String,
    pub segments: Vec<SpeechSegment>,
    pub language: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
}

/// Speech segment with timing and speaker info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSegment {
    pub text: String,
    pub start_time: f32, // In seconds
    pub end_time: f32,   // In seconds
    pub speaker_id: Option<String>,
    pub confidence: f32,
    pub words: Vec<WordTiming>,
}

/// Individual word with precise timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTiming {
    pub word: String,
    pub start_time: f32, // In seconds
    pub end_time: f32,   // In seconds
    pub confidence: f32,
}

/// Speaker information from diarization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speaker {
    pub id: String,
    pub name: Option<String>,
    pub total_duration_ms: u64,
}

// FFI declarations for Speech Framework
#[link(name = "Foundation", kind = "framework")]
#[link(name = "Speech", kind = "framework")]
extern "C" {
    /// Transcribe audio using Speech Framework
    /// Returns JSON-encoded SpeechTranscriptionResult as C string (must be freed)
    fn transcribe_audio_request(
        audio_bytes: *const u8,
        audio_len: usize,
        language: *const c_char,
        timeout_ms: u64,
        include_timing: bool,
    ) -> *mut c_char;

    /// Perform speaker diarization on audio
    /// Returns JSON-encoded speaker list as C string (must be freed)
    fn diarize_audio_request(
        audio_bytes: *const u8,
        audio_len: usize,
        num_speakers: u32,
        timeout_ms: u64,
    ) -> *mut c_char;
}

/// Apple Speech Framework bridge for speech recognition
pub struct SpeechBridge;

impl SpeechBridge {
    /// Transcribe audio using Apple Speech Framework
    ///
    /// Performs speech-to-text conversion with optional word-level timing
    /// using Apple's native Speech Framework with SFSpeechRecognizer.
    ///
    /// # Arguments
    /// * `audio_data` - Raw audio bytes (WAV/M4A format)
    /// * `language` - BCP 47 language code (e.g., "en-US", "fr-FR")
    /// * `timeout_ms` - Maximum processing time in milliseconds
    /// * `word_timing` - If true, include word-level timing information
    ///
    /// # Returns
    /// `SpeechTranscriptionResult` with text, segments, and word timings
    ///
    /// # Errors
    /// Returns error if audio processing fails or timeout exceeded
    pub async fn transcribe(
        audio_data: &[u8],
        language: Option<&str>,
        timeout_ms: Option<u64>,
        word_timing: bool,
    ) -> Result<SpeechTranscriptionResult> {
        let lang = language.unwrap_or("en-US");
        let timeout = timeout_ms.unwrap_or(60000); // Default 60 second timeout

        // Safety: Call Swift bridge with proper memory management
        unsafe {
            let lang_cstr = std::ffi::CString::new(lang)
                .map_err(|e| anyhow!("Invalid language string: {}", e))?;

            let result_ptr = transcribe_audio_request(
                audio_data.as_ptr(),
                audio_data.len(),
                lang_cstr.as_ptr(),
                timeout,
                word_timing,
            );

            if result_ptr.is_null() {
                return Err(anyhow!("Speech Framework returned null pointer"));
            }

            // Convert C string to Rust string
            let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();

            // Free the C string memory allocated by Swift
            libc::free(result_ptr as *mut libc::c_void);

            // Parse JSON result
            let transcription: SpeechTranscriptionResult = serde_json::from_str(&result_str)
                .map_err(|e| anyhow!("Failed to parse transcription: {}", e))?;

            Ok(transcription)
        }
    }

    /// Perform speaker diarization on audio
    ///
    /// Identifies and separates different speakers in the audio.
    /// Returns speaker segments with timing information.
    ///
    /// # Arguments
    /// * `audio_data` - Raw audio bytes
    /// * `num_speakers` - Expected number of speakers (0 = auto-detect)
    /// * `timeout_ms` - Maximum processing time in milliseconds
    ///
    /// # Returns
    /// Vector of speakers with timing information
    ///
    /// # Errors
    /// Returns error if audio processing fails
    pub async fn diarize(
        audio_data: &[u8],
        num_speakers: Option<u32>,
        timeout_ms: Option<u64>,
    ) -> Result<Vec<Speaker>> {
        let num = num_speakers.unwrap_or(0); // 0 = auto-detect
        let timeout = timeout_ms.unwrap_or(120000); // Default 2 minute timeout

        // Safety: Call Swift bridge
        unsafe {
            let result_ptr =
                diarize_audio_request(audio_data.as_ptr(), audio_data.len(), num, timeout);

            if result_ptr.is_null() {
                return Err(anyhow!("Diarization returned null pointer"));
            }

            let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();

            libc::free(result_ptr as *mut libc::c_void);

            let speakers: Vec<Speaker> = serde_json::from_str(&result_str)
                .map_err(|e| anyhow!("Failed to parse diarization: {}", e))?;

            Ok(speakers)
        }
    }

    /// Check if audio file is in supported format
    pub fn is_audio_valid(audio_data: &[u8]) -> bool {
        if audio_data.len() < 4 {
            return false;
        }

        // WAV: "RIFF" header (52 49 46 46)
        if audio_data[0] == 0x52
            && audio_data[1] == 0x49
            && audio_data[2] == 0x46
            && audio_data[3] == 0x46
        {
            return true;
        }

        // M4A: "ftyp" box (66 74 79 70)
        if audio_data.len() > 11
            && audio_data[4] == 0x66
            && audio_data[5] == 0x74
            && audio_data[6] == 0x79
            && audio_data[7] == 0x70
        {
            return true;
        }

        false
    }

    /// Validate language code format
    pub fn is_language_valid(language: &str) -> bool {
        // Basic BCP 47 validation (e.g., "en-US", "fr-FR", "zh-Hans-CN")
        let parts: Vec<&str> = language.split('-').collect();

        if parts.is_empty() || parts.len() > 3 {
            return false;
        }

        // Language subtag: 2-3 letters
        if parts[0].len() < 2 || parts[0].len() > 3 {
            return false;
        }

        // All parts should be alphanumeric
        parts
            .iter()
            .all(|p| p.chars().all(|c| c.is_ascii_alphanumeric()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_audio_detection() {
        let wav_data = vec![0x52, 0x49, 0x46, 0x46, 0x00, 0x00];
        assert!(SpeechBridge::is_audio_valid(&wav_data));
    }

    #[test]
    fn test_m4a_audio_detection() {
        let m4a_data = vec![
            0x00, 0x00, 0x00, 0x00, // Size
            0x66, 0x74, 0x79, 0x70, // "ftyp"
            0x6D, 0x70, 0x34, 0x61, // "mp4a"
        ];
        assert!(SpeechBridge::is_audio_valid(&m4a_data));
    }

    #[test]
    fn test_invalid_audio_detection() {
        let invalid_data = vec![0x00, 0x00, 0x00, 0x00];
        assert!(!SpeechBridge::is_audio_valid(&invalid_data));
    }

    #[test]
    fn test_valid_language_codes() {
        assert!(SpeechBridge::is_language_valid("en"));
        assert!(SpeechBridge::is_language_valid("en-US"));
        assert!(SpeechBridge::is_language_valid("zh-Hans-CN"));
        assert!(SpeechBridge::is_language_valid("fr-FR"));
    }

    #[test]
    fn test_invalid_language_codes() {
        assert!(!SpeechBridge::is_language_valid(""));
        assert!(!SpeechBridge::is_language_valid("a")); // Too short
        assert!(!SpeechBridge::is_language_valid("en-US-CA-XX")); // Too many parts
        assert!(!SpeechBridge::is_language_valid("en_US")); // Invalid separator
    }
}
