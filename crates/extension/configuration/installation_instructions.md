# DeepWiki MCP Server Extension Installation

This extension provides access to DeepWiki's documentation search capabilities through the Model Context Protocol (MCP).

## Automatic Setup

The extension automatically downloads and configures the required bridge binary when first used. No manual installation of additional components is required!

## Configuration

The extension will automatically download the appropriate bridge binary for your platform (Linux, macOS, or Windows) from GitHub releases.

### Basic Setup (Free - Public Repositories Only)

For access to public repositories only, no authentication is required:

```json
{
  "context_servers": {
    "deepwiki-mcp-server-extension": {
      "endpoint": "https://mcp.deepwiki.com",
      "protocol": "sse"
    }
  }
}
```

### Advanced Setup (Authenticated - Public & Private Repositories)

For access to both public and private repositories, you'll need a Devin API key:

1. Sign up for a Devin account at [Devin.ai](https://devin.ai)
2. Generate an API key from your account settings
3. Configure the extension with your API key:

```json
{
  "context_servers": {
    "deepwiki-mcp-server-extension": {
      "endpoint": "https://mcp.devin.ai",
      "protocol": "sse",
      "devin_api_key": "YOUR_DEVIN_API_KEY"
    }
  }
}
```

## Available Tools

The DeepWiki MCP server provides three main tools:

- **`read_wiki_structure`** - Get a list of documentation topics for a GitHub repository
- **`read_wiki_contents`** - View documentation about a GitHub repository
- **`ask_question`** - Ask any question about a GitHub repository and get an AI-powered, context-grounded response

## Wire Protocols

The server supports two wire protocols:

- **SSE (Server-Sent Events)** - Recommended for most integrations (`/sse` endpoint)
- **Streamable HTTP** - Alternative protocol (`/mcp` endpoint)

For maximum compatibility, the SSE protocol is used by default.

## Troubleshooting

### Common Issues

- **Binary Download Failed**: Ensure you have internet connectivity and GitHub is accessible
- **Permission Denied**: The extension automatically handles file permissions, but check if your system allows file downloads
- **Wrong Architecture**: The extension automatically detects your platform, but you can check the logs if there are issues

### Configuration Issues

- Ensure your Devin API key is valid if using the authenticated endpoint
- Check that the endpoint URL is correct for your chosen service
- Verify that the protocol setting matches your client's capabilities

### Manual Installation (Advanced)

If automatic download fails, you can manually install the bridge binary:

1. Download the appropriate binary from [GitHub Releases](https://github.com/keshav1998/deepwiki-mcp-server/releases)
2. Extract and place it in your extension's `bin/` directory
3. Make it executable (Unix systems): `chmod +x bin/deepwiki-mcp-bridge`

For more information, visit:
- [DeepWiki MCP Documentation](https://docs.devin.ai/work-with-devin/deepwiki-mcp)
- [Devin MCP Documentation](https://docs.devin.ai/work-with-devin/devin-mcp)
