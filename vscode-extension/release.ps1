#!/usr/bin/env pwsh
# Release script for Professional Context Engine VS Code Extension

Write-Host "🚀 Starting release process for Professional Context Engine..." -ForegroundColor Green

# Clean previous builds
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
npm run clean

# Install dependencies
Write-Host "📦 Installing dependencies..." -ForegroundColor Yellow
npm install

# Run linting
Write-Host "🔍 Running linter..." -ForegroundColor Yellow
npm run lint

# Compile TypeScript
Write-Host "🔨 Compiling TypeScript..." -ForegroundColor Yellow
npm run compile

# Create VSIX package
Write-Host "📦 Creating VSIX package..." -ForegroundColor Yellow
npm run package

# Check if VSIX was created
$vsixFile = Get-ChildItem -Name "*.vsix" | Select-Object -First 1
if ($vsixFile) {
    Write-Host "✅ VSIX package created successfully: $vsixFile" -ForegroundColor Green
    Write-Host "📊 Package size: $((Get-Item $vsixFile).Length / 1KB) KB" -ForegroundColor Cyan
    
    # Show installation instructions
    Write-Host "`n📋 Installation Instructions:" -ForegroundColor Cyan
    Write-Host "1. Install locally: code --install-extension $vsixFile" -ForegroundColor White
    Write-Host "2. Or install via VS Code UI: Extensions > ... > Install from VSIX" -ForegroundColor White
    Write-Host "3. Or publish to marketplace: vsce publish" -ForegroundColor White
    
    Write-Host "`n🎉 Release process completed successfully!" -ForegroundColor Green
} else {
    Write-Host "❌ Failed to create VSIX package" -ForegroundColor Red
    exit 1
}