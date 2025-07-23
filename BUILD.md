# Building DeepWiki MCP Server Extension

This document provides build instructions for the DeepWiki MCP Server Zed extension following modern Rust practices.

## Prerequisites

- Rust 1.70.0 or later
- `wasm32-wasip1` target installed:
  ```bash
  rustup target add wasm32-wasip1
  ```

## Standard Cargo Commands

### Development Builds

```bash
# Check compilation without building
cargo check --target wasm32-wasip1

# Full development build
cargo build --target wasm32-wasip1
```

### Release Builds

```bash
# Build optimized WASM binary for distribution
cargo build --target wasm32-wasip1 --release
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run lints
cargo clippy --target wasm32-wasip1

# Run lints with pedantic checks (strict mode)
cargo clippy --target wasm32-wasip1 -- -D warnings
```

## Development Workflow

The extension uses Lefthook for automated quality checks. When you commit, the following will run automatically:

- Code formatting validation
- WASM-specific clippy lints
- Full test suite
- Extension manifest validation

### Manual Quality Checks

If you want to run the same checks manually:

```bash
# Run all pre-commit hooks manually
lefthook run pre-commit

# Run individual hooks
lefthook run pre-commit format
lefthook run pre-commit clippy-wasm
lefthook run pre-commit test-workspace
```

## Extension Installation (Development)

### Using Zed's Install Dev Extension

1. Build the extension:
   ```bash
   cargo build --target wasm32-wasip1 --release
   ```

2. In Zed, open the command palette (Cmd+Shift+P / Ctrl+Shift+P)

3. Run "Extensions: Install Dev Extension"

4. Select this project directory

5. The extension will be compiled and installed automatically

### Manual Installation

```bash
# Build the extension
cargo build --target wasm32-wasip1 --release

# The built extension will be in:
# target/wasm32-wasip1/release/deepwiki_mcp_server.wasm
```

## Project Structure

```
deepwiki-mcp-server/
├── src/
│   ├── lib.rs              # Main extension implementation
│   └── tests.rs            # Test modules
├── configuration/          # Extension UI configuration
│   ├── default_settings.jsonc
│   └── installation_instructions.md
├── Cargo.toml             # Package configuration
├── extension.toml         # Zed extension manifest
└── lefthook.yml          # Git hooks configuration
```

## Build Artifacts

- **WASM binary**: Compiled extension for Zed
- **Proxy binary**: Downloaded automatically from zed-mcp-proxy releases
- **Configuration**: Embedded in the extension binary

## Troubleshooting

### WASM Target Missing

```bash
rustup target add wasm32-wasip1
```

### Build Fails with Linker Errors

Ensure you have the latest Rust toolchain:

```bash
rustup update
```

### Tests Fail

Check that all dependencies are properly installed:

```bash
cargo clean
cargo build --target wasm32-wasip1
cargo test
```

### Extension Not Loading in Zed

1. Check the Zed log for errors
2. Ensure the extension is built for the correct target
3. Verify the `extension.toml` configuration is valid

## Modern Rust Practices

This project follows 2025 Rust best practices:

- **No Custom Build Scripts**: Uses standard `cargo` commands only
- **WASM-First**: Optimized for WebAssembly compilation
- **Quality Gates**: Automated formatting, linting, and testing
- **Dependency Management**: Careful dependency selection for WASM compatibility
- **Professional Tooling**: Lefthook, clippy pedantic mode, comprehensive testing

## Related Documentation

- [Extension Configuration](configuration/installation_instructions.md)
- [Zed Extension API](https://zed.dev/docs/extensions)
- [zed-mcp-proxy Repository](https://github.com/keshav1998/zed-mcp-proxy)