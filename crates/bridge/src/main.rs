//! MCP Bridge Binary
//!
//! This binary serves as a bridge between Zed's STDIO-based MCP client and
//! HTTP-based DeepWiki/Devin MCP servers. It translates JSON-RPC messages
//! from stdin/stdout to HTTP requests/responses.

use crate::mcp_bridge::{server::StdioMcpServer, BridgeConfig};
use anyhow::Result;

mod mcp_bridge;
use std::env;
use tracing::{debug, error, info};
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

    info!("Starting DeepWiki MCP Bridge");

    // Load configuration from environment
    let config = load_config_from_env()?;
    debug!(
        "Bridge configuration: endpoint={}, protocol={}",
        config.endpoint, config.protocol
    );

    // Create the STDIO MCP server
    let mut server = StdioMcpServer::new(config);

    // Run the bridge
    if let Err(e) = server.run().await {
        error!("Bridge failed: {}", e);
        std::process::exit(1);
    }

    info!("DeepWiki MCP Bridge stopped");
    Ok(())
}

/// Load configuration from environment variables
fn load_config_from_env() -> Result<BridgeConfig> {
    let endpoint =
        env::var("DEEPWIKI_ENDPOINT").unwrap_or_else(|_| "https://mcp.deepwiki.com".to_string());

    let protocol = env::var("DEEPWIKI_PROTOCOL").unwrap_or_else(|_| "mcp".to_string());

    let api_key = env::var("DEVIN_API_KEY").ok();

    let timeout_seconds = env::var("DEEPWIKI_TIMEOUT")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .unwrap_or(60);

    let debug = env::var("DEBUG")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    // Validate configuration
    if endpoint.contains("mcp.devin.ai") && api_key.is_none() {
        anyhow::bail!("DEVIN_API_KEY is required when using the authenticated Devin endpoint");
    }

    Ok(BridgeConfig {
        endpoint,
        protocol,
        api_key,
        timeout_seconds,
        debug,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    const MOCK_API_KEY: &str = "mock_test_key_12345";

    #[test]
    fn test_load_config_defaults() {
        // Clear environment variables
        for key in [
            "DEEPWIKI_ENDPOINT",
            "DEEPWIKI_PROTOCOL",
            "DEVIN_API_KEY",
            "DEEPWIKI_TIMEOUT",
            "DEBUG",
        ] {
            env::remove_var(key);
        }

        let config = load_config_from_env().unwrap();
        assert_eq!(config.endpoint, "https://mcp.deepwiki.com");
        assert_eq!(config.protocol, "mcp");
        assert!(config.api_key.is_none());
        assert_eq!(config.timeout_seconds, 60);
        assert!(!config.debug);
    }

    #[test]
    fn test_load_config_custom() {
        // Clear first to ensure clean state
        for key in [
            "DEEPWIKI_ENDPOINT",
            "DEEPWIKI_PROTOCOL",
            "DEVIN_API_KEY",
            "DEEPWIKI_TIMEOUT",
            "DEBUG",
        ] {
            env::remove_var(key);
        }

        env::set_var("DEEPWIKI_ENDPOINT", "https://example.com");
        env::set_var("DEEPWIKI_PROTOCOL", "sse");
        env::set_var("DEVIN_API_KEY", MOCK_API_KEY);
        env::set_var("DEEPWIKI_TIMEOUT", "120");
        env::set_var("DEBUG", "true");

        let config = load_config_from_env().unwrap();
        assert_eq!(config.endpoint, "https://example.com");
        assert_eq!(config.protocol, "sse");
        assert_eq!(config.api_key, Some(MOCK_API_KEY.to_string()));
        assert_eq!(config.timeout_seconds, 120);
        assert!(config.debug);

        // Clean up
        for key in [
            "DEEPWIKI_ENDPOINT",
            "DEEPWIKI_PROTOCOL",
            "DEVIN_API_KEY",
            "DEEPWIKI_TIMEOUT",
            "DEBUG",
        ] {
            env::remove_var(key);
        }
    }

    #[test]
    fn test_devin_endpoint_validation_logic() {
        // Clear all environment variables first
        for key in [
            "DEEPWIKI_ENDPOINT",
            "DEEPWIKI_PROTOCOL",
            "DEVIN_API_KEY",
            "DEEPWIKI_TIMEOUT",
            "DEBUG",
        ] {
            env::remove_var(key);
        }

        // Set only the Devin endpoint without API key
        env::set_var("DEEPWIKI_ENDPOINT", "https://mcp.devin.ai");

        let result = load_config_from_env();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("DEVIN_API_KEY is required"));

        // Clean up
        for key in [
            "DEEPWIKI_ENDPOINT",
            "DEEPWIKI_PROTOCOL",
            "DEVIN_API_KEY",
            "DEEPWIKI_TIMEOUT",
            "DEBUG",
        ] {
            env::remove_var(key);
        }
    }

    #[test]
    fn test_invalid_timeout_uses_default() {
        // Clear all environment variables first
        for key in [
            "DEEPWIKI_ENDPOINT",
            "DEEPWIKI_PROTOCOL",
            "DEVIN_API_KEY",
            "DEEPWIKI_TIMEOUT",
            "DEBUG",
        ] {
            env::remove_var(key);
        }

        // Set only invalid timeout - use default endpoint to avoid validation issues
        env::set_var("DEEPWIKI_TIMEOUT", "invalid");

        let config = load_config_from_env().unwrap();
        assert_eq!(config.timeout_seconds, 60);

        // Clean up
        for key in [
            "DEEPWIKI_ENDPOINT",
            "DEEPWIKI_PROTOCOL",
            "DEVIN_API_KEY",
            "DEEPWIKI_TIMEOUT",
            "DEBUG",
        ] {
            env::remove_var(key);
        }
    }
}
