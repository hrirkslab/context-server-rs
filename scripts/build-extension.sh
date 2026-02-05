#!/bin/bash

# Professional Context Engine - VS Code Extension Build Script
# This script builds the VS Code extension from a clean state

set -e

echo "🚀 Building Professional Context Engine VS Code Extension..."

# Navigate to extension directory
cd vscode-extension

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf node_modules out *.vsix .build-info*

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Compile TypeScript
echo "🔨 Compiling TypeScript..."
npm run compile

# Run linting
echo "🔍 Running linter..."
npm run lint || echo "⚠️ Linting warnings found, continuing..."

# Create VSIX package
echo "📦 Creating VSIX package..."
npx vsce package --no-dependencies

# Find and display the created package
VSIX_FILE=$(ls *.vsix | head -1)
if [ -n "$VSIX_FILE" ]; then
    FILE_SIZE=$(du -h "$VSIX_FILE" | cut -f1)
    echo "✅ Extension package created: $VSIX_FILE ($FILE_SIZE)"
    echo ""
    echo "📋 Installation instructions:"
    echo "   code --install-extension $VSIX_FILE"
    echo ""
    echo "📚 See INSTALLATION.md for detailed setup instructions"
else
    echo "❌ Failed to create VSIX package"
    exit 1
fi

echo "🎉 Build completed successfully!"