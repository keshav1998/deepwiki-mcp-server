//! # DeepWiki MCP Server Extension for Zed
//!
//! A **Model Context Protocol (MCP) server extension** for the Zed IDE that provides seamless
//! integration with DeepWiki and Devin AI documentation services.
//!
//! ## Architecture
//!
//! This extension uses a **separated architecture** with automatic binary download:
//!
//! ```text
//! Zed ↔ Extension (WASM) → Auto-Downloaded Proxy (Native) ↔ HTTP MCP Server
//! ```
//!
//! ### Components
//!
//! 1. **Extension (WASM)** - This library
//!    - Lightweight Zed extension compiled to WebAssembly
//!    - Automatically downloads platform-specific proxy binary
//!    - Provides configuration UI and command setup
//!    - No async/HTTP dependencies (WASM-compatible)
//!
//! 2. **Proxy Binary (Native)** - [zed-mcp-proxy](https://github.com/keshav1998/zed-mcp-proxy)
//!    - Auto-downloaded from separate repository releases
//!    - Full HTTP/async capabilities with tokio and reqwest
//!    - Translates between STDIO (Zed) and HTTP (DeepWiki/Devin)
//!    - Handles MCP protocol communication with official Rust MCP SDK
//!
//! ## Features
//!
//! - **Free DeepWiki Access**: Query public repository documentation
//! - **Devin AI Integration**: Enhanced AI-powered documentation with API key
//! - **Automatic Setup**: Bridge binary downloaded automatically per platform
//! - **Type-Safe Configuration**: JSON schema validation for settings
//! - **Secure Authentication**: Environment variable-based API key handling
//! - **Protocol Compliance**: Full MCP (Model Context Protocol) support
//! - **Cross-Platform**: Supports Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), Windows
//!
//! ## Usage
//!
//! This library is automatically loaded by Zed when the extension is installed.
//! Users configure the extension through Zed's settings:
//!
//! ```json
//! {
//!   "context_servers": {
//!     "deepwiki-mcp-server": {
//!       "endpoint": "https://mcp.deepwiki.com"
//!     }
//!   }
//! }
//! ```
//!
//! ## Example
//!
//! The extension automatically handles:
//! 1. Downloading the appropriate proxy binary for the user's platform
//! 2. Configuring the MCP server connection
//! 3. Providing the command setup for Zed to execute
//!
//! ```rust,no_run
//! use zed_extension_api::Extension;
//!
//! // This is handled automatically by Zed
//! let extension = DeepWikiMcpExtension::new();
//! ```

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use zed::settings::ContextServerSettings;
use zed_extension_api::{
    self as zed, current_platform, download_file, latest_github_release, make_file_executable,
    serde_json, Architecture, Command, ContextServerConfiguration, ContextServerId,
    DownloadedFileType, GithubReleaseOptions, Os, Project, Result,
};

/// The main extension struct that implements Zed's Extension trait.
///
/// This struct handles:
/// - Automatic proxy binary download and management
/// - Configuration parsing and validation
/// - Command setup for MCP server communication
/// - Platform-specific binary resolution
///
/// # Architecture
///
/// The extension follows a proxy pattern where it downloads and manages
/// a separate native binary (`zed-mcp-proxy`) that handles the actual
/// MCP protocol communication. This allows the WASM extension to remain
/// lightweight while providing full MCP functionality.
pub struct DeepWikiMcpExtension;

/// Configuration settings for the DeepWiki MCP server.
///
/// This struct defines the user-configurable options for the extension.
/// It supports JSON schema generation for Zed's configuration UI.
///
/// # Fields
///
/// * `endpoint` - The MCP server endpoint URL to connect to
///
/// # Supported Endpoints
///
/// - **DeepWiki Free**: `https://mcp.deepwiki.com` - Public repositories only
/// - **Devin AI**: `https://mcp.devin.ai` - Enhanced AI with authentication
///
/// # Example Configuration
///
/// ```json
/// {
///   "endpoint": "https://mcp.deepwiki.com"
/// }
/// ```
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DeepWikiContextServerSettings {
    /// MCP server endpoint URL.
    ///
    /// Supported endpoints:
    /// - `https://mcp.deepwiki.com` for free public repositories only
    /// - `https://mcp.devin.ai` for authenticated access (OAuth2 handled automatically)
    ///
    /// The proxy binary automatically detects the endpoint type and configures
    /// the appropriate transport and authentication mechanisms.
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
}

