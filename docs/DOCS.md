# Documentation Hub

Welcome to the **DeepWiki MCP Server Extension** documentation. This page serves as a central hub for all documentation related to the extension and its associated proxy binary.

## 📚 Documentation Structure

### Core Documentation

- **[README.md](README.md)** - Main project documentation with installation, configuration, and usage instructions
- **[BUILD.md](BUILD.md)** - Detailed build instructions and development setup
- **[API Documentation](#api-documentation)** - Generated Rust API documentation

### Repository Architecture

This project follows a **separated architecture** pattern:

1. **Extension Repository** (This repository)
   - Zed extension compiled to WebAssembly
   - Handles Zed integration and configuration
   - Automatically downloads and manages proxy binary

2. **Proxy Repository** - [zed-mcp-proxy](https://github.com/keshav1998/zed-mcp-proxy)
   - Native binary handling MCP protocol
   - Built with official Rust MCP SDK
   - Cross-platform releases

## 🔧 API Documentation

### Extension Library Documentation

The extension's Rust API documentation is generated using `cargo doc`:

```bash
# Generate extension documentation
cargo doc --target wasm32-wasip1 --no-deps --open
```

**Generated Documentation**: `target/wasm32-wasip1/doc/deepwiki_mcp_server/index.html`

#### Key Types and Functions

- **`DeepWikiMcpExtension`** - Main extension struct implementing Zed's Extension trait
- **`DeepWikiContextServerSettings`** - Configuration settings with JSON schema
- **Binary Management** - Automatic download and platform detection functions

### Proxy Binary Documentation

The proxy binary's documentation covers the MCP protocol implementation:

```bash
# Generate proxy documentation (from temp-bridge-extraction/)
cd temp-bridge-extraction
cargo doc --no-deps --open
```

**Generated Documentation**: `temp-bridge-extraction/target/doc/zed_mcp_proxy/index.html`

## 🏗️ Architecture Documentation

### High-Level Architecture

```text
┌─────────────────┐    Configuration    ┌──────────────────┐
│      User       │ ─────────────────► │  Zed Settings    │
│   (Developer)   │                     │     (JSON)       │
└─────────────────┘                     └──────────────────┘
                                                 │
                                                 ▼
┌─────────────────┐    Extension API    ┌──────────────────┐
│   Zed Editor    │ ◄─────────────────► │ Extension (WASM) │
│                 │                     │ deepwiki-mcp-    │
│                 │                     │    server        │
└─────────────────┘                     └──────────────────┘
                                                 │
                                          Auto-downloads
                                           and manages
                                                 ▼
                                        ┌──────────────────┐
                                        │ Proxy Binary     │
                                        │ zed-mcp-proxy    │
                                        │    (Native)      │
                                        └──────────────────┘
                                                 │
                                        STDIO ↕ HTTP/SSE
                                                 ▼
                                        ┌──────────────────┐
                                        │   MCP Server     │
                                        │                  │
                                        │ • DeepWiki       │
                                        │ • Devin AI       │
                                        └──────────────────┘
```

### Component Responsibilities

#### Extension (WASM)
- **Platform Detection**: Identifies user's OS and architecture
- **Binary Management**: Downloads appropriate proxy binary from GitHub releases
- **Configuration Parsing**: Validates user settings with JSON schema
- **Command Generation**: Creates proper command-line invocation for Zed
- **Zed Integration**: Implements Extension trait for seamless IDE integration

#### Proxy Binary (Native)
- **Protocol Bridge**: Translates between STDIO (Zed) and HTTP/SSE (MCP servers)
- **Transport Detection**: Automatically selects HTTP or SSE based on endpoint URL
- **Authentication**: Handles OAuth2 flows for protected endpoints
- **Message Forwarding**: Bidirectional async message passing
- **Error Handling**: Comprehensive error reporting and recovery

### Data Flow

1. **Initialization**
   ```text
   User configures extension → Zed loads extension → Extension downloads proxy
   ```

2. **Runtime Communication**
   ```text
   Zed ↔ STDIO ↔ Proxy ↔ HTTP/SSE ↔ MCP Server
   ```

3. **Message Types**
   - **MCP Protocol Messages**: JSON-RPC 2.0 over transport layer
   - **Tool Calls**: Function invocations with parameters
   - **Resource Access**: File and data retrieval
   - **Authentication**: OAuth2 token exchange

## 🔍 Configuration Documentation

### Extension Settings Schema

The extension uses JSON Schema for configuration validation:

```json
{
  "type": "object",
  "properties": {
    "endpoint": {
      "type": "string",
      "description": "MCP server endpoint URL",
      "default": "https://mcp.deepwiki.com",
      "examples": [
        "https://mcp.deepwiki.com",
        "https://mcp.devin.ai"
      ]
    }
  }
}
```

### Supported Endpoints

| Endpoint | Description | Authentication | Transport |
|----------|-------------|----------------|-----------|
| `https://mcp.deepwiki.com` | Free public repository access | None | HTTP |
| `https://mcp.devin.ai` | Enhanced AI documentation | OAuth2 | HTTP |
| Custom `/sse` endpoints | Server-Sent Events | Configurable | SSE |

## 🧪 Testing Documentation

### Test Structure

```text
src/tests.rs
├── unit_tests/          # Core functionality tests
│   ├── Configuration parsing and validation
│   ├── Binary name generation
│   ├── Asset naming for different platforms
│   └── Settings serialization/deserialization
├── integration_tests/   # End-to-end functionality tests
│   ├── Extension-proxy integration
│   ├── Binary download and execution
│   ├── MCP protocol readiness
│   └── Cross-platform compatibility
```

### Running Tests

```bash
# Run all tests with modern test runner
cargo nextest run

# Run specific test categories
cargo test unit_tests
cargo test integration_tests

# Run with output for debugging
cargo test -- --nocapture
```

### Test Coverage

- **27 total tests** with 100% pass rate
- **Unit tests**: Configuration, binary management, platform detection
- **Integration tests**: Proxy functionality, MCP protocol, compatibility
- **Platform tests**: Cross-platform asset naming and binary execution

## 🚀 Development Documentation

### Development Workflow

1. **Setup Development Environment**
   ```bash
   # Install Rust toolchain
   rustup target add wasm32-wasip1
   
   # Install development tools
   cargo install cargo-nextest
   cargo install lefthook
   ```

2. **Code Quality Tools**
   ```bash
   # Format code
   cargo fmt --all
   
   # Lint code
   cargo clippy --target wasm32-wasip1 -- -D warnings
   
   # Run tests
   cargo nextest run
   ```

3. **Git Hooks (Lefthook)**
   - **Pre-commit**: Format, lint, and test before commits
   - **Pre-push**: Full test suite and WASM build verification
   - Configuration in `.config/lefthook.yml`

### Build Targets

#### Extension (WASM)
```bash
# Development build
cargo build --target wasm32-wasip1

# Release build
cargo build --target wasm32-wasip1 --release
```

#### Proxy Binary (Native)
```bash
# Development build (from temp-bridge-extraction/)
cd temp-bridge-extraction
cargo build

# Release build
cargo build --release
```

## 📖 External Documentation

### Related Resources

- **[Model Context Protocol Specification](https://modelcontextprotocol.io/)** - Official MCP protocol documentation
- **[Zed Extension API](https://zed.dev/docs/extensions)** - Zed's extension development guide
- **[rmcp Crate Documentation](https://docs.rs/rmcp/)** - Official Rust MCP SDK
- **[Rust WebAssembly Book](https://rustwasm.github.io/book/)** - WASM development with Rust

### Community and Support

- **[GitHub Issues](https://github.com/keshav1998/deepwiki-mcp-server/issues)** - Bug reports and feature requests
- **[GitHub Discussions](https://github.com/keshav1998/deepwiki-mcp-server/discussions)** - General questions and community support
- **[Zed Community](https://zed.dev/community)** - Zed editor community resources

## 📝 Contributing to Documentation

### Documentation Standards

1. **API Documentation**: Use comprehensive rustdoc comments
2. **README Updates**: Keep user-facing documentation current
3. **Architecture Changes**: Update this documentation hub
4. **Examples**: Provide working code examples
5. **Testing**: Document test procedures and coverage

### Documentation Build Process

```bash
# Generate all documentation
cargo doc --target wasm32-wasip1 --no-deps
cd temp-bridge-extraction && cargo doc --no-deps

# Verify documentation links
cargo doc --target wasm32-wasip1 --no-deps --open
```

### Style Guide

- Use **clear, concise language**
- Include **practical examples**
- Document **edge cases and error conditions**
- Maintain **consistency** across all documentation
- Update **both README and API docs** for changes

---

**Last Updated**: Generated automatically by documentation build process
**Generated Documentation**: Available in `target/*/doc/` directories after running `cargo doc`
