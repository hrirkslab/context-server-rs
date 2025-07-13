# Manual MCP Server Testing Guide

## Option 1: Run Server in Separate PowerShell Window

1. Open a new PowerShell window
2. Navigate to the project directory:
   ```powershell
   cd "c:\Users\karki\source\repos\local-chat-llm\context-server-rs"
   ```

3. Run the server:
   ```powershell
   cargo run
   ```

4. The server will start and wait for JSON-RPC input on stdin

## Option 2: Use MCP Inspector (Recommended)

The MCP Inspector provides a web UI for testing MCP servers:
```powershell
npx @modelcontextprotocol/inspector "cargo run" --working-directory "c:\Users\karki\source\repos\local-chat-llm\context-server-rs"
```

Then open: http://localhost:6274/

## Option 3: Integration with Clients

### Claude Desktop
Add to your Claude Desktop config file (`%APPDATA%\Claude\claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "context-server-rs": {
      "command": "cargo",
      "args": ["run"],
      "cwd": "c:\\Users\\karki\\source\\repos\\local-chat-llm\\context-server-rs"
    }
  }
}
```

### VS Code with MCP Extension
Add to your VS Code settings.json:
```json
{
  "mcp": {
    "servers": {
      "context-server-rs": {
        "command": "cargo",
        "args": ["run"],
        "cwd": "c:\\Users\\karki\\source\\repos\\local-chat-llm\\context-server-rs"
      }
    }
  }
}
```

## Manual JSON-RPC Testing

If you want to test JSON-RPC messages manually, here are the key requests:

### Initialize Request:
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test-client","version":"1.0.0"}}}
```

### Initialized Notification (Required after initialize):
```json
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
```

### List Tools Request:
```json
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
```

### Create Project Tool Call:
```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"create_project","arguments":{"name":"test-project","description":"A test project"}}}
```

### List Projects Tool Call:
```json
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"list_projects","arguments":{}}}
```
