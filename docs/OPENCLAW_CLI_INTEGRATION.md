# OpenClaw Integration Guide for Context Server

## Overview

This guide shows how to integrate the **Context Server CLI** with **OpenClaw** for providing dynamic project context to AI agents running on Debian VMs.

## Architecture

```
┌─────────────────┐
│  Telegram User  │
└────────┬────────┘
         │ /query command
         ▼
┌─────────────────┐         ┌──────────────────────┐
│  Telegram Bot   │────────▶│  OpenClaw Agent      │
└─────────────────┘         │  (Decision Making)   │
                            └──────────┬───────────┘
                                       │ executes
                                       ▼
                            ┌──────────────────────┐
                            │  Context Server CLI  │
                            │  query/list/search   │
                            └──────────┬───────────┘
                                       │
                                       ▼
                            ┌──────────────────────┐
                            │  SQLite Database     │
                            │  (~/.config/...)     │
                            └──────────────────────┘
                                       │
                                       ▼
                            ┌──────────────────────┐
                            │  AI Agent (Claude)   │
                            │  Parses context +    │
                            │  Generates response  │
                            └──────────┬───────────┘
                                       │
                                       ▼
                            ┌──────────────────────┐
                            │  Telegram Response   │
                            │  Sends to user       │
                            └──────────────────────┘
```

## Setup Steps

### Step 1: Install Context Server on Debian VM

```bash
# Clone repository
git clone <repo-url> context-server-rs
cd context-server-rs

# Build release binary
cargo build --release

# Install to PATH
sudo cp target/release/context-server-rs /usr/local/bin/
chmod +x /usr/local/bin/context-server-rs

# Initialize database (creates ~/.config/context-server-rs/context.db)
context-server-rs serve &
sleep 2
pkill context-server-rs
```

Verify installation:
```bash
context-server-rs --version
context-server-rs list business_rule --format json
```

### Step 2: Configure OpenClaw Tools

Add context-server-rs commands to OpenClaw agent configuration:

**Option A: YAML Configuration (Recommended)**

Create `~/.openclaw/tools/context-server.yaml`:

```yaml
name: "Context Server"
description: "Query project context for code generation decisions"

tools:
  - id: "context-query"
    name: "Query Contexts"
    description: "Query business rules, decisions, and policies for a task"
    command: |
      context-server-rs query \
        --task "${{ input.task }}" \
        --project "${{ input.project }}" \
        --format json
    input_schema:
      type: object
      properties:
        task:
          type: string
          description: "Task name (e.g., 'auth', 'api', 'storage')"
        project:
          type: string
          description: "Project name (optional)"
      required: ["task"]

  - id: "context-search"
    name: "Search Contexts"
    description: "Full-text search for context by keyword"
    command: |
      context-server-rs search \
        "${{ input.query }}" \
        --project "${{ input.project }}" \
        --format json
    input_schema:
      type: object
      properties:
        query:
          type: string
          description: "Search keyword or phrase"
        project:
          type: string
          description: "Project name (optional)"
      required: ["query"]

  - id: "context-list"
    name: "List Contexts"
    description: "List all contexts of a specific type"
    command: |
      context-server-rs list "${{ input.type }}" \
        --project "${{ input.project }}" \
        --format json
    input_schema:
      type: object
      properties:
        type:
          type: string
          enum:
            - business_rule
            - architectural_decision
            - performance_requirement
            - security_policy
            - feature
          description: "Context type to list"
        project:
          type: string
          description: "Project name (optional)"
      required: ["type"]

  - id: "context-get"
    name: "Get Context"
    description: "Retrieve specific context by ID"
    command: |
      context-server-rs get "${{ input.id }}" --format json
    input_schema:
      type: object
      properties:
        id:
          type: string
          description: "Context ID (e.g., 'rule-001')"
      required: ["id"]
```

**Option B: JSON Configuration**

Create `~/.openclaw/config.json`:

```json
{
  "tools": {
    "context-server": {
      "query": {
        "description": "Query business rules and decisions for a task",
        "command": "context-server-rs query --task '{task}' --project '{project}' --format json",
        "returns": "json"
      },
      "search": {
        "description": "Search contexts by keyword",
        "command": "context-server-rs search '{query}' --project '{project}' --format json",
        "returns": "json"
      }
    }
  }
}
```

### Step 3: Configure OpenClaw Agent Prompt

Add context-aware instructions to OpenClaw agent system prompt:

