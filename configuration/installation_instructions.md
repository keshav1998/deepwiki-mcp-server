# DeepWiki MCP Server Extension Installation

This extension provides access to DeepWiki's documentation search capabilities through the Model Context Protocol (MCP).


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