/// Returns the default MCP server endpoint.
///
/// The default endpoint provides free access to public repository documentation
/// through DeepWiki's MCP server.
///
/// # Returns
///
/// The default endpoint URL as a String: `"https://mcp.deepwiki.com"`
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
    /// Ensures the proxy binary is available, downloading it if necessary.
    ///
    /// This method handles the automatic download and setup of the platform-specific
    /// `zed-mcp-proxy` binary from the GitHub releases. It checks if the binary
    /// already exists and downloads it only if needed.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Path to the proxy binary
    /// * `Err(String)` - Error message if download or setup fails
    ///
    /// # Binary Management
    ///
    /// - Binaries are stored in a local `bin/` directory
    /// - Platform-specific naming (`.exe` suffix on Windows)
    /// - Automatic executable permissions on Unix systems
    /// - Downloaded from [zed-mcp-proxy releases](https://github.com/keshav1998/zed-mcp-proxy/releases)
    ///
    /// # Supported Platforms
    ///
    /// - Linux: x86_64, aarch64
    /// - macOS: x86_64 (Intel), aarch64 (Apple Silicon)
    /// - Windows: x86_64
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

    /// Downloads the proxy binary from GitHub releases.
    ///
    /// This method handles the complete download process:
    /// 1. Determines the current platform (OS and architecture)
    /// 2. Fetches the latest release from the zed-mcp-proxy repository
    /// 3. Downloads the appropriate platform-specific asset
    /// 4. Extracts the binary and sets proper permissions
    ///
    /// # Arguments
    ///
    /// * `target_path` - The local path where the binary should be saved
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Download and setup completed successfully
    /// * `Err(String)` - Error message if any step fails
    ///
    /// # Asset Naming Convention
    ///
    /// Assets follow the pattern: `zed-mcp-proxy-{arch}-{os}.{ext}`
    /// - Linux: `.tar.gz` archives
    /// - macOS: `.tar.gz` archives
    /// - Windows: `.zip` archives
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

    /// Gets the platform-specific binary name.
    ///
    /// Returns the appropriate executable name for the current platform:
    /// - Windows: `zed-mcp-proxy.exe`
    /// - Unix systems: `zed-mcp-proxy`
    ///
    /// # Returns
    ///
    /// The binary filename as a String
    pub fn get_binary_name() -> String {
        let (os, _) = current_platform();
        match os {
            Os::Windows => "zed-mcp-proxy.exe".to_string(),
            _ => "zed-mcp-proxy".to_string(),
        }
    }

    /// Generates the GitHub release asset name for a given platform.
    ///
    /// This method constructs the expected asset filename based on the
    /// target operating system and architecture, following the naming
    /// convention used by the zed-mcp-proxy repository releases.
    ///
    /// # Arguments
    ///
    /// * `os` - Target operating system
    /// * `arch` - Target CPU architecture
    ///
    /// # Returns
    ///
    /// The asset filename as a String
    ///
    /// # Asset Naming Pattern
    ///
    /// `zed-mcp-proxy-{arch}-{os}.{ext}`
    ///
    /// Where:
    /// - `{arch}`: `aarch64`, `i686`, or `x86_64`
    /// - `{os}`: `apple-darwin`, `unknown-linux-gnu`, or `pc-windows-msvc`
    /// - `{ext}`: `tar.gz` for Unix, `zip` for Windows
    pub fn get_asset_name(os: Os, arch: Architecture) -> String {
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

    /// Determines the appropriate file type for the download_file function.
    ///
    /// This method maps asset filenames to the corresponding DownloadedFileType
    /// enum values used by Zed's download_file API.
    ///
    /// # Arguments
    ///
    /// * `asset_name` - The asset filename
    ///
    /// # Returns
    ///
    /// The appropriate DownloadedFileType:
    /// - `GzipTar` for `.tar.gz` files
    /// - `Zip` for `.zip` files
    /// - `Uncompressed` for direct binary downloads
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
