# Professional Context Engine VS Code Extension
## Complete Installation & Deployment Guide

### 🚀 Quick Start

1. **Build the Extension**
   ```bash
   cd vscode-extension
   npm run build:production
   ```

2. **Install in VS Code**
   ```bash
   npm run install:local
   ```

3. **Configure & Test**
   - Open VS Code Settings (`Ctrl+,`)
   - Search for "Context Engine"
   - Set server URL to your Context Engine server
   - Test connection: `Ctrl+Shift+P` → "Context Engine: Test Connection"

---

## 📦 Building the Extension

### Prerequisites
- Node.js 16.x or higher
- npm or yarn
- VS Code 1.74.0 or higher
- Professional Context Engine server

### Build Commands

```bash
# Install dependencies
npm install

# Development build
npm run compile

# Production build (recommended)
npm run build:production

# Watch mode for development
npm run watch

# Run tests
npm test

# Lint code
npm run lint
```

### Production Build Process

The `npm run build:production` command performs:
1. ✅ Cleans previous builds
2. ✅ Installs dependencies
3. ✅ Runs linter
4. ✅ Compiles TypeScript
5. ✅ Runs tests
6. ✅ Validates package.json
7. ✅ Creates VSIX package
8. ✅ Generates installation instructions

---

## 🔧 Installation Methods

### Method 1: VSIX File Installation (Recommended)

1. **Build or Download VSIX**
   ```bash
   npm run build:production
   # Creates: professional-context-engine-1.0.0.vsix
   ```

2. **Install via VS Code UI**
   - Open VS Code
   - Press `Ctrl+Shift+P` (Windows/Linux) or `Cmd+Shift+P` (Mac)
   - Type "Extensions: Install from VSIX..."
   - Select the `.vsix` file
   - Click "Install"
   - Reload VS Code when prompted

3. **Install via Command Line**
   ```bash
   code --install-extension professional-context-engine-1.0.0.vsix
   ```

### Method 2: Development Installation

```bash
# Clone and build
git clone <repository-url>
cd vscode-extension
npm install
npm run compile

# Install for development
npm run install:local
```

### Method 3: Manual Installation

1. **Locate Extensions Directory**
   - Windows: `%USERPROFILE%\.vscode\extensions`
   - macOS: `~/.vscode/extensions`
   - Linux: `~/.vscode/extensions`

2. **Extract Extension**
   ```bash
   # VSIX is a ZIP file
   unzip professional-context-engine-1.0.0.vsix -d ~/.vscode/extensions/
   ```

3. **Restart VS Code**

---

## ⚙️ Configuration

### Initial Setup

1. **Open Settings**
   - `File` → `Preferences` → `Settings`
   - Or press `Ctrl+,`

2. **Search for "Context Engine"**

3. **Configure Basic Settings**
   ```json
   {
     "contextEngine.serverUrl": "http://localhost:3000",
     "contextEngine.autoAnalyzeOnSave": true,
     "contextEngine.realTimeSuggestions": true,
     "contextEngine.enableHoverSuggestions": true,
     "contextEngine.enableCodeActions": true
   }
   ```

### Advanced Configuration

```json
{
  "contextEngine.supportedLanguages": [
    "rust", "typescript", "javascript", "python", "java", "cpp", "csharp"
  ],
  "contextEngine.debugMode": false,
  "contextEngine.connectionTimeout": 10000,
  "contextEngine.maxSuggestions": 10,
  "contextEngine.cacheTimeout": 30000,
  "contextEngine.analysisRules": [
    {
      "name": "TODO Extractor",
      "language": "rust",
      "pattern": "TODO:\\s*(.+)",
      "contextType": "general",
      "extractionMethod": "RegexCapture",
      "confidence": 0.8
    }
  ]
}
```

### Workspace Configuration

Create `.vscode/settings.json` in your project:
```json
{
  "contextEngine.serverUrl": "http://your-team-server:3000",
  "contextEngine.analysisRules": [
    // Project-specific rules
  ]
}
```

---

## 🧪 Testing & Verification

### Connection Test
```bash
# Open Command Palette (Ctrl+Shift+P)
# Run: Context Engine: Test Connection
```

### Feature Tests

1. **File Analysis**
   - Open a supported code file
   - Save the file
   - Check for analysis completion notification

2. **Context Suggestions**
   - Hover over code elements
   - Look for context suggestions in tooltip
   - Check Explorer panel for "Context Suggestions" tree

3. **Context Creation**
   - Select code text
   - Right-click → "Create Context Entry"
   - Fill in details and verify creation

4. **Project Insights**
   - Command Palette → "Context Engine: Show Project Insights"
   - Verify analytics dashboard opens

---

## 🚀 Deployment Strategies

### Individual Developer

```bash
# Build and install locally
npm run build:production
npm run install:local
```

### Team Distribution

1. **Build Extension**
   ```bash
   npm run build:production
   ```

