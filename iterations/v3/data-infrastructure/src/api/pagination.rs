//! Pagination utilities for API endpoints
//!
//! Provides offset-based and cursor-based pagination support for list endpoints.
//! Adapted from Open-WebUI patterns for Agent-Agency.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

/// Offset-based pagination parameters
///
/// Used for most list endpoints where total count is available.
/// Provides page-based navigation with configurable page size.
///
/// Query parameters:
/// - `page`: Page number (1-indexed, default: 1)
/// - `limit` or `per_page`: Items per page (default: 20, max: 100)
///
/// Example:
/// ```
/// GET /api/v1/tasks?page=2&limit=50
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,
    
    /// Items per page (alias: `per_page`)
    #[serde(alias = "per_page", default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
        }
    }
}

impl PaginationParams {
    /// Create pagination params from query string
    pub fn from_query(query: &HashMap<String, String>) -> Self {
        let page = query
            .get("page")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1)
            .max(1);
        
        let limit = query
            .get("limit")
            .or_else(|| query.get("per_page"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(20)
            .min(100)
            .max(1);
        
        Self { page, limit }
    }
    
    /// Calculate offset for database queries (0-indexed)
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.limit
    }
    
    /// Get limit for database queries
    pub fn limit(&self) -> u32 {
        self.limit
    }
    
    /// Get SQL LIMIT clause value
    pub fn sql_limit(&self) -> i64 {
        self.limit as i64
    }
    
    /// Get SQL OFFSET clause value
    pub fn sql_offset(&self) -> i64 {
        self.offset() as i64
    }
}

/// Cursor-based pagination parameters
///
/// Used for large datasets where offset-based pagination becomes inefficient.
/// Provides cursor-based navigation for better performance.
///
/// Query parameters:
/// - `cursor`: Opaque cursor string from previous response (optional)
/// - `limit`: Items per page (default: 20, max: 100)
///
/// Example:
/// ```
/// GET /api/v1/events?cursor=eyJpZCI6IjEyMyIsInRpbWVzdGFtcCI6IjIwMjQtMDEtMDEifQ&limit=50
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CursorPaginationParams {
    /// Cursor from previous response (base64-encoded JSON)
    pub cursor: Option<String>,
    
    /// Items per page
    #[serde(default = "default_limit")]
    pub limit: u32,
}

impl Default for CursorPaginationParams {
    fn default() -> Self {
        Self {
            cursor: None,
            limit: 20,
        }
    }
}

impl CursorPaginationParams {
    /// Create cursor pagination params from query string
    pub fn from_query(query: &HashMap<String, String>) -> Self {
        let cursor = query.get("cursor").cloned();
        
        let limit = query
            .get("limit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(20)
            .min(100)
            .max(1);
        
        Self { cursor, limit }
    }
    
    /// Get limit for database queries
    pub fn limit(&self) -> u32 {
        self.limit
    }
    
    /// Get SQL LIMIT clause value
    pub fn sql_limit(&self) -> i64 {
        self.limit as i64
    }
    
    /// Parse cursor to extract position information
    ///
    /// Returns (id, timestamp) tuple if cursor is valid
    pub fn parse_cursor(&self) -> Option<(String, Option<chrono::DateTime<chrono::Utc>>)> {
        self.cursor.as_ref().and_then(|c| {
            // Decode base64
            use base64::{Engine as _, engine::general_purpose};
            let decoded = general_purpose::STANDARD.decode(c).ok()?;
            let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
            
            let id = json.get("id")?.as_str()?.to_string();
            let timestamp = json.get("timestamp")
                .and_then(|t| t.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            
            Some((id, timestamp))
        })
    }
    
    /// Create cursor from position information
    pub fn create_cursor(id: &str, timestamp: Option<chrono::DateTime<chrono::Utc>>) -> String {
        use base64::{Engine as _, engine::general_purpose};
        
        let json = serde_json::json!({
            "id": id,
            "timestamp": timestamp.map(|t| t.to_rfc3339()),
        });
        
        let bytes = serde_json::to_vec(&json).unwrap_or_default();
        general_purpose::STANDARD.encode(&bytes)
    }
}

/// Paginated response wrapper
///
/// Standard response format for paginated endpoints.
/// Includes items, pagination metadata, and navigation helpers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PaginatedResponse<T> {
    /// Items for this page
    pub items: Vec<T>,
    
    /// Total number of items across all pages
    pub total: u64,
    
    /// Current page number (1-indexed)
    pub page: u32,
    
    /// Number of items per page
    pub per_page: u32,
    
    /// Total number of pages
    pub total_pages: u32,
    
    /// Whether there are more pages
    pub has_more: bool,
    
    /// Whether there is a previous page
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    /// Create paginated response from items and total count
    pub fn new(items: Vec<T>, total: u64, params: &PaginationParams) -> Self {
        let total_pages = (total as f64 / params.limit as f64).ceil() as u32;
        let has_more = params.page < total_pages;
        let has_prev = params.page > 1;
        
        Self {
            items,
            total,
            page: params.page,
            per_page: params.limit,
            total_pages,
            has_more,
            has_prev,
        }
    }
    
    /// Create empty paginated response
    pub fn empty(params: &PaginationParams) -> Self {
        Self {
            items: Vec::new(),
            total: 0,
            page: params.page,
            per_page: params.limit,
            total_pages: 0,
            has_more: false,
            has_prev: false,
        }
    }
}

/// Cursor-based paginated response wrapper
///
/// Response format for cursor-based pagination endpoints.
/// Includes items, next cursor, and navigation helpers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CursorPaginatedResponse<T> {
    /// Items for this page
    pub items: Vec<T>,
    
