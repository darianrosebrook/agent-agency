// @darianrosebrook
// Vision Framework bridge for macOS document analysis
// Provides OCR, document structure, and table extraction via Vision Framework

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Result from Vision Framework document analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionAnalysisResult {
    pub blocks: Vec<VisionBlock>,
    pub tables: Vec<VisionTable>,
    pub confidence: f32,
    pub processing_time_ms: u64,
}

/// Individual text block from document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionBlock {
    pub text: String,
    pub role: String, // "title", "bullet", "code", "table", "figure", "paragraph"
    pub bbox: BoundingBox,
    pub confidence: f32,
}

/// Bounding box for layout positioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Table structure extracted from document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionTable {
    pub rows: usize,
    pub columns: usize,
    pub cells: Vec<TableCell>,
    pub bbox: BoundingBox,
}

/// Individual table cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub row: usize,
    pub column: usize,
    pub text: String,
}

// FFI declarations for Vision Framework
#[link(name = "Foundation", kind = "framework")]
#[link(name = "Vision", kind = "framework")]
extern "C" {
    /// Analyze document image and extract structured content
    /// Returns JSON-encoded VisionAnalysisResult as C string (must be freed by caller)
    fn analyze_document_request(
        image_bytes: *const u8,
        image_len: usize,
        timeout_ms: u64,
    ) -> *mut c_char;
}

/// Vision Framework bridge for document analysis
pub struct VisionBridge;

impl VisionBridge {
    /// Analyze document image using Vision Framework
    ///
    /// Performs OCR, document structure detection, and table extraction
    /// using Apple's native Vision Framework with RecognizeDocumentsRequest.
    ///
    /// # Arguments
    /// * `image_data` - Raw image bytes (PNG/JPEG)
    /// * `timeout_ms` - Maximum processing time in milliseconds
    ///
    /// # Returns
    /// `VisionAnalysisResult` with extracted blocks and tables
    ///
    /// # Errors
    /// Returns error if image processing fails or timeout exceeded
    pub async fn analyze_document(
        image_data: &[u8],
        timeout_ms: Option<u64>,
    ) -> Result<VisionAnalysisResult> {
        let timeout = timeout_ms.unwrap_or(5000); // Default 5 second timeout

        // Safety: Call Swift bridge with proper memory management
        unsafe {
            let result_ptr = analyze_document_request(
                image_data.as_ptr(),
                image_data.len(),
                timeout,
            );

            if result_ptr.is_null() {
                return Err(anyhow!("Vision Framework returned null pointer"));
            }

            // Convert C string to Rust string
            let result_str = CStr::from_ptr(result_ptr)
                .to_string_lossy()
                .to_string();

            // Free the C string memory allocated by Swift
            libc::free(result_ptr as *mut libc::c_void);

            // Parse JSON result
            let analysis: VisionAnalysisResult =
                serde_json::from_str(&result_str)
                    .map_err(|e| anyhow!("Failed to parse Vision result: {}", e))?;

            Ok(analysis)
        }
    }

    /// Check if image is readable and valid
    pub fn is_image_valid(image_data: &[u8]) -> bool {
        // Check for valid image magic bytes
        if image_data.len() < 4 {
            return false;
        }

        // PNG: 89 50 4E 47
        if image_data[0] == 0x89 && image_data[1] == 0x50 {
            return true;
        }

        // JPEG: FF D8 FF
        if image_data[0] == 0xFF && image_data[1] == 0xD8 {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_image_detection() {
        let png_data = vec![0x89, 0x50, 0x4E, 0x47];
        assert!(VisionBridge::is_image_valid(&png_data));
    }

    #[test]
    fn test_jpeg_image_detection() {
        let jpeg_data = vec![0xFF, 0xD8, 0xFF];
        assert!(VisionBridge::is_image_valid(&jpeg_data));
    }

    #[test]
    fn test_invalid_image_detection() {
        let invalid_data = vec![0x00, 0x00, 0x00];
        assert!(!VisionBridge::is_image_valid(&invalid_data));
    }

    #[test]
    fn test_empty_image_detection() {
        let empty_data: Vec<u8> = vec![];
        assert!(!VisionBridge::is_image_valid(&empty_data));
    }
}
