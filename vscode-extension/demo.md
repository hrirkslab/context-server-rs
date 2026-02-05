# Professional Context Engine VS Code Extension Demo

This document provides a comprehensive demo of the VS Code extension features and functionality.

## Prerequisites

1. Professional Context Engine server running on `http://localhost:3000`
2. VS Code with the extension installed and enabled
3. A workspace with supported code files (Rust, TypeScript, JavaScript, Python, etc.)

## Demo Scenarios

### 1. Basic Setup and Connection

1. **Install and Activate Extension**
   - Install the extension from VSIX or marketplace
   - Extension should activate automatically when VS Code starts
   - Check status bar for Context Engine indicators

2. **Test Connection**
   - Open Command Palette (`Ctrl+Shift+P`)
   - Run `Context Engine: Test Connection`
   - Should show success message if server is running

3. **Configure Settings**
   - Open Settings (`Ctrl+,`)
   - Search for "Context Engine"
   - Verify default settings:
     - Server URL: `http://localhost:3000`
     - Auto-analyze on save: `true`
     - Real-time suggestions: `true`
     - Hover suggestions: `true`
     - Code actions: `true`

### 2. File Analysis and Suggestions

1. **Open a Code File**
   - Open a Rust, TypeScript, or other supported file
   - Extension should automatically analyze the file
   - Check Output panel for analysis logs

2. **View Context Suggestions**
   - Open Explorer panel
   - Look for "Context Suggestions" tree view
   - Should show suggestions grouped by priority
   - Click on suggestions to see details

3. **Hover Suggestions**
   - Hover over code elements
   - Should see context suggestions in hover tooltip
   - Suggestions should be relevant to the hovered code

4. **Code Actions**
   - Right-click on code
   - Look for Context Engine actions in context menu
   - Try "Create Context Entry" and "Analyze Current File"

### 3. Context Creation

1. **Create Context from Selection**
   - Select some code
   - Right-click and choose "Create Context Entry"
   - Fill in the context details:
     - Title: "Authentication Logic"
     - Type: "business_rule"
     - Description: "User authentication business rules"
   - Submit and verify success message

2. **Manual Context Creation**
   - Run `Context Engine: Create Context Entry` from Command Palette
   - Create context without code selection
   - Verify context is stored and available

### 4. Analysis Rules Configuration

1. **View Existing Rules**
   - Run `Context Engine: Configure Analysis Rules`
   - Choose "View Existing Rules"
   - Should show default rules if any

2. **Add Custom Rule**
   - Run `Context Engine: Configure Analysis Rules`
   - Choose "Add New Rule"
   - Create a rule:
     - Name: "TODO Extractor"
     - Language: "rust"
     - Pattern: `TODO:\s*(.+)`
     - Context Type: "general"
     - Extraction Method: "RegexCapture"
     - Confidence: 0.8
   - Verify rule is saved

3. **Test Custom Rule**
   - Add a TODO comment to a Rust file
   - Save the file
   - Check if suggestion is generated based on the rule

### 5. Project Insights and Analytics

1. **Context Health Report**
   - Run `Context Engine: Show Context Health`
   - Should open webview with health metrics
   - Check overall health score and recommendations

2. **Project Insights**
   - Run `Context Engine: Show Project Insights`
   - Should show usage statistics and trends
   - Verify metrics are displayed correctly

3. **Context Query**
   - Run `Context Engine: Query Context`
   - Enter search query: "authentication"
   - Should show relevant context results

### 6. Real-time Features

1. **File Watching**
   - Make changes to a monitored file
   - Save the file
   - Verify automatic analysis occurs
   - Check for updated suggestions

2. **WebSocket Connection**
   - Check browser developer tools or extension logs
   - Should see WebSocket connection established
   - Make changes and verify real-time updates

### 7. Configuration Management

1. **Export Configuration**
   - Run `Context Engine: Export Configuration`
   - Choose save location
   - Verify JSON file is created with current settings

2. **Import Configuration**
   - Modify the exported JSON file
   - Run `Context Engine: Import Configuration`
   - Select the modified file
   - Verify settings are updated

### 8. Advanced Features

1. **Suggestion Triggers**
   - Run `Context Engine: Configure Suggestion Triggers`
   - Add a custom trigger:
     - Name: "Large Function Warning"
     - Type: "LineCount"
     - Condition: "lineCount > 50"
     - Template: "Consider breaking this function into smaller parts"

2. **Toggle Features**
   - Run `Context Engine: Toggle Real-time Suggestions`
   - Verify feature is disabled/enabled
   - Run `Context Engine: Toggle Auto-analyze`
   - Test behavior changes

## Expected Behaviors

### Successful Scenarios

- ✅ Extension activates without errors
- ✅ Connection to Context Engine server succeeds
- ✅ File analysis generates relevant suggestions
- ✅ Hover shows context information
- ✅ Code actions are available and functional
- ✅ Context creation works from selection and manually
- ✅ Analysis rules can be configured and applied
- ✅ Project insights display meaningful data
- ✅ Real-time updates work via WebSocket
- ✅ Configuration export/import functions correctly

### Error Handling

- ⚠️ Server connection failure shows appropriate error message
- ⚠️ Invalid analysis rules are rejected with validation errors
- ⚠️ WebSocket disconnection triggers reconnection attempts
- ⚠️ Large file analysis shows progress indication
- ⚠️ Network errors are handled gracefully

## Troubleshooting

### Common Issues

1. **Extension Not Activating**
   - Check VS Code version (requires 1.74.0+)
   - Look for errors in Developer Console (`Help > Toggle Developer Tools`)
   - Verify extension is enabled in Extensions panel

2. **Connection Failures**
   - Verify Context Engine server is running
   - Check server URL in settings
   - Test with `curl http://localhost:3000/health`

3. **No Suggestions Appearing**
   - Verify file type is supported
   - Check if auto-analyze is enabled
   - Manually run "Analyze Current File" command

4. **WebSocket Issues**
   - Check browser network tab for WebSocket connection
   - Verify server supports WebSocket connections
   - Look for reconnection attempts in logs

### Debug Mode

1. Enable debug mode in settings: `contextEngine.debugMode: true`
2. Check Output panel for detailed logs
3. Use Developer Console for JavaScript errors
4. Monitor network requests in browser dev tools

## Performance Testing

### Large File Handling
- Open files with 1000+ lines
- Verify analysis completes within reasonable time
- Check memory usage doesn't spike excessively

### Multiple File Monitoring
- Open multiple files simultaneously
- Make changes to several files
- Verify all files are analyzed correctly

### WebSocket Stress Test
- Keep extension running for extended periods
- Monitor WebSocket connection stability
- Test reconnection after network interruptions

## Integration Testing

### Team Collaboration
- Multiple developers using same Context Engine server
- Verify real-time updates across team members
- Test configuration sharing

### CI/CD Integration
- Extension behavior in automated environments
- Configuration validation in build pipelines
- Error reporting and logging

This demo script provides comprehensive coverage of the VS Code extension functionality and helps verify that Task 6.6 has been successfully implemented.