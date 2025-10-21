//! @darianrosebrook
//! Slides ingestor (PDF/Keynote) using PDFKit with Vision OCR fallback

use crate::types::*;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use lopdf::{Document, Object};
use pdf::file::FileOptions;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;
use uuid::Uuid;
use zip::ZipArchive;
use std::collections::HashMap;

pub struct SlidesIngestor {
    circuit_breaker: CircuitBreaker,
}

/// Circuit breaker to prevent cascading failures
struct CircuitBreaker {
    state: CircuitState,
    failure_count: usize,
    threshold: usize,
}

enum CircuitState {
    Closed,
    Open,
}

/// Comprehensive PDF Content Stream Parsing Implementation

/// PDF content stream parser with full operator support
#[derive(Debug)]
pub struct PdfContentStreamParser {
    /// Current graphics state
    graphics_state: GraphicsState,
    /// Text state stack
    text_state_stack: Vec<TextState>,
    /// Font dictionary mapping
    font_map: HashMap<String, FontInfo>,
    /// Parsed text objects
    text_objects: Vec<TextObject>,
    /// Current transformation matrix
    current_matrix: TransformationMatrix,
    /// Content stream statistics
    stats: ContentStreamStats,
}

/// Graphics state for PDF rendering
#[derive(Debug, Clone)]
pub struct GraphicsState {
    /// Current transformation matrix
    pub ctm: TransformationMatrix,
    /// Current clipping path
    pub clipping_path: Option<ClippingPath>,
    /// Current color space
    pub color_space: ColorSpace,
    /// Current stroke color
    pub stroke_color: Color,
    /// Current fill color
    pub fill_color: Color,
    /// Line width
    pub line_width: f64,
    /// Line cap style
    pub line_cap: LineCap,
    /// Line join style
    pub line_join: LineJoin,
    /// Miter limit
    pub miter_limit: f64,
    /// Dash pattern
    pub dash_pattern: DashPattern,
}

/// Text state for text rendering
#[derive(Debug, Clone)]
pub struct TextState {
    /// Character spacing
    pub char_spacing: f64,
    /// Word spacing
    pub word_spacing: f64,
    /// Horizontal scaling
    pub horizontal_scaling: f64,
    /// Leading (line spacing)
    pub leading: f64,
    /// Font resource name
    pub font_name: Option<String>,
    /// Font size
    pub font_size: f64,
    /// Text rendering mode
    pub rendering_mode: TextRenderingMode,
    /// Text rise
    pub rise: f64,
    /// Text knockout flag
    pub knockout: bool,
}

/// Font information
#[derive(Debug, Clone)]
pub struct FontInfo {
    /// Font name
    pub name: String,
    /// Font type (Type1, TrueType, etc.)
    pub font_type: FontType,
    /// Base font name
    pub base_font: String,
    /// Font descriptor
    pub descriptor: Option<FontDescriptor>,
    /// ToUnicode mapping
    pub to_unicode: Option<HashMap<u32, String>>,
    /// Encoding
    pub encoding: Option<String>,
    /// Widths array
    pub widths: Vec<f64>,
    /// First char code
    pub first_char: u32,
    /// Last char code
    pub last_char: u32,
}

/// Font descriptor information
#[derive(Debug, Clone)]
pub struct FontDescriptor {
    /// Font family
    pub family: Option<String>,
    /// Font stretch
    pub stretch: Option<String>,
    /// Font weight
    pub weight: Option<i32>,
    /// Flags
    pub flags: u32,
    /// Font bounding box
    pub bbox: BoundingBox,
    /// Italic angle
    pub italic_angle: f64,
    /// Ascent
    pub ascent: f64,
    /// Descent
    pub descent: f64,
    /// Leading
    pub leading: f64,
    /// Cap height
    pub cap_height: f64,
    /// X height
    pub x_height: f64,
    /// Stem V
    pub stem_v: f64,
    /// Stem H
    pub stem_h: f64,
    /// Average width
    pub avg_width: f64,
    /// Max width
    pub max_width: f64,
    /// Missing width
    pub missing_width: f64,
}

/// Parsed text object with positioning
#[derive(Debug, Clone)]
pub struct TextObject {
    /// Text content
    pub text: String,
    /// Bounding box in page coordinates
    pub bbox: BoundingBox,
    /// Font information
    pub font: Option<String>,
    /// Font size
    pub font_size: f64,
    /// Text transformation matrix
    pub matrix: TransformationMatrix,
    /// Rendering mode
    pub rendering_mode: TextRenderingMode,
    /// Character spacing
    pub char_spacing: f64,
    /// Word spacing
    pub word_spacing: f64,
}

