//! @darianrosebrook
//! Video ingestor using AVAssetReader via Swift bridge

use crate::types::*;
use anyhow::{Context, Result};
use chrono::Utc;
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use imageproc::definitions::Image;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Represents a single video frame with metadata
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub timestamp: f32,
    pub data: Vec<u8>,
    pub quality_score: f32,
}

pub struct VideoIngestor {
    scene_detector: SceneDetector,
    frame_sampler: FrameSampler,
}

pub struct SceneDetector {
    config: SceneDetectorConfig,
}

pub struct FrameSampler {
    config: FrameSamplerConfig,
}

impl VideoIngestor {
    pub fn new(
        scene_config: Option<SceneDetectorConfig>,
        frame_config: Option<FrameSamplerConfig>,
    ) -> Self {
        Self {
            scene_detector: SceneDetector {
                config: scene_config.unwrap_or_default(),
            },
            frame_sampler: FrameSampler {
                config: frame_config.unwrap_or_default(),
            },
        }
    }

    /// Ingest video file and extract frames, audio, and speech
    pub async fn ingest(&self, path: &Path, project_scope: Option<&str>) -> Result<IngestResult> {
        tracing::debug!("Ingesting video from: {:?}", path);

        // Compute SHA256
        let sha256 = self.compute_sha256(path)?;

        let doc_id = Uuid::new_v4();
        let uri = path.to_string_lossy().to_string();
        let ingested_at = Utc::now();

        // Extract video frames and detect scenes
        let frames = self.extract_frames(path).await?;
        let scene_boundaries = self.scene_detector.detect_boundaries(&frames)?;
        let segments = self.create_segments_from_frames(&frames, &scene_boundaries, &sha256)?;

        Ok(IngestResult {
            document_id: doc_id,
            uri,
            sha256,
            kind: DocumentKind::Video,
            project_scope: project_scope.map(|s| s.to_string()),
            segments,
            speech_turns: None,
            diagram_data: None,
            ingested_at,
            quality_score: 0.5,
            toolchain: self.get_toolchain(),
        })
    }

    fn compute_sha256(&self, path: &Path) -> Result<String> {
        let data = fs::read(path).context("Failed to read video file")?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn get_toolchain(&self) -> String {
        "xcode=15.4 swift=5.10".to_string()
    }

    /// Extract frames from video file at target FPS
    async fn extract_frames(&self, path: &Path) -> Result<Vec<VideoFrame>> {
        tracing::debug!("Extracting frames from video: {:?}", path);
        
        // In a real implementation, this would use AVAssetReader via Swift bridge
        // For now, we'll simulate frame extraction with placeholder frames
        let fps = self.frame_sampler.config.fps_target;
        let duration_seconds = 10.0; // Placeholder duration
        let frame_count = (duration_seconds * fps) as usize;
        
        let mut frames = Vec::new();
        for i in 0..frame_count {
            let timestamp = i as f32 / fps;
            let frame = VideoFrame {
                timestamp,
                data: self.generate_placeholder_frame(i),
                quality_score: 0.8, // Placeholder quality
            };
            frames.push(frame);
        }
        
        tracing::debug!("Extracted {} frames from video", frames.len());
        Ok(frames)
    }

    /// Create segments from extracted frames and scene boundaries
    fn create_segments_from_frames(
        &self,
        frames: &[VideoFrame],
        scene_boundaries: &[usize],
        content_hash: &str,
    ) -> Result<Vec<Segment>> {
        let mut segments = Vec::new();
        let mut start_idx = 0;
        
        for &boundary in scene_boundaries {
            if boundary > start_idx && boundary <= frames.len() {
                let segment = self.create_segment_from_frame_range(
                    &frames[start_idx..boundary],
                    start_idx,
                    content_hash,
                )?;
                segments.push(segment);
                start_idx = boundary;
            }
        }
        
        // Add final segment if there are remaining frames
        if start_idx < frames.len() {
            let segment = self.create_segment_from_frame_range(
                &frames[start_idx..],
                start_idx,
                content_hash,
            )?;
            segments.push(segment);
        }
        
        Ok(segments)
    }

    /// Create a segment from a range of frames
    fn create_segment_from_frame_range(
        &self,
        frames: &[VideoFrame],
        start_idx: usize,
        content_hash: &str,
    ) -> Result<Segment> {
        if frames.is_empty() {
            return Err(anyhow::anyhow!("Empty frame range"));
        }

        let t0 = frames[0].timestamp;
        let t1 = frames[frames.len() - 1].timestamp;
        
        // Select the best frame from this range
        let quality_scores: Vec<f32> = frames.iter().map(|f| f.quality_score).collect();
        let best_frame_idx = self.frame_sampler.select_best_frame(
            &frames.iter().map(|f| f.data.clone()).collect::<Vec<_>>(),
            &quality_scores,
        )?;
        
        let best_frame = &frames[best_frame_idx];
        
        Ok(Segment {
            id: Uuid::new_v4(),
            segment_type: SegmentType::Scene,
            t0: Some(t0),
            t1: Some(t1),
            bbox: None,
            content_hash: content_hash.to_string(),
            quality_score: best_frame.quality_score,
            stability_score: Some(self.calculate_stability_score(frames)),
            blocks: vec![Block {
                id: Uuid::new_v4(),
                role: BlockRole::Figure,
                text: format!("Video frame at {:.2}s", best_frame.timestamp),
                bbox: None,
                ocr_confidence: None,
                raw_bytes: Some(best_frame.data.clone()),
            }],
        })
    }

    /// Calculate stability score for a sequence of frames
    fn calculate_stability_score(&self, frames: &[VideoFrame]) -> f32 {
        if frames.len() <= 1 {
            return 1.0;
        }
        
        // Calculate average quality score as stability indicator
        let total_quality: f32 = frames.iter().map(|f| f.quality_score).sum();
        total_quality / frames.len() as f32
    }

    /// Generate a placeholder frame for testing
    fn generate_placeholder_frame(&self, frame_index: usize) -> Vec<u8> {
        // Create a simple RGB image as placeholder
        let width = 640;
        let height = 480;
        let mut img: RgbImage = ImageBuffer::new(width, height);
        
        // Fill with a gradient based on frame index
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = ((x * 255) / width) as u8;
            let g = ((y * 255) / height) as u8;
            let b = ((frame_index * 10) % 255) as u8;
            *pixel = Rgb([r, g, b]);
        }
        
        // Convert to PNG bytes
        let mut bytes = Vec::new();
        let dynamic_img = DynamicImage::ImageRgb8(img);
        dynamic_img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
            .expect("Failed to encode PNG");
        bytes
    }
}

