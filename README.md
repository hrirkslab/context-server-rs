# Professional Context Engine

A comprehensive context management system for AI-powered development, featuring both a Rust-based MCP server and a VS Code extension for intelligent context assistance.

## 🚀 Quick Start

### MCP Context Server
```bash
# Build and run the MCP server
cargo run --release

# Or simply run (default mode)
context-server-rs
```

### CLI Mode (OpenClaw Integration)
```bash
# Query contexts by task
context-server-rs query --task auth --project myapp --format json

# List all business rules
context-server-rs list business_rule --project myapp

# Search for "pagination"
context-server-rs search "pagination" --project myapp

# Get specific context by ID
context-server-rs get "rule-123" --project myapp
```

See [CLI Usage Guide](docs/CLI_USAGE.md) for complete documentation. Perfect for **OpenClaw agent integration** and **programmatic access**.

### VS Code Extension
```bash
# Build the VS Code extension
./scripts/build-extension.sh  # Linux/Mac
# or
.\scripts\build-extension.ps1  # Windows

# Install the extension
code --install-extension vscode-extension/professional-context-engine-1.0.0.vsix
```

## 📦 Repository Structure

```
├── src/                          # Rust MCP server source code
│   ├── api/                      # HTTP API endpoints
│   ├── context/                  # Context management logic
│   ├── db/                       # Database layer
│   ├── models/                   # Data models
│   ├── repositories/             # Data access layer
│   └── services/                 # Business logic services
├── vscode-extension/             # VS Code extension
│   ├── src/                      # TypeScript source code
│   ├── INSTALLATION.md           # Installation guide
│   ├── QUICK_START.md            # Quick setup guide
│   └── README.md                 # Extension documentation
├── docs/                         # Project documentation
├── examples/                     # Usage examples
├── tests/                        # Integration tests
├── scripts/                      # Build and utility scripts
│   ├── build-extension.sh         # Extension build script (Linux/Mac)
│   ├── build-extension.ps1        # Extension build script (Windows)
└── README.md                     # This file
```

## 🎯 Features

### MCP Context Server
- **Model Context Protocol (MCP) compliant** - Works with Claude Desktop, Cursor IDE, and other MCP clients
- **Intelligent context management** - Store, query, and analyze project context
- **Real-time synchronization** - WebSocket support for live updates
- **Advanced search** - Semantic and full-text search capabilities
- **Plugin architecture** - Extensible with custom plugins
- **Multi-project support** - Manage context across multiple projects

### CLI Interface (NEW!)
- **Dual-mode binary** - Runs as MCP server or CLI tool automatically
- **OpenClaw integration** - Designed for autonomous agent workflows
- **Multiple output formats** - JSON (programmatic), Text (terminal), YAML (config)
- **Fast query execution** - ~10-500ms per context query from database
- **SOLID architecture** - Trait-based design for extensibility
- **Command types** - Query (task-based), List (type-based), Search (full-text), Get (ID-based)

See [CLI Quick Reference](docs/CLI_QUICK_REFERENCE.md) and [OpenClaw Integration Guide](docs/OPENCLAW_CLI_INTEGRATION.md)

### VS Code Extension
- **Real-time context suggestions** - Hover and code action providers
- **Intelligent file monitoring** - Automatic analysis of code changes
- **Context creation** - Create context entries from selected code
- **Project insights** - Analytics dashboard for context health
- **Team collaboration** - Real-time synchronization across team members
- **Custom analysis rules** - Configure project-specific context extraction

## 🔧 Installation & Setup

### Prerequisites
- Rust 1.70+ (for MCP server)
- Node.js 16+ (for VS Code extension)
- VS Code 1.74+ (for extension)

### MCP Context Server

1. **Clone and build**:
   ```bash
   git clone <repository-url>
   cd professional-context-engine
   cargo build --release
   ```

2. **Run the server**:
   ```bash
   cargo run --release
   ```

3. **Configure MCP clients** (see [MCP Integration Guide](#mcp-integration))

### VS Code Extension

1. **Build the extension**:
   ```bash
   # Linux/Mac
   ./scripts/build-extension.sh
   
   # Windows
   .\scripts\build-extension.ps1
   ```

2. **Install in VS Code**:
   ```bash
   code --install-extension vscode-extension/professional-context-engine-1.0.0.vsix
   ```

3. **Configure the extension**:
   - Open VS Code Settings (`Ctrl+,`)
   - Search for "Context Engine"
   - Set server URL to `http://localhost:3000`

For detailed setup instructions, see:
- [VS Code Extension Installation Guide](vscode-extension/INSTALLATION.md)
- [VS Code Extension Quick Start](vscode-extension/QUICK_START.md)

## 🔌 MCP Integration

### Claude Desktop

Add to your Claude Desktop configuration:

**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "context-server": {
      "command": "path/to/your/context-server-executable",
      "args": [],
      "env": {}
    }
  }
}
```

### Cursor IDE

Configure in Cursor's MCP settings:
```json
{
  "mcpServers": {
    "context-server": {
      "command": "cargo",
      "args": ["run", "--release"],
      "cwd": "/path/to/professional-context-engine"
    }
  }
}
```

## 🛠️ Development

### Building from Source

```bash
# Build MCP server
cargo build --release

# Build VS Code extension
cd vscode-extension
npm install
npm run compile
npx vsce package
```

### Running Tests

```bash
# Run Rust tests
cargo test

# Run VS Code extension tests
cd vscode-extension
npm test
```

### Development Workflow

1. **MCP Server Development**:
   ```bash
   # Run in development mode
   cargo run
   
   # Run with debug logging
   RUST_LOG=debug cargo run
   ```

2. **VS Code Extension Development**:
   ```bash
   cd vscode-extension
   npm run watch  # Compile in watch mode
   # Then press F5 in VS Code to launch extension host
   ```

## 📚 Documentation

- [CLI Usage Guide](docs/CLI_USAGE.md) - Complete CLI command reference
- [CLI Quick Reference](docs/CLI_QUICK_REFERENCE.md) - One-liners and common workflows
- [OpenClaw Integration Guide](docs/OPENCLAW_CLI_INTEGRATION.md) - Setup with Telegram & AI agents
- [Dual-Mode Operation Guide](docs/DUAL_MODE_OPERATION.md) - MCP server and CLI usage
- [Deployment Guide](docs/DEPLOYMENT.md)
- [Shipping Guide](docs/SHIPPING_GUIDE.md)
- [Production Readiness](docs/PRODUCTION_READINESS.md)
- [VS Code Extension Guide](vscode-extension/README.md)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Issues**: Report bugs and request features via [GitHub Issues]
- **Documentation**: Check the [docs/](docs/) directory
- **Examples**: See [examples/](examples/) for usage examples

## 🎯 Roadmap

- [ ] Enhanced semantic search capabilities
- [ ] Additional IDE integrations (IntelliJ, Vim)
- [ ] Cloud deployment options
- [ ] Advanced analytics and reporting
- [ ] Machine learning-powered context suggestions

---

**Built with ❤️ for developers who love intelligent, context-aware coding assistance.**