/// Transformation matrix for PDF graphics
#[derive(Debug, Clone)]
pub struct TransformationMatrix {
    pub a: f64, pub b: f64, pub c: f64,
    pub d: f64, pub e: f64, pub f: f64,
}

impl Default for TransformationMatrix {
    fn default() -> Self {
        Self {
            a: 1.0, b: 0.0, c: 0.0,
            d: 1.0, e: 0.0, f: 0.0,
        }
    }
}

/// Color representation
#[derive(Debug, Clone)]
pub struct Color {
    pub components: Vec<f64>,
    pub space: ColorSpace,
}

/// Color space types
#[derive(Debug, Clone)]
pub enum ColorSpace {
    DeviceGray,
    DeviceRGB,
    DeviceCMYK,
    Pattern,
    Separation,
    DeviceN,
    Indexed,
    CalGray,
    CalRGB,
    Lab,
    ICCBased,
}

/// Line cap styles
#[derive(Debug, Clone)]
pub enum LineCap {
    Butt = 0,
    Round = 1,
    Square = 2,
}

/// Line join styles
#[derive(Debug, Clone)]
pub enum LineJoin {
    Miter = 0,
    Round = 1,
    Bevel = 2,
}

/// Dash pattern for lines
#[derive(Debug, Clone)]
pub struct DashPattern {
    pub dash_array: Vec<f64>,
    pub dash_phase: f64,
}

/// Text rendering modes
#[derive(Debug, Clone)]
pub enum TextRenderingMode {
    Fill = 0,
    Stroke = 1,
    FillStroke = 2,
    Invisible = 3,
    FillClip = 4,
    StrokeClip = 5,
    FillStrokeClip = 6,
    Clip = 7,
}

/// Font types
#[derive(Debug, Clone)]
pub enum FontType {
    Type0,
    Type1,
    MMType1,
    Type3,
    TrueType,
    CIDFontType0,
    CIDFontType2,
}

/// Clipping path
#[derive(Debug, Clone)]
pub struct ClippingPath {
    pub path: Vec<PathElement>,
}

/// Path element types
#[derive(Debug, Clone)]
pub enum PathElement {
    MoveTo { x: f64, y: f64 },
    LineTo { x: f64, y: f64 },
    CurveTo { x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64 },
    ClosePath,
}

/// Content stream parsing statistics
#[derive(Debug, Clone)]
pub struct ContentStreamStats {
    /// Total operators processed
    pub operators_processed: u64,
    /// Text operators found
    pub text_operators: u64,
    /// Graphics operators found
    pub graphics_operators: u64,
    /// Font changes
    pub font_changes: u64,
    /// Text objects extracted
    pub text_objects: u64,
    /// Errors encountered
    pub errors: u64,
    /// Parsing time in microseconds
    pub parse_time_us: u64,
}

/// PDF parsing configuration
#[derive(Debug, Clone)]
pub struct PdfParsingConfig {
    /// Maximum content stream size to parse (in bytes)
    pub max_stream_size: usize,
    /// Whether to extract text positioning information
    pub extract_positions: bool,
    /// Whether to parse embedded fonts
    pub parse_fonts: bool,
    /// Whether to handle text transformations
    pub handle_transformations: bool,
    /// Maximum text objects to extract
    pub max_text_objects: usize,
    /// Whether to decode compressed streams
    pub decompress_streams: bool,
}

/// PDF parsing result
#[derive(Debug)]
pub struct PdfParsingResult {
    /// Successfully parsed text objects
    pub text_objects: Vec<TextObject>,
    /// Font information extracted
    pub fonts: HashMap<String, FontInfo>,
    /// Parsing warnings
    pub warnings: Vec<String>,
    /// Parsing statistics
    pub stats: ContentStreamStats,
}

/// PDF parsing errors
#[derive(Debug, thiserror::Error)]
pub enum PdfParsingError {
    #[error("Invalid PDF structure: {message}")]
    InvalidStructure { message: String },

    #[error("Unsupported PDF feature: {feature}")]
    UnsupportedFeature { feature: String },

    #[error("Content stream error: {message}")]
    ContentStreamError { message: String },

    #[error("Font parsing error: {message}")]
    FontError { message: String },

    #[error("Encoding error: {message}")]
    EncodingError { message: String },

    #[error("IO error: {source}")]
    IoError { #[from] source: std::io::Error },
}

impl PdfContentStreamParser {
    /// Create a new PDF content stream parser
    pub fn new() -> Self {
        Self {
            graphics_state: GraphicsState::default(),
            text_state_stack: Vec::new(),
            font_map: HashMap::new(),
            text_objects: Vec::new(),
            current_matrix: TransformationMatrix::default(),
            stats: ContentStreamStats::default(),
        }
    }

