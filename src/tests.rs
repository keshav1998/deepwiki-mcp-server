//! Extension Integration & Protocol Tests for DeepWiki MCP Zed Extension
//!
//! ## Scope
//! - Test extension registration and context server command construction
//! - Sanity tests for MCP handlers (list_tools, call_tool)
//! - Foundation for further end-to-end and protocol compliance tests
//!
//! Run with: `cargo test --workspace --all-features`
//!
//! Note: Where feasible, favor async, isolated fast tests. Extend with mocking/fakes as MCP logic matures.

use zed_extension_api as zed;

#[cfg(test)]
mod extension_tests {
    use super::*;

    #[test]
    fn test_extension_registers() {
        // Verifies that the extension can be instantiated (static check)
        use crate::DeepWikiExtension;
        let mut ext = DeepWikiExtension;
        // Compile-time test: Extension trait is implemented.
        let _trait_obj: &mut dyn zed::Extension = &mut ext;
    }
}

#[cfg(test)]
mod mcp_protocol_tests {
    use super::*;
    use crate::mcp::{MCPError, call_tool, list_tools};

    #[tokio::test]
    async fn test_list_tools_baseline() {
        // Initially expect no tools; update as you add real ones in list_tools().
        let tools = list_tools().await;
        assert_eq!(
            tools.len(),
            0,
            "Tool list should be empty for baseline test"
        );
    }

    #[tokio::test]
    async fn test_call_tool_unknown_returns_not_implemented() {
        let res = call_tool("does_not_exist", serde_json::json!({})).await;
        assert!(
            matches!(res, Err(MCPError::NotImplemented)),
            "Expected NotImplemented for unknown tool"
        );
    }
}

// Additional protocol compliance, streaming, error-path, and config-integration tests
// should be added as MCP and Zed plumbing expands.
