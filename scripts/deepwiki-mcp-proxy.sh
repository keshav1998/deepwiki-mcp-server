#!/bin/bash

# DeepWiki MCP Proxy Script
# This script acts as a simple proxy between Zed and the DeepWiki MCP server
# It forwards JSON-RPC requests from stdin to the DeepWiki server and returns responses

# Enable debug logging
DEBUG=${DEBUG:-0}
debug_log() {
    if [ "$DEBUG" = "1" ]; then
        echo "DEBUG: $1" >&2
    fi
}

# Get configuration from environment variables
DEEPWIKI_ENDPOINT="${DEEPWIKI_ENDPOINT:-https://mcp.deepwiki.com}"
DEEPWIKI_PROTOCOL="${DEEPWIKI_PROTOCOL:-mcp}"
DEEPWIKI_URL="${DEEPWIKI_ENDPOINT}/${DEEPWIKI_PROTOCOL}"

# Session management - use a temporary file to persist state
SESSION_FILE=$(mktemp)
echo "" > "$SESSION_FILE"
INITIALIZED=false

# Function to get session ID
get_session_id() {
    cat "$SESSION_FILE" 2>/dev/null || echo ""
}

# Function to set session ID
set_session_id() {
    echo "$1" > "$SESSION_FILE"
    debug_log "Session ID set to: $1"
}

# Function to make HTTP request to DeepWiki
make_request() {
    local json_data="$1"
    local current_session_id=$(get_session_id)

    debug_log "Making request to $DEEPWIKI_URL"
    debug_log "Request data: $json_data"
    debug_log "Current session ID: '$current_session_id'"

    local headers=(-H "Content-Type: application/json" -H "Accept: application/json, text/event-stream")

    # Add session ID header if we have one
    if [ -n "$current_session_id" ]; then
        headers+=(-H "Mcp-Session-Id: $current_session_id")
        debug_log "Using session ID header"
    else
        debug_log "No session ID header"
    fi

    # Make the request and capture both headers and body
    local temp_headers=$(mktemp)
    local response=$(curl -s -D "$temp_headers" "${headers[@]}" -d "$json_data" "$DEEPWIKI_URL")

    debug_log "Raw response: $response"

    # Extract session ID from headers if present
    local new_session_id=$(grep -i "mcp-session-id:" "$temp_headers" | cut -d' ' -f2- | tr -d '\r\n')
    if [ -n "$new_session_id" ]; then
        set_session_id "$new_session_id"
        debug_log "Updated session ID: $new_session_id"
    fi

    rm "$temp_headers"

    # Parse SSE response to extract JSON
    local json_response=$(echo "$response" | grep "^data: " | sed 's/^data: //' | grep -v "^ping$" | head -1)
    debug_log "Parsed JSON: $json_response"
    echo "$json_response"
}

# Function to handle initialization
handle_initialize() {
    local request_id="$1"
    debug_log "Handling initialize with request ID: $request_id"

    local init_request='{
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "deepwiki-mcp-server-zed", "version": "0.1.0"}
        }
    }'

    local response=$(make_request "$init_request")
    if [ $? -eq 0 ] && [ -n "$response" ] && [ "$response" != "null" ]; then
        INITIALIZED=true
        debug_log "Initialization successful"
        echo "$response" | jq -c --arg id "$request_id" '.id = ($id | tonumber)'
    else
        debug_log "Initialization failed"
        echo "{\"jsonrpc\":\"2.0\",\"id\":$request_id,\"error\":{\"code\":-32603,\"message\":\"Failed to initialize DeepWiki session\"}}"
    fi
}

# Function to forward requests to DeepWiki
forward_request() {
    local method="$1"
    local params="$2"
    local request_id="$3"
    local session_id=$(get_session_id)

    debug_log "Forwarding request: method=$method, params=$params, id=$request_id"
    debug_log "Current session ID: '$session_id'"
    debug_log "Initialized flag: '$INITIALIZED'"

    if [ "$INITIALIZED" != "true" ]; then
        debug_log "Session not initialized"
        echo "{\"jsonrpc\":\"2.0\",\"id\":$request_id,\"error\":{\"code\":-32603,\"message\":\"Session not initialized\"}}"
        return
    fi

    if [ -z "$session_id" ]; then
        debug_log "No session ID available"
        echo "{\"jsonrpc\":\"2.0\",\"id\":$request_id,\"error\":{\"code\":-32603,\"message\":\"No session ID available\"}}"
        return
    fi

    local forward_request
    if [ "$params" = "null" ] || [ -z "$params" ]; then
        forward_request="{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"$method\"}"
    else
        forward_request="{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"$method\",\"params\":$params}"
    fi

    debug_log "Forward request JSON: $forward_request"

    local response=$(make_request "$forward_request")
    debug_log "Forward response: $response"

    if [ $? -eq 0 ] && [ -n "$response" ] && [ "$response" != "null" ]; then
        echo "$response" | jq -c --arg id "$request_id" '.id = ($id | tonumber)'
    else
        debug_log "Failed to get valid response"
        echo "{\"jsonrpc\":\"2.0\",\"id\":$request_id,\"error\":{\"code\":-32603,\"message\":\"Failed to forward request to DeepWiki\"}}"
    fi
}

# Cleanup function
cleanup() {
    debug_log "Cleaning up temporary files"
    rm -f "$SESSION_FILE"
}

# Set up cleanup trap
trap cleanup EXIT

# Main loop - read JSON-RPC requests from stdin
while IFS= read -r line; do
    if [ -z "$line" ]; then
        continue
    fi

    debug_log "Received request: $line"

    # Parse the JSON-RPC request
    method=$(echo "$line" | jq -r '.method // empty')
    params=$(echo "$line" | jq -c '.params // null')
    request_id=$(echo "$line" | jq -r '.id // null')

    debug_log "Parsed: method=$method, params=$params, id=$request_id"

    case "$method" in
        "initialize")
            handle_initialize "$request_id"
            ;;
        "initialized")
            # Notification - no response needed
            debug_log "Received initialized notification"
            ;;
        "notifications/message")
            # Handle logging/debugging messages - no response needed
            debug_log "Received notification message"
            ;;
        "tools/list"|"tools/call")
            forward_request "$method" "$params" "$request_id"
            ;;
        *)
            if [ "$request_id" != "null" ]; then
                debug_log "Unknown method: $method"
                echo "{\"jsonrpc\":\"2.0\",\"id\":$request_id,\"error\":{\"code\":-32601,\"message\":\"Method '$method' not found\"}}"
            fi
            ;;
    esac
done
