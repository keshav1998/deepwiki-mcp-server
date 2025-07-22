//! STDIO MCP Server Implementation
//!
//! This module provides a server that listens on stdin for JSON-RPC MCP messages,
//! forwards them to HTTP-based DeepWiki/Devin endpoints, and returns responses on stdout.

use super::{
    client::{McpHttpClient, McpHttpClientBuilder},
    protocol::{
        CallToolParams, InitializeParams, McpError, McpRequest, McpResponse, PromptsListResponse,
        ResourcesListResponse, ToolsListResponse,
    },
    BridgeConfig, BridgeError, BridgeResult,
};
use futures::stream::StreamExt;
use serde_json::Value;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{debug, error, info, warn};

/// STDIO MCP Server that bridges to HTTP endpoints
pub struct StdioMcpServer {
    config: BridgeConfig,
    client: Option<McpHttpClient>,
    initialized: bool,
}

impl StdioMcpServer {
    /// Create a new STDIO MCP server
    pub fn new(config: BridgeConfig) -> Self {
        Self {
            config,
            client: None,
            initialized: false,
        }
    }

    /// Start the server and begin processing messages
    pub async fn run(&mut self) -> BridgeResult<()> {
        info!(
            "Starting MCP STDIO server for endpoint: {}",
            self.config.endpoint
        );

        // Set up input stream from stdin
        let stdin = tokio::io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = FramedRead::new(reader, LinesCodec::new());

        // Set up output to stdout
        let mut stdout = tokio::io::stdout();

        // Main message processing loop
        while let Some(line_result) = lines.next().await {
            match line_result {
                Ok(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    debug!("Received line: {}", line);

                    match self.process_message(&line).await {
                        Ok(Some(response)) => {
                            let response_str = serde_json::to_string(&response)?;
                            debug!("Sending response: {}", response_str);

                            stdout.write_all(response_str.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                        Ok(None) => {
                            // Notification - no response needed
                            debug!("Processed notification, no response");
                        }
                        Err(e) => {
                            error!("Error processing message: {}", e);

                            // Try to parse the original message to get an ID for error response
                            let error_response =
                                if let Ok(request) = serde_json::from_str::<Value>(&line) {
                                    let id = request.get("id").cloned();
                                    Self::create_error_response(id, &e)
                                } else {
                                    Self::create_error_response(None, &e)
                                };

                            let error_str = serde_json::to_string(&error_response)?;
                            stdout.write_all(error_str.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }

        info!("MCP STDIO server shutting down");
        Ok(())
    }

    /// Process a single JSON-RPC message
    async fn process_message(&mut self, line: &str) -> BridgeResult<Option<McpResponse>> {
        // Parse the JSON-RPC request
        let request: McpRequest = serde_json::from_str(line)?;

        debug!("Processing method: {}", request.method);

        // Handle the request based on method
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await.map(Some),
            "notifications/initialized" => self.handle_initialized(request).await,
            "tools/list" => self.handle_tools_list(request).await.map(Some),
            "tools/call" => self.handle_tool_call(request).await.map(Some),
            "resources/list" => self.handle_resources_list(request).map(Some),
            "resources/read" => self.handle_resource_read(request).map(Some),
            "prompts/list" => self.handle_prompts_list(request).map(Some),
            "prompts/get" => self.handle_prompt_get(request).map(Some),
            _ => {
                warn!("Unknown method: {}", request.method);
                if request.id.is_some() {
                    Ok(Some(Self::create_method_not_found_response(
                        request.id,
                        &request.method,
                    )))
                } else {
                    Ok(None) // Ignore unknown notifications
                }
            }
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&mut self, request: McpRequest) -> BridgeResult<McpResponse> {
        if self.initialized {
            return Ok(Self::create_error_response(
                request.id,
                &BridgeError::Protocol {
                    message: "Already initialized".to_string(),
                },
            ));
        }

        // Create HTTP client
        let client = McpHttpClientBuilder::new()
            .endpoint(self.config.endpoint.clone())
            .api_key(self.config.api_key.clone())
            .timeout_seconds(self.config.timeout_seconds)
            .build()?;

        // Parse initialize params or use defaults
        let init_params = if let Some(params) = request.params {
            serde_json::from_value::<InitializeParams>(params)
                .unwrap_or_else(|_| InitializeParams::default())
        } else {
            InitializeParams::default()
        };

        // Store the client for future requests
        self.client = Some(client);

        // Initialize with the remote server
        let response = match self.client.as_mut().unwrap().initialize(init_params).await {
            Ok(init_response) => {
                self.initialized = true;
                info!("Successfully initialized MCP session");

                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::to_value(init_response)?),
                    error: None,
                }
            }
            Err(e) => {
                error!("Failed to initialize: {}", e);
                Self::create_error_response(request.id, &e)
            }
        };

        Ok(response)
    }

    /// Handle initialized notification
    async fn handle_initialized(
        &mut self,
        _request: McpRequest,
    ) -> BridgeResult<Option<McpResponse>> {
        if !self.initialized {
            warn!("Received initialized notification but not initialized");
            return Ok(None);
        }

        // Send initialized notification to remote server
        if let Some(client) = &mut self.client {
            client
                .send_notification("notifications/initialized", None)
                .await?;
        }

        debug!("Sent initialized notification to remote server");
        Ok(None) // Notifications don't return responses
    }

    /// Handle tools/list request
    async fn handle_tools_list(&mut self, request: McpRequest) -> BridgeResult<McpResponse> {
        self.ensure_initialized(&request)?;

        let client = self.client.as_mut().unwrap();

        match client.list_tools().await {
            Ok(tools) => {
                let response = ToolsListResponse {
                    tools,
                    next_cursor: None,
                };

                Ok(McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::to_value(response)?),
                    error: None,
                })
            }
            Err(e) => {
                error!("Failed to list tools: {}", e);
                Ok(Self::create_error_response(request.id, &e))
            }
        }
    }

    /// Handle tools/call request
    async fn handle_tool_call(&mut self, request: McpRequest) -> BridgeResult<McpResponse> {
        self.ensure_initialized(&request)?;

        let params = request.params.ok_or_else(|| BridgeError::Protocol {
            message: "Missing parameters for tool call".to_string(),
        })?;

        let call_params: CallToolParams = serde_json::from_value(params)?;
        let client = self.client.as_mut().unwrap();

        match client
            .call_tool(&call_params.name, call_params.arguments)
            .await
        {
            Ok(tool_response) => Ok(McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::to_value(tool_response)?),
                error: None,
            }),
            Err(e) => {
                error!("Failed to call tool '{}': {}", call_params.name, e);
                Ok(Self::create_error_response(request.id, &e))
            }
        }
    }

    /// Handle resources/list request
    fn handle_resources_list(&mut self, request: McpRequest) -> BridgeResult<McpResponse> {
        self.ensure_initialized(&request)?;

        // For now, return empty resources list
        // This can be extended to support actual resource listing from the remote server
        let response = ResourcesListResponse {
            resources: vec![],
            next_cursor: None,
        };

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::to_value(response)?),
            error: None,
        })
    }

    /// Handle resources/read request
    fn handle_resource_read(&mut self, request: McpRequest) -> BridgeResult<McpResponse> {
        self.ensure_initialized(&request)?;

        // For now, return error as resources are not implemented
        Ok(Self::create_error_response(
            request.id,
            &BridgeError::Protocol {
                message: "Resource reading not implemented".to_string(),
            },
        ))
    }

    /// Handle prompts/list request
    fn handle_prompts_list(&mut self, request: McpRequest) -> BridgeResult<McpResponse> {
        self.ensure_initialized(&request)?;

        // For now, return empty prompts list
        let response = PromptsListResponse {
            prompts: vec![],
            next_cursor: None,
        };

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::to_value(response)?),
            error: None,
        })
    }

    /// Handle prompts/get request
    fn handle_prompt_get(&mut self, request: McpRequest) -> BridgeResult<McpResponse> {
        self.ensure_initialized(&request)?;

        // For now, return error as prompts are not implemented
        Ok(Self::create_error_response(
            request.id,
            &BridgeError::Protocol {
                message: "Prompt retrieval not implemented".to_string(),
            },
        ))
    }

    /// Ensure the server is initialized
    fn ensure_initialized(&self, _request: &McpRequest) -> BridgeResult<()> {
        if !self.initialized {
            return Err(BridgeError::Protocol {
                message: "Server not initialized".to_string(),
            });
        }
        Ok(())
    }

    /// Create an error response
    fn create_error_response(id: Option<Value>, error: &BridgeError) -> McpResponse {
        let mcp_error = match error {
            BridgeError::Json(_) => McpError::parse_error(error.to_string()),
            BridgeError::Protocol { .. } => McpError::invalid_request(error.to_string()),
            BridgeError::Auth { .. } => McpError::invalid_params(error.to_string()),
            BridgeError::Server { code, message } => McpError {
                code: *code,
                message: message.clone(),
                data: None,
            },
            _ => McpError::internal_error(error.to_string()),
        };

        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(mcp_error),
        }
    }

    /// Create a method not found response
    fn create_method_not_found_response(id: Option<Value>, method: &str) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError::method_not_found(format!(
                "Method '{method}' not found"
            ))),
        }
    }
}

