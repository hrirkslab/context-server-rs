# Example OpenClaw System Context

This directory contains example context files for integrating this context server with OpenClaw autonomous agents.

## Files in this Directory

1. **openclaw_constraints.sql** - Operational constraints and guardrails
2. **openclaw_dependencies.sql** - Component dependencies and impact analysis
3. **openclaw_business_rules.sql** - Business logic and domain rules
4. **openclaw_security_policies.sql** - Security requirements and patterns
5. **openclaw_tags.sql** - Tag categories for organizing context

## Loading Context into the Server

```bash
# Start the server
cargo run --release

# In another terminal, load the context (using SQLite CLI)
sqlite3 ~/.config/context-server-rs/context.db < examples/openclaw_constraints.sql
sqlite3 ~/.config/context-server-rs/context.db < examples/openclaw_dependencies.sql
```

## OpenClaw Integration Points

### 1. Before Executing Actions
OpenClaw should query constraints:
```
GET /query_context?feature_area=deployment&task_type=safety_checks
```

### 2. Logging Actions
OpenClaw should log every action:
```
POST /create_entity
{
  "entity_type": "audit_trail",
  "initiator": "openclaw",
  "action": "deployed_service_v2.0",
  "status": "completed"
}
```

### 3. Checking Impact
Before making changes:
```
GET /get_dependencies?component=database_service&type=dependencies_of
```

## Expected OpenClaw Workflow

1. **Initialize** - Load all constraints and business rules
2. **Query** - Check what actions are allowed for the current task
3. **Plan** - Plan changes within constraint boundaries
4. **Execute** - Run the action
5. **Log** - Record what was done and the outcome
6. **Monitor** - Check for violations or issues
7. **Report** - Store metrics and results back to context server
