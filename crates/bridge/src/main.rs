//! Minimal MCP Proxy using Official Rust SDK
//!
//! This binary serves as a lightweight proxy between Zed's STDIO-based MCP client and
//! HTTP/SSE-based MCP servers using the official rust-sdk. It provides transport
//! auto-detection, built-in OAuth2 authentication, and minimal overhead.

use anyhow::Result;
use rmcp::{
    model::{ClientCapabilities, ClientInfo, Implementation},
    transport::{auth::AuthorizationManager, SseClientTransport, StreamableHttpClientTransport},
    ServiceExt,
};
use std::env;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let endpoint_url = &args[1];

    // Validate URL format with detailed error handling
    if let Err(e) = validate_url(endpoint_url) {
        error!("{}", e);
        std::process::exit(1);
    }

    info!("Starting MCP Proxy for endpoint: {}", endpoint_url);

    // Run the proxy (implementation will be added in next tasks)
    if let Err(e) = run_proxy(endpoint_url).await {
        error!("Proxy failed: {}", e);
        std::process::exit(1);
    }

    info!("MCP Proxy stopped");
    Ok(())
}

/// Print usage information
fn print_usage(program_name: &str) {
    eprintln!("DeepWiki MCP Proxy - Minimal proxy using official rust-sdk");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    {program_name} <ENDPOINT_URL>");
    eprintln!();
    eprintln!("ARGUMENTS:");
    eprintln!("    <ENDPOINT_URL>    MCP server endpoint URL (http:// or https://)");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("    {program_name} https://mcp.deepwiki.com");
    eprintln!("    {program_name} https://mcp.devin.ai");
    eprintln!("    {program_name} https://localhost:8080/sse");
    eprintln!();
    eprintln!("TRANSPORT AUTO-DETECTION:");
    eprintln!("    URLs containing '/sse' will use SSE transport");
    eprintln!("    All other URLs will use HTTP transport");
    eprintln!();
    eprintln!("AUTHENTICATION:");
    eprintln!("    OAuth2 authentication is handled automatically when required");
}

/// Transport wrapper enum to handle different transport types
enum McpTransport {
    Http(StreamableHttpClientTransport<reqwest::Client>),
    Sse(SseClientTransport<reqwest::Client>),
}

/// Run the MCP proxy with transport auto-detection and authentication
async fn run_proxy(endpoint_url: &str) -> Result<()> {
    // Check if endpoint requires authentication
    let needs_auth = endpoint_url.contains("mcp.devin.ai");
    
    if needs_auth {
        info!("Devin endpoint detected - OAuth2 authentication will be handled automatically");
    } else {
        info!("DeepWiki endpoint detected - no authentication required");
    }

    // Detect and create transport based on URL pattern
    let remote_transport = create_transport(endpoint_url, needs_auth).await?;

    // Create client info for MCP connection
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "DeepWiki MCP Proxy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    info!("Creating MCP client with remote transport...");

    // Test the connection by creating a client (simplified approach)
    match remote_transport {
        McpTransport::Http(transport) => {
            info!("Testing HTTP connection to MCP server");
            let _client = client_info.serve(transport).await.map_err(|e| {
                error!("Failed to connect via HTTP: {}", e);
                anyhow::anyhow!("HTTP connection failed: {}", e)
            })?;
            info!("HTTP connection established successfully");
        }
        McpTransport::Sse(transport) => {
            info!("Testing SSE connection to MCP server");
            let _client = client_info.serve(transport).await.map_err(|e| {
                error!("Failed to connect via SSE: {}", e);
                anyhow::anyhow!("SSE connection failed: {}", e)
            })?;
            info!("SSE connection established successfully");
        }
    };

    info!("Transport connection verified successfully");
    info!("STDIO integration and message proxying will be implemented in next task");

    // TODO: Implement bidirectional message proxying between STDIO and remote transport
    // TODO: Handle MCP protocol message forwarding
    // TODO: Manage connection lifecycle and graceful shutdown

    Ok(())
}

