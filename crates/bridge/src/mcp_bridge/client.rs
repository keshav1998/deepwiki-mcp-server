//! HTTP Client for DeepWiki/Devin MCP Servers
//!
//! This module provides an HTTP client that communicates with hosted MCP servers
//! and translates between JSON-RPC over HTTP and the standard MCP protocol.

use super::{
    auth::AuthManager,
    protocol::{
        CallToolParams, InitializeParams, InitializeResponse, McpRequest, McpResponse, Tool,
        ToolCallResponse, ToolsListResponse,
    },
    BridgeError, BridgeResult, SessionInfo,
};
use reqwest::{Client, Response};
use serde_json::Value;

use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// HTTP client for MCP server communication
pub struct McpHttpClient {
    client: Client,
    session: SessionInfo,
    auth_manager: AuthManager,
    timeout: Duration,
}

impl McpHttpClient {
    /// Create a new HTTP client
    pub fn new(
        endpoint: &str,
        api_key: Option<String>,
        timeout_seconds: u64,
    ) -> BridgeResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .user_agent("zed-deepwiki-mcp/0.1.0")
            .build()?;

        let session = SessionInfo::new(endpoint.to_string(), api_key.clone());
        let auth_manager = AuthManager::new(api_key);

        Ok(Self {
            client,
            session,
            auth_manager,
            timeout: Duration::from_secs(timeout_seconds),
        })
    }

    /// Get the session information
    #[allow(dead_code)]
    pub fn session(&self) -> &SessionInfo {
        &self.session
    }

    /// Send an MCP request to the HTTP endpoint
    pub async fn send_request(&mut self, request: McpRequest) -> BridgeResult<McpResponse> {
        let url = self.build_url(&request);
        let headers = self.auth_manager.get_headers(&self.session)?;

        debug!("Sending MCP request to {}: {}", url, request.method);

        let http_request = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .build()?;

        let response = timeout(self.timeout, self.client.execute(http_request))
            .await
            .map_err(|_| BridgeError::Timeout {
                message: format!("Request timed out after {} seconds", self.timeout.as_secs()),
            })?
            .map_err(BridgeError::Http)?;

        self.handle_response(response, &request).await
    }

    /// Handle initialization request
    pub async fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> BridgeResult<InitializeResponse> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(1.into())),
            method: "initialize".to_string(),
            params: Some(serde_json::to_value(params)?),
        };

        let response = self.send_request(request).await?;

        if let Some(result) = response.result {
            let init_response: InitializeResponse = serde_json::from_value(result)?;
            info!("Successfully initialized MCP session: {}", self.session.id);
            Ok(init_response)
        } else if let Some(error) = response.error {
            Err(BridgeError::Server {
                code: error.code,
                message: error.message,
            })
        } else {
            Err(BridgeError::Protocol {
                message: "Invalid initialize response".to_string(),
            })
        }
    }

    /// Send notification (no response expected)
    pub async fn send_notification(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> BridgeResult<()> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: None, // Notifications don't have IDs
            method: method.to_string(),
            params,
        };

        let url = self.build_url(&request);
        let headers = self.auth_manager.get_headers(&self.session)?;

        debug!("Sending MCP notification: {}", method);

        let _response = timeout(
            self.timeout,
            self.client
                .post(&url)
                .headers(headers)
                .json(&request)
                .send(),
        )
        .await
        .map_err(|_| BridgeError::Timeout {
            message: format!(
                "Notification timed out after {} seconds",
                self.timeout.as_secs()
            ),
        })?
        .map_err(BridgeError::Http)?;

        Ok(())
    }

    /// List available tools
    pub async fn list_tools(&mut self) -> BridgeResult<Vec<Tool>> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(2.into())),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = self.send_request(request).await?;

        if let Some(result) = response.result {
            let tools_response: ToolsListResponse = serde_json::from_value(result)?;
            Ok(tools_response.tools)
        } else if let Some(error) = response.error {
            Err(BridgeError::Server {
                code: error.code,
                message: error.message,
            })
        } else {
            Err(BridgeError::Protocol {
                message: "Invalid tools/list response".to_string(),
            })
        }
    }

    /// Call a tool
    pub async fn call_tool(
        &mut self,
        name: &str,
        arguments: Option<Value>,
    ) -> BridgeResult<ToolCallResponse> {
        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(3.into())),
            method: "tools/call".to_string(),
            params: Some(serde_json::to_value(params)?),
        };

        let response = self.send_request(request).await?;

        if let Some(result) = response.result {
            let tool_response: ToolCallResponse = serde_json::from_value(result)?;
            Ok(tool_response)
        } else if let Some(error) = response.error {
            Err(BridgeError::Server {
                code: error.code,
                message: error.message,
            })
        } else {
            Err(BridgeError::Protocol {
                message: "Invalid tools/call response".to_string(),
            })
        }
    }

    /// Build the appropriate URL for the request
    fn build_url(&self, _request: &McpRequest) -> String {
        let base_url = &self.session.endpoint;

        // Handle different protocols and endpoints
        if base_url.contains("mcp.deepwiki.com") || base_url.contains("mcp.devin.ai") {
            // Both DeepWiki and Devin use SSE endpoints
            format!("{base_url}/sse")
        } else {
            // Custom endpoint
            format!("{base_url}/mcp")
        }
    }

    /// Handle HTTP response and convert to MCP response
    async fn handle_response(
        &self,
        response: Response,
        _request: &McpRequest,
    ) -> BridgeResult<McpResponse> {
        let status = response.status();

        if !status.is_success() {
            error!(
                "HTTP error {}: {}",
                status,
                status.canonical_reason().unwrap_or("Unknown")
            );
            return Err(BridgeError::Server {
                code: i32::from(status.as_u16()),
                message: format!("HTTP error: {status}"),
            });
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        // Handle different response types
        if content_type.starts_with("text/event-stream") {
            // Server-Sent Events response
            self.handle_sse_response(response).await
        } else if content_type.starts_with("application/json") {
            // Regular JSON response
            self.handle_json_response(response).await
        } else {
            // Fallback: try to parse as JSON
            warn!(
                "Unexpected content type: {}, attempting JSON parse",
                content_type
            );
            self.handle_json_response(response).await
        }
    }

    /// Handle Server-Sent Events response
    async fn handle_sse_response(&self, response: Response) -> BridgeResult<McpResponse> {
        let text = response.text().await?;

        // Parse SSE format - look for data: lines
        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "ping" {
                    continue; // Skip ping messages
                }

                // Try to parse as JSON
                if let Ok(value) = serde_json::from_str::<Value>(data) {
                    return Ok(McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id: value.get("id").cloned(),
                        result: Some(value),
                        error: None,
                    });
                }
            }
        }

        Err(BridgeError::Protocol {
            message: "No valid data found in SSE response".to_string(),
        })
    }

    /// Handle regular JSON response
    async fn handle_json_response(&self, response: Response) -> BridgeResult<McpResponse> {
        let json: Value = response.json().await?;

        // Check if it's already a proper JSON-RPC response
        if json.get("jsonrpc").is_some() {
            Ok(serde_json::from_value(json)?)
        } else {
            // Wrap raw response in JSON-RPC format
            Ok(McpResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: Some(json),
                error: None,
            })
        }
    }

    /// Check if the session is still valid
    #[allow(dead_code)]
    pub fn is_session_valid(&self) -> bool {
        // Sessions are valid for 1 hour
        self.session.age_seconds() < 3600
    }

    /// Refresh the session if needed
    #[allow(dead_code)]
    pub fn refresh_session_if_needed(&mut self) {
        if !self.is_session_valid() {
            info!("Session expired, creating new session");
            let api_key = self.auth_manager.get_api_key().cloned();
            self.session = SessionInfo::new(self.session.endpoint.clone(), api_key);
        }
    }
}