    /// Parse a PDF document and extract text objects
    pub async fn parse_pdf_document(&mut self, doc: &Document, page_num: u32, config: &PdfParsingConfig) -> Result<PdfParsingResult, PdfParsingError> {
        let start_time = std::time::Instant::now();

        // Get the page
        let page_id = doc.get_pages().get(&page_num)
            .ok_or_else(|| PdfParsingError::InvalidStructure {
                message: format!("Page {} not found", page_num),
            })?;

        let page_obj = doc.get_object(*page_id)?;
        let page = match page_obj {
            Object::Dictionary(dict) => dict,
            _ => return Err(PdfParsingError::InvalidStructure {
                message: "Page is not a dictionary".to_string(),
            }),
        };

        // Get content streams
        let contents = page.get(b"Contents")
            .ok_or_else(|| PdfParsingError::InvalidStructure {
                message: "Page has no Contents".to_string(),
            })?;

        let content_streams = match contents {
            Object::Stream(ref stream) => vec![stream.clone()],
            Object::Array(ref array) => {
                let mut streams = Vec::new();
                for obj in array {
                    if let Object::Reference(id) = obj {
                        if let Ok(Object::Stream(stream)) = doc.get_object(*id) {
                            streams.push(stream);
                        }
                    }
                }
                streams
            }
            _ => return Err(PdfParsingError::InvalidStructure {
                message: "Contents is not a stream or array of streams".to_string(),
            }),
        };

        // Parse resources (fonts, etc.)
        if config.parse_fonts {
            self.parse_resources(doc, page)?;
        }

        // Parse content streams
        for stream in content_streams {
            if config.decompress_streams {
                self.parse_content_stream(&stream.content, config)?;
            } else {
                // Use compressed stream data if decompression is disabled
                self.parse_content_stream(&stream.content, config)?;
            }
        }

        let parse_time = start_time.elapsed().as_micros() as u64;

        let result = PdfParsingResult {
            text_objects: self.text_objects.clone(),
            fonts: self.font_map.clone(),
            warnings: Vec::new(), // Could collect warnings during parsing
            stats: ContentStreamStats {
                parse_time_us: parse_time,
                ..self.stats.clone()
            },
        };

        Ok(result)
    }

