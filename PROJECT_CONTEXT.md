# Project Context: Context Server for AI Code Generation

## Project Overview

This project is a **lightweight HTTP REST API context server** built in Rust that captures and maintains essential project information that AI agents (like GitHub Copilot) cannot automatically discover or infer. The server provides curated, high-value context that enables AI agents to generate better production-quality code.

## Key Characteristics

- **Standard HTTP REST API**: This project uses Axum to provide REST API endpoints.
- **Embedded SQLite Database**: Uses Rusqlite for data persistence without external DB dependencies.
- **Tokio Async Runtime**: Leverages Rust's async/await capabilities for efficient handling of requests.

## Clarification: HTTP Server vs. Model Context Protocol (MCP)

This project is **not** an implementation of the Model Context Protocol (MCP). Rather, it's a standard HTTP REST API server with the following key distinctions:

### What is Model Context Protocol (MCP)?

[Model Context Protocol (MCP)](https://modelcontextprotocol.io/introduction) is an open protocol that standardizes how applications provide context to LLMs. MCP has:

- A specific client-server architecture
- Standardized communication format
- Built-in capabilities for resources, tools, prompts, and sampling
- A formalized specification

### This Project's Approach

Our Context Server:
- Uses standard HTTP/REST conventions
- Implements custom endpoints specific to our context storage needs
- Focuses on storing and retrieving project context for AI code assistance
- Doesn't implement or adhere to the MCP specification
- Is simpler and more focused on our specific use case

## Current Technical Stack

- **Language**: Rust
- **Web Framework**: Axum 0.7
- **Database**: SQLite (via Rusqlite)
- **Serialization**: Serde/Serde_json
- **Async Runtime**: Tokio

## Getting Started

1. Build and run the server:
   ```
   cargo run --release
   ```

2. The server will start at `http://127.0.0.1:8080/`

3. Use the REST API endpoints to store and retrieve context information

## Key API Endpoints

- `GET /health` - Health check endpoint
- `GET /projects` - List all projects
- `POST /projects` - Create a new project
- `GET /business_rules` - List all business rules
- `POST /context/query` - Query for context based on feature area, components, etc.
