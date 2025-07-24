# DeepWiki MCP Proxy Handoff Documentation

## Executive Summary

The DeepWiki MCP Server Extension for Zed has been successfully separated into two distinct components: a lightweight WASM-based Zed extension (`deepwiki-mcp-server`) and a native binary proxy (`zed-mcp-proxy`). This separation allows for better maintainability, platform-specific optimizations, and cleaner architecture while maintaining full functionality.

The `zed-mcp-proxy` component serves as the critical bridge between the Zed editor and MCP-compatible documentation servers, handling all protocol-specific communication, authentication, and transport management. This handoff document provides comprehensive information on how to use, maintain, and further develop the proxy component in relation to the Zed extension.

## Architecture Overview

The system uses a separated architecture with two distinct components:

1. **Zed Extension (WASM)** - `deepwiki-mcp-server`
   - Lightweight WebAssembly component running inside Zed editor
   - Handles UI configuration and extension setup
   - Automatically manages proxy binary lifecycle

2. **MCP Proxy (Native Binary)** - `zed-mcp-proxy`
   - Standalone Rust binary with full HTTP/async capabilities
   - Acts as bridge between Zed and MCP servers
   - Handles protocol translation and authentication

```
Zed Editor ↔ Extension (WASM) → Auto-Downloaded Proxy (Native) ↔ HTTP MCP Server
```

## Proxy Binary Responsibilities

The `zed-mcp-proxy` binary serves as the critical bridge component with these key responsibilities:

1. **Protocol Translation**
   - Converts between STDIO (from Zed) and HTTP/SSE (to MCP servers)
   - Implements full Model Context Protocol (MCP) specification
   - Handles JSON-RPC message formatting and parsing

2. **Transport Management**
   - Automatically detects and configures transport type (HTTP or SSE)
   - Manages persistent connections to MCP endpoints
   - Handles timeout and reconnection logic

3. **Authentication**
   - Supports API key authentication for Devin AI service
   - Securely processes environment variables for credentials
   - Maintains authentication state during session

4. **Error Handling**
   - Provides detailed error reporting back to extension
   - Handles network failures and protocol errors gracefully
   - Implements proper logging for troubleshooting

## Extension-Proxy Integration

The extension automatically manages the proxy binary:

1. **Binary Download**
   - Extension determines current platform (OS and architecture)
   - Downloads appropriate binary from GitHub releases
   - Handles platform-specific archive extraction (.tar.gz or .zip)
   - Sets executable permissions where needed

2. **Binary Invocation**
   - Extension constructs command with user configuration
   - Passes MCP endpoint URL as command-line argument
   - Routes STDIO between Zed, extension, and proxy

3. **Configuration Management**
   - Extension collects user settings (endpoint, credentials)
   - Validates configuration before passing to proxy
   - Provides default values for optional settings

## Cross-Platform Support

The proxy is built for multiple platforms:

- **Linux**: x86_64, ARM64
- **macOS**: Intel (x86_64), Apple Silicon (ARM64)
- **Windows**: x86_64

Asset naming convention: `zed-mcp-proxy-{arch}-{os}.{ext}`
- Example: `zed-mcp-proxy-x86_64-apple-darwin.tar.gz`

## MCP Protocol Implementation

The proxy implements the full Model Context Protocol (v2024-11-05):

1. **Initialization**
   - Capability negotiation
   - Protocol version validation
   - Session setup

2. **Core Features**
   - Tools for interactive documentation queries
   - Resources for repository file access
   - Prompts for predefined query templates

3. **Transport Options**
   - HTTP with JSON payloads
   - Server-Sent Events (SSE) for streaming responses
   - Automatic detection based on endpoint URL pattern

## Communication Flow

The detailed communication flow between components works as follows:

1. **User Request**
   - User submits a documentation query in Zed's assistant panel
   - Zed routes request to the extension via WASM interface

2. **Extension Processing**
   - Extension validates request format and configuration
   - Prepares command to invoke proxy binary with proper arguments
   - Sets up STDIO channels for bidirectional communication

3. **Proxy Execution**
   - Proxy establishes HTTP/SSE connection to configured MCP endpoint
   - Translates STDIO messages to HTTP/JSON requests
   - Manages authentication headers when needed

4. **MCP Server Communication**
   - Proxy follows MCP protocol for request/response
   - Handles streaming responses when available
   - Processes resource requests for file access

5. **Response Handling**
   - Proxy converts HTTP/SSE responses back to STDIO format
   - Extension processes responses and sends to Zed
   - User receives formatted documentation

## Deployment Flow

When a user installs the Zed extension:

1. Extension is loaded by Zed (WASM module)
2. On first context server command request:
   - Extension checks for proxy binary in `bin/` directory
   - If not found, downloads from GitHub releases
   - Makes binary executable (Unix systems)
3. Extension constructs command with user configuration
4. Zed spawns proxy process with STDIO connected to extension
5. Proxy connects to configured MCP server endpoint
6. MCP protocol communication begins

## Configuration Options

The proxy accepts these command-line arguments:

```
zed-mcp-proxy <endpoint-url>
```

Where:
- `endpoint-url` is the MCP server to connect to (e.g., `https://mcp.deepwiki.com`)

