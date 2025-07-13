use crate::models::context::PerformanceRequirement;
use crate::repositories::PerformanceRequirementRepository;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// SQLite implementation of PerformanceRequirementRepository
pub struct SqlitePerformanceRequirementRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqlitePerformanceRequirementRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PerformanceRequirementRepository for SqlitePerformanceRequirementRepository {
    async fn create(
        &self,
        requirement: &PerformanceRequirement,
    ) -> Result<PerformanceRequirement, McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT INTO performance_requirements (id, project_id, component_area, requirement_type, target_value, optimization_patterns, avoid_patterns, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            (
                &requirement.id,
                &requirement.project_id,
                requirement.component_area.as_deref(),
                requirement.requirement_type.as_deref(),
                requirement.target_value.as_deref(),
                requirement.optimization_patterns.as_deref(),
                requirement.avoid_patterns.as_deref(),
                requirement.created_at.as_deref(),
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(requirement.clone())
    }

    async fn find_by_project_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<PerformanceRequirement>, McpError> {
        let db = self.db.lock().unwrap();
        let mut requirements = Vec::new();

        let mut stmt = db.prepare("SELECT id, project_id, component_area, requirement_type, target_value, optimization_patterns, avoid_patterns, created_at FROM performance_requirements WHERE project_id = ?")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let requirement_rows = stmt
            .query_map([project_id], |row| {
                Ok(PerformanceRequirement {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    component_area: row.get(2)?,
                    requirement_type: row.get(3)?,
                    target_value: row.get(4)?,
                    optimization_patterns: row.get(5)?,
                    avoid_patterns: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for requirement in requirement_rows {
            match requirement {
                Ok(requirement) => requirements.push(requirement),
                Err(e) => tracing::warn!("Failed to parse performance requirement: {}", e),
            }
        }

        Ok(requirements)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<PerformanceRequirement>, McpError> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare("SELECT id, project_id, component_area, requirement_type, target_value, optimization_patterns, avoid_patterns, created_at FROM performance_requirements WHERE id = ?")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let mut requirement_iter = stmt
            .query_map([id], |row| {
                Ok(PerformanceRequirement {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    component_area: row.get(2)?,
                    requirement_type: row.get(3)?,
                    target_value: row.get(4)?,
                    optimization_patterns: row.get(5)?,
                    avoid_patterns: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        match requirement_iter.next() {
            Some(Ok(requirement)) => Ok(Some(requirement)),
            Some(Err(e)) => Err(McpError::internal_error(
                format!("Database error: {}", e),
                None,
            )),
            None => Ok(None),
        }
    }

    async fn update(
        &self,
        requirement: &PerformanceRequirement,
    ) -> Result<PerformanceRequirement, McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "UPDATE performance_requirements SET project_id = ?, component_area = ?, requirement_type = ?, target_value = ?, optimization_patterns = ?, avoid_patterns = ? WHERE id = ?",
            (
                &requirement.project_id,
                requirement.component_area.as_deref(),
                requirement.requirement_type.as_deref(),
                requirement.target_value.as_deref(),
                requirement.optimization_patterns.as_deref(),
                requirement.avoid_patterns.as_deref(),
                &requirement.id,
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(requirement.clone())
    }

    async fn delete(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().unwrap();

        let rows_affected = db
            .execute("DELETE FROM performance_requirements WHERE id = ?", [id])
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(rows_affected > 0)
    }
}
