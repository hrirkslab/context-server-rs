# Context Server CLI Quick Reference

## One-Liners for OpenClaw Integration

**Query authentication rules:**
```bash
context-server-rs query --task auth --project myapp --format json
```

**Search for "pagination":**
```bash
context-server-rs search "pagination" --project myapp --format json
```

**List all security policies:**
```bash
context-server-rs list security_policy --format json
```

**Get specific context by ID:**
```bash
context-server-rs get "rule-123" --format json
```

## Command Structure

```
context-server-rs [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]

GLOBAL OPTIONS:
  -d, --db <PATH>          Database path (default: ~/.config/context-server-rs/context.db)
  -f, --format <FORMAT>    Output format: json, text, yaml (default: json)
  -p, --project <PROJECT>  Filter by project name

COMMANDS:
  serve                    Start MCP server (default)
  query [--task <TASK>]    Query contexts by task
  list <TYPE>              List contexts by type
  search <QUERY>           Full-text search
  get <ID>                 Get specific context by ID
```

## Output Formats

| Format | Use Case | Example |
|--------|----------|---------|
| `json` | Programmatic parsing | OpenClaw, CLI piping with jq |
| `text` | Human-readable | Terminal inspection |
| `yaml` | Configuration | Editing, documentation |

## Common Workflows

### 1. Before Coding - Query Context
```bash
# Understand business rules before writing auth code
context-server-rs query --task auth --project myapp --format json

# Filter with jq
context-server-rs query --task auth --format json | jq '.data.business_rules'
```

### 2. Search for Examples
```bash
# Find similar implementations
context-server-rs search "caching" --project myapp --format json

# Extract result IDs
context-server-rs search "pagination" --format json | jq '.data[].id'
```

### 3. List Entity Types
```bash
# See all business rules
context-server-rs list business_rule --format json

# Count items
context-server-rs list feature --format json | jq '.count'
```

### 4. Get Detailed Context
```bash
# Fetch full specification
context-server-rs get "rule-001" --format json

# Pretty-print YAML
context-server-rs get "rule-001" --format yaml
```

## Integration Examples

### OpenClaw Agent Config

```python
# In agent system prompt
context_tools = [
    "context-query: context-server-rs query --task '{task}' --project '{project}' --format json",
    "context-search: context-server-rs search '{query}' --project '{project}' --format json",
    "context-list: context-server-rs list '{type}' --format json"
]
```

### Telegram Bot Handler

```python
async def query_context(update, context):
    task = context.args[0]
    result = subprocess.run(
        ["context-server-rs", "query", "--task", task, "--format", "json"],
        capture_output=True, text=True
    )
    context_data = json.loads(result.stdout)
    # Send to OpenClaw agent...
```

### Shell Script Integration

```bash
#!/bin/bash
PROJECT=${1:-default}

echo "=== Security Policies ==="
context-server-rs list security_policy --project "$PROJECT" --format text

echo -e "\n=== Business Rules ==="
context-server-rs list business_rule --project "$PROJECT" --format text
```

## Performance Tips

1. **Always filter by project when possible**
   ```bash
   context-server-rs list business_rule --project myapp  # Fast
   context-server-rs list business_rule                  # Slow (all projects)
   ```

2. **Use JSON for programmatic access** (faster parsing than text/yaml)
   ```bash
   context-server-rs query --task auth --format json
   ```

3. **Cache results in OpenClaw** for repeated queries within same context

4. **Limit search with LIKE patterns**
   ```bash
   context-server-rs search "auth%"  # More specific = faster
   ```

## Error Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Command execution error |
| 2 | Invalid arguments |

Check stderr for error details:
```bash
context-server-rs query --invalid 2>&1
```

## Typical Response Times

| Operation | Time |
|-----------|------|
| CLI startup | 100-200ms |
| Simple query | 10-50ms |
| Complex search | 100-500ms |
| Total | ~200-700ms |

## Deployment Checklist

- [ ] Context Server built: `cargo build --release`
- [ ] Binary in PATH: `/usr/local/bin/context-server-rs`
- [ ] Database initialized: `~/.config/context-server-rs/context.db` exists
- [ ] Telegram bot installed: `pip install python-telegram-bot`
- [ ] OpenClaw tools configured: YAML/JSON config in place
- [ ] Test query works: `context-server-rs list business_rule --format json`
- [ ] Bot responds to /query: Telegram bot running and responding

## File Structure

```
context-server-rs/
├── src/
│   ├── cli/
│   │   ├── mod.rs              # CLI module entry
│   │   ├── commands.rs         # CliCommand trait
│   │   ├── router.rs           # Cli & Commands structs, CliRouter
│   │   ├── output.rs           # OutputFormatter trait
│   │   └── handlers/
│   │       ├── query.rs        # Query handler
│   │       ├── list.rs         # List handler
│   │       ├── search.rs       # Search handler
│   │       └── get.rs          # Get handler
│   ├── main.rs                 # Dual-mode entry point
│   └── ...
├── docs/
│   ├── CLI_USAGE.md            # Full usage guide
│   └── OPENCLAW_CLI_INTEGRATION.md  # OpenClaw setup
└── Cargo.toml                  # Dependencies
```

## Common Mistakes

❌ **Wrong:** Running from current directory
```bash
./context-server-rs query  # May not work
```

✅ **Right:** Use absolute path
```bash
/usr/local/bin/context-server-rs query
context-server-rs query          # If in PATH
```

---

❌ **Wrong:** Not filtering by project
```bash
context-server-rs list business_rule  # Slow, all projects
```

✅ **Right:** Filter by project
```bash
context-server-rs list business_rule --project myapp  # Fast
```

---

❌ **Wrong:** Piping binary data
```bash
context-server-rs list feature | grep "name"  # Bad parsing
```

✅ **Right:** Parse JSON properly
```bash
context-server-rs list feature --format json | jq '.data[] | .name'
```

---

❌ **Wrong:** Not handling errors
```bash
context-server-rs query --invalid  # Silent failure
```

✅ **Right:** Check exit codes
```bash
context-server-rs query --invalid || echo "Query failed: $?"
```

## Testing Your Setup

```bash
# 1. Verify binary
context-server-rs --version

# 2. Query database exists
context-server-rs list business_rule --format json

# 3. Search works
context-server-rs search "test" --format json

# 4. Format conversion
context-server-rs list security_policy --format yaml

# 5. Performance test
time context-server-rs query --task default --format json
```

Expected output for healthy setup:
- No errors
- Valid JSON/YAML output
- Query completes in <1 second
- Database accessible

