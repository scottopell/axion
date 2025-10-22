#!/bin/bash

# Build script for Axion Web Version

set -e

echo "Building Axion for Web (WASM)..."
echo ""

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed"
    echo "Install it with: cargo install wasm-pack"
    exit 1
fi

# Build the WASM package
echo "Step 1: Building WASM module..."
wasm-pack build --target web --out-dir www/pkg

echo ""
echo "Step 2: Build complete!"
echo ""
echo "To run the web version:"
echo "  cd www"
echo "  python3 -m http.server 8080"
echo ""
echo "Then open http://localhost:8080 in your browser"
echo ""
