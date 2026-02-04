# OpenClaw Integration Guide for Context Server

This document describes how to integrate the MCP Context Server with OpenClaw for autonomous system management and governance.

## Overview

The Context Server provides a persistent, queryable knowledge base for OpenClaw that includes:
1. **Constraints** - Operational guardrails and safety limits
2. **Audit Trails** - Complete history of all actions and decisions
3. **Dependencies** - Component relationships for impact analysis
4. **Tags/Categories** - Organizing context for quick retrieval
5. **Business Rules** - Domain logic and patterns

## Architecture

```
┌──────────────────────┐
│   OpenClaw Agent     │
│  (autonomous tasks)  │
└──────────┬───────────┘
           │ MCP Calls
           ▼
┌──────────────────────────────────────────────┐
│     MCP Context Server (Rust)                │
│     - SQLite Database                        │
│     - CRUD endpoints                         │
│     - Query service                          │
└──────────┬──────────────────┬────────────────┘
           │                  │
           ▼                  ▼
    ┌─────────────┐   ┌──────────────┐
    │ SQLite DB   │   │ Repositories │
    │ - Audit Log │   │ - Constraints│
    │ - Deps      │   │ - Deps       │
    │ - Contexts  │   │ - Tags       │
    └─────────────┘   └──────────────┘
```

## Integration Points

### 1. Initialize Context on Startup

OpenClaw should load all constraints and business rules at startup:

```bash
# Start server
cargo run --release &
SERVER_PID=$!

# Load example context
sleep 2  # Wait for server to initialize
sqlite3 ~/.config/context-server-rs/context.db < examples/openclaw_constraints.sql
sqlite3 ~/.config/context-server-rs/context.db < examples/openclaw_dependencies.sql

# OpenClaw can now query
curl http://localhost:3000/query_context?area=deployment
```

### 2. Query Constraints Before Action

Before executing any action, OpenClaw should query applicable constraints:

```rust
// Example: Check constraints before deploying
let constraints = context_server.list_constraints(
    project_id: "project_001",
    target: "deployment:production"
)?;

for constraint in constraints {
    if !constraint.enabled {
        continue;
    }
    
    match constraint.constraint_type {
        ConstraintType::ApprovalRequired => {
            // Request approval from team leads
            wait_for_approval()?;
        }
        ConstraintType::SafetyGuard => {
            // Check the specific guard
            if constraint.value.contains("staging_test") {
                run_staging_tests()?;
            }
        }
        ConstraintType::ResourceLimit => {
            // Check current usage
            if current_usage() > parse_limit(&constraint.value) {
                return Err("Resource limit exceeded");
            }
        }
    }
}
```

### 3. Log All Actions to Audit Trail

Every action OpenClaw takes should be logged:

```rust
use context_server_rs::models::{AuditEventType, AuditTrail};

let audit = AuditTrail::new(
    event_type: AuditEventType::Created,
    entity_type: "deployment".to_string(),
    entity_id: format!("deploy_{}", timestamp),
    initiator: "openclaw".to_string(),
    change_summary: "Deployed service-v2.0 to production".to_string(),
)
.with_project_id("project_001".to_string())
.with_metadata(json!({
    "version": "2.0",
    "environment": "production",
    "duration_seconds": 45,
    "status": "success"
}));

context_server.log_event(&audit)?;
```

### 4. Analyze Impact with Dependency Chains

Before making changes, query dependencies for impact analysis:

```rust
// Check what depends on the database
let dependents = context_server.get_dependents_of(
    project_id: "project_001",
    component: "database"
)?;

for dependent in dependents {
    println!(
        "WARNING: {} depends on database ({})", 
        dependent.source_component, 
        dependent.criticality
    );
    
    if dependent.criticality == "critical" {
        // Need extra safety checks for critical components
        require_extended_testing = true;
    }
}
```

### 5. Query Context for Decision Making

Use the query system to get relevant context for decisions:

```
Example queries:
- GET /query_context?area=database&type=backup_procedures
- GET /query_context?area=deployment&type=rollback_procedures
- GET /query_context?area=security&type=authentication_rules
```

## Recommended OpenClaw Integration Wrapper

Create a wrapper module in OpenClaw to abstract context server interactions:

```rust
pub struct ContextServerClient {
    base_url: String,
}

impl ContextServerClient {
    pub async fn can_execute_action(
        &self,
        project_id: &str,
        action: &str,
        target: &str,
    ) -> Result<bool, Box<dyn Error>> {
        // Check if action is allowed given constraints
        let constraints = self.get_constraints(project_id, target).await?;
        
        for constraint in constraints {
            if constraint.enforcement_action == "BLOCK_OPERATION" {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    pub async fn get_impact_analysis(
        &self,
        project_id: &str,
        component: &str,
    ) -> Result<ImpactReport, Box<dyn Error>> {
        let deps = self.get_dependencies(project_id, component).await?;
        
        let critical_count = deps.iter().filter(|d| d.criticality == "critical").count();
        let high_count = deps.iter().filter(|d| d.criticality == "high").count();
        
        Ok(ImpactReport {
            total_affected: deps.len(),
            critical_components: critical_count,
            high_priority_components: high_count,
            details: deps,
        })
    }

    pub async fn log_action(
        &self,
        project_id: &str,
        action: &str,
        result: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), Box<dyn Error>> {
        // Log to audit trail
        let audit = AuditTrail::new(
            event_type: AuditEventType::Created,
            entity_type: "openclaw_action".to_string(),
            entity_id: uuid::Uuid::new_v4().to_string(),
            initiator: "openclaw".to_string(),
            change_summary: format!("{}: {}", action, result),
        )
        .with_project_id(project_id.to_string())
        .with_metadata(metadata.unwrap_or_default());

        self.log_event(&audit).await?;
        Ok(())
    }
}
```

