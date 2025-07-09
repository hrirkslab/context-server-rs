use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use rusqlite::Connection;
use crate::models::context::BusinessRule;
use crate::repositories::BusinessRuleRepository;
use rmcp::model::ErrorData as McpError;

/// SQLite implementation of BusinessRuleRepository
pub struct SqliteBusinessRuleRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteBusinessRuleRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BusinessRuleRepository for SqliteBusinessRuleRepository {
    async fn create(&self, rule: &BusinessRule) -> Result<BusinessRule, McpError> {
        let db = self.db.lock().unwrap();
        
        db.execute(
            "INSERT INTO business_rules (id, project_id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                &rule.id,
                &rule.project_id,
                &rule.rule_name,
                &rule.description,
                rule.domain_area.as_deref(),
                rule.implementation_pattern.as_deref(),
                rule.constraints.as_deref(),
                rule.examples.as_deref(),
                rule.created_at.as_deref(),
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(rule.clone())
    }

    async fn find_by_project_id(&self, project_id: &str) -> Result<Vec<BusinessRule>, McpError> {
        let db = self.db.lock().unwrap();
        let mut rules = Vec::new();
        
        let mut stmt = db.prepare("SELECT id, project_id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at FROM business_rules WHERE project_id = ?")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        
        let rule_rows = stmt.query_map([project_id], |row| {
            Ok(BusinessRule {
                id: row.get(0)?,
                project_id: row.get(1)?,
                rule_name: row.get(2)?,
                description: row.get(3)?,
                domain_area: row.get(4)?,
                implementation_pattern: row.get(5)?,
                constraints: row.get(6)?,
                examples: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for rule in rule_rows {
            match rule {
                Ok(rule) => rules.push(rule),
                Err(e) => tracing::warn!("Failed to parse business rule: {}", e),
            }
        }

        Ok(rules)
    }

    async fn find_by_domain_area(&self, project_id: &str, domain_area: &str) -> Result<Vec<BusinessRule>, McpError> {
        let db = self.db.lock().unwrap();
        let mut rules = Vec::new();
        
        let mut stmt = db.prepare("SELECT id, project_id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at FROM business_rules WHERE project_id = ? AND (domain_area = ? OR domain_area IS NULL)")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        
        let rule_rows = stmt.query_map([project_id, domain_area], |row| {
            Ok(BusinessRule {
                id: row.get(0)?,
                project_id: row.get(1)?,
                rule_name: row.get(2)?,
                description: row.get(3)?,
                domain_area: row.get(4)?,
                implementation_pattern: row.get(5)?,
                constraints: row.get(6)?,
                examples: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for rule in rule_rows {
            match rule {
                Ok(rule) => rules.push(rule),
                Err(e) => tracing::warn!("Failed to parse business rule: {}", e),
            }
        }

        Ok(rules)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<BusinessRule>, McpError> {
        let db = self.db.lock().unwrap();
        
        let mut stmt = db.prepare("SELECT id, project_id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at FROM business_rules WHERE id = ?")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        
        let mut rule_iter = stmt.query_map([id], |row| {
            Ok(BusinessRule {
                id: row.get(0)?,
                project_id: row.get(1)?,
                rule_name: row.get(2)?,
                description: row.get(3)?,
                domain_area: row.get(4)?,
                implementation_pattern: row.get(5)?,
                constraints: row.get(6)?,
                examples: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        match rule_iter.next() {
            Some(Ok(rule)) => Ok(Some(rule)),
            Some(Err(e)) => Err(McpError::internal_error(format!("Database error: {}", e), None)),
            None => Ok(None),
        }
    }

    async fn update(&self, rule: &BusinessRule) -> Result<BusinessRule, McpError> {
        let db = self.db.lock().unwrap();
        
        db.execute(
            "UPDATE business_rules SET project_id = ?, rule_name = ?, description = ?, domain_area = ?, implementation_pattern = ?, constraints = ?, examples = ? WHERE id = ?",
            (
                &rule.project_id,
                &rule.rule_name,
                &rule.description,
                rule.domain_area.as_deref(),
                rule.implementation_pattern.as_deref(),
                rule.constraints.as_deref(),
                rule.examples.as_deref(),
                &rule.id,
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(rule.clone())
    }

    async fn delete(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().unwrap();
        
        let rows_affected = db.execute("DELETE FROM business_rules WHERE id = ?", [id])
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(rows_affected > 0)
    }
}
