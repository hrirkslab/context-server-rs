# GitHub Copilot Instructions

## Project Overview
We're building a Model Context Protocol (MCP) server using Rust that saves project context to a database. The server will:

1. Extract and process code context from projects
2. Store the context in a database for efficient retrieval
3. Provide MCP tools and resources for accessing the stored context
4. Support real-time updates when code changes

## Technical Stack
- **Language**: Rust
- **MCP SDK**: rmcp (Official Rust MCP SDK)
- **Database**: SQLite (embedded database)
- **Protocol**: Model Context Protocol (stdio transport)
- **Concurrency**: Utilize Rust's async/await with Tokio runtime

## Code Structure
- `/src/main.rs` - Entry point with MCP server initialization
- `/src/db/` - Database connection and operations
- `/src/context_server.rs` - MCP ServerHandler implementation
- `/src/models/` - Data structures and schema definitions

## Key Features to Implement
- MCP tool for querying context based on feature area and task type
- MCP tools for managing projects, business rules, and architectural decisions
- Efficient storage schema for code context
- Fast context retrieval via MCP protocol
- Authentication and authorization (future)
- Change detection and incremental updates (future)

## Non-Functional Requirements
- High performance with minimal latency
- Thread safety for concurrent operations
- Comprehensive error handling
- Proper logging for debugging and monitoring
- Test coverage for critical components

## Development Approach
1. Start with core data structures and database schema ✅
2. Implement MCP ServerHandler and tools ✅
3. Set up MCP protocol communication via stdio ✅
4. Add more MCP tools for CRUD operations
5. Add MCP resources for browsing context
6. Optimize for performance and scale