```markdown
## Project Context Tools

You have access to a Context Server with project-specific information:

### Query Contexts
Before generating code, query relevant context:
```bash
context-server-rs query --task <task-name> --project <project-name> --format json
```

Use this to understand:
- Business rules and constraints
- Architectural decisions
- Performance requirements
- Security policies

### Search Examples

Search for specific topics:
```bash
context-server-rs search "authentication" --format json
context-server-rs search "caching" --format json
context-server-rs search "rate limiting" --format json
```

### Usage Guidelines

1. **Always Query First:** Before generating code, query the context server
2. **Parse JSON Response:** Extract relevant rules and decisions
3. **Apply Constraints:** Ensure generated code follows the business rules
4. **Document Decisions:** Reference which context rules were applied
5. **Search for Examples:** Look for existing patterns in the context

### Tools Available

- `context-query`: Query by task (auth, api, storage, etc.)
- `context-search`: Full-text search
- `context-list`: List all of a type
- `context-get`: Get specific by ID

Always use `--format json` for clean output.
```

### Step 4: Create Telegram Bot Integration

Create `telegram_bot.py` to bridge OpenClaw and Telegram:

```python
#!/usr/bin/env python3

import json
import re
import subprocess
from telegram import Update
from telegram.ext import (
    Application, CommandHandler, MessageHandler, 
    filters, ContextTypes
)
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

async def query_context(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle /query command from Telegram"""
    if not context.args:
        await update.message.reply_text(
            "Usage: /query <task> [project]\n"
            "Example: /query auth myapp"
        )
        return

    task = context.args[0]
    project = context.args[1] if len(context.args) > 1 else "default"
    
    try:
        # Call context server CLI
        result = subprocess.run(
            ["context-server-rs", "query", "--task", task, 
             "--project", project, "--format", "json"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        if result.returncode != 0:
            await update.message.reply_text(
                f"Error: {result.stderr}"
            )
            return
        
        # Parse JSON and send to OpenClaw
        context_data = json.loads(result.stdout)
        
        # Send to OpenClaw agent
        prompt = f"""User is requesting context for task '{task}' in project '{project}'.

Context retrieved from database:
{json.dumps(context_data, indent=2)}

Please analyze this context and provide relevant insights for generating code related to this task."""
        
        # Call OpenClaw agent (pseudo-code)
        response = await call_openclaw_agent(prompt)
        
        # Send response back to Telegram
        await update.message.reply_text(response)
        
    except subprocess.TimeoutExpired:
        await update.message.reply_text("Query timeout - database may be large")
    except json.JSONDecodeError:
        await update.message.reply_text("Invalid JSON response from context server")
    except Exception as e:
        await update.message.reply_text(f"Error: {str(e)}")


async def search_context(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle /search command"""
    if not context.args:
        await update.message.reply_text(
            "Usage: /search <keyword>\n"
            "Example: /search pagination"
        )
        return

    query = " ".join(context.args)
    
    try:
        result = subprocess.run(
            ["context-server-rs", "search", query, "--format", "json"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        if result.returncode != 0:
            await update.message.reply_text(f"Error: {result.stderr}")
            return
        
        context_data = json.loads(result.stdout)
        
        # Send to OpenClaw with search results
        prompt = f"""Users searched for '{query}' in project context.

Search results:
{json.dumps(context_data, indent=2)}

Please summarize these findings and explain how they relate to the search term."""
        
        response = await call_openclaw_agent(prompt)
        await update.message.reply_text(response)
        
    except Exception as e:
        await update.message.reply_text(f"Search failed: {str(e)}")


async def call_openclaw_agent(prompt: str) -> str:
    """Call OpenClaw agent with prompt (pseudo-code)"""
    # This would call your OpenClaw agent
    # Implementation depends on OpenClaw API
    
    result = subprocess.run(
        ["openclaw-agent", "run", "--prompt", prompt],
        capture_output=True,
        text=True,
        timeout=30
    )
    
    return result.stdout or "No response from agent"


def main():
    """Start Telegram bot"""
    # Get token from environment
    token = os.getenv("TELEGRAM_BOT_TOKEN")
    if not token:
        raise ValueError("TELEGRAM_BOT_TOKEN not set")
    
    app = Application.builder().token(token).build()
    
    # Register handlers
    app.add_handler(CommandHandler("query", query_context))
    app.add_handler(CommandHandler("search", search_context))
    
    # Start bot
    app.run_polling(allowed_updates=Update.ALL_TYPES)


if __name__ == "__main__":
    import os
    main()
```

Run the bot:
```bash
export TELEGRAM_BOT_TOKEN="your-token-here"
python3 telegram_bot.py
```

## Usage Examples

### Example 1: Query Authentication Rules

**User → Telegram:**
```
/query auth myapp
```

**Bot → Context Server:**
```bash
context-server-rs query --task auth --project myapp --format json
```

