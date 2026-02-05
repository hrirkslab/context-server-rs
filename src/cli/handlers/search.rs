/// Search command handler - Full-text search across all contexts
/// Single Responsibility: Handle search queries only
use anyhow::Result;
use serde_json::{json, Value};
use rusqlite::Connection;
use crate::cli::commands::CliCommand;

pub struct SearchCommand {
    pub db_path: String,
    pub query: String,
    pub project: Option<String>,
}

impl SearchCommand {
    pub fn new(db_path: String, query: String, project: Option<String>) -> Self {
        Self { db_path, query, project }
    }

    /// Search across all context types
    fn search_contexts(&self) -> Result<Value> {
        let conn = Connection::open(&self.db_path)?;
        let project = self.project.as_deref().unwrap_or("default");
        let search_pattern = format!("%{}%", self.query);

        // Search in business rules
        let mut stmt = conn.prepare(
            "SELECT id, rule_name, description, 'business_rule' as type
             FROM business_rules
             WHERE (project_id = ? OR project_id IS NULL)
             AND (rule_name LIKE ? OR description LIKE ?)
             ORDER BY created_at DESC LIMIT 20"
        )?;

        let business_rules = stmt.query_map(
            [project, search_pattern.as_str(), search_pattern.as_str()],
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "description": row.get::<_, String>(2)?,
                    "type": row.get::<_, String>(3)?,
                }))
            }
        )?
        .collect::<Result<Vec<_>, _>>()?;

        // Search in architectural decisions
        let mut stmt = conn.prepare(
            "SELECT id, decision_title, context, 'architectural_decision' as type
             FROM architectural_decisions
             WHERE (project_id = ? OR project_id IS NULL)
             AND (decision_title LIKE ? OR context LIKE ?)
             ORDER BY created_at DESC LIMIT 20"
        )?;

        let arch_decisions = stmt.query_map(
            [project, search_pattern.as_str(), search_pattern.as_str()],
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "description": row.get::<_, String>(2)?,
                    "type": row.get::<_, String>(3)?,
                }))
            }
        )?
        .collect::<Result<Vec<_>, _>>()?;

        // Search in security policies
        let mut stmt = conn.prepare(
            "SELECT id, policy_name, requirements, 'security_policy' as type
             FROM security_policies
             WHERE (project_id = ? OR project_id IS NULL)
             AND (policy_name LIKE ? OR requirements LIKE ?)
             ORDER BY created_at DESC LIMIT 20"
        )?;

        let security = stmt.query_map(
            [project, search_pattern.as_str(), search_pattern.as_str()],
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "description": row.get::<_, String>(2)?,
                    "type": row.get::<_, String>(3)?,
                }))
            }
        )?
        .collect::<Result<Vec<_>, _>>()?;

        let mut all_results = business_rules;
        all_results.extend(arch_decisions);
        all_results.extend(security);

        Ok(json!({
            "status": "success",
            "query": self.query,
            "count": all_results.len(),
            "data": all_results
        }))
    }
}

impl CliCommand for SearchCommand {
    fn execute(&self) -> Result<Value> {
        self.search_contexts()
    }
}