/// Builder for `McpHttpClient`
pub struct McpHttpClientBuilder {
    endpoint: Option<String>,
    api_key: Option<String>,
    timeout_seconds: u64,
}

impl Default for McpHttpClientBuilder {
    fn default() -> Self {
        Self {
            endpoint: None,
            api_key: None,
            timeout_seconds: 60,
        }
    }
}

impl McpHttpClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    pub fn api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key;
        self
    }

    pub fn timeout_seconds(mut self, timeout: u64) -> Self {
        self.timeout_seconds = timeout;
        self
    }

    pub fn build(self) -> BridgeResult<McpHttpClient> {
        let endpoint = self.endpoint.ok_or_else(|| BridgeError::Config {
            message: "Endpoint is required".to_string(),
        })?;

        McpHttpClient::new(&endpoint, self.api_key, self.timeout_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder() {
        let client = McpHttpClientBuilder::new()
            .endpoint("https://mcp.deepwiki.com".to_string())
            .timeout_seconds(30)
            .build();

        assert!(client.is_ok());
    }

    #[test]
    fn test_url_building() {
        let client = McpHttpClient::new("https://mcp.deepwiki.com", None, 60).unwrap();

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(1.into())),
            method: "initialize".to_string(),
            params: None,
        };

        let url = client.build_url(&request);
        assert_eq!(url, "https://mcp.deepwiki.com/sse");
    }
}