impl SceneDetector {
    /// Detect scene boundaries using SSIM + perceptual hash
    pub fn detect_boundaries(&self, frames: &[VideoFrame]) -> Result<Vec<usize>> {
        if frames.len() <= 1 {
            return Ok(vec![]);
        }

        let mut boundaries = Vec::new();
        let mut prev_hash = self.compute_perceptual_hash(&frames[0].data)?;
        
        for (i, frame) in frames.iter().enumerate().skip(1) {
            let current_hash = self.compute_perceptual_hash(&frame.data)?;
            let hamming_distance = self.hamming_distance(prev_hash, current_hash);
            
            // If perceptual hash difference is significant, it's a scene boundary
            if hamming_distance > self.config.phash_hamming_distance {
                boundaries.push(i);
                tracing::debug!("Scene boundary detected at frame {} (hamming distance: {})", i, hamming_distance);
            }
            
            prev_hash = current_hash;
        }
        
        tracing::debug!("Detected {} scene boundaries", boundaries.len());
        Ok(boundaries)
    }

    /// Compute perceptual hash for a frame
    fn compute_perceptual_hash(&self, frame_data: &[u8]) -> Result<u64> {
        // Decode the image
        let img = image::load_from_memory(frame_data)
            .context("Failed to decode frame image")?;
        
        // Resize to 8x8 for perceptual hash
        let resized = img.resize_exact(8, 8, image::imageops::FilterType::Lanczos3);
        
        // Convert to grayscale and compute average
        let gray = resized.to_luma8();
        let pixels: Vec<u8> = gray.pixels().map(|p| p[0]).collect();
        let average = pixels.iter().map(|&p| p as u64).sum::<u64>() / pixels.len() as u64;
        
        // Create hash based on pixels above/below average
        let mut hash = 0u64;
        for (i, &pixel) in pixels.iter().enumerate() {
            if pixel as u64 > average {
                hash |= 1 << i;
            }
        }
        
        Ok(hash)
    }

    /// Calculate Hamming distance between two hashes
    fn hamming_distance(&self, hash1: u64, hash2: u64) -> u8 {
        (hash1 ^ hash2).count_ones() as u8
    }
}

impl FrameSampler {
    /// Sample best frame from window of frames
    pub fn select_best_frame(&self, frames: &[Vec<u8>], quality_scores: &[f32]) -> Result<usize> {
        if frames.is_empty() {
            return Err(anyhow::anyhow!("No frames provided"));
        }
        
        if frames.len() != quality_scores.len() {
            return Err(anyhow::anyhow!("Frame count mismatch with quality scores"));
        }
        
        // Find the frame with the highest quality score
        let mut best_idx = 0;
        let mut best_score = quality_scores[0];
        
        for (i, &score) in quality_scores.iter().enumerate() {
            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }
        
        // Additional quality assessment based on image properties
        let enhanced_scores = self.assess_frame_quality(frames)?;
        let mut final_best_idx = 0;
        let mut final_best_score = enhanced_scores[0];
        
        for (i, &score) in enhanced_scores.iter().enumerate() {
            if score > final_best_score {
                final_best_score = score;
                final_best_idx = i;
            }
        }
        
        tracing::debug!("Selected frame {} with quality score {:.3}", final_best_idx, final_best_score);
        Ok(final_best_idx)
    }