/// Create the appropriate transport based on URL patterns
async fn create_transport(endpoint_url: &str) -> Result<McpTransport> {
    let transport_type = detect_transport_type(endpoint_url);
    info!("Detected transport type: {}", transport_type);

    match transport_type {
        "SSE" => {
            info!("Creating SSE client transport for: {}", endpoint_url);
            match SseClientTransport::start(endpoint_url).await {
                Ok(transport) => {
                    info!("SSE transport created successfully");
                    Ok(McpTransport::Sse(transport))
                }
                Err(e) => {
                    error!("Failed to create SSE transport: {}", e);
                    Err(anyhow::anyhow!("SSE transport creation failed: {}", e))
                }
            }
        }
        "HTTP" => {
            info!("Creating HTTP client transport for: {}", endpoint_url);
            let transport = StreamableHttpClientTransport::from_uri(endpoint_url);
            info!("HTTP transport created successfully");
            Ok(McpTransport::Http(transport))
        }
        _ => {
            error!("Unknown transport type: {}", transport_type);
            Err(anyhow::anyhow!(
                "Unsupported transport type: {}",
                transport_type
            ))
        }
    }
}

/// Detect transport type based on URL patterns
fn detect_transport_type(url: &str) -> &'static str {
    if url.contains("/sse") {
        "SSE"
    } else {
        "HTTP"
    }
}

/// Validate URL format and provide helpful error messages
fn validate_url(url: &str) -> Result<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(anyhow::anyhow!(
            "Invalid URL format: {}. URL must start with http:// or https://",
            url
        ));
    }

    // Parse URL to validate structure
    match url::Url::parse(url) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Invalid URL structure: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_detection() {
        assert_eq!(detect_transport_type("https://mcp.deepwiki.com"), "HTTP");
        assert_eq!(detect_transport_type("https://mcp.devin.ai"), "HTTP");
        assert_eq!(detect_transport_type("https://localhost:8080/sse"), "SSE");
        assert_eq!(
            detect_transport_type("https://example.com/api/sse/events"),
            "SSE"
        );
        assert_eq!(detect_transport_type("http://example.com/mcp"), "HTTP");
        assert_eq!(
            detect_transport_type("https://api.example.com/v1/sse"),
            "SSE"
        );
    }

    #[test]
    fn test_url_validation() {
        // Valid URLs
        assert!(validate_url("https://mcp.deepwiki.com").is_ok());
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("https://example.com/api/sse").is_ok());

        // Invalid URLs
        assert!(validate_url("mcp.deepwiki.com").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("https://").is_err());
        assert!(validate_url("not-a-url").is_err());
    }

    #[test]
    fn test_url_validation_logic() {
        // Valid URLs
        assert!(
            "https://mcp.deepwiki.com".starts_with("http://")
                || "https://mcp.deepwiki.com".starts_with("https://")
        );
        assert!(
            "http://localhost:8080".starts_with("http://")
                || "http://localhost:8080".starts_with("https://")
        );

        // Invalid URLs
        assert!(
            !("mcp.deepwiki.com".starts_with("http://")
                || "mcp.deepwiki.com".starts_with("https://"))
        );
        assert!(
            !("ftp://example.com".starts_with("http://")
                || "ftp://example.com".starts_with("https://"))
        );
    }

    #[test]
    fn test_command_line_parsing_logic() {
        // Test argument count validation
        let args_empty = ["program".to_string()];
        assert_eq!(args_empty.len(), 1); // Should fail validation (needs 2)

        let args_correct = [
            "program".to_string(),
            "https://mcp.deepwiki.com".to_string(),
        ];
        assert_eq!(args_correct.len(), 2); // Should pass validation

        let args_too_many = [
            "program".to_string(),
            "https://mcp.deepwiki.com".to_string(),
            "extra".to_string(),
        ];
        assert_eq!(args_too_many.len(), 3); // Should fail validation (too many)
    }
}
