# Repository Separation Plan - DeepWiki MCP Server

## Overview

This document outlines the plan to separate the current monorepo into two focused repositories:
- **deepwiki-mcp-server** (Zed extension) - Current repo, will retain extension functionality
- **zed-mcp-proxy** (MCP proxy binary) - New repo, will contain the extracted bridge

## Current Workspace Structure

### Root Level (Extension Repository)
```
deepwiki-mcp-server/
├── Cargo.toml              # Root package configuration (extension)
├── extension.toml           # Zed extension manifest
├── lefthook.yml            # Git hooks configuration
├── src/
│   ├── lib.rs              # Extension implementation
│   └── tests.rs            # Extension tests
├── configuration/          # Extension UI configuration
│   └── schema.json
├── README.md               # Project documentation
├── LICENSE                 # MIT license
└── .github/                # CI/CD workflows
    └── workflows/
```

### Bridge Crate (To Be Extracted)
```
crates/bridge/
├── Cargo.toml              # Bridge crate configuration
├── src/
│   └── main.rs             # MCP proxy binary implementation
└── (tests integrated in main.rs)
```

## Dependency Analysis

### Extension Dependencies (Staying)
- **zed_extension_api**: 0.6.0 - Core Zed extension API
- **schemars**: 0.8 - JSON schema generation for configuration
- **serde**: 1.0 - Serialization framework
- **serde_json**: 1.0 - JSON handling

### Bridge Dependencies (Moving to zed-mcp-proxy)
- **rmcp**: 0.2.1 - Official Rust MCP SDK
- **anyhow**: 1.0 - Error handling
- **reqwest**: 0.12 - HTTP client for transport
- **tokio**: 1.46 - Async runtime
- **tokio-util**: 0.7 - Tokio utilities
- **tracing**: 0.1 - Logging framework
- **tracing-subscriber**: 0.3 - Logging configuration
- **url**: 2.5 - URL parsing

### No Shared Dependencies
The analysis shows clean separation - no dependencies are shared between the extension and bridge, making the split straightforward.

## File Mapping

### Files Staying in deepwiki-mcp-server
- `src/lib.rs` - Extension implementation (228 lines)
- `src/tests.rs` - Extension test suite (24 tests)
- `extension.toml` - Zed extension manifest
- `configuration/schema.json` - UI configuration schema
- `Cargo.toml` - Root package (will be updated to remove bridge)
- `lefthook.yml` - Development tools configuration
- `README.md` - Extension-focused documentation
- `LICENSE` - MIT license (copy to both repos)

### Files Moving to zed-mcp-proxy
- `crates/bridge/src/main.rs` - MCP proxy implementation (695 lines, 11 tests)
- `crates/bridge/Cargo.toml` - Bridge configuration (will become root Cargo.toml)

### Files to be Created/Updated
- `zed-mcp-proxy/README.md` - New proxy-focused documentation
- `zed-mcp-proxy/.github/workflows/` - New CI/CD for cross-platform builds
- `zed-mcp-proxy/release.toml` - Release automation configuration
- Updated extension code to reference new proxy repository

## Repository Relationship

### Current Structure
```
deepwiki-mcp-server (monorepo)
├── Extension (WASM)
└── Bridge (Native Binary)
```

### Target Structure
```
deepwiki-mcp-server (extension repo)
└── Extension (WASM) → downloads proxy from releases

zed-mcp-proxy (proxy repo)
└── MCP Proxy Binary (Native) → publishes releases
```

### Integration Points
- Extension downloads proxy binary from `zed-mcp-proxy` GitHub releases
- Version compatibility maintained through semantic versioning
- Cross-repository documentation references

## Technical Requirements

### zed-mcp-proxy Repository
- **Language**: Rust (standalone binary)
- **Targets**: x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc, x86_64-apple-darwin, aarch64-apple-darwin
- **CI/CD**: GitHub Actions with cross-compilation matrix
- **Release**: Automated binary builds with GitHub releases
- **Testing**: Unit tests, integration tests, cross-platform validation

### deepwiki-mcp-server Repository (Updated)
- **Language**: Rust (WASM target)
- **Target**: wasm32-wasip1 (Zed extension)
- **CI/CD**: GitHub Actions for WASM compilation
- **Dependencies**: Updated to download from zed-mcp-proxy releases
- **Testing**: Extension-specific tests, Zed integration validation

## Modern Rust Practices Applied

### Development Workflow
- **Lefthook**: Automated Git hooks for formatting, linting, testing
- **cargo fmt**: Consistent code formatting
- **cargo clippy**: Comprehensive linting with strict warnings
- **cargo test**: Automated test execution

### CI/CD Pipeline
- **GitHub Actions**: Modern workflow with caching
- **Cross-compilation**: Multi-platform binary builds
- **Automated releases**: Semantic versioning with cargo-release
- **Quality gates**: Formatting, linting, testing, security scanning

### Tooling
- **No custom build scripts**: Pure Cargo commands
- **Modern dependencies**: Latest stable versions
- **Security scanning**: cargo-audit for vulnerability detection
- **Dependency management**: cargo-deny for policy enforcement

## Commit History Preservation

### Strategy
- Use **git-filter-repo** for clean history extraction
- Preserve all commits related to bridge development
- Clean up commit messages to remove component-specific prefixes
- Maintain proper attribution and timestamps

### Implementation
```bash
# Extract bridge with history
git filter-repo --subdirectory-filter crates/bridge

# Clean up commit messages
git filter-repo --message-callback 'return re.sub(b"\\[(bridge|proxy)\\]\\s*", b"", message)'
```

## Migration Timeline

1. **Phase 1**: Extract bridge repository with history preservation
2. **Phase 2**: Set up modern CI/CD for zed-mcp-proxy  
3. **Phase 3**: Update extension to reference new proxy repository
4. **Phase 4**: Comprehensive testing and validation
5. **Phase 5**: Documentation updates and cleanup

## Success Criteria

- [x] Both repositories compile and test successfully
- [x] Extension can download proxy binary from new repository
- [x] Complete commit history preserved in both repositories
- [x] Modern Rust development workflow established
- [x] Cross-platform builds working for proxy
- [x] All existing functionality maintained

## Notes

- Clean dependency separation makes this migration low-risk
- No breaking changes to end-user functionality
- Improved maintainability with focused repositories
- Better CI/CD pipeline efficiency
- Enhanced development workflow with modern tooling