2. **Share VSIX File**
   - Upload to team file share
   - Include installation instructions
   - Provide team configuration file

3. **Team Installation Script**
   ```bash
   #!/bin/bash
   # install-context-engine.sh
   
   VSIX_URL="https://your-server/professional-context-engine-1.0.0.vsix"
   VSIX_FILE="professional-context-engine-1.0.0.vsix"
   
   # Download extension
   curl -L -o "$VSIX_FILE" "$VSIX_URL"
   
   # Install extension
   code --install-extension "$VSIX_FILE"
   
   # Clean up
   rm "$VSIX_FILE"
   
   echo "Professional Context Engine installed successfully!"
   ```

### Enterprise Deployment

1. **Centralized Configuration**
   ```json
   // Global settings.json
   {
     "contextEngine.serverUrl": "https://context-engine.company.com",
     "contextEngine.analysisRules": [
       // Company-wide rules
     ]
   }
   ```

2. **Group Policy Deployment** (Windows)
   - Deploy VSIX via Group Policy
   - Configure default settings
   - Manage updates centrally

3. **Docker Container**
   ```dockerfile
   FROM mcr.microsoft.com/vscode/devcontainers/base:ubuntu
   
   # Install VS Code extensions
   COPY professional-context-engine-1.0.0.vsix /tmp/
   RUN code --install-extension /tmp/professional-context-engine-1.0.0.vsix
   
   # Configure extension
   COPY .vscode/settings.json /workspace/.vscode/
   ```

---

## 🔄 Updates & Maintenance

### Updating the Extension

1. **Build New Version**
   ```bash
   # Update version in package.json
   npm version patch  # or minor/major
   npm run build:production
   ```

2. **Install Update**
   ```bash
   # Uninstall old version
   npm run uninstall:local
   
   # Install new version
   npm run install:local
   ```

### Automated Updates

```bash
#!/bin/bash
# update-context-engine.sh

EXTENSION_ID="context-engine.professional-context-engine"
LATEST_VSIX="https://releases/latest.vsix"

# Check if extension is installed
if code --list-extensions | grep -q "$EXTENSION_ID"; then
    echo "Updating Context Engine extension..."
    code --uninstall-extension "$EXTENSION_ID"
    curl -L -o "latest.vsix" "$LATEST_VSIX"
    code --install-extension "latest.vsix"
    rm "latest.vsix"
    echo "Update completed!"
else
    echo "Extension not installed. Installing..."
    curl -L -o "latest.vsix" "$LATEST_VSIX"
    code --install-extension "latest.vsix"
    rm "latest.vsix"
    echo "Installation completed!"
fi
```

---

## 🐛 Troubleshooting

### Common Issues

1. **Extension Not Loading**
   ```bash
   # Check VS Code logs
   code --log debug
   
   # Verify extension is installed
   code --list-extensions | grep context-engine
   ```

2. **Connection Failures**
   - Verify Context Engine server is running
   - Check firewall settings
   - Test with `curl http://localhost:3000/health`

3. **TypeScript Compilation Errors**
   ```bash
   # Clean and rebuild
   npm run clean
   npm install
   npm run compile
   ```

4. **VSIX Creation Fails**
   ```bash
   # Install vsce globally
   npm install -g @vscode/vsce
   
   # Create package manually
   vsce package --no-dependencies
   ```

### Debug Mode

Enable debug logging:
```json
{
  "contextEngine.debugMode": true
}
```

Check logs:
- VS Code Output panel → "Professional Context Engine"
- Developer Console: `Help` → `Toggle Developer Tools`

### Support

- Check README.md for feature documentation
- Review CHANGELOG.md for version history
- Run demo scenarios from demo.md
- Check integration tests in test files

---

## 📈 Performance Optimization

### Large Projects
```json
{
  "contextEngine.cacheTimeout": 60000,
  "contextEngine.maxSuggestions": 5,
  "contextEngine.connectionTimeout": 15000
}
```

### Network Optimization
```json
{
  "contextEngine.realTimeSuggestions": false,
  "contextEngine.autoAnalyzeOnSave": false
}
```

### Memory Management
- Restart VS Code periodically for large projects
- Clear extension cache: Command Palette → "Developer: Reload Window"
- Monitor memory usage in Task Manager

---

## 🔒 Security Considerations

### Network Security
- Use HTTPS for production servers
- Configure firewall rules
- Implement authentication if needed

### Data Privacy
- Review what data is sent to the server
- Configure analysis rules carefully
- Use workspace-specific settings for sensitive projects

### Access Control
```json
{
  "contextEngine.serverUrl": "https://secure-server.company.com",
  "contextEngine.authToken": "${env:CONTEXT_ENGINE_TOKEN}"
}
```

---

This guide provides comprehensive instructions for building, installing, configuring, and deploying the Professional Context Engine VS Code extension in various environments.