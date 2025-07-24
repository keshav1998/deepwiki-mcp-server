// Test comment to trigger Lefthook hooks
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
    /// - <https://mcp.devin.ai> for authenticated access (`OAuth2` handled automatically)
    #[serde(default = "default_endpoint")]
    endpoint: String,
}

fn default_endpoint() -> String {
    "https://mcp.deepwiki.com".to_string()
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
        let settings = ContextServerSettings::for_project("deepwiki-mcp-server", project)?;

        let config = settings.settings.map_or_else(
            || DeepWikiContextServerSettings {
                endpoint: default_endpoint(),
            },
            |settings_value| {
                serde_json::from_value(settings_value).unwrap_or_else(|_| {
                    DeepWikiContextServerSettings {
                        endpoint: default_endpoint(),
                    }
                })
            },
        );

        // Use new minimal proxy with endpoint URL as argument
        // OAuth2 authentication is handled automatically by the proxy
        Ok(Command {
            command: bridge_path,
            args: vec![config.endpoint],
            env: vec![],
        })
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        let installation_instructions =
            include_str!("../configuration/installation_instructions.md").to_string();

        let settings = ContextServerSettings::for_project("deepwiki-mcp-server", project);

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
    fn ensure_bridge_binary(&self) -> Result<String> {
        let binary_name = Self::get_binary_name();
        let binary_path = format!("bin/{binary_name}");

        // Check if binary already exists
        if fs::metadata(&binary_path).is_ok() {
            return Ok(binary_path);
        }

        // Create bin directory if it doesn't exist
        fs::create_dir_all("bin").map_err(|e| format!("Failed to create bin directory: {e}"))?;

        // Download the binary from GitHub releases
        Self::download_bridge_binary(&binary_path)?;

        Ok(binary_path)
    }

    /// Download the bridge binary from GitHub releases
    fn download_bridge_binary(target_path: &str) -> Result<()> {
        let (os, arch) = current_platform();

        // Get the latest release
        let release = latest_github_release(
            "keshav1998/zed-mcp-proxy",
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
            Os::Windows => "zed-mcp-proxy.exe".to_string(),
            _ => "zed-mcp-proxy".to_string(),
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

        // Asset naming pattern: zed-mcp-proxy-{arch}-{os}.{ext}
        let extension = if os == Os::Windows { "zip" } else { "tar.gz" };

        format!("zed-mcp-proxy-{arch_str}-{os_str}.{extension}")
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
// Test comment for hook validation
// Test comment for WASM validation
