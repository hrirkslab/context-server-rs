# Context Server CLI Usage Guide

## Overview

The Context Server now supports **dual-mode operation**:

1. **MCP Server Mode** (default): Traditional MCP/stdio interface for IDEs like VS Code
2. **CLI Mode**: Command-line interface for OpenClaw integration and programmatic access

## Quick Start

### Server Mode (Default)

Run without arguments or with `serve` to start the MCP server:

```bash
# Start MCP server
context-server-rs

# Equivalent: explicit serve command
context-server-rs serve
```

### CLI Mode (OpenClaw Integration)

Query contexts directly via CLI:

```bash
# Query by task and project
context-server-rs query --task "authentication" --project "my-app"

# List all business rules
context-server-rs list business_rule

# Search for "pagination" across all contexts
context-server-rs search "pagination"

# Get specific context by ID
context-server-rs get "rule-123"
```

## Command Reference

### Query Command

Query contexts by task and project name.

**Usage:**
```bash
context-server-rs query [OPTIONS]
```

**Options:**
- `-t, --task <TASK>` - Task name (optional, defaults to "default")
- `-p, --project <PROJECT>` - Project name for filtering
- `-d, --db <PATH>` - Custom database path
- `-f, --format <FORMAT>` - Output format: `json` (default), `text`, `yaml`

**Example:**
```bash
# Query authentication task in myapp project
context-server-rs query --task auth --project myapp --format json

# Query default task, output as text
context-server-rs query --format text
```

**Output (JSON):**
```json
{
  "status": "success",
  "data": {
    "business_rules": [
      {
        "id": "br-001",
        "name": "User Authentication",
        "description": "All users must be authenticated",
        "domain": "auth"
      }
    ],
    "architectural_decisions": [],
    "performance_requirements": [],
    "security_policies": [],
    "features": []
  }
}
```

---

### List Command

List all contexts of a specific type.

**Usage:**
```bash
context-server-rs list <TYPE> [OPTIONS]
```

**Types:**
- `business_rule` - Business rules and constraints
- `architectural_decision` - Architecture ADRs
- `performance_requirement` - Performance targets
- `security_policy` - Security policies
- `feature` - Feature specifications

**Options:**
- `-p, --project <PROJECT>` - Filter by project
- `-d, --db <PATH>` - Custom database path
- `-f, --format <FORMAT>` - Output format: `json`, `text`, `yaml`

**Examples:**
```bash
# List all business rules
context-server-rs list business_rule

# List security policies for a project
context-server-rs list security_policy --project myapp

# List features as YAML
context-server-rs list feature --format yaml
```

**Output (JSON):**
```json
{
  "status": "success",
  "entity_type": "business_rule",
  "count": 3,
  "data": [
    {
      "id": "br-001",
      "name": "User Authentication"
    }
  ]
}
```

---

### Search Command

Full-text search across business rules, architectural decisions, security policies, performance requirements, and features.

**Usage:**
```bash
context-server-rs search <QUERY> [OPTIONS]
```

**Options:**
- `-p, --project <PROJECT>` - Filter by project
- `-d, --db <PATH>` - Custom database path
- `-f, --format <FORMAT>` - Output format: `json`, `text`, `yaml`

**Examples:**
```bash
# Search for "pagination"
context-server-rs search "pagination"

# Search in specific project
context-server-rs search "cache" --project myapp

# Search, output as text
context-server-rs search "authentication" --format text
```

**Output (JSON):**
```json
{
  "status": "success",
  "query": "pagination",
  "count": 1,
  "data": [
    {
      "id": "perf-001",
      "name": "Pagination Performance",
      "description": "Search results must paginate with < 100ms response time",
      "type": "performance_requirement"
    }
  ]
}
```

---

### Get Command

Retrieve a specific context by ID.

**Usage:**
```bash
context-server-rs get <ID> [OPTIONS]
```

**Options:**
- `-d, --db <PATH>` - Custom database path
- `-f, --format <FORMAT>` - Output format: `json`, `text`, `yaml`

**Examples:**
```bash
# Get by ID
context-server-rs get "rule-123"

# Get and format as YAML
context-server-rs get "br-001" --format yaml
```

**Output (JSON):**
```json
{
  "status": "success",
  "entity_type": "business_rule",
  "data": {
    "id": "rule-123",
    "name": "Rate Limiting",
    "description": "API endpoints must enforce rate limits"
  }
}
```

---

## Global Options

These options work with all commands:

- `-d, --db <PATH>` - Override default database path
  - Default: `~/.config/context-server-rs/context.db`
  
- `-f, --format <FORMAT>` - Output format
  - `json` (default): Machine-readable JSON
  - `text`: Human-readable key-value format
  - `yaml`: YAML format for configuration files
  
- `-p, --project <PROJECT>` - Filter results by project
  - Defaults to "default" if not specified

**Example with all options:**
```bash
context-server-rs query \
  --task auth \
  --project myapp \
  --format json \
  --db /tmp/custom.db
```

---

## OpenClaw Integration

### Setup

1. **Add context-server-rs to OpenClaw agents:**

```yaml
# In OpenClaw agent config
tools:
  - name: "context-query"
    description: "Query project context from Context Server"
    command: |
      context-server-rs query --task "$task" --project "$project" --format json
    
  - name: "context-search"
    description: "Search contexts by keyword"
    command: |
      context-server-rs search "$query" --project "$project" --format json
    
  - name: "context-list"
    description: "List all contexts of a type"
    command: |
      context-server-rs list "$type" --format json
```

### Usage in OpenClaw

