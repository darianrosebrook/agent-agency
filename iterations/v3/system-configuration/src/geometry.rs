//! Common geometric types used across the codebase
//!
//! These types are shared across multiple crates for consistency
//! in computer vision and image processing operations.

use serde::{Deserialize, Serialize};

/// A rectangular bounding box with floating-point coordinates
///
/// Used for object detection, OCR regions, and other spatial operations
/// in computer vision and image processing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    /// X coordinate of the top-left corner
    pub x: f32,
    /// Y coordinate of the top-left corner
    pub y: f32,
    /// Width of the bounding box
    pub width: f32,
    /// Height of the bounding box
    pub height: f32,
}

impl BoundingBox {
    /// Create a new bounding box
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Get the area of the bounding box
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Check if this bounding box intersects with another
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        !(self.x + self.width <= other.x
            || other.x + other.width <= self.x
            || self.y + self.height <= other.y
            || other.y + other.height <= self.y)
    }

    /// Calculate intersection over union (IoU) with another bounding box
    pub fn iou(&self, other: &BoundingBox) -> f32 {
        if !self.intersects(other) {
            return 0.0;
        }

        let intersection_x1 = self.x.max(other.x);
        let intersection_y1 = self.y.max(other.y);
        let intersection_x2 = (self.x + self.width).min(other.x + other.width);
        let intersection_y2 = (self.y + self.height).min(other.y + other.height);

        let intersection_area = (intersection_x2 - intersection_x1) * (intersection_y2 - intersection_y1);
        let union_area = self.area() + other.area() - intersection_area;

        if union_area > 0.0 {
            intersection_area / union_area
        } else {
            0.0
        }
    }

    /// Get the center point of the bounding box
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Scale the bounding box by a factor
    pub fn scale(&self, factor: f32) -> Self {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height / 2.0;
        let new_width = self.width * factor;
        let new_height = self.height * factor;

        Self {
            x: center_x - new_width / 2.0,
            y: center_y - new_height / 2.0,
            width: new_width,
            height: new_height,
        }
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}
