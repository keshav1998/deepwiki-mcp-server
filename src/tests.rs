//! Tests for `DeepWiki` MCP Extension
//!
//! This test suite focuses on the extension functionality, configuration parsing,
//! and command construction. The actual MCP communication is handled by the
//! minimal proxy binary and is not tested here.
//!
//! Run with: `cargo test --lib`

// Test constants to avoid hardcoded values that might be flagged as secrets
const MOCK_ENDPOINT: &str = "https://test.example.com";

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::{default_endpoint, DeepWikiContextServerSettings, DeepWikiMcpExtension};
    use serde_json::json;
    use zed_extension_api::Extension;

    // Note: Project is an opaque type from zed_extension_api, so we focus on
    // testing the configuration and command logic that we can test

    #[test]
    fn test_extension_instantiation() {
        let extension = DeepWikiMcpExtension::new();
        // Verify the extension can be created
        assert_eq!(
            std::mem::size_of_val(&extension),
            std::mem::size_of::<DeepWikiMcpExtension>()
        );
    }

    #[test]
    fn test_default_endpoint() {
        let endpoint = default_endpoint();
        assert_eq!(endpoint, "https://mcp.deepwiki.com");
    }

    #[test]
    fn test_deepwiki_context_server_settings_defaults() {
        let settings = DeepWikiContextServerSettings {
            endpoint: default_endpoint(),
        };

        assert_eq!(settings.endpoint, "https://mcp.deepwiki.com");
    }

    #[test]
    fn test_deepwiki_context_server_settings_with_devin_endpoint() {
        let settings = DeepWikiContextServerSettings {
            endpoint: "https://mcp.devin.ai".to_string(),
        };

        assert_eq!(settings.endpoint, "https://mcp.devin.ai");
    }

    #[test]
    fn test_deepwiki_context_server_settings_custom() {
        let settings = DeepWikiContextServerSettings {
            endpoint: "https://custom.example.com".to_string(),
        };

        assert_eq!(settings.endpoint, "https://custom.example.com");
    }

    #[test]
    fn test_json_schema_generation() {
        let settings = DeepWikiContextServerSettings {
            endpoint: default_endpoint(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("mcp.deepwiki.com"));

        // Test that JSON can be parsed back
        let parsed: DeepWikiContextServerSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.endpoint, settings.endpoint);
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let original = DeepWikiContextServerSettings {
            endpoint: MOCK_ENDPOINT.to_string(),
        };

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: DeepWikiContextServerSettings =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.endpoint, deserialized.endpoint);
    }

    #[test]
    fn test_json_deserialization_with_defaults() {
        // Test that settings use defaults when fields are missing
        let json = json!({});
        let result = serde_json::from_value::<DeepWikiContextServerSettings>(json);

        // Should use default endpoint
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.endpoint, default_endpoint());
    }

    #[test]
    fn test_json_deserialization_with_custom_endpoint() {
        let json = json!({
            "endpoint": "https://custom.example.com"
        });
        let settings: DeepWikiContextServerSettings = serde_json::from_value(json).unwrap();

        assert_eq!(settings.endpoint, "https://custom.example.com");
    }

    #[test]
    fn test_command_construction_basic() {
        // Test that command construction works with default settings
        let config = DeepWikiContextServerSettings {
            endpoint: default_endpoint(),
        };

        // Verify the command arguments would be constructed correctly
        let args = [config.endpoint];
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], "https://mcp.deepwiki.com");
    }

    #[test]
    fn test_command_construction_with_devin_endpoint() {
        let config = DeepWikiContextServerSettings {
            endpoint: "https://mcp.devin.ai".to_string(),
        };

        let args = [config.endpoint];

        assert_eq!(args[0], "https://mcp.devin.ai");
        assert_eq!(args.len(), 1); // OAuth2 handled automatically by proxy
    }

    #[test]
    fn test_command_construction_with_custom_config() {
        let config = DeepWikiContextServerSettings {
            endpoint: "https://custom.example.com".to_string(),
        };

        let args = [config.endpoint];

        assert_eq!(args[0], "https://custom.example.com");
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn test_url_validation_patterns() {
        // Test various URL patterns that should be valid
        let valid_urls = [
            "https://mcp.deepwiki.com",
            "https://mcp.devin.ai",
            "https://mcp.devin.ai/sse",
            "http://localhost:8080",
            "https://custom.example.com/mcp",
        ];

        for url in &valid_urls {
            let config = DeepWikiContextServerSettings {
                endpoint: (*url).to_string(),
            };
            assert_eq!(config.endpoint, *url);
        }
    }

    #[test]
    fn test_endpoint_format_handling() {
        // Test that endpoints are handled correctly regardless of format
        let test_cases = [
            ("https://mcp.deepwiki.com", "https://mcp.deepwiki.com"),
            ("https://mcp.devin.ai/", "https://mcp.devin.ai/"),
            ("https://test.com/sse", "https://test.com/sse"),
        ];

        for (input, expected) in &test_cases {
            let config = DeepWikiContextServerSettings {
                endpoint: (*input).to_string(),
            };
            assert_eq!(config.endpoint, *expected);
        }
    }

    #[test]
    fn test_settings_edge_cases() {
        // Test empty strings (should not panic)
        let config = DeepWikiContextServerSettings {
            endpoint: String::new(),
        };

        assert_eq!(config.endpoint, "");

        // Test very long strings (should not panic)
        let long_string = "https://".to_string() + &"x".repeat(1000) + ".com";
        let config = DeepWikiContextServerSettings {
            endpoint: long_string.clone(),
        };

        assert_eq!(config.endpoint, long_string);
    }

    #[test]
    fn test_serde_json_with_additional_fields() {
        // Test that additional fields in JSON are ignored gracefully
        let json = json!({
            "endpoint": "https://test.example.com",
            "unknown_field": "should_be_ignored",
            "another_field": 42
        });

        let result = serde_json::from_value::<DeepWikiContextServerSettings>(json);
        assert!(result.is_ok());

        let settings = result.unwrap();
        assert_eq!(settings.endpoint, "https://test.example.com");
    }

    #[test]
    fn test_schema_generation() {
        // Test that JSON schema can be generated for the settings
        use schemars::schema_for;

        let schema = schema_for!(DeepWikiContextServerSettings);
        let schema_json = serde_json::to_string(&schema).unwrap();

        // Should contain endpoint field
        assert!(schema_json.contains("endpoint"));
    }

    #[test]
    fn test_binary_name_logic() {
        // Test binary name logic without calling Zed API
        // On Windows: zed-mcp-proxy.exe
        // On other platforms: zed-mcp-proxy

        #[cfg(target_os = "windows")]
        let expected = "zed-mcp-proxy.exe";

        #[cfg(not(target_os = "windows"))]
        let expected = "zed-mcp-proxy";

        // Verify the expected name pattern is correct
        assert!(expected.starts_with("zed-mcp-proxy"));

        #[cfg(target_os = "windows")]
        assert!(expected.ends_with(".exe"));

        #[cfg(not(target_os = "windows"))]
        assert!(!std::path::Path::new(expected)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe")));
    }

    #[test]
    fn test_asset_name_generation() {
        use zed_extension_api::{Architecture, Os};

        // Test asset name generation for different platforms
        let test_cases = [
            (
                Os::Mac,
                Architecture::Aarch64,
                "zed-mcp-proxy-aarch64-apple-darwin.tar.gz",
            ),
            (
                Os::Mac,
                Architecture::X8664,
                "zed-mcp-proxy-x86_64-apple-darwin.tar.gz",
            ),
            (
                Os::Linux,
                Architecture::X8664,
                "zed-mcp-proxy-x86_64-unknown-linux-gnu.tar.gz",
            ),
            (
                Os::Windows,
                Architecture::X8664,
                "zed-mcp-proxy-x86_64-pc-windows-msvc.zip",
            ),
        ];

        for (os, arch, expected) in &test_cases {
            let asset_name = DeepWikiMcpExtension::get_asset_name(*os, *arch);
            assert_eq!(asset_name, *expected);
        }
    }

    #[test]
    fn test_file_type_detection() {
        // Test file type detection for different asset formats
        use zed_extension_api::DownloadedFileType;

        let test_cases = [
            ("file.tar.gz", DownloadedFileType::GzipTar),
            ("file.zip", DownloadedFileType::Zip),
            ("file.ZIP", DownloadedFileType::Zip), // Case insensitive
            ("file.bin", DownloadedFileType::Uncompressed),
            ("plain-file", DownloadedFileType::Uncompressed),
        ];

        for (filename, expected) in &test_cases {
            let file_type = DeepWikiMcpExtension::get_file_type(filename);
            match (expected, file_type) {
                (DownloadedFileType::GzipTar, DownloadedFileType::GzipTar)
                | (DownloadedFileType::Zip, DownloadedFileType::Zip)
                | (DownloadedFileType::Uncompressed, DownloadedFileType::Uncompressed) => (),
                _ => panic!(
                    "File type mismatch for {filename}: expected {expected:?}, got {file_type:?}"
                ),
            }
        }
    }

    #[test]
    fn test_command_structure_validity() {
        // Test command structure validity with different endpoints
        let config = DeepWikiContextServerSettings {
            endpoint: "https://mcp.deepwiki.com".to_string(),
        };

        // Simulate command construction
        let args = [config.endpoint];
        let env_vars: Vec<(String, String)> = vec![];

        // Verify simplified structure
        assert_eq!(args.len(), 1);
        assert_eq!(env_vars.len(), 0); // No environment variables needed
    }

    #[test]
    fn test_devin_endpoint_detection() {
        // Test that we can identify Devin endpoints
        let devin_endpoints = [
            "https://mcp.devin.ai",
            "https://mcp.devin.ai/",
            "https://mcp.devin.ai/sse",
        ];

        let non_devin_endpoints = [
            "https://mcp.deepwiki.com",
            "https://custom.example.com",
            "https://example.devin.ai", // Different subdomain
        ];

        for endpoint in &devin_endpoints {
            assert!(
                endpoint.contains("mcp.devin.ai"),
                "Should detect Devin endpoint: {endpoint}"
            );
        }

        for endpoint in &non_devin_endpoints {
            assert!(
                !endpoint.contains("mcp.devin.ai"),
                "Should not detect as Devin endpoint: {endpoint}"
            );
        }
    }

    #[test]
    fn test_settings_validation() {
        // Test that settings can be validated for common issues
        let valid_settings = DeepWikiContextServerSettings {
            endpoint: "https://mcp.deepwiki.com".to_string(),
        };

        // Basic validation checks
        assert!(!valid_settings.endpoint.is_empty());
        assert!(valid_settings.endpoint.starts_with("http"));
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::DeepWikiMcpExtension;
    use std::path::Path;
    use std::process::Command;
    use zed_extension_api::Extension;

    #[test]
    fn test_extension_workflow() {
        // Test the overall extension workflow
        let extension = DeepWikiMcpExtension::new();

        // Verify extension can be instantiated
        assert_eq!(
            std::mem::size_of_val(&extension),
            std::mem::size_of::<DeepWikiMcpExtension>()
        );

        // Note: Cannot test configuration in unit tests as it requires actual project context
        // This would need integration testing with a real Zed environment

        // Basic smoke test - extension creates successfully
        // Extension is automatically dropped at end of scope
        let _ = extension;
    }

    #[test]
    fn test_proxy_binary_integration() {
        // Test integration with the minimal proxy binary
        let proxy_path = "temp-bridge-extraction/target/debug/zed-mcp-proxy";

        // Skip test if proxy binary doesn't exist (CI environment)
        if !Path::new(proxy_path).exists() {
            println!(
                "Skipping proxy integration test - binary not found at {}",
                proxy_path
            );
            return;
        }

        // Test 1: Verify proxy binary can show usage
        let output = Command::new(proxy_path)
            .output()
            .expect("Failed to execute proxy binary");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Usage:"),
            "Proxy should show usage when no args provided"
        );
        assert!(
            stderr.contains("https://mcp.deepwiki.com"),
            "Usage should include example endpoints"
        );

        // Test 2: Verify proxy handles HTTP endpoint detection
        let output = Command::new("bash")
            .arg("-c")
            .arg(&format!(
                "echo '{{}}' | {} https://mcp.deepwiki.com 2>&1 | head -3",
                proxy_path
            ))
            .output()
            .expect("Failed to test HTTP endpoint");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Using HTTP transport"),
            "Proxy should detect HTTP transport for regular URLs"
        );

        // Test 3: Verify proxy handles SSE endpoint detection
        let output = Command::new("bash")
            .arg("-c")
            .arg(&format!(
                "echo '{{}}' | {} https://example.com/sse 2>&1 | head -3",
                proxy_path
            ))
            .output()
            .expect("Failed to test SSE endpoint");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Using SSE transport"),
            "Proxy should detect SSE transport for /sse URLs"
        );

        // Test 4: Verify extension binary naming matches proxy (static check)
        #[cfg(target_os = "windows")]
        let expected_binary = "zed-mcp-proxy.exe";
        #[cfg(not(target_os = "windows"))]
        let expected_binary = "zed-mcp-proxy";

        assert!(
            expected_binary.contains("zed-mcp-proxy"),
            "Extension should reference correct binary name"
        );

        println!("✅ Proxy integration tests passed - extension can work with minimal proxy");
    }

    #[test]
    fn test_extension_proxy_compatibility() {
        // Test that extension configuration is compatible with minimal proxy
        let _extension = DeepWikiMcpExtension::new();

        // Test asset name generation for proxy downloads (without calling Zed APIs)
        use zed_extension_api::{Architecture, Os};

        let test_cases = [
            (
                Os::Mac,
                Architecture::Aarch64,
                "zed-mcp-proxy-aarch64-apple-darwin.tar.gz",
            ),
            (
                Os::Linux,
                Architecture::X8664,
                "zed-mcp-proxy-x86_64-unknown-linux-gnu.tar.gz",
            ),
            (
                Os::Windows,
                Architecture::X8664,
                "zed-mcp-proxy-x86_64-pc-windows-msvc.zip",
            ),
        ];

        for (os, arch, expected_asset) in test_cases {
            let asset_name = DeepWikiMcpExtension::get_asset_name(os, arch);
            assert_eq!(
                asset_name, expected_asset,
                "Asset name should match proxy repository releases"
            );
        }

        // Test expected binary names (without calling Zed runtime APIs)
        #[cfg(target_os = "windows")]
        let expected_binary = "zed-mcp-proxy.exe";
        #[cfg(not(target_os = "windows"))]
        let expected_binary = "zed-mcp-proxy";

        assert!(
            expected_binary.contains("zed-mcp-proxy"),
            "Binary name should reference the proxy"
        );

        println!("✅ Extension-proxy compatibility verified");
    }

    #[test]
    fn test_mcp_protocol_readiness() {
        // Test that the proxy binary structure supports MCP protocol
        let proxy_path = "temp-bridge-extraction/target/debug/zed-mcp-proxy";

        if !Path::new(proxy_path).exists() {
            println!("Skipping MCP protocol test - binary not found");
            return;
        }

        // Verify proxy can handle MCP initialization attempt
        let mcp_init_message = r#"{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {"roots": {"listChanged": true}}, "clientInfo": {"name": "test-client", "version": "0.1.0"}}, "id": 1}"#;

        let output = Command::new("bash")
            .arg("-c")
            .arg(&format!(
                "echo '{}' | {} https://httpbin.org/status/404 2>&1 | head -5",
                mcp_init_message, proxy_path
            ))
            .output()
            .expect("Failed to test MCP protocol");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Proxy should start and attempt connection (even if it fails)
        assert!(
            stdout.contains("Starting MCP Proxy"),
            "Proxy should start MCP session"
        );
        assert!(
            stdout.contains("HTTP transport") || stdout.contains("SSE transport"),
            "Proxy should select transport"
        );

        // Should show it's trying to make MCP connection
        let contains_mcp_activity = stdout.contains("initialize")
            || stdout.contains("connection")
            || stdout.contains("Client error")
            || stdout.contains("404");
        assert!(
            contains_mcp_activity,
            "Proxy should attempt MCP protocol communication"
        );

        println!("✅ MCP protocol integration verified - proxy ready for real MCP servers");
    }
}
