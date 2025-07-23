# DeepWiki MCP Server Extension for Zed

A **Model Context Protocol (MCP) server extension** for the Zed IDE that provides seamless integration with DeepWiki and Devin AI documentation services.

## 🏗️ Architecture

This extension uses a **separated architecture** with automatic binary download:

```
Zed ↔ Extension (WASM) → Auto-Downloaded Proxy (Native) ↔ HTTP MCP Server
```

### Components

1. **Extension (WASM)** - This repository
   - Lightweight Zed extension compiled to WebAssembly
   - Automatically downloads platform-specific proxy binary
   - Provides configuration UI and command setup
   - No async/HTTP dependencies (WASM-compatible)

2. **Proxy Binary (Native)** - [zed-mcp-proxy](https://github.com/keshav1998/zed-mcp-proxy)
   - Auto-downloaded from separate repository releases
   - Full HTTP/async capabilities with tokio and reqwest
   - Translates between STDIO (Zed) and HTTP (DeepWiki/Devin)
   - Handles MCP protocol communication with official Rust MCP SDK

## 🚀 Features

- **Free DeepWiki Access**: Query public repository documentation
- **Devin AI Integration**: Enhanced AI-powered documentation with API key
- **Automatic Setup**: Bridge binary downloaded automatically per platform
- **Type-Safe Configuration**: JSON schema validation for settings
- **Secure Authentication**: Environment variable-based API key handling
- **Protocol Compliance**: Full MCP (Model Context Protocol) support
- **Cross-Platform**: Supports Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), Windows
- **Separated Concerns**: Extension focuses on Zed integration, proxy handles MCP protocol

## 🛠️ Installation

### Prerequisites

- Rust toolchain (latest stable)
- Zed IDE
- WASM target: `rustup target add wasm32-wasip1`
- The proxy binary is downloaded automatically - no separate installation needed

### Install from Zed Extensions Registry (Recommended)

1. **Open Zed**
2. **Open Command Palette** (`Cmd/Ctrl+Shift+P`)
3. **Type**: "zed: extensions"
4. **Search for**: "DeepWiki MCP"
5. **Click Install**

The extension will automatically download the appropriate proxy binary from the [zed-mcp-proxy repository](https://github.com/keshav1998/zed-mcp-proxy) for your platform when first used.

### Build from Source (Advanced)

If you want to build from source:

1. **Clone the repository**:
   ```bash
   git clone https://github.com/keshav1998/deepwiki-mcp-server
   cd deepwiki-mcp-server
   ```

2. **Build the extension**:
   ```bash
   cargo build --target wasm32-wasip1 --release
   ```

3. **Install as dev extension**:
   - In Zed, use "Extensions: Install Dev Extension"
   - Select this project directory
   - The proxy binary will be downloaded automatically on first use

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
├── src/
│   ├── lib.rs             # Extension implementation
│   └── tests.rs           # Extension tests
├── configuration/         # UI configuration files
├── extension.toml         # Zed extension manifest
├── lefthook.yml          # Git hooks configuration
├── BUILD.md              # Build instructions
└── Cargo.toml            # Package configuration
```

**Note**: The proxy binary is maintained in a separate repository: [zed-mcp-proxy](https://github.com/keshav1998/zed-mcp-proxy)

### Building the Extension

```bash
# Build extension (WASM)
cargo build --target wasm32-wasip1 --release

# Development build
cargo build --target wasm32-wasip1

# Run tests
cargo test
```

See [BUILD.md](BUILD.md) for detailed build instructions and modern Rust development practices.

### Code Quality

```bash
# Format code
cargo fmt --all

# Run clippy (WASM-specific)
cargo clippy --target wasm32-wasip1 -- -D warnings

# Fix clippy issues automatically
cargo clippy --fix --allow-dirty --allow-staged --target wasm32-wasip1
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

1. **Automatic download failed**:
   - Check internet connectivity and GitHub access to zed-mcp-proxy repository
   - Restart Zed to retry the download
   - Check Zed's extension logs for details

2. **Proxy binary not working**:
   - The extension automatically handles binary installation from zed-mcp-proxy releases
   - If issues persist, try reinstalling the extension
   - Check platform compatibility (Linux x86_64/ARM64, macOS Intel/Apple Silicon, Windows x86_64)

3. **Authentication errors**:
   ```bash
   # Check API key in settings
   # Verify endpoint configuration
   ```

4. **Manual proxy installation needed**:
   ```bash
   # Download from: https://github.com/keshav1998/zed-mcp-proxy/releases
   # Extract to extension's bin/ directory as 'zed-mcp-proxy' (or 'zed-mcp-proxy.exe' on Windows)
   # Extension handles the rest automatically
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
- **Separation of Concerns**: Extension for Zed integration, proxy for MCP protocol
- **Type Safety**: Use strong typing throughout
- **Error Handling**: Comprehensive error handling with context

### Development Workflow

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make changes following our coding standards**
4. **Add tests for new functionality**
5. **Run the full test suite**: `cargo test`
6. **Submit a pull request**

### Proxy Development

For changes to the MCP proxy functionality, contribute to the [zed-mcp-proxy repository](https://github.com/keshav1998/zed-mcp-proxy).

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