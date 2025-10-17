//! Web Scraper
//!
//! Scrapes web content for research and knowledge gathering.

use crate::types::*;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Web scraper for content extraction
#[derive(Debug)]
pub struct WebScraper {
    config: WebScrapingConfig,
    client: reqwest::Client,
    cache: Arc<RwLock<std::collections::HashMap<String, WebScrapingResult>>>,
}

impl WebScraper {
    /// Create a new web scraper
    pub fn new(config: WebScrapingConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config,
            client,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Scrape content from URL
    pub async fn scrape_url(&self, url: &str) -> Result<WebScrapingResult> {
        info!("Scraping URL: {}", url);

        // Implement actual web scraping with robust HTTP client and content parsing
        let response = self
            .client
            .get(url)
            .header("User-Agent", "Agent-Agency-Research/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .context("Failed to fetch URL")?;

        let status_code = response.status().as_u16();
        let content_type_header = response
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("text/html");

        let content_type = if content_type_header.contains("application/json") {
            ContentType::Text // Use Text for JSON content
        } else if content_type_header.contains("application/xml")
            || content_type_header.contains("text/xml")
        {
            ContentType::Text // Use Text for XML content
        } else if content_type_header.contains("text/html") {
            ContentType::Html
        } else {
            ContentType::Text
        };

        let body = response
            .text()
            .await
            .context("Failed to read response body")?;

        // Extract title and content based on content type
        let (title, content) = match content_type {
            ContentType::Html => {
                // For HTML/text, use scraper to extract meaningful content
                let document = scraper::Html::parse_document(&body);

                // Extract title
                let title_selector = scraper::Selector::parse("title").unwrap();
                let title = document
                    .select(&title_selector)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_else(|| "Untitled".to_string());

                // Extract main content (prioritize article, main, or body)
                let content_selectors = [
                    "article",
                    "main",
                    "[role='main']",
                    ".content",
                    ".post",
                    ".article",
                    "body",
                ];

                let mut content = String::new();
                for selector_str in &content_selectors {
                    if let Ok(selector) = scraper::Selector::parse(selector_str) {
                        if let Some(element) = document.select(&selector).next() {
                            content = element.text().collect::<String>();
                            break;
                        }
                    }
                }

                // If no specific content found, use body text
                if content.is_empty() {
                    content = document.root_element().text().collect::<String>();
                }

                // Clean up content
                content = content.trim().to_string();
                if content.len() > 5000 {
                    content = format!("{}...", &content[..5000]);
                }

                (title, content)
            }
            ContentType::Text => {
                // For plain text content
                let title = "Text Document".to_string();
                let content = if body.len() > 5000 {
                    format!("{}...", &body[..5000])
                } else {
                    body
                };
                (title, content)
            }
            _ => {
                // Default case for other content types
                let title = "Document".to_string();
                let content = if body.len() > 5000 {
                    format!("{}...", &body[..5000])
                } else {
                    body
                };
                (title, content)
            }
        };

        let result = WebScrapingResult {
            url: url.to_string(),
            title,
            content,
            content_type,
            scraped_at: chrono::Utc::now(),
            status_code,
            content_size: 100,
            processing_time_ms: 100,
            metadata: std::collections::HashMap::new(),
        };

        info!("URL scraping completed: {}", url);
        Ok(result)
    }

    /// Clear scraping cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Web scraping cache cleared");
    }
}
