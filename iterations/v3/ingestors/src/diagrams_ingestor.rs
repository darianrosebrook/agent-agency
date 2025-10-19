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
        // For now, we'll create a simple edge
        // In a real implementation, you would analyze the line coordinates
        // and determine which entities it connects
        
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
    fn generate_entity_name(&self, entity_type: &str, attributes: &HashMap<String, String>) -> String {
        // Try to get an ID or class attribute
        if let Some(id) = attributes.get("id") {
            format!("{}:{}", entity_type, id)
        } else if let Some(class) = attributes.get("class") {
            format!("{}:{}", entity_type, class)
        } else {
            format!("{}:{}", entity_type, Uuid::new_v4().to_string()[..8].to_string())
        }
    }

    /// Render SVG to PNG (placeholder implementation)
    fn render_svg_to_png(&self, _svg_content: &str) -> Result<Vec<u8>> {
        // In a real implementation, you would use a library like resvg or usvg
        // to render the SVG to PNG format
        
        // For now, return a placeholder PNG
        let width = 800;
        let height = 600;
        let mut png_data = Vec::new();
        
        // Create a simple PNG header (this is a minimal implementation)
        png_data.extend_from_slice(&[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        ]);
        
        // Add minimal PNG data (this is just a placeholder)
        png_data.extend_from_slice(&[0x00; 1000]); // Placeholder data
        
        Ok(png_data)
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
        let src_entity = entities.iter().find(|e| e.attributes.get("id") == Some(&source_id));
        let dst_entity = entities.iter().find(|e| e.attributes.get("id") == Some(&target_id));
        
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

    /// Render GraphML to PNG (placeholder implementation)
    fn render_graphml_to_png(&self, _entities: &[DiagramEntity], _edges: &[DiagramEdge]) -> Result<Vec<u8>> {
        // In a real implementation, you would use a graph visualization library
        // to render the graph structure to PNG format
        
        // For now, return a placeholder PNG
        let mut png_data = Vec::new();
        
        // Create a simple PNG header
        png_data.extend_from_slice(&[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        ]);
        
        // Add minimal PNG data (this is just a placeholder)
        png_data.extend_from_slice(&[0x00; 1000]); // Placeholder data
        
        Ok(png_data)
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
