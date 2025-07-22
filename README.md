# DeepWiki MCP Server Extension for Zed

A **Model Context Protocol (MCP) server extension** for the Zed IDE that provides seamless integration with DeepWiki and Devin AI documentation services.

## 🏗️ Architecture

This extension uses a **two-part architecture** optimized for Zed's WebAssembly-based extension system:

```
Zed ↔ Extension (WASM) → Bridge Binary (Native) ↔ HTTP MCP Server
```

### Components

1. **Extension (WASM)** - `crates/extension/`
   - Lightweight Zed extension compiled to WebAssembly
   - Provides configuration UI and command setup
   - No async/HTTP dependencies (WASM-compatible)

2. **Bridge Binary (Native)** - `crates/bridge/`
   - Standalone native binary with full HTTP/async capabilities
   - Translates between STDIO (Zed) and HTTP (DeepWiki/Devin)
   - Handles MCP protocol communication

## 🚀 Features

- **Free DeepWiki Access**: Query public repository documentation
- **Devin AI Integration**: Enhanced AI-powered documentation with API key
- **Type-Safe Configuration**: JSON schema validation for settings
- **Secure Authentication**: Environment variable-based API key handling
- **Protocol Compliance**: Full MCP (Model Context Protocol) support

## 🛠️ Installation

### Prerequisites

- Rust toolchain (latest stable)
- Zed IDE
- WASM target: `rustup target add wasm32-wasip1`

### Build from Source

1. **Clone the repository**:
   ```bash
   git clone https://github.com/keshav1998/deepwiki-mcp-server
   cd deepwiki-mcp-server
   ```

2. **Build everything**:
   ```bash
   ./build.sh
   ```

3. **Install the bridge binary**:
   ```bash
   cp dist/bin/deepwiki-mcp-bridge ~/.local/bin/
   # or to /usr/local/bin/ for system-wide access
   ```

4. **Install the extension in Zed**:
   ```bash
   # Method 1: Direct installation
   zed --install-extension ./dist
   
   # Method 2: Manual installation
   cp -r dist/* ~/.config/zed/extensions/deepwiki-mcp-server/
   ```

## ⚙️ Configuration

### Basic Setup (Free DeepWiki)

Add to your Zed `settings.json`:

```json
{
  "context_servers": {
    "deepwiki-mcp-server": {
      "endpoint": "https://mcp.deepwiki.com",
      "protocol": "mcp"
    }
  }
}
```

### Advanced Setup (Devin AI with Authentication)

```json
{
  "context_servers": {
    "deepwiki-mcp-server": {
      "endpoint": "https://mcp.devin.ai",
      "protocol": "mcp",
      "devin_api_key": "your-api-key-here"
    }
  }
}
```

### Environment Variables

For secure API key management:

```bash
export DEVIN_API_KEY="your-api-key-here"
```

## 🔧 Development

### Project Structure

```
deepwiki-mcp-server/
├── crates/
│   ├── extension/          # Zed WASM extension
│   │   ├── src/
│   │   │   ├── lib.rs     # Extension implementation
│   │   │   └── tests.rs   # Extension tests
│   │   └── configuration/ # UI configuration files
│   └── bridge/            # Native bridge binary
│       └── src/
│           ├── main.rs    # Bridge entry point
│           └── mcp_bridge/ # MCP protocol implementation
├── extension.toml         # Zed extension manifest
├── build.sh              # Build script
└── scripts/              # Legacy shell scripts (deprecated)
```

### Building Individual Components

```bash
# Build extension (WASM)
cargo build --manifest-path crates/extension/Cargo.toml --target wasm32-wasip1

# Build bridge (native)
cargo build --manifest-path crates/bridge/Cargo.toml --release

# Run tests
cargo test --workspace
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace --tests -- -D warnings

# Fix clippy issues automatically
cargo clippy --fix --allow-dirty --allow-staged --workspace
```

## 🏃‍♂️ Usage

Once installed and configured:

1. **Open a project in Zed**
2. **Access the assistant panel**
3. **Use context-aware queries** like:
   - "How do I implement authentication in this codebase?"
   - "Show me examples of error handling patterns"
   - "What are the available API endpoints?"

The extension will automatically query relevant documentation and provide contextual answers.

## 🔍 MCP Protocol Support

This extension implements the full **Model Context Protocol v2024-11-05** specification:

- ✅ **Tools**: Interactive documentation queries
- ✅ **Resources**: Repository file access
- ✅ **Prompts**: Predefined query templates
- ✅ **Initialization**: Capability negotiation
- ✅ **Error Handling**: Robust error reporting

## 🛡️ Security

- **WASM Sandboxing**: Extension runs in WebAssembly sandbox
- **Process Isolation**: Bridge runs as separate process
- **Environment Variables**: Sensitive data via env vars only
- **No Hardcoded Secrets**: All credentials externally managed
- **Capability-Based**: Fine-grained permission system

## 🐛 Troubleshooting

### Common Issues

1. **Bridge binary not found**:
   ```bash
   # Ensure bridge is in PATH
   which deepwiki-mcp-bridge
   # Should return: /home/user/.local/bin/deepwiki-mcp-bridge
   ```

2. **WASM compilation fails**:
   ```bash
   # Install WASM target
   rustup target add wasm32-wasip1
   ```

3. **Authentication errors**:
   ```bash
   # Check API key
   echo $DEVIN_API_KEY
   # Verify endpoint in settings
   ```

### Debug Mode

Enable debug logging:

```bash
export DEBUG=1
export RUST_LOG=debug
```

Then check Zed's extension logs for detailed information.

## 🤝 Contributing

We welcome contributions! Please see our development guidelines:

### Code Standards

- **Rust Best Practices**: Follow Rust API guidelines
- **Security First**: No hardcoded secrets, validate inputs
- **WASM Compatibility**: Keep extension dependencies minimal
- **Type Safety**: Use strong typing throughout
- **Error Handling**: Comprehensive error handling with context

### Development Workflow

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make changes following our coding standards**
4. **Add tests for new functionality**
5. **Run the full test suite**: `cargo test --workspace`
6. **Submit a pull request**

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Zed Team**: For the excellent extension API and WebAssembly architecture
- **MCP Specification**: For providing a robust protocol for AI tool integration
- **DeepWiki & Devin**: For providing the documentation and AI services

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/keshav1998/deepwiki-mcp-server/issues)
- **Discussions**: [GitHub Discussions](https://github.com/keshav1998/deepwiki-mcp-server/discussions)
- **Email**: [me@kmsh.dev](mailto:me@kmsh.dev)

---

**Built with ❤️ for the Zed community**