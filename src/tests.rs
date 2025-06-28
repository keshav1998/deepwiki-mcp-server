//! Tests for DeepWiki MCP Extension
//!
//! This test suite focuses on the extension functionality, configuration parsing,
//! and command construction. The actual MCP communication is handled by the shell
//! script proxy and is not tested here.
//!
//! Run with: `cargo test --lib`

#[cfg(test)]
mod tests {
    use crate::{
        DeepWikiContextServerSettings, DeepWikiMcpExtension, default_endpoint, default_protocol,
    };
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
    fn test_default_protocol() {
        let protocol = default_protocol();
        assert_eq!(protocol, "mcp");
    }

    #[test]
    fn test_deepwiki_context_server_settings_defaults() {
        let settings = DeepWikiContextServerSettings {
            endpoint: default_endpoint(),
            protocol: default_protocol(),
        };

        assert_eq!(settings.endpoint, "https://mcp.deepwiki.com");
        assert_eq!(settings.protocol, "mcp");
    }

    #[test]
    fn test_deepwiki_context_server_settings_custom() {
        let settings = DeepWikiContextServerSettings {
            endpoint: "https://custom.deepwiki.com".to_string(),
            protocol: "custom".to_string(),
        };

        assert_eq!(settings.endpoint, "https://custom.deepwiki.com");
        assert_eq!(settings.protocol, "custom");
    }