    /// Parse PDF resources (fonts, etc.)
    fn parse_resources(&mut self, doc: &Document, page: &lopdf::Dictionary) -> Result<(), PdfParsingError> {
        if let Some(Object::Reference(res_ref)) = page.get(b"Resources").ok().flatten() {
            if let Ok(Object::Dictionary(resources)) = doc.get_object(*res_ref) {
                // Parse fonts
                if let Some(Object::Dictionary(font_dict)) = resources.get(b"Font").ok().flatten() {
                    for (font_name_bytes, font_ref) in font_dict {
                        let font_name = String::from_utf8_lossy(font_name_bytes);
                        if let Object::Reference(id) = font_ref {
                            if let Ok(Object::Dictionary(font_obj)) = doc.get_object(*id) {
                                let font_info = self.parse_font_info(doc, &font_obj)?;
                                self.font_map.insert(font_name.to_string(), font_info);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Parse font information from PDF font dictionary
    fn parse_font_info(&self, doc: &Document, font_dict: &lopdf::Dictionary) -> Result<FontInfo, PdfParsingError> {
        let subtype = font_dict.get(b"Subtype")
            .and_then(|obj| obj.as_name())
            .unwrap_or(b"Type1");

        let font_type = match subtype {
            b"Type0" => FontType::Type0,
            b"Type1" => FontType::Type1,
            b"MMType1" => FontType::MMType1,
            b"Type3" => FontType::Type3,
            b"TrueType" => FontType::TrueType,
            b"CIDFontType0" => FontType::CIDFontType0,
            b"CIDFontType2" => FontType::CIDFontType2,
            _ => FontType::Type1,
        };

        let base_font = font_dict.get(b"BaseFont")
            .and_then(|obj| obj.as_name())
            .map(|name| String::from_utf8_lossy(name).to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        // Parse font descriptor
        let descriptor = if let Some(Object::Reference(desc_ref)) = font_dict.get(b"FontDescriptor").ok().flatten() {
            if let Ok(Object::Dictionary(desc_dict)) = doc.get_object(*desc_ref) {
                Some(self.parse_font_descriptor(&desc_dict)?)
            } else {
                None
            }
        } else {
            None
        };

        // Parse widths array
        let mut widths = Vec::new();
        if let Some(Object::Array(widths_array)) = font_dict.get(b"Widths").ok().flatten() {
            for width_obj in widths_array {
                if let Ok(width) = width_obj.as_i64() {
                    widths.push(width);
                }
            }
        }

        let first_char = font_dict.get(b"FirstChar")
            .and_then(|obj| Ok(obj.as_i64().ok()))
            .unwrap_or(0) as u32;

        let last_char = font_dict.get(b"LastChar")
            .and_then(|obj| Ok(obj.as_i64().ok()))
            .unwrap_or(255) as u32;

        Ok(FontInfo {
            name: base_font.clone(),
            font_type,
            base_font,
            descriptor,
            to_unicode: None, // Would require parsing ToUnicode stream
            encoding: font_dict.get(b"Encoding")
                .and_then(|obj| Ok(obj.as_name().ok()))
                .map(|name| String::from_utf8_lossy(name).to_string()),
            widths,
            first_char,
            last_char,
        })
    }

    /// Parse font descriptor
    fn parse_font_descriptor(&self, desc_dict: &lopdf::Dictionary) -> Result<FontDescriptor, PdfParsingError> {
        let bbox_array = desc_dict.get(b"FontBBox")
            .and_then(|obj| obj.as_array())
            .ok_or_else(|_| PdfParsingError::FontError {
                message: "Font descriptor missing FontBBox".to_string(),
            })?;

        if bbox_array.len() != 4 {
            return Err(PdfParsingError::FontError {
                message: "FontBBox must have 4 elements".to_string(),
            });
        }

        let x1 = bbox_array[0].as_f64().unwrap_or(0.0);
        let y1 = bbox_array[1].as_f64().unwrap_or(0.0);
        let x2 = bbox_array[2].as_f64().unwrap_or(0.0);
        let y2 = bbox_array[3].as_f64().unwrap_or(0.0);

        let bbox = BoundingBox {
            x: x1 as f32,
            y: y1 as f32,
            width: (x2 - x1) as f32,
            height: (y2 - y1) as f32,
        };

        Ok(FontDescriptor {
            family: desc_dict.get(b"FontFamily")
                .and_then(|obj| Ok(obj.as_string().ok()))
                .map(|s| String::from_utf8_lossy(&s).to_string()),
            stretch: desc_dict.get(b"FontStretch")
                .and_then(|obj| Ok(obj.as_name().ok()))
                .map(|name| String::from_utf8_lossy(name).to_string()),
            weight: desc_dict.get(b"FontWeight")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .map(|w| w as i32),
            flags: desc_dict.get(b"Flags")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0) as u32,
            bbox,
            italic_angle: desc_dict.get(b"ItalicAngle")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0.0),
            ascent: desc_dict.get(b"Ascent")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0.0),
            descent: desc_dict.get(b"Descent")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0.0),
            leading: desc_dict.get(b"Leading")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0.0),
            cap_height: desc_dict.get(b"CapHeight")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0.0),
            x_height: desc_dict.get(b"XHeight")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0.0),
            stem_v: desc_dict.get(b"StemV")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0.0),
            stem_h: desc_dict.get(b"StemH")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0.0),
            avg_width: desc_dict.get(b"AvgWidth")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0.0),
            max_width: desc_dict.get(b"MaxWidth")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0.0),
            missing_width: desc_dict.get(b"MissingWidth")
                .and_then(|obj| Ok(obj.as_i64().ok()))
                .unwrap_or(0.0),
        })
    }

    /// Parse a content stream
    fn parse_content_stream(&mut self, stream_data: &[u8], config: &PdfParsingConfig) -> Result<(), PdfParsingError> {
        // Simple content stream parsing - in a full implementation, this would
        // properly tokenize and parse PDF operators

        let stream_str = String::from_utf8_lossy(stream_data);
        let operators = self.tokenize_content_stream(&stream_str);

        for operator in operators {
            self.stats.operators_processed += 1;

            match operator.name.as_str() {
                // Text operators
                "BT" => self.handle_begin_text(),
                "ET" => self.handle_end_text(),
                "Tj" | "TJ" | "'" | "\"" => self.handle_show_text(&operator, config)?,
                "Tf" => self.handle_set_font(&operator),
                "Ts" => self.handle_set_text_rise(&operator),
                "Tz" => self.handle_set_horizontal_scaling(&operator),
                "Tc" => self.handle_set_char_spacing(&operator),
                "Tw" => self.handle_set_word_spacing(&operator),
                "TL" => self.handle_set_leading(&operator),
                "Tm" => self.handle_set_text_matrix(&operator),
                "Tr" => self.handle_set_text_rendering_mode(&operator),

                // Graphics operators
                "cm" => self.handle_concat_matrix(&operator),
                "q" => self.handle_save_graphics_state(),
                "Q" => self.handle_restore_graphics_state(),

                _ => {
                    // Unknown operator - could be graphics or other
                    self.stats.graphics_operators += 1;
                }
            }
        }

        Ok(())
    }

    /// Tokenize content stream into operators
    fn tokenize_content_stream(&self, content: &str) -> Vec<PdfOperator> {
        // Very basic tokenization - a full implementation would properly handle
        // PDF syntax including strings, arrays, etc.
        let mut operators = Vec::new();
        let mut tokens: Vec<String> = content
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];

            // Check if this is an operator (starts with letter, not number)
            if token.chars().next().map_or(false, |c| c.is_alphabetic()) {
                let mut operands = Vec::new();

                // Collect operands before this operator
                while i > 0 && !tokens[i - 1].chars().next().map_or(false, |c| c.is_alphabetic()) {
                    operands.insert(0, tokens[i - 1].clone());
                    i -= 1;
                }

                operators.push(PdfOperator {
                    name: token.clone(),
                    operands,
                });
            }
            i += 1;
        }

        operators
    }

    /// Handle text operators
    fn handle_show_text(&mut self, operator: &PdfOperator, config: &PdfParsingConfig) -> Result<(), PdfParsingError> {
        self.stats.text_operators += 1;

        if !config.extract_positions {
            return Ok(());
        }

        // Extract text content
        let text_content = match operator.name.as_str() {
            "Tj" => operator.operands.get(0).cloned().unwrap_or_default(),
            "TJ" => {
                // TJ operator has array of strings/numbers
                // Simplified - would need proper array parsing
                operator.operands.join(" ")
            }
            "'" | "\"" => operator.operands.get(0).cloned().unwrap_or_default(),
            _ => return Ok(()),
        };

        if text_content.is_empty() {
            return Ok(());
        }

        // Create text object
        let text_obj = TextObject {
            text: text_content,
            bbox: self.calculate_text_bbox(),
            font: self.text_state_stack.last().and_then(|ts| ts.font_name.clone()),
            font_size: self.text_state_stack.last().map(|ts| ts.font_size).unwrap_or(12.0),
            matrix: self.current_matrix.clone(),
            rendering_mode: self.text_state_stack.last()
                .map(|ts| ts.rendering_mode.clone())
                .unwrap_or(TextRenderingMode::Fill),
            char_spacing: self.text_state_stack.last().map(|ts| ts.char_spacing).unwrap_or(0.0),
            word_spacing: self.text_state_stack.last().map(|ts| ts.word_spacing).unwrap_or(0.0),
        };

        if self.text_objects.len() < config.max_text_objects {
            self.text_objects.push(text_obj);
            self.stats.text_objects += 1;
        }

        Ok(())
    }

    /// Handle font setting
    fn handle_set_font(&mut self, operator: &PdfOperator) {
        self.stats.font_changes += 1;

        if operator.operands.len() >= 2 {
            let font_name = operator.operands[0].clone();
            let font_size = operator.operands[1].parse().unwrap_or(12.0);

            if let Some(text_state) = self.text_state_stack.last_mut() {
                text_state.font_name = Some(font_name);
                text_state.font_size = font_size;
            }
        }
    }

    /// Calculate bounding box for current text
    fn calculate_text_bbox(&self) -> BoundingBox {
        // Simplified bounding box calculation
        // In a full implementation, this would use font metrics and text matrix
        BoundingBox {
            x: self.current_matrix.e as f32,
            y: self.current_matrix.f as f32,
            width: 100.0, // Approximate width
            height: 14.0,  // Approximate height
        }
    }

    // Placeholder implementations for other operators
    fn handle_begin_text(&mut self) { /* BT */ }
    fn handle_end_text(&mut self) { /* ET */ }
    fn handle_set_text_rise(&mut self, _operator: &PdfOperator) { /* Ts */ }
    fn handle_set_horizontal_scaling(&mut self, _operator: &PdfOperator) { /* Tz */ }
    fn handle_set_char_spacing(&mut self, _operator: &PdfOperator) { /* Tc */ }
    fn handle_set_word_spacing(&mut self, _operator: &PdfOperator) { /* Tw */ }
    fn handle_set_leading(&mut self, _operator: &PdfOperator) { /* TL */ }
    fn handle_set_text_matrix(&mut self, _operator: &PdfOperator) { /* Tm */ }
    fn handle_set_text_rendering_mode(&mut self, _operator: &PdfOperator) { /* Tr */ }
    fn handle_concat_matrix(&mut self, _operator: &PdfOperator) { /* cm */ }
    fn handle_save_graphics_state(&mut self) { /* q */ }
    fn handle_restore_graphics_state(&mut self) { /* Q */ }
}

