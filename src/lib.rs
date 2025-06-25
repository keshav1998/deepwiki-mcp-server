//! DeepWiki MCP Zed Extension
//!
//! This crate exposes the DeepWiki MCP server as a Zed context server extension.
//! Implements the zed_extension_api::Extension trait to launch and manage the MCP server
//! from the Zed editor context. Uses Zed config and [context_servers] settings for endpoint
//! orchestration. For full integration/testing details, see README and configuration docs.

pub mod mcp;

use zed_extension_api::{self as zed, Command, ContextServerId, Project, Result};

/// Marker extension struct for Zed registration.
pub struct DeepWikiExtension;

impl zed::Extension for DeepWikiExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Command> {
        Ok(Command {
            command: "./deepwiki-mcp-bin".into(),
            args: vec![],
            env: vec![],
        })
    }
}

// Registers our extension with Zed (required).
zed::register_extension!(DeepWikiExtension);
