use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[derive(Deserialize)]
struct MCPRequest {
    _jsonrpc: String,
    id: serde_json::Value,
    _method: String,
    _params: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct MCPResponse {
    jsonrpc: String,
    id: serde_json::Value,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let req: MCPRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Handle the method (for demo, we just echo back)
        let result =
            Some(serde_json::json!({"content": [{"text": "Hello from Rust MCP context server!"}]}));

        let resp = MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result,
            error: None,
        };

        let resp_str = serde_json::to_string(&resp).unwrap();
        writeln!(stdout, "{resp_str}").unwrap();
        stdout.flush().unwrap();
    }
}