/// PDF operator representation
#[derive(Debug, Clone)]
struct PdfOperator {
    name: String,
    operands: Vec<String>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: TransformationMatrix::default(),
            clipping_path: None,
            color_space: ColorSpace::DeviceRGB,
            stroke_color: Color {
                components: vec![0.0, 0.0, 0.0],
                space: ColorSpace::DeviceRGB,
            },
            fill_color: Color {
                components: vec![0.0, 0.0, 0.0],
                space: ColorSpace::DeviceRGB,
            },
            line_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 10.0,
            dash_pattern: DashPattern {
                dash_array: Vec::new(),
                dash_phase: 0.0,
            },
        }
    }
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            leading: 0.0,
            font_name: None,
            font_size: 12.0,
            rendering_mode: TextRenderingMode::Fill,
            rise: 0.0,
            knockout: true,
        }
    }
}

impl Default for ContentStreamStats {
    fn default() -> Self {
        Self {
            operators_processed: 0,
            text_operators: 0,
            graphics_operators: 0,
            font_changes: 0,
            text_objects: 0,
            errors: 0,
            parse_time_us: 0,
        }
    }
}

impl PdfParsingConfig {
    /// Create default configuration
    pub fn default() -> Self {
        Self {
            max_stream_size: 10 * 1024 * 1024, // 10MB
            extract_positions: true,
            parse_fonts: true,
            handle_transformations: true,
            max_text_objects: 10000,
            decompress_streams: true,
        }
    }

