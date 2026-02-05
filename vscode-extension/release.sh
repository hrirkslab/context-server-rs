#!/bin/bash
# Release script for Professional Context Engine VS Code Extension

echo "🚀 Starting release process for Professional Context Engine..."

# Clean previous builds
echo "🧹 Cleaning previous builds..."
npm run clean

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Run linting
echo "🔍 Running linter..."
npm run lint

# Compile TypeScript
echo "🔨 Compiling TypeScript..."
npm run compile

# Create VSIX package
echo "📦 Creating VSIX package..."
npm run package

# Check if VSIX was created
VSIX_FILE=$(ls *.vsix 2>/dev/null | head -1)
if [ -n "$VSIX_FILE" ]; then
    echo "✅ VSIX package created successfully: $VSIX_FILE"
    echo "📊 Package size: $(du -h "$VSIX_FILE" | cut -f1)"
    
    # Show installation instructions
    echo ""
    echo "📋 Installation Instructions:"
    echo "1. Install locally: code --install-extension $VSIX_FILE"
    echo "2. Or install via VS Code UI: Extensions > ... > Install from VSIX"
    echo "3. Or publish to marketplace: vsce publish"
    
    echo ""
    echo "🎉 Release process completed successfully!"
else
    echo "❌ Failed to create VSIX package"
    exit 1
fi