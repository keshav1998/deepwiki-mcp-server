use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use zed::settings::ContextServerSettings;
use zed_extension_api::{
    self as zed, current_platform, download_file, latest_github_release, make_file_executable,
    serde_json, Architecture, Command, ContextServerConfiguration, ContextServerId,
    DownloadedFileType, GithubReleaseOptions, Os, Project, Result,
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
        // Ensure bridge binary is available
        let bridge_path = self.ensure_bridge_binary()?;

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
            command: bridge_path,
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

impl DeepWikiMcpExtension {
    /// Ensure the bridge binary is available, downloading it if necessary
    #[allow(clippy::unused_self)]
    fn ensure_bridge_binary(&mut self) -> Result<String> {
        let binary_name = Self::get_binary_name();
        let binary_path = format!("bin/{binary_name}");

        // Check if binary already exists
        if fs::metadata(&binary_path).is_ok() {
            return Ok(binary_path);
        }

        // Create bin directory if it doesn't exist
        if fs::create_dir_all("bin").is_err() {
            return Err("Failed to create bin directory".into());
        }

        // Download the binary from GitHub releases
        Self::download_bridge_binary(&binary_path)?;

        Ok(binary_path)
    }

    /// Download the bridge binary from GitHub releases
    fn download_bridge_binary(target_path: &str) -> Result<()> {
        let (os, arch) = current_platform();

        // Get the latest release
        let release = latest_github_release(
            "keshav1998/deepwiki-mcp-server",
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        // Find the appropriate asset for the current platform
        let asset_name = Self::get_asset_name(os, arch);

        let asset = release
            .assets
            .into_iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("No release asset found for platform: {asset_name}"))?;

        // Download and extract the binary
        let download_path = format!("{target_path}.download");
        download_file(
            &asset.download_url,
            &download_path,
            Self::get_file_type(&asset_name),
        )?;

        // Handle different file types
        if asset_name.ends_with(".tar.gz")
            || std::path::Path::new(&asset_name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            // For archives, the binary should be extracted to the bin directory
            // The download_file function with Archive type should handle extraction
            let extracted_binary = format!("bin/{}", Self::get_binary_name());
            if fs::metadata(&extracted_binary).is_ok() {
                fs::rename(&extracted_binary, target_path)
                    .map_err(|e| format!("Failed to move extracted binary: {e}"))?;
            }
        } else {
            // For direct binary downloads, just rename
            fs::rename(&download_path, target_path)
                .map_err(|e| format!("Failed to move downloaded binary: {e}"))?;
        }

        // Make the binary executable on Unix systems
        if matches!(os, Os::Mac | Os::Linux) {
            make_file_executable(target_path)?;
        }

        Ok(())
    }

    /// Get the binary name for the current platform
    fn get_binary_name() -> String {
        let (os, _) = current_platform();
        match os {
            Os::Windows => "deepwiki-mcp-bridge.exe".to_string(),
            _ => "deepwiki-mcp-bridge".to_string(),
        }
    }

    /// Get the asset name pattern for the current platform
    fn get_asset_name(os: Os, arch: Architecture) -> String {
        let os_str = match os {
            Os::Mac => "apple-darwin",
            Os::Linux => "unknown-linux-gnu",
            Os::Windows => "pc-windows-msvc",
        };

        let arch_str = match arch {
            Architecture::Aarch64 => "aarch64",
            Architecture::X86 => "i686",
            Architecture::X8664 => "x86_64",
        };

        // Asset naming pattern: deepwiki-mcp-bridge-{arch}-{os}.{ext}
        let extension = if os == Os::Windows { "zip" } else { "tar.gz" };

        format!("deepwiki-mcp-bridge-{arch_str}-{os_str}.{extension}")
    }

    /// Get the appropriate file type for `download_file`
    fn get_file_type(asset_name: &str) -> DownloadedFileType {
        if asset_name.ends_with(".tar.gz") {
            DownloadedFileType::GzipTar
        } else if std::path::Path::new(asset_name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            DownloadedFileType::Zip
        } else {
            DownloadedFileType::Uncompressed
        }
    }
}

#[cfg(test)]
mod tests;

zed::register_extension!(DeepWikiMcpExtension);