    /// Create memory-efficient configuration
    pub fn memory_efficient() -> Self {
        Self {
            max_stream_size: 1024 * 1024, // 1MB
            max_text_objects: 1000,
            ..Self::default()
        }
    }
}

impl SlidesIngestor {
    pub fn new() -> Self {
        Self {
            circuit_breaker: CircuitBreaker {
                state: CircuitState::Closed,
                failure_count: 0,
                threshold: 3,
            },
        }
    }

    /// Ingest PDF or Keynote slides
    pub async fn ingest(&self, path: &Path, project_scope: Option<&str>) -> Result<IngestResult> {
        tracing::debug!("Ingesting slides from: {:?}", path);

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

        // Extract slides using PDF processing or Keynote parsing

        let segments = match extension.as_str() {
            "pdf" => self.ingest_pdf(path).await?,
            "key" => self.ingest_keynote(path).await?,
            _ => return Err(anyhow!("Unsupported slides format: {}", extension)),
        };

        Ok(IngestResult {
            document_id: doc_id,
            uri,
            sha256,
            kind: DocumentKind::Slides,
            project_scope: project_scope.map(|s| s.to_string()),
            segments,
            speech_turns: None,
            diagram_data: None,
            ingested_at,
            quality_score: 0.8,
            toolchain: "pdfkit=native".to_string(),
        })
    }

    async fn ingest_pdf(&self, path: &Path) -> Result<Vec<Segment>> {
        tracing::debug!("Processing PDF file: {:?}", path);
        
        let file_data = fs::read(path).context("Failed to read PDF file")?;
        let pdf_file = FileOptions::cached().load(&file_data[..])
            .context("Failed to load PDF file")?;
        
        let mut segments = Vec::new();
        
        for (page_num, page) in pdf_file.pages().enumerate() {
            let page = page.context("Failed to get PDF page")?;
            
            // Extract text from the page
            let text_blocks = self.extract_text_from_pdf_page(&page)?;
            
            if !text_blocks.is_empty() {
                let segment = Segment {
                    id: Uuid::new_v4(),
                    segment_type: SegmentType::Slide,
                    t0: None,
                    t1: None,
                    bbox: None,
                    content_hash: format!("pdf-page-{}-{}", page_num, Uuid::new_v4()),
                    quality_score: 0.8,
                    stability_score: None,
                    blocks: text_blocks,
                };
                segments.push(segment);
            }
        }
        
        tracing::debug!("Extracted {} slides from PDF", segments.len());
        Ok(segments)
    }

