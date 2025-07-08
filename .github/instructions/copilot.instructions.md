# GitHub Copilot Instructions

## Project Overview
We're building a Model-Context-Provider (MCP) server using Rust that saves project context to a database. The server will:

1. Extract and process code context from projects
2. Store the context in a database for efficient retrieval
3. Provide APIs for accessing the stored context
4. Support real-time updates when code changes

## Technical Stack
- **Language**: Rust
- **Database**: Consider PostgreSQL, SQLite, or a document DB depending on scale requirements
- **API**: RESTful and/or gRPC
- **Concurrency**: Utilize Rust's async/await with Tokio runtime

## Code Structure
- `/src/main.rs` - Entry point with server initialization
- `/src/db/` - Database connection and operations
- `/src/api/` - API endpoints and handlers
- `/src/context/` - Context extraction and processing
- `/src/models/` - Data structures and schema definitions

## Key Features to Implement
- File system traversal to gather project structure
- Code parsing and semantic analysis
- Efficient storage schema for code context
- Fast context retrieval APIs
- Authentication and authorization
- Change detection and incremental updates

## Non-Functional Requirements
- High performance with minimal latency
- Thread safety for concurrent operations
- Comprehensive error handling
- Proper logging for debugging and monitoring
- Test coverage for critical components

## Development Approach
1. Start with core data structures and database schema
2. Implement basic context extraction functionality
3. Set up API endpoints for CRUD operations
4. Add authentication and security measures
5. Optimize for performance and scale