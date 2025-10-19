//! @darianrosebrook
//! Diagrams ingestor (SVG/GraphML) with graph extraction

use crate::types::*;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use roxmltree::Document;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub struct DiagramsIngestor {
    svg_parser: SvgParser,
    graphml_parser: GraphMLParser,
}

pub struct SvgParser;
pub struct GraphMLParser;

impl DiagramsIngestor {
    pub fn new() -> Self {
        Self {
            svg_parser: SvgParser,
            graphml_parser: GraphMLParser,
        }
    }

    /// Ingest diagram file (SVG/GraphML)
    pub async fn ingest(&self, path: &Path, project_scope: Option<&str>) -> Result<IngestResult> {
        tracing::debug!("Ingesting diagram from: {:?}", path);

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

        // Parse diagram and build semantic graph structure

        let diagram_data = match extension.as_str() {
            "svg" => self.parse_svg(path).await?,
            "graphml" => self.parse_graphml(path).await?,
            _ => return Err(anyhow!("Unsupported diagram format: {}", extension)),
        };

        let segment = Segment {
            id: Uuid::new_v4(),
            segment_type: SegmentType::Diagram,
            t0: None,
            t1: None,
            bbox: None,
            content_hash: sha256.clone(),
            quality_score: 0.85,
            stability_score: None,
            blocks: vec![Block {
                id: Uuid::new_v4(),
                role: BlockRole::Figure,
                text: format!("Diagram with {} nodes", diagram_data.entities.len()),
                bbox: None,
                ocr_confidence: None,
                raw_bytes: diagram_data.render_png.clone(),
            }],
        };

        Ok(IngestResult {
            document_id: doc_id,
            uri,
            sha256,
            kind: DocumentKind::Diagram,
            project_scope: project_scope.map(|s| s.to_string()),
            segments: vec![segment],
            speech_turns: None,
            diagram_data: Some(diagram_data),
            ingested_at,
            quality_score: 0.85,
            toolchain: "svg/graphml=native".to_string(),
        })
    }

    async fn parse_svg(&self, path: &Path) -> Result<DiagramData> {
        tracing::debug!("Parsing SVG file: {:?}", path);
        
        let content = fs::read_to_string(path).context("Failed to read SVG file")?;
        let doc = Document::parse(&content).context("Failed to parse SVG XML")?;
        
        let mut entities = Vec::new();
        let mut edges = Vec::new();
        
        // Parse SVG elements
        self.parse_svg_elements(&doc.root(), &mut entities, &mut edges)?;
        
        // Generate PNG render (placeholder)
        let render_png = self.render_svg_to_png(&content)?;
        
        Ok(DiagramData {
            entities,
            edges,
            render_png: Some(render_png),
        })
    }

    /// Parse SVG elements recursively
    fn parse_svg_elements(
        &self,
        node: &roxmltree::Node,
        entities: &mut Vec<DiagramEntity>,
        edges: &mut Vec<DiagramEdge>,
    ) -> Result<()> {
        match node.tag_name().name() {
            "circle" | "rect" | "ellipse" | "polygon" | "path" => {
                // These are shape elements that can represent nodes
                let entity = self.create_entity_from_svg_element(node)?;
                entities.push(entity);
            }
            "line" | "polyline" => {
                // These are line elements that can represent edges
                if let Some(edge) = self.create_edge_from_svg_element(node, entities)? {
                    edges.push(edge);
                }
            }
            "text" => {
                // Text elements can be labels
                let entity = self.create_text_entity_from_svg_element(node)?;
                entities.push(entity);
            }
            _ => {}
        }
        
        // Recursively process child elements
        for child in node.children() {
            if child.is_element() {
                self.parse_svg_elements(&child, entities, edges)?;
            }
        }
        
        Ok(())
    }

