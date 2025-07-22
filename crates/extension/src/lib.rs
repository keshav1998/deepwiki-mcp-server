use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use zed::settings::ContextServerSettings;
use zed_extension_api::{
    self as zed, serde_json, Command, ContextServerConfiguration, ContextServerId, Project, Result,
};

struct DeepWikiMcpExtension;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct DeepWikiContextServerSettings {
    /// `DeepWiki` MCP server endpoint
    /// - <https://mcp.deepwiki.com> for free public repositories only
    /// - <https://mcp.devin.ai> for authenticated access to public and private repositories
    #[serde(default = "default_endpoint")]
    endpoint: String,

    /// Wire protocol to use (sse or mcp)
    #[serde(default = "default_protocol")]
    protocol: String,

    /// Optional Devin API key for authenticated access to private repositories
    /// Required when using <https://mcp.devin.ai> endpoint
    #[serde(default)]
    devin_api_key: Option<String>,
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
        // Get user settings or use defaults
        let settings =
            ContextServerSettings::for_project("deepwiki-mcp-server-extension", project)?;

        let config = settings.settings.map_or_else(
            || DeepWikiContextServerSettings {
                endpoint: default_endpoint(),
                protocol: default_protocol(),
                devin_api_key: None,
            },
            |settings_value| {
                serde_json::from_value(settings_value).unwrap_or_else(|_| {
                    DeepWikiContextServerSettings {
                        endpoint: default_endpoint(),
                        protocol: default_protocol(),
                        devin_api_key: None,
                    }
                })
            },
        );

        // Validate configuration
        if config.endpoint.contains("mcp.devin.ai") && config.devin_api_key.is_none() {
            return Err("devin_api_key is required when using the authenticated Devin endpoint (https://mcp.devin.ai)".into());
        }

        // Build environment variables - API key is handled securely via Command.env
        let mut env_vars = vec![
            ("DEEPWIKI_ENDPOINT".to_string(), config.endpoint),
            ("DEEPWIKI_PROTOCOL".to_string(), config.protocol),
        ];

        // Add API key if provided (sensitive data handled by Command.env)
        if let Some(api_key) = config.devin_api_key {
            env_vars.push(("DEVIN_API_KEY".to_string(), api_key));
        }

        Ok(Command {
            command: "deepwiki-mcp-bridge".to_string(),
            args: vec![],
            env: env_vars,
        })
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        let installation_instructions =
            include_str!("../configuration/installation_instructions.md").to_string();

        let settings = ContextServerSettings::for_project("deepwiki-mcp-server-extension", project);

        let mut default_settings =
            include_str!("../configuration/default_settings.jsonc").to_string();

        // Update default settings with current user configuration if available
        if let Ok(user_settings) = settings {
            if let Some(settings_value) = user_settings.settings {
                if let Ok(deepwiki_settings) =
                    serde_json::from_value::<DeepWikiContextServerSettings>(settings_value)
                {
                    // Update endpoint
                    default_settings = default_settings.replace(
                        "\"https://mcp.deepwiki.com\"",
                        &format!("\"{}\"", deepwiki_settings.endpoint),
                    );

                    // Update protocol
                    default_settings = default_settings
                        .replace("\"sse\"", &format!("\"{}\"", deepwiki_settings.protocol));

                    // Update API key if provided
                    if let Some(api_key) = deepwiki_settings.devin_api_key {
                        default_settings = default_settings
                            .replace("// \"devin_api_key\"", "\"devin_api_key\"")
                            .replace("\"YOUR_DEVIN_API_KEY\"", &format!("\"{api_key}\""));
                    }
                }
            }
        }

        let settings_schema =
            serde_json::to_string(&schemars::schema_for!(DeepWikiContextServerSettings))
                .map_err(|e| e.to_string())?;

        Ok(Some(ContextServerConfiguration {
            installation_instructions,
            default_settings,
            settings_schema,
        }))
    }
}

#[cfg(test)]
mod tests;

zed::register_extension!(DeepWikiMcpExtension);
