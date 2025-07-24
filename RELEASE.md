# Release Management Guide

This document outlines the release process for both the DeepWiki MCP Server Extension and its associated proxy binary.

## 📋 Overview

The project uses a **separated architecture** with two independent release cycles:

1. **Extension Repository** (`deepwiki-mcp-server`) - Zed extension (WASM)
2. **Proxy Repository** (`zed-mcp-proxy`) - Native binary

Both repositories use `cargo-release` for automated release management with semantic versioning.

## 🏗️ Architecture and Release Strategy

### Release Dependencies

```
Extension Release ← depends on ← Proxy Release
```

- **Proxy releases** are independent and create GitHub releases with cross-platform binaries
- **Extension releases** reference specific proxy versions through binary download URLs

### Version Alignment

- **Proxy**: Uses standard semantic versioning (`v1.0.0`, `v1.1.0`, etc.)
- **Extension**: Uses extension-prefixed versioning (`extension-v1.0.0`)
- **Compatibility**: Extension specifies compatible proxy versions in release notes

## 🚀 Release Process

### Prerequisites

1. **Install cargo-release**:
   ```bash
   cargo install cargo-release
   ```

2. **Clean working directory**:
   ```bash
   git status  # Should show no uncommitted changes
   ```

3. **Update dependencies**:
   ```bash
   cargo update
   ```

### Proxy Repository Release

1. **Navigate to proxy directory**:
   ```bash
   cd temp-bridge-extraction  # or actual proxy repo
   ```

2. **Prepare release**:
   - Update CHANGELOG.md with new features and fixes
   - Ensure all tests pass: `cargo test`
   - Verify cross-compilation: `cargo check --target x86_64-unknown-linux-gnu`

3. **Create release**:
   ```bash
   # Patch release (0.1.0 -> 0.1.1)
   cargo release patch

   # Minor release (0.1.x -> 0.2.0)
   cargo release minor

   # Major release (0.x.y -> 1.0.0)
   cargo release major
   ```

4. **Verify release**:
   - Check that tags are created: `git tag -l`
   - Verify CHANGELOG.md updates
   - Confirm CI/CD builds binaries for all platforms

### Extension Repository Release

1. **Update proxy reference** (if needed):
   - Update binary download URLs in `src/lib.rs` if using new proxy version
   - Update documentation references

2. **Prepare release**:
   ```bash
   cargo fmt --check
   cargo clippy --target wasm32-wasip1 -- -D warnings
   cargo nextest run
   cargo build --target wasm32-wasip1 --release
   ```

3. **Create release**:
   ```bash
   # Patch release (0.1.0 -> 0.1.1)
   cargo release patch

   # Minor release (0.1.x -> 0.2.0)  
   cargo release minor

   # Major release (0.x.y -> 1.0.0)
   cargo release major
   ```

4. **Verify release**:
   - Check `extension.toml` version update
   - Verify CHANGELOG.md updates
   - Test extension loading in Zed

## 📋 Release Checklist

### Pre-Release (Both Repositories)

- [ ] All tests passing (`cargo nextest run`)
- [ ] No linting errors (`cargo clippy`)
- [ ] Code formatted (`cargo fmt --check`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated with changes
- [ ] Version compatibility verified

### Proxy Release Specific

- [ ] Cross-platform compilation verified
- [ ] Integration tests with MCP servers passing
- [ ] OAuth2 authentication tested (if applicable)
- [ ] Binary size optimized (`cargo build --release`)
- [ ] Performance benchmarks acceptable

### Extension Release Specific

- [ ] WASM compilation successful
- [ ] Zed extension API compatibility verified
- [ ] Configuration schema validated
- [ ] Binary download URLs updated (if proxy version changed)
- [ ] Platform-specific asset names correct

### Post-Release

- [ ] Git tags created and pushed
- [ ] GitHub releases created (for proxy)
- [ ] Release notes published
- [ ] Documentation updated
- [ ] Zed extension registry updated (if applicable)

## 🔧 Configuration Files

### Proxy Release Configuration (`release.toml`)

```toml
# Enable semantic versioning and changelog management
pre-release-replacements = [
    { file = "CHANGELOG.md", search = "## \\[Unreleased\\]", replace = "## [Unreleased]\n\n## [{{version}}] - {{date}}" },
]

# Configure git operations
allow-branch = ["*"]
sign-commit = false
sign-tag = false
publish = false
tag-message = "Release {{version}}"
tag-prefix = "v"
dependent-version = "upgrade"
```

### Extension Release Configuration (`release.toml`)

```toml
# Enable semantic versioning and changelog management
pre-release-replacements = [
    { file = "CHANGELOG.md", search = "## \\[Unreleased\\]", replace = "## [Unreleased]\n\n## [{{version}}] - {{date}}" },
    { file = "extension.toml", search = "version = \".*\"", replace = "version = \"{{version}}\"" },
    { file = "Cargo.toml", search = "version = \".*\"", replace = "version = \"{{version}}\"" },
]

# Configure git operations for extension repository
allow-branch = ["main", "master"]
sign-commit = false
sign-tag = false
push-remote = "origin"
publish = false
tag-message = "Release deepwiki-mcp-server extension {{version}}"
tag-prefix = "extension-v"

# Pre-release hooks for WASM extension quality assurance
pre-release-hook = [
    "cargo fmt --check",
    "cargo clippy --target wasm32-wasip1 -- -D warnings",
    "cargo nextest run",
    "cargo build --target wasm32-wasip1 --release",
]
dependent-version = "upgrade"
```

## 🐛 Troubleshooting

### Common Issues

1. **Uncommitted changes error**:
   ```bash
   git add .
   git commit -m "Prepare for release"
   ```

2. **Missing CHANGELOG.md**:
   - Ensure CHANGELOG.md exists with `## [Unreleased]` section
   - Check file permissions

3. **Pre-release hook failures**:
   - Fix any failing tests or linting issues
   - Ensure all dependencies are up to date

4. **Git tag conflicts**:
   ```bash
   git tag -d v1.0.0  # Delete local tag
   git push origin :refs/tags/v1.0.0  # Delete remote tag
   ```

### Validation Commands

```bash
# Validate release configuration
cargo release config

# Test release process without making changes
cargo release --no-verify --no-push --no-tag patch

# Check current version
cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="deepwiki-mcp-server") | .version'
```

## 📚 References

- [cargo-release Documentation](https://github.com/crate-ci/cargo-release)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)

## 🔐 Security Considerations

- Never include API keys or secrets in release artifacts
- Verify binary signatures for proxy releases
- Use environment variables for sensitive configuration
- Review dependency updates for security vulnerabilities

---

**Note**: This document should be updated whenever the release process changes or new automation is added.