/// Configuration builder for the STDIO server
pub struct StdioMcpServerBuilder {
    #[allow(dead_code)]
    config: BridgeConfig,
}

impl Default for StdioMcpServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl StdioMcpServerBuilder {
    pub fn new() -> Self {
        Self {
            config: BridgeConfig::default(),
        }
    }

    #[allow(dead_code)]
    pub fn endpoint(mut self, endpoint: String) -> Self {
        self.config.endpoint = endpoint;
        self
    }

    #[allow(dead_code)]
    pub fn protocol(mut self, protocol: String) -> Self {
        self.config.protocol = protocol;
        self
    }

    #[allow(dead_code)]
    pub fn api_key(mut self, api_key: Option<String>) -> Self {
        self.config.api_key = api_key;
        self
    }

    #[allow(dead_code)]
    pub fn timeout_seconds(mut self, timeout: u64) -> Self {
        self.config.timeout_seconds = timeout;
        self
    }

    #[allow(dead_code)]
    pub fn debug(mut self, debug: bool) -> Self {
        self.config.debug = debug;
        self
    }

    #[allow(dead_code)]
    pub fn build(self) -> StdioMcpServer {
        StdioMcpServer::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_builder() {
        let server = StdioMcpServerBuilder::new()
            .endpoint("https://mcp.deepwiki.com".to_string())
            .protocol("sse".to_string())
            .timeout_seconds(30)
            .debug(true)
            .build();

        assert_eq!(server.config.endpoint, "https://mcp.deepwiki.com");
        assert_eq!(server.config.protocol, "sse");
        assert_eq!(server.config.timeout_seconds, 30);
        assert!(server.config.debug);
        assert!(!server.initialized);
    }

    #[test]
    fn test_error_response_creation() {
        let _server = StdioMcpServer::new(BridgeConfig::default());
        let error = BridgeError::Protocol {
            message: "Test error".to_string(),
        };

        let response = StdioMcpServer::create_error_response(Some(Value::Number(1.into())), &error);

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(Value::Number(1.into())));
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_method_not_found_response() {
        let _server = StdioMcpServer::new(BridgeConfig::default());
        let response = StdioMcpServer::create_method_not_found_response(
            Some(Value::Number(1.into())),
            "unknown/method",
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(Value::Number(1.into())));
        assert!(response.result.is_none());

        if let Some(error) = response.error {
            assert_eq!(error.code, -32601);
            assert!(error.message.contains("unknown/method"));
        }
    }
}