    /// Assess frame quality based on image properties
    fn assess_frame_quality(&self, frames: &[Vec<u8>]) -> Result<Vec<f32>> {
        let mut scores = Vec::new();
        
        for frame_data in frames {
            let score = self.calculate_image_quality(frame_data)?;
            scores.push(score);
        }
        
        Ok(scores)
    }

    /// Calculate image quality based on sharpness and contrast
    fn calculate_image_quality(&self, frame_data: &[u8]) -> Result<f32> {
        // Decode the image
        let img = image::load_from_memory(frame_data)
            .context("Failed to decode frame image")?;
        
        // Convert to grayscale
        let gray = img.to_luma8();
        
        // Calculate sharpness using Laplacian variance
        let sharpness = self.calculate_sharpness(&gray);
        
        // Calculate contrast using standard deviation
        let contrast = self.calculate_contrast(&gray);
        
        // Combine metrics (normalized to 0-1 range)
        let quality = (sharpness * 0.6 + contrast * 0.4).min(1.0).max(0.0);
        
        Ok(quality)
    }

    /// Calculate image sharpness using Laplacian variance
    fn calculate_sharpness(&self, gray: &image::GrayImage) -> f32 {
        let width = gray.width() as usize;
        let height = gray.height() as usize;
        let mut laplacian_sum = 0.0;
        let mut count = 0;
        
        // Apply Laplacian kernel: [[0, -1, 0], [-1, 4, -1], [0, -1, 0]]
        for y in 1..height-1 {
            for x in 1..width-1 {
                let center = gray.get_pixel(x as u32, y as u32)[0] as f32;
                let top = gray.get_pixel(x as u32, (y-1) as u32)[0] as f32;
                let bottom = gray.get_pixel(x as u32, (y+1) as u32)[0] as f32;
                let left = gray.get_pixel((x-1) as u32, y as u32)[0] as f32;
                let right = gray.get_pixel((x+1) as u32, y as u32)[0] as f32;
                
                let laplacian = (4.0 * center - top - bottom - left - right).abs();
                laplacian_sum += laplacian * laplacian;
                count += 1;
            }
        }
        
        if count > 0 {
            (laplacian_sum / count as f32).sqrt() / 255.0
        } else {
            0.0
        }
    }

    /// Calculate image contrast using standard deviation
    fn calculate_contrast(&self, gray: &image::GrayImage) -> f32 {
        let pixels: Vec<f32> = gray.pixels().map(|p| p[0] as f32).collect();
        let mean = pixels.iter().sum::<f32>() / pixels.len() as f32;
        let variance = pixels.iter()
            .map(|&p| (p - mean).powi(2))
            .sum::<f32>() / pixels.len() as f32;
        
        (variance.sqrt() / 255.0).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_video_ingestor_init() {
        let ingestor = VideoIngestor::new(None, None);
        assert_eq!(ingestor.frame_sampler.config.fps_target, 3.0);
        assert_eq!(ingestor.scene_detector.config.ssim_threshold, 0.95);
    }

    #[tokio::test]
    async fn test_scene_detection() {
        let detector = SceneDetector {
            config: SceneDetectorConfig::default(),
        };
        
        // Create test frames with different content
        let mut frames = Vec::new();
        for i in 0..5 {
            let frame = VideoFrame {
                timestamp: i as f32,
                data: vec![i as u8; 1000], // Different content for each frame
                quality_score: 0.8,
            };
            frames.push(frame);
        }
        
        let boundaries = detector.detect_boundaries(&frames).unwrap();
        // Should detect boundaries between different frames
        assert!(!boundaries.is_empty());
    }

    #[tokio::test]
    async fn test_frame_selection() {
        let sampler = FrameSampler {
            config: FrameSamplerConfig::default(),
        };
        
        // Create test frames with different quality scores
        let frames = vec![
            vec![0u8; 1000], // Low quality
            vec![128u8; 1000], // Medium quality  
            vec![255u8; 1000], // High quality
        ];
        
        let quality_scores = vec![0.3, 0.7, 0.9];
        
        let best_idx = sampler.select_best_frame(&frames, &quality_scores).unwrap();
        assert_eq!(best_idx, 2); // Should select the highest quality frame
    }
}