Environment variables supported:
- `DEVIN_API_KEY` - Authentication for Devin AI service
- `DEBUG` - Enable debug logging (set to "1")
- `RUST_LOG` - Configure logging level (e.g., "debug")

## Development Guidelines

When working on the proxy:

1. **Maintain Strict Binary Interface**
   - Always accept endpoint URL as first argument
   - Use STDIO for all extension communication
   - Maintain backward compatibility with extension

2. **Error Handling**
   - Provide clear error messages via stderr
   - Follow proper exit codes for different error types
   - Include context information in error messages

3. **Protocol Compliance**
   - Strictly adhere to MCP specification
   - Use the official Rust MCP SDK for protocol implementation
   - Support all required MCP capabilities

4. **Cross-Platform Compatibility**
   - Test on all supported platforms before release
   - Ensure proper error handling on all OSes
   - Maintain CI/CD workflows for cross-platform builds

## Testing

The proxy should pass these integration tests:

1. **Basic Operation**
   - Shows usage when no arguments provided
   - Successfully detects HTTP transport for regular URLs
   - Successfully detects SSE transport for /sse URLs

2. **Protocol Handling**
   - Processes MCP initialization messages
   - Attempts connection to configured endpoint
   - Reports transport selection and connection status

3. **Error Scenarios**
   - Gracefully handles invalid endpoints
   - Reports clear error messages for connection failures
   - Maintains clean shutdown on error conditions

## Troubleshooting Guide

Common issues and solutions:

### Proxy Binary Download Failures

**Symptoms:**
- Extension reports "Failed to download proxy binary"
- "No binary found for platform" error message

**Solutions:**
1. Check internet connectivity and GitHub access
2. Verify GitHub release assets follow naming convention
3. Check file permissions in Zed's extension directory
4. Manually download binary from GitHub and place in `bin/` directory

### Communication Errors

**Symptoms:**
- "Failed to connect to MCP server" errors
- Silent failures with no responses
- Extension timeout errors

**Solutions:**
1. Verify endpoint URL is correct and accessible
2. Check for API key validity if using authenticated service
3. Inspect proxy logs with `DEBUG=1 RUST_LOG=debug`
4. Verify proxy binary has execute permissions

### Protocol Version Mismatches

**Symptoms:**
- "Unsupported protocol version" errors
- "Capability negotiation failed" messages

**Solutions:**
1. Update proxy to latest version
2. Verify MCP server supports the protocol version used
3. Check for version compatibility in error messages

## Example Usage

### Basic Configuration (Zed settings.json)

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

### Authentication Configuration (Zed settings.json)

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

### Manual Proxy Invocation (for testing)

```bash
# Basic usage
./zed-mcp-proxy https://mcp.deepwiki.com

# With debug logging
DEBUG=1 RUST_LOG=debug ./zed-mcp-proxy https://mcp.deepwiki.com

# With authentication
DEVIN_API_KEY=your-api-key-here ./zed-mcp-proxy https://mcp.devin.ai
```

## Maintenance Responsibilities

The maintainer of `zed-mcp-proxy` is responsible for:

1. Building and releasing proxy binaries for all platforms
2. Maintaining GitHub release assets with consistent naming
3. Updating the proxy to support new MCP protocol versions
4. Ensuring compatibility with the Zed extension
5. Addressing security vulnerabilities and dependency updates
6. Providing detailed release notes for version changes

## Future Enhancements

Planned improvements for the proxy:

1. Enhanced authentication mechanisms
2. Support for custom headers and proxies
3. Advanced logging and diagnostics
4. Performance optimizations for large responses
5. Additional transport options (WebSockets, gRPC)
6. Custom MCP capabilities for DeepWiki-specific features

## Contact and Support

For issues with the proxy:
- GitHub Issues: https://github.com/keshav1998/zed-mcp-proxy/issues
- Email: me@kmsh.dev

For extension-related questions:
- GitHub Issues: https://github.com/keshav1998/deepwiki-mcp-server/issues

## Version Compatibility Matrix

| zed-mcp-proxy | deepwiki-mcp-server | MCP Protocol | Features                    |
|---------------|---------------------|--------------|----------------------------|
| 1.0.x         | 1.0.x               | 2024-11-05   | Basic documentation queries |
| 1.1.x         | 1.1.x               | 2024-11-05   | Authentication support      |
| 1.2.x         | 1.2.x - 1.3.x       | 2024-11-05   | Enhanced error handling     |
| 2.0.x         | 2.0.x               | 2025+        | Advanced capabilities       |

Always ensure that proxy and extension versions are compatible according to this matrix.

## Performance Considerations

- The proxy is designed to be lightweight and resource-efficient
- Memory usage: ~10-20MB during normal operation
- Startup time: <100ms on modern systems
- Designed for long-running sessions with stable memory usage
- Auto-terminates when Zed closes or extension is disabled

## Security Best Practices

1. **API Key Handling**
   - Never hardcode API keys in source code
   - Always use environment variables or secure configuration
   - Proxy sanitizes logs to prevent credential leakage

2. **Communication Security**
   - All HTTP communication uses TLS (HTTPS)
   - No sensitive data persisted to disk
   - Process isolation limits security exposure

3. **Update Management**
   - Regularly update proxy to latest version
   - Monitor security advisories for dependencies
   - Follow GitHub security alerts