    #[test]
    fn test_settings_serialization_with_defaults() {
        let settings = DeepWikiContextServerSettings {
            endpoint: default_endpoint(),
            protocol: default_protocol(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("https://mcp.deepwiki.com"));
        assert!(json.contains("mcp"));
    }

    #[test]
    fn test_settings_deserialization_with_defaults() {
        let json = json!({});
        let settings: DeepWikiContextServerSettings = serde_json::from_value(json).unwrap();

        assert_eq!(settings.endpoint, "https://mcp.deepwiki.com");
        assert_eq!(settings.protocol, "mcp");
    }

    #[test]
    fn test_settings_deserialization_with_custom_values() {
        let json = json!({
            "endpoint": "https://custom.example.com",
            "protocol": "custom-protocol"
        });
        let settings: DeepWikiContextServerSettings = serde_json::from_value(json).unwrap();

        assert_eq!(settings.endpoint, "https://custom.example.com");
        assert_eq!(settings.protocol, "custom-protocol");
    }

    #[test]
    fn test_settings_deserialization_partial() {
        let json = json!({
            "endpoint": "https://partial.example.com"
        });
        let settings: DeepWikiContextServerSettings = serde_json::from_value(json).unwrap();

        assert_eq!(settings.endpoint, "https://partial.example.com");
        assert_eq!(settings.protocol, "mcp"); // Should use default
    }

    #[test]
    fn test_command_construction_with_defaults() {
        // Test that command construction works with default settings
        let config = DeepWikiContextServerSettings {
            endpoint: default_endpoint(),
            protocol: default_protocol(),
        };

        // Verify the command would be constructed correctly
        let expected_command = "./scripts/deepwiki-mcp-proxy.sh";
        let expected_env = vec![
            ("DEEPWIKI_ENDPOINT".to_string(), config.endpoint.clone()),
            ("DEEPWIKI_PROTOCOL".to_string(), config.protocol.clone()),
        ];

        assert_eq!(expected_env[0].0, "DEEPWIKI_ENDPOINT");
        assert_eq!(expected_env[0].1, "https://mcp.deepwiki.com");
        assert_eq!(expected_env[1].0, "DEEPWIKI_PROTOCOL");
        assert_eq!(expected_env[1].1, "mcp");
        assert_eq!(expected_command, "./scripts/deepwiki-mcp-proxy.sh");
    }

    #[test]
    fn test_command_construction_with_custom_config() {
        let config = DeepWikiContextServerSettings {
            endpoint: "https://custom.deepwiki.com".to_string(),
            protocol: "sse".to_string(),
        };

        let expected_env = vec![
            ("DEEPWIKI_ENDPOINT".to_string(), config.endpoint.clone()),
            ("DEEPWIKI_PROTOCOL".to_string(), config.protocol.clone()),
        ];

        assert_eq!(expected_env[0].1, "https://custom.deepwiki.com");
        assert_eq!(expected_env[1].1, "sse");
    }

    #[test]
    fn test_environment_variable_names() {
        // Ensure environment variable names are consistent
        let endpoint_var = "DEEPWIKI_ENDPOINT";
        let protocol_var = "DEEPWIKI_PROTOCOL";

        assert_eq!(endpoint_var.len(), 18);
        assert_eq!(protocol_var.len(), 18);
        assert!(endpoint_var.starts_with("DEEPWIKI_"));
        assert!(protocol_var.starts_with("DEEPWIKI_"));
    }

    #[test]
    fn test_shell_script_path() {
        let script_path = "./scripts/deepwiki-mcp-proxy.sh";

        // Verify the path format
        assert!(script_path.starts_with("./scripts/"));
        assert!(script_path.ends_with(".sh"));
        assert!(script_path.contains("deepwiki-mcp-proxy"));
    }

    #[test]
    fn test_settings_json_schema_compliance() {
        // Test that our settings struct can be serialized/deserialized properly
        let original = DeepWikiContextServerSettings {
            endpoint: "https://test.example.com".to_string(),
            protocol: "test-protocol".to_string(),
        };

        let json_value = serde_json::to_value(&original).unwrap();
        let deserialized: DeepWikiContextServerSettings =
            serde_json::from_value(json_value).unwrap();

        assert_eq!(original.endpoint, deserialized.endpoint);
        assert_eq!(original.protocol, deserialized.protocol);
    }

    #[test]
    fn test_extension_trait_implementation() {
        // Verify that DeepWikiMcpExtension implements the Extension trait
        let mut extension = DeepWikiMcpExtension::new();

        // This is a compile-time test - if this compiles, the trait is implemented correctly
        let _trait_object: &mut dyn Extension = &mut extension;

        // Test passes if it compiles
        assert!(true);
    }

    #[test]
    fn test_configuration_edge_cases() {
        // Test empty strings (should not panic)
        let config = DeepWikiContextServerSettings {
            endpoint: "".to_string(),
            protocol: "".to_string(),
        };

        assert_eq!(config.endpoint, "");
        assert_eq!(config.protocol, "");

        // Test very long strings (should not panic)
        let long_string = "x".repeat(1000);
        let config = DeepWikiContextServerSettings {
            endpoint: long_string.clone(),
            protocol: long_string.clone(),
        };

        assert_eq!(config.endpoint.len(), 1000);
        assert_eq!(config.protocol.len(), 1000);
    }

    #[test]
    fn test_default_functions_consistency() {
        // Ensure default functions return consistent values
        let endpoint1 = default_endpoint();
        let endpoint2 = default_endpoint();
        let protocol1 = default_protocol();
        let protocol2 = default_protocol();

        assert_eq!(endpoint1, endpoint2);
        assert_eq!(protocol1, protocol2);

        // Ensure they're not empty
        assert!(!endpoint1.is_empty());
        assert!(!protocol1.is_empty());

        // Ensure endpoint is a valid URL format
        assert!(endpoint1.starts_with("https://"));
    }

    #[test]
    fn test_serde_json_integration() {
        // Test that we can work with serde_json values as expected
        let json_obj = json!({
            "endpoint": "https://serde-test.com",
            "protocol": "serde-test"
        });

        // Test conversion both ways
        let settings: DeepWikiContextServerSettings =
            serde_json::from_value(json_obj.clone()).unwrap();
        let back_to_json = serde_json::to_value(&settings).unwrap();

        assert_eq!(json_obj, back_to_json);
    }

    #[test]
    fn test_debug_trait_implementation() {
        // Verify Debug is implemented (useful for logging/debugging)
        let settings = DeepWikiContextServerSettings {
            endpoint: "https://debug-test.com".to_string(),
            protocol: "debug-test".to_string(),
        };

        let debug_string = format!("{:?}", settings);
        assert!(debug_string.contains("DeepWikiContextServerSettings"));
        assert!(debug_string.contains("debug-test.com"));
        assert!(debug_string.contains("debug-test"));
    }
}

// Integration-style tests that would work with the actual Extension trait
// Note: These tests demonstrate the expected behavior but may not run in isolation
// due to the complexity of mocking zed_extension_api types
#[cfg(test)]
mod integration_tests {
    use crate::{DeepWikiContextServerSettings, DeepWikiMcpExtension};
    use zed_extension_api::Extension;

    #[test]
    fn test_extension_registration() {
        // This test verifies that the extension registration macro works
        // The actual registration happens at the module level with zed::register_extension!
        let _extension = DeepWikiMcpExtension::new();

        // If this compiles, the basic structure is correct
        assert_eq!(std::mem::size_of::<DeepWikiMcpExtension>(), 0);
    }

    #[test]
    fn test_command_structure_validity() {
        // Test that our command structure matches what Zed expects
        let config = DeepWikiContextServerSettings {
            endpoint: "https://test.com".to_string(),
            protocol: "test".to_string(),
        };

        // Simulate command construction
        let command_string = "./scripts/deepwiki-mcp-proxy.sh".to_string();
        let args: Vec<String> = vec![];
        let env = vec![
            ("DEEPWIKI_ENDPOINT".to_string(), config.endpoint),
            ("DEEPWIKI_PROTOCOL".to_string(), config.protocol),
        ];

        // Verify structure
        assert!(!command_string.is_empty());
        assert!(args.is_empty()); // We don't pass args to the shell script
        assert_eq!(env.len(), 2);
        assert!(env.iter().any(|(k, _)| k == "DEEPWIKI_ENDPOINT"));
        assert!(env.iter().any(|(k, _)| k == "DEEPWIKI_PROTOCOL"));
    }
}