## End-to-End Workflow Example

```
OpenClaw: "I want to deploy service v2.0 to production"
Context Server: Check deployment constraints...

1. Query Constraints:
   - Staging test required? ✓
   - Approved by team leads? ✗ (Need approval)
   - Maintenance window OK? ✓

OpenClaw: Wait for approval from team leads...
Team Lead: "Approved"

2. Query Dependencies:
   - Database (critical) ← API depends on this
   - Redis (high) ← Cache dependency
   - Auth Service (critical) ← Auth dependency
   
OpenClaw: "High impact deployment detected"

3. Pre-Deployment Checks:
   - Run tests ✓
   - Verify backups ready ✓
   - Prepare rollback ✓

4. Deploy:
   - Deploy v2.0 ✓
   - Log action to audit trail ✓

5. Monitor & Report:
   - Health checks passing ✓
   - Performance metrics normal ✓
   - Log completion to audit trail ✓

OpenClaw: "Deployment completed successfully at 2024-02-04T15:30:00Z"
Context Server: Audit trail updated with complete action history
```

## Configuration Recommendations

### Minimal Safe Configuration
```sql
-- Basic safety constraints
- Staging test required for all deployments
- Manual approval for production
- Database backup before schema changes
```

### Standard Configuration
```sql
-- Add constraint types
- Resource limits (connections, memory, CPU)
- Security requirements (TLS, authentication)
- Performance targets (response time, availability)
- Audit and monitoring requirements
```

### Advanced Configuration
```sql
-- Full governance setup
- Complex approval workflows
- Progressive rollout policies
- Automatic rollback triggers
- Integration with external systems
```

## Testing the Integration

### 1. Unit Test
```bash
cargo test --test openclaw_integration_test
```

### 2. Load Example Context
```bash
cargo run --release &
sqlite3 ~/.config/context-server-rs/context.db < examples/openclaw_constraints.sql
sqlite3 ~/.config/context-server-rs/context.db < examples/openclaw_dependencies.sql
```

### 3. Manual Query Test
```bash
# List all constraints
curl http://localhost:3000/list_entities?entity_type=constraint

# Get dependencies for a component
curl http://localhost:3000/get_dependencies?component=database

# Check audit trail
curl http://localhost:3000/list_entities?entity_type=audit_trail
```

### 4. Simulate OpenClaw Workflow
```bash
# Start server
cargo run --release &

# Load context
sqlite3 ~/.config/context-server-rs/context.db < examples/openclaw_constraints.sql

# Query constraints before action
curl http://localhost:3000/query_context?area=deployment

# Log an action
curl -X POST http://localhost:3000/create_entity \
  -H "Content-Type: application/json" \
  -d '{"entity_type": "audit_trail", "initiator": "openclaw", "action": "deployment", "status": "success"}'

# Verify audit trail
curl http://localhost:3000/list_entities?entity_type=audit_trail
```

## Monitoring & Observability

The context server provides built-in observability:

1. **Audit Trails**: Every action is logged with full state transitions
2. **Query Patterns**: See what OpenClaw most frequently queries
3. **Constraint Violations**: Track when constraints are breached
4. **Performance Metrics**: Monitor query response times

Query audit trail for OpenClaw actions:
```sql
SELECT * FROM audit_trails WHERE initiator = 'openclaw' ORDER BY timestamp DESC LIMIT 50;
```

## Troubleshooting

### OpenClaw can't find the server
- Verify server is running: `ps aux | grep context-server`
- Check configuration: `cat ~/.config/context-server-rs/config.toml`
- Test connectivity: `curl http://localhost:3000/health`

### Constraints not being enforced
- Verify constraints are loaded: `curl http://localhost:3000/list_entities?entity_type=constraint`
- Check `enabled` field is true
- Verify constraint matches the target action

### Audit trail not recording actions
- Ensure OpenClaw is calling `create_entity` for audit events
- Check timestamps are in ISO 8601 format
- Verify project_id matches existing projects

## Next Steps

1. Deploy context server in your environment
2. Load organizational constraints and policies
3. Configure OpenClaw to query context server on startup
4. Test with staging deployments
5. Monitor audit trails for compliance
6. Iterate and refine constraints based on learnings

---

For more information, see [OPENCLAW_CONTEXT_README.md](OPENCLAW_CONTEXT_README.md)