**Database Returns:**
```json
{
  "business_rules": [
    {
      "id": "br-001",
      "name": "All Users Must Be Authenticated",
      "description": "Every API endpoint handling user data requires authentication"
    }
  ],
  "security_policies": [
    {
      "id": "sp-001",
      "name": "JWT Token Expiration",
      "description": "JWT tokens must expire after 24 hours"
    }
  ]
}
```

**OpenClaw Agent → AI:**
```
"Based on the project context for authentication in myapp:
- All users must be authenticated
- JWT tokens expire after 24 hours
- Implement token refresh mechanism
- Validate signatures on every request"
```

**AI → Telegram:**
```
Here's the authentication implementation you need...
[Generated code following the rules above]
```

### Example 2: Search for Pagination Patterns

**User → Telegram:**
```
/search pagination
```

**Bot → Context Server:**
```bash
context-server-rs search "pagination" --format json
```

**AI Response:**
```
Found 3 related contexts for pagination:
1. Performance Requirement: "Paginate results by 50-100 items"
2. Architectural Decision: "Use cursor-based pagination"
3. Business Rule: "Never expose total count for security"
```

### Example 3: Generate Feature Implementation

**Workflow:**
1. User asks to implement a feature
2. OpenClaw queries: `context-server-rs list feature --format json`
3. OpenClaw searches: `context-server-rs search "api-patterns" --format json`
4. OpenClaw queries: `context-server-rs query --task "api" --format json`
5. AI generates code following all constraints
6. Response sent to Telegram

## Performance Optimization

### 1. Caching in OpenClaw

Cache frequently accessed contexts:

```python
# In OpenClaw agent system prompt
cache_contexts = {
    "auth": None,  # Cache auth context
    "api": None    # Cache API context
}

async def get_cached_context(task, project="default"):
    key = f"{task}:{project}"
    
    if key in cache_contexts and cache_contexts[key]:
        return cache_contexts[key]
    
    # Query context server
    result = subprocess.run(
        ["context-server-rs", "query", "--task", task, 
         "--project", project, "--format", "json"],
        capture_output=True, text=True
    )
    
    context_data = json.loads(result.stdout)
    cache_contexts[key] = context_data
    
    return context_data
```

### 2. Database Indexing

The Context Server automatically indexes:
- `id` (primary key)
- `name`
- `description`  
- `project_id`

For large datasets, manually add indexes:

```bash
sqlite3 ~/.config/context-server-rs/context.db <<EOF
CREATE INDEX idx_project_id ON business_rules(project_id);
CREATE INDEX idx_name ON business_rules(name);
EOF
```

### 3. Output Filtering

Use `jq` in OpenClaw for efficient parsing:

```bash
# Get only rule names
context-server-rs list business_rule --format json | \
  jq '.items[] | {id, name}'

# Filter by project
context-server-rs list business_rule --format json | \
  jq '.items[] | select(.project == "myapp")'
```

## Troubleshooting

### Database not found
**Error:** `No such file or directory`

**Fix:**
```bash
# Initialize database
context-server-rs serve &
sleep 2
kill %1

# Verify
ls ~/.config/context-server-rs/context.db
```

### OpenClaw tool syntax errors
**Error:** `Invalid command`

**Fix:** Ensure command paths are absolute:
```bash
# Wrong
context-server-rs query...

# Correct
/usr/local/bin/context-server-rs query...
```

### Telegram command not recognized
**Error:** `Unknown command`

**Fix:** Ensure bot has permissions:
```bash
# In Telegram: /setcommands
/query - Query project context
/search - Search contexts
```

## Security Considerations

### 1. Database Protection

```bash
# Restrict database access
chmod 600 ~/.config/context-server-rs/context.db

# Run bot with limited user
sudo useradd -m -s /bin/bash openclaw
sudo chown openclaw:openclaw ~/.config/context-server-rs/context.db
```

### 2. Command Injection Prevention

In Telegram bot, always use subprocess list form (not shell):

```python
# Safe
subprocess.run(["context-server-rs", "search", user_input], ...)

# Unsafe - don't do this!
os.system(f"context-server-rs search {user_input}")
```

### 3. API Key Protection

```bash
# Telegram token
export TELEGRAM_BOT_TOKEN="xxx"  # Via .env or secrets manager

# OpenClaw credentials
export OPENCLAW_API_KEY="xxx"    # Via environment
```

## Next Steps

1. **Deploy to Production**
   - Use systemd service for context-server-rs
   - Use PM2 or similar for Telegram bot
   - Set up monitoring/alerting

2. **Extend Context Types**
   - Add custom business rules
   - Define project-specific contexts
   - Create templates for new entities

3. **Integration with IDE**
   - Still support MCP server mode
   - Allow IDE to query context
   - Auto-suggest applicable rules

4. **Analytics**
   - Track which contexts are most used
   - Monitor query performance
   - Optimize database for typical queries

