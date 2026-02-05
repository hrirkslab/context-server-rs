/// List command handler - List all contexts by type
/// Single Responsibility: Handle type-based listing only
use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use rusqlite::Connection;
use crate::cli::commands::CliCommand;

pub struct ListCommand {
    pub db_path: String,
    pub entity_type: String,
    pub project: Option<String>,
}

impl ListCommand {
    pub fn new(db_path: String, entity_type: String, project: Option<String>) -> Self {
        Self { db_path, entity_type, project }
    }

    /// List entities by type
    fn list_entities(&self) -> Result<Value> {
        let conn = Connection::open(&self.db_path)?;
        let project = self.project.as_deref().unwrap_or("default");

        let (table, id_col, name_col) = match self.entity_type.to_lowercase().as_str() {
            "business_rule" => ("business_rules", "id", "rule_name"),
            "architectural_decision" => ("architectural_decisions", "id", "decision_title"),
            "performance_requirement" => ("performance_requirements", "id", "component_area"),
            "security_policy" => ("security_policies", "id", "policy_name"),
            "feature" => ("features", "id", "feature_name"),
            _ => return Err(anyhow!("Unknown entity type: {}", self.entity_type)),
        };

        let query = format!(
            "SELECT {id}, {name} FROM {table} WHERE project_id = ? OR project_id IS NULL ORDER BY created_at DESC",
            id = id_col, name = name_col, table = table
        );

        let mut stmt = conn.prepare(&query)?;
        let entities = stmt.query_map([project], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(json!({
            "status": "success",
            "entity_type": self.entity_type,
            "count": entities.len(),
            "data": entities
        }))
    }
}

impl CliCommand for ListCommand {
    fn execute(&self) -> Result<Value> {
        self.list_entities()
    }
}
