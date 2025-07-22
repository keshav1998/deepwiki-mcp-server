//! MCP Bridge Module
//!
//! This module provides a bridge between Zed's STDIO-based MCP client and
//! HTTP-based DeepWiki/Devin MCP servers. It translates JSON-RPC messages
//! from stdin/stdout to HTTP requests/responses.

pub mod auth;
pub mod client;
pub mod protocol;
pub mod server;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Configuration for the MCP bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// The endpoint URL for the MCP server
    pub endpoint: String,
    /// Protocol to use (sse or http)
    pub protocol: String,
    /// Optional API key for authenticated endpoints
    pub api_key: Option<String>,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Enable debug logging
    pub debug: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://mcp.deepwiki.com".to_string(),
            protocol: "sse".to_string(),
            api_key: None,
            timeout_seconds: 60,
            debug: false,
        }
    }
}

/// Errors that can occur in the MCP bridge
#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Authentication failed: {message}")]
    Auth { message: String },

    #[error("Protocol error: {message}")]
    Protocol { message: String },

    #[error("Server error: {code} - {message}")]
    Server { code: i32, message: String },

    #[error("Timeout: {message}")]
    Timeout { message: String },

    #[error("Configuration error: {message}")]
    Config { message: String },
}

/// Result type for bridge operations
pub type BridgeResult<T> = Result<T, BridgeError>;

/// Session information for MCP connections
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Unique session ID
    pub id: String,
    /// Endpoint URL
    pub endpoint: String,
    /// Authentication headers
    pub headers: HashMap<String, String>,
    /// Session creation timestamp
    pub created_at: std::time::Instant,
}

impl SessionInfo {
    /// Create a new session
    pub fn new(endpoint: String, api_key: Option<String>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Accept".to_string(), "application/json".to_string());

        if let Some(key) = api_key {
            headers.insert("Authorization".to_string(), format!("Bearer {key}"));
        }

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            endpoint,
            headers,
            created_at: std::time::Instant::now(),
        }
    }

    /// Get session age in seconds
    pub fn age_seconds(&self) -> u64 {
        self.created_at.elapsed().as_secs()
    }
}

/// MCP protocol constants
pub mod constants {
    pub const JSONRPC_VERSION: &str = "2.0";
    pub const MCP_VERSION: &str = "2024-11-05";

    // Method names
    pub const METHOD_INITIALIZE: &str = "initialize";
    pub const METHOD_INITIALIZED: &str = "notifications/initialized";
    pub const METHOD_TOOLS_LIST: &str = "tools/list";
    pub const METHOD_TOOLS_CALL: &str = "tools/call";
    pub const METHOD_RESOURCES_LIST: &str = "resources/list";
    pub const METHOD_RESOURCES_READ: &str = "resources/read";
    pub const METHOD_PROMPTS_LIST: &str = "prompts/list";
    pub const METHOD_PROMPTS_GET: &str = "prompts/get";

    // Error codes
    pub const ERROR_PARSE: i32 = -32700;
    pub const ERROR_INVALID_REQUEST: i32 = -32600;
    pub const ERROR_METHOD_NOT_FOUND: i32 = -32601;
    pub const ERROR_INVALID_PARAMS: i32 = -32602;
    pub const ERROR_INTERNAL: i32 = -32603;
}

/// Utility functions
pub mod utils {
    use super::*;

    /// Create a standard JSON-RPC error response
    pub fn create_error_response(
        id: Option<serde_json::Value>,
        code: i32,
        message: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": constants::JSONRPC_VERSION,
            "id": id,
            "error": {
                "code": code,
                "message": message
            }
        })
    }

    /// Extract method name from JSON-RPC request
    pub fn extract_method(request: &serde_json::Value) -> Option<&str> {
        request.get("method")?.as_str()
    }

    /// Extract request ID from JSON-RPC request
    pub fn extract_id(request: &serde_json::Value) -> Option<serde_json::Value> {
        request.get("id").cloned()
    }

    /// Check if request is a notification (no ID)
    pub fn is_notification(request: &serde_json::Value) -> bool {
        !request
            .as_object()
            .map_or(false, |obj| obj.contains_key("id"))
    }
}
