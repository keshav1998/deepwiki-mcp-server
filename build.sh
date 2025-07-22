#!/bin/bash

# DeepWiki MCP Server Build Script
# This script builds both the Zed extension (WASM) and the native bridge binary

set -e

echo "🚀 Building DeepWiki MCP Server Extension"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "extension.toml" ]; then
    print_error "extension.toml not found. Please run this script from the project root."
    exit 1
fi

# Clean previous builds
print_status "Cleaning previous builds..."
cargo clean

# Build the bridge binary first (native)
print_status "Building native bridge binary..."
if cargo build --release --manifest-path crates/bridge/Cargo.toml; then
    print_status "✅ Bridge binary built successfully"
else
    print_error "❌ Failed to build bridge binary"
    exit 1
fi

# Build the extension (WASM)
print_status "Building WASM extension..."
if cargo build --release --manifest-path crates/extension/Cargo.toml --target wasm32-wasip1; then
    print_status "✅ Extension WASM built successfully"
else
    print_error "❌ Failed to build extension WASM"
    exit 1
fi

# Create distribution directory
print_status "Creating distribution package..."
mkdir -p dist/bin

# Copy the bridge binary
cp target/release/deepwiki-mcp-bridge dist/bin/

# Copy the extension WASM
cp target/wasm32-wasip1/release/libdeepwiki_mcp_server_extension.wasm dist/

# Copy configuration files
cp extension.toml dist/
cp -r crates/extension/configuration dist/

# Copy documentation
cp README.md dist/ 2>/dev/null || echo "README.md not found, skipping"
cp LICENSE dist/ 2>/dev/null || echo "LICENSE not found, skipping"

print_status "📦 Distribution package created in 'dist/' directory"
echo
echo "Contents:"
echo "  - dist/bin/deepwiki-mcp-bridge (native binary)"
echo "  - dist/libdeepwiki_mcp_server_extension.wasm (Zed extension)"
echo "  - dist/extension.toml (extension manifest)"
echo "  - dist/configuration/ (extension configuration)"
echo

print_status "🎉 Build completed successfully!"
echo
echo "Next steps:"
echo "  1. Install the bridge binary: cp dist/bin/deepwiki-mcp-bridge ~/.local/bin/"
echo "  2. Install the extension in Zed extensions directory"
echo "  3. Or use: zed --install-extension ./dist"
