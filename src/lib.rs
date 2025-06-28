use schemars::JsonSchema;
use serde::Deserialize;
use zed::settings::ContextServerSettings;
use zed_extension_api::{self as zed, serde_json, Command, ContextServerId, Project, Result};

struct DeepWikiMcpExtension;

#[derive(Debug, Deserialize, JsonSchema)]
struct DeepWikiContextServerSettings {
    /// DeepWiki MCP server endpoint (optional, defaults to official server)
    #[serde(default = "default_endpoint")]
    endpoint: String,
    /// Wire protocol to use (optional, defaults to 'mcp')
    #[serde(default = "default_protocol")]
    protocol: String,
}

fn default_endpoint() -> String {
    "https://mcp.deepwiki.com".to_string()
}

fn default_protocol() -> String {
    "mcp".to_string()
}

impl zed::Extension for DeepWikiMcpExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        // Get user settings or use defaults (DeepWiki is free, no auth required)
        let settings = ContextServerSettings::for_project("deepwiki-mcp-extension", project)?;

        let config = if let Some(settings_value) = settings.settings {
            serde_json::from_value(settings_value).unwrap_or_else(|_| {
                DeepWikiContextServerSettings {
                    endpoint: default_endpoint(),
                    protocol: default_protocol(),
                }
            })
        } else {
            DeepWikiContextServerSettings {
                endpoint: default_endpoint(),
                protocol: default_protocol(),
            }
        };

        Ok(Command {
            command: "./scripts/deepwiki-mcp-proxy.sh".to_string(),
            args: vec![],
            env: vec![
                ("DEEPWIKI_ENDPOINT".to_string(), config.endpoint),
                ("DEEPWIKI_PROTOCOL".to_string(), config.protocol),
            ],
        })
    }
}

zed::register_extension!(DeepWikiMcpExtension);