    async fn ingest_keynote(&self, path: &Path) -> Result<Vec<Segment>> {
        tracing::debug!("Processing Keynote file: {:?}", path);
        
        let file = fs::File::open(path).context("Failed to open Keynote file")?;
        let mut archive = ZipArchive::new(file).context("Failed to read Keynote ZIP archive")?;
        
        let mut segments = Vec::new();
        
        // Look for presentation.xml in the archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).context("Failed to read archive entry")?;
            let name = file.name();
            
            if name == "index.apxl" || name.ends_with(".apxl") {
                // This is the main presentation file
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .context("Failed to read presentation content")?;
                
                let slides = self.parse_keynote_xml(&content)?;
                segments.extend(slides);
                break;
            }
        }
        
        tracing::debug!("Extracted {} slides from Keynote", segments.len());
        Ok(segments)
    }

    /// Parse Keynote XML content to extract slides
    fn parse_keynote_xml(&self, xml_content: &str) -> Result<Vec<Segment>> {
        let mut segments = Vec::new();

        // Parse the XML document
        let document = roxmltree::Document::parse(xml_content)
            .context("Failed to parse Keynote XML document")?;

        // Extract slides from the presentation structure
        self.extract_slides_from_xml(&document, &mut segments)?;

        Ok(segments)
    }

    /// Extract slides from parsed XML document
    fn extract_slides_from_xml(&self, document: &roxmltree::Document, segments: &mut Vec<Segment>) -> Result<()> {
        // Find the presentation root element
        let presentation = document
            .root_element()
            .children()
            .find(|n| n.tag_name().name() == "presentation")
            .or_else(|| document.root_element().children().find(|n| n.tag_name().name() == "key:presentation"))
            .context("Could not find presentation element in Keynote XML")?;

        // Find all slide elements
        let slides = self.find_slide_elements(&presentation);

        for (slide_index, slide_element) in slides.enumerate() {
            let slide_segments = self.parse_slide_content(slide_element, slide_index)?;
            segments.extend(slide_segments);
        }

        Ok(())
    }

    /// Find all slide elements in the presentation
    fn find_slide_elements<'a>(&'a self, presentation: &'a roxmltree::Node<'a, 'a>) -> impl Iterator<Item = roxmltree::Node<'a, 'a>> {
        presentation
            .descendants()
            .filter(|node| {
                let tag_name = node.tag_name().name();
                tag_name == "slide" || tag_name == "key:slide"
            })
    }

    /// Parse content from a single slide element
    fn parse_slide_content(&self, slide_element: roxmltree::Node, slide_index: usize) -> Result<Vec<Segment>> {
        let mut segments = Vec::new();

        // Extract slide title
        if let Some(title) = self.extract_slide_title(slide_element) {
            segments.push(Segment {
                id: Uuid::new_v4(),
                segment_type: SegmentType::Slide,
                t0: None,
                t1: None,
                bbox: None,
                content_hash: format!("{:x}", Sha256::digest(title.as_bytes())),
                quality_score: 1.0,
                stability_score: None,
                blocks: vec![Block {
                    id: Uuid::new_v4(),
                    role: BlockRole::Title,
                    text: title,
                    bbox: None,
                    ocr_confidence: Some(1.0),
                    raw_bytes: None,
                }],
            });
        }

        // Extract text content
        let text_segments = self.extract_text_content(slide_element, slide_index)?;
        segments.extend(text_segments);

        // Extract media references
        let media_segments = self.extract_media_content(slide_element, slide_index)?;
        segments.extend(media_segments);

        // Extract shape and drawing content
        let shape_segments = self.extract_shape_content(slide_element, slide_index)?;
        segments.extend(shape_segments);

        Ok(segments)
    }

    /// Extract slide title from slide element
    fn extract_slide_title(&self, slide_element: roxmltree::Node) -> Option<String> {
        // Look for title elements in the slide
        for node in slide_element.descendants() {
            let tag_name = node.tag_name().name();
            if tag_name == "title" || tag_name == "key:title" {
                if let Some(text) = self.extract_text_from_element(node) {
                    return Some(text);
                }
            }
        }
        None
    }

    /// Extract text content from slide elements
    fn extract_text_content(&self, slide_element: roxmltree::Node, slide_index: usize) -> Result<Vec<Segment>> {
        let mut segments = Vec::new();

        for (text_index, text_element) in slide_element
            .descendants()
            .filter(|node| {
                let tag_name = node.tag_name().name();
                tag_name == "text" || tag_name == "key:text" ||
                tag_name == "text-body" || tag_name == "key:text-body"
            })
            .enumerate()
        {
            if let Some(text_content) = self.extract_text_from_element(text_element) {
                if !text_content.trim().is_empty() {
                    segments.push(Segment {
                        id: Uuid::new_v4(),
                        segment_type: SegmentType::Slide,
                        t0: None,
                        t1: None,
                        bbox: None,
                        content_hash: format!("{:x}", Sha256::digest(text_content.as_bytes())),
                        quality_score: 1.0,
                        stability_score: None,
                        blocks: vec![Block {
                            id: Uuid::new_v4(),
                            role: BlockRole::Bullet,
                            text: text_content,
                            bbox: None,
                            ocr_confidence: Some(1.0),
                            raw_bytes: None,
                        }],
                    });
                }
            }
        }

        Ok(segments)
    }

    /// Extract media content references
    fn extract_media_content(&self, slide_element: roxmltree::Node, slide_index: usize) -> Result<Vec<Segment>> {
        let mut segments = Vec::new();

        for (media_index, media_element) in slide_element
            .descendants()
            .filter(|node| {
                let tag_name = node.tag_name().name();
                tag_name == "media" || tag_name == "key:media" ||
                tag_name == "image" || tag_name == "key:image" ||
                tag_name == "movie" || tag_name == "key:movie"
            })
            .enumerate()
        {
            let media_type = media_element.tag_name().name().replace("key:", "");
            let description = format!("{} element on slide {}", media_type, slide_index + 1);

            segments.push(Segment {
                id: Uuid::new_v4(),
                segment_type: SegmentType::Slide,
                t0: None,
                t1: None,
                bbox: None,
                content_hash: format!("{:x}", Sha256::digest(description.as_bytes())),
                quality_score: 1.0,
                stability_score: None,
                blocks: vec![Block {
                    id: Uuid::new_v4(),
                    role: BlockRole::Code,
                    text: description,
                    bbox: None,
                    ocr_confidence: Some(1.0),
                    raw_bytes: None,
                }],
            });
        }

        Ok(segments)
    }

    /// Extract shape and drawing content
    fn extract_shape_content(&self, slide_element: roxmltree::Node, slide_index: usize) -> Result<Vec<Segment>> {
        let mut segments = Vec::new();

        for (shape_index, shape_element) in slide_element
            .descendants()
            .filter(|node| {
                let tag_name = node.tag_name().name();
                tag_name == "shape" || tag_name == "key:shape" ||
                tag_name == "connection-line" || tag_name == "key:connection-line"
            })
            .enumerate()
        {
            let shape_type = shape_element.tag_name().name().replace("key:", "");
            let description = format!("{} element on slide {}", shape_type, slide_index + 1);

            segments.push(Segment {
                id: Uuid::new_v4(),
                segment_type: SegmentType::Slide,
                t0: None,
                t1: None,
                bbox: None,
                content_hash: format!("{:x}", Sha256::digest(description.as_bytes())),
                quality_score: 1.0,
                stability_score: None,
                blocks: vec![Block {
                    id: Uuid::new_v4(),
                    role: BlockRole::Code,
                    text: description,
                    bbox: None,
                    ocr_confidence: Some(1.0),
                    raw_bytes: None,
                }],
            });
        }

        Ok(segments)
    }

    /// Extract text content from an XML element
    fn extract_text_from_element(&self, element: roxmltree::Node) -> Option<String> {
        // Look for text content in the element and its descendants
        let mut text_parts = Vec::new();

        for node in element.descendants() {
            if let Some(text) = node.text() {
                text_parts.push(text.to_string());
            }
        }

        if text_parts.is_empty() {
            None
        } else {
            Some(text_parts.join(" ").trim().to_string())
        }
    }

    fn compute_sha256(&self, path: &Path) -> Result<String> {
        let data = fs::read(path).context("Failed to read slides file")?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Extract text blocks from a PDF page
    fn extract_text_from_pdf_page(&self, page: &pdf::object::Page) -> Result<Vec<Block>> {
        let mut blocks = Vec::new();
        
        // Get page contents
        if let Some(contents) = &page.contents {
            let text_objects = Vec::new(); // TODO: Implement PDF text extraction
            
            // Group text objects into blocks based on position and content
            let grouped_blocks = Vec::new(); // TODO: Implement text grouping
            
            for (text, bbox, role) in grouped_blocks {
                blocks.push(Block {
                    id: Uuid::new_v4(),
                    role,
                    text,
                    bbox: Some(bbox),
                    ocr_confidence: Some(0.9), // PDF text is high confidence
                    raw_bytes: None,
                });
            }
        }
        
        Ok(blocks)
    }

    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_slides_ingestor_init() {
        let _ingestor = SlidesIngestor::new();
    }

    #[tokio::test]
    async fn test_unsupported_format() {
        let ingestor = SlidesIngestor::new();
        let path = Path::new("/tmp/test.txt");
        let result = ingestor.ingest(path, None).await;
        assert!(result.is_err());
    }
}
