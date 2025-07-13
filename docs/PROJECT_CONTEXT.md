# Project Context: MCP Context Server for AI Code Generation

## Project Overview

This project is a **Model Context Protocol (MCP) server** built in Rust that captures and maintains essential project information that AI agents (like Claude Desktop and Cursor IDE) cannot automatically discover or infer. The server provides curated, high-value context that enables AI agents to generate better production-quality code.

## Key Characteristics

- **Official MCP Implementation**: Uses the [rmcp SDK](https://github.com/modelcontextprotocol/rust-sdk) (official Rust MCP SDK)
- **Protocol Compliant**: Follows the [Model Context Protocol specification](https://modelcontextprotocol.io/introduction)
- **Embedded SQLite Database**: Uses Rusqlite for data persistence without external DB dependencies
- **Tokio Async Runtime**: Leverages Rust's async/await capabilities for efficient handling of requests
- **Multiple Transports**: Currently supports stdio, with SSE and HTTP streaming planned

## What is Model Context Protocol (MCP)?

[Model Context Protocol (MCP)](https://modelcontextprotocol.io/introduction) is an open protocol that standardizes how applications provide context to LLMs. MCP has:

- A specific client-server architecture
- Standardized communication format
- Built-in capabilities for resources, tools, prompts, and sampling
- A formalized specification
- Tool discovery and automatic schema validation
- Multiple transport options (stdio, SSE, HTTP streaming)

## This MCP Server Implementation

Our MCP Context Server:
- ✅ Implements the official MCP specification
- ✅ Uses the rmcp SDK for protocol compliance
- ✅ Provides MCP tools for context querying and management
- ✅ Supports tool discovery by MCP clients
- ✅ Includes JSON schema validation for all tools
- ✅ Focuses on storing and retrieving project context for AI code assistance

## Current Technical Stack

- **Language**: Rust
- **MCP SDK**: rmcp 0.2.0 (Official Rust MCP SDK)
- **Database**: SQLite (via Rusqlite)
- **Serialization**: Serde/Serde_json
- **Async Runtime**: Tokio
- **Protocol**: Model Context Protocol (stdio transport)

## Getting Started

1. Build and run the MCP server:
   ```
   cargo run --release
   ```

2. The server starts using stdio transport for MCP communication

3. Connect MCP clients (Claude Desktop, Cursor IDE, etc.) by configuring their MCP server settings

## MCP Tools Provided

- `query_context` - Query for context based on project, feature area, and task type
- `list_projects` - List all available projects
- `create_project` - Create a new project in the context database

## MCP Integration Examples

### Claude Desktop Configuration
```json
{
  "mcpServers": {
    "context-server": {
      "command": "/path/to/context-server-rs",
      "args": []
    }
  }
}
```

### MCP Tool Usage
```json
{
  "tool": "query_context",
  "arguments": {
    "project_id": "my-app",
    "feature_area": "authentication", 
    "task_type": "implement",
    "components": ["login", "signup"]
  }
}
```
