# DeepWiki MCP Zed Extension

## Overview

**DeepWiki MCP** is a Rust-based Zed extension providing a Model Context Protocol (MCP) server to empower Zed's AI/Assistant ecosystem with external tool access, robust context, and streaming results. This project aims to enable rapid tool discovery and invocation, seamless integration with Zed's context server systems, and future extensibility for advanced AI workflows.

## Features

- **MCP Tool Discovery (`tools/list`)**: Schema-rich listing of all tools, Zed-assistant compatible.
- **Tool Invocation (`tools/call`)**: Safe, robust, streaming support for tool outputs.
- **Configurable Endpoints/Settings**: User/workspace config via Zed's `[context_servers]`.
- **Testable Rust Codebase**: Async streaming, JSON-RPC, and protocol compliance.
- **Automated Rust tests**: For protocol, integration, and edge case validation.
- **Reference-level compatibility**: Behavior modeled after Exa, Firecrawl, and Neon MCP Zed servers.

---

## Project Structure

```
deepwiki-mcp/
├── extension.toml               # Zed extension manifest
├── Cargo.toml                   # Rust crate/project metadata (cdylib for Zed)
├── src/
│   ├── main.rs                  # Standalone CLI (testing/prototyping MCP)
│   ├── lib.rs                   # Zed extension registration and context server glue
│   ├── mcp.rs                   # MCP protocol, tool registry/handlers, test stubs
│   └── tests.rs                 # Rust integration/unit tests for MCP and extension
├── configuration/
│   ├── default_settings.jsonc   # (optional) Sample settings and default config
│   └── installation_instructions.md # (optional) User setup guide
├── .zed/
│   └── *.rules                  # Only rules files here are versioned
└── README.md                    # This file
```

---

## Usage

### Build & Test (Rust)

```sh
# Clone and enter dir
git clone <your-repo-url>
cd deepwiki-mcp

# Build for dev/test
cargo build

# Run all tests (ensures protocol baseline)
cargo test
```

### Zed Development (Install as Extension)

1. Open Zed.  
2. Go to `Extensions` panel → "Install Dev Extension".
3. Select the `deepwiki-mcp` directory root.
4. Add the extension’s `[context_servers.deepwiki-mcp]` block to your Zed config (`settings.jsonc` or similar).
5. Optionally, update settings for endpoint, keys, etc. as per your tool/environment.

### Configuration Example (Zed)

```toml
[context_servers.deepwiki-mcp]
name = "DeepWiki MCP"
# Add custom config here if needed.
```

---

## How it Works

- The Zed extension launches the MCP server (Rust-compiled executable) using settings in `extension.toml`.
- All MCP tools and schemas are discoverable by the Zed assistant, via `/tools/list`.
- Tool invocation is streamed and results/errors returned to the Zed UI per MCP spec.

---

## Testing and Linting

- **Standard Rust CI:** Use `cargo check`, `cargo clippy`, `cargo test`.
- **No `pre-commit`**: This project does not use pre-commit hooks. All quality, linting, and validation should be enforced by running standard Cargo commands.
- **Reference servers:** For advanced/behavioral validation, compare your server with Exa, Firecrawl, and Neon MCP extensions/servers.

---

## Contribution

- Follow standard Rust and Zed extension practices. `.mcp.rs`, function scopes, naming, and lints are enforced for zero warnings.
- Submit issues or pull requests for enhancement, fixes, or questions.
- Please do **not** add `pre-commit` or related config unless discussed with project maintainers.

---

## References

- [Model Context Protocol (MCP) Spec](https://modelcontextprotocol.io/)
- [Zed Extension Guide](https://zed.dev/docs/extensions/mcp-extensions)
- Public servers: Exa MCP (`zed-exa-mcp-extension`), Firecrawl, Neon Postgres MCP extension.

---

## License

This project is licensed under the MIT License (c) 2025 Keshav Mishra.  
See [LICENSE](./LICENSE) for details.

---

*© 2025 DeepWiki MCP Extension Project*

---

### Git commit commands

```sh
git add src/mcp.rs src/lib.rs README.md
git commit -m "🧹 Pristine: warning-free, idiomatic Rust MCP code, snake_case + serde, minimal pub, best docs"
```