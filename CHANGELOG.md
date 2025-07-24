# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- DeepWiki MCP Server Extension for Zed IDE
- Automatic proxy binary download and management
- Support for DeepWiki free public repository access
- Support for Devin AI authenticated access with OAuth2
- Cross-platform binary support (Linux x86_64/ARM64, macOS Intel/Apple Silicon, Windows x86_64)
- JSON schema-based configuration validation
- Modern separated architecture (WASM extension + native proxy)
- Comprehensive documentation with API reference
- Full test coverage with 27 tests (100% pass rate)
- Modern development workflow with Lefthook git hooks
- Automated release management with cargo-release
- CI/CD pipeline for WASM compilation

### Changed
- Migrated from monolithic architecture to separated extension + proxy design
- Replaced custom build scripts with standard Cargo commands
- Updated to use official Rust MCP SDK (rmcp v0.2.1) in proxy binary

### Technical Details
- Extension compiled to WebAssembly (wasm32-wasip1 target)
- Proxy binary auto-downloaded from GitHub releases
- MCP protocol compliance with full feature support
- Transport auto-detection (HTTP/SSE based on endpoint URLs)
- Built-in OAuth2 authentication handling
- Comprehensive error handling and logging
- Platform-specific asset management
- Type-safe configuration with schemars JSON schema

### Security
- WASM sandboxing for extension security
- Process isolation between extension and proxy
- Environment variable-based secret management
- No hardcoded credentials or API keys
- Capability-based permission system

## [0.1.0] - 2025-01-XX

### Added
- Initial release of DeepWiki MCP Server Extension
- Model Context Protocol integration for Zed editor
- Seamless documentation access through AI assistants
- Free DeepWiki and premium Devin AI endpoint support
- Automatic binary management and cross-platform compatibility
- Production-ready implementation with comprehensive testing