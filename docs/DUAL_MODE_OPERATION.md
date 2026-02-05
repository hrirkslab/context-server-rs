# Context Server - Dual-Mode Operation Guide

## Overview

The Context Server supports **two operational modes**:

1. **MCP Server Mode** (Default) - Traditional MCP/stdio interface
2. **CLI Mode** - Command-line interface for scripts and OpenClaw integration

The same binary automatically detects which mode to use based on arguments.

---

## Mode Detection Logic

The binary intelligently selects the mode:

```
┌─ Start context-server-rs ─┐
│                            │
├─ No arguments?            ├─► Run as MCP Server
│  OR                       │
├─ Arguments = "serve"?     ├─► Run as MCP Server
│  OR                       │
├─ No recognized CLI args?  ├─► Run as MCP Server
│                            │
└─ CLI command detected?    ├─► Run as CLI (query/list/search/get)
   (query/list/search/get)   │
```

---

## MCP Server Mode

### Default Operation (No Arguments)

```bash
# All of these start MCP server mode:
context-server-rs
context-server-rs serve
context-server-rs serve --port 9000
```

**Output:**
```
    Compiling context-server-rs v0.1.0
     Finished release [optimized] target(s) in 0.49s
      Running `target/release/context-server-rs`
Starting MCP Context Server
Database initialized at /home/user/.config/context-server-rs/context.db
Enhanced MCP Context Server started successfully
```

### VS Code Integration

**For VS Code Extension:**
```json
{
  "context-server": {
    "command": "context-server-rs",
    "args": ["serve"],
    "env": {}
  }
}
```

### Claude Desktop Integration

**Add to `claude_desktop_config.json`:**
```json
{
  "mcpServers": {
    "context-server": {
      "command": "context-server-rs",
      "args": ["serve"]
    }
  }
}
```

### Systemd Service (Production)

**Create `/etc/systemd/system/context-server.service`:**
```ini
[Unit]
Description=Context Server MCP
After=network.target

[Service]
Type=simple
User=context-server
WorkingDirectory=/home/context-server
ExecStart=/usr/local/bin/context-server-rs serve
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

**Enable and start:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable context-server
sudo systemctl start context-server
```

---

## CLI Mode

### Query Mode

Query contexts by task and project:

```bash
# Query auth contexts
context-server-rs query --task auth --project myapp

# Query without project (uses "default")
context-server-rs query --task api

# Output as YAML
context-server-rs query --task auth --format yaml
```

**Output:**
```json
{
  "status": "success",
  "data": {
    "business_rules": [...],
    "architectural_decisions": [...],
    "performance_requirements": [...],
    "security_policies": [...]
  }
}
```

### List Mode

List all contexts of a specific type:

```bash
# List business rules
context-server-rs list business_rule

# List for specific project
context-server-rs list security_policy --project myapp

# Count entities
context-server-rs list feature --format json | jq '.count'
```

**Output:**
```json
{
  "entity_type": "business_rule",
  "count": 5,
  "items": [
    {
      "id": "rule-001",
      "name": "User Authentication",
      "description": "All users must be authenticated"
    }
  ]
}
```

### Search Mode

Full-text search across contexts:

```bash
# Search for "pagination"
context-server-rs search "pagination"

# Search in specific project
context-server-rs search "cache" --project myapp

# Get JSON for parsing
context-server-rs search "api" --format json | jq '.results'
```

### Get Mode

Retrieve specific context by ID:

```bash
# Get by ID
context-server-rs get "rule-001"

# Format as YAML
context-server-rs get "arch-decision-5" --format yaml
```

---

## Global Options (Works in All Modes)

```bash
-d, --db <PATH>
    Override database path
    Default: ~/.config/context-server-rs/context.db
    
-f, --format <FORMAT>
    Output format for CLI mode
    Options: json (default), text, yaml
    
-p, --project <PROJECT>
    Filter results by project
    Affects: query, list, search
```

**Examples:**
```bash
# Custom database
context-server-rs list business_rule --db /tmp/custom.db

# Custom format
context-server-rs query --task auth --format yaml

# Project filtering
context-server-rs search "auth" --project myapp
```

---

## Real-World Scenarios

### Scenario 1: IDE Integration (MCP Server)

**Setup:**
1. Build: `cargo build --release`
2. Install: `sudo cp target/release/context-server-rs /usr/local/bin/`
3. Configure MCP client to use `context-server-rs`
4. Restart IDE to load context server

**Usage:**
- IDE connects via MCP
- Requests context via MCP protocol
- Server responds with context data
- IDE displays suggestions based on context

**Command:**
```bash
context-server-rs serve
```

**No manual command invocation - MCP handles it all!**

---

### Scenario 2: OpenClaw Integration (CLI Mode)

**Setup:**
1. Build: `cargo build --release`
2. Install: `sudo cp target/release/context-server-rs /usr/local/bin/`
3. Add tool definitions to OpenClaw config
4. Reference in agent system prompt

