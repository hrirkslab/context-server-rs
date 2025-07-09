# MCP Context Server for AI Code Generation

This guide explains how to use the Rust-based Model Context Protocol (MCP) server as a context provider for AI agents (such as Claude Desktop and Cursor IDE) and for integration with developer tools.

## What is Model Context Protocol (MCP)?

The [Model Context Protocol (MCP)](https://modelcontextprotocol.io/introduction) is an open protocol that standardizes how applications provide context to LLMs. MCP follows a client-server architecture where:

- **MCP Hosts**: Programs like Claude Desktop, IDEs, or AI tools that want to access data through MCP
- **MCP Clients**: Protocol clients that maintain 1:1 connections with servers  
- **MCP Servers**: Lightweight programs that expose specific capabilities through the standardized Model Context Protocol
- **Local Data Sources**: Your computer's files, databases, and services that MCP servers can securely access

## This MCP Server vs. Standard HTTP APIs

Our MCP Context Server:
- ✅ Uses the official [Model Context Protocol specification](https://spec.modelcontextprotocol.io/specification/2024-11-05/)
- ✅ Implements MCP tools for structured data exchange
- ✅ Supports automatic tool discovery by MCP clients
- ✅ Uses standard MCP transports (stdio, SSE, HTTP streaming)
- ✅ Built with the official [Rust MCP SDK (rmcp)](https://github.com/modelcontextprotocol/rust-sdk)

Benefits over HTTP APIs:
- **Standardized communication** with LLM applications
- **Tool discovery** - clients automatically discover available tools
- **Type safety** - JSON schemas define tool parameters
- **Better integration** with Claude Desktop, Cursor, and other MCP clients

## 1. Run the MCP Context Server

1. Build and start the MCP Context Server:
   ```sh
   cargo run --release
   ```
   The server will start using stdio transport (standard input/output) for MCP communication.

2. The server is now ready to accept MCP client connections.

## 2. Connect MCP Clients

### Claude Desktop Integration

1. Add this server to your Claude Desktop configuration file:

**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

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

### VS Code with MCP Extension Integration

Add this to your VS Code settings.json for MCP extension support:

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

### Cursor IDE Integration

Configure similar to VS Code with MCP support.

### MCP Inspector (for testing)

Use the [MCP Inspector](https://github.com/modelcontextprotocol/inspector) to test your server:

```bash
npx @modelcontextprotocol/inspector
```

## 3. Available MCP Tools

Once connected, clients can discover and use these MCP tools:

### `query_context`
Query project context based on feature area, task type, and components.

**Parameters:**
```json
{
  "project_id": "your-project-id",
  "feature_area": "authentication", 
  "task_type": "implement",
  "components": ["login", "signup"]
}
```

**Returns:** Business rules, architectural decisions, performance requirements, security policies, and conventions relevant to your query.

### `list_projects`
List all available projects in the context database.

**Parameters:** None

**Returns:** Array of all projects with their details.

### `create_project`
Create a new project in the context database.

**Parameters:**
```json
{
  "name": "My Project",
  "description": "Optional description",
  "repository_url": "https://github.com/user/repo"
}
```

**Returns:** The created project details.

## 4. Using with Claude Desktop

Once configured, you can ask Claude to:

- "Query the context for authentication implementation in my Flutter project"
- "What are the performance requirements for list rendering in my app?"
- "Show me the architectural decisions for my project"
- "Create a new project called 'Mobile Banking App'"

Claude will automatically call the appropriate MCP tools and provide context-aware responses.

## 5. Example MCP Interaction

Here's how an MCP client like Claude Desktop interacts with our server:

1. **Tool Discovery**: Client discovers available tools (`query_context`, `list_projects`, `create_project`)

2. **Context Query**: Client calls `query_context` tool:
   ```json
   {
     "project_id": "flutter-shop-app",
     "feature_area": "authentication", 
     "task_type": "implement",
     "components": ["login", "password_reset"]
   }
   ```

3. **Structured Response**: Server returns curated context:
   ```json
   {
     "business_rules": [
       {
         "rule_name": "Email Verification Required",
         "description": "All new accounts must verify email before activation"
       }
     ],
     "security_policies": [
       {
         "policy_name": "Password Requirements",
         "requirements": "Use bcrypt with 12 rounds minimum"
       }
     ],
     "architectural_decisions": [
       {
         "decision_title": "State Management",
         "decision": "Use BLoC pattern for authentication flows"
       }
     ]
   }
   ```

4. **AI Context**: The LLM uses this context to generate appropriate code that follows your project's rules and patterns.

## 6. Development and Testing

### Testing the Server

```bash
# Run the server
cargo run

# Test with MCP Inspector
npx @modelcontextprotocol/inspector

# Or test with a simple stdio client
echo '{"jsonrpc": "2.0", "method": "initialize", "params": {...}, "id": 1}' | cargo run
```

### Adding Context Data

You can add context data by:
1. Using the `create_project` MCP tool
2. Directly inserting into the SQLite database
3. Building additional MCP tools for data management (future enhancement)

## Key Features

- ✅ **MCP Protocol Compliance**: Full Model Context Protocol implementation
- ✅ **SQLite Storage**: Embedded database for efficient context storage
- ✅ **Multiple Transports**: stdio transport (SSE and HTTP streaming planned)
- ✅ **Type Safety**: JSON schema validation for all tool parameters
- ✅ **Tool Discovery**: Automatic tool discovery by MCP clients
- ✅ **Context Querying**: Advanced context filtering and retrieval
- 🚧 **Claude Desktop Integration**: Ready for Claude Desktop configuration
- 🚧 **Additional Tools**: More CRUD operations coming soon
