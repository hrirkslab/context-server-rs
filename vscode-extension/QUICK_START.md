# Professional Context Engine VS Code Extension
## Quick Start Guide

### 🚀 5-Minute Setup

#### Step 1: Install the Extension
```bash
# Download and install the VSIX file
code --install-extension professional-context-engine-1.0.0.vsix
```

#### Step 2: Configure Server Connection
1. Open VS Code Settings (`Ctrl+,`)
2. Search for "Context Engine"
3. Set **Server URL** to your Context Engine server (e.g., `http://localhost:3000`)

#### Step 3: Test Connection
1. Open Command Palette (`Ctrl+Shift+P`)
2. Type "Context Engine: Test Connection"
3. Verify you see "Successfully connected to Context Engine"

#### Step 4: Start Using
1. Open any supported code file (Rust, TypeScript, JavaScript, Python, etc.)
2. Save the file to trigger automatic analysis
3. Hover over code to see context suggestions
4. Check the "Context Suggestions" panel in Explorer

### 🎯 Key Features

#### Real-time Context Suggestions
- **Hover Suggestions**: Hover over code to see relevant context
- **Code Actions**: Right-click for context management options
- **Tree View**: Browse suggestions in Explorer panel

#### File Analysis
- **Auto-analyze**: Files are analyzed when saved
- **Manual Analysis**: Use "Analyze Current File" command
- **Multi-language**: Supports Rust, TypeScript, JavaScript, Python, Java, C++, C#

#### Context Creation
- **From Selection**: Select code → Right-click → "Create Context Entry"
- **Manual Entry**: Use "Create Context Entry" command
- **Rich Metadata**: Add titles, types, descriptions, and tags

#### Project Insights
- **Health Reports**: View context quality and coverage
- **Usage Analytics**: See how context is being used
- **Query Interface**: Search your project's context database

### 📋 Common Commands

| Command | Shortcut | Description |
|---------|----------|-------------|
| Show Context Suggestions | `Ctrl+Shift+P` → "Context Engine: Show Context Suggestions" | Display suggestions for current file |
| Create Context Entry | Right-click → "Create Context Entry" | Create context from selected code |
| Analyze Current File | `Ctrl+Shift+P` → "Context Engine: Analyze Current File" | Run analysis on current file |
| Test Connection | `Ctrl+Shift+P` → "Context Engine: Test Connection" | Verify server connection |
| Show Project Insights | `Ctrl+Shift+P` → "Context Engine: Show Project Insights" | View project analytics |
| Configure Settings | `Ctrl+,` → Search "Context Engine" | Adjust extension settings |

### ⚙️ Essential Settings

```json
{
  "contextEngine.serverUrl": "http://localhost:3000",
  "contextEngine.autoAnalyzeOnSave": true,
  "contextEngine.realTimeSuggestions": true,
  "contextEngine.enableHoverSuggestions": true,
  "contextEngine.enableCodeActions": true
}
```

### 🔧 Troubleshooting

#### Connection Issues
```bash
# Test server manually
curl http://localhost:3000/health

# Check VS Code Output panel
# View → Output → Select "Professional Context Engine"
```

#### No Suggestions Appearing
1. Verify file type is supported
2. Check if auto-analyze is enabled
3. Manually run "Analyze Current File"
4. Check server connection

#### Extension Not Loading
1. Verify VS Code version (1.74.0+ required)
2. Check Extensions panel for errors
3. Restart VS Code
4. Check Developer Console (`Help` → `Toggle Developer Tools`)

### 📚 Next Steps

1. **Explore Advanced Features**
   - Configure custom analysis rules
   - Set up suggestion triggers
   - Use project insights dashboard

2. **Team Setup**
   - Share configuration files
   - Set up team server
   - Configure workspace settings

3. **Customize for Your Project**
   - Add project-specific analysis rules
   - Configure context types
   - Set up automated workflows

### 💡 Tips & Tricks

- **Keyboard Shortcuts**: Create custom shortcuts for frequently used commands
- **Workspace Settings**: Use `.vscode/settings.json` for project-specific configuration
- **Context Types**: Use appropriate context types for better organization
- **Regular Analysis**: Enable auto-analyze to keep context up-to-date
- **Team Collaboration**: Share context insights with team members

### 📞 Need Help?

- **Documentation**: See README.md for detailed information
- **Installation**: Check INSTALLATION.md for setup instructions
- **Issues**: Report problems via GitHub Issues
- **Configuration**: See configuration examples in documentation

---

**Happy coding with intelligent context assistance! 🎉**