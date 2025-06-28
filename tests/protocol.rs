use assert_cmd::Command;
use predicates::prelude::*;

fn binary_path() -> &'static str {
    // Update the binary name if you change Cargo.toml [package] > name
    "deepwiki-mcp-server"
}

/// Test suite for MCP protocol boundary: tools/list
#[test]
fn test_tools_list_success() {
    let input = r#"{"jsonrpc":"2.0", "id":1, "method":"tools/list", "params":null}"#;
    let mut cmd = Command::cargo_bin(binary_path()).expect("binary should build");
    cmd.write_stdin(input);

    let assert = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""result":["#));
    let output = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(
        output.contains(r#""read_wiki_structure""#),
        "Should list all tools: {output}"
    );
    assert!(output.contains(r#""ask_question""#));
    assert!(output.contains(r#""jsonrpc":"2.0""#));
    assert!(output.contains(r#""id":1"#));
}

/// test unknown method is handled as error
#[test]
fn test_unknown_method_error() {
    let input = r#"{"jsonrpc":"2.0", "id":99, "method":"tools/unknown"}"#;
    let mut cmd = Command::cargo_bin(binary_path()).expect("binary should build");
    cmd.write_stdin(input);

    let assert = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""error":{"code":-32601"#));
    let output = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(output.contains(r#""jsonrpc":"2.0""#));
    assert!(output.contains(r#""id":99"#));
}

/// test a valid tools/call for read_wiki_structure (disabled - crashes outside Zed environment)
#[ignore]
#[test]
fn test_call_read_wiki_structure_success() {
    let input = r#"{"jsonrpc":"2.0", "id":5, "method":"tools/call", "params":{"name":"read_wiki_structure","arguments":{"repoName":"foo/bar"}}}"#;
    let mut cmd = Command::cargo_bin(binary_path()).expect("binary should build");
    cmd.write_stdin(input);

    // The HTTP client will fail outside Zed environment, so we expect an error response
    let assert = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""error":{"code":-32602"#));
    let output = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(output.contains(r#""jsonrpc":"2.0""#));
    assert!(output.contains(r#""id":5"#));
}

/// test call with missing required arguments
#[test]
fn test_call_read_wiki_structure_missing_args() {
    let input = r#"{"jsonrpc":"2.0", "id":6, "method":"tools/call", "params":{"name":"read_wiki_structure","arguments":{}}}"#;
    let mut cmd = Command::cargo_bin(binary_path()).expect("binary should build");
    cmd.write_stdin(input);

    let assert = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""error":{"code":-32602"#));
    let output = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(output.contains(r#""id":6"#));
}

/// test call with invalid argument type
#[test]
fn test_call_invalid_argument_type() {
    let input = r#"{"jsonrpc":"2.0", "id":7, "method":"tools/call", "params":{"name":"ask_question","arguments":{"repoName":123,"question":42}}}"#;
    let mut cmd = Command::cargo_bin(binary_path()).expect("binary should build");
    cmd.write_stdin(input);

    let assert = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""error":{"code":-32602"#));
    let output = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(output.contains(r#""id":7"#));
}

/// test tools/call with unknown tool
#[test]
fn test_call_unknown_tool() {
    let input = r#"{"jsonrpc":"2.0", "id":8, "method":"tools/call", "params":{"name":"not_real","arguments":{}}}"#;
    let mut cmd = Command::cargo_bin(binary_path()).expect("binary should build");
    cmd.write_stdin(input);

    let assert = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""error":{"code":-32001"#));
    let output = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(output.contains(r#""id":8"#));
}

/// test tools/list returns error if test-mode env is set
#[test]
fn test_tools_list_empty_triggers_error() {
    let input = r#"{"jsonrpc":"2.0", "id":42, "method":"tools/list", "params":null}"#;
    let mut cmd = Command::cargo_bin(binary_path()).expect("binary should build");
    cmd.env("MCP_TEST_EMPTY_TOOLS", "1");
    cmd.write_stdin(input);

    let assert = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""error":{"code":32001"#));
    let output = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(output.to_lowercase().contains("no tools available"));
    assert!(output.contains(r#""id":42"#));
}

/// test malformed input (not JSON)
#[test]
fn test_protocol_parse_error() {
    let input = r#"{ this is not valid json }"#;
    let mut cmd = Command::cargo_bin(binary_path()).expect("binary should build");
    cmd.write_stdin(input);

    let assert = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""error":{"code":-32700"#));
    let _output = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
}