**Query authentication rules for a project:**
```
Agent: Query auth rules for project "myapp"
→ Calls: context-server-rs query --task auth --project myapp --format json
→ Returns: Business rules, architectural decisions, security policies
```

**Search for pagination requirements:**
```
Agent: Find pagination-related context
→ Calls: context-server-rs search "pagination" --format json
→ Returns: All matching requirements across tables
```

**List all security policies:**
```
Agent: Get all security policies
→ Calls: context-server-rs list security_policy --format json
→ Returns: Complete security policy list
```

---

## Output Formats

### JSON (Default)

Machine-readable JSON for programmatic access:

```json
{
  "status": "success",
  "entity_type": "business_rule",
  "data": {
    "id": "rule-001",
    "name": "Principle",
    "description": "..."
  }
}
```

**Use when:** Parsing in OpenClaw, automation, piping to tools

### Text (Human-Readable)

Formatted for terminal display:

```
Entity Type: business_rule
ID: rule-001
Name: Principle
Description: ...
```

**Use when:** Manual inspection, simple text processing

### YAML

Configuration-friendly format:

```yaml
entity_type: business_rule
id: rule-001
name: Principle
description: ...
```

**Use when:** Configuration files, documentation, manual editing

---

## Performance Characteristics

- **CLI invocation overhead:** ~100-200ms (startup)
- **Database query time:** 10-500ms (depending on dataset size)
- **Output formatting:** <10ms
- **Total per command:** ~200-700ms practical

**Optimization tips:**
- Filter by project when possible: `--project myapp`
- Use lower result limits in search (auto-limited to 20 per table)
- Cache results in OpenClaw for repeated queries within same task

---

## Error Handling

Commands return exit codes:
- `0`: Success
- `1`: Command execution error
- `2`: Invalid arguments

Error messages go to stderr; results go to stdout.

**Example error:**
```
Error: Database not found at ~/.config/context-server-rs/context.db
Hint: Initialize database with 'context-server-rs migrate' or provide --db path
```

---

## Troubleshooting

### Database Not Found
```bash
Error: No such file or directory (os error 2)
```
**Solution:** Ensure database is initialized:
```bash
context-server-rs serve  # Initialize database
# Or specify custom path: --db /path/to/context.db
```

### Command Not Found
```bash
Error: Unknown entity type 'unknown_type'
```
**Solution:** Use valid types: `business_rule`, `architectural_decision`, `performance_requirement`, `security_policy`, `feature`

### Invalid JSON Output
```bash
Error: Invalid UTF-8 in response
```
**Solution:** Check database doesn't have corrupted entries; try `--format text` instead

---

## Complete Command Examples

### Scenario: Adding new feature context

1. **List existing features:**
```bash
context-server-rs list feature --format text
```

2. **Query related business rules:**
```bash
context-server-rs query --task "feature-planning" --project myapp
```

3. **Search for similar features:**
```bash
context-server-rs search "dashboard" --format json | jq '.data'
```

### Scenario: OpenClaw integration flow

```bash
# 1. Agent wants to generate auth module
context-server-rs query --task auth --project myapp --format json

# 2. Agent wants to check similar implementations
context-server-rs search "authentication" --format json

# 3. Agent wants to ensure compilation compliance
context-server-rs get "security-rule-001" --format json
```

---

## Advanced Usage

### Piping with jq

Extract specific fields from JSON output:

```bash
# Get all business rule names
context-server-rs list business_rule --format json | jq '.data[].name'

# Get security policies for specific project
context-server-rs list security_policy --project myapp --format json | \
  jq '.data[] | select(.project == "myapp")'
```

### Scripting

Create reusable context queries:

```bash
#!/bin/bash
# get_project_context.sh

PROJECT=${1:-default}
DB=${2:-~/.config/context-server-rs/context.db}

echo "=== Business Rules ==="
context-server-rs list business_rule --project "$PROJECT" --db "$DB" --format text

echo -e "\n=== Architectural Decisions ==="
context-server-rs list architectural_decision --project "$PROJECT" --db "$DB" --format text

echo -e "\n=== Security Policies ==="
context-server-rs list security_policy --project "$PROJECT" --db "$DB" --format text
```

Usage:
```bash
./get_project_context.sh myapp
```

---

## Build and Deploy

### Building from Source

```bash
cargo build --release
# Binary: target/release/context-server-rs
```

### Docker Deployment

```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/context-server-rs /usr/local/bin/

ENTRYPOINT ["context-server-rs"]
CMD ["serve"]
```

Usage in OpenClaw container:
```bash
docker run -v ~/.config/context-server-rs:/root/.config/context-server-rs \
  context-server:latest query --task auth --format json
```

---

## Architecture Notes (SOLID Principles)

The CLI system follows SOLID design principles:

- **Single Responsibility:** Each command handler has one reason to change
- **Liskov Substitution:** All commands implement `CliCommand` trait
- **Interface Segregation:** Output formatting separated from commands
- **Dependency Inversion:** CliRouter depends on abstractions, not concrete handlers
- **Open/Closed:** New commands added without modifying existing code

This makes the CLI extensible: New commands, output formats, and entity types can be added without modifying existing code.

---

## Next Steps

1. **OpenClaw Integration:** Add context-server-rs tool definitions to OpenClaw agent configs
2. **Telegram Bot Bridge:** Create webhook endpoint that calls CLI and returns results
3. **Caching:** Implement result caching in OpenClaw for frequently accessed contexts
4. **Extended Queries:** Add support for complex queries (AND, OR, filters)