    /// Create a diagram entity from an SVG element
    fn create_entity_from_svg_element(&self, node: &roxmltree::Node) -> Result<DiagramEntity> {
        let mut attributes = HashMap::new();
        
        // Extract attributes
        for attr in node.attributes() {
            attributes.insert(attr.name().to_string(), serde_json::Value::String(attr.value().to_string()));
        }
        
        // Determine entity type based on tag name
        let entity_type = match node.tag_name().name() {
            "circle" | "ellipse" => "circle",
            "rect" => "rectangle",
            "polygon" => "polygon",
            "path" => "path",
            _ => "shape",
        };
        
        // Generate a normalized name
        let normalized_name = self.generate_entity_name(&entity_type, &attributes);
        
        Ok(DiagramEntity {
            id: Uuid::new_v4(),
            entity_type: entity_type.to_string(),
            normalized_name,
            attributes,
        })
    }

    /// Create a text entity from an SVG text element
    fn create_text_entity_from_svg_element(&self, node: &roxmltree::Node) -> Result<DiagramEntity> {
        let text_content = node.text().unwrap_or("").to_string();
        let mut attributes = HashMap::new();
        
        // Extract attributes
        for attr in node.attributes() {
            attributes.insert(attr.name().to_string(), attr.value().into());
        }
        
        Ok(DiagramEntity {
            id: Uuid::new_v4(),
            entity_type: "text".to_string(),
            normalized_name: text_content.clone(),
            attributes,
        })
    }

    /// Create an edge from an SVG line element
    fn create_edge_from_svg_element(
        &self,
        node: &roxmltree::Node,
        entities: &[DiagramEntity],
    ) -> Result<Option<DiagramEdge>> {
        // TODO: Implement proper edge analysis from line coordinates and entity connections
        // - [ ] Analyze SVG line/path coordinates to determine connection points
        // - [ ] Implement entity proximity detection for connection inference
        // - [ ] Support different connector types (straight lines, curves, arrows)
        // - [ ] Add edge directionality and cardinality detection
        // - [ ] Implement edge labeling and metadata extraction
        // - [ ] Support edge styling (colors, thickness, patterns)
        // - [ ] Add edge validation and consistency checking
        
        if entities.len() >= 2 {
            Ok(Some(DiagramEdge {
                id: Uuid::new_v4(),
                src: entities[0].id,
                dst: entities[1].id,
                label: None,
            }))
        } else {
            Ok(None)
        }
    }

    /// Generate a normalized entity name
    fn generate_entity_name(&self, entity_type: &str, attributes: &HashMap<String, serde_json::Value>) -> String {
        // Try to get an ID or class attribute
        if let Some(id) = attributes.get("id") {
            format!("{}:{}", entity_type, id)
        } else if let Some(class) = attributes.get("class") {
            format!("{}:{}", entity_type, class)
        } else {
            format!("{}:{}", entity_type, Uuid::new_v4().to_string()[..8].to_string())
        }
    }

