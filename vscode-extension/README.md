# Professional Context Engine - VS Code Extension

This VS Code extension provides intelligent context management and AI-powered development assistance by integrating with the Professional Context Engine server.

## Features

### 🧠 Intelligent Context Suggestions
- Real-time context suggestions based on your code
- Hover over code to see relevant context information
- Code actions for quick context management

### 📁 File Monitoring
- Automatic analysis of code changes
- Real-time file watching for supported languages
- Configurable analysis triggers

### ⚙️ Customizable Analysis Rules
- Define custom patterns for context extraction
- Language-specific analysis rules
- Configurable confidence levels

### 📊 Project Insights
- Context health monitoring
- Usage analytics and trends
- Development velocity metrics

### 🔧 Configuration Management
- Export/import extension settings
- Team-wide configuration sharing
- Workspace-specific settings

## Installation

1. Install the extension from the VS Code marketplace
2. Configure the Context Engine server URL in settings
3. Start using intelligent context suggestions!

## Configuration

### Basic Settings

- `contextEngine.serverUrl`: URL of the Context Engine server (default: `http://localhost:3000`)
- `contextEngine.autoAnalyzeOnSave`: Automatically analyze files when saved (default: `true`)
- `contextEngine.realTimeSuggestions`: Show real-time context suggestions (default: `true`)
- `contextEngine.enableHoverSuggestions`: Show context suggestions on hover (default: `true`)
- `contextEngine.enableCodeActions`: Enable context-aware code actions (default: `true`)

### Advanced Settings

- `contextEngine.supportedLanguages`: List of supported programming languages
- `contextEngine.analysisRules`: Custom analysis rules for context extraction
- `contextEngine.suggestionTriggers`: Custom triggers for context suggestions

## Commands

### Context Management
- `Context Engine: Show Context Suggestions` - Display available suggestions for current file
- `Context Engine: Refresh Context Analysis` - Re-analyze current file
- `Context Engine: Create Context Entry` - Create new context from selected code
- `Context Engine: Analyze Current File` - Run full analysis on current file

### Configuration
- `Context Engine: Open Context Engine Settings` - Open extension settings
- `Context Engine: Configure Analysis Rules` - Manage custom analysis rules
- `Context Engine: Configure Suggestion Triggers` - Manage suggestion triggers
- `Context Engine: Export Configuration` - Export settings to file
- `Context Engine: Import Configuration` - Import settings from file

### Analytics & Insights
- `Context Engine: Show Context Health` - Display context health report
- `Context Engine: Show Project Insights` - Show project analytics
- `Context Engine: Query Context` - Search context database

### Utilities
- `Context Engine: Test Connection` - Test connection to Context Engine server
- `Context Engine: Toggle Real-time Suggestions` - Enable/disable real-time suggestions
- `Context Engine: Toggle Auto-analyze` - Enable/disable auto-analyze on save

## Usage

### Getting Context Suggestions

1. Open a supported code file
2. The extension will automatically analyze the file
3. Hover over code to see context suggestions
4. Use Ctrl+Shift+P and search for "Context Engine" commands

### Creating Context Entries

1. Select code you want to create context for
2. Right-click and choose "Create Context Entry"
3. Fill in the context details
4. The context will be stored and available for future suggestions

### Configuring Analysis Rules

1. Open Command Palette (Ctrl+Shift+P)
2. Run "Context Engine: Configure Analysis Rules"
3. Add custom rules for your project needs
4. Rules will be applied during file analysis

## Supported Languages

- Rust
- TypeScript
- JavaScript
- Python
- Java
- C++
- C#

Additional languages can be configured in settings.

## Requirements

- VS Code 1.74.0 or higher
- Professional Context Engine server running
- Network access to the Context Engine server

## Extension Settings

This extension contributes the following settings:

* `contextEngine.serverUrl`: Configure the Context Engine server URL
* `contextEngine.autoAnalyzeOnSave`: Enable/disable automatic analysis on file save
* `contextEngine.realTimeSuggestions`: Enable/disable real-time suggestions
* `contextEngine.enableHoverSuggestions`: Enable/disable hover suggestions
* `contextEngine.enableCodeActions`: Enable/disable code actions
* `contextEngine.supportedLanguages`: List of supported programming languages
* `contextEngine.analysisRules`: Custom analysis rules
* `contextEngine.suggestionTriggers`: Custom suggestion triggers

## Known Issues

- WebSocket connection may occasionally disconnect and require reconnection
- Large files may take longer to analyze
- Some advanced features require Context Engine server v2.0+

## Release Notes

### 1.0.0

Initial release of Professional Context Engine VS Code extension.

Features:
- Real-time context suggestions
- File monitoring and analysis
- Customizable analysis rules
- Project insights and analytics
- Configuration management

## Contributing

This extension is part of the Professional Context Engine project. Please refer to the main project repository for contribution guidelines.

## License

This extension is licensed under the same license as the Professional Context Engine project.