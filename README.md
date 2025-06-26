# DeepWiki MCP Zed Extension

![CI status](https://github.com/yourusername/deepwiki-mcp/actions/workflows/ci.yml/badge.svg)

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

- Follow standard Rust and Zed extension practices. All protocol, field, function, and module naming uses idiomatic Rust and Serde with strict clippy/lint checks (no warnings).
- Before PR:
  - Run `cargo check`, `cargo clippy -- -D warnings`, and `cargo test --all-features --all-targets`
  - Run E2E protocol tests: `cargo test --test protocol` (see below)
- Submit issues or pull requests for enhancement, fixes, or questions.
- Please do **not** add `pre-commit` or related config unless discussed with project maintainers.

---

## Reporting Issues & Using the Issue Template

If you discover a bug or protocol edge case, please:

1. Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md) provided in this repository.
2. Include a clear, minimal reproduction: steps to run your command, sample MCP envelope or error, and environment details.
3. Double-check your Rust toolchain and MCP backend version.
4. Indicate whether you are running latest `main` or `develop` branch.
5. Check for similar issues before filing a new one.

Your contribution helps keep this extension robust for the whole open source community!

---

## Setup & Testing

To contribute or validate a build:

```sh
git clone <repo-url>
cd deepwiki-mcp

# Install Rust stable if needed
rustup default stable

# Build and check warnings
cargo check --all-targets --all-features
cargo clippy -- -D warnings

# Run all unit and integration tests (including E2E protocol tests)
cargo test --all-features --all-targets
cargo test --test protocol
```

### End-to-End Protocol Testing

All MCP boundaries and envelope/protocol handling are tested in `tests/protocol.rs` using assert_cmd.
- You can also run the binary directly and pipe MCP-compliant JSON-RPC to it and check output using tools like `jq`.

---

## Continuous Integration

All PRs and pushes are tested automatically via GitHub Actions:
- Build, lint, unit/integration/E2E protocol test suite
- CI badge reflects the status of the last commit and all tests/warnings

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