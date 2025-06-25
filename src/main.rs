mod mcp;

use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct MCPRequest {
    jsonrpc: String,
    id: serde_json::Value,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct MCPEnvelope {
    jsonrpc: String,
    id: serde_json::Value,
    result: Option<serde_json::Value>,
    error: Option<MCPErrorObject>,
}

#[derive(Serialize)]
struct MCPErrorObject {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}

/// Checks if a JSON value is MCP-compliant envelope.
fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let req: MCPRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let err_env = MCPEnvelope {
                    jsonrpc: "2.0".to_string(),
                    id: serde_json::json!(null),
                    result: None,
                    error: Some(MCPErrorObject {
                        code: -32700, // Parse error
                        message: format!("Invalid JSON-RPC request: {e}"),
                        data: None,
                    }),
                };
                let resp_str = serde_json::to_string(&err_env).unwrap();
                writeln!(stdout, "{resp_str}").unwrap();
                stdout.flush().unwrap();
                continue;
            }
        };

        // Dispatch to MCP tool handlers
        let (result, error) = match req.method.as_str() {
            "tools/list" => {
                // Call async handler for list_tools and flatten to sync via block_on for this CLI/proxy
                let list = futures::executor::block_on(crate::mcp::list_tools());
                let tools = serde_json::to_value(list).ok();
                (tools, None)
            }
            "tools/call" => {
                // tools/call expects a params object with "name" and "arguments"
                if let Some(params) = req.params {
                    let tool_name = params.get("name").and_then(|v| v.as_str());
                    let arguments = params
                        .get("arguments")
                        .cloned()
                        .unwrap_or(serde_json::json!({}));
                    if let Some(tool_name) = tool_name {
                        match futures::executor::block_on(crate::mcp::call_tool(
                            tool_name, arguments,
                        )) {
                            Ok(res) => (Some(res), None),
                            Err(e) => (
                                None,
                                Some(MCPErrorObject {
                                    code: 32002,
                                    message: format!(
                                        "Handler error for tool '{}': {:?}",
                                        tool_name, e
                                    ),
                                    data: None,
                                }),
                            ),
                        }
                    } else {
                        (
                            None,
                            Some(MCPErrorObject {
                                code: -32602,
                                message: "Missing or invalid 'name' param for tools/call"
                                    .to_string(),
                                data: None,
                            }),
                        )
                    }
                } else {
                    (
                        None,
                        Some(MCPErrorObject {
                            code: -32602,
                            message: "Missing params for tools/call".to_string(),
                            data: None,
                        }),
                    )
                }
            }
            _ => (
                None,
                Some(MCPErrorObject {
                    code: -32601,
                    message: format!("Method '{}' not found", req.method),
                    data: None,
                }),
            ),
        };

        let resp = MCPEnvelope {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result,
            error,
        };
        let resp_str = serde_json::to_string(&resp).unwrap();
        writeln!(stdout, "{resp_str}").unwrap();
        stdout.flush().unwrap();
    }
}
