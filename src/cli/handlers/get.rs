/// Get command handler - Retrieve a specific context by ID
/// Single Responsibility: Handle entity retrieval by ID only
use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use rusqlite::Connection;
use crate::cli::commands::CliCommand;

pub struct GetCommand {
    pub db_path: String,
    pub id: String,
}

impl GetCommand {
    pub fn new(db_path: String, id: String) -> Self {
        Self { db_path, id }
    }

    /// Get a single entity by ID
    fn get_entity(&self) -> Result<Value> {
        let conn = Connection::open(&self.db_path)?;

        // Try each table until we find it
        let tables = vec![
            "business_rules",
            "architectural_decisions",
            "performance_requirements",
            "security_policies",
            "features",
        ];

        for table in tables {
            let mut stmt = match conn.prepare(&format!("SELECT * FROM {} WHERE id = ? LIMIT 1", table)) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let result: Result<Vec<_>> = stmt.query_map([&self.id], |row| {
                let mut obj = json!({});
                let cols = row.as_ref().column_count();
                
                // Get column names dynamically
                for i in 0..cols {
                    let col_name = row.as_ref().column_name(i).unwrap_or("unknown");
                    if let Ok(val) = row.get::<_, String>(i) {
                        if let Some(obj_mut) = obj.as_object_mut() {
                            obj_mut.insert(col_name.to_string(), Value::String(val));
                        }
                    }
                }
                Ok(obj)
            })?
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
            .map_err(|e| anyhow!("Failed to collect results: {}", e));

            if let Ok(mut data) = result {
                if !data.is_empty() {
                    return Ok(json!({
                        "status": "success",
                        "entity_type": table.trim_end_matches('s'),
                        "data": data.pop().unwrap_or(json!({}))
                    }));
                }
            }
        }

        Err(anyhow!("Entity with id '{}' not found", self.id))
    }
}

impl CliCommand for GetCommand {
    fn execute(&self) -> Result<Value> {
        self.get_entity()
    }
}