**Usage:**
```bash
# Agent queries context
context-server-rs query --task feature-dev --project myapp --format json

# Agent receives structured data
{
  "status": "success",
  "data": {
    "business_rules": [ {...} ],
    ...
  }
}

# Agent parses and uses for code generation
```

**Example OpenClaw Tool Call:**
```yaml
tools:
  - name: "query_context"
    command: |
      context-server-rs query \
        --task "${{ input.task }}" \
        --project "${{ input.project }}" \
        --format json
```

---

### Scenario 3: Automation Script (CLI Mode)

**Backup all contexts to YAML:**
```bash
#!/bin/bash
BACKUP_DIR="/tmp/context-backup"
mkdir -p "$BACKUP_DIR"

for type in business_rule architectural_decision performance_requirement security_policy feature; do
    context-server-rs list "$type" --format yaml > "$BACKUP_DIR/$type.yaml"
done

echo "Backup complete in $BACKUP_DIR"
```

**Find all authentication-related contexts:**
```bash
#!/bin/bash
context-server-rs search "auth" --format json | \
  jq '.results[] | {type: .entity_type, name: .name}'
```

---

### Scenario 4: Telegram Bot Integration (CLI Mode)

**Python bot triggers CLI:**
```python
async def handle_context_query(update, context):
    task = context.args[0] if context.args else "default"
    
    # Call context-server-rs CLI
    result = subprocess.run(
        ["context-server-rs", "query", "--task", task, "--format", "json"],
        capture_output=True,
        text=True
    )
    
    # Parse JSON
    data = json.loads(result.stdout)
    
    # Send to OpenClaw agent
    agent_response = await openClaw.generate(
        context=data["data"],
        task=task
    )
    
    # Send back to Telegram
    await update.message.reply_text(agent_response)
```

---

## Logging and Debugging

### MCP Server Mode (Verbose)

```bash
# High verbosity
RUST_LOG=debug context-server-rs serve

# Specific module
RUST_LOG=context_server_rs=trace context-server-rs serve
```

### CLI Mode (Quiet)

CLI mode automatically suppresses most log output. To debug:

```bash
# Enable debug logging in CLI mode
RUST_LOG=debug context-server-rs query --task auth

# Write logs to file
RUST_LOG=debug context-server-rs query --task auth 2> cli-debug.log
```

---

## Troubleshooting

### Binary Not Found
```bash
Error: context-server-rs not found
```

**Fix:**
```bash
# Check if in PATH
which context-server-rs

# If not, install manually
cargo build --release
sudo cp target/release/context-server-rs /usr/local/bin/
```

### Server Won't Start
```bash
Error: Address already in use
```

**Fix:**
```bash
# Kill existing process
pkill context-server-rs

# Or use different port
context-server-rs serve --port 9001
```

### CLI Command Returns Wrong Mode
```bash
# If you see "Starting MCP Context Server" when you wanted CLI
# You may not have the right arguments

# Wrong (starts server)
context-server-rs --task auth

# Correct (runs query)
context-server-rs query --task auth
```

### Database Issues
```bash
Error: No such file or directory
```

**Fix:**
```bash
# Initialize database
context-server-rs serve &
sleep 2
pkill context-server-rs

# Verify database exists
ls ~/.config/context-server-rs/context.db
```

---

## Performance Characteristics

### MCP Server Mode
- **Startup:** 100-200ms
- **Per request:** 10-500ms (depends on query complexity)
- **Memory:** ~50-100MB idle
- **Throughput:** ~100+ concurrent connections

### CLI Mode
- **Startup:** 100-200ms
- **Query execution:** 10-500ms
- **Output formatting:** <10ms
- **Total time:** ~200-700ms per invocation

**Optimization Tips:**
- Use project filtering to reduce dataset
- Cache results in client (OpenClaw)
- Use JSON format for fastest parsing
- Batch queries where possible

---

## Architecture Notes

The dual-mode operation is achieved through:

1. **Main Entry Point (`main.rs`):**
   - Parses arguments using Clap
   - Detects mode from arguments
   - Routes to appropriate handler

2. **CLI Module (`src/cli/`):**
   - Commands abstraction (CliCommand trait)
   - Handlers for each command type
   - Output formatting (JSON, Text, YAML)
   - Router for command orchestration

3. **SOLID Principles Applied:**
   - Single Responsibility: Each handler has one purpose
   - Liskov Substitution: All handlers implement CliCommand
   - Interface Segregation: OutputFormatter separated from commands
   - Dependency Inversion: Router depends on abstractions
   - Open/Closed: New commands don't require modifying existing code

---

## Next Steps

1. **Deploy to Production**
   - Use systemd service for automatic startup
   - Monitor logs with `journalctl`
   - Set up log rotation

2. **Expand CLI Commands**
   - Add `export` command for bulk data export
   - Add `validate` command for context consistency
   - Add `schema` command to inspect database

3. **Enhance OpenClaw Integration**
   - Create OpenClaw-specific response formatting
   - Add context relevance scoring
   - Implement result caching

4. **IDE Support**
   - Create VS Code extension using MCP server
   - Support for Cursor IDE
   - Support for JetBrains IDEs (via plugin)

