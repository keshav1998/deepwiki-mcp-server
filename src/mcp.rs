//! Core MCP Protocol Handlers for DeepWiki Zed Extension
//!
//! This module will implement the essential Model Context Protocol logic required for:
//!   - /tools/list: Discovery and JSON schema advertisement of available tools
//!   - /tools/call: Execution, streaming, and result/error emission for tools
//!   - Payload shaping and argument validation per MCP spec
//!   - Integration points for authentication, config, and streaming
//!
//! Goals:
//!  - Provide strongly-typed Rust APIs for tool schema/response/annotation building
//!  - Facilitate ergonomic extension with new tools (resources, prompts, etc.)
//!  - Enable robust/isolated testing of all protocol-level handlers
//!
//! For details on the MCP tool structure and best practices, see the docs in
//! `.zed/.chat_context/zed-mcp-extension-project-plan.md` and modelcontextprotocol.io/specification.
//!
//! Typical usage: Called by the binary/command Zed launches for its context server integration.

/// Example structure for a tool definition.
/// Expand this as needed to support all MCP tool schema fields.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    #[serde(default)]
    pub annotations: Option<ToolAnnotations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAnnotations {
    pub title: Option<String>,
    pub readOnlyHint: Option<bool>,
    pub destructiveHint: Option<bool>,
    pub idempotentHint: Option<bool>,
    pub openWorldHint: Option<bool>,
}

// -- Stub Handlers -----------------------------------------------------------

/// Returns all available tools (for /tools/list)
pub async fn list_tools() -> Vec<ToolSchema> {
    vec![
        ToolSchema {
            name: "read_wiki_structure".to_string(),
            description: Some("Get a list of documentation topics for a GitHub repository".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "repoName": { "type": "string" }
                },
                "required": ["repoName"]
            }),
            annotations: Some(ToolAnnotations {
                title: Some("Read Wiki Structure".to_string()),
                readOnlyHint: Some(true),
                destructiveHint: None,
                idempotentHint: Some(true),
                openWorldHint: Some(false),
            }),
        },
        ToolSchema {
            name: "read_wiki_contents".to_string(),
            description: Some("View documentation about a GitHub repository".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "repoName": { "type": "string" }
                },
                "required": ["repoName"]
            }),
            annotations: Some(ToolAnnotations {
                title: Some("Read Wiki Contents".to_string()),
                readOnlyHint: Some(true),
                destructiveHint: None,
                idempotentHint: Some(true),
                openWorldHint: Some(false),
            }),
        },
        ToolSchema {
            name: "ask_question".to_string(),
            description: Some("Ask any question about a GitHub repository and get an AI-powered, context-grounded response".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "repoName": { "type": "string" },
                    "question": { "type": "string" }
                },
                "required": ["repoName", "question"]
            }),
            annotations: Some(ToolAnnotations {
                title: Some("Ask Question".to_string()),
                readOnlyHint: Some(true),
                destructiveHint: None,
                idempotentHint: None,
                openWorldHint: None,
            }),
        },
    ]
}

/// Executes a tool by name with the given arguments (for /tools/call)
pub async fn call_tool(
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, MCPError> {
    match tool_name {
        "read_wiki_structure" => {
            // Validate args
            let repo_name = arguments
                .get("repoName")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError::InvalidArgs("Missing or invalid repoName".to_string()))?;
            // Stub/mock response structure
            Ok(serde_json::json!({
                "topics": [
                    { "title": "Introduction", "path": "README.md" },
                    { "title": "Installation", "path": "docs/installation.md" },
                    { "title": "API Reference", "path": "docs/api.md" }
                ],
                "repo": repo_name
            }))
        }
        "read_wiki_contents" => {
            let repo_name = arguments
                .get("repoName")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError::InvalidArgs("Missing or invalid repoName".to_string()))?;
            // Stub/mock content
            Ok(serde_json::json!({
                "repo": repo_name,
                "content": "# Introduction\nWelcome to your repo docs.\n\n## Usage\nHow to use it goes here..."
            }))
        }
        "ask_question" => {
            let repo_name = arguments
                .get("repoName")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError::InvalidArgs("Missing or invalid repoName".to_string()))?;
            let question = arguments
                .get("question")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError::InvalidArgs("Missing or invalid question".to_string()))?;
            // Stub/mock answer and sources
            Ok(serde_json::json!({
                "answer": format!("Mock answer to '{}'", question),
                "sources": [
                    format!("https://github.com/{}/blob/main/README.md", repo_name)
                ]
            }))
        }
        _ => Err(MCPError::NotImplemented),
    }
}

/// Top-level error type for MCP tool/call handling
#[derive(Debug)]
pub enum MCPError {
    /// Tool not implemented in server.
    NotImplemented,
    /// Arguments validation failed.
    InvalidArgs(String),
    /// Internal processing error.
    Internal(String),
}

// -- Tests stubbed below -----------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_tools_empty() {
        let tools = list_tools().await;
        assert_eq!(tools.len(), 0);
    }

    #[tokio::test]
    async fn test_call_tool_not_implemented() {
        let result = call_tool("unknown_tool", serde_json::json!({})).await;
        assert!(matches!(result, Err(MCPError::NotImplemented)));
    }
}