    /// Render SVG to PNG using image crate
    fn render_svg_to_png(&self, svg_content: &str) -> Result<Vec<u8>> {
        // Parse SVG content to extract dimensions and basic structure
        let doc = roxmltree::Document::parse(svg_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse SVG: {}", e))?;
        
        // Extract viewBox or width/height from SVG
        let (width, height) = self.extract_svg_dimensions(&doc)?;
        
        // Create a simple PNG representation of the SVG
        let mut img = image::ImageBuffer::new(width, height);
        
        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = image::Rgb([255, 255, 255]);
        }
        
        // Parse and render basic SVG elements
        self.render_svg_elements(&doc, &mut img)?;
        
        // Convert to PNG bytes
        let mut png_data = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut png_data);
            img.write_to(&mut cursor, image::ImageFormat::Png)
                .map_err(|e| anyhow::anyhow!("Failed to write PNG: {}", e))?;
        }
        
        Ok(png_data)
    }
    
    /// Extract SVG dimensions from viewBox or width/height attributes
    fn extract_svg_dimensions(&self, doc: &roxmltree::Document) -> Result<(u32, u32)> {
        if let Some(svg_element) = doc.descendants().find(|n| n.tag_name().name() == "svg") {
            // Try to get viewBox first
            if let Some(viewbox) = svg_element.attribute("viewBox") {
                let parts: Vec<&str> = viewbox.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let (Ok(w), Ok(h)) = (parts[2].parse::<f32>(), parts[3].parse::<f32>()) {
                        return Ok((w as u32, h as u32));
                    }
                }
            }
            
            // Fallback to width/height attributes
            let width = svg_element.attribute("width")
                .and_then(|w| w.parse::<f32>().ok())
                .unwrap_or(800.0) as u32;
            let height = svg_element.attribute("height")
                .and_then(|h| h.parse::<f32>().ok())
                .unwrap_or(600.0) as u32;
            
            return Ok((width, height));
        }
        
        // Default dimensions if no SVG element found
        Ok((800, 600))
    }
    
    /// Render basic SVG elements to the image
    fn render_svg_elements(&self, doc: &roxmltree::Document, img: &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Result<()> {
        for node in doc.descendants() {
            match node.tag_name().name() {
                "rect" => self.render_rect(&node, img)?,
                "circle" => self.render_circle(&node, img)?,
                "line" => self.render_line(&node, img)?,
                "text" => self.render_text(&node, img)?,
                _ => {
                    // TODO: Implement comprehensive SVG element support instead of skipping
                    // - [ ] Add support for circle, ellipse, polygon, and polyline elements
                    // - [ ] Implement path element parsing and rendering
                    // - [ ] Add support for gradient fills and strokes
                    // - [ ] Implement clip-path and mask support
                    // - [ ] Add transform matrix support for all elements
                    // - [ ] Support SVG groups and nested transformations
                    // - [ ] Add CSS styling and class-based rendering
                    // TODO: Implement comprehensive SVG element processing beyond basic shapes
                    // - [ ] Add support for circle, ellipse, polygon, and polyline elements
                    // - [ ] Implement path element parsing and rendering
                    // - [ ] Add support for gradient fills and strokes
                    // - [ ] Implement clip-path and mask support
                    // - [ ] Add transform matrix support for all elements
                    // - [ ] Support SVG groups and nested transformations
                    // - [ ] Add CSS styling and class-based rendering
                }
            }
        }
        Ok(())
    }
    
    /// Render SVG rectangle element
    fn render_rect(&self, node: &roxmltree::Node, img: &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Result<()> {
        let x = node.attribute("x").and_then(|x| x.parse::<f32>().ok()).unwrap_or(0.0) as u32;
        let y = node.attribute("y").and_then(|y| y.parse::<f32>().ok()).unwrap_or(0.0) as u32;
        let width = node.attribute("width").and_then(|w| w.parse::<f32>().ok()).unwrap_or(100.0) as u32;
        let height = node.attribute("height").and_then(|h| h.parse::<f32>().ok()).unwrap_or(100.0) as u32;
        
        // TODO: Implement comprehensive SVG color parsing instead of simplified version
        // - [ ] Support CSS color names, hex codes, and RGB/RGBA values
        // - [ ] Implement HSL/HSLA color space support
        // - [ ] Add support for color gradients and patterns
        // - [ ] Support CSS custom properties and color functions
        // - [ ] Implement color opacity and alpha channel handling
        // - [ ] Add color interpolation for animations
        // - [ ] Support ICC color profiles and color management
        // TODO: Implement comprehensive SVG color parsing with CSS support
        // - [ ] Support CSS color names, hex codes, and RGB/RGBA values
        // - [ ] Implement HSL/HSLA color space support
        // - [ ] Add support for currentColor and inherit keywords
        // - [ ] Implement CSS color functions (rgb(), hsl(), etc.)
        // - [ ] Add color interpolation for animations
        // - [ ] Support ICC color profiles and color management
        // - [ ] Implement color validation and fallback handling
        let color = self.parse_color(node.attribute("fill").unwrap_or("black"));
        
        // Draw rectangle
        for py in y..(y + height).min(img.height()) {
            for px in x..(x + width).min(img.width()) {
                *img.get_pixel_mut(px, py) = color;
            }
        }
        
        Ok(())
    }
    
    /// Render SVG circle element
    fn render_circle(&self, node: &roxmltree::Node, img: &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Result<()> {
        let cx = node.attribute("cx").and_then(|x| x.parse::<f32>().ok()).unwrap_or(0.0);
        let cy = node.attribute("cy").and_then(|y| y.parse::<f32>().ok()).unwrap_or(0.0);
        let r = node.attribute("r").and_then(|r| r.parse::<f32>().ok()).unwrap_or(50.0);
        
        let color = self.parse_color(node.attribute("fill").unwrap_or("black"));
        
        // Draw circle using midpoint circle algorithm
        let center_x = cx as i32;
        let center_y = cy as i32;
        let radius = r as i32;
        
        for y in (center_y - radius)..=(center_y + radius) {
            for x in (center_x - radius)..=(center_x + radius) {
                if x >= 0 && y >= 0 && x < img.width() as i32 && y < img.height() as i32 {
                    let dx = x - center_x;
                    let dy = y - center_y;
                    if dx * dx + dy * dy <= radius * radius {
                        *img.get_pixel_mut(x as u32, y as u32) = color;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Render SVG line element
    fn render_line(&self, node: &roxmltree::Node, img: &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Result<()> {
        let x1 = node.attribute("x1").and_then(|x| x.parse::<f32>().ok()).unwrap_or(0.0) as u32;
        let y1 = node.attribute("y1").and_then(|y| y.parse::<f32>().ok()).unwrap_or(0.0) as u32;
        let x2 = node.attribute("x2").and_then(|x| x.parse::<f32>().ok()).unwrap_or(100.0) as u32;
        let y2 = node.attribute("y2").and_then(|y| y.parse::<f32>().ok()).unwrap_or(100.0) as u32;
        
        let color = self.parse_color(node.attribute("stroke").unwrap_or("black"));
        
        // Draw line using Bresenham's algorithm
        self.draw_line(img, x1, y1, x2, y2, color);
        
        Ok(())
    }
    
    /// TODO: Implement proper SVG text rendering instead of simplified rectangle placeholder
    /// - [ ] Integrate with font rendering libraries (freetype, rusttype, etc.)
    /// - [ ] Support different font families, sizes, and weights
    /// - [ ] Implement text alignment (left, center, right, justify)
    /// - [ ] Add support for text decorations (underline, strikethrough)
    /// - [ ] Support multi-line text and text wrapping
    /// - [ ] Implement text path following and curved text
    /// - [ ] Add Unicode and international text support
    fn render_text(&self, node: &roxmltree::Node, img: &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Result<()> {
        let x = node.attribute("x").and_then(|x| x.parse::<f32>().ok()).unwrap_or(0.0) as u32;
        let y = node.attribute("y").and_then(|y| y.parse::<f32>().ok()).unwrap_or(0.0) as u32;
        
        let color = self.parse_color(node.attribute("fill").unwrap_or("black"));
        let text = node.text().unwrap_or("Text");
        
        // TODO: Replace rectangle placeholder with actual font rendering
        // - [ ] Load and render TrueType/OpenType fonts
        // - [ ] Implement glyph rasterization and anti-aliasing
        // - [ ] Support font kerning and ligature rendering
        // - [ ] Add subpixel rendering for better text quality
        // - [ ] Implement font caching for performance
        // - [ ] Support emoji and symbol font rendering
        // - [ ] Add text layout and line breaking algorithms
        // TODO: Implement proper font rendering instead of rectangle placeholder
        // - [ ] Integrate with font rendering libraries (freetype, rusttype, etc.)
        // - [ ] Support different font families, sizes, and weights
        // - [ ] Implement glyph rasterization and anti-aliasing
        // - [ ] Add font kerning and ligature rendering
        // - [ ] Support subpixel rendering for better text quality
        // - [ ] Implement font caching for performance
        // - [ ] Support emoji and symbol font rendering
        for py in y..(y + 20).min(img.height()) {
            for px in x..(x + text.len() as u32 * 8).min(img.width()) {
                *img.get_pixel_mut(px, py) = color;
            }
        }
        
        Ok(())
    }
    
    /// Parse color string to RGB
    fn parse_color(&self, color_str: &str) -> image::Rgb<u8> {
        match color_str {
            "red" => image::Rgb([255, 0, 0]),
            "green" => image::Rgb([0, 255, 0]),
            "blue" => image::Rgb([0, 0, 255]),
            "black" => image::Rgb([0, 0, 0]),
            "white" => image::Rgb([255, 255, 255]),
            "gray" | "grey" => image::Rgb([128, 128, 128]),
            _ => {
                // Try to parse hex color
                if color_str.starts_with('#') && color_str.len() == 7 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&color_str[1..3], 16),
                        u8::from_str_radix(&color_str[3..5], 16),
                        u8::from_str_radix(&color_str[5..7], 16),
                    ) {
                        return image::Rgb([r, g, b]);
                    }
                }
                image::Rgb([0, 0, 0]) // Default to black
            }
        }
    }
    
    /// Draw a line using Bresenham's algorithm
    fn draw_line(&self, img: &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>, x1: u32, y1: u32, x2: u32, y2: u32, color: image::Rgb<u8>) {
        let mut x1 = x1 as i32;
        let mut y1 = y1 as i32;
        let x2 = x2 as i32;
        let y2 = y2 as i32;
        
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        
        loop {
            if x1 >= 0 && y1 >= 0 && x1 < img.width() as i32 && y1 < img.height() as i32 {
                *img.get_pixel_mut(x1 as u32, y1 as u32) = color;
            }
            
            if x1 == x2 && y1 == y2 {
                break;
            }
            
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x1 += sx;
            }
            if e2 < dx {
                err += dx;
                y1 += sy;
            }
        }
    }

    async fn parse_graphml(&self, path: &Path) -> Result<DiagramData> {
        tracing::debug!("Parsing GraphML file: {:?}", path);
        
        let content = fs::read_to_string(path).context("Failed to read GraphML file")?;
        let doc = Document::parse(&content).context("Failed to parse GraphML XML")?;
        
        let mut entities = Vec::new();
        let mut edges = Vec::new();
        
        // Parse GraphML elements
        self.parse_graphml_elements(&doc.root(), &mut entities, &mut edges)?;
        
        // Generate PNG render (placeholder)
        let render_png = self.render_graphml_to_png(&entities, &edges)?;
        
        Ok(DiagramData {
            entities,
            edges,
            render_png: Some(render_png),
        })
    }

    /// Parse GraphML elements
    fn parse_graphml_elements(
        &self,
        node: &roxmltree::Node,
        entities: &mut Vec<DiagramEntity>,
        edges: &mut Vec<DiagramEdge>,
    ) -> Result<()> {
        match node.tag_name().name() {
            "node" => {
                let entity = self.create_entity_from_graphml_node(node)?;
                entities.push(entity);
            }
            "edge" => {
                let edge = self.create_edge_from_graphml_edge(node, entities)?;
                edges.push(edge);
            }
            _ => {}
        }
        
        // Recursively process child elements
        for child in node.children() {
            if child.is_element() {
                self.parse_graphml_elements(&child, entities, edges)?;
            }
        }
        
        Ok(())
    }

    /// Create a diagram entity from a GraphML node
    fn create_entity_from_graphml_node(&self, node: &roxmltree::Node) -> Result<DiagramEntity> {
        let mut attributes = HashMap::new();
        
        // Extract node ID
        let node_id = node.attribute("id").unwrap_or("unknown").to_string();
        
        // Extract attributes
        for attr in node.attributes() {
            attributes.insert(attr.name().to_string(), attr.value().into());
        }
        
        // Look for label in child elements
        let mut normalized_name = node_id.clone();
        for child in node.children() {
            if child.is_element() && child.tag_name().name() == "data" {
                if let Some(key) = child.attribute("key") {
                    if key == "label" || key == "d0" {
                        if let Some(text) = child.text() {
                            normalized_name = text.to_string();
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(DiagramEntity {
            id: Uuid::new_v4(),
            entity_type: "node".to_string(),
            normalized_name,
            attributes,
        })
    }

    /// Create an edge from a GraphML edge element
    fn create_edge_from_graphml_edge(
        &self,
        node: &roxmltree::Node,
        entities: &[DiagramEntity],
    ) -> Result<DiagramEdge> {
        let source_id = node.attribute("source").unwrap_or("").to_string();
        let target_id = node.attribute("target").unwrap_or("").to_string();
        
        // Find the corresponding entity IDs
        let src_entity = entities.iter().find(|e| e.attributes.get("id") == Some(&serde_json::Value::String(source_id.clone())));
        let dst_entity = entities.iter().find(|e| e.attributes.get("id") == Some(&serde_json::Value::String(target_id.clone())));
        
        let src_id = src_entity.map(|e| e.id).unwrap_or_else(Uuid::new_v4);
        let dst_id = dst_entity.map(|e| e.id).unwrap_or_else(Uuid::new_v4);
        
        // Look for edge label
        let mut label = None;
        for child in node.children() {
            if child.is_element() && child.tag_name().name() == "data" {
                if let Some(key) = child.attribute("key") {
                    if key == "label" || key == "d0" {
                        if let Some(text) = child.text() {
                            label = Some(text.to_string());
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(DiagramEdge {
            id: Uuid::new_v4(),
            src: src_id,
            dst: dst_id,
            label,
        })
    }

    /// Render GraphML to PNG using image crate
    fn render_graphml_to_png(&self, entities: &[DiagramEntity], edges: &[DiagramEdge]) -> Result<Vec<u8>> {
        // Calculate canvas dimensions based on entity positions
        let (width, height) = self.calculate_graph_dimensions(entities);
        
        // Create image buffer
        let mut img = image::ImageBuffer::new(width, height);
        
        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = image::Rgb([255, 255, 255]);
        }
        
        // Render edges first (so they appear behind nodes)
        for edge in edges {
            self.render_graphml_edge(&edge, &mut img)?;
        }
        
        // Render entities (nodes)
        for entity in entities {
            self.render_graphml_entity(&entity, &mut img)?;
        }
        
        // Convert to PNG bytes
        let mut png_data = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut png_data);
            img.write_to(&mut cursor, image::ImageFormat::Png)
                .map_err(|e| anyhow::anyhow!("Failed to write PNG: {}", e))?;
        }
        
        Ok(png_data)
    }
    
    /// Calculate graph dimensions based on entity positions
    fn calculate_graph_dimensions(&self, entities: &[DiagramEntity]) -> (u32, u32) {
        if entities.is_empty() {
            return (800, 600);
        }
        
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        
        for entity in entities {
            // Extract position from attributes
            if let Some(x_val) = entity.attributes.get("x") {
                if let Some(x) = x_val.as_f64() {
                    let x = x as f32;
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                }
            }
            if let Some(y_val) = entity.attributes.get("y") {
                if let Some(y) = y_val.as_f64() {
                    let y = y as f32;
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
        }
        
        // Add padding and ensure minimum size
        let padding = 50.0;
        let width = ((max_x - min_x + padding * 2.0).max(400.0)) as u32;
        let height = ((max_y - min_y + padding * 2.0).max(300.0)) as u32;
        
        (width, height)
    }
    
    /// Render a GraphML entity (node) to the image
    fn render_graphml_entity(&self, entity: &DiagramEntity, img: &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Result<()> {
        // Get position from attributes
        let x = entity.attributes.get("x")
            .and_then(|x| x.as_f64())
            .unwrap_or(100.0) as u32;
        let y = entity.attributes.get("y")
            .and_then(|y| y.as_f64())
            .unwrap_or(100.0) as u32;
        
        // Get size from attributes
        let width = entity.attributes.get("width")
            .and_then(|w| w.as_f64())
            .unwrap_or(100.0) as u32;
        let height = entity.attributes.get("height")
            .and_then(|h| h.as_f64())
            .unwrap_or(50.0) as u32;
        
        // Get color from attributes
        let color = entity.attributes.get("color")
            .and_then(|c| c.as_str())
            .map(|c| self.parse_color(c))
            .unwrap_or(image::Rgb([100, 150, 200])); // Default blue
        
        // Draw rectangle for the node
        for py in y..(y + height).min(img.height()) {
            for px in x..(x + width).min(img.width()) {
                *img.get_pixel_mut(px, py) = color;
            }
        }
        
        // Draw border
        let border_color = image::Rgb([0, 0, 0]);
        // Top and bottom borders
        for px in x..(x + width).min(img.width()) {
            if y < img.height() {
                *img.get_pixel_mut(px, y) = border_color;
            }
            if y + height - 1 < img.height() {
                *img.get_pixel_mut(px, y + height - 1) = border_color;
            }
        }
        // Left and right borders
        for py in y..(y + height).min(img.height()) {
            if x < img.width() {
                *img.get_pixel_mut(x, py) = border_color;
            }
            if x + width - 1 < img.width() {
                *img.get_pixel_mut(x + width - 1, py) = border_color;
            }
        }
        
        Ok(())
    }
    
    /// Render a GraphML edge to the image
    fn render_graphml_edge(&self, edge: &DiagramEdge, img: &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Result<()> {
        // TODO: Implement proper GraphML edge rendering with actual entity positions
        // - [ ] Look up actual entity positions from parsed GraphML node coordinates
        // - [ ] Support different edge styles (straight, curved, orthogonal)
        // - [ ] Implement edge routing algorithms to avoid overlapping nodes
        // - [ ] Add edge labels and metadata display
        // - [ ] Support edge directionality with arrowheads
        // - [ ] Implement edge bundling for complex graphs
        // - [ ] Add edge styling (colors, thickness, dash patterns)
        
        // Use UUID bytes to generate deterministic positions
        let src_bytes = edge.src.as_bytes();
        let dst_bytes = edge.dst.as_bytes();
        
        let src_x = ((src_bytes[0] as u32) * 256 + (src_bytes[1] as u32)) % 700 + 50;
        let src_y = ((src_bytes[2] as u32) * 256 + (src_bytes[3] as u32)) % 500 + 50;
        let dst_x = ((dst_bytes[0] as u32) * 256 + (dst_bytes[1] as u32)) % 700 + 50;
        let dst_y = ((dst_bytes[2] as u32) * 256 + (dst_bytes[3] as u32)) % 500 + 50;
        
        let color = image::Rgb([128, 128, 128]); // Gray for edges
        
        // Draw line using Bresenham's algorithm
        self.draw_line(img, src_x, src_y, dst_x, dst_y, color);
        
        Ok(())
    }

    fn compute_sha256(&self, path: &Path) -> Result<String> {
        let data = fs::read(path).context("Failed to read diagram file")?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

impl Default for DiagramsIngestor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_diagrams_ingestor_init() {
        let _ingestor = DiagramsIngestor::new();
    }

    #[tokio::test]
    async fn test_unsupported_format() {
        let ingestor = DiagramsIngestor::new();
        let path = Path::new("/tmp/test.txt");
        let result = ingestor.ingest(path, None).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_svg_elements() {
        let ingestor = DiagramsIngestor::new();
        let svg_content = r#"
            <svg>
                <circle cx="50" cy="50" r="40" fill="red" />
                <rect x="100" y="100" width="80" height="60" fill="blue" />
                <text x="200" y="200">Test Label</text>
            </svg>
        "#;
        
        let doc = roxmltree::Document::parse(svg_content).unwrap();
        let mut entities = Vec::new();
        let mut edges = Vec::new();
        
        let result = ingestor.parse_svg_elements(&doc.root(), &mut entities, &mut edges);
        assert!(result.is_ok());
        assert_eq!(entities.len(), 3); // circle, rect, text
        assert_eq!(edges.len(), 0); // no edges in this SVG
    }

    #[test]
    fn test_parse_graphml_elements() {
        let ingestor = DiagramsIngestor::new();
        let graphml_content = r#"
            <graphml>
                <node id="n1">
                    <data key="label">Node 1</data>
                </node>
                <node id="n2">
                    <data key="label">Node 2</data>
                </node>
                <edge source="n1" target="n2">
                    <data key="label">Edge 1</data>
                </edge>
            </graphml>
        "#;
        
        let doc = roxmltree::Document::parse(graphml_content).unwrap();
        let mut entities = Vec::new();
        let mut edges = Vec::new();
        
        let result = ingestor.parse_graphml_elements(&doc.root(), &mut entities, &mut edges);
        assert!(result.is_ok());
        assert_eq!(entities.len(), 2); // two nodes
        assert_eq!(edges.len(), 1); // one edge
    }
}
