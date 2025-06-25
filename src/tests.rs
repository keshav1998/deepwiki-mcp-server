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
    async fn test_list_tools_returns_expected_tools() {
        let tools = list_tools().await;
        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"read_wiki_structure".to_string()));
        assert!(tool_names.contains(&"read_wiki_contents".to_string()));
        assert!(tool_names.contains(&"ask_question".to_string()));
        assert_eq!(tools.len(), 3, "Tool list should have exactly 3 entries");
    }

    #[tokio::test]
    async fn test_call_tool_unknown_returns_not_implemented() {
        let res = call_tool("does_not_exist", serde_json::json!({})).await;
        assert!(
            matches!(res, Err(MCPError::NotImplemented)),
            "Expected NotImplemented for unknown tool"
        );
    }

    #[tokio::test]
    async fn test_call_read_wiki_structure_success() {
        let res = call_tool(
            "read_wiki_structure",
            serde_json::json!({"repoName": "modelcontextprotocol/modelcontextprotocol"}),
        )
        .await;
        assert!(res.is_ok());
        let val = res.unwrap();
        assert_eq!(val["repo"], "modelcontextprotocol/modelcontextprotocol");
        assert!(val["topics"].is_array());
    }

    #[tokio::test]
    async fn test_call_read_wiki_structure_missing_arg() {
        let res = call_tool("read_wiki_structure", serde_json::json!({})).await;
        assert!(
            matches!(res, Err(MCPError::InvalidArgs(_))),
            "Expected InvalidArgs for missing repoName"
        );
    }

    #[tokio::test]
    async fn test_call_read_wiki_contents_success() {
        let repo = "modelcontextprotocol/modelcontextprotocol";
        let res = call_tool("read_wiki_contents", serde_json::json!({"repoName": repo})).await;
        assert!(res.is_ok());
        let val = res.unwrap();
        assert_eq!(val["repo"], repo);
        assert!(val["content"].is_string());
    }

    #[tokio::test]
    async fn test_call_read_wiki_contents_invalid_args() {
        let res = call_tool("read_wiki_contents", serde_json::json!({"repo": "foo"})).await;
        assert!(
            matches!(res, Err(MCPError::InvalidArgs(_))),
            "Expected InvalidArgs for wrong arg"
        );
    }

    #[tokio::test]
    async fn test_call_ask_question_success() {
        let repo = "modelcontextprotocol/modelcontextprotocol";
        let question = "What transport protocols are supported?";
        let res = call_tool(
            "ask_question",
            serde_json::json!({"repoName": repo, "question": question}),
        )
        .await;
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val["answer"].as_str().unwrap().contains(question));
        assert!(val["sources"].is_array());
    }

    #[tokio::test]
    async fn test_call_ask_question_missing_args() {
        let repo = "modelcontextprotocol/modelcontextprotocol";
        let res = call_tool("ask_question", serde_json::json!({"repoName": repo})).await;
        assert!(
            matches!(res, Err(MCPError::InvalidArgs(_))),
            "Expected InvalidArgs if question is missing"
        );
    }
}

// Additional protocol compliance, streaming, error-path, and config-integration tests
// should be added as MCP and Zed plumbing expands.
