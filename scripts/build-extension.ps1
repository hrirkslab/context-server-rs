# Professional Context Engine - VS Code Extension Build Script (PowerShell)
# This script builds the VS Code extension from a clean state

Write-Host "🚀 Building Professional Context Engine VS Code Extension..." -ForegroundColor Blue

# Navigate to extension directory
Set-Location vscode-extension

# Clean previous builds
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
if (Test-Path "node_modules") { Remove-Item -Recurse -Force "node_modules" }
if (Test-Path "out") { Remove-Item -Recurse -Force "out" }
Get-ChildItem -Filter "*.vsix" | Remove-Item -Force
Get-ChildItem -Filter ".build-info*" | Remove-Item -Force

# Install dependencies
Write-Host "📦 Installing dependencies..." -ForegroundColor Yellow
npm install

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Failed to install dependencies" -ForegroundColor Red
    exit 1
}

# Compile TypeScript
Write-Host "🔨 Compiling TypeScript..." -ForegroundColor Yellow
npm run compile

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ TypeScript compilation failed" -ForegroundColor Red
    exit 1
}

# Run linting
Write-Host "🔍 Running linter..." -ForegroundColor Yellow
npm run lint
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️ Linting warnings found, continuing..." -ForegroundColor Yellow
}

# Create VSIX package
Write-Host "📦 Creating VSIX package..." -ForegroundColor Yellow
npx vsce package --no-dependencies

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Failed to create VSIX package" -ForegroundColor Red
    exit 1
}

# Find and display the created package
$vsixFile = Get-ChildItem -Filter "*.vsix" | Select-Object -First 1
if ($vsixFile) {
    $fileSize = [math]::Round($vsixFile.Length / 1KB, 2)
    Write-Host "✅ Extension package created: $($vsixFile.Name) ($fileSize KB)" -ForegroundColor Green
    Write-Host ""
    Write-Host "📋 Installation instructions:" -ForegroundColor Cyan
    Write-Host "   code --install-extension $($vsixFile.Name)" -ForegroundColor White
    Write-Host ""
    Write-Host "📚 See INSTALLATION.md for detailed setup instructions" -ForegroundColor Cyan
} else {
    Write-Host "❌ Failed to create VSIX package" -ForegroundColor Red
    exit 1
}

Write-Host "🎉 Build completed successfully!" -ForegroundColor Green