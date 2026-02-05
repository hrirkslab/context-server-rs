/// Query command handler - Search contexts by task/project
/// Single Responsibility: Handle task-based queries only
use anyhow::Result;
use serde_json::{json, Value};
use rusqlite::Connection;
use crate::cli::commands::CliCommand;

pub struct QueryCommand {
    pub db_path: String,
    pub task: Option<String>,
    pub project: Option<String>,
}

impl QueryCommand {
    pub fn new(db_path: String, task: Option<String>, project: Option<String>) -> Self {
        Self { db_path, task, project }
    }

    /// Query all relevant contexts for a task in a project
    fn query_contexts(&self) -> Result<Value> {
        let conn = Connection::open(&self.db_path)?;

        let mut results = json!({
            "status": "success",
            "data": {
                "business_rules": [],
                "architectural_decisions": [],
                "performance_requirements": [],
                "security_policies": [],
                "features": [],
            }
        });

        // Query business rules for this project/task
        let mut stmt = conn.prepare(
            "SELECT id, name, description, domain_area FROM business_rules
             WHERE project_id = ? OR project_id IS NULL
             ORDER BY created_at DESC"
        )?;

        let project = self.project.as_deref().unwrap_or("default");
        let business_rules = stmt.query_map([project], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "description": row.get::<_, String>(2)?,
                "domain": row.get::<_, String>(3)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        // Query architectural decisions
        let mut stmt = conn.prepare(
            "SELECT id, decision_title, context FROM architectural_decisions
             WHERE project_id = ? OR project_id IS NULL
             ORDER BY created_at DESC"
        )?;

        let arch_decisions = stmt.query_map([project], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "context": row.get::<_, String>(2)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        // Query performance requirements
        let mut stmt = conn.prepare(
            "SELECT id, component_area, requirement_type FROM performance_requirements
             WHERE project_id = ? OR project_id IS NULL
             ORDER BY created_at DESC"
        )?;

        let performance_reqs = stmt.query_map([project], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "component": row.get::<_, String>(1)?,
                "requirement_type": row.get::<_, String>(2)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        // Query security policies
        let mut stmt = conn.prepare(
            "SELECT id, policy_name, policy_area FROM security_policies
             WHERE project_id = ? OR project_id IS NULL
             ORDER BY created_at DESC"
        )?;

        let security_policies = stmt.query_map([project], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "area": row.get::<_, String>(2)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        if let Some(data) = results.get_mut("data").and_then(|d| d.as_object_mut()) {
            data.insert("business_rules".to_string(), Value::Array(business_rules));
            data.insert("architectural_decisions".to_string(), Value::Array(arch_decisions));
            data.insert("performance_requirements".to_string(), Value::Array(performance_reqs));
            data.insert("security_policies".to_string(), Value::Array(security_policies));
        }

        Ok(results)
    }
}

impl CliCommand for QueryCommand {
    fn execute(&self) -> Result<Value> {
        self.query_contexts()
    }
}
