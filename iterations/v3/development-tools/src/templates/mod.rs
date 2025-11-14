//! Code templates for rapid development
//!
//! Provides standardized templates for common Rust code patterns
//! to ensure consistency and accelerate development.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Template manager for loading and rendering code templates
pub struct TemplateManager {
    templates: HashMap<String, String>,
}

impl TemplateManager {
    /// Create a new template manager and load available templates
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut templates = HashMap::new();

        // Load lib.rs template
        if let Ok(content) = fs::read_to_string("src/templates/lib.rs.template") {
            templates.insert("lib.rs".to_string(), content);
        }

        // Load mod.rs template
        if let Ok(content) = fs::read_to_string("src/templates/mod.rs.template") {
            templates.insert("mod.rs".to_string(), content);
        }

        // Load types.rs template
        if let Ok(content) = fs::read_to_string("src/templates/types.rs.template") {
            templates.insert("types.rs".to_string(), content);
        }

        Ok(Self { templates })
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&String> {
        self.templates.get(name)
    }

    /// Render a template with variable substitution
    pub fn render_template(&self, name: &str, variables: &HashMap<&str, &str>) -> Option<String> {
        self.get_template(name).map(|template| {
            let mut result = template.clone();
            for (key, value) in variables {
                let placeholder = format!("{{{{{}}}}}", key);
                result = result.replace(&placeholder, value);
            }
            result
        })
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_manager_creation() {
        let manager = TemplateManager::new().unwrap();
        // Should at least have some templates loaded
        assert!(!manager.templates.is_empty());
    }

    #[test]
    fn test_template_rendering() {
        let manager = TemplateManager::new().unwrap();

        if let Some(rendered) =
            manager.render_template("lib.rs", &HashMap::from([("crate_name", "test_crate")]))
        {
            assert!(rendered.contains("test_crate"));
        }
    }
}