    /// Cursor for the next page (if more items available)
    pub next_cursor: Option<String>,
    
    /// Whether there are more items
    pub has_more: bool,
    
    /// Number of items returned
    pub count: u32,
}

impl<T> CursorPaginatedResponse<T> {
    /// Create cursor paginated response from items
    ///
    /// If items length equals limit, there may be more items.
    /// The last item's ID and timestamp are used to create the next cursor.
    pub fn new(
        items: Vec<T>,
        limit: u32,
        get_id: impl Fn(&T) -> String,
        get_timestamp: impl Fn(&T) -> Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        let count = items.len() as u32;
        let has_more = count == limit;
        
        let next_cursor = if has_more && !items.is_empty() {
            let last_item = items.last().unwrap();
            let id = get_id(last_item);
            let timestamp = get_timestamp(last_item);
            Some(CursorPaginationParams::create_cursor(&id, timestamp))
        } else {
            None
        };
        
        Self {
            items,
            next_cursor,
            has_more,
            count,
        }
    }
    
    /// Create empty cursor paginated response
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            next_cursor: None,
            has_more: false,
            count: 0,
        }
    }
}

/// Extract pagination params from Axum Query extractor
///
/// Helper function to extract PaginationParams from query string.
/// Can be used in handler functions.
pub fn extract_pagination(query: &HashMap<String, String>) -> PaginationParams {
    PaginationParams::from_query(query)
}

/// Extract cursor pagination params from Axum Query extractor
///
/// Helper function to extract CursorPaginationParams from query string.
/// Can be used in handler functions.
pub fn extract_cursor_pagination(query: &HashMap<String, String>) -> CursorPaginationParams {
    CursorPaginationParams::from_query(query)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pagination_params_default() {
        let params = PaginationParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.limit, 20);
        assert_eq!(params.offset(), 0);
    }
    
    #[test]
    fn test_pagination_params_from_query() {
        let mut query = HashMap::new();
        query.insert("page".to_string(), "3".to_string());
        query.insert("limit".to_string(), "50".to_string());
        
        let params = PaginationParams::from_query(&query);
        assert_eq!(params.page, 3);
        assert_eq!(params.limit, 50);
        assert_eq!(params.offset(), 100); // (3-1) * 50
    }
    
    #[test]
    fn test_pagination_params_per_page_alias() {
        let mut query = HashMap::new();
        query.insert("per_page".to_string(), "25".to_string());
        
        let params = PaginationParams::from_query(&query);
        assert_eq!(params.limit, 25);
    }
    
    #[test]
    fn test_pagination_params_limit_max() {
        let mut query = HashMap::new();
        query.insert("limit".to_string(), "200".to_string()); // Exceeds max of 100
        
        let params = PaginationParams::from_query(&query);
        assert_eq!(params.limit, 100); // Clamped to max
    }
    
    #[test]
    fn test_paginated_response() {
        let items = vec![1, 2, 3, 4, 5];
        let params = PaginationParams { page: 2, limit: 3 };
        let response = PaginatedResponse::new(items, 10, &params);
        
        assert_eq!(response.items.len(), 5);
        assert_eq!(response.total, 10);
        assert_eq!(response.page, 2);
        assert_eq!(response.per_page, 3);
        assert_eq!(response.total_pages, 4); // ceil(10/3)
        assert!(response.has_more);
        assert!(response.has_prev);
    }
    
    #[test]
    fn test_cursor_pagination_create_parse() {
        let id = "test-id-123";
        let timestamp = chrono::Utc::now();
        
        let cursor = CursorPaginationParams::create_cursor(id, Some(timestamp));
        assert!(!cursor.is_empty());
        
        let params = CursorPaginationParams { cursor: Some(cursor), limit: 20 };
        let parsed = params.parse_cursor();
        
        assert!(parsed.is_some());
        let (parsed_id, parsed_timestamp) = parsed.unwrap();
        assert_eq!(parsed_id, id);
        assert!(parsed_timestamp.is_some());
    }
}

